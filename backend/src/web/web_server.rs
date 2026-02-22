use actix_identity::IdentityMiddleware;
use actix_session::SessionMiddleware;
use actix_session::storage::CookieSessionStore;
use actix_web::{App, HttpServer, web};
use actix_web::cookie::Key;
use actix_web::middleware::Logger;
use actix_web::web::Data;
use crate::application::use_cases::services::Services;
use crate::domain::files::FileStorage;
use crate::domain::repository::{FileRepository, PlaylistRepository, PostRepository, TagRepository, UserRepository};
use crate::web::handlers::files::{download_file};
use crate::web::handlers::playlists::{create_playlist, delete_playlist, get_my_playlists, get_playlist_details};
use crate::web::handlers::posts::{create_post, get_post, search_posts};
use crate::web::handlers::tags::search_tags;
use crate::web::handlers::users::{get_current_user, login_user, logout_user, register_user};

pub async fn run_web_server<
    PR,
    TR,
    FR,
    PLR,
    UR,
    FS,
>(
    post_repo: PR,
    tag_repo: TR,
    file_repo: FR,
    playlist_repo: PLR,
    user_repo: UR,
    file_storage: FS,
    ip_address: String,
    port: u16,
    secret_key: String
) -> std::io::Result<()>
where
    PR:     PostRepository      + Clone + Send + Sync + 'static,
    TR:     TagRepository       + Clone + Send + Sync + 'static,
    FR:     FileRepository      + Clone + Send + Sync + 'static,
    PLR:    PlaylistRepository  + Clone + Send + Sync + 'static,
    UR:     UserRepository      + Clone + Send + Sync + 'static,
    FS:     FileStorage         + Clone + Send + Sync + 'static,
{
    let services = Services::new(
        post_repo,
        tag_repo,
        file_repo,
        file_storage,
    );

    let services_data = Data::new(services);


    let playlist_data = Data::new(playlist_repo);
    let user_data = Data::new(user_repo);
    
    let apply_key = Key::derive_from(secret_key.as_bytes());

    log::info!("binding web server to {ip_address}:{port}");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new(r#"%a "%r" %s %b "%{User-Agent}i" %T"#))
            .wrap(IdentityMiddleware::default())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                apply_key.clone()
            ))
            .app_data(services_data.clone())
            .app_data(playlist_data.clone())
            .app_data(user_data.clone())
            .service(
                web::scope("/api")

                    .service(
                        web::scope("/auth")
                            .route("me", web::get().to(get_current_user::<UR>))
                            .route("login", web::post().to(login_user::<UR>))
                            .route("register", web::post().to(register_user::<UR>))
                            .route("logout", web::post().to(logout_user))
                    )

                    .service(
                        web::scope("/playlists")
                            .route("", web::get().to(get_my_playlists::<PLR>))
                            .route("", web::post().to(create_playlist))
                            .route("/{id}", web::get().to(get_playlist_details))
                            .route("/{id}", web::delete().to(delete_playlist))
                    )

                    .service(
                        web::scope("/posts")
                            .route("", web::post().to(create_post::<PR, TR, FR, FS>))
                            .route("/search", web::post().to(search_posts::<PR, TR, FR, FS>))
                            .route("/{id}", web::get().to(get_post::<PR, TR, FR, FS>))
                    )

                    .service(
                        web::scope("/tags")
                        .route("/search", web::get().to(search_tags::<PR, TR, FR, FS>))
                        //TODO register get or create or delete it 
                    )

                    .service(
                        web::scope("/files")
                            .route("/{id}",web::get().to(download_file::<PR, TR, FR, FS>))
                    )
            )
    })
        .bind((ip_address, port))?
        .run()
        .await
}
