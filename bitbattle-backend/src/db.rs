use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Run the init migration
    let migration_sql = include_str!("../migrations/20241228_001_init.sql");

    // Check if tables already exist
    let table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'users')"
    )
    .fetch_one(pool)
    .await?;

    if !table_exists {
        sqlx::raw_sql(migration_sql).execute(pool).await?;
        tracing::info!("Database migrations completed successfully");
    } else {
        tracing::info!("Database tables already exist, skipping migrations");
    }

    Ok(())
}
