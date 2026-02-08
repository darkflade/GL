use actix_web::mime::Mime;
use crate::domain::model::{FileType, RepoError};

pub fn file_type_from_mime_and_ext(mime: Option<Mime>, ext: Option<&str>) -> Result<FileType, RepoError> {

    //TODO split to business logic type converter
    if let Some(mime) = mime {
        let essence = mime.essence_str();

         return match essence {
            e if e.starts_with("image/") => Ok(FileType::Picture),
            e if e.starts_with("video/") => Ok(FileType::Video),
            e if e.starts_with("audio/") => Ok(FileType::Audio),
            _ => Err(RepoError::StorageError),
        }
    }

    //TODO make errors talk
    let ext = ext.ok_or(RepoError::StorageError)?;

    match ext.to_lowercase().as_str() {
        "png" | "jpg" | "jpeg" | "webp" | "gif" => Ok(FileType::Picture),
        "mp4" | "mkv" | "webm" | "avi" | "mov" => Ok(FileType::Video),
        "mp3" | "wav" | "ogg" | "flac" => Ok(FileType::Audio),
        //TODO make errors talk
        _ => Err(RepoError::StorageError),
    }

}