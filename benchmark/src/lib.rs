//! @ai:module:intent AICMS Benchmark System library
//! @ai:module:layer application
//! @ai:module:public_api config, corpus, runner, evaluator, metrics, report, toolchain

pub mod config;
pub mod corpus;
pub mod evaluator;
pub mod metrics;
pub mod report;
pub mod runner;
pub mod toolchain;

pub use config::BenchmarkConfig;
pub use corpus::{CorpusLoader, Task};
pub use evaluator::Evaluator;
pub use metrics::{BenchmarkResults, MetricsAggregator, TaskMetrics};
pub use report::ReportGenerator;
pub use runner::{BenchmarkExecutor, ClaudeClient, ClaudeClientTrait, ClaudeCodeClient, ExecutionResult};
pub use toolchain::{ToolchainStatus, ToolchainValidator};
