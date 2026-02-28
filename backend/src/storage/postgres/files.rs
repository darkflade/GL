use crate::application::ports::FileRepository;
use crate::domain::model::File;
use crate::domain::model::FileID;
use crate::domain::model::RepoError;
use crate::storage::postgres::dto::FileMetaResponse;
use crate::storage::postgres::dto::FileResponse;
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::types::Json;

#[derive(Clone)]
pub struct PostgresFileRepository {
    pool: PgPool,
}

impl PostgresFileRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FileRepository for PostgresFileRepository {
    async fn create(&self, file: File) -> Result<FileID, RepoError> {
        let file_meta_json = serde_json::to_value(file.meta).map_err(|err| {
            log::error!("files.create failed to serialize file meta: {err}");
            RepoError::StorageError
        })?;

        sqlx::query!(
            r#"
                INSERT INTO files (id, path, hash, media_type, meta)
                VALUES ($1, $2, $3, $4, $5)
            "#,
            file.id,
            file.path.to_string_lossy().to_string(),
            file.hash,
            file.media_type as i16,
            file_meta_json
        )
        .execute(&self.pool)
        .await
        .map_err(|err| {
            log::error!("files.create db query failed: {err}");
            RepoError::StorageError
        })?;

        Ok(file.id)
    }

    async fn get(&self, id: FileID) -> Result<File, RepoError> {
        let response = sqlx::query_as!(
            FileResponse,
            r#"
                SELECT id,
                       path,
                       hash,
                       media_type,
                       meta as "meta: Json<FileMetaResponse>",
                       created_at
                FROM files
                WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            log::error!("files.get db query failed: {e}");
            RepoError::StorageError
        })?;

        File::try_from(response).map_err(|_| RepoError::StorageError)
    }
}
