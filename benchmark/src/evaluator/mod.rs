//! @ai:module:intent Evaluation components for benchmark results
//! @ai:module:layer application
//! @ai:module:public_api Evaluator, EvaluationResult, ClaudeScorer, ComparisonScore

pub mod annotation_scorer;
pub mod claude_scorer;
pub mod code_extractor;
pub mod compiler;
pub mod linter_adapter;
pub mod test_runner;

pub use annotation_scorer::{AnnotationScore, AnnotationScorer, AnnotationScorerTrait};
pub use claude_scorer::{
    default_comparison_prompt, ClaudeScorer, ClaudeScorerTrait, ComparisonScore,
    ImplementationScore, MockClaudeScorer,
};
pub use code_extractor::{CodeExtractor, CodeExtractorTrait, ExtractedCode, ExtractedFile};
pub use compiler::{CompilationChecker, CompilationCheckerTrait, CompilationResult};
pub use linter_adapter::{LinterAdapter, LinterAdapterTrait, LintIssue, LintResult, Severity};
pub use test_runner::{TestResult, TestRunner, TestRunnerTrait};

/// @ai:intent A source file with path and content (used for evaluation)
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: String,
    pub content: String,
}

use crate::corpus::Task;
use crate::runner::ExecutionResult;
use anyhow::Result;

/// @ai:intent Combined evaluation result for a task execution
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    pub task_id: String,
    pub mode: String,
    pub repetition: u32,
    pub compilation: Option<CompilationResult>,
    pub tests: Option<TestResult>,
    pub lint: Option<LintResult>,
    pub annotation_score: Option<AnnotationScore>,
    pub extracted_code: Option<String>,
    pub extracted_files: Option<Vec<ExtractedFile>>,
}

/// @ai:intent Main evaluator that combines all evaluation components
pub struct Evaluator {
    code_extractor: CodeExtractor,
    compiler: CompilationChecker,
    test_runner: TestRunner,
    linter: LinterAdapter,
    annotation_scorer: AnnotationScorer,
}

impl Evaluator {
    /// @ai:intent Create a new evaluator with all components
    /// @ai:effects pure
    pub fn new() -> Self {
        Self {
            code_extractor: CodeExtractor::new(),
            compiler: CompilationChecker::new(),
            test_runner: TestRunner::new(),
            linter: LinterAdapter::new(),
            annotation_scorer: AnnotationScorer::new(),
        }
    }

    /// @ai:intent Evaluate a single execution result
    ///            Extracts code from response and runs Claude's own tests
    /// @ai:effects fs:write, io
    pub fn evaluate(&self, task: &Task, execution: &ExecutionResult) -> Result<EvaluationResult> {
        let extracted_files = self
            .code_extractor
            .extract_files(&execution.response, task.language);

        if extracted_files.is_empty() {
            let response_preview = truncate_for_log(&execution.response, 200);
            tracing::warn!(
                "No code blocks extracted for task {} (mode={}). Response preview: {}",
                task.id,
                execution.mode.as_str(),
                response_preview
            );

            return Ok(EvaluationResult {
                task_id: task.id.clone(),
                mode: execution.mode.as_str().to_string(),
                repetition: execution.repetition,
                compilation: None,
                tests: None,
                lint: None,
                annotation_score: None,
                extracted_code: None,
                extracted_files: None,
            });
        }

        tracing::info!(
            "Extracted {} files for task {} (mode={}): {:?}",
            extracted_files.len(),
            task.id,
            execution.mode.as_str(),
            extracted_files.iter().map(|f| &f.path).collect::<Vec<_>>()
        );

        let source_files = self.code_extractor.to_source_files(&extracted_files);

        // Compile the project
        tracing::info!("Compiling {} files...", source_files.len());
        let compilation = match self.compiler.check_files(&source_files, task.language) {
            Ok(result) => {
                tracing::info!(
                    "Compilation {}: {} errors, {} warnings",
                    if result.success { "succeeded" } else { "failed" },
                    result.errors.len(),
                    result.warnings.len()
                );

                if !result.errors.is_empty() {
                    for err in &result.errors {
                        tracing::warn!("Compilation error: {}", err);
                    }
                }
                Some(result)
            }
            Err(e) => {
                tracing::error!("Compilation check failed: {}", e);
                None
            }
        };

        // Run Claude's own tests (included in the generated code)
        tracing::info!("Running tests...");
        let tests = match self.test_runner.run_own_tests(&source_files, task.language) {
            Ok(result) => {
                tracing::info!(
                    "Tests: {} passed, {} failed, {} total",
                    result.passed,
                    result.failed,
                    result.total
                );
                Some(result)
            }
            Err(e) => {
                tracing::error!("Test run failed: {}", e);
                None
            }
        };

        // Combine all code for linting and annotation scoring
        let combined_code: String = extracted_files
            .iter()
            .map(|f| format!("// file: {}\n{}", f.path, f.code))
            .collect::<Vec<_>>()
            .join("\n\n");

        let lint = Some(self.linter.lint(&combined_code));

        // Score annotations (no expected list, just count what's present)
        let annotation_score = Some(self.annotation_scorer.score(&combined_code, &[]));

        Ok(EvaluationResult {
            task_id: task.id.clone(),
            mode: execution.mode.as_str().to_string(),
            repetition: execution.repetition,
            compilation,
            tests,
            lint,
            annotation_score,
            extracted_code: Some(combined_code),
            extracted_files: Some(extracted_files),
        })
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// @ai:intent Truncate string for logging
/// @ai:effects pure
fn truncate_for_log(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.replace('\n', "\\n")
    } else {
        format!("{}...", s[..max_len].replace('\n', "\\n"))
    }
}

/// @ai:intent Extract function name from code
/// @ai:effects pure
#[cfg(test)]
fn extract_function_name(code: &str, language: crate::corpus::Language) -> String {
    use crate::corpus::Language;

    let pattern = match language {
        Language::Rust => r"fn\s+(\w+)",
        Language::Python => r"def\s+(\w+)",
        Language::TypeScript => r"function\s+(\w+)",
    };

    regex::Regex::new(pattern)
        .ok()
        .and_then(|re| re.captures(code))
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_function_name_rust() {
        let code = "fn factorial(n: u64) -> u64 { 1 }";
        assert_eq!(
            extract_function_name(code, crate::corpus::Language::Rust),
            "factorial"
        );
    }

    #[test]
    fn test_extract_function_name_python() {
        let code = "def calculate_sum(a, b):\n    return a + b";
        assert_eq!(
            extract_function_name(code, crate::corpus::Language::Python),
            "calculate_sum"
        );
    }
}
