use crate::application::ports::FileRepository;
use crate::domain::model::{File, FileID, FileMeta, FileStatus, FileType, RepoError};
use crate::storage::postgres::dto::FileResponse;
use async_trait::async_trait;
use sqlx::PgPool;

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
        let response = sqlx::query_as::<_, FileResponse>(
            r#"
                SELECT id,
                       path,
                       hash,
                       media_type,
                       status,
                       meta,
                       created_at
                FROM files
                WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => RepoError::NotFound,
            _ => {
                log::error!("files.get db query failed: {err}");
                RepoError::StorageError
            }
        })?;

        Ok(response.into())
    }

    async fn mark_ready(
        &self,
        id: FileID,
        path: &str,
        media_type: FileType,
        meta: Option<FileMeta>,
    ) -> Result<(), RepoError> {
        let meta_value = serde_json::to_value(meta).map_err(|err| {
            log::error!("files.mark_ready failed to serialize file meta: {err}");
            RepoError::StorageError
        })?;

        let result = sqlx::query(
            r#"
                UPDATE files
                SET path = $2,
                    media_type = $3,
                    meta = $4,
                    status = $5
                WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(path)
        .bind(i16::from(media_type))
        .bind(meta_value)
        .bind(i16::from(FileStatus::Ready))
        .execute(&self.pool)
        .await
        .map_err(|err| {
            log::error!("files.mark_ready db query failed: {err}");
            RepoError::StorageError
        })?;

        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }

        Ok(())
    }

    async fn mark_failed(&self, id: FileID) -> Result<(), RepoError> {
        self.set_status(id, FileStatus::Failed).await
    }

    async fn set_status(&self, id: FileID, status: FileStatus) -> Result<(), RepoError> {
        let result = sqlx::query(
            r#"
                UPDATE files
                SET status = $2
                WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(i16::from(status))
        .execute(&self.pool)
        .await
        .map_err(|err| {
            log::error!("files.set_status db query failed: {err}");
            RepoError::StorageError
        })?;

        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }

        Ok(())
    }
}
