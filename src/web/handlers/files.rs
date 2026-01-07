use actix_files::NamedFile;
use actix_web::{web, Error};
use crate::domain::files::FileStorage;
use futures_util::{TryStreamExt};
use uuid::Uuid;

pub async fn download_file<FS: FileStorage + Clone>(
    files: web::Data<FS>,
    path: web::Path<String>,
) -> Result<NamedFile, Error> {
    let file_id = path.into_inner();

    let uuid = Uuid::parse_str(&file_id).map_err(|_| actix_web::error::ErrorBadRequest("Invalid UUID"))?;

    let file_path = files.get(uuid).await.map_err(|_| {
        actix_web::error::ErrorNotFound("File not found")
    })?;

    Ok(NamedFile::open(file_path)?)
}
