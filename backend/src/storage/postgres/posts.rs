use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::types::Json;
use dto::TagResponse;
use dto::FileResponse;
use crate::domain::model::{NewPost, Post, PostID, RepoError, Tag, TagID, TagQuery};
use crate::domain::repository::PostRepository;
use crate::storage::postgres::dto;

#[derive(Clone)]
pub struct PostgresPostRepository {
    pool: PgPool,
}

impl PostgresPostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PostRepository for PostgresPostRepository {
    async fn create(
        &self,
        post: NewPost,
        tag_ids: &[TagID],
    ) -> Result<PostID, RepoError> {
        let mut tx = self.pool.begin().await.map_err(|_| RepoError::StorageError)?;

        sqlx::query!(
            "INSERT INTO posts (id, title, file_id) VALUES ($1, $2, $3)",
            post.id,
            post.title,
            post.file_id
        )
            .execute(&mut *tx)
            .await
            .map_err(|_| RepoError::StorageError)?;

        //:Result<PgQueryResult>
        for tag_id in tag_ids {
            sqlx::query!(
            "INSERT INTO post_tags (post_id, tag_id) VALUES ($1, $2)",
            post.id,
            tag_id
        )
                .execute(&mut *tx)
                .await
                .map_err(|_| RepoError::StorageError)?;

        }

        tx.commit().await.map_err(|_| RepoError::StorageError)?;

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
                            'value', t.value,
                            'category', t.category
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
                        'path', f.path
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
            .map_err(|_| RepoError::StorageError)?;

        let row = row.ok_or(RepoError::NotFound)?;


        Ok(
            Post {
                id: row.id,
                title: row.title,
                description: row.description,
                file: row.file.0.into(),
                tags: row.tags.0.into_iter().map(Tag::from).collect(),
                notes: vec![],
            }
        )
    }

    async fn search(&self, query: TagQuery) -> Result<Vec<Post>, RepoError> {
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
                            'value', t.value,
                            'category', t.category
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
                    WHEN t.value = ANY($1) THEN t.value 
                END) AS should_score
        
            FROM posts p
            LEFT JOIN post_tags pt ON pt.post_id = p.id
            LEFT JOIN tags t ON t.id = pt.tag_id
            LEFT JOIN files f ON f.id = p.file_id
        
            GROUP BY p.id

            HAVING
                COUNT(DISTINCT CASE 
                    WHEN t.value = ANY($2) THEN t.value 
                END) = cardinality($2)
                AND
                NOT EXISTS (
                    SELECT 1
                    FROM post_tags x
                    JOIN tags tx ON tx.id = x.tag_id
                    WHERE x.post_id = p.id
                      AND tx.value = ANY($3)
                )
        
            ORDER BY should_score DESC
            LIMIT 50
            "#,
            &query.should[..],
            &query.must[..],
            &query.must_not[..]
        )
            .fetch_all(&self.pool)
            .await
            .map_err(|_| RepoError::StorageError)?;

        Ok(rows.into_iter().map(|r| Post {
            id: r.id,
            title: r.title,
            description: r.description,
            file: r.file.0.into(),
            tags: r.tags.0.into_iter().map(Tag::from).collect(),
            notes: vec![],
        }).collect())
    }

    async fn get_all(&self) -> Result<Vec<Post>, RepoError> {
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
                            'value', t.value,
                            'category', t.category
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
            LIMIT 50
            "#
        )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                println!("{}", e);
                RepoError::StorageError
            })?;


        Ok(rows.into_iter().map(|r| Post {
            id:             r.id,
            title:          r.title,
            file:           r.file.0.into(),
            description:    r.description,
            tags:           r.tags.0.into_iter().map(Tag::from).collect(),
            notes:          vec![],
        }).collect())

    }

}