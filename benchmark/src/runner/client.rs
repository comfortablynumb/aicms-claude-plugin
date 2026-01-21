//! @ai:module:intent Claude API client for benchmark execution
//! @ai:module:layer infrastructure
//! @ai:module:public_api ClaudeClient, ClaudeResponse, TaskContext
//! @ai:module:stateless false

use crate::config::ApiConfig;
use crate::runner::rate_limiter::{RateLimiter, RateLimiterTrait};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// @ai:intent Context for task execution
#[derive(Debug, Clone)]
pub struct TaskContext {
    /// Unique task identifier (e.g., "impl-rust-user-crud")
    pub task_id: String,
    /// Execution mode: "baseline" or "aicms"
    pub mode: String,
    /// Whether this is AICMS mode (uses skill file)
    pub use_aicms_skill: bool,
}

/// @ai:intent Trait for Claude API client
#[allow(async_fn_in_trait)]
pub trait ClaudeClientTrait: Send + Sync {
    /// @ai:intent Send a message to Claude and get a response
    async fn send_message(
        &self,
        prompt: &str,
        system: Option<&str>,
        context: &TaskContext,
    ) -> Result<ClaudeResponse>;
}

/// @ai:intent Response from Claude API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeResponse {
    pub content: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub stop_reason: String,
}

/// @ai:intent Claude API request body
#[derive(Debug, Serialize)]
struct ApiRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<&'a str>,
    messages: Vec<Message<'a>>,
}

#[derive(Debug, Serialize)]
struct Message<'a> {
    role: &'static str,
    content: &'a str,
}

/// @ai:intent Claude API response body
#[derive(Debug, Deserialize)]
struct ApiResponse {
    content: Vec<ContentBlock>,
    usage: Usage,
    stop_reason: String,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    text: String,
}

#[derive(Debug, Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

/// @ai:intent Claude API client with rate limiting
pub struct ClaudeClient {
    client: reqwest::Client,
    config: ApiConfig,
    rate_limiter: Arc<RateLimiter>,
    api_key: String,
}

impl ClaudeClient {
    /// @ai:intent Create a new Claude client
    /// @ai:pre ANTHROPIC_API_KEY environment variable is set
    /// @ai:effects env
    pub fn new(config: ApiConfig) -> Result<Self> {
        let api_key =
            std::env::var("ANTHROPIC_API_KEY").context("ANTHROPIC_API_KEY not set in environment")?;

        let rate_limiter = Arc::new(RateLimiter::new(config.requests_per_minute));

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()?;

        Ok(Self {
            client,
            config,
            rate_limiter,
            api_key,
        })
    }

    /// @ai:intent Create a client with a custom rate limiter (for testing)
    /// @ai:effects pure
    pub fn with_rate_limiter(config: ApiConfig, api_key: String, rate_limiter: Arc<RateLimiter>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config,
            rate_limiter,
            api_key,
        }
    }
}

impl ClaudeClientTrait for ClaudeClient {
    /// @ai:intent Send a message to Claude and get a response
    /// @ai:effects network
    async fn send_message(
        &self,
        prompt: &str,
        system: Option<&str>,
        _context: &TaskContext,
    ) -> Result<ClaudeResponse> {
        self.rate_limiter.wait().await;

        let request = ApiRequest {
            model: &self.config.model,
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
            system,
            messages: vec![Message {
                role: "user",
                content: prompt,
            }],
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Claude API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Claude API error ({}): {}", status, error_text);
        }

        let api_response: ApiResponse = response
            .json()
            .await
            .context("Failed to parse Claude API response")?;

        let content = api_response
            .content
            .into_iter()
            .map(|c| c.text)
            .collect::<Vec<_>>()
            .join("\n");

        Ok(ClaudeResponse {
            content,
            input_tokens: api_response.usage.input_tokens,
            output_tokens: api_response.usage.output_tokens,
            stop_reason: api_response.stop_reason,
        })
    }
}

/// @ai:intent Mock client for testing
pub struct MockClaudeClient {
    response: String,
}

impl MockClaudeClient {
    /// @ai:intent Create a mock client that returns a fixed response
    /// @ai:effects pure
    pub fn new(response: String) -> Self {
        Self { response }
    }
}

impl ClaudeClientTrait for MockClaudeClient {
    /// @ai:intent Return mock response
    /// @ai:effects pure
    async fn send_message(
        &self,
        _prompt: &str,
        _system: Option<&str>,
        _context: &TaskContext,
    ) -> Result<ClaudeResponse> {
        Ok(ClaudeResponse {
            content: self.response.clone(),
            input_tokens: 100,
            output_tokens: 200,
            stop_reason: "end_turn".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_client() {
        let client = MockClaudeClient::new("fn factorial(n: u64) -> u64 { 1 }".to_string());
        let context = TaskContext {
            task_id: "test-task".to_string(),
            mode: "baseline".to_string(),
            use_aicms_skill: false,
        };
        let response = client.send_message("test", None, &context).await.unwrap();
        assert!(response.content.contains("factorial"));
    }
}
