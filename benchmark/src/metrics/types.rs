//! @ai:module:intent Metric types for benchmark results
//! @ai:module:layer domain
//! @ai:module:public_api TaskMetrics, AggregateStats, ModeComparison, TaskComparison
//! @ai:module:stateless true

use crate::evaluator::{ComparisonScore, EvaluationResult};
use serde::{Deserialize, Serialize};

/// @ai:intent Metrics for a single task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetrics {
    pub task_id: String,
    pub mode: String,
    pub repetition: u32,
    pub code_extracted: bool,
    pub compiled: bool,
    pub test_pass_rate: f64,
    pub lint_compliance: f64,
    pub lint_issues: Vec<String>,
    pub annotation_quality: f64,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub execution_time_ms: u64,
}

impl TaskMetrics {
    /// @ai:intent Create metrics from evaluation result
    /// @ai:effects pure
    pub fn from_evaluation(
        eval: &EvaluationResult,
        input_tokens: u32,
        output_tokens: u32,
        execution_time_ms: u64,
    ) -> Self {
        let code_extracted = eval.extracted_code.is_some();

        let compiled = eval
            .compilation
            .as_ref()
            .map(|c| c.success)
            .unwrap_or(false);

        let test_pass_rate = eval.tests.as_ref().map(|t| t.pass_rate()).unwrap_or(0.0);

        let lint_compliance = eval
            .lint
            .as_ref()
            .map(|l| l.compliance_rate())
            .unwrap_or(0.0);

        let lint_issues = eval
            .lint
            .as_ref()
            .map(|l| l.issues.iter().map(|i| i.message.clone()).collect())
            .unwrap_or_default();

        let annotation_quality = eval
            .annotation_score
            .as_ref()
            .map(|a| a.overall * 100.0)
            .unwrap_or(0.0);

        Self {
            task_id: eval.task_id.clone(),
            mode: eval.mode.clone(),
            repetition: eval.repetition,
            code_extracted,
            compiled,
            test_pass_rate,
            lint_compliance,
            lint_issues,
            annotation_quality,
            input_tokens,
            output_tokens,
            execution_time_ms,
        }
    }
}

/// @ai:intent Aggregated statistics across multiple runs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AggregateStats {
    pub task_count: u32,
    pub compilation_rate: f64,
    pub avg_test_pass_rate: f64,
    pub avg_lint_compliance: f64,
    pub avg_annotation_quality: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub avg_execution_time_ms: f64,
}

/// @ai:intent Comparison between baseline and AICMS modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeComparison {
    pub baseline: AggregateStats,
    pub aicms: AggregateStats,
    pub delta: DeltaStats,
}

/// @ai:intent Delta between two aggregate stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaStats {
    pub compilation_rate: f64,
    pub test_pass_rate: f64,
    pub lint_compliance: f64,
    pub annotation_quality: f64,
}

impl DeltaStats {
    /// @ai:intent Calculate delta between AICMS and baseline
    /// @ai:effects pure
    pub fn calculate(baseline: &AggregateStats, aicms: &AggregateStats) -> Self {
        Self {
            compilation_rate: aicms.compilation_rate - baseline.compilation_rate,
            test_pass_rate: aicms.avg_test_pass_rate - baseline.avg_test_pass_rate,
            lint_compliance: aicms.avg_lint_compliance - baseline.avg_lint_compliance,
            annotation_quality: aicms.avg_annotation_quality - baseline.avg_annotation_quality,
        }
    }
}

/// @ai:intent Statistics by category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    pub category: String,
    pub baseline: AggregateStats,
    pub aicms: AggregateStats,
}

/// @ai:intent Statistics by language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStats {
    pub language: String,
    pub baseline: AggregateStats,
    pub aicms: AggregateStats,
}

/// @ai:intent Statistics by difficulty
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyStats {
    pub difficulty: String,
    pub baseline: AggregateStats,
    pub aicms: AggregateStats,
}

/// @ai:intent Claude-based comparison for a single task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskComparison {
    pub task_id: String,
    pub comparison: ComparisonScore,
}

/// @ai:intent Aggregate stats from Claude comparisons
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClaudeComparisonStats {
    pub avg_baseline_score: f64,
    pub avg_aicms_score: f64,
    pub aicms_wins: u32,
    pub baseline_wins: u32,
    pub ties: u32,
}

/// @ai:intent Complete benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResults {
    pub timestamp: String,
    pub model: String,
    pub repetitions: u32,
    pub overall: ModeComparison,
    pub by_category: Vec<CategoryStats>,
    pub by_language: Vec<LanguageStats>,
    pub by_difficulty: Vec<DifficultyStats>,
    pub task_metrics: Vec<TaskMetrics>,
    /// Claude-based comparisons for each task (optional)
    #[serde(default)]
    pub claude_comparisons: Vec<TaskComparison>,
    /// Aggregate stats from Claude comparisons
    #[serde(default)]
    pub claude_stats: Option<ClaudeComparisonStats>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_calculation() {
        let baseline = AggregateStats {
            compilation_rate: 80.0,
            avg_test_pass_rate: 60.0,
            ..Default::default()
        };

        let aicms = AggregateStats {
            compilation_rate: 92.0,
            avg_test_pass_rate: 85.0,
            ..Default::default()
        };

        let delta = DeltaStats::calculate(&baseline, &aicms);
        assert!((delta.compilation_rate - 12.0).abs() < 0.01);
        assert!((delta.test_pass_rate - 25.0).abs() < 0.01);
    }
}
