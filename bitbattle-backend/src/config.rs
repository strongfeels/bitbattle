use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_uri: String,
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
    /// Access token expiry in minutes (default: 15)
    pub access_token_expiry_minutes: i64,
    /// Refresh token expiry in days (default: 7)
    pub refresh_token_expiry_days: i64,
    pub frontend_url: String,
    /// Comma-separated list of allowed CORS origins
    pub allowed_origins: Vec<String>,
    /// Database connection pool settings
    pub db_max_connections: u32,
    pub db_min_connections: u32,
    pub db_acquire_timeout_secs: u64,
    /// Whether to use JSON logging format (for production)
    pub json_logging: bool,
    /// Log level (trace, debug, info, warn, error)
    pub log_level: String,

    // AI Problem Generation settings
    /// Whether AI problem generation is enabled
    pub ai_problems_enabled: bool,
    /// AI provider to use: "openai" or "anthropic"
    pub ai_provider: String,
    /// OpenAI API key (optional)
    pub openai_api_key: Option<String>,
    /// OpenAI model to use
    pub openai_model: String,
    /// Minimum pool size for each difficulty
    pub ai_min_pool_easy: u32,
    pub ai_min_pool_medium: u32,
    pub ai_min_pool_hard: u32,
    /// Interval in seconds between generation checks
    pub ai_generation_interval_secs: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        let frontend_url = env::var("FRONTEND_URL")
            .unwrap_or_else(|_| "http://localhost:5173".to_string());

        // Parse allowed origins from env, fallback to frontend_url + common dev ports
        let allowed_origins = env::var("ALLOWED_ORIGINS")
            .map(|s| s.split(',').map(|o| o.trim().to_string()).collect())
            .unwrap_or_else(|_| {
                vec![
                    frontend_url.clone(),
                    "http://localhost:5173".to_string(),
                    "http://localhost:5174".to_string(),
                    "http://localhost:5175".to_string(),
                    "http://localhost:5176".to_string(),
                    "http://localhost:5177".to_string(),
                    "http://localhost:5178".to_string(),
                    "http://localhost:3000".to_string(),
                ]
            });

        Ok(Config {
            database_url: env::var("DATABASE_URL")?,
            google_client_id: env::var("GOOGLE_CLIENT_ID")?,
            google_client_secret: env::var("GOOGLE_CLIENT_SECRET")?,
            google_redirect_uri: env::var("GOOGLE_REDIRECT_URI")?,
            jwt_secret: env::var("JWT_SECRET")?,
            jwt_expiry_hours: env::var("JWT_EXPIRY_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .unwrap_or(24),
            access_token_expiry_minutes: env::var("ACCESS_TOKEN_EXPIRY_MINUTES")
                .unwrap_or_else(|_| "15".to_string())
                .parse()
                .unwrap_or(15),
            refresh_token_expiry_days: env::var("REFRESH_TOKEN_EXPIRY_DAYS")
                .unwrap_or_else(|_| "7".to_string())
                .parse()
                .unwrap_or(7),
            frontend_url,
            allowed_origins,
            db_max_connections: env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            db_min_connections: env::var("DB_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "2".to_string())
                .parse()
                .unwrap_or(2),
            db_acquire_timeout_secs: env::var("DB_ACQUIRE_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            json_logging: env::var("JSON_LOGGING")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false),
            log_level: env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "info".to_string()),

            // AI Problem Generation settings
            ai_problems_enabled: env::var("AI_PROBLEMS_ENABLED")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false),
            ai_provider: env::var("AI_PROVIDER")
                .unwrap_or_else(|_| "openai".to_string()),
            openai_api_key: env::var("OPENAI_API_KEY").ok(),
            openai_model: env::var("OPENAI_MODEL")
                .unwrap_or_else(|_| "gpt-4o-mini".to_string()),
            ai_min_pool_easy: env::var("AI_MIN_POOL_EASY")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            ai_min_pool_medium: env::var("AI_MIN_POOL_MEDIUM")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            ai_min_pool_hard: env::var("AI_MIN_POOL_HARD")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            ai_generation_interval_secs: env::var("AI_GENERATION_INTERVAL_SECS")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .unwrap_or(300),
        })
    }
}
