use crate::domain::model::{NewUser, RepoError, User};
use crate::domain::repository::UserRepository;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<User, RepoError> {
        let user = sqlx::query_as!(
            User,
            "SELECT id, username, password_hash FROM users WHERE id = $1",
            id,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| RepoError::NotFound)?;

        Ok(user)
    }
    async fn find_by_username(&self, username: &str) -> Result<User, RepoError> {
        let user = sqlx::query_as!(
            User,
            "SELECT id, username, password_hash FROM users WHERE username = $1",
            username,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| RepoError::NotFound)?;

        Ok(user)
    }

    async fn create(&self, user: NewUser) -> Result<Uuid, RepoError> {
        let id = Uuid::now_v7();
        sqlx::query!(
            "INSERT INTO users (id, username, password_hash) VALUES ($1, $2, $3)",
            id,
            user.username,
            user.password_hash,
        )
        .execute(&self.pool)
        .await
        .map_err(|_| RepoError::StorageError)?;

        Ok(id)
    }
}
