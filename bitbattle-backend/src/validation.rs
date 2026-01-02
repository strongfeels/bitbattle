use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::collections::HashMap;

/// Validation error response
#[derive(Debug, Serialize)]
pub struct ValidationError {
    pub error: String,
    pub field: Option<String>,
    pub details: Option<HashMap<String, String>>,
}

impl ValidationError {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            field: None,
            details: None,
        }
    }

    pub fn field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    pub fn detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }
}

impl IntoResponse for ValidationError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}

/// Result type for validation
pub type ValidationResult<T> = Result<T, ValidationError>;

// ============================================================================
// Username Validation
// ============================================================================

pub const USERNAME_MIN_LENGTH: usize = 1;  // Allow single char usernames
pub const USERNAME_MAX_LENGTH: usize = 15;
pub const USERNAME_PATTERN: &str = "alphanumeric, underscore, hyphen";

/// Validates a username
pub fn validate_username(username: &str) -> ValidationResult<&str> {
    let username = username.trim();

    if username.is_empty() {
        return Err(ValidationError::new("Username is required").field("username"));
    }

    if username.len() < USERNAME_MIN_LENGTH {
        return Err(ValidationError::new(format!(
            "Username must be at least {} characters",
            USERNAME_MIN_LENGTH
        ))
        .field("username")
        .detail("min_length", USERNAME_MIN_LENGTH.to_string()));
    }

    if username.len() > USERNAME_MAX_LENGTH {
        return Err(ValidationError::new(format!(
            "Username must be at most {} characters",
            USERNAME_MAX_LENGTH
        ))
        .field("username")
        .detail("max_length", USERNAME_MAX_LENGTH.to_string()));
    }

    // Allow alphanumeric, underscore, hyphen
    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ValidationError::new(
            "Username can only contain letters, numbers, underscores, and hyphens",
        )
        .field("username")
        .detail("pattern", USERNAME_PATTERN.to_string()));
    }

    // Prevent usernames that look like system names
    let reserved = ["admin", "system", "bot", "moderator", "mod", "null", "undefined"];
    if reserved.contains(&username.to_lowercase().as_str()) {
        return Err(ValidationError::new("This username is reserved").field("username"));
    }

    Ok(username)
}

// ============================================================================
// Room Code Validation
// ============================================================================

pub const ROOM_CODE_MIN_LENGTH: usize = 4;
pub const ROOM_CODE_MAX_LENGTH: usize = 30;  // Room codes like SMART-MASTER-8418

/// Validates a room code
pub fn validate_room_code(room_code: &str) -> ValidationResult<String> {
    let room_code = room_code.trim().to_uppercase();

    if room_code.is_empty() {
        return Err(ValidationError::new("Room code is required").field("room_code"));
    }

    if room_code.len() < ROOM_CODE_MIN_LENGTH {
        return Err(ValidationError::new(format!(
            "Room code must be at least {} characters",
            ROOM_CODE_MIN_LENGTH
        ))
        .field("room_code"));
    }

    if room_code.len() > ROOM_CODE_MAX_LENGTH {
        return Err(ValidationError::new(format!(
            "Room code must be at most {} characters",
            ROOM_CODE_MAX_LENGTH
        ))
        .field("room_code"));
    }

    // Allow alphanumeric and hyphens only
    if !room_code
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-')
    {
        return Err(ValidationError::new(
            "Room code can only contain letters, numbers, and hyphens",
        )
        .field("room_code"));
    }

    Ok(room_code)
}

// ============================================================================
// Code Submission Validation
// ============================================================================

pub const CODE_MAX_LENGTH: usize = 100_000; // 100KB max
pub const CODE_MIN_LENGTH: usize = 1;

/// Supported programming languages
pub const SUPPORTED_LANGUAGES: &[&str] = &[
    "javascript",
    "python",
    "rust",
    "go",
    "java",
    "c",
    "cpp",
];

/// Validates submitted code
pub fn validate_code(code: &str) -> ValidationResult<&str> {
    if code.is_empty() {
        return Err(ValidationError::new("Code cannot be empty").field("code"));
    }

    if code.len() > CODE_MAX_LENGTH {
        return Err(ValidationError::new(format!(
            "Code exceeds maximum length of {} characters",
            CODE_MAX_LENGTH
        ))
        .field("code")
        .detail("max_length", CODE_MAX_LENGTH.to_string())
        .detail("current_length", code.len().to_string()));
    }

    // Check for null bytes (potential security issue)
    if code.contains('\0') {
        return Err(ValidationError::new("Code contains invalid characters").field("code"));
    }

    Ok(code)
}

/// Validates programming language
pub fn validate_language(language: &str) -> ValidationResult<&str> {
    let language = language.trim().to_lowercase();

    if language.is_empty() {
        return Err(ValidationError::new("Language is required").field("language"));
    }

    if !SUPPORTED_LANGUAGES.contains(&language.as_str()) {
        return Err(ValidationError::new(format!(
            "Unsupported language. Supported: {}",
            SUPPORTED_LANGUAGES.join(", ")
        ))
        .field("language")
        .detail("supported", SUPPORTED_LANGUAGES.join(", ")));
    }

    // Return the original reference if valid
    Ok(language.leak()) // This is fine for a small set of languages
}

// ============================================================================
// Problem ID Validation
// ============================================================================

pub const PROBLEM_ID_MAX_LENGTH: usize = 100;

/// Validates a problem ID
pub fn validate_problem_id(problem_id: &str) -> ValidationResult<&str> {
    let problem_id = problem_id.trim();

    if problem_id.is_empty() {
        return Err(ValidationError::new("Problem ID is required").field("problem_id"));
    }

    if problem_id.len() > PROBLEM_ID_MAX_LENGTH {
        return Err(ValidationError::new("Problem ID is too long").field("problem_id"));
    }

    // Allow alphanumeric, underscore, hyphen
    if !problem_id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ValidationError::new(
            "Problem ID can only contain letters, numbers, underscores, and hyphens",
        )
        .field("problem_id"));
    }

    Ok(problem_id)
}

// ============================================================================
// Matchmaking Validation
// ============================================================================

pub const VALID_DIFFICULTIES: &[&str] = &["easy", "medium", "hard", "any"];
pub const VALID_GAME_MODES: &[&str] = &["casual", "ranked"];
pub const MIN_PLAYERS: usize = 1;  // Allow solo practice
pub const MAX_PLAYERS: usize = 4;

/// Validates difficulty setting
pub fn validate_difficulty(difficulty: &str) -> ValidationResult<&str> {
    let difficulty = difficulty.trim().to_lowercase();

    if !VALID_DIFFICULTIES.contains(&difficulty.as_str()) {
        return Err(ValidationError::new(format!(
            "Invalid difficulty. Valid options: {}",
            VALID_DIFFICULTIES.join(", ")
        ))
        .field("difficulty"));
    }

    Ok(difficulty.leak())
}

/// Validates game mode
pub fn validate_game_mode(game_mode: &str) -> ValidationResult<&str> {
    let game_mode = game_mode.trim().to_lowercase();

    if !VALID_GAME_MODES.contains(&game_mode.as_str()) {
        return Err(ValidationError::new(format!(
            "Invalid game mode. Valid options: {}",
            VALID_GAME_MODES.join(", ")
        ))
        .field("game_mode"));
    }

    Ok(game_mode.leak())
}

/// Validates player count
pub fn validate_player_count(count: usize) -> ValidationResult<usize> {
    if count == 0 {
        return Err(ValidationError::new("At least 1 player required")
            .field("players")
            .detail("min", "1".to_string()));
    }

    if count < MIN_PLAYERS {
        return Err(ValidationError::new(format!(
            "Minimum {} players required",
            MIN_PLAYERS
        ))
        .field("players")
        .detail("min", MIN_PLAYERS.to_string()));
    }

    if count > MAX_PLAYERS {
        return Err(ValidationError::new(format!(
            "Maximum {} players allowed",
            MAX_PLAYERS
        ))
        .field("players")
        .detail("max", MAX_PLAYERS.to_string()));
    }

    Ok(count)
}

// ============================================================================
// Connection ID Validation
// ============================================================================

pub const CONNECTION_ID_MAX_LENGTH: usize = 100;

/// Validates a connection ID
pub fn validate_connection_id(connection_id: &str) -> ValidationResult<&str> {
    let connection_id = connection_id.trim();

    if connection_id.is_empty() {
        return Err(ValidationError::new("Connection ID is required").field("connection_id"));
    }

    if connection_id.len() > CONNECTION_ID_MAX_LENGTH {
        return Err(ValidationError::new("Connection ID is too long").field("connection_id"));
    }

    // Allow alphanumeric and underscore only
    if !connection_id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_')
    {
        return Err(ValidationError::new("Invalid connection ID format").field("connection_id"));
    }

    Ok(connection_id)
}

// ============================================================================
// Generic Validators
// ============================================================================

/// Validates that a string is not empty
pub fn validate_not_empty<'a>(value: &'a str, field_name: &str) -> ValidationResult<&'a str> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ValidationError::new(format!("{} is required", field_name)).field(field_name.to_string()));
    }
    Ok(value)
}

/// Validates string length
pub fn validate_length<'a>(
    value: &'a str,
    field_name: &str,
    min: Option<usize>,
    max: Option<usize>,
) -> ValidationResult<&'a str> {
    if let Some(min_len) = min {
        if value.len() < min_len {
            return Err(ValidationError::new(format!(
                "{} must be at least {} characters",
                field_name, min_len
            ))
            .field(field_name.to_string()));
        }
    }

    if let Some(max_len) = max {
        if value.len() > max_len {
            return Err(ValidationError::new(format!(
                "{} must be at most {} characters",
                field_name, max_len
            ))
            .field(field_name.to_string()));
        }
    }

    Ok(value)
}

/// Validates a UUID string
pub fn validate_uuid(value: &str, field_name: &str) -> ValidationResult<uuid::Uuid> {
    value.parse::<uuid::Uuid>().map_err(|_| {
        ValidationError::new(format!("Invalid {} format", field_name)).field(field_name)
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username_valid() {
        assert!(validate_username("player123").is_ok());
        assert!(validate_username("Player_123").is_ok());
        assert!(validate_username("cool-player").is_ok());
        assert!(validate_username("S").is_ok()); // single char allowed
        assert!(validate_username("ab").is_ok());
    }

    #[test]
    fn test_validate_username_invalid() {
        assert!(validate_username("").is_err()); // empty
        assert!(validate_username("player@123").is_err()); // invalid char
        assert!(validate_username("admin").is_err()); // reserved
        assert!(validate_username(&"a".repeat(16)).is_err()); // too long (max 15)
    }

    #[test]
    fn test_validate_room_code() {
        assert!(validate_room_code("ROOM-1234").is_ok());
        assert!(validate_room_code("abcd").is_ok());
        assert!(validate_room_code("SMART-MASTER-8418").is_ok());  // Generated room codes
        assert!(validate_room_code("BRAVE-CODER-4303").is_ok());
        assert!(validate_room_code("").is_err());
        assert!(validate_room_code("ab").is_err()); // too short
        assert!(validate_room_code(&"a".repeat(31)).is_err()); // too long (max 30)
    }

    #[test]
    fn test_validate_code() {
        assert!(validate_code("print('hello')").is_ok());
        assert!(validate_code("").is_err());
        assert!(validate_code(&"a".repeat(100_001)).is_err()); // too long
    }

    #[test]
    fn test_validate_language() {
        assert!(validate_language("python").is_ok());
        assert!(validate_language("JavaScript").is_ok()); // case insensitive
        assert!(validate_language("ruby").is_err()); // not supported
    }

    #[test]
    fn test_validate_difficulty() {
        assert!(validate_difficulty("easy").is_ok());
        assert!(validate_difficulty("MEDIUM").is_ok());
        assert!(validate_difficulty("any").is_ok());
        assert!(validate_difficulty("extreme").is_err());
    }

    #[test]
    fn test_validate_game_mode() {
        assert!(validate_game_mode("casual").is_ok());
        assert!(validate_game_mode("RANKED").is_ok());
        assert!(validate_game_mode("competitive").is_err());
    }

    #[test]
    fn test_validate_player_count() {
        assert!(validate_player_count(1).is_ok());  // Solo practice
        assert!(validate_player_count(2).is_ok());
        assert!(validate_player_count(4).is_ok());
        assert!(validate_player_count(0).is_err());  // No players
        assert!(validate_player_count(5).is_err());
    }
}
