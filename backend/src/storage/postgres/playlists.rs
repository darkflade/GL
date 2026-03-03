use crate::application::contracts::{
    KeysetCursor, KeysetDirection, KeysetPageCursor, NewPlaylist, NewPlaylistItemContent,
    PaginationMode, PlaylistItemEvent, PlaylistQuery, SearchPlaylistsResponse, UpdatePlaylist,
};
use crate::application::ports::PlaylistRepository;
use crate::domain::model::{
    Playlist, PlaylistContent, PlaylistID, PlaylistItem, PlaylistSummary, Post, RepoError, Tag,
    UserID,
};
use crate::storage::postgres::dto::{FileResponse, PostNoteResponse, TagResponse};
use async_trait::async_trait;
use serde::Deserialize;
use sqlx::types::Json;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresPlaylistRepository {
    pool: PgPool,
}

#[derive(Debug, Deserialize)]
struct PlaylistItemPayload {
    id: Uuid,
    position: i64,
    post: Option<PlaylistPostPayload>,
    note: Option<String>,
}

#[derive(Debug)]
struct PlaylistItemRank {
    id: Uuid,
    rank: String,
}

#[derive(Debug, Deserialize)]
struct PlaylistPostPayload {
    id: Uuid,
    title: String,
    description: Option<String>,
    file: FileResponse,
    tags: Vec<TagResponse>,
    notes: Vec<PostNoteResponse>,
}

impl PostgresPlaylistRepository {
    const DEFAULT_KEYSET_LIMIT: i64 = 30;
    const MAX_KEYSET_LIMIT: i64 = 100;
    const RANK_ALPHABET: &'static [u8; 64] =
        b"-0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz";
    const MAX_RANK_DEPTH: usize = 128;

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

    fn rank_char_to_digit(ch: u8) -> Option<u8> {
        Self::RANK_ALPHABET
            .iter()
            .position(|c| *c == ch)
            .map(|idx| idx as u8)
    }

    fn rank_digit_to_char(digit: u8) -> u8 {
        Self::RANK_ALPHABET[digit as usize]
    }

    fn rank_digit_at(bound: Option<&str>, idx: usize, fallback: u8) -> Result<u8, RepoError> {
        match bound {
            Some(value) => match value.as_bytes().get(idx).copied() {
                Some(ch) => Self::rank_char_to_digit(ch).ok_or_else(|| {
                    log::error!(
                        "playlists.rank contains unsupported character: {}",
                        ch as char
                    );
                    RepoError::StorageError
                }),
                None => Ok(fallback),
            },
            None => Ok(fallback),
        }
    }

    fn rank_between(prev: Option<&str>, next: Option<&str>) -> Result<String, RepoError> {
        if let (Some(prev_rank), Some(next_rank)) = (prev, next) {
            if prev_rank >= next_rank {
                log::error!(
                    "playlists.rank invalid bounds: prev_rank={} next_rank={}",
                    prev_rank,
                    next_rank
                );
                return Err(RepoError::StorageError);
            }
        }

        let min_digit = 0_u8;
        let max_digit = (Self::RANK_ALPHABET.len() - 1) as u8;
        let mut output = Vec::new();

        for idx in 0..Self::MAX_RANK_DEPTH {
            let left = Self::rank_digit_at(prev, idx, min_digit)?;
            let right = Self::rank_digit_at(next, idx, max_digit)?;

            if left > right {
                log::error!(
                    "playlists.rank invalid digit range at idx {}: left={} right={}",
                    idx,
                    left,
                    right
                );
                return Err(RepoError::StorageError);
            }

            if left + 1 < right {
                let middle = left + (right - left) / 2;
                output.push(Self::rank_digit_to_char(middle));
                return String::from_utf8(output).map_err(|err| {
                    log::error!("playlists.rank failed to encode utf8 output: {err}");
                    RepoError::StorageError
                });
            }

            output.push(Self::rank_digit_to_char(left));
        }

        log::error!("playlists.rank failed to allocate an in-between rank (depth exceeded)");
        Err(RepoError::StorageError)
    }

    async fn insert_playlist_item(
        tx: &mut Transaction<'_, Postgres>,
        playlist_id: PlaylistID,
        rank: &str,
        content: NewPlaylistItemContent,
    ) -> Result<Uuid, RepoError> {
        let item_id = Uuid::now_v7();
        match content {
            NewPlaylistItemContent::Post { post_id } => {
                sqlx::query!(
                    r#"
                    INSERT INTO playlist_items (id, playlist_id, rank, post_id, note_text)
                    VALUES ($1, $2, $3, $4, NULL)
                    "#,
                    item_id,
                    playlist_id,
                    rank,
                    post_id
                )
                .execute(&mut **tx)
                .await
                .map_err(|err| {
                    log::error!(
                        "playlists.item.insert failed to insert post item {} for {}: {err}",
                        item_id,
                        playlist_id
                    );
                    RepoError::StorageError
                })?;
            }
            NewPlaylistItemContent::Note { text } => {
                sqlx::query!(
                    r#"
                    INSERT INTO playlist_items (id, playlist_id, rank, post_id, note_text)
                    VALUES ($1, $2, $3, NULL, $4)
                    "#,
                    item_id,
                    playlist_id,
                    rank,
                    text
                )
                .execute(&mut **tx)
                .await
                .map_err(|err| {
                    log::error!(
                        "playlists.item.insert failed to insert note item {} for {}: {err}",
                        item_id,
                        playlist_id
                    );
                    RepoError::StorageError
                })?;
            }
        }

        Ok(item_id)
    }

    async fn edit_playlist_item_content(
        tx: &mut Transaction<'_, Postgres>,
        playlist_id: PlaylistID,
        item_id: Uuid,
        content: NewPlaylistItemContent,
    ) -> Result<(), RepoError> {
        let result = match content {
            NewPlaylistItemContent::Post { post_id } => sqlx::query!(
                r#"
                    UPDATE playlist_items
                    SET post_id = $3, note_text = NULL
                    WHERE id = $1 AND playlist_id = $2
                    "#,
                item_id,
                playlist_id,
                post_id
            )
            .execute(&mut **tx)
            .await
            .map_err(|err| {
                log::error!(
                    "playlists.item.edit failed to update post content for {} in {}: {err}",
                    item_id,
                    playlist_id
                );
                RepoError::StorageError
            })?,
            NewPlaylistItemContent::Note { text } => sqlx::query!(
                r#"
                    UPDATE playlist_items
                    SET post_id = NULL, note_text = $3
                    WHERE id = $1 AND playlist_id = $2
                    "#,
                item_id,
                playlist_id,
                text
            )
            .execute(&mut **tx)
            .await
            .map_err(|err| {
                log::error!(
                    "playlists.item.edit failed to update note content for {} in {}: {err}",
                    item_id,
                    playlist_id
                );
                RepoError::StorageError
            })?,
        };

        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }

        Ok(())
    }

    async fn fetch_item_ranks(
        tx: &mut Transaction<'_, Postgres>,
        playlist_id: PlaylistID,
    ) -> Result<Vec<PlaylistItemRank>, RepoError> {
        sqlx::query!(
            r#"
            SELECT id, rank
            FROM playlist_items
            WHERE playlist_id = $1
            ORDER BY rank ASC
            "#,
            playlist_id
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(|err| {
            log::error!(
                "playlists.item.fetch failed to load item ranks for {}: {err}",
                playlist_id
            );
            RepoError::StorageError
        })
        .map(|rows| {
            rows.into_iter()
                .map(|row| PlaylistItemRank {
                    id: row.id,
                    rank: row.rank,
                })
                .collect()
        })
    }

    fn resolve_rank_window(
        playlist_id: PlaylistID,
        ordered_items: &[PlaylistItemRank],
        after_id: Option<Uuid>,
        exclude_id: Option<Uuid>,
    ) -> Result<(Option<String>, Option<String>), RepoError> {
        let filtered: Vec<&PlaylistItemRank> = ordered_items
            .iter()
            .filter(|item| Some(item.id) != exclude_id)
            .collect();

        if filtered.is_empty() {
            if after_id.is_some() {
                log::error!(
                    "playlists.rank.resolve cannot place after non-existing anchor in {}",
                    playlist_id
                );
                return Err(RepoError::NotFound);
            }
            return Ok((None, None));
        }

        match after_id {
            None => Ok((None, Some(filtered[0].rank.clone()))),
            Some(anchor_id) => {
                let anchor_index = filtered.iter().position(|item| item.id == anchor_id);
                let Some(anchor_index) = anchor_index else {
                    log::error!(
                        "playlists.rank.resolve anchor {} not found in {}",
                        anchor_id,
                        playlist_id
                    );
                    return Err(RepoError::NotFound);
                };
                let prev = Some(filtered[anchor_index].rank.clone());
                let next = filtered.get(anchor_index + 1).map(|item| item.rank.clone());
                Ok((prev, next))
            }
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
                        notes: post.notes.into_iter().map(Into::into).collect(),
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

        if let Some(mut items) = new_playlist.items {
            items.sort_by_key(|item| item.position);
            let mut prev_rank: Option<String> = None;

            for item in items {
                let rank = Self::rank_between(prev_rank.as_deref(), None)?;
                Self::insert_playlist_item(&mut tx, playlist_id, &rank, item.content).await?;
                prev_rank = Some(rank);
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
                                'position', ROW_NUMBER() OVER (ORDER BY pi.rank),
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
                                        ),
                                        'notes', COALESCE(
                                            (
                                                SELECT jsonb_agg(
                                                    jsonb_build_object(
                                                        'id', pn.id,
                                                        'text', pn.text,
                                                        'x', pn.pos_x,
                                                        'y', pn.pos_y
                                                    )
                                                    ORDER BY pn.id
                                                )
                                                FROM post_notes pn
                                                WHERE pn.post_id = p.id
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
                            ORDER BY pi.rank
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

        if let Some(remove_tag_ids) = update_playlist.remove_tag_ids {
            if !remove_tag_ids.is_empty() {
                sqlx::query!(
                    "DELETE FROM playlist_tags WHERE playlist_id = $1 AND tag_id = ANY($2)",
                    playlist_id,
                    &remove_tag_ids
                )
                .execute(&mut *tx)
                .await
                .map_err(|err| {
                    log::error!(
                        "playlists.update failed to remove tags for {}: {err}",
                        playlist_id
                    );
                    RepoError::StorageError
                })?;
            }
        }

        if let Some(add_tag_ids) = update_playlist.add_tag_ids {
            if !add_tag_ids.is_empty() {
                sqlx::query!(
                    r#"
                    INSERT INTO playlist_tags (playlist_id, tag_id)
                    SELECT $1, tag_id
                    FROM UNNEST($2::uuid[]) AS tag_id
                    ON CONFLICT (playlist_id, tag_id) DO NOTHING
                    "#,
                    playlist_id,
                    &add_tag_ids
                )
                .execute(&mut *tx)
                .await
                .map_err(|err| {
                    log::error!(
                        "playlists.update failed to add tags for {}: {err}",
                        playlist_id
                    );
                    RepoError::StorageError
                })?;
            }
        }

        if let Some(events) = update_playlist.item_events {
            let mut transform_events = Vec::new();
            let mut add_events = Vec::new();
            let mut remove_events = Vec::new();

            for event in events {
                match event {
                    PlaylistItemEvent::Edit { .. } | PlaylistItemEvent::Move { .. } => {
                        transform_events.push(event);
                    }
                    PlaylistItemEvent::Add { after_id, content } => {
                        add_events.push((after_id, content));
                    }
                    PlaylistItemEvent::Remove { item_id } => {
                        remove_events.push(item_id);
                    }
                }
            }

            for event in transform_events {
                match event {
                    PlaylistItemEvent::Edit { item_id, content } => {
                        Self::edit_playlist_item_content(&mut tx, playlist_id, item_id, content)
                            .await?;
                    }
                    PlaylistItemEvent::Move { item_id, after_id } => {
                        if after_id == Some(item_id) {
                            continue;
                        }

                        let ordered_items = Self::fetch_item_ranks(&mut tx, playlist_id).await?;
                        if !ordered_items.iter().any(|item| item.id == item_id) {
                            return Err(RepoError::NotFound);
                        }

                        let (prev_rank, next_rank) = Self::resolve_rank_window(
                            playlist_id,
                            &ordered_items,
                            after_id,
                            Some(item_id),
                        )?;
                        let new_rank =
                            Self::rank_between(prev_rank.as_deref(), next_rank.as_deref())?;

                        let result = sqlx::query!(
                            "UPDATE playlist_items SET rank = $3 WHERE id = $1 AND playlist_id = $2",
                            item_id,
                            playlist_id,
                            new_rank
                        )
                        .execute(&mut *tx)
                        .await
                        .map_err(|err| {
                            log::error!(
                                "playlists.update failed to move item {} for {}: {err}",
                                item_id,
                                playlist_id
                            );
                            RepoError::StorageError
                        })?;

                        if result.rows_affected() == 0 {
                            return Err(RepoError::NotFound);
                        }
                    }
                    _ => unreachable!("non-transform event in transform phase"),
                }
            }

            for (after_id, content) in add_events {
                let ordered_items = Self::fetch_item_ranks(&mut tx, playlist_id).await?;
                let (prev_rank, next_rank) =
                    Self::resolve_rank_window(playlist_id, &ordered_items, after_id, None)?;
                let new_rank = Self::rank_between(prev_rank.as_deref(), next_rank.as_deref())?;
                Self::insert_playlist_item(&mut tx, playlist_id, &new_rank, content).await?;
            }

            for item_id in remove_events {
                let result = sqlx::query!(
                    "DELETE FROM playlist_items WHERE id = $1 AND playlist_id = $2",
                    item_id,
                    playlist_id
                )
                .execute(&mut *tx)
                .await
                .map_err(|err| {
                    log::error!(
                        "playlists.update failed to remove item {} for {}: {err}",
                        item_id,
                        playlist_id
                    );
                    RepoError::StorageError
                })?;

                if result.rows_affected() == 0 {
                    return Err(RepoError::NotFound);
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
