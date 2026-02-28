use crate::storage::postgres::files::PostgresFileRepository;
use crate::storage::postgres::playlists::PostgresPlaylistRepository;
use crate::storage::postgres::posts::PostgresPostRepository;
use crate::storage::postgres::tags::PostgresTagRepository;
use crate::storage::postgres::users::PostgresUserRepository;
use crate::web::web_server;
use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use storage::file_storage::files::LocalFileStorage;

mod application;
mod domain;
mod logging;
mod storage;
mod web;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    logging::init_logger().context("failed to initialize logger")?;

    let db_url = std::env::var("DATABASE_URL").context("DATABASE_URL is not set")?;
    let server_ip_address = std::env::var("BACKEND_IP").unwrap_or_else(|_| "localhost".to_string());
    let server_port = std::env::var("BACKEND_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(8080);
    let secret_key =
        std::env::var("BACKEND_COOKIE_SECRET").context("BACKEND_COOKIE_SECRET is not set")?;

    log::info!("connecting to postgres");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .context("failed to connect to postgres")?;

    let post_repo = PostgresPostRepository::new(pool.clone());
    let tag_repo = PostgresTagRepository::new(pool.clone());
    let file_repo = PostgresFileRepository::new(pool.clone());
    let playlist_repo = PostgresPlaylistRepository::new(pool.clone());
    let user_repo = PostgresUserRepository::new(pool.clone());
    let file_storage = LocalFileStorage::new("./gl_posts");

    log::info!(
        "server startup complete, listening on http://{}:{}",
        server_ip_address,
        server_port
    );
    web_server::run_web_server(
        post_repo,
        playlist_repo,
        tag_repo,
        file_repo,
        user_repo,
        file_storage,
        server_ip_address,
        server_port,
        secret_key,
    )
    .await
    .context("failed to run web server")?;

    Ok(())
}
