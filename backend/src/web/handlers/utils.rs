use uuid::Uuid;
use crate::domain::model::RepoError;
use crate::web::error::AppError;

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
