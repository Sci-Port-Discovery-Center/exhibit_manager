use crate::repo::{exhibit_repo, jotform_repo, part_repo};
use sqlx::Result as SqlxResult;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;
use std::path::Path;

/// Type alias for the connection pool.
pub type DbPool = SqlitePool;

/// Initializes a new database connection pool, creating the database file if
/// it doesn't exist yet.
pub async fn create_pool(db_path: &Path) -> Result<DbPool, sqlx::Error> {
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true);

    SqlitePool::connect_with(options).await
}

/// Sets up the database schema using a connection from the pool.
pub async fn setup_database(pool: &DbPool) -> SqlxResult<()> {
    exhibit_repo::create_exhibit_tables(pool).await?;
    jotform_repo::create_jotform_tables(pool).await?;
    part_repo::create_part_tables(pool).await?;

    Ok(())
}
