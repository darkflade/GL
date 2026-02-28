use crate::application::use_cases::services::Services;
use crate::domain::files::FileStorage;
use crate::domain::repository::{
    FileRepository, PlaylistRepository, PostRepository, TagRepository,
};
use crate::web::error::AppError;
use crate::web::handlers::utils::{map_repo_error, parse_uuid};
use actix_web::web::Data;
use actix_web::{HttpResponse, web};

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

    let path_str = file_path.path.to_string_lossy();

    log::info!("file requested path={path_str}");

    //TODO make relative paths like a human
    //Something like
    /*
    enum Storage {
        Old,
        Current,
    }
    */
    // Then match and easy format
    let redirect_url = if path_str.starts_with("/media/old") {
        path_str.replace("/media/old", "/protected_old")
    } else {
        path_str.replace("/media/new", "/protected_current")
    };

    log::debug!("resolved x-accel redirect={redirect_url}");

    Ok(HttpResponse::Ok()
        .insert_header(("X-Accel-Redirect", redirect_url))
        .finish())
}
