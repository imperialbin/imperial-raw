use std::env;

use bb8_postgres::bb8::{Pool, RunError};
use bb8_postgres::tokio_postgres::{self, NoTls};
use bb8_postgres::PostgresConnectionManager;

pub async fn create_pool() -> Pool<PostgresConnectionManager<NoTls>> {
    // connect to the database client
    let mangager =
        PostgresConnectionManager::new_from_stringlike(env::var("DATABASE_URL").unwrap(), NoTls)
            .unwrap();

    log::info!("Created pool manager");

    // Create a connection pool
    match Pool::builder().min_idle(Some(1)).build(mangager).await {
        Ok(pool) => pool,
        Err(e) => panic!("bb8 error {}", e),
    }
}

pub async fn make_db_req(
    id: &str,
    pool: Pool<PostgresConnectionManager<NoTls>>,
) -> Result<Option<String>, RunError<tokio_postgres::Error>> {
    // get a client from the pool
    let client = pool.get().await?;

    // prepare a statement
    let statement = client
        .prepare("SELECT content FROM documents WHERE id = $1")
        .await?;

    // query the database with the id
    let row = match client.query_opt(&statement, &[&id]).await? {
        Some(row) => row,
        None => return Ok(None),
    };

    Ok(Some(row.get::<&str, String>("content")))
}
