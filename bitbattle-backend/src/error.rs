use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;

/// Standard API error response format
#[derive(Debug, Serialize)]
pub struct ApiError {
    /// Error code for programmatic handling (e.g., "VALIDATION_ERROR", "NOT_FOUND")
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Optional field that caused the error (for validation errors)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    /// Optional additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            field: None,
            details: None,
        }
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

/// Application error enum for all possible errors
#[derive(Debug)]
pub enum AppError {
    // Authentication errors
    Unauthorized(String),
    InvalidToken(String),
    TokenExpired,
    SessionRevoked,

    // Authorization errors
    Forbidden(String),

    // Validation errors
    ValidationError { field: String, message: String },
    InvalidInput(String),

    // Resource errors
    NotFound { resource: String, id: String },
    AlreadyExists { resource: String, field: String },

    // Database errors
    DatabaseError(String),

    // External service errors
    ExternalServiceError { service: String, message: String },

    // Rate limiting
    RateLimitExceeded,

    // General errors
    InternalError(String),
    BadRequest(String),
}

impl AppError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::InvalidToken(_) => StatusCode::UNAUTHORIZED,
            AppError::TokenExpired => StatusCode::UNAUTHORIZED,
            AppError::SessionRevoked => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            AppError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound { .. } => StatusCode::NOT_FOUND,
            AppError::AlreadyExists { .. } => StatusCode::CONFLICT,
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ExternalServiceError { .. } => StatusCode::BAD_GATEWAY,
            AppError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
        }
    }

    /// Get the error code for this error
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::Unauthorized(_) => "UNAUTHORIZED",
            AppError::InvalidToken(_) => "INVALID_TOKEN",
            AppError::TokenExpired => "TOKEN_EXPIRED",
            AppError::SessionRevoked => "SESSION_REVOKED",
            AppError::Forbidden(_) => "FORBIDDEN",
            AppError::ValidationError { .. } => "VALIDATION_ERROR",
            AppError::InvalidInput(_) => "INVALID_INPUT",
            AppError::NotFound { .. } => "NOT_FOUND",
            AppError::AlreadyExists { .. } => "ALREADY_EXISTS",
            AppError::DatabaseError(_) => "DATABASE_ERROR",
            AppError::ExternalServiceError { .. } => "EXTERNAL_SERVICE_ERROR",
            AppError::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            AppError::InternalError(_) => "INTERNAL_ERROR",
            AppError::BadRequest(_) => "BAD_REQUEST",
        }
    }

    /// Convert to API error response
    pub fn to_api_error(&self) -> ApiError {
        match self {
            AppError::Unauthorized(msg) => ApiError::new(self.error_code(), msg),
            AppError::InvalidToken(msg) => ApiError::new(self.error_code(), msg),
            AppError::TokenExpired => ApiError::new(self.error_code(), "Token has expired"),
            AppError::SessionRevoked => ApiError::new(self.error_code(), "Session has been revoked"),
            AppError::Forbidden(msg) => ApiError::new(self.error_code(), msg),
            AppError::ValidationError { field, message } => {
                ApiError::new(self.error_code(), message).with_field(field)
            }
            AppError::InvalidInput(msg) => ApiError::new(self.error_code(), msg),
            AppError::NotFound { resource, id } => {
                ApiError::new(self.error_code(), format!("{} not found", resource))
                    .with_details(serde_json::json!({ "resource": resource, "id": id }))
            }
            AppError::AlreadyExists { resource, field } => {
                ApiError::new(self.error_code(), format!("{} already exists", resource))
                    .with_field(field)
            }
            AppError::DatabaseError(_) => {
                // Don't expose internal database errors
                ApiError::new(self.error_code(), "A database error occurred")
            }
            AppError::ExternalServiceError { service, .. } => {
                ApiError::new(self.error_code(), format!("Error communicating with {}", service))
            }
            AppError::RateLimitExceeded => {
                ApiError::new(self.error_code(), "Too many requests. Please slow down.")
            }
            AppError::InternalError(_) => {
                // Don't expose internal errors
                ApiError::new(self.error_code(), "An internal error occurred")
            }
            AppError::BadRequest(msg) => ApiError::new(self.error_code(), msg),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::InvalidToken(msg) => write!(f, "Invalid token: {}", msg),
            AppError::TokenExpired => write!(f, "Token expired"),
            AppError::SessionRevoked => write!(f, "Session revoked"),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AppError::ValidationError { field, message } => {
                write!(f, "Validation error on {}: {}", field, message)
            }
            AppError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            AppError::NotFound { resource, id } => write!(f, "{} not found: {}", resource, id),
            AppError::AlreadyExists { resource, field } => {
                write!(f, "{} already exists: {}", resource, field)
            }
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AppError::ExternalServiceError { service, message } => {
                write!(f, "External service error ({}): {}", service, message)
            }
            AppError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            AppError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let api_error = self.to_api_error();

        // Log the error (full details for internal errors)
        match &self {
            AppError::DatabaseError(msg) => tracing::error!("Database error: {}", msg),
            AppError::InternalError(msg) => tracing::error!("Internal error: {}", msg),
            AppError::ExternalServiceError { service, message } => {
                tracing::error!("External service error ({}): {}", service, message)
            }
            _ => tracing::warn!("API error: {}", self),
        }

        (status, Json(api_error)).into_response()
    }
}

// Implement From for common error types
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound {
                resource: "Record".to_string(),
                id: "unknown".to_string(),
            },
            _ => AppError::DatabaseError(err.to_string()),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        use jsonwebtoken::errors::ErrorKind;
        match err.kind() {
            ErrorKind::ExpiredSignature => AppError::TokenExpired,
            ErrorKind::InvalidToken => AppError::InvalidToken("Malformed token".to_string()),
            ErrorKind::InvalidSignature => AppError::InvalidToken("Invalid signature".to_string()),
            _ => AppError::InvalidToken(err.to_string()),
        }
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::ExternalServiceError {
            service: "External API".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::InvalidInput(format!("JSON parsing error: {}", err))
    }
}

// Convert validation errors
impl From<crate::validation::ValidationError> for AppError {
    fn from(err: crate::validation::ValidationError) -> Self {
        AppError::ValidationError {
            field: err.field.unwrap_or_else(|| "unknown".to_string()),
            message: err.error,
        }
    }
}

/// Result type alias for handlers
pub type AppResult<T> = Result<T, AppError>;

/// Helper functions for creating common errors
impl AppError {
    pub fn unauthorized(message: impl Into<String>) -> Self {
        AppError::Unauthorized(message.into())
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        AppError::Forbidden(message.into())
    }

    pub fn not_found(resource: impl Into<String>, id: impl Into<String>) -> Self {
        AppError::NotFound {
            resource: resource.into(),
            id: id.into(),
        }
    }

    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        AppError::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        AppError::BadRequest(message.into())
    }

    pub fn internal(message: impl Into<String>) -> Self {
        AppError::InternalError(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(AppError::unauthorized("test").status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(AppError::forbidden("test").status_code(), StatusCode::FORBIDDEN);
        assert_eq!(AppError::not_found("User", "123").status_code(), StatusCode::NOT_FOUND);
        assert_eq!(AppError::validation("email", "invalid").status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(AppError::RateLimitExceeded.status_code(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(AppError::unauthorized("test").error_code(), "UNAUTHORIZED");
        assert_eq!(AppError::TokenExpired.error_code(), "TOKEN_EXPIRED");
        assert_eq!(AppError::not_found("User", "123").error_code(), "NOT_FOUND");
    }

    #[test]
    fn test_api_error_serialization() {
        let error = AppError::validation("email", "Invalid email format");
        let api_error = error.to_api_error();

        assert_eq!(api_error.code, "VALIDATION_ERROR");
        assert_eq!(api_error.message, "Invalid email format");
        assert_eq!(api_error.field, Some("email".to_string()));
    }
}
