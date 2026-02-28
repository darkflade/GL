use uuid::Uuid;
use crate::domain::model::{Cursor, KeysetCursor, PaginationMode, RepoError, TagQuery};
use crate::web::error::AppError;
use crate::web::handlers::dto::{SearchCursorParams, SearchQueryParams};

pub fn has_filters(tag_query: &TagQuery) -> bool {
    !(tag_query.must.is_empty() && tag_query.should.is_empty() && tag_query.must_not.is_empty())
}

pub fn parse_uuid(value: &str, field_name: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(value).map_err(|err| {
        log::warn!("invalid uuid in {field_name}: {err}; value={value}");
        AppError::bad_request(format!("Invalid {field_name}"))
    })
}

pub fn map_repo_error(error: RepoError, not_found_message: &str, context: &str) -> AppError {
    match error {
        RepoError::NotFound => AppError::not_found(not_found_message),
        RepoError::StorageError => AppError::internal(format!("{context}: storage failure")),
    }
}

impl From<SearchCursorParams> for KeysetCursor {
    fn from(cursor: SearchCursorParams) -> KeysetCursor {
        Self {
            mode: Some(PaginationMode::Keyset),
            last_id: cursor.last_id,
            last_score: cursor.last_score,
            limit: cursor.limit,
        }
    }
}

impl From<SearchCursorParams> for Cursor {
    fn from(cursor: SearchCursorParams) -> Cursor {
        Self {
            //Zero here
            page: cursor.page.unwrap_or_default(),
        }
    }
}