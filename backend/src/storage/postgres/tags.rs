use crate::application::contracts::{
    KeysetCursor, KeysetDirection, KeysetPageCursor, NewTag, PaginationMode,
    SearchTagRelationsResponse, SearchTagsResponse, TagBatchUpdate, TagRelationUpdateEvent,
    TagRelationsBatchUpdate, TagUpdateEvent,
};
use crate::application::ports::TagRepository;
use crate::domain::model::{RepoError, Tag, TagID, TagRelation};
use async_trait::async_trait;
use sqlx::{Error as SqlxError, FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresTagRepository {
    pool: PgPool,
}

impl PostgresTagRepository {
    const DEFAULT_KEYSET_LIMIT: i64 = 50;
    const MAX_KEYSET_LIMIT: i64 = 200;

    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn resolve_keyset_limit(cursor: &KeysetCursor) -> i64 {
        cursor
            .limit
            .unwrap_or(Self::DEFAULT_KEYSET_LIMIT)
            .clamp(1, Self::MAX_KEYSET_LIMIT)
    }

    fn build_tags_keyset_response(
        mut entries: Vec<(Tag, f64)>,
        limit: i64,
        direction: KeysetDirection,
        use_cursor: bool,
    ) -> SearchTagsResponse {
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
            entries.last().map(|(tag, score)| KeysetPageCursor {
                mode: PaginationMode::Keyset,
                direction: KeysetDirection::Next,
                last_id: tag.id,
                last_score: *score,
                limit,
            })
        } else {
            None
        };

        let prev_cursor = if has_prev {
            entries.first().map(|(tag, score)| KeysetPageCursor {
                mode: PaginationMode::Keyset,
                direction: KeysetDirection::Prev,
                last_id: tag.id,
                last_score: *score,
                limit,
            })
        } else {
            None
        };

        let tags = entries.into_iter().map(|(tag, _)| tag).collect();

        SearchTagsResponse {
            tags,
            has_next,
            has_prev,
            next_cursor,
            prev_cursor,
        }
    }

    fn build_relations_keyset_response(
        mut entries: Vec<(TagRelation, f64)>,
        limit: i64,
        direction: KeysetDirection,
        use_cursor: bool,
    ) -> SearchTagRelationsResponse {
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
            entries.last().map(|(relation, score)| KeysetPageCursor {
                mode: PaginationMode::Keyset,
                direction: KeysetDirection::Next,
                last_id: relation.id,
                last_score: *score,
                limit,
            })
        } else {
            None
        };

        let prev_cursor = if has_prev {
            entries.first().map(|(relation, score)| KeysetPageCursor {
                mode: PaginationMode::Keyset,
                direction: KeysetDirection::Prev,
                last_id: relation.id,
                last_score: *score,
                limit,
            })
        } else {
            None
        };

        let relations = entries.into_iter().map(|(relation, _)| relation).collect();

        SearchTagRelationsResponse {
            relations,
            has_next,
            has_prev,
            next_cursor,
            prev_cursor,
        }
    }

    fn map_db_error(err: SqlxError, context: &str) -> RepoError {
        if let SqlxError::Database(db_err) = &err {
            match db_err.code().as_deref() {
                Some("23503") => {
                    log::warn!("{context}: missing referenced row: {err}");
                    return RepoError::NotFound;
                }
                Some("23505") | Some("23514") => {
                    log::warn!("{context}: constraint conflict: {err}");
                    return RepoError::Conflict;
                }
                _ => {}
            }
        }

        log::error!("{context}: {err}");
        RepoError::StorageError
    }

    async fn insert_or_get_tag(
        tx: &mut Transaction<'_, Postgres>,
        new_tag: NewTag,
    ) -> Result<Tag, RepoError> {
        let row = sqlx::query!(
            "INSERT INTO tags (id, category, name) VALUES ($1, $2, $3)
             ON CONFLICT (category, name) DO UPDATE SET name = EXCLUDED.name
             RETURNING id, category, name, post_count AS \"count!\"",
            Uuid::now_v7(),
            new_tag.category as i16,
            new_tag.name
        )
        .fetch_one(tx.as_mut())
        .await
        .map_err(|err| Self::map_db_error(err, "tags.insert_or_get failed"))?;

        sqlx::query!(
            "INSERT INTO tag_relation_closure (ancestor_id, descendant_id, depth)
             VALUES ($1, $1, 0)
             ON CONFLICT (ancestor_id, descendant_id) DO NOTHING",
            row.id
        )
        .execute(tx.as_mut())
        .await
        .map_err(|err| Self::map_db_error(err, "tags.insert_or_get failed to ensure closure"))?;

        Ok(Tag {
            id: row.id,
            category: row.category.into(),
            name: row.name,
            count: row.count,
        })
    }

    async fn ensure_tag_exists(
        tx: &mut Transaction<'_, Postgres>,
        tag_id: TagID,
    ) -> Result<(), RepoError> {
        let exists = sqlx::query!("SELECT id FROM tags WHERE id = $1", tag_id)
            .fetch_optional(tx.as_mut())
            .await
            .map_err(|err| Self::map_db_error(err, "tags.ensure_exists failed"))?;

        if exists.is_some() {
            Ok(())
        } else {
            Err(RepoError::NotFound)
        }
    }

    fn ordered_pair(left: TagID, right: TagID) -> (TagID, TagID) {
        if left < right {
            (left, right)
        } else {
            (right, left)
        }
    }

    async fn rebuild_closure(tx: &mut Transaction<'_, Postgres>) -> Result<(), RepoError> {
        sqlx::query!("DELETE FROM tag_relation_closure")
            .execute(tx.as_mut())
            .await
            .map_err(|err| Self::map_db_error(err, "tags.rebuild_closure failed to clear"))?;

        sqlx::query!(
            "INSERT INTO tag_relation_closure (ancestor_id, descendant_id, depth)
             SELECT id, id, 0 FROM tags"
        )
        .execute(tx.as_mut())
        .await
        .map_err(|err| {
            Self::map_db_error(err, "tags.rebuild_closure failed to insert self rows")
        })?;

        sqlx::query!(
            r#"
            WITH RECURSIVE paths AS (
                SELECT
                    e.parent_id AS ancestor_id,
                    e.child_id AS descendant_id,
                    1::INT AS depth,
                    ARRAY[e.parent_id, e.child_id]::UUID[] AS visited
                FROM tag_relation_edges e
                UNION ALL
                SELECT
                    p.ancestor_id,
                    e.child_id AS descendant_id,
                    p.depth + 1,
                    p.visited || e.child_id
                FROM paths p
                JOIN tag_relation_edges e
                    ON e.parent_id = p.descendant_id
                WHERE NOT e.child_id = ANY(p.visited)
            )
            INSERT INTO tag_relation_closure (ancestor_id, descendant_id, depth)
            SELECT
                ancestor_id,
                descendant_id,
                MIN(depth) AS depth
            FROM paths
            GROUP BY ancestor_id, descendant_id
            "#
        )
        .execute(tx.as_mut())
        .await
        .map_err(|err| Self::map_db_error(err, "tags.rebuild_closure failed to insert paths"))?;

        Ok(())
    }
}

#[derive(FromRow)]
struct TagKeysetRow {
    id: Uuid,
    category: i16,
    name: String,
    count: i32,
    score: f64,
}

#[derive(FromRow)]
struct TagRelationKeysetRow {
    relation_cursor_id: Uuid,
    score: f64,
    parent_id: Uuid,
    parent_name: String,
    parent_count: i32,
    child_id: Uuid,
    child_name: String,
    child_count: i32,
}

#[async_trait]
impl TagRepository for PostgresTagRepository {
    async fn get_or_create(&self, tags: Vec<NewTag>) -> Result<Vec<Tag>, RepoError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| Self::map_db_error(err, "tags.get_or_create failed to begin tx"))?;

        let mut result = Vec::with_capacity(tags.len());
        for new_tag in tags {
            result.push(Self::insert_or_get_tag(&mut tx, new_tag).await?);
        }

        tx.commit()
            .await
            .map_err(|err| Self::map_db_error(err, "tags.get_or_create failed to commit"))?;

        Ok(result)
    }

    async fn search(&self, query: &str, limit: i64) -> Result<Vec<Tag>, RepoError> {
        let pattern = format!("{}%", query.to_lowercase());

        let rows = sqlx::query!(
            "
            SELECT id, category, name, post_count AS \"count!\"
            FROM tags
            WHERE name LIKE $1
            ORDER BY post_count DESC, name ASC
            LIMIT $2
            ",
            pattern,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| Self::map_db_error(err, "tags.search failed"))?;

        Ok(rows
            .into_iter()
            .map(|row| Tag {
                id: row.id,
                category: row.category.into(),
                name: row.name,
                count: row.count,
            })
            .collect())
    }

    async fn list_keyset(&self, cursor: KeysetCursor) -> Result<SearchTagsResponse, RepoError> {
        let limit = Self::resolve_keyset_limit(&cursor);
        let direction = cursor.direction.clone().unwrap_or_default();
        let use_cursor = cursor.last_id.is_some() && cursor.last_score.is_some();

        let rows = match direction {
            KeysetDirection::Next => {
                sqlx::query_as::<_, TagKeysetRow>(
                    r#"
                    SELECT
                        id,
                        category,
                        name,
                        post_count AS count,
                        post_count::DOUBLE PRECISION AS score
                    FROM tags
                    WHERE
                        ($1::DOUBLE PRECISION IS NULL OR $2::UUID IS NULL)
                        OR (
                            post_count::DOUBLE PRECISION < $1
                            OR (post_count::DOUBLE PRECISION = $1 AND id > $2)
                        )
                    ORDER BY post_count DESC, id ASC
                    LIMIT $3
                    "#,
                )
                .bind(cursor.last_score)
                .bind(cursor.last_id)
                .bind(limit + 1)
                .fetch_all(&self.pool)
                .await
            }
            KeysetDirection::Prev => {
                sqlx::query_as::<_, TagKeysetRow>(
                    r#"
                    SELECT
                        id,
                        category,
                        name,
                        post_count AS count,
                        post_count::DOUBLE PRECISION AS score
                    FROM tags
                    WHERE
                        ($1::DOUBLE PRECISION IS NULL OR $2::UUID IS NULL)
                        OR (
                            post_count::DOUBLE PRECISION > $1
                            OR (post_count::DOUBLE PRECISION = $1 AND id < $2)
                        )
                    ORDER BY post_count ASC, id DESC
                    LIMIT $3
                    "#,
                )
                .bind(cursor.last_score)
                .bind(cursor.last_id)
                .bind(limit + 1)
                .fetch_all(&self.pool)
                .await
            }
        }
        .map_err(|err| Self::map_db_error(err, "tags.list_keyset failed"))?;

        let entries = rows
            .into_iter()
            .map(|row| {
                (
                    Tag {
                        id: row.id,
                        category: row.category.into(),
                        name: row.name,
                        count: row.count,
                    },
                    row.score,
                )
            })
            .collect();

        Ok(Self::build_tags_keyset_response(
            entries, limit, direction, use_cursor,
        ))
    }

    async fn get_related(&self, tag_id: TagID) -> Result<Vec<Tag>, RepoError> {
        let rows = sqlx::query!(
            r#"
            WITH alias_seed AS (
                SELECT $1::UUID AS id
                UNION
                SELECT
                    CASE
                        WHEN a.tag_id = $1 THEN a.alias_id
                        ELSE a.tag_id
                    END AS id
                FROM tag_aliases a
                WHERE a.tag_id = $1 OR a.alias_id = $1
            ),
            related_ids AS (
                SELECT c.descendant_id AS id
                FROM tag_relation_closure c
                JOIN alias_seed s ON s.id = c.ancestor_id
                WHERE c.depth > 0
                UNION
                SELECT s.id
                FROM alias_seed s
                WHERE s.id <> $1
            )
            SELECT DISTINCT t.id, t.category, t.name, t.post_count AS "count!"
            FROM tags t
            JOIN related_ids r ON r.id = t.id
            ORDER BY t.post_count DESC, t.name ASC
            "#,
            tag_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| Self::map_db_error(err, "tags.get_related failed"))?;

        Ok(rows
            .into_iter()
            .map(|row| Tag {
                id: row.id,
                category: row.category.into(),
                name: row.name,
                count: row.count,
            })
            .collect())
    }

    async fn list_relations_keyset(
        &self,
        cursor: KeysetCursor,
    ) -> Result<SearchTagRelationsResponse, RepoError> {
        let limit = Self::resolve_keyset_limit(&cursor);
        let direction = cursor.direction.clone().unwrap_or_default();
        let use_cursor = cursor.last_id.is_some() && cursor.last_score.is_some();

        let rows = match direction {
            KeysetDirection::Next => {
                sqlx::query_as::<_, TagRelationKeysetRow>(
                    r#"
                    WITH relations AS (
                        SELECT
                            (
                                SUBSTR(md5(e.parent_id::TEXT || ':' || e.child_id::TEXT), 1, 8) || '-' ||
                                SUBSTR(md5(e.parent_id::TEXT || ':' || e.child_id::TEXT), 9, 4) || '-' ||
                                SUBSTR(md5(e.parent_id::TEXT || ':' || e.child_id::TEXT), 13, 4) || '-' ||
                                SUBSTR(md5(e.parent_id::TEXT || ':' || e.child_id::TEXT), 17, 4) || '-' ||
                                SUBSTR(md5(e.parent_id::TEXT || ':' || e.child_id::TEXT), 21, 12)
                            )::UUID AS relation_cursor_id,
                            (p.post_count + c.post_count)::DOUBLE PRECISION AS score,
                            e.parent_id,
                            p.name AS parent_name,
                            p.post_count AS parent_count,
                            e.child_id,
                            c.name AS child_name,
                            c.post_count AS child_count
                        FROM tag_relation_edges e
                        JOIN tags p ON p.id = e.parent_id
                        JOIN tags c ON c.id = e.child_id
                    )
                    SELECT *
                    FROM relations
                    WHERE
                        ($1::DOUBLE PRECISION IS NULL OR $2::UUID IS NULL)
                        OR (
                            score < $1
                            OR (score = $1 AND relation_cursor_id > $2)
                        )
                    ORDER BY score DESC, relation_cursor_id ASC
                    LIMIT $3
                    "#,
                )
                .bind(cursor.last_score)
                .bind(cursor.last_id)
                .bind(limit + 1)
                .fetch_all(&self.pool)
                .await
            }
            KeysetDirection::Prev => {
                sqlx::query_as::<_, TagRelationKeysetRow>(
                    r#"
                    WITH relations AS (
                        SELECT
                            (
                                SUBSTR(md5(e.parent_id::TEXT || ':' || e.child_id::TEXT), 1, 8) || '-' ||
                                SUBSTR(md5(e.parent_id::TEXT || ':' || e.child_id::TEXT), 9, 4) || '-' ||
                                SUBSTR(md5(e.parent_id::TEXT || ':' || e.child_id::TEXT), 13, 4) || '-' ||
                                SUBSTR(md5(e.parent_id::TEXT || ':' || e.child_id::TEXT), 17, 4) || '-' ||
                                SUBSTR(md5(e.parent_id::TEXT || ':' || e.child_id::TEXT), 21, 12)
                            )::UUID AS relation_cursor_id,
                            (p.post_count + c.post_count)::DOUBLE PRECISION AS score,
                            e.parent_id,
                            p.name AS parent_name,
                            p.post_count AS parent_count,
                            e.child_id,
                            c.name AS child_name,
                            c.post_count AS child_count
                        FROM tag_relation_edges e
                        JOIN tags p ON p.id = e.parent_id
                        JOIN tags c ON c.id = e.child_id
                    )
                    SELECT *
                    FROM relations
                    WHERE
                        ($1::DOUBLE PRECISION IS NULL OR $2::UUID IS NULL)
                        OR (
                            score > $1
                            OR (score = $1 AND relation_cursor_id < $2)
                        )
                    ORDER BY score ASC, relation_cursor_id DESC
                    LIMIT $3
                    "#,
                )
                .bind(cursor.last_score)
                .bind(cursor.last_id)
                .bind(limit + 1)
                .fetch_all(&self.pool)
                .await
            }
        }
        .map_err(|err| Self::map_db_error(err, "tags.list_relations_keyset failed"))?;

        let entries = rows
            .into_iter()
            .map(|row| {
                (
                    TagRelation {
                        id: row.relation_cursor_id,
                        parent_id: row.parent_id,
                        parent_name: row.parent_name,
                        parent_count: row.parent_count,
                        child_id: row.child_id,
                        child_name: row.child_name,
                        child_count: row.child_count,
                        score: row.score as i32,
                    },
                    row.score,
                )
            })
            .collect();

        Ok(Self::build_relations_keyset_response(
            entries, limit, direction, use_cursor,
        ))
    }

    async fn update_tags(&self, update: TagBatchUpdate) -> Result<(), RepoError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| Self::map_db_error(err, "tags.update failed to begin tx"))?;

        let mut should_rebuild_closure = false;

        for event in update.events {
            match event {
                TagUpdateEvent::Create { tags } => {
                    for new_tag in tags {
                        Self::insert_or_get_tag(&mut tx, new_tag).await?;
                    }
                }
                TagUpdateEvent::Edit { tags } => {
                    for tag in tags {
                        let rows_affected = sqlx::query!(
                            "UPDATE tags
                             SET category = $2, name = $3
                             WHERE id = $1",
                            tag.id,
                            tag.category as i16,
                            tag.name
                        )
                        .execute(tx.as_mut())
                        .await
                        .map_err(|err| Self::map_db_error(err, "tags.update failed to edit tag"))?
                        .rows_affected();

                        if rows_affected == 0 {
                            return Err(RepoError::NotFound);
                        }
                    }
                }
                TagUpdateEvent::Remove { tag_ids } => {
                    if tag_ids.is_empty() {
                        continue;
                    }

                    let rows_affected =
                        sqlx::query!("DELETE FROM tags WHERE id = ANY($1)", &tag_ids[..])
                            .execute(tx.as_mut())
                            .await
                            .map_err(|err| {
                                Self::map_db_error(err, "tags.update failed to remove tags")
                            })?
                            .rows_affected();

                    if rows_affected > 0 {
                        should_rebuild_closure = true;
                    }
                }
            }
        }

        if should_rebuild_closure {
            Self::rebuild_closure(&mut tx).await?;
        }

        tx.commit()
            .await
            .map_err(|err| Self::map_db_error(err, "tags.update failed to commit"))?;

        Ok(())
    }

    async fn update_relations(&self, update: TagRelationsBatchUpdate) -> Result<(), RepoError> {
        let mut tx =
            self.pool.begin().await.map_err(|err| {
                Self::map_db_error(err, "tags.update_relations failed to begin tx")
            })?;

        let mut should_rebuild_closure = false;

        for event in update.events {
            match event {
                TagRelationUpdateEvent::Link {
                    parent_id,
                    child_ids,
                } => {
                    Self::ensure_tag_exists(&mut tx, parent_id).await?;

                    for child_id in child_ids {
                        if parent_id == child_id {
                            return Err(RepoError::Conflict);
                        }

                        Self::ensure_tag_exists(&mut tx, child_id).await?;

                        let creates_cycle = sqlx::query_scalar!(
                            r#"
                            WITH RECURSIVE reachable AS (
                                SELECT $1::UUID AS id
                                UNION
                                SELECT e.child_id
                                FROM tag_relation_edges e
                                JOIN reachable r
                                    ON e.parent_id = r.id
                            )
                            SELECT 1::BIGINT
                            FROM reachable
                            WHERE id = $2
                            LIMIT 1
                            "#,
                            child_id,
                            parent_id
                        )
                        .fetch_optional(tx.as_mut())
                        .await
                        .map_err(|err| {
                            Self::map_db_error(err, "tags.update_relations failed on cycle check")
                        })?
                        .is_some();

                        if creates_cycle {
                            return Err(RepoError::Conflict);
                        }

                        let rows_affected = sqlx::query!(
                            "INSERT INTO tag_relation_edges (parent_id, child_id)
                             VALUES ($1, $2)
                             ON CONFLICT (parent_id, child_id) DO NOTHING",
                            parent_id,
                            child_id
                        )
                        .execute(tx.as_mut())
                        .await
                        .map_err(|err| {
                            Self::map_db_error(err, "tags.update_relations failed to add edge")
                        })?
                        .rows_affected();

                        if rows_affected > 0 {
                            should_rebuild_closure = true;
                        }
                    }
                }
                TagRelationUpdateEvent::Unlink {
                    parent_id,
                    child_ids,
                } => {
                    if child_ids.is_empty() {
                        continue;
                    }

                    let rows_affected = sqlx::query!(
                        "DELETE FROM tag_relation_edges
                         WHERE parent_id = $1 AND child_id = ANY($2)",
                        parent_id,
                        &child_ids[..]
                    )
                    .execute(tx.as_mut())
                    .await
                    .map_err(|err| {
                        Self::map_db_error(err, "tags.update_relations failed to remove edges")
                    })?
                    .rows_affected();

                    if rows_affected > 0 {
                        should_rebuild_closure = true;
                    }
                }
                TagRelationUpdateEvent::Alias { tag_id, alias_ids } => {
                    Self::ensure_tag_exists(&mut tx, tag_id).await?;

                    for alias_id in alias_ids {
                        if tag_id == alias_id {
                            continue;
                        }
                        Self::ensure_tag_exists(&mut tx, alias_id).await?;

                        let (left, right) = Self::ordered_pair(tag_id, alias_id);

                        sqlx::query!(
                            "INSERT INTO tag_aliases (tag_id, alias_id)
                             VALUES ($1, $2)
                             ON CONFLICT (tag_id, alias_id) DO NOTHING",
                            left,
                            right
                        )
                        .execute(tx.as_mut())
                        .await
                        .map_err(|err| {
                            Self::map_db_error(err, "tags.update_relations failed to add alias")
                        })?;
                    }
                }
                TagRelationUpdateEvent::Unalias { tag_id, alias_ids } => {
                    for alias_id in alias_ids {
                        if tag_id == alias_id {
                            continue;
                        }

                        let (left, right) = Self::ordered_pair(tag_id, alias_id);

                        sqlx::query!(
                            "DELETE FROM tag_aliases WHERE tag_id = $1 AND alias_id = $2",
                            left,
                            right
                        )
                        .execute(tx.as_mut())
                        .await
                        .map_err(|err| {
                            Self::map_db_error(err, "tags.update_relations failed to remove alias")
                        })?;
                    }
                }
            }
        }

        if should_rebuild_closure {
            Self::rebuild_closure(&mut tx).await?;
        }

        tx.commit()
            .await
            .map_err(|err| Self::map_db_error(err, "tags.update_relations failed to commit"))?;

        Ok(())
    }
}
