use crate::domain::model::{FileType, RepoError};
use actix_web::mime::Mime;

pub struct FileDetection {
    pub media_type: FileType,
    pub extension: &'static str,
}

pub fn detect_from_magic_bytes(bytes: &[u8]) -> Option<FileDetection> {
    if bytes.len() >= 8 && bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Some(FileDetection {
            media_type: FileType::Picture,
            extension: "png",
        });
    }

    if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
        return Some(FileDetection {
            media_type: FileType::Picture,
            extension: "jpg",
        });
    }

    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some(FileDetection {
            media_type: FileType::Picture,
            extension: "gif",
        });
    }

    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some(FileDetection {
            media_type: FileType::Picture,
            extension: "webp",
        });
    }

    if bytes.len() >= 12 && &bytes[4..8] == b"ftyp" {
        if bytes.get(8..12) == Some(b"qt  ") {
            return Some(FileDetection {
                media_type: FileType::Video,
                extension: "mov",
            });
        }

        return Some(FileDetection {
            media_type: FileType::Video,
            extension: "mp4",
        });
    }

    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"AVI " {
        return Some(FileDetection {
            media_type: FileType::Video,
            extension: "avi",
        });
    }

    if bytes.len() >= 4 && bytes.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]) {
        if bytes.windows(4).any(|w| w == b"webm") {
            return Some(FileDetection {
                media_type: FileType::Video,
                extension: "webm",
            });
        }

        return Some(FileDetection {
            media_type: FileType::Video,
            extension: "mkv",
        });
    }

    if bytes.starts_with(b"ID3")
        || (bytes.len() >= 2 && bytes[0] == 0xFF && (bytes[1] & 0xE0) == 0xE0)
    {
        return Some(FileDetection {
            media_type: FileType::Audio,
            extension: "mp3",
        });
    }

    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE" {
        return Some(FileDetection {
            media_type: FileType::Audio,
            extension: "wav",
        });
    }

    if bytes.starts_with(b"OggS") {
        return Some(FileDetection {
            media_type: FileType::Audio,
            extension: "ogg",
        });
    }

    if bytes.starts_with(b"fLaC") {
        return Some(FileDetection {
            media_type: FileType::Audio,
            extension: "flac",
        });
    }

    None
}

pub fn file_type_from_mime_and_ext(
    mime: Option<Mime>,
    ext: Option<&str>,
) -> Result<FileType, RepoError> {
    //TODO split to business logic type converter
    if let Some(mime) = mime {
        let essence = mime.essence_str();

        return match essence {
            e if e.starts_with("image/") => Ok(FileType::Picture),
            e if e.starts_with("video/") => Ok(FileType::Video),
            e if e.starts_with("audio/") => Ok(FileType::Audio),
            _ => Err(RepoError::StorageError),
        };
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

pub fn detect_file_type(
    bytes: &[u8],
    _mime: Option<Mime>,
    _ext: Option<&str>,
) -> Result<FileDetection, RepoError> {
    if let Some(detected) = detect_from_magic_bytes(bytes) {
        return Ok(detected);
    }

    Err(RepoError::Conflict)
}
