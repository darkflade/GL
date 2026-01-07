use std::path::{PathBuf};
use async_trait::async_trait;
use tokio::fs;
use uuid::Uuid;
use crate::domain::files::FileStorage;
use crate::domain::model::{FileID, StorageError};

#[derive(Clone)]
pub struct LocalFileStorage {
    root: PathBuf,
}

impl LocalFileStorage {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self { root: root.into() }
    }

    fn build_path(&self, id: &str, ext: Option<&str>) -> PathBuf {
        let mut p = self.root.join(id);
        if let Some(ext) = ext {
            p.set_extension(ext);
        }
        p
    }
}

#[async_trait]
impl FileStorage for LocalFileStorage {
    async fn save(&self, bytes: &[u8], ext: Option<&str>) -> Result<FileID, StorageError> {
        fs::create_dir_all(&self.root)
            .await
            .map_err(|_| StorageError::Io)?;

        let id = Uuid::new_v4();
        let path = self.build_path(&id.to_string(), ext);

        fs::write(&path, bytes)
            .await
            .map_err(|_| StorageError::Io)?;

        Ok(id)
    }

    async fn get(&self, id: FileID) -> Result<PathBuf, StorageError> {
        let mut entries = fs::read_dir(&self.root)
            .await
            .map_err(|_| StorageError::Io)?;

        while let Some(entry) = entries.next_entry().await.map_err(|_| StorageError::Io)? {
            let path = entry.path();
            if path.file_stem().and_then(|s| s.to_str()) == Some(&id.to_string()) {
                return Ok(path);
            }
        }

        Err(StorageError::NotFound)
    }

    async fn save_temp_file(&self, temp_path: PathBuf, ext: Option<&str>) -> Result<FileID, StorageError> {
        let id = Uuid::new_v4();
        let filename = match ext {
            Some(e) => format!("{}.{}", id, e),
            None => id.to_string(),
        };
        let target_path = self.root.join(filename);

        // Просто перемещаем файл (rename атомарен и мгновенен в пределах одной ФС)
        fs::rename(temp_path, target_path)
            .await
            .map_err(|_| StorageError::Io)?;

        Ok(id)
    }
}

