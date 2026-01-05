use actix_web::{web, App, HttpServer};
use actix_web::web::Data;
use async_trait::async_trait;
use sqlx::PgPool;
use crate::domain::files::FileStorage;
use crate::domain::repository::{PlaylistRepository, PostRepository, TagRepository};
use crate::storage::files::LocalFileStorage;
use crate::storage::postgres::PostgresPostRepository;

pub async fn run_web_server<
    PR,
    TR,
    PLR,
    FS
>(
    post_repo: PR,
    tag_repo: TR,
    playlist_repo: PLR,
    file_storage: FS,
    ip_address: String,
    port: u16,
) -> std::io::Result<()>
where
    PR: PostRepository + Send + Sync + 'static,
    TR: TagRepository + Send + Sync + 'static,
    PLR: PlaylistRepository + Send + Sync + 'static,
    FS: FileStorage + Send + Sync + 'static,
{
    let post_data = Data::new(post_repo);
    let tag_data = Data::new(tag_repo);
    let playlist_data = Data::new(playlist_repo);
    let files_data = Data::new(file_storage);


    HttpServer::new(move || {
        App::new()
            .app_data(post_data.clone())
            .app_data(tag_data.clone())
            .app_data(playlist_data.clone())
            .app_data(files_data.clone())
            /*
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/auth")
                            .route("me", web::get().to(get_user_info))
                            .route("login", web::post().to(login_user))
                            .route("register", web::post().to(register_user))
                    )
                    .service(
                        web::scope("/playlists")
                            .route("", web::get().to(get_my_playlist))
                            .route("", web::post().to(create_playlist))
                            .route("/{id}", web::get().to(get_playlist_details))
                            .route("/{id}", web::delete().to(delete_playlist))
                    )
                    .service(
                        web::scope("/posts")
                            .route("", web::get().to(search_posts))
                            .route("", web::post().to(create_post))
                            .route("/{id}", web::get().to(get_post))
                    )
                    .service(
                        web::scope("/files/{id}").to(download_file)
                    )
            )

             */
    })
        .bind((ip_address, port))?
        .run()
        .await
}