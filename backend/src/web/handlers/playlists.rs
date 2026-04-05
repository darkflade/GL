use crate::application::contracts::{
    NewPlaylist, PaginationMode, PlaylistQuery, TagQuery, UpdatePlaylist,
};
use crate::application::ports::{
    FileRepository, PlaylistRepository, PostRepository, TagRepository,
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

pub async fn get_my_playlists<PR, PLR, TR, FR, FS>(
    services: web::Data<Services<PR, PLR, TR, FR, FS>>,
    user: Option<Identity>,
    query: web::Json<SearchQueryParams>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: PlaylistRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let user_uuid = resolve_user_id(user)?;

    let tag_query = query.tag_query.clone().unwrap_or_default();
    let text_query = query.text_query.clone().unwrap_or_default();
    let playlist_query: PlaylistQuery = PlaylistQuery {
        tags: TagQuery::from(tag_query.clone()),
        text: text_query.clone(),
    };

    let cursor = query.cursor.clone().unwrap_or_default();
    let cursor_mode = cursor.mode.clone().unwrap_or_default();

    match cursor_mode {
        PaginationMode::Keyset => {
            if text_query.is_empty() && !has_filters(&tag_query) {
                let playlists = services
                    .get_all_playlists
                    .execute(user_uuid, cursor.into())
                    .await
                    .map_err(|err| {
                        map_repo_error(err, "Playlists not found", "playlists.search")
                    })?;

                return Ok(HttpResponse::Ok().json(playlists));
            }

            let playlists = services
                .search_playlists
                .execute(user_uuid, playlist_query, cursor.into())
                .await
                .map_err(|err| map_repo_error(err, "Playlists not found", "playlists.search"))?;

            Ok(HttpResponse::Ok().json(playlists))
        }
        PaginationMode::Offset => Err(AppError::BadRequest(
            "Offset mode doesn't support in playlist".to_string(),
        )),
    }
}
pub async fn create_playlist<PR, PLR, TR, FR, FS>(
    services: web::Data<Services<PR, PLR, TR, FR, FS>>,
    user: Option<Identity>,
    payload: web::Json<NewPlaylist>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: PlaylistRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let user_id = resolve_user_id(user)?;

    let playlist_id = services
        .create_playlist
        .execute(user_id, payload.into_inner())
        .await
        .map_err(|err| map_repo_error(err, "Failed to create playlist", "playlists.create"))?;

    Ok(HttpResponse::Created().json(playlist_id))
}
pub async fn get_playlist_details<PR, PLR, TR, FR, FS>(
    services: web::Data<Services<PR, PLR, TR, FR, FS>>,
    user: Option<Identity>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: PlaylistRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let user_id = resolve_user_id(user)?;
    let playlist_id = parse_uuid(&path.into_inner(), "playlist id")?;

    let playlist = services
        .get_playlist
        .execute(user_id, playlist_id)
        .await
        .map_err(|err| map_repo_error(err, "Playlist not found", "playlists.get"))?;

    Ok(HttpResponse::Ok().json(playlist))
}

pub async fn delete_playlist<PR, PLR, TR, FR, FS>(
    services: web::Data<Services<PR, PLR, TR, FR, FS>>,
    user: Option<Identity>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: PlaylistRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let user_id = resolve_user_id(user)?;
    let playlist_id = parse_uuid(&path.into_inner(), "playlist id")?;

    services
        .delete_playlist
        .execute(user_id, playlist_id)
        .await
        .map_err(|err| map_repo_error(err, "Playlist not found", "playlists.delete"))?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn update_playlist<PR, PLR, TR, FR, FS>(
    services: web::Data<Services<PR, PLR, TR, FR, FS>>,
    user: Option<Identity>,
    path: web::Path<String>,
    payload: web::Json<UpdatePlaylist>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: PlaylistRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let user_id = resolve_user_id(user)?;
    let playlist_id = parse_uuid(&path.into_inner(), "playlist id")?;

    services
        .update_playlist
        .execute(user_id, playlist_id, payload.into_inner())
        .await
        .map_err(|err| map_repo_error(err, "Playlist not found", "playlists.update"))?;

    Ok(HttpResponse::NoContent().finish())
}
