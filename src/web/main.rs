use actix_web::{App, HttpServer};
use actix_web::web::Data;
use sqlx::PgPool;
use crate::storage::files::LocalFileStorage;
use crate::storage::postgres::PostgresPostRepository;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = PgPool::connect("postgres://postgres:@localhost/glab")
        .await
        .unwrap();

    let repo = PostgresPostRepository::new(pool.clone());
    let files = LocalFileStorage::new("./gl_posts");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(repo.clone()))
            .app_data(Data::new(files.clone()))
            //.service(create_post)
            //.service(get_post)
            //.service(get_file)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}