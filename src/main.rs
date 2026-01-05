use sqlx::PgPool;
use crate::storage::postgres::PostgresPostRepository;

mod domain;
mod application;
mod storage;
mod file_storage;
mod web;

#[tokio::main]
async fn main() {
    let connect_link = "postgres://postgres:@localhost/glab";

    let pool = PgPool::connect(&connect_link)
        .await
        .unwrap();

    let mut repo =PostgresPostRepository::new(pool);
    
}
