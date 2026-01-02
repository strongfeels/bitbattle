use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

impl RefreshToken {
    /// Store a new refresh token
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        token_id: Uuid,
        expires_at: DateTime<Utc>,
        user_agent: Option<&str>,
        ip_address: Option<&str>,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO refresh_tokens (user_id, token_id, expires_at, user_agent, ip_address)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(token_id)
        .bind(expires_at)
        .bind(user_agent)
        .bind(ip_address)
        .fetch_one(pool)
        .await
    }

    /// Find a refresh token by its token_id (the jti claim)
    pub async fn find_by_token_id(pool: &PgPool, token_id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT * FROM refresh_tokens
            WHERE token_id = $1 AND revoked_at IS NULL AND expires_at > NOW()
            "#,
        )
        .bind(token_id)
        .fetch_optional(pool)
        .await
    }

    /// Revoke a specific refresh token
    pub async fn revoke(pool: &PgPool, token_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = NOW()
            WHERE token_id = $1 AND revoked_at IS NULL
            "#,
        )
        .bind(token_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Revoke all refresh tokens for a user (logout from all devices)
    pub async fn revoke_all_for_user(pool: &PgPool, user_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = NOW()
            WHERE user_id = $1 AND revoked_at IS NULL
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Clean up expired tokens
    pub async fn cleanup_expired(pool: &PgPool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM refresh_tokens WHERE expires_at < NOW()
            "#,
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Check if a token is valid (exists, not revoked, not expired)
    pub async fn is_valid(pool: &PgPool, token_id: Uuid) -> Result<bool, sqlx::Error> {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM refresh_tokens
                WHERE token_id = $1 AND revoked_at IS NULL AND expires_at > NOW()
            )
            "#,
        )
        .bind(token_id)
        .fetch_one(pool)
        .await?;

        Ok(exists)
    }

    /// Get all active sessions for a user
    pub async fn get_user_sessions(pool: &PgPool, user_id: Uuid) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT * FROM refresh_tokens
            WHERE user_id = $1 AND revoked_at IS NULL AND expires_at > NOW()
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }
}
