use serde::{Deserialize};
use crate::domain::model::{Cursor, TagQuery};

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
    pub cursor: Option<Cursor>,
}

impl From<Option<Cursor>> for Cursor {
    fn from(cursor: Option<Cursor>) -> Self {
        Self {
            page: match cursor {
                Some(cursor) => cursor.page,
                None => 0,
            }
        }
    }
}