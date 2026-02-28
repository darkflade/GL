use crate::domain::model::{
    Cursor, CursorID, File, FileID, FileMeta, PlaylistContent, PlaylistID, PlaylistItem,
    PlaylistItemID, Post, PostID, Tag, TagCategory, TagID,
};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use std::any::Any;
use std::path::PathBuf;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

#[derive(Debug, Deserialize)]
pub struct FileResponse {
    pub id: FileID,
    pub path: String,
    pub hash: Option<String>,
    pub media_type: i16,
    pub meta: Option<Json<FileMetaResponse>>,
    #[serde(deserialize_with = "deserialize_optional_offset_datetime")]
    pub created_at: Option<OffsetDateTime>,
}

fn deserialize_optional_offset_datetime<'de, D>(
    deserializer: D,
) -> Result<Option<OffsetDateTime>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) => OffsetDateTime::parse(&s, &Rfc3339)
            .map(Some)
            .map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

impl From<FileResponse> for File {
    fn from(row: FileResponse) -> Self {
        Self {
            id: row.id,
            path: PathBuf::from(row.path),
            hash: row.hash,
            media_type: row.media_type.into(),
            meta: row.meta.map(FileMeta::from),
            created_at: row.created_at,
            thumbnail: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct FileMetaResponse {
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub extension: Option<String>,
    pub duration_ms: Option<i64>,
}

impl From<Json<FileMetaResponse>> for FileMeta {
    fn from(j: Json<FileMetaResponse>) -> Self {
        j.0.into()
    }
}

impl From<FileMetaResponse> for FileMeta {
    fn from(m: FileMetaResponse) -> Self {
        FileMeta {
            width: m.width.and_then(|w| u32::try_from(w).ok()),
            height: m.height.and_then(|h| u32::try_from(h).ok()),
            extension: m.extension,
            duration_ms: m.duration_ms.and_then(|d| u64::try_from(d).ok()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TagResponse {
    pub id: TagID,
    pub name: String,
    pub category: i16,
    pub count: i16,
}

impl From<TagResponse> for Tag {
    fn from(t: TagResponse) -> Self {
        Tag {
            id: t.id,
            name: t.name,
            category: t.category.into(),
            count: t.count.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub enum TagCategoryResponse {
    Artist = 0,
    Copyright = 1,
    Character = 2,
    General = 3,
}

impl From<TagCategoryResponse> for TagCategory {
    fn from(c: TagCategoryResponse) -> Self {
        match c {
            TagCategoryResponse::Artist => TagCategory::Artist,
            TagCategoryResponse::Copyright => TagCategory::Copyright,
            TagCategoryResponse::Character => TagCategory::Character,
            TagCategoryResponse::General => TagCategory::General,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlaylistItemResponse {
    pub id: PlaylistItemID,
    pub playlist_id: PlaylistID,
    pub position: u32,
    pub content: PlaylistContent,
}
impl From<PlaylistItemResponse> for PlaylistItem {
    fn from(p: PlaylistItemResponse) -> Self {
        Self {
            id: p.id,
            position: p.position,
            content: p.content,
        }
    }
}
