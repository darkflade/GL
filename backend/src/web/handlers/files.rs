use crate::application::ports::{
    FileRepository, PlaylistRepository, PostRepository, TagRepository,
};
use crate::application::use_cases::services::Services;
use crate::domain::files::FileStorage;
use crate::domain::model::FileStatus;
use crate::web::error::AppError;
use crate::web::handlers::utils::{map_repo_error, parse_uuid};
use actix_web::web::Data;
use actix_web::{HttpResponse, web};
use serde::Deserialize;
use uuid::Uuid;

pub async fn download_file<PR, PLR, TR, FR, FS>(
    services: Data<Services<PR, PLR, TR, FR, FS>>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: PlaylistRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let file_id = path.into_inner();

    let file_uuid = parse_uuid(&file_id, "file id")?;
    let file_path = services
        .get_file
        .execute(file_uuid)
        .await
        .map_err(|err| map_repo_error(err, "File not found", "files.get"))?;

    if !matches!(file_path.status, FileStatus::Ready) {
        return Err(AppError::conflict("File is not ready"));
    }

    let path_str = file_path.path.to_string_lossy();

    log::info!("file requested path={path_str}");

    let redirect_url = map_media_path_to_accel_path(path_str.as_ref())?;

    log::debug!("resolved x-accel redirect={redirect_url}");

    Ok(HttpResponse::Ok()
        .insert_header(("X-Accel-Redirect", redirect_url))
        .finish())
}

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    #[serde(default)]
    pub size: ThumbSize,
}

#[derive(Debug, Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThumbSize {
    #[default]
    Small,
    Large,
}

impl ThumbSize {
    fn as_dir(self) -> &'static str {
        match self {
            ThumbSize::Small => "480",
            ThumbSize::Large => "1080",
        }
    }
}

pub async fn download_thumb<PR, PLR, TR, FR, FS>(
    services: Data<Services<PR, PLR, TR, FR, FS>>,
    path: web::Path<String>,
    query_params: web::Query<QueryParams>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: PlaylistRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let file_id = path.into_inner();

    let file_uuid = parse_uuid(&file_id, "file id")?;
    let file_path = services
        .get_file
        .execute(file_uuid)
        .await
        .map_err(|err| map_repo_error(err, "File not found", "files.get"))?;

    if !matches!(file_path.status, FileStatus::Ready) {
        return Err(AppError::conflict("File is not ready"));
    }

    let path_str = file_path.path.to_string_lossy();

    log::info!("thumb requested path={path_str}");

    let accel_prefix = media_to_accel_prefix(path_str.as_ref())?;
    let (shard_a, shard_b) = uuid_shards(file_uuid);
    let redirect_url = format!(
        "{}/thumb/{}/{}/{}/{}",
        accel_prefix,
        query_params.size.as_dir(),
        shard_a,
        shard_b,
        file_uuid
    );

    log::debug!("resolved x-accel redirect={redirect_url}");

    Ok(HttpResponse::Ok()
        .insert_header(("X-Accel-Redirect", redirect_url))
        .finish())
}

fn map_media_path_to_accel_path(path: &str) -> Result<String, AppError> {
    if let Some(rest) = path.strip_prefix("/media/new/") {
        return Ok(format!("/protected_current/{rest}"));
    }
    if let Some(rest) = path.strip_prefix("/media/old/") {
        return Ok(format!("/protected_old/{rest}"));
    }

    Err(AppError::internal(format!(
        "unsupported media root for path: {path}"
    )))
}

fn media_to_accel_prefix(path: &str) -> Result<&'static str, AppError> {
    if path.starts_with("/media/new/") {
        return Ok("/protected_current");
    }
    if path.starts_with("/media/old/") {
        return Ok("/protected_old");
    }

    Err(AppError::internal(format!(
        "unsupported media root for path: {path}"
    )))
}

fn uuid_shards(id: Uuid) -> (String, String) {
    let bytes = id.as_bytes();
    (format!("{:02x}", bytes[14]), format!("{:02x}", bytes[15]))
}
