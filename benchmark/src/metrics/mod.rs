//! @ai:module:intent Metrics collection and aggregation
//! @ai:module:layer application
//! @ai:module:public_api TaskMetrics, AggregateStats, BenchmarkResults, MetricsAggregator, TaskComparison, ClaudeComparisonStats

pub mod aggregator;
pub mod types;

pub use aggregator::{MetricsAggregator, MetricsAggregatorTrait};
pub use types::{
    AggregateStats, BenchmarkResults, CategoryStats, ClaudeComparisonStats, DeltaStats,
    DifficultyStats, LanguageStats, ModeComparison, TaskComparison, TaskMetrics,
};
