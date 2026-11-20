use crate::application::contracts::{
    NewCollection, PaginationMode, CollectionQuery, TagQuery, UpdateCollection,
};
use crate::application::ports::{
    FileRepository, CollectionRepository, PostRepository, TagRepository,
};
use crate::application::use_cases::services::Services;
use crate::domain::files::FileStorage;
use crate::web::error::AppError;
use crate::web::handlers::dto::SearchQueryParams;
use crate::web::handlers::utils::{has_filters, map_repo_error, parse_uuid};
use actix_identity::Identity;
use actix_web::{HttpResponse, web};
use uuid::Uuid;

fn resolve_user_id(user: Option<Identity>) -> Result<Uuid, AppError> {
    let user_id_str = match user {
        Some(u) => u.id().map_err(|err| {
            log::warn!("Failed to resolve identity id from session: {err}");
            AppError::unauthorized("Unauthorized")
        })?,
        None => return Err(AppError::unauthorized("Unauthorized")),
    };

    parse_uuid(&user_id_str, "user id")
}

pub async fn get_my_collections<PR, PLR, TR, FR, FS>(
    search: web::Data<crate::storage::postgres::search::PostgresSearchRepository>,
    services: web::Data<Services<PR, PLR, TR, FR, FS>>,
    user: Option<Identity>,
    query: web::Json<SearchQueryParams>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: CollectionRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let user_uuid = resolve_user_id(user)?;

    let options = crate::storage::postgres::search::Options::parse(&query)?;
    if options.advanced() {
        if matches!(query.cursor.as_ref().and_then(|c| c.mode.as_ref()), Some(PaginationMode::Offset)) { return Err(AppError::bad_request("Collections require keyset pagination")); }
        let result = search.search(&options, &query, Some(user_uuid)).await?;
        let ids: Vec<_> = result.rows.iter().map(|(id, _)| *id).collect();
        let collections = search.summaries(user_uuid, &ids).await?;
        return Ok(HttpResponse::Ok().json(result.response(serde_json::json!({"collections": collections}))));
    }
    let tag_query = query.tag_query.clone().unwrap_or_default();
    let text_query = query.text_query.clone().unwrap_or_default();
    let collection_query: CollectionQuery = CollectionQuery {
        tags: TagQuery::from(tag_query.clone()),
        text: text_query.clone(),
    };

    let cursor = query.cursor.clone().unwrap_or_default();
    let cursor_mode = cursor.mode.clone().unwrap_or_default();

    match cursor_mode {
        PaginationMode::Keyset => {
            if text_query.is_empty() && !has_filters(&tag_query) {
                let collections = services
                    .get_all_collections
                    .execute(user_uuid, cursor.into())
                    .await
                    .map_err(|err| {
                        map_repo_error(err, "Collections not found", "collections.search")
                    })?;

                return Ok(HttpResponse::Ok().json(collections));
            }

            let collections = services
                .search_collections
                .execute(user_uuid, collection_query, cursor.into())
                .await
                .map_err(|err| map_repo_error(err, "Collections not found", "collections.search"))?;

            Ok(HttpResponse::Ok().json(collections))
        }
        PaginationMode::Offset => Err(AppError::BadRequest(
            "Offset mode doesn't support in collection".to_string(),
        )),
    }
}
pub async fn create_collection<PR, PLR, TR, FR, FS>(
    services: web::Data<Services<PR, PLR, TR, FR, FS>>,
    user: Option<Identity>,
    payload: web::Json<NewCollection>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: CollectionRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let user_id = resolve_user_id(user)?;

    let collection_id = services
        .create_collection
        .execute(user_id, payload.into_inner())
        .await
        .map_err(|err| map_repo_error(err, "Failed to create collection", "collections.create"))?;

    Ok(HttpResponse::Created().json(collection_id))
}
pub async fn get_collection_details<PR, PLR, TR, FR, FS>(
    services: web::Data<Services<PR, PLR, TR, FR, FS>>,
    user: Option<Identity>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: CollectionRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let user_id = resolve_user_id(user)?;
    let collection_id = parse_uuid(&path.into_inner(), "collection id")?;

    let collection = services
        .get_collection
        .execute(user_id, collection_id)
        .await
        .map_err(|err| map_repo_error(err, "Collection not found", "collections.get"))?;

    Ok(HttpResponse::Ok().json(collection))
}

pub async fn delete_collection<PR, PLR, TR, FR, FS>(
    services: web::Data<Services<PR, PLR, TR, FR, FS>>,
    user: Option<Identity>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: CollectionRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let user_id = resolve_user_id(user)?;
    let collection_id = parse_uuid(&path.into_inner(), "collection id")?;

    services
        .delete_collection
        .execute(user_id, collection_id)
        .await
        .map_err(|err| map_repo_error(err, "Collection not found", "collections.delete"))?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn update_collection<PR, PLR, TR, FR, FS>(
    services: web::Data<Services<PR, PLR, TR, FR, FS>>,
    user: Option<Identity>,
    path: web::Path<String>,
    payload: web::Json<UpdateCollection>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: CollectionRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let user_id = resolve_user_id(user)?;
    let collection_id = parse_uuid(&path.into_inner(), "collection id")?;

    services
        .update_collection
        .execute(user_id, collection_id, payload.into_inner())
        .await
        .map_err(|err| map_repo_error(err, "Collection not found", "collections.update"))?;

    Ok(HttpResponse::NoContent().finish())
}
