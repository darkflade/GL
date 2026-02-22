use std::path::{PathBuf};
use actix_web::web::Bytes;
use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use crate::domain::files::FileStorage;
use crate::domain::model::{FileID, RelativePath, StorageError};

#[derive(Clone)]
pub struct LocalFileStorage {
    root: PathBuf,
}

impl LocalFileStorage {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self { root: root.into() }
    }
/*
    fn build_path(&self, id: &str, ext: Option<&str>) -> PathBuf {
        let mut p = self.root.join(id);
        if let Some(ext) = ext {
            p.set_extension(ext);
        }
        p
    }
*/
    fn generate_rel_path(&self, id: Uuid, ext: Option<&str>) -> PathBuf {
        let uuid_str = id.to_string();
        let p1 = &uuid_str[0..2];
        let p2 = &uuid_str[2..4];

        let mut path = PathBuf::new();
        path.push(p1);
        path.push(p2);

        if let Some(e) = ext {
            path.push(format!("{}.{}", id, e));
        } else {
            path.push(id.to_string());
        }
        path
    }
}

#[async_trait]
impl FileStorage for LocalFileStorage {
    async fn save_stream<S>(
        &self,
        mut stream: S,
        ext: Option<&str>
    ) -> Result<(FileID, RelativePath), StorageError>
    where
        S: Stream<Item = Result<Bytes, StorageError>> + Unpin + Send,
    {

        let id = Uuid::now_v7();
        let relative_path_buf = self.generate_rel_path(id, ext);
        let full_destination_path = self.root.join(&relative_path_buf);

        if let Some(parent) = full_destination_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|_| StorageError::Io)?;
        }

        let mut file = fs::File::create(&full_destination_path)
            .await
            .map_err(|_| StorageError::Io)?;

        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(|_| StorageError::Io)?;
            file.write_all(&bytes)
                .await
                .map_err(|_| StorageError::Io)?;
        }

        let relative_path_string = relative_path_buf.to_string_lossy().to_string();
        Ok((id, relative_path_string))
    }

    async fn save_temp_file(&self, temp_path: PathBuf, ext: Option<&str>) -> Result<(FileID, RelativePath), StorageError> {
        let id = Uuid::now_v7();
        let relative_path_buf = self.generate_rel_path(id, ext);

        let full_destination_path = self.root.join(&relative_path_buf);

        if let Some(parent) = full_destination_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|_| StorageError::Io)?;
        }

        fs::rename(temp_path, full_destination_path)
            .await
            .map_err(|_| StorageError::Io)?;
        
        let relative_path_string = relative_path_buf.to_string_lossy().to_string();
        
        Ok((id, relative_path_string))
    }
}
