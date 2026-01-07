use actix_identity::Identity;
use actix_web::{web, Error, HttpResponse};
use uuid::Uuid;
use crate::domain::repository::PlaylistRepository;

pub async fn get_my_playlists<PLR: PlaylistRepository + Clone>(
    playlist_repo: web::Data<PLR>,
    user: Option<Identity>,
) -> Result<HttpResponse, Error> {
    
    let user_id_str = match user {
        Some(u) => u.id()?,
        None => return Ok(HttpResponse::Unauthorized().body("Unauthorized")),
    };
    
    let user_uuid = Uuid::parse_str(&user_id_str)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid User ID in cookie"))?;

    let playlist = playlist_repo.get_by_user(user_uuid).await
        .map_err(|_| actix_web::error::ErrorInternalServerError("DB Error"))?;
    
    Ok(HttpResponse::Ok().json(playlist))
}
pub async fn create_playlist() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("success"))
}
pub async fn get_playlist_details() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("success"))
}
pub async fn delete_playlist() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("success"))
}
