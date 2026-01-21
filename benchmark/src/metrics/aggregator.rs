//! @ai:module:intent Statistical aggregation for benchmark metrics
//! @ai:module:layer application
//! @ai:module:public_api MetricsAggregator
//! @ai:module:stateless true

use crate::corpus::Task;
use crate::metrics::types::{
    AggregateStats, BenchmarkResults, CategoryStats, ClaudeComparisonStats, DeltaStats,
    DifficultyStats, LanguageStats, ModeComparison, TaskComparison, TaskMetrics,
};
use std::collections::HashMap;

/// @ai:intent Trait for metrics aggregation
pub trait MetricsAggregatorTrait: Send + Sync {
    /// @ai:intent Aggregate task metrics into benchmark results
    fn aggregate(
        &self,
        metrics: &[TaskMetrics],
        tasks: &[Task],
        model: &str,
        repetitions: u32,
    ) -> BenchmarkResults;
}

/// @ai:intent Aggregates task metrics into statistical summaries
pub struct MetricsAggregator;

impl MetricsAggregator {
    /// @ai:intent Create a new metrics aggregator
    /// @ai:effects pure
    pub fn new() -> Self {
        Self
    }

    /// @ai:intent Calculate aggregate stats for a set of metrics
    /// @ai:effects pure
    fn calculate_aggregate(metrics: &[&TaskMetrics]) -> AggregateStats {
        if metrics.is_empty() {
            return AggregateStats::default();
        }

        let task_count = metrics.len() as u32;
        let compiled_count = metrics.iter().filter(|m| m.compiled).count();

        let compilation_rate = (compiled_count as f64 / task_count as f64) * 100.0;
        let avg_test_pass_rate = average(metrics.iter().map(|m| m.test_pass_rate));
        let avg_lint_compliance = average(metrics.iter().map(|m| m.lint_compliance));
        let avg_annotation_quality = average(metrics.iter().map(|m| m.annotation_quality));

        let total_input_tokens: u64 = metrics.iter().map(|m| m.input_tokens as u64).sum();
        let total_output_tokens: u64 = metrics.iter().map(|m| m.output_tokens as u64).sum();
        let avg_execution_time_ms = average(metrics.iter().map(|m| m.execution_time_ms as f64));

        AggregateStats {
            task_count,
            compilation_rate,
            avg_test_pass_rate,
            avg_lint_compliance,
            avg_annotation_quality,
            total_input_tokens,
            total_output_tokens,
            avg_execution_time_ms,
        }
    }

    /// @ai:intent Split metrics by mode
    /// @ai:effects pure
    fn split_by_mode(metrics: &[TaskMetrics]) -> (Vec<&TaskMetrics>, Vec<&TaskMetrics>) {
        let baseline: Vec<_> = metrics.iter().filter(|m| m.mode == "baseline").collect();
        let aicms: Vec<_> = metrics.iter().filter(|m| m.mode == "aicms").collect();
        (baseline, aicms)
    }
}

impl Default for MetricsAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// @ai:intent Calculate average of an iterator of f64
/// @ai:effects pure
fn average<I: Iterator<Item = f64>>(iter: I) -> f64 {
    let (sum, count) = iter.fold((0.0, 0u32), |(s, c), v| (s + v, c + 1));

    if count == 0 {
        0.0
    } else {
        sum / count as f64
    }
}

impl MetricsAggregatorTrait for MetricsAggregator {
    /// @ai:intent Aggregate metrics into benchmark results
    /// @ai:effects pure
    fn aggregate(
        &self,
        metrics: &[TaskMetrics],
        tasks: &[Task],
        model: &str,
        repetitions: u32,
    ) -> BenchmarkResults {
        let (baseline, aicms) = Self::split_by_mode(metrics);

        let baseline_stats = Self::calculate_aggregate(&baseline);
        let aicms_stats = Self::calculate_aggregate(&aicms);
        let delta = DeltaStats::calculate(&baseline_stats, &aicms_stats);

        let task_map: HashMap<_, _> = tasks.iter().map(|t| (t.id.as_str(), t)).collect();

        let by_category = aggregate_by_category(metrics, &task_map);
        let by_language = aggregate_by_language(metrics, &task_map);
        let by_difficulty = aggregate_by_difficulty(metrics, &task_map);

        BenchmarkResults {
            timestamp: chrono::Utc::now().to_rfc3339(),
            model: model.to_string(),
            repetitions,
            overall: ModeComparison {
                baseline: baseline_stats,
                aicms: aicms_stats,
                delta,
            },
            by_category,
            by_language,
            by_difficulty,
            task_metrics: metrics.to_vec(),
            claude_comparisons: vec![],
            claude_stats: None,
        }
    }

}

impl MetricsAggregator {
    /// @ai:intent Add Claude comparisons to results and calculate stats
    /// @ai:effects pure
    pub fn add_claude_comparisons(
        &self,
        results: &mut BenchmarkResults,
        comparisons: Vec<TaskComparison>,
    ) {
        if comparisons.is_empty() {
            return;
        }

        let mut baseline_scores = Vec::new();
        let mut aicms_scores = Vec::new();
        let mut aicms_wins = 0u32;
        let mut baseline_wins = 0u32;
        let mut ties = 0u32;

        for comp in &comparisons {
            baseline_scores.push(comp.comparison.baseline.overall as f64);
            aicms_scores.push(comp.comparison.aicms.overall as f64);

            match comp.comparison.winner.as_str() {
                "aicms" => aicms_wins += 1,
                "baseline" => baseline_wins += 1,
                _ => ties += 1,
            }
        }

        let avg_baseline = average(baseline_scores.into_iter());
        let avg_aicms = average(aicms_scores.into_iter());

        results.claude_comparisons = comparisons;
        results.claude_stats = Some(ClaudeComparisonStats {
            avg_baseline_score: avg_baseline,
            avg_aicms_score: avg_aicms,
            aicms_wins,
            baseline_wins,
            ties,
        });
    }
}

/// @ai:intent Aggregate metrics by task category
/// @ai:effects pure
fn aggregate_by_category(
    metrics: &[TaskMetrics],
    task_map: &HashMap<&str, &Task>,
) -> Vec<CategoryStats> {
    let categories = ["implement", "bugfix", "refactor", "inference"];

    categories
        .iter()
        .map(|cat| {
            let cat_metrics: Vec<_> = metrics
                .iter()
                .filter(|m| {
                    task_map
                        .get(m.task_id.as_str())
                        .map(|t| t.category.as_str() == *cat)
                        .unwrap_or(false)
                })
                .collect();

            let (baseline, aicms): (Vec<_>, Vec<_>) =
                cat_metrics.iter().partition(|m| m.mode == "baseline");

            let baseline_refs: Vec<_> = baseline.iter().copied().collect();
            let aicms_refs: Vec<_> = aicms.iter().copied().collect();

            CategoryStats {
                category: cat.to_string(),
                baseline: MetricsAggregator::calculate_aggregate(&baseline_refs),
                aicms: MetricsAggregator::calculate_aggregate(&aicms_refs),
            }
        })
        .collect()
}

/// @ai:intent Aggregate metrics by programming language
/// @ai:effects pure
fn aggregate_by_language(
    metrics: &[TaskMetrics],
    task_map: &HashMap<&str, &Task>,
) -> Vec<LanguageStats> {
    let languages = ["rust", "python", "typescript"];

    languages
        .iter()
        .map(|lang| {
            let lang_metrics: Vec<_> = metrics
                .iter()
                .filter(|m| {
                    task_map
                        .get(m.task_id.as_str())
                        .map(|t| t.language.as_str() == *lang)
                        .unwrap_or(false)
                })
                .collect();

            let (baseline, aicms): (Vec<_>, Vec<_>) =
                lang_metrics.iter().partition(|m| m.mode == "baseline");

            let baseline_refs: Vec<_> = baseline.iter().copied().collect();
            let aicms_refs: Vec<_> = aicms.iter().copied().collect();

            LanguageStats {
                language: lang.to_string(),
                baseline: MetricsAggregator::calculate_aggregate(&baseline_refs),
                aicms: MetricsAggregator::calculate_aggregate(&aicms_refs),
            }
        })
        .collect()
}

/// @ai:intent Aggregate metrics by difficulty level
/// @ai:effects pure
fn aggregate_by_difficulty(
    metrics: &[TaskMetrics],
    task_map: &HashMap<&str, &Task>,
) -> Vec<DifficultyStats> {
    let difficulties = ["easy", "medium", "hard"];

    difficulties
        .iter()
        .map(|diff| {
            let diff_metrics: Vec<_> = metrics
                .iter()
                .filter(|m| {
                    task_map
                        .get(m.task_id.as_str())
                        .map(|t| t.difficulty.as_str() == *diff)
                        .unwrap_or(false)
                })
                .collect();

            let (baseline, aicms): (Vec<_>, Vec<_>) =
                diff_metrics.iter().partition(|m| m.mode == "baseline");

            let baseline_refs: Vec<_> = baseline.iter().copied().collect();
            let aicms_refs: Vec<_> = aicms.iter().copied().collect();

            DifficultyStats {
                difficulty: diff.to_string(),
                baseline: MetricsAggregator::calculate_aggregate(&baseline_refs),
                aicms: MetricsAggregator::calculate_aggregate(&aicms_refs),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_average() {
        let values = vec![10.0, 20.0, 30.0];
        assert!((average(values.into_iter()) - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_average_empty() {
        let values: Vec<f64> = vec![];
        assert!((average(values.into_iter()) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_aggregate() {
        let m1 = TaskMetrics {
            task_id: "t1".to_string(),
            mode: "baseline".to_string(),
            repetition: 0,
            code_extracted: true,
            compiled: true,
            test_pass_rate: 80.0,
            lint_compliance: 100.0,
            lint_issues: vec![],
            annotation_quality: 70.0,
            input_tokens: 100,
            output_tokens: 200,
            execution_time_ms: 1000,
        };

        let m2 = TaskMetrics {
            task_id: "t2".to_string(),
            mode: "baseline".to_string(),
            repetition: 0,
            code_extracted: true,
            compiled: false,
            test_pass_rate: 60.0,
            lint_compliance: 80.0,
            lint_issues: vec![],
            annotation_quality: 50.0,
            input_tokens: 150,
            output_tokens: 250,
            execution_time_ms: 1500,
        };

        let metrics: Vec<&TaskMetrics> = vec![&m1, &m2];
        let stats = MetricsAggregator::calculate_aggregate(&metrics);

        assert_eq!(stats.task_count, 2);
        assert!((stats.compilation_rate - 50.0).abs() < 0.01);
        assert!((stats.avg_test_pass_rate - 70.0).abs() < 0.01);
    }
}
