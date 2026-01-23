use actix_web::{web, Error, HttpResponse};
use actix_web::error::{ErrorBadRequest, ErrorNotFound};
use crate::application::use_cases::services::Services;
use crate::domain::files::FileStorage;
use crate::domain::repository::{FileRepository, PostRepository, TagRepository};
use crate::web::handlers::dto::SearchParams;

pub async fn search_tags<PR, TR, FR, FS>(
    services: web::Data<Services<PR, TR, FR, FS>>,
    params: web::Query<SearchParams>,
) -> Result<HttpResponse, Error>
where
    PR: PostRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    
    let query = &params.query;
    let limit = 10;
    
    if query.is_empty() {
        return Err(ErrorBadRequest("No query given"))
    }
    
    match services.search_tags.execute(query, limit).await {
        Ok(tags) => Ok(HttpResponse::Ok().json(tags)),
        Err(e) => Ok(HttpResponse::NotFound().body(format!("NOt found or {:?}",e)))
    }

}