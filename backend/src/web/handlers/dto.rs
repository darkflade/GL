use crate::domain::model::{PaginationMode, PostID, TagQuery};
use serde::Deserialize;

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
