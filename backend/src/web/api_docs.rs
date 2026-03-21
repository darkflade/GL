use actix_web::{HttpResponse, web};
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct ErrorBodyDoc {
    pub error: String,
}

#[derive(Serialize, ToSchema)]
pub enum TagCategoryDoc {
    Artist,
    Copyright,
    Character,
    General,
}

#[derive(Serialize, ToSchema)]
pub struct TagDoc {
    pub id: Uuid,
    pub name: String,
    pub category: TagCategoryDoc,
    pub count: i32,
}

#[derive(Serialize, ToSchema)]
pub struct NewTagDoc {
    pub category: TagCategoryDoc,
    pub name: String,
}

#[derive(Serialize, ToSchema)]
pub struct TagEditDoc {
    pub id: Uuid,
    pub category: TagCategoryDoc,
    pub name: String,
}

#[derive(Serialize, ToSchema)]
pub struct TagBatchUpdateDoc {
    pub events: Vec<TagUpdateEventDoc>,
}

#[derive(Serialize, ToSchema)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum TagUpdateEventDoc {
    Create { tags: Vec<NewTagDoc> },
    Edit { tags: Vec<TagEditDoc> },
    Remove { tag_ids: Vec<Uuid> },
}

#[derive(Serialize, ToSchema)]
pub struct TagRelationsBatchUpdateDoc {
    pub events: Vec<TagRelationUpdateEventDoc>,
}

#[derive(Serialize, ToSchema)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum TagRelationUpdateEventDoc {
    Link {
        parent_id: Uuid,
        child_ids: Vec<Uuid>,
    },
    Unlink {
        parent_id: Uuid,
        child_ids: Vec<Uuid>,
    },
    Alias {
        tag_id: Uuid,
        alias_ids: Vec<Uuid>,
    },
    Unalias {
        tag_id: Uuid,
        alias_ids: Vec<Uuid>,
    },
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PaginationModeDoc {
    Offset,
    Keyset,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum KeysetDirectionDoc {
    Next,
    Prev,
}

#[derive(Serialize, ToSchema)]
pub struct KeysetPageCursorDoc {
    pub mode: PaginationModeDoc,
    pub direction: KeysetDirectionDoc,
    pub last_id: Uuid,
    pub last_score: f64,
    pub limit: i64,
}

#[derive(Serialize, ToSchema)]
pub struct SearchCursorDoc {
    pub mode: Option<PaginationModeDoc>,
    pub page: Option<i64>,
    pub last_id: Option<Uuid>,
    pub last_score: Option<f64>,
    pub limit: Option<i64>,
    pub direction: Option<KeysetDirectionDoc>,
}

#[derive(Serialize, ToSchema)]
pub struct TagQueryDoc {
    pub must: Vec<String>,
    pub should: Vec<String>,
    pub must_not: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct SearchQueryDoc {
    pub text_query: Option<String>,
    pub tag_query: Option<TagQueryDoc>,
    pub cursor: Option<SearchCursorDoc>,
}

#[derive(Serialize, ToSchema)]
pub enum FileTypeDoc {
    Picture,
    Video,
    Audio,
}

#[derive(Serialize, ToSchema)]
pub enum ThumbSizeTypeDoc {
    Small,
    Medium,
    Large,
}

#[derive(Serialize, ToSchema)]
pub struct FileMetaDoc {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub extension: Option<String>,
    pub duration_ms: Option<u64>,
}

#[derive(Serialize, ToSchema)]
pub struct ThumbnailDoc {
    pub height: u32,
    pub weight: u32,
    pub path: String,
    pub size_type: ThumbSizeTypeDoc,
    pub created_at: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct FileDoc {
    pub id: Uuid,
    pub path: String,
    pub hash: Option<String>,
    pub media_type: FileTypeDoc,
    pub meta: Option<FileMetaDoc>,
    pub created_at: Option<String>,
    pub thumbnail: Option<ThumbnailDoc>,
}

#[derive(Serialize, ToSchema)]
pub struct PostNoteDoc {
    pub id: Uuid,
    pub text: String,
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, ToSchema)]
pub struct PostDoc {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub file: FileDoc,
    pub tags: Vec<TagDoc>,
    pub notes: Vec<PostNoteDoc>,
}

#[derive(Serialize, ToSchema)]
pub struct SearchPostsResponseDoc {
    pub posts: Vec<PostDoc>,
    pub total_pages: Option<i64>,
    pub has_next: Option<bool>,
    pub has_prev: Option<bool>,
    pub next_cursor: Option<KeysetPageCursorDoc>,
    pub prev_cursor: Option<KeysetPageCursorDoc>,
}

#[derive(Serialize, ToSchema)]
pub struct UpdatePostNoteDoc {
    pub id: Option<Uuid>,
    pub text: String,
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, ToSchema)]
pub struct UpdatePostDoc {
    pub title: Option<String>,
    pub description: Option<String>,
    pub add_tag_ids: Option<Vec<Uuid>>,
    pub remove_tag_ids: Option<Vec<Uuid>>,
    pub notes: Option<Vec<UpdatePostNoteDoc>>,
}

#[derive(Serialize, ToSchema)]
pub struct CreatePostMultipartDoc {
    pub meta: String,
    #[schema(value_type = String, format = Binary)]
    pub file: String,
}

#[derive(Serialize, ToSchema)]
pub struct NewPlaylistDoc {
    pub title: String,
    pub description: Option<String>,
    pub tag_ids: Option<Vec<Uuid>>,
    pub cover: Option<Uuid>,
    pub items: Option<Vec<NewPlaylistItemDoc>>,
}

#[derive(Serialize, ToSchema)]
pub struct NewPlaylistItemDoc {
    pub position: u32,
    pub content: NewPlaylistItemContentDoc,
}

#[derive(Serialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NewPlaylistItemContentDoc {
    Post { post_id: Uuid },
    Note { text: String },
}

#[derive(Serialize, ToSchema)]
pub struct UpdatePlaylistDoc {
    pub title: Option<String>,
    pub description: Option<String>,
    pub add_tag_ids: Option<Vec<Uuid>>,
    pub remove_tag_ids: Option<Vec<Uuid>>,
    pub cover: Option<Uuid>,
    pub item_events: Option<Vec<PlaylistItemEventDoc>>,
}

#[derive(Serialize, ToSchema)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum PlaylistItemEventDoc {
    Add {
        after_id: Option<Uuid>,
        content: NewPlaylistItemContentDoc,
    },
    Edit {
        item_id: Uuid,
        content: NewPlaylistItemContentDoc,
    },
    Remove {
        item_id: Uuid,
    },
    Move {
        item_id: Uuid,
        after_id: Option<Uuid>,
    },
}

#[derive(Serialize, ToSchema)]
pub enum PlaylistContentDoc {
    Post(PostDoc),
    Note(String),
}

#[derive(Serialize, ToSchema)]
pub struct PlaylistItemDoc {
    pub id: Uuid,
    pub position: u32,
    pub content: PlaylistContentDoc,
}

#[derive(Serialize, ToSchema)]
pub struct PlaylistDoc {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<TagDoc>,
    pub cover: Option<Uuid>,
    pub items: Vec<PlaylistItemDoc>,
}

#[derive(Serialize, ToSchema)]
pub struct PlaylistSummaryDoc {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub cover: Option<Uuid>,
    pub item_count: i64,
    pub tags: Vec<TagDoc>,
}

#[derive(Serialize, ToSchema)]
pub struct SearchPlaylistsResponseDoc {
    pub playlists: Vec<PlaylistSummaryDoc>,
    pub has_next: bool,
    pub has_prev: bool,
    pub next_cursor: Option<KeysetPageCursorDoc>,
    pub prev_cursor: Option<KeysetPageCursorDoc>,
}

#[utoipa::path(
    get,
    path = "/api/tags/search",
    params(
        ("query" = String, Query, description = "Search query string", example = "landscape")
    ),
    responses(
        (status = 200, description = "Tags list", body = [TagDoc]),
        (status = 400, description = "Bad request", body = ErrorBodyDoc),
        (status = 500, description = "Internal server error", body = ErrorBodyDoc)
    ),
    tag = "tags"
)]
fn search_tags_docs() {}

#[utoipa::path(
    get,
    path = "/api/tags/{id}/related",
    params(
        ("id" = Uuid, Path, description = "Tag id")
    ),
    responses(
        (status = 200, description = "Related tags", body = [TagDoc]),
        (status = 400, description = "Bad request", body = ErrorBodyDoc),
        (status = 404, description = "Tag not found", body = ErrorBodyDoc),
        (status = 500, description = "Internal server error", body = ErrorBodyDoc)
    ),
    tag = "tags"
)]
fn get_related_tags_docs() {}

#[utoipa::path(
    patch,
    path = "/api/tags",
    request_body = TagBatchUpdateDoc,
    responses(
        (status = 204, description = "Tags updated"),
        (status = 400, description = "Bad request", body = ErrorBodyDoc),
        (status = 404, description = "Tag not found", body = ErrorBodyDoc),
        (status = 409, description = "Conflict", body = ErrorBodyDoc),
        (status = 500, description = "Internal server error", body = ErrorBodyDoc)
    ),
    tag = "tags"
)]
fn update_tags_docs() {}

#[utoipa::path(
    patch,
    path = "/api/tags/relations",
    request_body = TagRelationsBatchUpdateDoc,
    responses(
        (status = 204, description = "Tag relations updated"),
        (status = 400, description = "Bad request", body = ErrorBodyDoc),
        (status = 404, description = "Tag not found", body = ErrorBodyDoc),
        (status = 409, description = "Conflict", body = ErrorBodyDoc),
        (status = 500, description = "Internal server error", body = ErrorBodyDoc)
    ),
    tag = "tags"
)]
fn update_tag_relations_docs() {}

#[utoipa::path(
    post,
    path = "/api/posts/search",
    request_body = SearchQueryDoc,
    responses(
        (status = 200, description = "Posts search result", body = SearchPostsResponseDoc),
        (status = 400, description = "Bad request", body = ErrorBodyDoc),
        (status = 500, description = "Internal server error", body = ErrorBodyDoc)
    ),
    tag = "posts"
)]
fn search_posts_docs() {}

#[utoipa::path(
    post,
    path = "/api/posts",
    request_body(
        content = CreatePostMultipartDoc,
        content_type = "multipart/form-data",
        description = "Multipart body with `meta` JSON string and `file` binary"
    ),
    responses(
        (status = 201, description = "Created post id", body = Uuid),
        (status = 400, description = "Bad request", body = ErrorBodyDoc),
        (status = 500, description = "Internal server error", body = ErrorBodyDoc)
    ),
    tag = "posts"
)]
fn create_post_docs() {}

#[utoipa::path(
    get,
    path = "/api/posts/{id}",
    params(
        ("id" = Uuid, Path, description = "Post id")
    ),
    responses(
        (status = 200, description = "Post details", body = PostDoc),
        (status = 400, description = "Bad request", body = ErrorBodyDoc),
        (status = 404, description = "Post not found", body = ErrorBodyDoc),
        (status = 500, description = "Internal server error", body = ErrorBodyDoc)
    ),
    tag = "posts"
)]
fn get_post_docs() {}

#[utoipa::path(
    delete,
    path = "/api/posts/{id}",
    params(
        ("id" = Uuid, Path, description = "Post id")
    ),
    responses(
        (status = 204, description = "Post deleted"),
        (status = 400, description = "Bad request", body = ErrorBodyDoc),
        (status = 404, description = "Post not found", body = ErrorBodyDoc),
        (status = 500, description = "Internal server error", body = ErrorBodyDoc)
    ),
    tag = "posts"
)]
fn delete_post_docs() {}

#[utoipa::path(
    patch,
    path = "/api/posts/{id}",
    request_body = UpdatePostDoc,
    params(
        ("id" = Uuid, Path, description = "Post id")
    ),
    responses(
        (status = 204, description = "Post updated"),
        (status = 400, description = "Bad request", body = ErrorBodyDoc),
        (status = 404, description = "Post not found", body = ErrorBodyDoc),
        (status = 500, description = "Internal server error", body = ErrorBodyDoc)
    ),
    tag = "posts"
)]
fn update_post_docs() {}

#[utoipa::path(
    post,
    path = "/api/playlists/search",
    request_body = SearchQueryDoc,
    responses(
        (status = 200, description = "Playlists search result", body = SearchPlaylistsResponseDoc),
        (status = 400, description = "Bad request", body = ErrorBodyDoc),
        (status = 401, description = "Unauthorized", body = ErrorBodyDoc),
        (status = 500, description = "Internal server error", body = ErrorBodyDoc)
    ),
    tag = "playlists"
)]
fn search_playlists_docs() {}

#[utoipa::path(
    post,
    path = "/api/playlists",
    request_body = NewPlaylistDoc,
    responses(
        (status = 201, description = "Created playlist id", body = Uuid),
        (status = 400, description = "Bad request", body = ErrorBodyDoc),
        (status = 401, description = "Unauthorized", body = ErrorBodyDoc),
        (status = 500, description = "Internal server error", body = ErrorBodyDoc)
    ),
    tag = "playlists"
)]
fn create_playlist_docs() {}

#[utoipa::path(
    get,
    path = "/api/playlists/{id}",
    params(
        ("id" = Uuid, Path, description = "Playlist id")
    ),
    responses(
        (status = 200, description = "Playlist details", body = PlaylistDoc),
        (status = 400, description = "Bad request", body = ErrorBodyDoc),
        (status = 401, description = "Unauthorized", body = ErrorBodyDoc),
        (status = 404, description = "Playlist not found", body = ErrorBodyDoc),
        (status = 500, description = "Internal server error", body = ErrorBodyDoc)
    ),
    tag = "playlists"
)]
fn get_playlist_details_docs() {}

#[utoipa::path(
    delete,
    path = "/api/playlists/{id}",
    params(
        ("id" = Uuid, Path, description = "Playlist id")
    ),
    responses(
        (status = 204, description = "Playlist deleted"),
        (status = 400, description = "Bad request", body = ErrorBodyDoc),
        (status = 401, description = "Unauthorized", body = ErrorBodyDoc),
        (status = 404, description = "Playlist not found", body = ErrorBodyDoc),
        (status = 500, description = "Internal server error", body = ErrorBodyDoc)
    ),
    tag = "playlists"
)]
fn delete_playlist_docs() {}

#[utoipa::path(
    patch,
    path = "/api/playlists/{id}",
    request_body = UpdatePlaylistDoc,
    params(
        ("id" = Uuid, Path, description = "Playlist id")
    ),
    responses(
        (status = 204, description = "Playlist updated"),
        (status = 400, description = "Bad request", body = ErrorBodyDoc),
        (status = 401, description = "Unauthorized", body = ErrorBodyDoc),
        (status = 404, description = "Playlist not found", body = ErrorBodyDoc),
        (status = 500, description = "Internal server error", body = ErrorBodyDoc)
    ),
    tag = "playlists"
)]
fn update_playlist_docs() {}

#[derive(OpenApi)]
#[openapi(
    paths(
        search_tags_docs,
        get_related_tags_docs,
        update_tags_docs,
        update_tag_relations_docs
    ),
    components(schemas(
        TagDoc,
        NewTagDoc,
        TagEditDoc,
        TagBatchUpdateDoc,
        TagUpdateEventDoc,
        TagRelationsBatchUpdateDoc,
        TagRelationUpdateEventDoc,
        TagCategoryDoc,
        ErrorBodyDoc
    )),
    tags(
        (name = "tags", description = "Tag operations")
    )
)]
pub struct TagsApiDoc;

#[derive(OpenApi)]
#[openapi(
    paths(
        search_posts_docs,
        create_post_docs,
        get_post_docs,
        delete_post_docs,
        update_post_docs
    ),
    components(schemas(
        ErrorBodyDoc,
        TagCategoryDoc,
        TagDoc,
        PaginationModeDoc,
        KeysetDirectionDoc,
        KeysetPageCursorDoc,
        SearchCursorDoc,
        TagQueryDoc,
        SearchQueryDoc,
        FileTypeDoc,
        ThumbSizeTypeDoc,
        FileMetaDoc,
        ThumbnailDoc,
        FileDoc,
        PostNoteDoc,
        PostDoc,
        SearchPostsResponseDoc,
        UpdatePostNoteDoc,
        UpdatePostDoc,
        CreatePostMultipartDoc
    )),
    tags(
        (name = "posts", description = "Post operations")
    )
)]
pub struct PostsApiDoc;

#[derive(OpenApi)]
#[openapi(
    paths(
        search_playlists_docs,
        create_playlist_docs,
        get_playlist_details_docs,
        delete_playlist_docs,
        update_playlist_docs
    ),
    components(schemas(
        ErrorBodyDoc,
        TagCategoryDoc,
        TagDoc,
        PaginationModeDoc,
        KeysetDirectionDoc,
        KeysetPageCursorDoc,
        SearchCursorDoc,
        TagQueryDoc,
        SearchQueryDoc,
        FileTypeDoc,
        ThumbSizeTypeDoc,
        FileMetaDoc,
        ThumbnailDoc,
        FileDoc,
        PostNoteDoc,
        PostDoc,
        NewPlaylistDoc,
        NewPlaylistItemDoc,
        NewPlaylistItemContentDoc,
        UpdatePlaylistDoc,
        PlaylistItemEventDoc,
        PlaylistContentDoc,
        PlaylistItemDoc,
        PlaylistDoc,
        PlaylistSummaryDoc,
        SearchPlaylistsResponseDoc
    )),
    tags(
        (name = "playlists", description = "Playlist operations")
    )
)]
pub struct PlaylistsApiDoc;

pub fn generate_openapi_yaml() -> Result<String, serde_yaml::Error> {
    generate_service_openapi_yaml("tags").map(|yaml| yaml.expect("tags service must exist"))
}

fn generate_service_openapi(service_name: &str) -> Option<utoipa::openapi::OpenApi> {
    match service_name {
        "tags" => Some(TagsApiDoc::openapi()),
        "posts" => Some(PostsApiDoc::openapi()),
        "playlists" => Some(PlaylistsApiDoc::openapi()),
        _ => None,
    }
}

fn generate_service_openapi_yaml(service_name: &str) -> Result<Option<String>, serde_yaml::Error> {
    match generate_service_openapi(service_name) {
        Some(openapi) => serde_yaml::to_string(&openapi).map(Some),
        None => Ok(None),
    }
}

pub async fn get_openapi_yaml() -> HttpResponse {
    match generate_service_openapi_yaml("tags") {
        Ok(Some(yaml)) => HttpResponse::Ok()
            .content_type("application/yaml; charset=utf-8")
            .body(yaml),
        Ok(None) => HttpResponse::NotFound().json(ErrorBodyDoc {
            error: "unknown docs service: tags".to_string(),
        }),
        Err(err) => HttpResponse::InternalServerError().json(ErrorBodyDoc {
            error: format!("failed to generate openapi yaml: {err}"),
        }),
    }
}

pub async fn get_openapi_service_yaml(service_name: web::Path<String>) -> HttpResponse {
    let service_name = service_name.into_inner().to_lowercase();

    match generate_service_openapi_yaml(&service_name) {
        Ok(Some(yaml)) => HttpResponse::Ok()
            .content_type("application/yaml; charset=utf-8")
            .body(yaml),
        Ok(None) => HttpResponse::NotFound().json(ErrorBodyDoc {
            error: format!("unknown docs service: {service_name}"),
        }),
        Err(err) => HttpResponse::InternalServerError().json(ErrorBodyDoc {
            error: format!("failed to generate openapi yaml: {err}"),
        }),
    }
}

pub async fn get_openapi_service_swagger_html(service_name: web::Path<String>) -> HttpResponse {
    let service_name = service_name.into_inner().to_lowercase();

    if generate_service_openapi(&service_name).is_none() {
        return HttpResponse::NotFound().json(ErrorBodyDoc {
            error: format!("unknown docs service: {service_name}"),
        });
    }

    let html = format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>{service} API docs</title>
  <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css" />
  <style>
    body {{ margin: 0; }}
    #swagger-ui {{ max-width: 1200px; margin: 0 auto; }}
  </style>
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
  <script>
    window.ui = SwaggerUIBundle({{
      url: '/api/docs/files/{service}',
      dom_id: '#swagger-ui'
    }});
  </script>
</body>
</html>
"#,
        service = service_name
    );

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

pub async fn get_docs_index_html() -> HttpResponse {
    let html = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>API docs</title>
  <style>
    body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; margin: 2rem; }
    h1 { margin-top: 0; }
    ul { line-height: 1.8; }
  </style>
</head>
<body>
  <h1>API docs</h1>
  <ul>
    <li><a href="/api/docs/files/tags">Tags OpenAPI YAML</a></li>
    <li><a href="/api/docs/plain/tags">Tags Swagger HTML</a></li>
    <li><a href="/api/docs/files/posts">Posts OpenAPI YAML</a></li>
    <li><a href="/api/docs/plain/posts">Posts Swagger HTML</a></li>
    <li><a href="/api/docs/files/playlists">Playlists OpenAPI YAML</a></li>
    <li><a href="/api/docs/plain/playlists">Playlists Swagger HTML</a></li>
  </ul>
</body>
</html>
"#;

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}
