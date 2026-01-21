//! @ai:module:intent JSON report generation
//! @ai:module:layer infrastructure
//! @ai:module:public_api JsonReporter
//! @ai:module:stateless true

use crate::metrics::BenchmarkResults;
use anyhow::Result;
use std::path::Path;

/// @ai:intent Trait for JSON report generation
pub trait JsonReporterTrait: Send + Sync {
    /// @ai:intent Generate JSON report from results
    fn generate(&self, results: &BenchmarkResults, output_path: &Path) -> Result<()>;
}

/// @ai:intent Generates JSON reports from benchmark results
pub struct JsonReporter;

impl JsonReporter {
    /// @ai:intent Create a new JSON reporter
    /// @ai:effects pure
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonReporterTrait for JsonReporter {
    /// @ai:intent Generate JSON report to file
    /// @ai:effects fs:write
    fn generate(&self, results: &BenchmarkResults, output_path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(results)?;
        std::fs::write(output_path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{AggregateStats, DeltaStats, ModeComparison};
    use tempfile::TempDir;

    #[test]
    fn test_generate_json_report() {
        let reporter = JsonReporter::new();
        let temp = TempDir::new().unwrap();
        let output = temp.path().join("results.json");

        let results = BenchmarkResults {
            timestamp: "2026-01-19T00:00:00Z".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            repetitions: 1,
            overall: ModeComparison {
                baseline: AggregateStats::default(),
                aicms: AggregateStats::default(),
                delta: DeltaStats {
                    compilation_rate: 0.0,
                    test_pass_rate: 0.0,
                    lint_compliance: 0.0,
                    annotation_quality: 0.0,
                },
            },
            by_category: vec![],
            by_language: vec![],
            by_difficulty: vec![],
            task_metrics: vec![],
            claude_comparisons: vec![],
            claude_stats: None,
        };

        reporter.generate(&results, &output).unwrap();
        assert!(output.exists());

        let content = std::fs::read_to_string(&output).unwrap();
        assert!(content.contains("claude-sonnet"));
    }
}
