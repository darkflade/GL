use std::path::PathBuf;
use actix_multipart::Multipart;
use actix_web::{web, Error, HttpResponse};
use tokio::fs::File;
use uuid::Uuid;
use futures_util::{StreamExt, TryStreamExt};
use tokio::io::AsyncWriteExt;
use crate::application::repository::CreatePostUseCase;
use crate::domain::files::FileStorage;
use crate::domain::model::{NewTag, RepoError, TagCategory, TagQuery};
use crate::domain::repository::{PostRepository, TagRepository};

pub async fn search_posts<PR: PostRepository + Clone>(
    post_repo:  web::Data<PR>,
    query:      web::Json<TagQuery>,
) -> Result<HttpResponse, Error>
{
    let tag_query = query.into_inner();

    match post_repo.search(tag_query).await {
        Ok(posts) => Ok(HttpResponse::Ok().json(posts)),
        //TODO add logger
        Err(_) => Ok(HttpResponse::InternalServerError().body("Search failed")),
    }
}

#[derive(serde::Deserialize)]
struct CreatePostMeta {
    title: String,
    tags: Vec<String>,
}

pub async fn create_post<PR, TR, FS>(
    mut payload:    Multipart,
    post_repo:      web::Data<PR>,
    tag_repo:       web::Data<TR>,
    files:          web::Data<FS>,
) -> Result<HttpResponse, Error>
where
    PR: PostRepository + Clone,
    TR: TagRepository + Clone,
    FS: FileStorage + Clone,
{
    let mut title = None;
    let mut tags = Vec::new();
    let mut temp_file_path: Option<PathBuf> = None;
    let mut file_ext = None;

    let temp_dir = std::env::temp_dir().join("glabs_upload");
    tokio::fs::create_dir_all(&temp_dir).await?;

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition().unwrap();
        let field_name = content_disposition.get_name().unwrap_or("");

        match field_name {
            "meta" => {
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    data.extend_from_slice(&chunk?);
                }
                if let Ok(meta) = serde_json::from_slice::<CreatePostMeta>(&data) {
                    title = Some(meta.title);
                    tags = meta.tags;
                }
            }
            "file" => {
                let temp_filename = format!("upload_{}", Uuid::new_v4());
                let path = temp_dir.join(&temp_filename);

                let mut f = File::create(&path).await?;

                if let Some(filename) = content_disposition.get_filename() {
                    if let Some(ext) = std::path::Path::new(filename).extension() {
                        file_ext = Some(ext.to_string_lossy().to_string());
                    }
                }

                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    f.write_all(&data).await?;
                }
                f.flush().await?;

                temp_file_path = Some(path);
            }
            _ => {
                while let Some(_) = field.next().await {}
            }
        }
    }


    let title = match title {
        Some(title) => title,
        None => return Ok(HttpResponse::BadRequest().body("Missing 'meta' field or invalid JSON")),
    };
    let path = match temp_file_path {
        Some(p ) => p,
        None => return Ok(HttpResponse::BadRequest().body("Missing 'file' field")),
    };

    let use_case = CreatePostUseCase {
        posts: post_repo.get_ref().clone(),
        tags: tag_repo.get_ref().clone(),
        files: files.get_ref().clone(),
    };

    let new_tags: Vec<NewTag> = tags.into_iter().map(|t| NewTag {
        category: TagCategory::General,
        value: t,
    }).collect();

    let res = use_case.execute(
        title,
        path,
        file_ext.as_deref(),
        new_tags,
    ).await;

    match res {
        Ok(id) => Ok(HttpResponse::Ok().json(id)),
        //TODO logger
        Err(e) => Ok(HttpResponse::InternalServerError().body(format!("{:?}", e))),
    }
}

pub async fn get_post<PR: PostRepository + Clone>(
    post_repo:  web::Data<PR>,
    path:       web::Path<String>,
) -> Result<HttpResponse, Error>
{
    let id_str = path.into_inner();

    let id = Uuid::parse_str(&id_str)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid UUID"))?;

    let result = post_repo.get(id).await;

    match result {
        Ok(post) => Ok(HttpResponse::Ok().json(post)),
        Err(RepoError::NotFound) => Ok(HttpResponse::NotFound().body("Post Not Found")),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }

}