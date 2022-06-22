use std::env;

use bb8_postgres::bb8::Pool;
use bb8_postgres::tokio_postgres::NoTls;
use bb8_postgres::PostgresConnectionManager;

pub async fn create_pool() -> Pool<PostgresConnectionManager<NoTls>> {
    // connect to the database client
    let mangager =
        PostgresConnectionManager::new_from_stringlike(env::var("DATABASE_URL").unwrap(), NoTls)
            .unwrap();

    // Create a connection pool
    match Pool::builder().min_idle(Some(1)).build(mangager).await {
        Ok(pool) => pool,
        Err(e) => panic!("bb8 error {}", e),
    }
}
