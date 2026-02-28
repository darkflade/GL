use crate::domain::model::{
    File, KeysetCursor, NextKeysetCursor, PaginationMode, Playlist, PlaylistContent, PlaylistID,
    PlaylistItem, PlaylistQuery, PlaylistSummary, Post, RepoError, SearchPlaylistsResponse, Tag,
    UserID,
};
use crate::domain::repository::PlaylistRepository;
use crate::storage::postgres::dto::TagResponse;
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::types::Json;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresPlaylistRepository {
    pool: PgPool,
}

impl PostgresPlaylistRepository {
    const DEFAULT_KEYSET_LIMIT: i64 = 30;
    const MAX_KEYSET_LIMIT: i64 = 100;

    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn resolve_keyset_limit(cursor: &KeysetCursor) -> i64 {
        cursor
            .limit
            .unwrap_or(Self::DEFAULT_KEYSET_LIMIT)
            .clamp(1, Self::MAX_KEYSET_LIMIT)
    }

    fn build_keyset_response(
        mut entries: Vec<(PlaylistSummary, f64)>,
        limit: i64,
    ) -> SearchPlaylistsResponse {
        let has_next = entries.len() as i64 > limit;
        if has_next {
            entries.truncate(limit as usize);
        }

        let next_cursor = if has_next {
            entries.last().map(|(playlist, score)| NextKeysetCursor {
                mode: PaginationMode::Keyset,
                last_id: playlist.id,
                last_score: *score,
                limit,
            })
        } else {
            None
        };

        SearchPlaylistsResponse {
            playlists: entries.into_iter().map(|(playlist, _)| playlist).collect(),
            has_next,
            next_cursor,
        }
    }
}

#[async_trait]
impl PlaylistRepository for PostgresPlaylistRepository {
    async fn get(&self, user_id: UserID, playlist_id: PlaylistID) -> Result<Playlist, RepoError> {
        let playlist_row = sqlx::query!(
            r#"
            SELECT
                p.id,
                p.title,
                COALESCE(p.description, '') AS "description!",
                p.cover_file_id AS cover
            FROM playlists p
            WHERE p.id = $1 AND p.owner_id = $2
            "#,
            playlist_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| {
            log::error!(
                "playlists.get failed to fetch playlist {}: {err}",
                playlist_id
            );
            RepoError::StorageError
        })?;

        let playlist_row = playlist_row.ok_or(RepoError::NotFound)?;

        let tags_rows = sqlx::query!(
            r#"
            SELECT
                t.id,
                t.name,
                t.category AS "category!: i16",
                t.post_count AS "count!: i16"
            FROM playlist_tags pt
            JOIN tags t ON t.id = pt.tag_id
            WHERE pt.playlist_id = $1
            ORDER BY t.name ASC
            "#,
            playlist_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| {
            log::error!(
                "playlists.get failed to fetch tags for {}: {err}",
                playlist_id
            );
            RepoError::StorageError
        })?;

        let items_rows = sqlx::query!(
            r#"
            SELECT
                pi.id as item_id,
                pi.position,
                pi.post_id,
                pi.note_text,

                -- Post Data
                p.title as post_title,
                p.description as post_desc,

                -- File Data
                f.id as file_id,
                f.path as file_path,
                f.hash as file_hash,
                f.media_type as "media_type: i16",
                f.meta as file_meta,
                f.created_at as created_at

            FROM playlist_items pi
            LEFT JOIN posts p ON pi.post_id = p.id
            LEFT JOIN files f ON p.file_id = f.id
            WHERE pi.playlist_id = $1
            ORDER BY pi.position ASC
            "#,
            playlist_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| {
            log::error!(
                "playlists.get failed to fetch items for {}: {err}",
                playlist_id
            );
            RepoError::StorageError
        })?;

        let items = items_rows
            .into_iter()
            .map(|row| {
                let content = if let Some(p_id) = row.post_id {
                    let file_obj = File {
                        id: row.file_id,
                        path: PathBuf::from(row.file_path),
                        hash: row.file_hash,
                        media_type: row.media_type.into(),
                        meta: row
                            .file_meta
                            .and_then(|meta| serde_json::from_value(meta).ok()),
                        created_at: row.created_at,
                        thumbnail: None,
                    };
                    PlaylistContent::Post(Post {
                        id: p_id,
                        title: row.post_title,
                        description: row.post_desc,
                        //TODO Decide Tags
                        tags: vec![],
                        file: file_obj,
                        //TODO Playlist Notes
                        notes: vec![],
                    })
                } else {
                    PlaylistContent::Note(row.note_text.unwrap_or_default())
                };

                PlaylistItem {
                    id: row.item_id,
                    position: row.position as u32,
                    content,
                }
            })
            .collect();

        Ok(Playlist {
            id: playlist_id,
            title: playlist_row.title,
            description: playlist_row.description,
            tags: tags_rows
                .into_iter()
                .map(|row| Tag {
                    id: row.id,
                    name: row.name,
                    category: row.category.into(),
                    count: row.count as i32,
                })
                .collect(),
            cover: playlist_row.cover,
            items,
        })
    }

    async fn search(
        &self,
        user_id: UserID,
        query: PlaylistQuery,
        cursor: KeysetCursor,
    ) -> Result<SearchPlaylistsResponse, RepoError> {
        log::debug!("playlists.search user={user_id} query={query:?} cursor={cursor:?}");

        let limit = Self::resolve_keyset_limit(&cursor);
        let query_limit = limit + 1;

        let use_cursor = cursor.last_id.is_some() && cursor.last_score.is_some();
        let last_id = cursor.last_id.unwrap_or_else(Uuid::nil);
        let last_score = cursor.last_score.unwrap_or(f64::MAX);

        let text = query.text.trim();
        let use_text_filter = !text.is_empty();
        let text_pattern = format!("%{text}%");

        let rows = sqlx::query!(
            r#"
            WITH ranked_playlists AS (
                SELECT
                    pl.id,
                    pl.title,
                    COALESCE(pl.description, '') AS description,
                    pl.cover_file_id AS cover,
                    COUNT(DISTINCT pi.id)::bigint AS item_count,
                    COALESCE(
                        jsonb_agg(
                            DISTINCT jsonb_build_object(
                                'id', t.id,
                                'name', t.name,
                                'category', t.category,
                                'count', t.post_count
                            )
                        ) FILTER (WHERE t.id IS NOT NULL),
                        '[]'::jsonb
                    ) AS tags,
                    COUNT(DISTINCT CASE
                        WHEN t.name = ANY($1) THEN t.name
                    END)::bigint AS should_score
                FROM playlists pl
                LEFT JOIN playlist_items pi ON pi.playlist_id = pl.id
                LEFT JOIN playlist_tags plt ON plt.playlist_id = pl.id
                LEFT JOIN tags t ON t.id = plt.tag_id
                WHERE
                    pl.owner_id = $2
                    AND (
                        $3 = false
                        OR pl.title ILIKE $4
                        OR COALESCE(pl.description, '') ILIKE $4
                    )
                GROUP BY pl.id
                HAVING
                    COUNT(DISTINCT CASE
                        WHEN t.name = ANY($5) THEN t.name
                    END) = cardinality($5)
                    AND
                    NOT EXISTS (
                        SELECT 1
                        FROM playlist_tags x
                        JOIN tags tx ON tx.id = x.tag_id
                        WHERE x.playlist_id = pl.id
                          AND tx.name = ANY($6)
                    )
            )
            SELECT
                id,
                title,
                description AS "description!",
                cover,
                item_count AS "item_count!: i64",
                tags AS "tags!: Json<Vec<TagResponse>>",
                should_score AS "should_score!: i64"
            FROM ranked_playlists
            WHERE
                $7 = false
                OR should_score::double precision < $8
                OR (should_score::double precision = $8 AND id < $9)
            ORDER BY should_score DESC, id DESC
            LIMIT $10
            "#,
            &query.tags.should[..],
            user_id,
            use_text_filter,
            text_pattern,
            &query.tags.must[..],
            &query.tags.must_not[..],
            use_cursor,
            last_score,
            last_id,
            query_limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| {
            log::error!("playlists.search db query failed: {err}");
            RepoError::StorageError
        })?;

        let parsed_rows = rows
            .into_iter()
            .map(|row| {
                (
                    PlaylistSummary {
                        id: row.id,
                        title: row.title,
                        description: row.description,
                        cover: row.cover,
                        item_count: row.item_count,
                        tags: row.tags.0.into_iter().map(Tag::from).collect(),
                    },
                    row.should_score as f64,
                )
            })
            .collect();

        Ok(Self::build_keyset_response(parsed_rows, limit))
    }

    async fn get_all(
        &self,
        user_id: UserID,
        cursor: KeysetCursor,
    ) -> Result<SearchPlaylistsResponse, RepoError> {
        log::debug!("playlists.get_all user={user_id} cursor={cursor:?}");

        let limit = Self::resolve_keyset_limit(&cursor);
        let query_limit = limit + 1;
        let use_cursor = cursor.last_id.is_some();
        let last_id = cursor.last_id.unwrap_or_else(Uuid::nil);

        let rows = sqlx::query!(
            r#"
            SELECT
                pl.id,
                pl.title,
                COALESCE(pl.description, '') AS "description!",
                pl.cover_file_id AS cover,
                COUNT(DISTINCT pi.id)::bigint AS "item_count!: i64",
                COALESCE(
                    jsonb_agg(
                        DISTINCT jsonb_build_object(
                            'id', t.id,
                            'name', t.name,
                            'category', t.category,
                            'count', t.post_count
                        )
                    ) FILTER (WHERE t.id IS NOT NULL),
                    '[]'::jsonb
                ) AS "tags!: Json<Vec<TagResponse>>",
                0::bigint AS "should_score!: i64"
            FROM playlists pl
            LEFT JOIN playlist_items pi ON pi.playlist_id = pl.id
            LEFT JOIN playlist_tags plt ON plt.playlist_id = pl.id
            LEFT JOIN tags t ON t.id = plt.tag_id
            WHERE
                pl.owner_id = $1
                AND ($2 = false OR pl.id < $3)
            GROUP BY pl.id
            ORDER BY pl.id DESC
            LIMIT $4
            "#,
            user_id,
            use_cursor,
            last_id,
            query_limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| {
            log::error!("playlists.get_all db query failed: {err}");
            RepoError::StorageError
        })?;

        let parsed_rows = rows
            .into_iter()
            .map(|row| {
                (
                    PlaylistSummary {
                        id: row.id,
                        title: row.title,
                        description: row.description,
                        cover: row.cover,
                        item_count: row.item_count,
                        tags: row.tags.0.into_iter().map(Tag::from).collect(),
                    },
                    row.should_score as f64,
                )
            })
            .collect();

        Ok(Self::build_keyset_response(parsed_rows, limit))
    }
}
