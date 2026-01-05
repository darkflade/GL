use std::path::{PathBuf};
use async_trait::async_trait;
use crate::domain::model::{FileID, StorageError};

#[async_trait]
pub trait FileStorage {
    async fn save(&self, bytes: &[u8], ext: Option<&str>) -> Result<FileID, StorageError>;
    async fn get(&self, id: FileID) -> Result<PathBuf, StorageError>;
}