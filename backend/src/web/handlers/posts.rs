use std::path::Path;
use actix_multipart::Multipart;
use actix_web::{HttpResponse, web};
use futures_util::{StreamExt, TryStreamExt};
use serde_json::from_slice;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use crate::application::use_cases::services::Services;
use crate::domain::files::FileStorage;
use crate::domain::model::{ByteStream, Cursor, NewTag, StorageError, TagCategory};
use crate::domain::repository::{FileRepository, PostRepository, TagRepository};
use crate::web::error::AppError;
use crate::web::handlers::dto::{CreatePostMeta, SearchQueryParams};
use crate::web::handlers::utils::{map_repo_error, parse_uuid};

pub async fn search_posts<PR, TR, FR, FS>(
    services:  web::Data<Services<PR, TR, FR, FS>>,
    query:      web::Json<SearchQueryParams>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{

    let tag_query = query.tag_query.clone().unwrap_or_default();
    let cursor: Cursor = query.cursor.clone().into();

    log::info!("search posts requested");

    if tag_query.must.is_empty() &&
        tag_query.should.is_empty() &&
        tag_query.must_not.is_empty()
    {
        let posts = services
            .get_all_posts
            .execute(cursor)
            .await
            .map_err(|err| map_repo_error(err, "Posts not found", "posts.get_all"))?;
        return Ok(HttpResponse::Ok().json(posts));
    }

    let posts = services
        .search_posts
        .execute(tag_query, cursor)
        .await
        .map_err(|err| map_repo_error(err, "Posts not found", "posts.search"))?;

    Ok(HttpResponse::Ok().json(posts))
}

pub async fn create_post<PR, TR, FR, FS>(
    mut payload:    Multipart,
    services:       web::Data<Services<PR, TR, FR, FS>>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let mut meta: Option<CreatePostMeta> = None;

    while let Some(mut field) = payload
        .try_next()
        .await
        .map_err(|err| AppError::bad_request(format!("invalid multipart payload: {err}")))?
    {
        let field_name = field
            .content_disposition()
            .and_then(|cd| cd.get_name())
            .unwrap_or("")
            .to_owned();

        match field_name.as_str() {
            "meta" => {
                let mut body = Vec::new();
                while let Some(chunk) = field.next().await {
                    let bytes = chunk.map_err(|err| {
                        AppError::bad_request(format!("invalid meta chunk in multipart payload: {err}"))
                    })?;
                    body.extend_from_slice(&bytes);
                }
                meta = Some(from_slice(&body).map_err(|err| {
                    AppError::bad_request(format!("invalid meta json payload: {err}"))
                })?);
            }
            "file" => {
                let meta_data = match meta {
                    Some(m) => m,
                    None => return Err(AppError::bad_request("Meta must be before file")),
                };

                let content_type = field.content_type().cloned();

                let file_ext = field
                    .content_disposition()
                    .and_then(|cd| cd.get_filename())
                    .and_then(|n| Path::new(n).extension())
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_string());

                let (tx, rx) = mpsc::channel::<Result<web::Bytes, StorageError>>(8);

                actix_web::rt::spawn(async move {
                    while let Some(chunk) = field.next().await {
                        if tx.send(chunk.map_err(|_| StorageError::Io)).await.is_err() {
                            break;
                        }
                    }
                });

                let stream: ByteStream = Box::pin(ReceiverStream::new(rx));

                let new_tags: Vec<NewTag> = meta_data.tags.iter().map(|t| NewTag {
                    //TODO There is something wrong
                    category: TagCategory::General,
                    value: t.clone(),
                }).collect();

                let id = services.create_post.execute(
                    meta_data.title.clone(),
                    stream,
                    file_ext.as_deref(),
                    content_type,
                    new_tags,
                )
                .await
                .map_err(|err| map_repo_error(err, "Post not found", "posts.create"))?;

                return Ok(HttpResponse::Created().json(id));
            }
            _ => {
                continue;
            }
        }
    }


    Err(AppError::bad_request("Missing file"))
}

pub async fn get_post<PR, TR, FR, FS>(
    services:  web::Data<Services<PR, TR, FR, FS>>,
    path:       web::Path<String>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{

    let id_str = path.into_inner();

    log::debug!("post requested id={id_str}");

    let id = parse_uuid(&id_str, "post id")?;
    let post = services
        .get_post
        .execute(id)
        .await
        .map_err(|err| map_repo_error(err, "Post not found", "posts.get"))?;

    Ok(HttpResponse::Ok().json(post))

}
