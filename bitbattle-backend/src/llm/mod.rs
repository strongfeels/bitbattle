mod openai;
mod traits;

pub use openai::OpenAiProvider;
pub use traits::{LlmError, LlmProvider, LlmResponse};

use std::sync::Arc;

use crate::config::Config;

/// Create an LLM provider based on configuration
pub fn create_provider(config: &Config) -> Option<Arc<dyn LlmProvider>> {
    if !config.ai_problems_enabled {
        return None;
    }

    match config.ai_provider.as_str() {
        "openai" => {
            if let Some(ref api_key) = config.openai_api_key {
                Some(Arc::new(OpenAiProvider::new(
                    api_key.clone(),
                    config.openai_model.clone(),
                )))
            } else {
                tracing::warn!("AI provider set to openai but OPENAI_API_KEY not configured");
                None
            }
        }
        "anthropic" => {
            tracing::warn!("Anthropic provider not yet implemented, falling back to none");
            None
        }
        _ => {
            tracing::warn!("Unknown AI provider: {}", config.ai_provider);
            None
        }
    }
}
