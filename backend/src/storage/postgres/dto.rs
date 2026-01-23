use std::path::PathBuf;
use serde::Deserialize;
use sqlx::types::Json;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use crate::domain::model::{File, FileID, FileMeta, PostID, Tag, TagCategory, TagID};

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


fn deserialize_optional_offset_datetime<'de, D>(deserializer: D) -> Result<Option<OffsetDateTime>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) => OffsetDateTime::parse(&s, &Rfc3339).map(Some).map_err(serde::de::Error::custom),
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
        }
    }
}

impl From<Json<FileMetaResponse>> for FileMeta {
    fn from(j: Json<FileMetaResponse>) -> Self {
        j.0.into()
    }
}

#[derive(Debug, Deserialize)]
pub struct FileMetaResponse {
    pub width: i32,
    pub height: i32,
    pub duration_ms: i64,
}

impl From<FileMetaResponse> for FileMeta {
    fn from(m: FileMetaResponse) -> Self {
        FileMeta {
            width: Some(m.width as u32),
            height: Some(m.height as u32),
            duration_ms: Some(m.duration_ms as u64),
        }
    }
}

pub struct PostResponse {
    pub id: PostID,
    pub title: String,
    pub file_id: FileID,
    pub description: Option<String>,
    pub tag_ids: Vec<TagID>
}

#[derive(Debug, Deserialize)]
pub struct TagResponse {
    pub id: TagID,
    pub value: String,
    pub category: i16,
}

impl From<TagResponse> for Tag {
    fn from(t: TagResponse) -> Self {
        Tag {
            id: t.id,
            value: t.value,
            category: t.category.into(),
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

/*
#[derive(Clone, Serialize, Deserialize)]
pub struct FileMeta {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_ms: Option<u64>,
}

 */
