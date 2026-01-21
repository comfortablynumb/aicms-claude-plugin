//! @ai:module:intent Markdown report generation
//! @ai:module:layer infrastructure
//! @ai:module:public_api MarkdownReporter
//! @ai:module:stateless true

use crate::metrics::{AggregateStats, BenchmarkResults, DeltaStats};
use anyhow::Result;
use std::fmt::Write as FmtWrite;
use std::path::Path;

/// @ai:intent Trait for Markdown report generation
pub trait MarkdownReporterTrait: Send + Sync {
    /// @ai:intent Generate Markdown report from results
    fn generate(&self, results: &BenchmarkResults, output_path: &Path) -> Result<()>;
}

/// @ai:intent Generates Markdown reports from benchmark results
pub struct MarkdownReporter;

impl MarkdownReporter {
    /// @ai:intent Create a new Markdown reporter
    /// @ai:effects pure
    pub fn new() -> Self {
        Self
    }

    /// @ai:intent Format a delta value with sign
    /// @ai:effects pure
    fn format_delta(value: f64) -> String {
        if value >= 0.0 {
            format!("+{:.1}%", value)
        } else {
            format!("{:.1}%", value)
        }
    }

    /// @ai:intent Generate overall summary section
    /// @ai:effects pure
    fn generate_summary(results: &BenchmarkResults) -> String {
        let mut output = String::new();

        writeln!(output, "# AICMS Benchmark Results").unwrap();
        writeln!(output).unwrap();
        writeln!(output, "**Date:** {}", results.timestamp).unwrap();
        writeln!(output, "**Model:** {}", results.model).unwrap();
        writeln!(output, "**Repetitions:** {}", results.repetitions).unwrap();
        writeln!(output).unwrap();

        output
    }

    /// @ai:intent Generate comparison table
    /// @ai:effects pure
    fn generate_comparison_table(
        baseline: &AggregateStats,
        aicms: &AggregateStats,
        delta: &DeltaStats,
    ) -> String {
        let mut output = String::new();

        writeln!(output, "## Overall Results").unwrap();
        writeln!(output).unwrap();
        writeln!(
            output,
            "| Metric | Baseline | AICMS | Delta |"
        )
        .unwrap();
        writeln!(output, "|--------|----------|-------|-------|").unwrap();

        writeln!(
            output,
            "| Compilation Rate | {:.1}% | {:.1}% | {} |",
            baseline.compilation_rate,
            aicms.compilation_rate,
            Self::format_delta(delta.compilation_rate)
        )
        .unwrap();

        writeln!(
            output,
            "| Test Pass Rate | {:.1}% | {:.1}% | {} |",
            baseline.avg_test_pass_rate,
            aicms.avg_test_pass_rate,
            Self::format_delta(delta.test_pass_rate)
        )
        .unwrap();

        writeln!(
            output,
            "| Lint Compliance | {:.1}% | {:.1}% | {} |",
            baseline.avg_lint_compliance,
            aicms.avg_lint_compliance,
            Self::format_delta(delta.lint_compliance)
        )
        .unwrap();

        writeln!(
            output,
            "| Annotation Quality | {:.1}% | {:.1}% | {} |",
            baseline.avg_annotation_quality,
            aicms.avg_annotation_quality,
            Self::format_delta(delta.annotation_quality)
        )
        .unwrap();

        writeln!(output).unwrap();
        output
    }

    /// @ai:intent Generate category breakdown section
    /// @ai:effects pure
    fn generate_category_section(results: &BenchmarkResults) -> String {
        let mut output = String::new();

        writeln!(output, "## Results by Category").unwrap();
        writeln!(output).unwrap();
        writeln!(
            output,
            "| Category | Baseline Compile | AICMS Compile | Baseline Tests | AICMS Tests |"
        )
        .unwrap();
        writeln!(output, "|----------|-----------------|---------------|----------------|-------------|").unwrap();

        for cat in &results.by_category {
            writeln!(
                output,
                "| {} | {:.1}% | {:.1}% | {:.1}% | {:.1}% |",
                cat.category,
                cat.baseline.compilation_rate,
                cat.aicms.compilation_rate,
                cat.baseline.avg_test_pass_rate,
                cat.aicms.avg_test_pass_rate
            )
            .unwrap();
        }

        writeln!(output).unwrap();
        output
    }

    /// @ai:intent Generate language breakdown section
    /// @ai:effects pure
    fn generate_language_section(results: &BenchmarkResults) -> String {
        let mut output = String::new();

        writeln!(output, "## Results by Language").unwrap();
        writeln!(output).unwrap();
        writeln!(
            output,
            "| Language | Baseline Compile | AICMS Compile | Baseline Tests | AICMS Tests |"
        )
        .unwrap();
        writeln!(output, "|----------|-----------------|---------------|----------------|-------------|").unwrap();

        for lang in &results.by_language {
            writeln!(
                output,
                "| {} | {:.1}% | {:.1}% | {:.1}% | {:.1}% |",
                lang.language,
                lang.baseline.compilation_rate,
                lang.aicms.compilation_rate,
                lang.baseline.avg_test_pass_rate,
                lang.aicms.avg_test_pass_rate
            )
            .unwrap();
        }

        writeln!(output).unwrap();
        output
    }

    /// @ai:intent Generate difficulty breakdown section
    /// @ai:effects pure
    fn generate_difficulty_section(results: &BenchmarkResults) -> String {
        let mut output = String::new();

        writeln!(output, "## Results by Difficulty").unwrap();
        writeln!(output).unwrap();
        writeln!(
            output,
            "| Difficulty | Baseline Compile | AICMS Compile | Baseline Tests | AICMS Tests |"
        )
        .unwrap();
        writeln!(output, "|------------|-----------------|---------------|----------------|-------------|").unwrap();

        for diff in &results.by_difficulty {
            writeln!(
                output,
                "| {} | {:.1}% | {:.1}% | {:.1}% | {:.1}% |",
                diff.difficulty,
                diff.baseline.compilation_rate,
                diff.aicms.compilation_rate,
                diff.baseline.avg_test_pass_rate,
                diff.aicms.avg_test_pass_rate
            )
            .unwrap();
        }

        writeln!(output).unwrap();
        output
    }

    /// @ai:intent Generate token usage section
    /// @ai:effects pure
    fn generate_token_section(results: &BenchmarkResults) -> String {
        let mut output = String::new();

        writeln!(output, "## Token Usage").unwrap();
        writeln!(output).unwrap();
        writeln!(
            output,
            "| Mode | Total Input Tokens | Total Output Tokens | Avg Execution Time |"
        )
        .unwrap();
        writeln!(output, "|------|-------------------|--------------------|--------------------|").unwrap();

        writeln!(
            output,
            "| Baseline | {} | {} | {:.0}ms |",
            results.overall.baseline.total_input_tokens,
            results.overall.baseline.total_output_tokens,
            results.overall.baseline.avg_execution_time_ms
        )
        .unwrap();

        writeln!(
            output,
            "| AICMS | {} | {} | {:.0}ms |",
            results.overall.aicms.total_input_tokens,
            results.overall.aicms.total_output_tokens,
            results.overall.aicms.avg_execution_time_ms
        )
        .unwrap();

        writeln!(output).unwrap();
        output
    }
}

impl Default for MarkdownReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownReporterTrait for MarkdownReporter {
    /// @ai:intent Generate Markdown report to file
    /// @ai:effects fs:write
    fn generate(&self, results: &BenchmarkResults, output_path: &Path) -> Result<()> {
        let mut content = String::new();

        content.push_str(&Self::generate_summary(results));
        content.push_str(&Self::generate_comparison_table(
            &results.overall.baseline,
            &results.overall.aicms,
            &results.overall.delta,
        ));
        content.push_str(&Self::generate_category_section(results));
        content.push_str(&Self::generate_language_section(results));
        content.push_str(&Self::generate_difficulty_section(results));
        content.push_str(&Self::generate_token_section(results));

        std::fs::write(output_path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::ModeComparison;
    use tempfile::TempDir;

    #[test]
    fn test_format_delta_positive() {
        assert_eq!(MarkdownReporter::format_delta(12.5), "+12.5%");
    }

    #[test]
    fn test_format_delta_negative() {
        assert_eq!(MarkdownReporter::format_delta(-5.3), "-5.3%");
    }

    #[test]
    fn test_generate_markdown_report() {
        let reporter = MarkdownReporter::new();
        let temp = TempDir::new().unwrap();
        let output = temp.path().join("results.md");

        let results = BenchmarkResults {
            timestamp: "2026-01-19T00:00:00Z".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            repetitions: 1,
            overall: ModeComparison {
                baseline: AggregateStats {
                    compilation_rate: 80.0,
                    avg_test_pass_rate: 70.0,
                    ..Default::default()
                },
                aicms: AggregateStats {
                    compilation_rate: 92.0,
                    avg_test_pass_rate: 85.0,
                    ..Default::default()
                },
                delta: DeltaStats {
                    compilation_rate: 12.0,
                    test_pass_rate: 15.0,
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
        assert!(content.contains("# AICMS Benchmark Results"));
        assert!(content.contains("+12.0%"));
    }
}
