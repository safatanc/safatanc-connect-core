use anyhow::Result;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;

use crate::config::DatabaseConfig;

pub type DbPool = Arc<PgPool>;

/// Initialize the database connection pool
pub async fn init_db_pool(config: &DatabaseConfig) -> Result<DbPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.connection_string)
        .await?;

    // Run migrations if in development mode
    #[cfg(debug_assertions)]
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(Arc::new(pool))
}

/// Check database connection
pub async fn check_connection(pool: &PgPool) -> Result<()> {
    // Simple query to check if the database is responsive
    sqlx::query("SELECT 1").execute(pool).await?;
    Ok(())
}
