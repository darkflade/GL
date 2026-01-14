use actix_web::{web, Error, HttpResponse};
use actix_web::web::Data;
use crate::domain::files::FileStorage;
use uuid::Uuid;
use crate::application::use_cases::services::Services;
use crate::domain::model::StoredFile;
use crate::domain::repository::{FileRepository, PostRepository, TagRepository};

pub async fn download_file<PR, TR, FR, FS>(
    services:   Data<Services<PR, TR, FR, FS>>,
    path:       web::Path<String>,
) -> Result<HttpResponse, Error> 
where
    PR: PostRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let file_id = path.into_inner();

    let file_uuid = Uuid::parse_str(&file_id).map_err(|_| actix_web::error::ErrorBadRequest("Invalid UUID"))?;

    let file_path = services.get_file.execute(file_uuid).await.map_err(|_| {
        actix_web::error::ErrorNotFound("File not found")
    })?;

    let path_str = file_path.path.to_string_lossy();

    println!("Requested {}", path_str);

    //TODO make relative paths like a human
    //Something like
    /*
    enum Storage {
        Old,
        Current,
    }
    */
    // Then match and easy format
    let redirect_url = if path_str.starts_with("/home") {
        format!("/protected_old/{}", path_str.replace("/home/darkflade/Pictures/Wallpapers/", ""))
    } else {
        format!("/protected_current/{}", path_str.replace("/usr/share/nginx/files/", ""))
    };

    println!("Wha {}", redirect_url);

    Ok(HttpResponse::Ok()
        .insert_header(("X-Accel-Redirect", redirect_url))
        .finish()
    )
}
