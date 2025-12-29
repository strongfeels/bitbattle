use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
    Json,
};
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret,
    CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};

use crate::auth::{jwt::create_token, AuthUser};
use crate::models::User;
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
    pub token: String,
    pub user: UserResponse,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
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

    // Create JWT
    let jwt = match create_token(
        user.id,
        &user.email,
        &user.display_name,
        &state.config.jwt_secret,
        state.config.jwt_expiry_hours,
    ) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to create JWT: {:?}", e);
            return Redirect::temporary(&format!(
                "{}?error=token_error",
                state.config.frontend_url
            ));
        }
    };

    tracing::info!("User {} logged in successfully (new: {})", user.display_name, is_new_user);

    // Redirect to frontend with token (and newUser flag if applicable)
    let redirect_url = if is_new_user {
        format!("{}?token={}&newUser=true", state.config.frontend_url, jwt)
    } else {
        format!("{}?token={}", state.config.frontend_url, jwt)
    };
    Redirect::temporary(&redirect_url)
}

// GET /auth/me - Get current user
pub async fn get_current_user(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    match User::find_by_id(&state.db_pool, auth_user.user_id).await {
        Ok(Some(user)) => Json(UserResponse {
            id: user.id.to_string(),
            email: user.email,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
        })
        .into_response(),
        Ok(None) => (axum::http::StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    }
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
) -> impl IntoResponse {
    let username = request.username.trim();

    // Validate username
    if username.is_empty() || username.len() > 20 {
        return (axum::http::StatusCode::BAD_REQUEST, "Username must be 1-20 characters").into_response();
    }

    // Only allow alphanumeric, underscores, and hyphens
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return (axum::http::StatusCode::BAD_REQUEST, "Username can only contain letters, numbers, underscores, and hyphens").into_response();
    }

    match User::update_display_name(&state.db_pool, auth_user.user_id, username).await {
        Ok(_) => axum::Json(serde_json::json!({"success": true})).into_response(),
        Err(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Failed to update username").into_response(),
    }
}
