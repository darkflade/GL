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
                "INSERT INTO tags (id, category, name) VALUES ($1, $2, $3)
                 ON CONFLICT (category, name) DO UPDATE SET name = EXCLUDED.name
                 RETURNING id, category, name, post_count AS count",
                Uuid::now_v7(),
                new_tag.category as i32,
                new_tag.value
            )
                .fetch_one(&self.pool)
                .await
                .map_err(|_| RepoError::StorageError)?;

            result.push(Tag {
                id: rec.id,
                category: rec.category.into(),
                name: rec.name,
                count: rec.count,
            });
        }
        Ok(result)
    }
    async fn search(&self, query: &str, limit: i64) -> Result<Vec<Tag>, RepoError> {
        let pattern =  format!("{}%", query.to_lowercase());
        let rows = sqlx::query!(
            "
            SELECT id, category, name, post_count AS count
            FROM tags
            WHERE name LIKE $1
            ORDER BY count DESC
            LIMIT $2
            ",
            pattern,
            limit
        )
            .fetch_all(&self.pool)
            .await
            .map_err(|_| RepoError::StorageError)?;

        Ok(rows.into_iter().map(|r| Tag {
            id: r.id,
            category: r.category.into(),
            name: r.name,
            count: r.count,
        }).collect())
    }
}