use crate::application::ports::{
    FileRepository, PlaylistRepository, PostRepository, TagRepository,
};
use crate::application::use_cases::services::Services;
use crate::domain::files::FileStorage;
use crate::web::error::AppError;
use crate::web::handlers::dto::SearchParams;
use crate::web::handlers::utils::map_repo_error;
use actix_web::{HttpResponse, web};

pub async fn search_tags<PR, PLR, TR, FR, FS>(
    services: web::Data<Services<PR, PLR, TR, FR, FS>>,
    params: web::Query<SearchParams>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: PlaylistRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let query = &params.query;
    let limit = 10;

    if query.is_empty() {
        return Err(AppError::bad_request("No query given"));
    }

    let tags = services
        .search_tags
        .execute(query, limit)
        .await
        .map_err(|err| map_repo_error(err, "Tags not found", "tags.search"))?;

    Ok(HttpResponse::Ok().json(tags))
}
