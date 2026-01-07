use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::model::{NewTag, RepoError, Tag};
use crate::domain::repository::TagRepository;

#[derive(Clone)]
pub struct PostgresTagRepository {
    pool: PgPool,
}

impl PostgresTagRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TagRepository for PostgresTagRepository {

    async fn get_or_create(&self, tags: Vec<NewTag>) -> Result<Vec<Tag>, RepoError> {

        let mut result = Vec::new();
        for new_tag in tags {
            let rec = sqlx::query!(
                "INSERT INTO tags (id, category, value) VALUES ($1, $2, $3)
                 ON CONFLICT (category, value) DO UPDATE SET value = EXCLUDED.value
                 RETURNING id, category, value",
                Uuid::new_v4(),
                new_tag.category as i32,
                new_tag.value
            )
                .fetch_one(&self.pool)
                .await
                .map_err(|_| RepoError::StorageError)?;

            result.push(Tag {
                id: rec.id,
                category: unsafe { std::mem::transmute(rec.category as i8) },
                value: rec.value
            });
        }
        Ok(result)
    }
    async fn search(&self, query: &str, limit: i64) -> Result<Vec<Tag>, RepoError> {
        let pattern =  format!("{}%", query);
        let rows = sqlx::query!(
            "SELECT id, category, value FROM tags WHERE value ILIKE $1 LIMIT $2",
            pattern,
            limit
        )
            .fetch_all(&self.pool)
            .await
            .map_err(|_| RepoError::StorageError)?;

        Ok(rows.into_iter().map(|r| Tag {
            id: r.id,
            category: unsafe { std::mem::transmute(r.category as i8) },
            value: r.value,
        }).collect())
    }
}