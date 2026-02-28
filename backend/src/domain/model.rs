use std::path::PathBuf;
use std::pin::Pin;
use actix_web::web::Bytes;
use futures_util::Stream;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

pub type ByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, StorageError>> + Send>>;

pub type FileID = Uuid;
pub type CursorID = Uuid;
pub type PostID = Uuid;
pub type NoteID = Uuid;
pub type TagID = Uuid;
pub type PlaylistID = Uuid;
pub type PlaylistItemID = Uuid;
pub type UserID = Uuid;
pub type RelativePath = String;


#[derive(Clone, Serialize, Deserialize)]
pub enum FileType {
    Picture = 0,
    Video = 1,
    Audio = 2,
}
#[derive(Clone, Serialize, Deserialize)]
pub enum ThumbSizeType {
    Small = 0,
    Medium = 1,
    Large = 2,
}

impl From<i16> for FileType {
    fn from(v: i16) -> Self {
        match v {
            0 => FileType::Picture,
            1 => FileType::Video,
            2 => FileType::Audio,
            _ => FileType::Picture
        }
    }
}

impl From<FileType> for i16 {
    fn from(v: FileType) -> Self {
        v as i16
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum TagCategory {
    Artist = 0,
    Copyright = 1,
    Character = 2,
    General = 3,
}

impl From<i16> for TagCategory {
    fn from(v: i16) -> Self {
        match v {
            0 => TagCategory::Artist,
            1 => TagCategory::Copyright,
            2 => TagCategory::Character,
            _ => TagCategory::General
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: PostID,
    pub title: String,
    pub description: Option<String>,
    pub file: File,
    pub tags: Vec<Tag>,
    pub notes: Vec<PostNote>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PostNote {
    pub id: NoteID,
    pub text: String,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: TagID,
    pub name: String,
    pub category: TagCategory,
    pub count: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct File {
    pub id: FileID,
    pub path: PathBuf,
    pub hash: Option<String>,
    pub media_type: FileType,
    pub meta: Option<FileMeta>,
    pub created_at: Option<OffsetDateTime>,
    pub thumbnail: Option<Thumbnail>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Thumbnail {
    height: u32,
    weight: u32,
    path: PathBuf,
    size_type: ThumbSizeType,
    created_at: Option<OffsetDateTime>,

}

impl Default for File {
    fn default() -> Self {
        File {
            id: Uuid::now_v7(),
            path: PathBuf::from(""),
            hash: None,
            media_type: FileType::Picture,
            meta: None,
            created_at: None,
            thumbnail: None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FileMeta {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub extension: Option<String>,
    pub duration_ms: Option<u64>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StoredFile {
    pub id: FileID,
    pub path: PathBuf,
    pub hash: String,
    pub media_type: FileType,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: PlaylistID,
    pub title: String,
    pub description: String,
    pub tags: Vec<Tag>,
    pub cover: Option<FileID>,
    pub items: Vec<PlaylistItem>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlaylistItem {
    pub id: PlaylistItemID,
    pub position: u32,
    pub content: PlaylistContent,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum PlaylistContent {
    Post(Post),
    Note(String)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlaylistSummary {
    pub id: PlaylistID,
    pub title: String,
    pub description: String,
    pub cover: Option<FileID>,
    pub item_count: i64,
    pub tags: Vec<Tag>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserID,
    pub username: String,
    pub password_hash: String,
}

// Classes for request
#[derive(Clone, Serialize, Deserialize)]
pub struct NewTag {
    pub category: TagCategory,
    pub value: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NewPost {
    pub id: PostID,
    pub title: String,
    pub file_id: FileID,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password_hash: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TagQuery {
    pub must: Vec<String>,
    pub should: Vec<String>,
    pub must_not: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PlaylistQuery {
    pub tags: Option<TagQuery>,
    pub text: Option<String>,
}

// Errors
#[derive(Debug)]
pub enum RepoError {
    NotFound,
    StorageError,
}
#[derive(Debug)]
pub enum StorageError {
    NotFound,
    StorageError,
    Io,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Cursor{
    pub page: i64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum PaginationMode {
    Offset,
    Keyset,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct KeysetCursor {
    pub mode: Option<PaginationMode>,
    pub last_id: Option<PostID>,
    pub last_score: Option<f64>,
    pub limit: Option<i64>,
}

//TODO Return previous cursor also
#[derive(Clone, Serialize, Deserialize)]
pub struct NextKeysetCursor {
    pub mode: PaginationMode,
    pub last_id: PostID,
    pub last_score: f64,
    pub limit: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SearchPostsOffsetResponse {
    pub posts: Vec<Post>,
    pub total_pages: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SearchPostsKeysetResponse {
    pub posts: Vec<Post>,
    pub has_next: bool,
    pub next_cursor: Option<NextKeysetCursor>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SearchPlaylistsResponse {
    pub playlists: Vec<PlaylistSummary>,
    pub has_next: bool,
    pub next_cursor: Option<NextKeysetCursor>,
}
