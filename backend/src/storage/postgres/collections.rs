use crate::application::contracts::{
    CollectionItemEvent, CollectionQuery, KeysetCursor, KeysetDirection, KeysetPageCursor,
    NewCollection, NewCollectionItemContent, PaginationMode, SearchCollectionsResponse,
    UpdateCollection,
};
use crate::application::ports::CollectionRepository;
use crate::domain::model::{
    Collection, CollectionContent, CollectionID, CollectionItem, CollectionSummary, Post,
    RepoError, Tag, UserID,
};
use crate::storage::postgres::dto::{FileResponse, PostNoteResponse, TagResponse};
use async_trait::async_trait;
use serde::Deserialize;
use sqlx::types::Json;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresCollectionRepository {
    pool: PgPool,
}

#[derive(Debug, Deserialize)]
struct CollectionItemPayload {
    id: Uuid,
    position: i64,
    post: Option<CollectionPostPayload>,
    note: Option<String>,
}

#[derive(Debug)]
struct CollectionItemRank {
    id: Uuid,
    rank: String,
}

#[derive(Debug, Deserialize)]
struct CollectionPostPayload {
    id: Uuid,
    title: String,
    description: Option<String>,
    file: FileResponse,
    tags: Vec<TagResponse>,
    notes: Vec<PostNoteResponse>,
}

impl PostgresCollectionRepository {
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
        mut entries: Vec<(CollectionSummary, f64)>,
        limit: i64,
        direction: KeysetDirection,
        use_cursor: bool,
    ) -> SearchCollectionsResponse {
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
            entries.last().map(|(collection, score)| KeysetPageCursor {
                mode: PaginationMode::Keyset,
                direction: KeysetDirection::Next,
                last_id: collection.id,
                last_score: *score,
                limit,
            })
        } else {
            None
        };

        let prev_cursor = if has_prev {
            entries.first().map(|(collection, score)| KeysetPageCursor {
                mode: PaginationMode::Keyset,
                direction: KeysetDirection::Prev,
                last_id: collection.id,
                last_score: *score,
                limit,
            })
        } else {
            None
        };

        SearchCollectionsResponse {
            collections: entries
                .into_iter()
                .map(|(collection, _)| collection)
                .collect(),
            has_next,
            has_prev,
            next_cursor,
            prev_cursor,
        }
    }

    // Fixed-width hexadecimal keys sort numerically under PostgreSQL's C collation.
    // At either edge use a constant stride, avoiding repeated interval halving.
    fn rank_between(prev: Option<&str>, next: Option<&str>) -> Result<String, RepoError> {
        let parse = |s: &str| {
            if s.len() != 32 {
                return Err(RepoError::StorageError);
            }
            u128::from_str_radix(s, 16).map_err(|_| RepoError::StorageError)
        };
        let left = prev.map(parse).transpose()?;
        let right = next.map(parse).transpose()?;
        let step = 1_u128 << 64;
        let value = match (left, right) {
            (None, None) => Some(1_u128 << 127),
            (Some(l), None) => l.checked_add(step).filter(|v| *v < u128::MAX),
            (None, Some(r)) => r.checked_sub(step).filter(|v| *v > 0),
            (Some(l), Some(r)) if r > l && r - l > 1 => Some(l + (r - l) / 2),
            _ => None,
        }
        .ok_or(RepoError::StorageError)?;
        Ok(format!("{value:032x}"))
    }

    async fn allocate_rank(
        tx: &mut Transaction<'_, Postgres>,
        collection_id: CollectionID,
        after_id: Option<Uuid>,
        moving_id: Option<Uuid>,
    ) -> Result<String, RepoError> {
        let mut items = Self::fetch_item_ranks(tx, collection_id).await?;
        let (prev, next) = Self::resolve_rank_window(collection_id, &items, after_id, moving_id)?;
        if let Ok(rank) = Self::rank_between(prev.as_deref(), next.as_deref()) {
            return Ok(rank);
        }
        // Rare dense interior intervals are rebalanced in the same locked transaction.
        let stride = u128::MAX / (items.len() as u128 + 2);
        for (index, item) in items.iter_mut().enumerate() {
            item.rank = format!("{:032x}", stride * (index as u128 + 1));
            sqlx::query(
                "UPDATE collection_items SET rank = $1 WHERE id = $2 AND collection_id = $3",
            )
            .bind(&item.rank)
            .bind(item.id)
            .bind(collection_id)
            .execute(&mut **tx)
            .await
            .map_err(|_| RepoError::StorageError)?;
        }
        let (prev, next) = Self::resolve_rank_window(collection_id, &items, after_id, moving_id)?;
        Self::rank_between(prev.as_deref(), next.as_deref())
    }

    async fn insert_collection_item(
        tx: &mut Transaction<'_, Postgres>,
        collection_id: CollectionID,
        rank: &str,
        content: NewCollectionItemContent,
    ) -> Result<Uuid, RepoError> {
        let item_id = Uuid::now_v7();
        match content {
            NewCollectionItemContent::Post { post_id } => {
                sqlx::query!(
                    r#"
                    INSERT INTO collection_items (id, collection_id, rank, post_id, note_text)
                    VALUES ($1, $2, $3, $4, NULL)

                    "#,
                    item_id,
                    collection_id,
                    rank,
                    post_id
                )
                .execute(&mut **tx)
                .await
                .map_err(|err| {
                    log::error!(
                        "collections.item.insert failed to insert post item {} for {}: {err}",
                        item_id,
                        collection_id
                    );
                    RepoError::StorageError
                })?;
            }
            NewCollectionItemContent::Note { text } => {
                sqlx::query!(
                    r#"
                    INSERT INTO collection_items (id, collection_id, rank, post_id, note_text)
                    VALUES ($1, $2, $3, NULL, $4)

                    "#,
                    item_id,
                    collection_id,
                    rank,
                    text
                )
                .execute(&mut **tx)
                .await
                .map_err(|err| {
                    log::error!(
                        "collections.item.insert failed to insert note item {} for {}: {err}",
                        item_id,
                        collection_id
                    );
                    RepoError::StorageError
                })?;
            }
        }

        Ok(item_id)
    }

    async fn edit_collection_item_content(
        tx: &mut Transaction<'_, Postgres>,
        collection_id: CollectionID,
        item_id: Uuid,
        content: NewCollectionItemContent,
    ) -> Result<(), RepoError> {
        let result = match content {
            NewCollectionItemContent::Post { post_id } => sqlx::query!(
                r#"
                    UPDATE collection_items
                    SET post_id = $3, note_text = NULL
                    WHERE id = $1 AND collection_id = $2

                    "#,
                item_id,
                collection_id,
                post_id
            )
            .execute(&mut **tx)
            .await
            .map_err(|err| {
                log::error!(
                    "collections.item.edit failed to update post content for {} in {}: {err}",
                    item_id,
                    collection_id
                );
                RepoError::StorageError
            })?,
            NewCollectionItemContent::Note { text } => sqlx::query!(
                r#"
                    UPDATE collection_items
                    SET post_id = NULL, note_text = $3
                    WHERE id = $1 AND collection_id = $2

                    "#,
                item_id,
                collection_id,
                text
            )
            .execute(&mut **tx)
            .await
            .map_err(|err| {
                log::error!(
                    "collections.item.edit failed to update note content for {} in {}: {err}",
                    item_id,
                    collection_id
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
        collection_id: CollectionID,
    ) -> Result<Vec<CollectionItemRank>, RepoError> {
        sqlx::query!(
            r#"
            SELECT id, rank
            FROM collection_items
            WHERE collection_id = $1
            ORDER BY rank ASC

            "#,
            collection_id
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(|err| {
            log::error!(
                "collections.item.fetch failed to load item ranks for {}: {err}",
                collection_id
            );
            RepoError::StorageError
        })
        .map(|rows| {
            rows.into_iter()
                .map(|row| CollectionItemRank {
                    id: row.id,
                    rank: row.rank,
                })
                .collect()
        })
    }

    fn resolve_rank_window(
        collection_id: CollectionID,
        ordered_items: &[CollectionItemRank],
        after_id: Option<Uuid>,
        exclude_id: Option<Uuid>,
    ) -> Result<(Option<String>, Option<String>), RepoError> {
        let filtered: Vec<&CollectionItemRank> = ordered_items
            .iter()
            .filter(|item| Some(item.id) != exclude_id)
            .collect();

        if filtered.is_empty() {
            if after_id.is_some() {
                log::error!(
                    "collections.rank.resolve cannot place after non-existing anchor in {}",
                    collection_id
                );
                return Err(RepoError::NotFound);
            }
            return Ok((None, None));
        }

        match after_id {
            //Todo
            None => Ok((None, Some(filtered[0].rank.clone()))),
            Some(anchor_id) => {
                let anchor_index = filtered.iter().position(|item| item.id == anchor_id);
                let Some(anchor_index) = anchor_index else {
                    log::error!(
                        "collections.rank.resolve anchor {} not found in {}",
                        anchor_id,
                        collection_id
                    );
                    return Err(RepoError::NotFound);
                };
                let prev = Some(filtered[anchor_index].rank.clone());
                let next = filtered.get(anchor_index + 1).map(|item| item.rank.clone());
                Ok((prev, next))
            }
        }
    }

    fn map_collection_items(items: Vec<CollectionItemPayload>) -> Vec<CollectionItem> {
        items
            .into_iter()
            .map(|item| {
                let content = match item.post {
                    Some(post) => CollectionContent::Post(Post {
                        id: post.id,
                        title: post.title,
                        description: post.description,
                        tags: post.tags.into_iter().map(Tag::from).collect(),
                        file: post.file.into(),
                        notes: post.notes.into_iter().map(Into::into).collect(),
                    }),
                    None => CollectionContent::Note(item.note.unwrap_or_default()),
                };

                CollectionItem {
                    id: item.id,
                    position: item.position.max(0) as u32,
                    content,
                }
            })
            .collect()
    }
}

#[async_trait]
impl CollectionRepository for PostgresCollectionRepository {
    async fn create(
        &self,
        user_id: UserID,
        new_collection: NewCollection,
    ) -> Result<CollectionID, RepoError> {
        let mut tx = self.pool.begin().await.map_err(|err| {
            log::error!("collections.create failed to begin transaction: {err}");
            RepoError::StorageError
        })?;

        let collection_id = Uuid::now_v7();

        sqlx::query!(
            r#"
            INSERT INTO collections (id, title, description, cover_file_id, owner_id)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            collection_id,
            new_collection.title,
            new_collection.description,
            new_collection.cover,
            user_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|err| {
            log::error!(
                "collections.create failed to insert collection {} for {}: {err}",
                collection_id,
                user_id
            );
            RepoError::StorageError
        })?;

        if let Some(tag_ids) = new_collection.tag_ids {
            for tag_id in tag_ids {
                sqlx::query!(
                    "INSERT INTO collection_tags (collection_id, tag_id) VALUES ($1, $2)",
                    collection_id,
                    tag_id
                )
                .execute(&mut *tx)
                .await
                .map_err(|err| {
                    log::error!(
                        "collections.create failed to attach tag {} to {}: {err}",
                        tag_id,
                        collection_id
                    );
                    RepoError::StorageError
                })?;
            }
        }

        if let Some(mut items) = new_collection.items {
            items.sort_by_key(|item| item.position);
            let mut prev_rank: Option<String> = None;

            for item in items {
                let rank = Self::rank_between(prev_rank.as_deref(), None)?;
                Self::insert_collection_item(&mut tx, collection_id, &rank, item.content).await?;
                prev_rank = Some(rank);
            }
        }

        tx.commit().await.map_err(|err| {
            log::error!(
                "collections.create failed to commit transaction for {}: {err}",
                collection_id
            );
            RepoError::StorageError
        })?;

        Ok(collection_id)
    }

    async fn get(
        &self,
        user_id: UserID,
        collection_id: CollectionID,
    ) -> Result<Collection, RepoError> {
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
                        FROM collection_tags pt
                        JOIN tags t ON t.id = pt.tag_id
                        WHERE pt.collection_id = pl.id
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
                            FROM collection_items pi
                            LEFT JOIN posts p ON p.id = pi.post_id
                            LEFT JOIN files f ON f.id = p.file_id
                            WHERE pi.collection_id = pl.id
                            ORDER BY pi.rank
                        ) raw_items
                    ),
                    '[]'::jsonb
                ) AS "items!: Json<Vec<CollectionItemPayload>>"
            FROM collections pl
            WHERE pl.id = $1 AND pl.owner_id = $2

            "#,
            collection_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| {
            log::error!(
                "collections.get failed to fetch collection {}: {err}",
                collection_id
            );
            RepoError::StorageError
        })?;

        let row = row.ok_or(RepoError::NotFound)?;

        Ok(Collection {
            id: collection_id,
            title: row.title,
            description: row.description,
            tags: row.tags.0.into_iter().map(Tag::from).collect(),
            cover: row.cover,
            items: Self::map_collection_items(row.items.0),
        })
    }

    async fn update(
        &self,
        user_id: UserID,
        collection_id: CollectionID,
        update_collection: UpdateCollection,
    ) -> Result<(), RepoError> {
        let mut tx = self.pool.begin().await.map_err(|err| {
            log::error!(
                "collections.update failed to begin transaction for {}: {err}",
                collection_id
            );
            RepoError::StorageError
        })?;

        let exists = sqlx::query!(
            "SELECT id FROM collections WHERE id = $1 AND owner_id = $2 FOR UPDATE",
            collection_id,
            user_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|err| {
            log::error!(
                "collections.update failed to check collection {} ownership for {}: {err}",
                collection_id,
                user_id
            );
            RepoError::StorageError
        })?;

        if exists.is_none() {
            return Err(RepoError::NotFound);
        }

        if update_collection.title.is_some()
            || update_collection.description.is_some()
            || update_collection.cover.is_some()
        {
            sqlx::query!(
                r#"
                UPDATE collections
                SET
                    title = COALESCE($3, title),
                    description = COALESCE($4, description),
                    cover_file_id = COALESCE($5, cover_file_id),
                    updated_at = NOW()
                WHERE id = $1 AND owner_id = $2
                "#,
                collection_id,
                user_id,
                update_collection.title,
                update_collection.description,
                update_collection.cover
            )
            .execute(&mut *tx)
            .await
            .map_err(|err| {
                log::error!(
                    "collections.update failed to update base fields for {}: {err}",
                    collection_id
                );
                RepoError::StorageError
            })?;
        }

        if let Some(remove_tag_ids) = update_collection.remove_tag_ids {
            if !remove_tag_ids.is_empty() {
                sqlx::query!(
                    "DELETE FROM collection_tags WHERE collection_id = $1 AND tag_id = ANY($2)",
                    collection_id,
                    &remove_tag_ids
                )
                .execute(&mut *tx)
                .await
                .map_err(|err| {
                    log::error!(
                        "collections.update failed to remove tags for {}: {err}",
                        collection_id
                    );
                    RepoError::StorageError
                })?;
            }
        }

        if let Some(add_tag_ids) = update_collection.add_tag_ids {
            if !add_tag_ids.is_empty() {
                sqlx::query!(
                    r#"
                    INSERT INTO collection_tags (collection_id, tag_id)
                    SELECT $1, tag_id
                    FROM UNNEST($2::uuid[]) AS tag_id
                    ON CONFLICT (collection_id, tag_id) DO NOTHING
                    "#,
                    collection_id,
                    &add_tag_ids
                )
                .execute(&mut *tx)
                .await
                .map_err(|err| {
                    log::error!(
                        "collections.update failed to add tags for {}: {err}",
                        collection_id
                    );
                    RepoError::StorageError
                })?;
            }
        }

        if let Some(events) = update_collection.item_events {
            let mut transform_events = Vec::new();
            let mut add_events = Vec::new();
            let mut remove_events = Vec::new();

            for event in events {
                match event {
                    CollectionItemEvent::Edit { .. } | CollectionItemEvent::Move { .. } => {
                        transform_events.push(event);
                    }
                    CollectionItemEvent::Add { after_id, content } => {
                        add_events.push((after_id, content));
                    }
                    CollectionItemEvent::Remove { item_id } => {
                        remove_events.push(item_id);
                    }
                }
            }

            for event in transform_events {
                match event {
                    CollectionItemEvent::Edit { item_id, content } => {
                        Self::edit_collection_item_content(
                            &mut tx,
                            collection_id,
                            item_id,
                            content,
                        )
                        .await?;
                    }
                    CollectionItemEvent::Move { item_id, after_id } => {
                        if after_id == Some(item_id) {
                            continue;
                        }

                        let ordered_items = Self::fetch_item_ranks(&mut tx, collection_id).await?;
                        if !ordered_items.iter().any(|item| item.id == item_id) {
                            return Err(RepoError::NotFound);
                        }

                        let new_rank =
                            Self::allocate_rank(&mut tx, collection_id, after_id, Some(item_id))
                                .await?;

                        let result = sqlx::query!(
                            "UPDATE collection_items SET rank = $3 WHERE id = $1 AND collection_id = $2
",
                            item_id,
                            collection_id,
                            new_rank
                        )
                        .execute(&mut *tx)
                        .await
                        .map_err(|err| {
                            log::error!(
                                "collections.update failed to move item {} for {}: {err}",
                                item_id,
                                collection_id
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
                let new_rank = Self::allocate_rank(&mut tx, collection_id, after_id, None).await?;
                Self::insert_collection_item(&mut tx, collection_id, &new_rank, content).await?;
            }

            for item_id in remove_events {
                let result = sqlx::query!(
                    "DELETE FROM collection_items WHERE id = $1 AND collection_id = $2
",
                    item_id,
                    collection_id
                )
                .execute(&mut *tx)
                .await
                .map_err(|err| {
                    log::error!(
                        "collections.update failed to remove item {} for {}: {err}",
                        item_id,
                        collection_id
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
                "collections.update failed to commit transaction for {}: {err}",
                collection_id
            );
            RepoError::StorageError
        })?;

        Ok(())
    }

    async fn delete(&self, user_id: UserID, collection_id: CollectionID) -> Result<(), RepoError> {
        let result = sqlx::query!(
            "DELETE FROM collections WHERE id = $1 AND owner_id = $2",
            collection_id,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|err| {
            log::error!("collections.delete failed for {}: {err}", collection_id);
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
        query: CollectionQuery,
        cursor: KeysetCursor,
    ) -> Result<SearchCollectionsResponse, RepoError> {
        log::debug!("collections.search user={user_id} query={query:?} cursor={cursor:?}");

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

        let parsed_rows: Vec<(CollectionSummary, f64)> = match direction {
            KeysetDirection::Next => sqlx::query!(
                r#"
                    WITH ranked_collections AS (
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
                        FROM collections pl
                        LEFT JOIN collection_items pi ON pi.collection_id = pl.id
                        LEFT JOIN collection_tags plt ON plt.collection_id = pl.id
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
                                FROM collection_tags x
                                JOIN tags tx ON tx.id = x.tag_id
                                WHERE x.collection_id = pl.id
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
                    FROM ranked_collections
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
                            CollectionSummary {
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
                    WITH ranked_collections AS (
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
                        FROM collections pl
                        LEFT JOIN collection_items pi ON pi.collection_id = pl.id
                        LEFT JOIN collection_tags plt ON plt.collection_id = pl.id
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
                                FROM collection_tags x
                                JOIN tags tx ON tx.id = x.tag_id
                                WHERE x.collection_id = pl.id
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
                    FROM ranked_collections
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
                            CollectionSummary {
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
            log::error!("collections.search db query failed: {err}");
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
    ) -> Result<SearchCollectionsResponse, RepoError> {
        log::debug!("collections.get_all user={user_id} cursor={cursor:?}");

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

        let parsed_rows: Vec<(CollectionSummary, f64)> = match direction {
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
                    FROM collections pl
                    LEFT JOIN collection_items pi ON pi.collection_id = pl.id
                    LEFT JOIN collection_tags plt ON plt.collection_id = pl.id
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
                            CollectionSummary {
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
                    FROM collections pl
                    LEFT JOIN collection_items pi ON pi.collection_id = pl.id
                    LEFT JOIN collection_tags plt ON plt.collection_id = pl.id
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
                            CollectionSummary {
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
            log::error!("collections.get_all db query failed: {err}");
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

#[cfg(test)]
mod rank_tests {
    use super::PostgresCollectionRepository as R;

    #[test]
    fn repeated_edge_inserts_keep_constant_length() {
        let initial = R::rank_between(None, None).unwrap();
        let mut first = initial.clone();
        let mut last = initial;
        for _ in 0..100_000 {
            let before = R::rank_between(None, Some(&first)).unwrap();
            let after = R::rank_between(Some(&last), None).unwrap();
            assert!(before < first && last < after);
            assert_eq!(before.len(), 32);
            assert_eq!(after.len(), 32);
            first = before;
            last = after;
        }
    }

    #[test]
    fn interior_and_exhausted_intervals() {
        let left = format!("{:032x}", 255_u128);
        let right = format!("{:032x}", 257_u128);
        let middle = R::rank_between(Some(&left), Some(&right)).unwrap();
        assert!(left < middle && middle < right);
        assert!(R::rank_between(Some(&left), Some(&middle)).is_err());
        assert!(R::rank_between(Some(&right), Some(&left)).is_err());
    }
}
