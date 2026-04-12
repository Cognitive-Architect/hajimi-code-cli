//! PostgreSQL connection pool with pgvector support
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect("postgres://hajimi:hajimi@localhost:5432/memory")
        .await
}
