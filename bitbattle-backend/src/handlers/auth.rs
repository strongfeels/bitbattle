use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
    Json,
};
use chrono::{Duration, Utc};
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret,
    CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};

use crate::auth::{jwt, AuthUser};
use crate::error::{AppError, AppResult};
use crate::models::{RefreshToken, User};
use crate::AppState;

#[derive(Deserialize)]
pub struct SetUsernameRequest {
    pub username: String,
}

#[derive(Deserialize)]
pub struct AuthCallbackQuery {
    pub code: String,
    #[allow(dead_code)]
    pub state: Option<String>,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_in: i64,
    pub refresh_token_expires_in: i64,
    pub user: UserResponse,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub access_token_expires_in: i64,
}

// GET /auth/google - Redirect to Google OAuth
pub async fn google_auth_redirect(State(state): State<AppState>) -> impl IntoResponse {
    let client = create_oauth_client(&state);

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    Redirect::temporary(auth_url.as_str())
}

// GET /auth/callback - Handle Google OAuth callback
pub async fn google_auth_callback(
    State(state): State<AppState>,
    Query(params): Query<AuthCallbackQuery>,
) -> impl IntoResponse {
    let client = create_oauth_client(&state);

    // Exchange code for token
    let token_result = client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(oauth2::reqwest::async_http_client)
        .await;

    let token = match token_result {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to exchange code: {:?}", e);
            return Redirect::temporary(&format!(
                "{}?error=auth_failed",
                state.config.frontend_url
            ));
        }
    };

    // Get user info from Google
    let user_info = get_google_user_info(token.access_token().secret()).await;
    let google_user = match user_info {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("Failed to get user info: {:?}", e);
            return Redirect::temporary(&format!(
                "{}?error=user_info_failed",
                state.config.frontend_url
            ));
        }
    };

    // Find or create user
    let (user, is_new_user) = match User::find_by_google_id(&state.db_pool, &google_user.id).await {
        Ok(Some(user)) => (user, false),
        Ok(None) => {
            // Create new user with temporary name
            match User::create(
                &state.db_pool,
                &google_user.id,
                &google_user.email,
                &google_user.name,
                google_user.picture.as_deref(),
            )
            .await
            {
                Ok(u) => (u, true),
                Err(e) => {
                    tracing::error!("Failed to create user: {:?}", e);
                    return Redirect::temporary(&format!(
                        "{}?error=db_error",
                        state.config.frontend_url
                    ));
                }
            }
        }
        Err(e) => {
            tracing::error!("Database error: {:?}", e);
            return Redirect::temporary(&format!(
                "{}?error=db_error",
                state.config.frontend_url
            ));
        }
    };

    // Create token pair
    let (token_pair, refresh_token_id) = match jwt::create_token_pair(
        user.id,
        &user.email,
        &user.display_name,
        &state.config.jwt_secret,
        state.config.access_token_expiry_minutes,
        state.config.refresh_token_expiry_days,
    ) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to create tokens: {:?}", e);
            return Redirect::temporary(&format!(
                "{}?error=token_error",
                state.config.frontend_url
            ));
        }
    };

    // Store refresh token in database
    let refresh_expires_at = Utc::now() + Duration::days(state.config.refresh_token_expiry_days);
    if let Err(e) = RefreshToken::create(
        &state.db_pool,
        user.id,
        refresh_token_id,
        refresh_expires_at,
        None, // user_agent - could extract from headers
        None, // ip_address - could extract from request
    ).await {
        tracing::error!("Failed to store refresh token: {:?}", e);
        // Continue anyway - the refresh token just won't work
    }

    tracing::info!("User {} logged in successfully (new: {})", user.display_name, is_new_user);

    // Redirect to frontend with tokens (and newUser flag if applicable)
    let redirect_url = if is_new_user {
        format!(
            "{}?access_token={}&refresh_token={}&newUser=true",
            state.config.frontend_url,
            token_pair.access_token,
            token_pair.refresh_token
        )
    } else {
        format!(
            "{}?access_token={}&refresh_token={}",
            state.config.frontend_url,
            token_pair.access_token,
            token_pair.refresh_token
        )
    };
    Redirect::temporary(&redirect_url)
}

// GET /auth/me - Get current user
pub async fn get_current_user(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> AppResult<Json<UserResponse>> {
    let user = User::find_by_id(&state.db_pool, auth_user.user_id)
        .await?
        .ok_or_else(|| AppError::not_found("User", auth_user.user_id.to_string()))?;

    Ok(Json(UserResponse {
        id: user.id.to_string(),
        email: user.email,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
    }))
}

fn create_oauth_client(state: &AppState) -> BasicClient {
    BasicClient::new(
        ClientId::new(state.config.google_client_id.clone()),
        Some(ClientSecret::new(state.config.google_client_secret.clone())),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).unwrap(),
        Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new(state.config.google_redirect_uri.clone()).unwrap())
}

#[derive(Deserialize)]
struct GoogleUserInfo {
    id: String,
    email: String,
    name: String,
    picture: Option<String>,
}

async fn get_google_user_info(access_token: &str) -> Result<GoogleUserInfo, reqwest::Error> {
    let client = reqwest::Client::new();
    client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await?
        .json::<GoogleUserInfo>()
        .await
}

// POST /auth/set-username - Set username for new users
pub async fn set_username(
    State(state): State<AppState>,
    auth_user: AuthUser,
    axum::Json(request): axum::Json<SetUsernameRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let username = request.username.trim();

    // Validate username
    if username.is_empty() || username.len() > 15 {
        return Err(AppError::validation("username", "Username must be 1-15 characters"));
    }

    // Only allow alphanumeric, underscores, and hyphens
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(AppError::validation(
            "username",
            "Username can only contain letters, numbers, underscores, and hyphens",
        ));
    }

    User::update_display_name(&state.db_pool, auth_user.user_id, username).await?;

    Ok(Json(serde_json::json!({"success": true})))
}

// POST /auth/refresh - Refresh access token using refresh token
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(request): Json<RefreshTokenRequest>,
) -> AppResult<Json<RefreshTokenResponse>> {
    // Validate the refresh token
    let claims = jwt::validate_refresh_token(&request.refresh_token, &state.config.jwt_secret)?;

    // Check if token exists in database and is not revoked
    let is_valid = RefreshToken::is_valid(&state.db_pool, claims.token_id).await?;
    if !is_valid {
        tracing::warn!("Refresh token not found or revoked: {}", claims.token_id);
        return Err(AppError::SessionRevoked);
    }

    // Get user info for the new access token
    let user = User::find_by_id(&state.db_pool, claims.sub)
        .await?
        .ok_or_else(|| {
            tracing::warn!("User not found for refresh token: {}", claims.sub);
            AppError::not_found("User", claims.sub.to_string())
        })?;

    // Create new access token
    let access_token = jwt::create_access_token(
        user.id,
        &user.email,
        &user.display_name,
        &state.config.jwt_secret,
        state.config.access_token_expiry_minutes,
    )?;

    Ok(Json(RefreshTokenResponse {
        access_token,
        access_token_expires_in: state.config.access_token_expiry_minutes * 60,
    }))
}

// POST /auth/logout - Revoke refresh token
pub async fn logout(
    State(state): State<AppState>,
    Json(request): Json<RefreshTokenRequest>,
) -> AppResult<Json<serde_json::Value>> {
    // Validate the refresh token to get the token_id
    let claims = match jwt::validate_refresh_token(&request.refresh_token, &state.config.jwt_secret) {
        Ok(c) => c,
        Err(_) => {
            // Token is invalid/expired - just return success since effectively logged out
            return Ok(Json(serde_json::json!({"success": true})));
        }
    };

    // Revoke the token
    RefreshToken::revoke(&state.db_pool, claims.token_id).await?;

    Ok(Json(serde_json::json!({"success": true})))
}

// POST /auth/logout-all - Revoke all refresh tokens for user
pub async fn logout_all(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> AppResult<Json<serde_json::Value>> {
    let count = RefreshToken::revoke_all_for_user(&state.db_pool, auth_user.user_id).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "sessions_revoked": count
    })))
}
