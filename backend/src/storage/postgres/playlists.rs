use crate::application::contracts::{
    KeysetCursor, KeysetDirection, KeysetPageCursor, NewPlaylist, NewPlaylistItemContent,
    PaginationMode, PlaylistQuery, SearchPlaylistsResponse, UpdatePlaylist,
};
use crate::application::ports::PlaylistRepository;
use crate::domain::model::{
    Playlist, PlaylistContent, PlaylistID, PlaylistItem, PlaylistSummary, Post, RepoError, Tag,
    UserID,
};
use crate::storage::postgres::dto::{FileResponse, TagResponse};
use async_trait::async_trait;
use serde::Deserialize;
use sqlx::PgPool;
use sqlx::types::Json;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresPlaylistRepository {
    pool: PgPool,
}

#[derive(Debug, Deserialize)]
struct PlaylistItemPayload {
    id: Uuid,
    position: i32,
    post: Option<PlaylistPostPayload>,
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PlaylistPostPayload {
    id: Uuid,
    title: String,
    description: Option<String>,
    file: FileResponse,
    tags: Vec<TagResponse>,
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
        direction: KeysetDirection,
        use_cursor: bool,
    ) -> SearchPlaylistsResponse {
        let has_more_in_direction = entries.len() as i64 > limit;
        if has_more_in_direction {
            entries.truncate(limit as usize);
        }

        if matches!(direction, KeysetDirection::Prev) {
            entries.reverse();
        }

        let has_next = if matches!(direction, KeysetDirection::Next) {
            has_more_in_direction
        } else {
            use_cursor
        };
        let has_prev = if matches!(direction, KeysetDirection::Prev) {
            has_more_in_direction
        } else {
            use_cursor
        };

        let next_cursor = if has_next {
            entries.last().map(|(playlist, score)| KeysetPageCursor {
                mode: PaginationMode::Keyset,
                direction: KeysetDirection::Next,
                last_id: playlist.id,
                last_score: *score,
                limit,
            })
        } else {
            None
        };

        let prev_cursor = if has_prev {
            entries.first().map(|(playlist, score)| KeysetPageCursor {
                mode: PaginationMode::Keyset,
                direction: KeysetDirection::Prev,
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
            has_prev,
            next_cursor,
            prev_cursor,
        }
    }

    fn map_playlist_items(items: Vec<PlaylistItemPayload>) -> Vec<PlaylistItem> {
        items
            .into_iter()
            .map(|item| {
                let content = match item.post {
                    Some(post) => PlaylistContent::Post(Post {
                        id: post.id,
                        title: post.title,
                        description: post.description,
                        tags: post.tags.into_iter().map(Tag::from).collect(),
                        file: post.file.into(),
                        //TODO load notes
                        notes: vec![],
                    }),
                    None => PlaylistContent::Note(item.note.unwrap_or_default()),
                };

                PlaylistItem {
                    id: item.id,
                    position: item.position.max(0) as u32,
                    content,
                }
            })
            .collect()
    }
}

#[async_trait]
impl PlaylistRepository for PostgresPlaylistRepository {
    async fn create(
        &self,
        user_id: UserID,
        new_playlist: NewPlaylist,
    ) -> Result<PlaylistID, RepoError> {
        let mut tx = self.pool.begin().await.map_err(|err| {
            log::error!("playlists.create failed to begin transaction: {err}");
            RepoError::StorageError
        })?;

        let playlist_id = Uuid::now_v7();

        sqlx::query!(
            r#"
            INSERT INTO playlists (id, title, description, cover_file_id, owner_id)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            playlist_id,
            new_playlist.title,
            new_playlist.description,
            new_playlist.cover,
            user_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|err| {
            log::error!(
                "playlists.create failed to insert playlist {} for {}: {err}",
                playlist_id,
                user_id
            );
            RepoError::StorageError
        })?;

        if let Some(tag_ids) = new_playlist.tag_ids {
            for tag_id in tag_ids {
                sqlx::query!(
                    "INSERT INTO playlist_tags (playlist_id, tag_id) VALUES ($1, $2)",
                    playlist_id,
                    tag_id
                )
                .execute(&mut *tx)
                .await
                .map_err(|err| {
                    log::error!(
                        "playlists.create failed to attach tag {} to {}: {err}",
                        tag_id,
                        playlist_id
                    );
                    RepoError::StorageError
                })?;
            }
        }

        if let Some(items) = new_playlist.items {
            for item in items {
                let position = i32::try_from(item.position).map_err(|err| {
                    log::error!(
                        "playlists.create invalid item position {} for {}: {err}",
                        item.position,
                        playlist_id
                    );
                    RepoError::StorageError
                })?;
                let item_id = Uuid::now_v7();

                match item.content {
                    NewPlaylistItemContent::Post { post_id } => {
                        sqlx::query!(
                            r#"
                            INSERT INTO playlist_items (id, playlist_id, position, post_id, note_text)
                            VALUES ($1, $2, $3, $4, NULL)
                            "#,
                            item_id,
                            playlist_id,
                            position,
                            post_id
                        )
                        .execute(&mut *tx)
                        .await
                        .map_err(|err| {
                            log::error!(
                                "playlists.create failed to insert post item {} for {}: {err}",
                                item_id,
                                playlist_id
                            );
                            RepoError::StorageError
                        })?;
                    }
                    NewPlaylistItemContent::Note { text } => {
                        sqlx::query!(
                            r#"
                            INSERT INTO playlist_items (id, playlist_id, position, post_id, note_text)
                            VALUES ($1, $2, $3, NULL, $4)
                            "#,
                            item_id,
                            playlist_id,
                            position,
                            text
                        )
                        .execute(&mut *tx)
                        .await
                        .map_err(|err| {
                            log::error!(
                                "playlists.create failed to insert note item {} for {}: {err}",
                                item_id,
                                playlist_id
                            );
                            RepoError::StorageError
                        })?;
                    }
                }
            }
        }

        tx.commit().await.map_err(|err| {
            log::error!(
                "playlists.create failed to commit transaction for {}: {err}",
                playlist_id
            );
            RepoError::StorageError
        })?;

        Ok(playlist_id)
    }

    async fn get(&self, user_id: UserID, playlist_id: PlaylistID) -> Result<Playlist, RepoError> {
        let row = sqlx::query!(
            r#"
            SELECT
                pl.id,
                pl.title,
                pl.description,
                pl.cover_file_id AS cover,
                COALESCE(
                    (
                        SELECT jsonb_agg(
                            jsonb_build_object(
                                'id', t.id,
                                'name', t.name,
                                'category', t.category,
                                'count', t.post_count
                            )
                            ORDER BY t.name
                        )
                        FROM playlist_tags pt
                        JOIN tags t ON t.id = pt.tag_id
                        WHERE pt.playlist_id = pl.id
                    ),
                    '[]'::jsonb
                ) AS "tags!: Json<Vec<TagResponse>>",
                COALESCE(
                    (
                        SELECT jsonb_agg(item ORDER BY (item->>'position')::int)
                        FROM (
                            SELECT jsonb_build_object(
                                'id', pi.id,
                                'position', pi.position,
                                'post',
                                CASE
                                    WHEN pi.post_id IS NULL THEN NULL
                                    ELSE jsonb_build_object(
                                        'id', p.id,
                                        'title', p.title,
                                        'description', p.description,
                                        'file', jsonb_build_object(
                                            'id', f.id,
                                            'path', f.path,
                                            'hash', f.hash,
                                            'media_type', f.media_type,
                                            'meta', f.meta,
                                            'created_at', f.created_at
                                        ),
                                        'tags', COALESCE(
                                            (
                                                SELECT jsonb_agg(
                                                    jsonb_build_object(
                                                        'id', t2.id,
                                                        'name', t2.name,
                                                        'category', t2.category,
                                                        'count', t2.post_count
                                                    )
                                                    ORDER BY t2.name
                                                )
                                                FROM post_tags ptt
                                                JOIN tags t2 ON t2.id = ptt.tag_id
                                                WHERE ptt.post_id = p.id
                                            ),
                                            '[]'::jsonb
                                        )
                                    )
                                END,
                                'note', pi.note_text
                            ) AS item
                            FROM playlist_items pi
                            LEFT JOIN posts p ON p.id = pi.post_id
                            LEFT JOIN files f ON f.id = p.file_id
                            WHERE pi.playlist_id = pl.id
                        ) raw_items
                    ),
                    '[]'::jsonb
                ) AS "items!: Json<Vec<PlaylistItemPayload>>"
            FROM playlists pl
            WHERE pl.id = $1 AND pl.owner_id = $2
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

        let row = row.ok_or(RepoError::NotFound)?;

        Ok(Playlist {
            id: playlist_id,
            title: row.title,
            description: row.description,
            tags: row.tags.0.into_iter().map(Tag::from).collect(),
            cover: row.cover,
            items: Self::map_playlist_items(row.items.0),
        })
    }

    async fn update(
        &self,
        user_id: UserID,
        playlist_id: PlaylistID,
        update_playlist: UpdatePlaylist,
    ) -> Result<(), RepoError> {
        let mut tx = self.pool.begin().await.map_err(|err| {
            log::error!(
                "playlists.update failed to begin transaction for {}: {err}",
                playlist_id
            );
            RepoError::StorageError
        })?;

        let exists = sqlx::query!(
            "SELECT id FROM playlists WHERE id = $1 AND owner_id = $2",
            playlist_id,
            user_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|err| {
            log::error!(
                "playlists.update failed to check playlist {} ownership for {}: {err}",
                playlist_id,
                user_id
            );
            RepoError::StorageError
        })?;

        if exists.is_none() {
            return Err(RepoError::NotFound);
        }

        if update_playlist.title.is_some()
            || update_playlist.description.is_some()
            || update_playlist.cover.is_some()
        {
            sqlx::query!(
                r#"
                UPDATE playlists
                SET
                    title = COALESCE($3, title),
                    description = COALESCE($4, description),
                    cover_file_id = COALESCE($5, cover_file_id),
                    updated_at = NOW()
                WHERE id = $1 AND owner_id = $2
                "#,
                playlist_id,
                user_id,
                update_playlist.title,
                update_playlist.description,
                update_playlist.cover
            )
            .execute(&mut *tx)
            .await
            .map_err(|err| {
                log::error!(
                    "playlists.update failed to update base fields for {}: {err}",
                    playlist_id
                );
                RepoError::StorageError
            })?;
        }

        if let Some(tag_ids) = update_playlist.tag_ids {
            sqlx::query!(
                "DELETE FROM playlist_tags WHERE playlist_id = $1",
                playlist_id
            )
            .execute(&mut *tx)
            .await
            .map_err(|err| {
                log::error!(
                    "playlists.update failed to clear tags for {}: {err}",
                    playlist_id
                );
                RepoError::StorageError
            })?;

            for tag_id in tag_ids {
                sqlx::query!(
                    "INSERT INTO playlist_tags (playlist_id, tag_id) VALUES ($1, $2)",
                    playlist_id,
                    tag_id
                )
                .execute(&mut *tx)
                .await
                .map_err(|err| {
                    log::error!(
                        "playlists.update failed to attach tag {} to {}: {err}",
                        tag_id,
                        playlist_id
                    );
                    RepoError::StorageError
                })?;
            }
        }

        if let Some(items) = update_playlist.items {
            sqlx::query!(
                "DELETE FROM playlist_items WHERE playlist_id = $1",
                playlist_id
            )
            .execute(&mut *tx)
            .await
            .map_err(|err| {
                log::error!(
                    "playlists.update failed to clear items for {}: {err}",
                    playlist_id
                );
                RepoError::StorageError
            })?;

            for item in items {
                let position = i32::try_from(item.position).map_err(|err| {
                    log::error!(
                        "playlists.update invalid item position {} for {}: {err}",
                        item.position,
                        playlist_id
                    );
                    RepoError::StorageError
                })?;
                let item_id = Uuid::now_v7();

                match item.content {
                    NewPlaylistItemContent::Post { post_id } => {
                        sqlx::query!(
                            r#"
                            INSERT INTO playlist_items (id, playlist_id, position, post_id, note_text)
                            VALUES ($1, $2, $3, $4, NULL)
                            "#,
                            item_id,
                            playlist_id,
                            position,
                            post_id
                        )
                        .execute(&mut *tx)
                        .await
                        .map_err(|err| {
                            log::error!(
                                "playlists.update failed to insert post item {} for {}: {err}",
                                item_id,
                                playlist_id
                            );
                            RepoError::StorageError
                        })?;
                    }
                    NewPlaylistItemContent::Note { text } => {
                        sqlx::query!(
                            r#"
                            INSERT INTO playlist_items (id, playlist_id, position, post_id, note_text)
                            VALUES ($1, $2, $3, NULL, $4)
                            "#,
                            item_id,
                            playlist_id,
                            position,
                            text
                        )
                        .execute(&mut *tx)
                        .await
                        .map_err(|err| {
                            log::error!(
                                "playlists.update failed to insert note item {} for {}: {err}",
                                item_id,
                                playlist_id
                            );
                            RepoError::StorageError
                        })?;
                    }
                }
            }
        }

        tx.commit().await.map_err(|err| {
            log::error!(
                "playlists.update failed to commit transaction for {}: {err}",
                playlist_id
            );
            RepoError::StorageError
        })?;

        Ok(())
    }

    async fn delete(&self, user_id: UserID, playlist_id: PlaylistID) -> Result<(), RepoError> {
        let result = sqlx::query!(
            "DELETE FROM playlists WHERE id = $1 AND owner_id = $2",
            playlist_id,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|err| {
            log::error!("playlists.delete failed for {}: {err}", playlist_id);
            RepoError::StorageError
        })?;

        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }

        Ok(())
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
        let requested_direction = cursor.direction.unwrap_or_default();
        let direction = if use_cursor {
            requested_direction
        } else {
            KeysetDirection::Next
        };
        let last_id = cursor.last_id.unwrap_or_else(Uuid::nil);
        let last_score = cursor.last_score.unwrap_or(f64::MAX);

        let text = query.text.trim();
        let use_text_filter = !text.is_empty();
        let text_pattern = format!("%{text}%");

        let parsed_rows: Vec<(PlaylistSummary, f64)> = match direction {
            KeysetDirection::Next => sqlx::query!(
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
            .map(|rows| {
                rows.into_iter()
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
                    .collect()
            }),
            KeysetDirection::Prev => sqlx::query!(
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
                        OR should_score::double precision > $8
                        OR (should_score::double precision = $8 AND id > $9)
                    ORDER BY should_score ASC, id ASC
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
            .map(|rows| {
                rows.into_iter()
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
                    .collect()
            }),
        }
        .map_err(|err| {
            log::error!("playlists.search db query failed: {err}");
            RepoError::StorageError
        })?;

        Ok(Self::build_keyset_response(
            parsed_rows,
            limit,
            direction,
            use_cursor,
        ))
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
        let requested_direction = cursor.direction.unwrap_or_default();
        let direction = if use_cursor {
            requested_direction
        } else {
            KeysetDirection::Next
        };
        let last_id = cursor.last_id.unwrap_or_else(Uuid::nil);

        let parsed_rows: Vec<(PlaylistSummary, f64)> = match direction {
            KeysetDirection::Next => sqlx::query!(
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
            .map(|rows| {
                rows.into_iter()
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
                    .collect()
            }),
            KeysetDirection::Prev => sqlx::query!(
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
                        AND ($2 = false OR pl.id > $3)
                    GROUP BY pl.id
                    ORDER BY pl.id ASC
                    LIMIT $4
                    "#,
                user_id,
                use_cursor,
                last_id,
                query_limit,
            )
            .fetch_all(&self.pool)
            .await
            .map(|rows| {
                rows.into_iter()
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
                    .collect()
            }),
        }
        .map_err(|err| {
            log::error!("playlists.get_all db query failed: {err}");
            RepoError::StorageError
        })?;

        Ok(Self::build_keyset_response(
            parsed_rows,
            limit,
            direction,
            use_cursor,
        ))
    }
}
