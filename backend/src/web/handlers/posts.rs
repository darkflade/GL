use crate::application::contracts::{Cursor, KeysetCursor, NewTag, PaginationMode, UpdatePost};
use crate::application::ports::{
    FileRepository, PlaylistRepository, PostRepository, TagRepository,
};
use crate::application::use_cases::services::Services;
use crate::domain::files::FileStorage;
use crate::domain::model::{ByteStream, StorageError, TagCategory};
use crate::web::error::AppError;
use crate::web::handlers::dto::{CreatePostMeta, SearchQueryParams};
use crate::web::handlers::utils::{has_filters, map_repo_error, parse_uuid};
use actix_multipart::Multipart;
use actix_web::{HttpResponse, web};
use futures_util::{StreamExt, TryStreamExt};
use serde_json::from_slice;
use std::path::Path;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub async fn search_posts<PR, PLR, TR, FR, FS>(
    services: web::Data<Services<PR, PLR, TR, FR, FS>>,
    query: web::Json<SearchQueryParams>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: PlaylistRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let tag_query = query.tag_query.clone().unwrap_or_default();
    let cursor = query.cursor.clone().unwrap_or_default();
    let cursor_mode = cursor.mode.clone().unwrap_or_default();

    log::info!("search posts requested mode={cursor_mode:?}");

    match cursor_mode {
        PaginationMode::Offset => {
            let offset_cursor: Cursor = cursor.into();

            if !has_filters(&tag_query) {
                let posts = services
                    .get_all_posts
                    .execute(offset_cursor)
                    .await
                    .map_err(|err| map_repo_error(err, "Posts not found", "posts.get_all"))?;
                return Ok(HttpResponse::Ok().json(posts));
            }

            let posts = services
                .search_posts
                .execute(tag_query.clone().into(), offset_cursor)
                .await
                .map_err(|err| map_repo_error(err, "Posts not found", "posts.search"))?;

            Ok(HttpResponse::Ok().json(posts))
        }
        PaginationMode::Keyset => {
            let keyset_cursor: KeysetCursor = cursor.into();

            if !has_filters(&tag_query) {
                let posts = services
                    .get_all_posts_keyset
                    .execute(keyset_cursor)
                    .await
                    .map_err(|err| {
                        map_repo_error(err, "Posts not found", "posts.get_all_keyset")
                    })?;

                return Ok(HttpResponse::Ok().json(posts));
            }

            let posts = services
                .search_posts_keyset
                .execute(tag_query.into(), keyset_cursor)
                .await
                .map_err(|err| map_repo_error(err, "Posts not found", "posts.search_keyset"))?;

            Ok(HttpResponse::Ok().json(posts))
        }
    }
}

pub async fn create_post<PR, PLR, TR, FR, FS>(
    mut payload: Multipart,
    services: web::Data<Services<PR, PLR, TR, FR, FS>>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: PlaylistRepository + Clone,
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
                        AppError::bad_request(format!(
                            "invalid meta chunk in multipart payload: {err}"
                        ))
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

                let new_tags: Vec<NewTag> = meta_data
                    .tags
                    .iter()
                    .map(|t| NewTag {
                        //TODO There is something wrong
                        category: TagCategory::General,
                        value: t.clone(),
                    })
                    .collect();

                let id = services
                    .create_post
                    .execute(
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

pub async fn get_post<PR, PLR, TR, FR, FS>(
    services: web::Data<Services<PR, PLR, TR, FR, FS>>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: PlaylistRepository + Clone,
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

pub async fn delete_post<PR, PLR, TR, FR, FS>(
    services: web::Data<Services<PR, PLR, TR, FR, FS>>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: PlaylistRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let id = parse_uuid(&path.into_inner(), "post id")?;

    services
        .delete_post
        .execute(id)
        .await
        .map_err(|err| map_repo_error(err, "Post not found", "posts.delete"))?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn update_post<PR, PLR, TR, FR, FS>(
    services: web::Data<Services<PR, PLR, TR, FR, FS>>,
    path: web::Path<String>,
    payload: web::Json<UpdatePost>,
) -> Result<HttpResponse, AppError>
where
    PR: PostRepository + Clone,
    PLR: PlaylistRepository + Clone,
    TR: TagRepository + Clone,
    FR: FileRepository + Clone,
    FS: FileStorage + Clone,
{
    let id = parse_uuid(&path.into_inner(), "post id")?;

    services
        .update_post
        .execute(id, payload.into_inner())
        .await
        .map_err(|err| map_repo_error(err, "Post not found", "posts.update"))?;

    Ok(HttpResponse::NoContent().finish())
}
