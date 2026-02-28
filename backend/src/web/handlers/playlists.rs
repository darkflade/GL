use actix_identity::Identity;
use actix_web::{HttpResponse, web};
use crate::application::use_cases::services::Services;
use crate::domain::files::FileStorage;
use crate::domain::model::{KeysetCursor, PaginationMode, SearchPlaylistsResponse};
use crate::domain::repository::{FileRepository, PlaylistRepository, PostRepository, TagRepository};
use crate::web::error::AppError;
use crate::web::handlers::dto::SearchQueryParams;
use crate::web::handlers::utils::{has_filters, map_repo_error, parse_uuid};

pub async fn get_my_playlists<PR, PLR, TR, FR, FS>(
    services:       web::Data<Services<PR, PLR, TR, FR, FS>>,
    user:           Option<Identity>,
    query:          web::Json<SearchQueryParams>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: PlaylistRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    
    let user_id_str = match user {
        Some(u) => u.id().map_err(|err| {
            log::warn!("failed to resolve identity id from session: {err}");
            AppError::unauthorized("Unauthorized")
        })?,
        None => return Err(AppError::unauthorized("Unauthorized")),
    };

    let user_uuid = parse_uuid(&user_id_str, "user id")?;

    let tag_query = query.tag_query.clone().unwrap_or_default();
    let text_query = query.text_query.clone().unwrap_or_default();

    let cursor = query.cursor.clone().unwrap_or_default();
    let cursor_mode = cursor.mode.clone().unwrap_or_default();

    match cursor_mode {
        PaginationMode::Keyset => {
            if text_query.is_empty() && !has_filters(&tag_query) {
                let playlists = services.get_all_playlists
                    .execute(user_uuid, cursor.into())
                    .await
                    .map_err(|err| map_repo_error(err, "Playlists not found", "posts.search"))?;

                Ok(HttpResponse::Ok().json(playlists))
            }

        }
        PaginationMode::Offset => {
            Err(AppError::BadRequest("Offset mode doesn't support in playlist".to_string()))
        }
    }

}
pub async fn create_playlist<PR, PLR, TR, FR, FS>(
    services:       web::Data<Services<PR, PLR, TR, FR, FS>>,
    user:           Option<Identity>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: PlaylistRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    Ok(HttpResponse::Ok().body("success"))
}
pub async fn get_playlist_details<PR, PLR, TR, FR, FS>(
    services:       web::Data<Services<PR, PLR, TR, FR, FS>>,
    user:           Option<Identity>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: PlaylistRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    Ok(HttpResponse::Ok().body("success"))
}
pub async fn delete_playlist<PR, PLR, TR, FR, FS>(
    services:       web::Data<Services<PR, PLR, TR, FR, FS>>,
    user:           Option<Identity>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: PlaylistRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    Ok(HttpResponse::Ok().body("success"))
}
