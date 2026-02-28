use crate::domain::model::{FileID, RelativePath, StorageError};
use actix_web::web::Bytes;
use async_trait::async_trait;
use futures_util::Stream;
use std::path::PathBuf;

#[async_trait]
pub trait FileStorage {
    async fn save_stream<S>(
        &self,
        stream: S,
        ext: Option<&str>,
    ) -> Result<(FileID, RelativePath), StorageError>
    where
        S: Stream<Item = Result<Bytes, StorageError>> + Unpin + Send;

    async fn save_temp_file(
        &self,
        temp_path: PathBuf,
        ext: Option<&str>,
    ) -> Result<(FileID, RelativePath), StorageError>;
}
