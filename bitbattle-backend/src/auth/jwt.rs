use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Access token claims (short-lived)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,        // User ID
    pub email: String,
    pub name: String,
    pub exp: i64,         // Expiry timestamp
    pub iat: i64,         // Issued at timestamp
    pub token_type: String, // "access" or "refresh"
}

/// Refresh token claims (long-lived)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshClaims {
    pub sub: Uuid,        // User ID
    pub token_id: Uuid,   // Unique token ID for revocation
    pub exp: i64,         // Expiry timestamp
    pub iat: i64,         // Issued at timestamp
    pub token_type: String, // Always "refresh"
}

/// Token pair response
#[derive(Debug, Serialize, Clone)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_in: i64,  // seconds
    pub refresh_token_expires_in: i64, // seconds
}

/// Create an access token (short-lived, default 15 minutes)
pub fn create_access_token(
    user_id: Uuid,
    email: &str,
    name: &str,
    secret: &str,
    expiry_minutes: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        name: name.to_string(),
        exp: (now + Duration::minutes(expiry_minutes)).timestamp(),
        iat: now.timestamp(),
        token_type: "access".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// Create a refresh token (long-lived, default 7 days)
pub fn create_refresh_token(
    user_id: Uuid,
    secret: &str,
    expiry_days: i64,
) -> Result<(String, Uuid), jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let token_id = Uuid::new_v4();

    let claims = RefreshClaims {
        sub: user_id,
        token_id,
        exp: (now + Duration::days(expiry_days)).timestamp(),
        iat: now.timestamp(),
        token_type: "refresh".to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok((token, token_id))
}

/// Create both access and refresh tokens
pub fn create_token_pair(
    user_id: Uuid,
    email: &str,
    name: &str,
    secret: &str,
    access_expiry_minutes: i64,
    refresh_expiry_days: i64,
) -> Result<(TokenPair, Uuid), jsonwebtoken::errors::Error> {
    let access_token = create_access_token(user_id, email, name, secret, access_expiry_minutes)?;
    let (refresh_token, token_id) = create_refresh_token(user_id, secret, refresh_expiry_days)?;

    Ok((TokenPair {
        access_token,
        refresh_token,
        access_token_expires_in: access_expiry_minutes * 60,
        refresh_token_expires_in: refresh_expiry_days * 24 * 60 * 60,
    }, token_id))
}

/// Legacy function for backwards compatibility
pub fn create_token(
    user_id: Uuid,
    email: &str,
    name: &str,
    secret: &str,
    expiry_hours: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    // Convert hours to minutes for the new function
    create_access_token(user_id, email, name, secret, expiry_hours * 60)
}

/// Validate an access token
pub fn validate_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

/// Validate a refresh token
pub fn validate_refresh_token(token: &str, secret: &str) -> Result<RefreshClaims, jsonwebtoken::errors::Error> {
    let token_data = decode::<RefreshClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    // Verify it's actually a refresh token
    if token_data.claims.token_type != "refresh" {
        return Err(jsonwebtoken::errors::Error::from(
            jsonwebtoken::errors::ErrorKind::InvalidToken
        ));
    }

    Ok(token_data.claims)
}
