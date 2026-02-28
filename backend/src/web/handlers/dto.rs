use serde::{Deserialize};
use crate::domain::model::{PaginationMode, PostID, SearchPlaylistsResponse, TagQuery};

#[derive(Deserialize)]
pub struct CreatePostMeta {
    pub title: String,
    pub tags: Vec<String>,
}

#[derive(Deserialize)]
pub struct SearchParams {
    pub query: String,
}

//Common interface for search query
#[derive(Deserialize)]
pub struct SearchQueryParams {
    pub text_query: Option<String>,
    pub tag_query: Option<TagQuery>,
    pub cursor: Option<SearchCursorParams>,
}

#[derive(Deserialize, Default, Clone)]
pub struct SearchCursorParams {
    pub mode: Option<PaginationMode>,
    pub page: Option<i64>,
    pub last_id: Option<PostID>,
    pub last_score: Option<f64>,
    pub limit: Option<i64>,
}

impl Default for TagQuery {
    fn default() -> Self {
        Self{
            must:       vec![],
            should:     vec![],
            must_not:   vec![],
        }
    }
}

impl Default for PaginationMode {
    fn default() -> Self {
        PaginationMode::Keyset
    }
}

impl Default for SearchPlaylistsResponse {
    fn default() -> Self {
        Self{
            playlists: vec![],
            has_next: false,
            next_cursor: None,
        }
    }
}