use crate::domain::model::{FileID, NoteID, PlaylistSummary, Post, PostID, TagCategory, TagID};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct NewTag {
    pub category: TagCategory,
    pub value: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UpdateTag {
    pub category: TagCategory,
    pub value: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct TagQuery {
    pub must: Vec<String>,
    pub should: Vec<String>,
    pub must_not: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NewPost {
    pub id: PostID,
    pub title: String,
    pub file_id: FileID,
    pub tag_ids: Vec<TagID>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UpdatePostNote {
    pub id: Option<NoteID>,
    pub text: String,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UpdatePost {
    pub title: Option<String>,
    pub description: Option<String>,
    pub tag_ids: Option<Vec<TagID>>,
    pub notes: Option<Vec<UpdatePostNote>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NewPlaylist {
    pub title: String,
    pub description: Option<String>,
    pub tag_ids: Option<Vec<TagID>>,
    pub cover: Option<FileID>,
    pub items: Option<Vec<NewPlaylistItem>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NewPlaylistItem {
    pub position: u32,
    pub content: NewPlaylistItemContent,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NewPlaylistItemContent {
    Post { post_id: PostID },
    Note { text: String },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UpdatePlaylist {
    pub title: Option<String>,
    pub description: Option<String>,
    pub tag_ids: Option<Vec<TagID>>,
    pub cover: Option<FileID>,
    pub items: Option<Vec<NewPlaylistItem>>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PlaylistQuery {
    pub tags: TagQuery,
    pub text: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password_hash: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub password_hash: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Cursor {
    pub page: i64,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum PaginationMode {
    Offset,
    #[default]
    Keyset,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct KeysetCursor {
    pub mode: Option<PaginationMode>,
    pub last_id: Option<Uuid>,
    pub last_score: Option<f64>,
    pub limit: Option<i64>,
    pub direction: Option<KeysetDirection>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum KeysetDirection {
    #[default]
    Next,
    Prev,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct KeysetPageCursor {
    pub mode: PaginationMode,
    pub direction: KeysetDirection,
    pub last_id: Uuid,
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
    pub has_prev: bool,
    pub next_cursor: Option<KeysetPageCursor>,
    pub prev_cursor: Option<KeysetPageCursor>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct SearchPlaylistsResponse {
    pub playlists: Vec<PlaylistSummary>,
    pub has_next: bool,
    pub has_prev: bool,
    pub next_cursor: Option<KeysetPageCursor>,
    pub prev_cursor: Option<KeysetPageCursor>,
}
