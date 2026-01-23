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