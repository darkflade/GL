use crate::domain::model::{
    FileID, NoteID, CollectionItemID, CollectionSummary, Post, PostID, Tag, TagCategory, TagID,
    TagRelation,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct NewTag {
    pub category: TagCategory,
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TagBatchUpdate {
    pub events: Vec<TagUpdateEvent>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum TagUpdateEvent {
    Create { tags: Vec<NewTag> },
    Edit { tags: Vec<TagEdit> },
    Remove { tag_ids: Vec<TagID> },
    Merge { source_id: TagID, target_id: TagID },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TagEdit {
    pub id: TagID,
    pub category: TagCategory,
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TagRelationsBatchUpdate {
    pub events: Vec<TagRelationUpdateEvent>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum TagRelationUpdateEvent {
    Link {
        parent_id: TagID,
        child_ids: Vec<TagID>,
    },
    Unlink {
        parent_id: TagID,
        child_ids: Vec<TagID>,
    },
    Alias {
        tag_id: TagID,
        alias_ids: Vec<TagID>,
    },
    Unalias {
        tag_id: TagID,
        alias_ids: Vec<TagID>,
    },
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
    pub add_tag_ids: Option<Vec<TagID>>,
    pub remove_tag_ids: Option<Vec<TagID>>,
    pub notes: Option<Vec<UpdatePostNote>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NewCollection {
    pub title: String,
    pub description: Option<String>,
    pub tag_ids: Option<Vec<TagID>>,
    pub cover: Option<FileID>,
    pub items: Option<Vec<NewCollectionItem>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NewCollectionItem {
    pub position: u32,
    pub content: NewCollectionItemContent,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NewCollectionItemContent {
    Post { post_id: PostID },
    Note { text: String },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UpdateCollection {
    pub title: Option<String>,
    pub description: Option<String>,
    pub add_tag_ids: Option<Vec<TagID>>,
    pub remove_tag_ids: Option<Vec<TagID>>,
    pub cover: Option<FileID>,
    pub item_events: Option<Vec<CollectionItemEvent>>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum CollectionItemEvent {
    Add {
        after_id: Option<CollectionItemID>,
        content: NewCollectionItemContent,
    },
    Edit {
        item_id: CollectionItemID,
        content: NewCollectionItemContent,
    },
    Remove {
        item_id: CollectionItemID,
    },
    Move {
        item_id: CollectionItemID,
        after_id: Option<CollectionItemID>,
    },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CollectionQuery {
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
pub struct SearchCollectionsResponse {
    pub collections: Vec<CollectionSummary>,
    pub has_next: bool,
    pub has_prev: bool,
    pub next_cursor: Option<KeysetPageCursor>,
    pub prev_cursor: Option<KeysetPageCursor>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct SearchTagsResponse {
    pub tags: Vec<Tag>,
    pub has_next: bool,
    pub has_prev: bool,
    pub next_cursor: Option<KeysetPageCursor>,
    pub prev_cursor: Option<KeysetPageCursor>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct SearchTagRelationsResponse {
    pub relations: Vec<TagRelation>,
    pub has_next: bool,
    pub has_prev: bool,
    pub next_cursor: Option<KeysetPageCursor>,
    pub prev_cursor: Option<KeysetPageCursor>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TagNeighborhood { pub roots: Vec<Tag>, pub tag: Tag, pub parents: Vec<Tag>, pub children: Vec<Tag>, pub aliases: Vec<Tag> }
