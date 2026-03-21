use crate::application::contracts::KeysetCursor;
use crate::application::contracts::{TagBatchUpdate, TagRelationsBatchUpdate};
use crate::application::ports::{
    FileRepository, PlaylistRepository, PostRepository, TagRepository,
};
use crate::application::use_cases::services::Services;
use crate::domain::files::FileStorage;
use crate::web::error::AppError;
use crate::web::handlers::dto::{SearchCursorParams, SearchParams};
use crate::web::handlers::utils::{map_repo_error, parse_uuid};
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

pub async fn list_tags_keyset<PR, PLR, TR, FR, FS>(
    services: web::Data<Services<PR, PLR, TR, FR, FS>>,
    query: web::Query<SearchCursorParams>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: PlaylistRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let cursor: KeysetCursor = query.into_inner().into();

    let tags = services
        .list_tags_keyset
        .execute(cursor)
        .await
        .map_err(|err| map_repo_error(err, "Tags not found", "tags.list_keyset"))?;

    Ok(HttpResponse::Ok().json(tags))
}

pub async fn get_related_tags<PR, PLR, TR, FR, FS>(
    services: web::Data<Services<PR, PLR, TR, FR, FS>>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: PlaylistRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let tag_id = parse_uuid(&path.into_inner(), "tag id")?;

    let tags = services
        .get_related_tags
        .execute(tag_id)
        .await
        .map_err(|err| map_repo_error(err, "Tags not found", "tags.related"))?;

    Ok(HttpResponse::Ok().json(tags))
}

pub async fn update_tags<PR, PLR, TR, FR, FS>(
    services: web::Data<Services<PR, PLR, TR, FR, FS>>,
    payload: web::Json<TagBatchUpdate>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: PlaylistRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let update = payload.into_inner();
    if update.events.is_empty() {
        return Err(AppError::bad_request("No tag events given"));
    }

    services
        .update_tags
        .execute(update)
        .await
        .map_err(|err| map_repo_error(err, "Tag update failed", "tags.update"))?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn update_tag_relations<PR, PLR, TR, FR, FS>(
    services: web::Data<Services<PR, PLR, TR, FR, FS>>,
    payload: web::Json<TagRelationsBatchUpdate>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: PlaylistRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let update = payload.into_inner();
    if update.events.is_empty() {
        return Err(AppError::bad_request("No tag relation events given"));
    }

    services
        .update_tag_relations
        .execute(update)
        .await
        .map_err(|err| map_repo_error(err, "Tag relations update failed", "tags.relations"))?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn list_tag_relations_keyset<PR, PLR, TR, FR, FS>(
    services: web::Data<Services<PR, PLR, TR, FR, FS>>,
    query: web::Query<SearchCursorParams>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: PlaylistRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let cursor: KeysetCursor = query.into_inner().into();

    let relations = services
        .list_tag_relations_keyset
        .execute(cursor)
        .await
        .map_err(|err| map_repo_error(err, "Tag relations not found", "tags.relations.list"))?;

    Ok(HttpResponse::Ok().json(relations))
}
