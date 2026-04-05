use crate::domain::files::FileStorage;
use crate::domain::model::{FileID, RelativePath, StorageError};
use actix_web::web::Bytes;
use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

#[derive(Clone)]
pub struct LocalFileStorage {
    root: PathBuf,
}

impl LocalFileStorage {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self { root: root.into() }
    }

    fn generate_rel_path(&self, id: Uuid, ext: Option<&str>) -> PathBuf {
        let (p1, p2) = uuid_shards(id);

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

fn uuid_shards(id: Uuid) -> (String, String) {
    let bytes = id.as_bytes();
    (format!("{:02x}", bytes[14]), format!("{:02x}", bytes[15]))
}

#[async_trait]
impl FileStorage for LocalFileStorage {
    async fn save_stream<S>(
        &self,
        mut stream: S,
        ext: Option<&str>,
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
            file.write_all(&bytes).await.map_err(|_| StorageError::Io)?;
        }

        let full_path_string = full_destination_path.to_string_lossy().to_string();
        Ok((id, full_path_string))
    }

    async fn save_temp_file(
        &self,
        temp_path: PathBuf,
        ext: Option<&str>,
    ) -> Result<(FileID, RelativePath), StorageError> {
        let id = Uuid::now_v7();
        let relative_path_buf = self.generate_rel_path(id, ext);

        let full_destination_path = self.root.join(&relative_path_buf);

        if let Some(parent) = full_destination_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|_| StorageError::Io)?;
        }

        fs::rename(temp_path, &full_destination_path)
            .await
            .map_err(|_| StorageError::Io)?;

        let full_path_string = full_destination_path.to_string_lossy().to_string();
        Ok((id, full_path_string))
    }
}
