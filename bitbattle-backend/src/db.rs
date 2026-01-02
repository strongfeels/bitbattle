use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

use crate::config::Config;

pub async fn create_pool(config: &Config) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.db_max_connections)
        .min_connections(config.db_min_connections)
        .acquire_timeout(Duration::from_secs(config.db_acquire_timeout_secs))
        .idle_timeout(Duration::from_secs(600)) // 10 minutes
        .max_lifetime(Duration::from_secs(1800)) // 30 minutes
        .connect(&config.database_url)
        .await
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Run the init migration
    let init_sql = include_str!("../migrations/20241228_001_init.sql");
    let refresh_tokens_sql = include_str!("../migrations/20241229_002_refresh_tokens.sql");
    let ai_problems_sql = include_str!("../migrations/20250101_004_ai_problems.sql");

    // Check if users table exists (base migration)
    let users_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'users')"
    )
    .fetch_one(pool)
    .await?;

    if !users_exists {
        sqlx::raw_sql(init_sql).execute(pool).await?;
        tracing::info!("Initial database migration completed");
    } else {
        tracing::info!("Users table already exists, skipping initial migration");
    }

    // Check if refresh_tokens table exists
    let refresh_tokens_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'refresh_tokens')"
    )
    .fetch_one(pool)
    .await?;

    if !refresh_tokens_exists {
        sqlx::raw_sql(refresh_tokens_sql).execute(pool).await?;
        tracing::info!("Refresh tokens migration completed");
    } else {
        tracing::info!("Refresh tokens table already exists, skipping migration");
    }

    // Check if ai_problems table exists
    let ai_problems_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'ai_problems')"
    )
    .fetch_one(pool)
    .await?;

    if !ai_problems_exists {
        sqlx::raw_sql(ai_problems_sql).execute(pool).await?;
        tracing::info!("AI problems migration completed");
    } else {
        tracing::info!("AI problems table already exists, skipping migration");
    }

    Ok(())
}
