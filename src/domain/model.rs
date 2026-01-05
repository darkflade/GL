use std::path::PathBuf;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

pub type FileID = Uuid;
pub type PostID = Uuid;
pub type NoteID = Uuid;
pub type TagID = Uuid;
pub type PlaylistID = Uuid;
pub type PlaylistItemID = Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: PostID,
    pub title: String,
    pub tag_ids: Vec<TagID>,
    pub file_id: FileID,
    pub notes: Option<Vec<PostNote>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PostNote {
    pub id: NoteID,
    pub post_id: PostID,
    pub text: String,
    pub position: NotePosition,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NotePosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StoredFile {
    pub id: FileID,
    pub path: PathBuf,
    pub hash: String,
    pub media_type: FileType,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct File {
    pub id: FileID,
    pub path: PathBuf,
    pub hash: Option<String>,
    pub media_type: FileType,
    pub meta: FileMeta
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FileMeta {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_ms: Option<u64>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum FileType {
    Picture,
    Video,
    Audio
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: TagID,
    pub category: TagCategory,
    pub value: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum TagCategory {
    Artist = 0,
    Copyright = 1,
    Character = 2,
    General = 3,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct PlaylistWithItems {
    pub playlist: Playlist,
    pub items: Vec<PlaylistItem>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: PlaylistID,
    pub title: String,
    pub description: String,
    pub tag_ids: Vec<TagID>,
    pub cover: Option<FileID>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlaylistItem {
    pub id: PlaylistItemID,
    pub playlist_id: PlaylistID,
    pub position: u32,
    pub content: PlaylistContent,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum PlaylistContent {
    Post(Post),
    Note(String)
}

#[derive(Serialize)]
pub struct PlaylistSummary {
    pub id: PlaylistID,
    pub title: String,
    pub description: String,
    pub cover: Option<FileID>,
    pub item_count: i64,
    pub tags: Vec<Tag>,
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
pub struct TagQuery {
    pub must: Vec<TagID>,
    pub should: Vec<TagID>,
    pub must_not: Vec<TagID>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlaylistQuery {
    pub tags: Option<TagQuery>,
    pub name: Option<String>,
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
