use crate::application::contracts::{KeysetDirection, PaginationMode};
use serde::Deserialize;
use uuid::Uuid;

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
    pub tag_query: Option<TagQueryParams>,
    pub cursor: Option<SearchCursorParams>,
}

#[derive(Deserialize, Default, Clone, Debug)]
pub struct TagQueryParams {
    pub must: Vec<String>,
    pub should: Vec<String>,
    pub must_not: Vec<String>,
}

#[derive(Deserialize, Default, Clone)]
pub struct SearchCursorParams {
    pub mode: Option<PaginationMode>,
    pub page: Option<i64>,
    pub last_id: Option<Uuid>,
    pub last_score: Option<f64>,
    pub limit: Option<i64>,
    pub direction: Option<KeysetDirection>,
}
