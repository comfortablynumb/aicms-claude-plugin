//! @ai:module:intent Task execution and API client
//! @ai:module:layer infrastructure
//! @ai:module:public_api ClaudeClient, ClaudeCodeClient, BenchmarkExecutor, RateLimiter, PromptMode

pub mod client;
pub mod claude_code_client;
pub mod executor;
pub mod rate_limiter;

pub use client::{ClaudeClient, ClaudeClientTrait, ClaudeResponse, MockClaudeClient, TaskContext};
pub use claude_code_client::ClaudeCodeClient;
pub use executor::{
    create_executor, BenchmarkExecutor, ExecutionResult, PromptMode, PromptTemplates,
};
pub use rate_limiter::{RateLimiter, RateLimiterTrait};
