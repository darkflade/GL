use std::path::{Path};
use actix_multipart::Multipart;
use actix_web::{web, Error, HttpResponse};
use actix_web::web::{Bytes, Data};
use uuid::Uuid;
use futures_util::{StreamExt, TryStreamExt};
use serde_json::from_slice;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use crate::application::use_cases::services::Services;
use crate::domain::files::FileStorage;
use crate::domain::model::{ByteStream, NewTag, RepoError, StorageError, TagCategory, TagQuery};
use crate::domain::repository::{FileRepository, PostRepository, TagRepository};
use crate::web::handlers::dto::CreatePostMeta;

pub async fn search_posts<PR, TR, FR, FS>(
    services:  Data<Services<PR, TR, FR, FS>>,
    query:      web::Json<TagQuery>,
) -> Result<HttpResponse, Error>
where
    PR: PostRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{

    let tag_query = query.into_inner();

    println!("search requested");

    if tag_query.must.is_empty() 
        && tag_query.should.is_empty() 
        && tag_query.must_not.is_empty() 
    {
        return match services.get_all_posts.execute().await {
            Ok(posts) => Ok(HttpResponse::Ok().json(posts)),
            //TODO add logger
            Err(_) => Ok(HttpResponse::InternalServerError().body("Search failed")),
        }
    }

    match services.search_posts.execute(tag_query).await {
        Ok(posts) => Ok(HttpResponse::Ok().json(posts)),
        //TODO add logger
        Err(_) => Ok(HttpResponse::InternalServerError().body("Search failed")),
    }
}

pub async fn create_post<PR, TR, FR, FS>(
    mut payload:    Multipart,
    services:       Data<Services<PR, TR, FR, FS>>,
) -> Result<HttpResponse, Error>
where
    PR: PostRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let mut meta: Option<CreatePostMeta> = None;

    while let Ok(Some(mut field)) = payload.try_next().await {
        let field_name = field
            .content_disposition()
            .and_then(|cd| cd.get_name())
            .unwrap_or("")
            .to_owned();

        match field_name.as_str() {
            "meta" => {
                let mut body = Vec::new();
                while let Some(chunk) = field.next().await {
                    body.extend_from_slice(&chunk?);
                }
                meta = Some(from_slice(&body).map_err(actix_web::error::ErrorBadRequest)?);
            }
            "file" => {
                let meta_data = match meta {
                    Some(m) => m,
                    None => return Ok(HttpResponse::BadRequest().body("Meta must be before file")),
                };

                let content_type = field.content_type().cloned();

                let file_ext = field
                    .content_disposition()
                    .and_then(|cd| cd.get_filename())
                    .and_then(|n| Path::new(n).extension())
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_string());

                let (tx, rx) = mpsc::channel::<Result<Bytes, StorageError>>(8);

                actix_web::rt::spawn(async move {
                    while let Some(chunk) = field.next().await {
                        let _ = tx.send(chunk.map_err(|_| StorageError::Io)).await;
                    }
                });

                let stream: ByteStream = Box::pin(ReceiverStream::new(rx));

                let new_tags: Vec<NewTag> = meta_data.tags.iter().map(|t| NewTag {
                    //TODO There is something wrong
                    category: TagCategory::General,
                    value: t.clone(),
                }).collect();

                let res = services.create_post.execute(
                    meta_data.title.clone(),
                    stream,
                    file_ext.as_deref(),
                    content_type,
                    new_tags,

                ).await;

                return match res {
                    Ok(id) => Ok(HttpResponse::Created().json(id)),
                    Err(e) => Ok(HttpResponse::InternalServerError().body(format!("Upload failed {:?}", e))),
                };
            }
            _ => {
                continue;
            }
        }
    }


    Ok(HttpResponse::BadRequest().body("Missing file"))
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