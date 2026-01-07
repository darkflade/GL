use sqlx::postgres::PgPoolOptions;
use crate::storage::files::LocalFileStorage;
use crate::web::web_server;
use crate::storage::postgres::posts::PostgresPostRepository;
use crate::storage::postgres::tags::PostgresTagRepository;
use crate::storage::postgres::playlists::PostgresPlaylistRepository;
use crate::storage::postgres::users::PostgresUserRepository;

mod domain;
mod application;
mod storage;
mod file_storage;
mod web;

#[tokio::main]
async fn main() {

    dotenv::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let server_ip_address = std::env::var("SERVER_IP")
        .unwrap_or_else(|_| "localhost".to_string());
    let server_port = std::env::var("SERVER_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(8080);
    let secret_key = std::env::var("SECRET_KEY").expect("SECRET_KEY");
    //let connect_link = "postgres://postgres:@localhost/glab";

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .expect("Failed to connect to Postgres");

    let post_repo = PostgresPostRepository::new(pool.clone());
    let tag_repo = PostgresTagRepository::new(pool.clone());
    let playlist_repo = PostgresPlaylistRepository::new(pool.clone());
    let user_repo = PostgresUserRepository::new(pool.clone());
    let file_storage = LocalFileStorage::new("./gl_posts");

    println!("Server running at http://{}:{}",server_ip_address, server_port);
    web_server::run_web_server(post_repo, tag_repo, playlist_repo, user_repo, file_storage, server_ip_address, server_port, secret_key).await.expect("Failed to run web server");
    
}
