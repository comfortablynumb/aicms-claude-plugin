//! @ai:module:intent Chart generation for benchmark results
//! @ai:module:layer infrastructure
//! @ai:module:public_api ChartGenerator
//! @ai:module:stateless true

use crate::metrics::BenchmarkResults;
use anyhow::Result;
use plotters::prelude::*;
use std::path::Path;

/// @ai:intent Trait for chart generation
pub trait ChartGeneratorTrait: Send + Sync {
    /// @ai:intent Generate all charts from results
    fn generate_all(&self, results: &BenchmarkResults, output_dir: &Path) -> Result<Vec<String>>;
}

/// @ai:intent Generates charts from benchmark results
pub struct ChartGenerator;

impl ChartGenerator {
    /// @ai:intent Create a new chart generator
    /// @ai:effects pure
    pub fn new() -> Self {
        Self
    }

    /// @ai:intent Generate comparison bar chart
    /// @ai:effects fs:write
    fn generate_comparison_chart(
        &self,
        results: &BenchmarkResults,
        output_path: &Path,
    ) -> Result<()> {
        let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
        root.fill(&WHITE)?;

        let metrics = [
            ("Compilation", results.overall.baseline.compilation_rate, results.overall.aicms.compilation_rate),
            ("Test Pass", results.overall.baseline.avg_test_pass_rate, results.overall.aicms.avg_test_pass_rate),
            ("Lint", results.overall.baseline.avg_lint_compliance, results.overall.aicms.avg_lint_compliance),
            ("Annotations", results.overall.baseline.avg_annotation_quality, results.overall.aicms.avg_annotation_quality),
        ];

        let mut chart = ChartBuilder::on(&root)
            .caption("AICMS vs Baseline Comparison", ("sans-serif", 30))
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(0..4i32, 0f64..100f64)?;

        chart
            .configure_mesh()
            .x_labels(4)
            .y_desc("Rate (%)")
            .x_desc("Metric")
            .x_label_formatter(&|x| {
                metrics
                    .get(*x as usize)
                    .map(|(name, _, _)| name.to_string())
                    .unwrap_or_default()
            })
            .draw()?;

        chart.draw_series(
            metrics
                .iter()
                .enumerate()
                .map(|(i, (_, baseline, _))| {
                    Rectangle::new(
                        [(i as i32, 0.0), (i as i32, *baseline)],
                        BLUE.mix(0.7).filled(),
                    )
                }),
        )?
        .label("Baseline")
        .legend(|(x, y)| Rectangle::new([(x, y - 5), (x + 20, y + 5)], BLUE.mix(0.7).filled()));

        chart.draw_series(
            metrics
                .iter()
                .enumerate()
                .map(|(i, (_, _, aicms))| {
                    Rectangle::new(
                        [(i as i32, 0.0), (i as i32, *aicms)],
                        GREEN.mix(0.7).filled(),
                    )
                }),
        )?
        .label("AICMS")
        .legend(|(x, y)| Rectangle::new([(x, y - 5), (x + 20, y + 5)], GREEN.mix(0.7).filled()));

        chart
            .configure_series_labels()
            .position(SeriesLabelPosition::UpperRight)
            .border_style(BLACK)
            .draw()?;

        root.present()?;
        Ok(())
    }

    /// @ai:intent Generate language breakdown chart
    /// @ai:effects fs:write
    fn generate_language_chart(
        &self,
        results: &BenchmarkResults,
        output_path: &Path,
    ) -> Result<()> {
        let root = BitMapBackend::new(output_path, (800, 500)).into_drawing_area();
        root.fill(&WHITE)?;

        let data: Vec<_> = results
            .by_language
            .iter()
            .map(|l| {
                (
                    l.language.as_str(),
                    l.baseline.compilation_rate,
                    l.aicms.compilation_rate,
                )
            })
            .collect();

        let mut chart = ChartBuilder::on(&root)
            .caption("Compilation Rate by Language", ("sans-serif", 25))
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(0..data.len() as i32, 0f64..100f64)?;

        chart
            .configure_mesh()
            .y_desc("Compilation Rate (%)")
            .x_label_formatter(&|x| {
                data.get(*x as usize)
                    .map(|(name, _, _)| name.to_string())
                    .unwrap_or_default()
            })
            .draw()?;

        chart.draw_series(data.iter().enumerate().map(|(i, (_, baseline, _))| {
            Rectangle::new(
                [(i as i32, 0.0), (i as i32, *baseline)],
                BLUE.mix(0.7).filled(),
            )
        }))?;

        chart.draw_series(data.iter().enumerate().map(|(i, (_, _, aicms))| {
            Rectangle::new(
                [(i as i32, 0.0), (i as i32, *aicms)],
                GREEN.mix(0.7).filled(),
            )
        }))?;

        root.present()?;
        Ok(())
    }

    /// @ai:intent Generate difficulty breakdown chart
    /// @ai:effects fs:write
    fn generate_difficulty_chart(
        &self,
        results: &BenchmarkResults,
        output_path: &Path,
    ) -> Result<()> {
        let root = BitMapBackend::new(output_path, (800, 500)).into_drawing_area();
        root.fill(&WHITE)?;

        let data: Vec<_> = results
            .by_difficulty
            .iter()
            .map(|d| {
                (
                    d.difficulty.as_str(),
                    d.baseline.avg_test_pass_rate,
                    d.aicms.avg_test_pass_rate,
                )
            })
            .collect();

        let mut chart = ChartBuilder::on(&root)
            .caption("Test Pass Rate by Difficulty", ("sans-serif", 25))
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(0..data.len() as i32, 0f64..100f64)?;

        chart
            .configure_mesh()
            .y_desc("Test Pass Rate (%)")
            .x_label_formatter(&|x| {
                data.get(*x as usize)
                    .map(|(name, _, _)| name.to_string())
                    .unwrap_or_default()
            })
            .draw()?;

        chart.draw_series(data.iter().enumerate().map(|(i, (_, baseline, _))| {
            Rectangle::new(
                [(i as i32, 0.0), (i as i32, *baseline)],
                BLUE.mix(0.7).filled(),
            )
        }))?;

        chart.draw_series(data.iter().enumerate().map(|(i, (_, _, aicms))| {
            Rectangle::new(
                [(i as i32, 0.0), (i as i32, *aicms)],
                GREEN.mix(0.7).filled(),
            )
        }))?;

        root.present()?;
        Ok(())
    }
}

impl Default for ChartGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ChartGeneratorTrait for ChartGenerator {
    /// @ai:intent Generate all charts
    /// @ai:effects fs:write
    fn generate_all(&self, results: &BenchmarkResults, output_dir: &Path) -> Result<Vec<String>> {
        std::fs::create_dir_all(output_dir)?;

        let mut generated = Vec::new();

        let comparison_path = output_dir.join("comparison.png");
        self.generate_comparison_chart(results, &comparison_path)?;
        generated.push("comparison.png".to_string());

        let language_path = output_dir.join("by_language.png");
        self.generate_language_chart(results, &language_path)?;
        generated.push("by_language.png".to_string());

        let difficulty_path = output_dir.join("by_difficulty.png");
        self.generate_difficulty_chart(results, &difficulty_path)?;
        generated.push("by_difficulty.png".to_string());

        Ok(generated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{AggregateStats, DeltaStats, LanguageStats, DifficultyStats, ModeComparison};
    use tempfile::TempDir;

    fn create_test_results() -> BenchmarkResults {
        BenchmarkResults {
            timestamp: "2026-01-19T00:00:00Z".to_string(),
            model: "test-model".to_string(),
            repetitions: 1,
            overall: ModeComparison {
                baseline: AggregateStats {
                    compilation_rate: 80.0,
                    avg_test_pass_rate: 70.0,
                    avg_lint_compliance: 60.0,
                    ..Default::default()
                },
                aicms: AggregateStats {
                    compilation_rate: 92.0,
                    avg_test_pass_rate: 85.0,
                    avg_lint_compliance: 88.0,
                    ..Default::default()
                },
                delta: DeltaStats {
                    compilation_rate: 12.0,
                    test_pass_rate: 15.0,
                    lint_compliance: 28.0,
                    annotation_quality: 0.0,
                },
            },
            by_category: vec![],
            by_language: vec![
                LanguageStats {
                    language: "rust".to_string(),
                    baseline: AggregateStats { compilation_rate: 85.0, ..Default::default() },
                    aicms: AggregateStats { compilation_rate: 95.0, ..Default::default() },
                },
                LanguageStats {
                    language: "python".to_string(),
                    baseline: AggregateStats { compilation_rate: 90.0, ..Default::default() },
                    aicms: AggregateStats { compilation_rate: 95.0, ..Default::default() },
                },
            ],
            by_difficulty: vec![
                DifficultyStats {
                    difficulty: "easy".to_string(),
                    baseline: AggregateStats { avg_test_pass_rate: 80.0, ..Default::default() },
                    aicms: AggregateStats { avg_test_pass_rate: 95.0, ..Default::default() },
                },
                DifficultyStats {
                    difficulty: "hard".to_string(),
                    baseline: AggregateStats { avg_test_pass_rate: 50.0, ..Default::default() },
                    aicms: AggregateStats { avg_test_pass_rate: 70.0, ..Default::default() },
                },
            ],
            task_metrics: vec![],
            claude_comparisons: vec![],
            claude_stats: None,
        }
    }

    #[test]
    fn test_generate_all_charts() {
        let generator = ChartGenerator::new();
        let temp = TempDir::new().unwrap();
        let results = create_test_results();

        let files = generator.generate_all(&results, temp.path()).unwrap();

        assert_eq!(files.len(), 3);
        assert!(temp.path().join("comparison.png").exists());
        assert!(temp.path().join("by_language.png").exists());
        assert!(temp.path().join("by_difficulty.png").exists());
    }
}
