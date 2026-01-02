use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::traits::{LlmError, LlmProvider, LlmResponse, TokenUsage};

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

/// OpenAI chat message
#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

/// OpenAI chat completion request
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

/// OpenAI API response structures
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
    model: String,
    usage: Option<ApiUsage>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    error: ApiErrorDetails,
}

#[derive(Debug, Deserialize)]
struct ApiErrorDetails {
    message: String,
    #[serde(rename = "type")]
    error_type: Option<String>,
}

/// OpenAI LLM provider implementation
pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &'static str {
        "openai"
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn complete(&self, system_prompt: &str, user_prompt: &str) -> Result<LlmResponse, LlmError> {
        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                },
            ],
            temperature: 0.7,
            max_tokens: 4000,
        };

        let response = self
            .client
            .post(OPENAI_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;

        let status = response.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            // Try to parse retry-after header
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(60);
            return Err(LlmError::RateLimited(retry_after));
        }

        let body = response
            .text()
            .await
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;

        if !status.is_success() {
            // Try to parse error response
            if let Ok(error) = serde_json::from_str::<ApiError>(&body) {
                return Err(LlmError::RequestFailed(error.error.message));
            }
            return Err(LlmError::RequestFailed(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        let completion: ChatCompletionResponse = serde_json::from_str(&body)
            .map_err(|e| LlmError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        let choice = completion
            .choices
            .first()
            .ok_or_else(|| LlmError::InvalidResponse("No choices in response".to_string()))?;

        // Check for content filtering
        if choice.finish_reason.as_deref() == Some("content_filter") {
            return Err(LlmError::ContentFiltered);
        }

        let content = choice
            .message
            .content
            .clone()
            .ok_or_else(|| LlmError::InvalidResponse("No content in message".to_string()))?;

        let usage = completion.usage.map(|u| TokenUsage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        Ok(LlmResponse {
            content,
            model: completion.model,
            usage,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_name() {
        let provider = OpenAiProvider::new("test-key".to_string(), "gpt-4o-mini".to_string());
        assert_eq!(provider.name(), "openai");
        assert_eq!(provider.model(), "gpt-4o-mini");
    }
}
