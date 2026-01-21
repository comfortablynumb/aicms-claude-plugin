//! @ai:module:intent Report generation for benchmark results
//! @ai:module:layer infrastructure
//! @ai:module:public_api ReportGenerator, JsonReporter, MarkdownReporter, ChartGenerator

pub mod charts;
pub mod json_report;
pub mod markdown_report;

pub use charts::{ChartGenerator, ChartGeneratorTrait};
pub use json_report::{JsonReporter, JsonReporterTrait};
pub use markdown_report::{MarkdownReporter, MarkdownReporterTrait};

use crate::metrics::BenchmarkResults;
use anyhow::Result;
use std::path::Path;

/// @ai:intent Combined report generator
pub struct ReportGenerator {
    json: JsonReporter,
    markdown: MarkdownReporter,
    charts: ChartGenerator,
}

impl ReportGenerator {
    /// @ai:intent Create a new report generator
    /// @ai:effects pure
    pub fn new() -> Self {
        Self {
            json: JsonReporter::new(),
            markdown: MarkdownReporter::new(),
            charts: ChartGenerator::new(),
        }
    }

    /// @ai:intent Generate all reports
    /// @ai:effects fs:write
    pub fn generate_all(&self, results: &BenchmarkResults, output_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(output_dir)?;

        self.json.generate(results, &output_dir.join("results.json"))?;
        self.markdown
            .generate(results, &output_dir.join("results.md"))?;
        self.charts.generate_all(results, output_dir)?;

        tracing::info!("Reports generated in {}", output_dir.display());
        Ok(())
    }

    /// @ai:intent Save the comparison prompt used for evaluation
    /// @ai:effects fs:write
    pub fn save_comparison_prompt(
        &self,
        prompt: &str,
        output_dir: &Path,
    ) -> Result<()> {
        let prompt_path = output_dir.join("comparison_prompt.md");
        std::fs::write(&prompt_path, prompt)?;

        tracing::info!("Comparison prompt saved to {}", prompt_path.display());
        Ok(())
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}
