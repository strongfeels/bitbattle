use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Token usage information from the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Response from an LLM completion
#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
}

/// Errors that can occur during LLM operations
#[derive(Debug, Error)]
pub enum LlmError {
    #[error("API request failed: {0}")]
    RequestFailed(String),

    #[error("Rate limited, retry after {0} seconds")]
    RateLimited(u32),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Content was filtered by safety systems")]
    ContentFiltered,

    #[error("Request timeout")]
    Timeout,
}

/// Trait for LLM providers (OpenAI, Anthropic, etc.)
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Get the provider name (e.g., "openai", "anthropic")
    fn name(&self) -> &'static str;

    /// Get the model being used (e.g., "gpt-4o-mini")
    fn model(&self) -> &str;

    /// Complete a chat request with system and user prompts
    async fn complete(&self, system_prompt: &str, user_prompt: &str) -> Result<LlmResponse, LlmError>;
}
