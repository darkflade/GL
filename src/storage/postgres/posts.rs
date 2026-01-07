use async_trait::async_trait;
use sqlx::PgPool;
use crate::domain::model::{NewPost, Post, PostID, RepoError, TagID, TagQuery};
use crate::domain::repository::PostRepository;

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
            SELECT p.id, p.title, p.file_id
            FROM posts p
            WHERE p.id = $1
            "#,
            id
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| RepoError::StorageError)?;

        let row = row.ok_or(RepoError::NotFound)?;

        let tags = sqlx::query!(
            r#"
            SELECT tag_id
            FROM post_tags
            WHERE post_id = $1
            "#,
            id
        )
            .fetch_all(&self.pool)
            .await
            .map_err(|_| RepoError::StorageError)?;

        Ok(Post {
            id: row.id,
            title: row.title,
            file_id: row.file_id,
            tag_ids: tags.into_iter().map(|r| r.tag_id).collect(),
            notes: None,
        })
    }

    async fn search(&self, query: TagQuery) -> Result<Vec<Post>, RepoError> {
        let rows = sqlx::query!(
            r#"
            SELECT p.id, p.title, p.file_id
            FROM posts p
            JOIN post_tags pt ON pt.post_id = p.id
            WHERE
                ($1::uuid[] IS NULL OR pt.tag_id = ANY($1))
            GROUP BY p.id
            HAVING
                COUNT(DISTINCT CASE WHEN pt.tag_id = ANY($2) THEN pt.tag_id END) = cardinality($2)
            "#,
            &query.should[..],
            &query.must[..]
        )
            .fetch_all(&self.pool)
            .await
            .map_err(|_| RepoError::StorageError)?;

        Ok(rows.into_iter().map(|r| Post {
            id: r.id,
            title: r.title,
            file_id: r.file_id,
            tag_ids: vec![],
            notes: None,
        }).collect())
    }

}