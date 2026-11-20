use crate::{
    domain::playback::PlaybackSettings,
    storage::postgres::playback::PostgresPlaybackRepository,
    web::{error::AppError, handlers::utils::parse_uuid},
};
use actix_identity::Identity;
use actix_web::{HttpResponse, web};
use serde::Deserialize;
use uuid::Uuid;
#[derive(Deserialize)]
pub struct Scope {
    collection_id: Option<Uuid>,
}
fn user_id(user: Option<Identity>) -> Result<Uuid, AppError> {
    let id = user
        .ok_or_else(|| AppError::unauthorized("Unauthorized"))?
        .id()
        .map_err(|_| AppError::unauthorized("Unauthorized"))?;
    parse_uuid(&id, "user id")
}
pub async fn get(
    repo: web::Data<PostgresPlaybackRepository>,
    user: Option<Identity>,
    scope: web::Query<Scope>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(repo.get(user_id(user)?, scope.collection_id).await?))
}
pub async fn save(
    repo: web::Data<PostgresPlaybackRepository>,
    user: Option<Identity>,
    scope: web::Query<Scope>,
    payload: web::Json<PlaybackSettings>,
) -> Result<HttpResponse, AppError> {
    let user = user_id(user)?;
    if !(1..=3600).contains(&payload.interval_seconds) {
        return Err(AppError::BadRequest(
            "Interval must be between 1 and 3600 seconds".into(),
        ));
    }
    repo.save(user, scope.collection_id, payload.into_inner())
        .await?;
    Ok(HttpResponse::NoContent().finish())
}
pub async fn reset(
    repo: web::Data<PostgresPlaybackRepository>,
    user: Option<Identity>,
    scope: web::Query<Scope>,
) -> Result<HttpResponse, AppError> {
    repo.reset(user_id(user)?, scope.collection_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize)]
pub struct Position {
    item_id: Uuid,
}
pub async fn position(
    repo: web::Data<PostgresPlaybackRepository>,
    user: Option<Identity>,
    scope: web::Query<Scope>,
    payload: web::Json<Position>,
) -> Result<HttpResponse, AppError> {
    let user = user_id(user)?;
    let collection = scope
        .collection_id
        .ok_or_else(|| AppError::bad_request("collection_id is required"))?;
    repo.save_position(user, collection, payload.item_id)
        .await?;
    Ok(HttpResponse::NoContent().finish())
}
