use crate::domain::model::{
    Cursor, KeysetCursor, NewPost, NextKeysetCursor, PaginationMode, Post, PostID, RepoError,
    SearchPostsKeysetResponse, SearchPostsOffsetResponse, Tag, TagID, TagQuery,
};
use crate::domain::repository::PostRepository;
use crate::storage::postgres::dto::{FileResponse, TagResponse};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::types::Json;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresPostRepository {
    pool: PgPool,
}

impl PostgresPostRepository {
    const OFFSET_LIMIT: i64 = 20;
    const DEFAULT_KEYSET_LIMIT: i64 = 30;
    const MAX_KEYSET_LIMIT: i64 = 100;

    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn build_keyset_response(
        mut entries: Vec<(Post, f64)>,
        limit: i64,
    ) -> SearchPostsKeysetResponse {
        let has_next = entries.len() as i64 > limit;
        if has_next {
            entries.truncate(limit as usize);
        }

        let next_cursor = if has_next {
            entries.last().map(|(post, score)| NextKeysetCursor {
                mode: PaginationMode::Keyset,
                last_id: post.id,
                last_score: *score,
                limit,
            })
        } else {
            None
        };

        let posts = entries.into_iter().map(|(post, _)| post).collect();

        SearchPostsKeysetResponse {
            posts,
            has_next,
            next_cursor,
        }
    }

    fn resolve_keyset_limit(cursor: &KeysetCursor) -> i64 {
        cursor
            .limit
            .unwrap_or(Self::DEFAULT_KEYSET_LIMIT)
            .clamp(1, Self::MAX_KEYSET_LIMIT)
    }

    fn count_total_pages(full_count: i64, limit: i64) -> i64 {
        if full_count == 0 {
            0
        } else {
            (full_count + limit - 1) / limit
        }
    }
}

#[async_trait]
impl PostRepository for PostgresPostRepository {
    async fn create(&self, post: NewPost, tag_ids: &[TagID]) -> Result<PostID, RepoError> {
        let mut tx = self.pool.begin().await.map_err(|err| {
            log::error!("posts.create failed to begin transaction: {err}");
            RepoError::StorageError
        })?;

        sqlx::query!(
            "INSERT INTO posts (id, title, file_id) VALUES ($1, $2, $3)",
            post.id,
            post.title,
            post.file_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|err| {
            log::error!("posts.create failed to insert post {}: {err}", post.id);
            RepoError::StorageError
        })?;

        for tag_id in tag_ids {
            sqlx::query!(
                "INSERT INTO post_tags (post_id, tag_id) VALUES ($1, $2)",
                post.id,
                tag_id
            )
            .execute(&mut *tx)
            .await
            .map_err(|err| {
                log::error!(
                    "posts.create failed to attach tag {} to post {}: {err}",
                    tag_id,
                    post.id
                );
                RepoError::StorageError
            })?;
        }

        tx.commit().await.map_err(|err| {
            log::error!(
                "posts.create failed to commit transaction for post {}: {err}",
                post.id
            );
            RepoError::StorageError
        })?;

        Ok(post.id)
    }

    async fn get(&self, id: PostID) -> Result<Post, RepoError> {
        let row = sqlx::query!(
            r#"
            SELECT
                p.id,
                p.title,
                p.description,
                COALESCE(
                    jsonb_agg(
                        jsonb_build_object(
                            'id', pt.tag_id,
                            'name', t.name,
                            'category', t.category,
                            'count', t.post_count
                        )
                    ) FILTER (WHERE pt.tag_id IS NOT NULL),
                '[]'::jsonb
                ) AS "tags!: Json<Vec<TagResponse>>",
                 (
                    SELECT jsonb_build_object(
                        'id', f.id,
                        'hash', f.hash,
                        'media_type', f.media_type,
                        'meta', f.meta,
                        'path', f.path,
                        'created_at', f.created_at
                    )
                    FROM files f
                    WHERE f.id = p.file_id
                ) AS "file!: Json<FileResponse>"
            FROM posts p
            LEFT JOIN post_tags pt ON pt.post_id = p.id
            LEFT JOIN tags t ON t.id = pt.tag_id
            LEFT JOIN files f ON f.id = p.file_id
            WHERE p.id = $1
            GROUP BY p.id
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            log::error!("posts.get db query failed: {e}");
            RepoError::StorageError
        })?;

        let row = row.ok_or(RepoError::NotFound)?;

        Ok(Post {
            id: row.id,
            title: row.title,
            description: row.description,
            file: row.file.0.into(),
            tags: row.tags.0.into_iter().map(Tag::from).collect(),
            //TODO load notes
            notes: vec![],
        })
    }

    async fn search(
        &self,
        query: TagQuery,
        cursor: Cursor,
    ) -> Result<SearchPostsOffsetResponse, RepoError> {
        let limit = Self::OFFSET_LIMIT;
        let page = cursor.page.max(0);

        let rows = sqlx::query!(
            r#"
            SELECT
                p.id,
                p.title,
                p.description,
                COUNT(*) OVER() as "full_count!",
                COALESCE(
                    jsonb_agg(
                        jsonb_build_object(
                            'id', pt.tag_id,
                            'name', t.name,
                            'category', t.category,
                            'count', t.post_count
                        )
                    ) FILTER (WHERE pt.tag_id IS NOT NULL),
                '[]'::jsonb
                ) AS "tags!: Json<Vec<TagResponse>>",
                (
                    SELECT jsonb_build_object(
                        'id', f.id,
                        'path', f.path,
                        'hash', f.hash,
                        'media_type', f.media_type,
                        'meta', f.meta,
                        'created_at', f.created_at
                    )
                    FROM files f
                    WHERE f.id = p.file_id
                ) AS "file!: Json<FileResponse>",

                COUNT(DISTINCT CASE
                    WHEN t.name = ANY($1) THEN t.name
                END) AS should_score

            FROM posts p
            LEFT JOIN post_tags pt ON pt.post_id = p.id
            LEFT JOIN tags t ON t.id = pt.tag_id
            LEFT JOIN files f ON f.id = p.file_id

            GROUP BY p.id


            HAVING
                COUNT(DISTINCT CASE
                    WHEN t.name = ANY($2) THEN t.name
                END) = cardinality($2)
                AND
                NOT EXISTS (
                    SELECT 1
                    FROM post_tags x
                    JOIN tags tx ON tx.id = x.tag_id
                    WHERE x.post_id = p.id
                      AND tx.name = ANY($3)
                )

            ORDER BY should_score DESC, p.id DESC

            LIMIT $4
            OFFSET $5
            "#,
            &query.should[..],
            &query.must[..],
            &query.must_not[..],
            limit,
            page * limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            log::error!("posts.search db query failed: {e}");
            RepoError::StorageError
        })?;

        let full_count = rows.first().map(|row| row.full_count).unwrap_or(0);
        let page_count = Self::count_total_pages(full_count, limit);

        Ok(SearchPostsOffsetResponse {
            posts: rows
                .into_iter()
                .map(|r| Post {
                    id: r.id,
                    title: r.title,
                    description: r.description,
                    file: r.file.0.into(),
                    tags: r.tags.0.into_iter().map(Tag::from).collect(),
                    notes: vec![],
                })
                .collect(),
            total_pages: page_count,
        })
    }

    async fn get_all(&self, cursor: Cursor) -> Result<SearchPostsOffsetResponse, RepoError> {
        let limit = Self::OFFSET_LIMIT;
        let page = cursor.page.max(0);

        let rows = sqlx::query!(
            r#"
            SELECT
                p.id,
                p.title,
                p.description,
                COUNT(*) OVER() as "full_count!",
                COALESCE(
                    jsonb_agg(
                        jsonb_build_object(
                            'id', pt.tag_id,
                            'name', t.name,
                            'category', t.category,
                            'count', t.post_count
                        )
                    ) FILTER (WHERE pt.tag_id IS NOT NULL),
                '[]'::jsonb
                ) AS "tags!: Json<Vec<TagResponse>>",
                (
                    SELECT jsonb_build_object(
                        'id', f.id,
                        'path', f.path,
                        'hash', f.hash,
                        'media_type', f.media_type,
                        'meta', f.meta,
                        'created_at', f.created_at
                    )
                    FROM files f
                    WHERE f.id = p.file_id
                ) AS "file!: Json<FileResponse>"
            FROM posts p
            LEFT JOIN post_tags pt ON pt.post_id = p.id
            LEFT JOIN tags t ON t.id = pt.tag_id
            LEFT JOIN files f ON f.id = p.file_id
            GROUP BY p.id
            ORDER BY p.id DESC

            LIMIT $1
            OFFSET $2
            "#,
            limit,
            page * limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            log::error!("posts.get_all db query failed: {e}");
            RepoError::StorageError
        })?;

        let full_count = rows.first().map(|row| row.full_count).unwrap_or(0);
        let page_count = Self::count_total_pages(full_count, limit);

        Ok(SearchPostsOffsetResponse {
            posts: rows
                .into_iter()
                .map(|r| Post {
                    id: r.id,
                    title: r.title,
                    file: r.file.0.into(),
                    description: r.description,
                    tags: r.tags.0.into_iter().map(Tag::from).collect(),
                    notes: vec![],
                })
                .collect(),
            total_pages: page_count,
        })
    }

    async fn search_keyset(
        &self,
        query: TagQuery,
        cursor: KeysetCursor,
    ) -> Result<SearchPostsKeysetResponse, RepoError> {
        let limit = Self::resolve_keyset_limit(&cursor);
        let query_limit = limit + 1;
        let use_cursor = cursor.last_id.is_some() && cursor.last_score.is_some();
        let last_id = cursor.last_id.unwrap_or(Uuid::nil());
        let last_score = cursor.last_score.unwrap_or(f64::MAX);

        let rows = sqlx::query!(
            r#"
            WITH ranked_posts AS (
                SELECT
                    p.id,
                    p.title,
                    p.description,
                    COALESCE(
                        jsonb_agg(
                            jsonb_build_object(
                                'id', pt.tag_id,
                                'name', t.name,
                                'category', t.category,
                                'count', t.post_count
                            )
                        ) FILTER (WHERE pt.tag_id IS NOT NULL),
                        '[]'::jsonb
                    ) AS tags,
                    (
                        SELECT jsonb_build_object(
                            'id', f.id,
                            'path', f.path,
                            'hash', f.hash,
                            'media_type', f.media_type,
                            'meta', f.meta,
                            'created_at', f.created_at
                        )
                        FROM files f
                        WHERE f.id = p.file_id
                    ) AS file,
                    COUNT(DISTINCT CASE
                        WHEN t.name = ANY($1) THEN t.name
                    END)::bigint AS should_score
                FROM posts p
                LEFT JOIN post_tags pt ON pt.post_id = p.id
                LEFT JOIN tags t ON t.id = pt.tag_id
                LEFT JOIN files f ON f.id = p.file_id
                GROUP BY p.id
                HAVING
                    COUNT(DISTINCT CASE
                        WHEN t.name = ANY($2) THEN t.name
                    END) = cardinality($2)
                    AND
                    NOT EXISTS (
                        SELECT 1
                        FROM post_tags x
                        JOIN tags tx ON tx.id = x.tag_id
                        WHERE x.post_id = p.id
                          AND tx.name = ANY($3)
                    )
            )
            SELECT
                id,
                title,
                description,
                tags AS "tags!: Json<Vec<TagResponse>>",
                file AS "file!: Json<FileResponse>",
                should_score AS "should_score!: i64"
            FROM ranked_posts
            WHERE
                $4 = false
                OR should_score::double precision < $5
                OR (should_score::double precision = $5 AND id < $6)
            ORDER BY should_score DESC, id DESC
            LIMIT $7
            "#,
            &query.should[..],
            &query.must[..],
            &query.must_not[..],
            use_cursor,
            last_score,
            last_id,
            query_limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| {
            log::error!("posts.search_keyset db query failed: {err}");
            RepoError::StorageError
        })?;

        let parsed_rows = rows
            .into_iter()
            .map(|row| {
                (
                    Post {
                        id: row.id,
                        title: row.title,
                        description: row.description,
                        file: row.file.0.into(),
                        tags: row.tags.0.into_iter().map(Tag::from).collect(),
                        notes: vec![],
                    },
                    row.should_score as f64,
                )
            })
            .collect();

        Ok(Self::build_keyset_response(parsed_rows, limit))
    }

    async fn get_all_keyset(
        &self,
        cursor: KeysetCursor,
    ) -> Result<SearchPostsKeysetResponse, RepoError> {
        let limit = Self::resolve_keyset_limit(&cursor);
        let query_limit = limit + 1;
        let use_cursor = cursor.last_id.is_some();
        let last_id = cursor.last_id.unwrap_or(Uuid::nil());

        let rows = sqlx::query!(
            r#"
            SELECT
                p.id,
                p.title,
                p.description,
                COALESCE(
                    jsonb_agg(
                        jsonb_build_object(
                            'id', pt.tag_id,
                            'name', t.name,
                            'category', t.category,
                            'count', t.post_count
                        )
                    ) FILTER (WHERE pt.tag_id IS NOT NULL),
                    '[]'::jsonb
                ) AS "tags!: Json<Vec<TagResponse>>",
                (
                    SELECT jsonb_build_object(
                        'id', f.id,
                        'path', f.path,
                        'hash', f.hash,
                        'media_type', f.media_type,
                        'meta', f.meta,
                        'created_at', f.created_at
                    )
                    FROM files f
                    WHERE f.id = p.file_id
                ) AS "file!: Json<FileResponse>",
                0::bigint AS "should_score!: i64"
            FROM posts p
            LEFT JOIN post_tags pt ON pt.post_id = p.id
            LEFT JOIN tags t ON t.id = pt.tag_id
            LEFT JOIN files f ON f.id = p.file_id
            WHERE $1 = false OR p.id < $2
            GROUP BY p.id
            ORDER BY p.id DESC
            LIMIT $3
            "#,
            use_cursor,
            last_id,
            query_limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| {
            log::error!("posts.get_all_keyset db query failed: {err}");
            RepoError::StorageError
        })?;

        let parsed_rows = rows
            .into_iter()
            .map(|row| {
                (
                    Post {
                        id: row.id,
                        title: row.title,
                        description: row.description,
                        file: row.file.0.into(),
                        tags: row.tags.0.into_iter().map(Tag::from).collect(),
                        notes: vec![],
                    },
                    row.should_score as f64,
                )
            })
            .collect();

        Ok(Self::build_keyset_response(parsed_rows, limit))
    }
}
