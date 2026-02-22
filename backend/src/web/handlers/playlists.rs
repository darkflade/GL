use actix_identity::Identity;
use actix_web::{HttpResponse, web};
use crate::domain::model::{Cursor, SearchPlaylistsResponse};
use crate::domain::repository::PlaylistRepository;
use crate::web::error::AppError;
use crate::web::handlers::dto::SearchQueryParams;
use crate::web::handlers::utils::{map_repo_error, parse_uuid};

pub async fn get_my_playlists<PLR: PlaylistRepository + Clone>(
    playlist_repo: web::Data<PLR>,
    user: Option<Identity>,
    query:      web::Json<SearchQueryParams>,
) -> Result<HttpResponse, AppError> {
    
    let user_id_str = match user {
        Some(u) => u.id().map_err(|err| {
            log::warn!("failed to resolve identity id from session: {err}");
            AppError::unauthorized("Unauthorized")
        })?,
        None => return Err(AppError::unauthorized("Unauthorized")),
    };

    let user_uuid = parse_uuid(&user_id_str, "user id")?;

    let tag_query = query.tag_query.clone().unwrap_or_default();
    let cursor: Cursor = query.cursor.clone().into();
    

    if tag_query.must.is_empty() &&
        tag_query.should.is_empty() &&
        tag_query.must_not.is_empty()
    {
        //TODO make name search
        Ok(HttpResponse::Ok().json(
            SearchPlaylistsResponse{
                playlists: vec![],
                total_pages: 1, 
            }))

    } else {
        let playlist = playlist_repo
            .search_by_tags(user_uuid, tag_query, cursor)
            .await
            .map_err(|err| map_repo_error(err, "Playlists not found", "playlists.search_by_tags"))?;

        Ok(HttpResponse::Ok().json(playlist))

    }

}
pub async fn create_playlist() -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().body("success"))
}
pub async fn get_playlist_details() -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().body("success"))
}
pub async fn delete_playlist() -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().body("success"))
}
