//! @ai:module:intent Task execution logic for benchmarks
//! @ai:module:layer application
//! @ai:module:public_api BenchmarkExecutor, ExecutionResult, PromptMode
//! @ai:module:stateless false

use crate::config::{BenchmarkConfig, RunConfig};
use crate::corpus::Task;
use crate::runner::client::{ClaudeClientTrait, TaskContext};
use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;

/// @ai:intent Strip AICMS annotations from code for baseline mode
/// @ai:effects pure
#[cfg(test)]
fn strip_aicms_annotations(code: &str) -> String {
    use regex::Regex;

    // Match lines that are ONLY comments containing @ai: annotations
    // This handles: /// @ai:..., // @ai:..., # @ai:..., * @ai:...
    let annotation_line = Regex::new(r"(?m)^\s*(///|//|#|\*)\s*@ai:[^\n]*\n?").unwrap();

    // Also remove @ai:module:* from //! doc comments
    let module_annotation = Regex::new(r"(?m)^\s*//!\s*@ai:[^\n]*\n?").unwrap();

    let result = annotation_line.replace_all(code, "");
    let result = module_annotation.replace_all(&result, "");

    // Clean up multiple consecutive blank lines that may result from stripping
    let multiple_blanks = Regex::new(r"\n{3,}").unwrap();
    multiple_blanks.replace_all(&result, "\n\n").to_string()
}

/// @ai:intent Mode for benchmark prompt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptMode {
    Baseline,
    Aicms,
}

impl PromptMode {
    /// @ai:intent Get string representation
    /// @ai:effects pure
    pub fn as_str(&self) -> &'static str {
        match self {
            PromptMode::Baseline => "baseline",
            PromptMode::Aicms => "aicms",
        }
    }
}

/// @ai:intent Result of executing a single task
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub task_id: String,
    pub mode: PromptMode,
    pub repetition: u32,
    pub response: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub execution_time_ms: u64,
}

/// @ai:intent Prompt templates loaded from files
pub struct PromptTemplates {
    pub baseline: String,
    pub aicms_skill: String,
}

impl PromptTemplates {
    /// @ai:intent Load prompt templates from directory
    /// @ai:effects fs:read
    pub fn load(prompts_dir: &Path, skill_file: &Path) -> Result<Self> {
        let baseline = std::fs::read_to_string(prompts_dir.join("baseline.md"))
            .context("Failed to read baseline.md")?;

        let aicms_skill =
            std::fs::read_to_string(skill_file).context("Failed to read SKILL.md")?;

        Ok(Self {
            baseline,
            aicms_skill,
        })
    }
}

/// @ai:intent Executes benchmark tasks against Claude
pub struct BenchmarkExecutor<C: ClaudeClientTrait> {
    client: Arc<C>,
    templates: PromptTemplates,
    run_config: RunConfig,
}

impl<C: ClaudeClientTrait> BenchmarkExecutor<C> {
    /// @ai:intent Create a new benchmark executor
    /// @ai:effects pure
    pub fn new(client: Arc<C>, templates: PromptTemplates, run_config: RunConfig) -> Self {
        Self {
            client,
            templates,
            run_config,
        }
    }

    /// @ai:intent Build the prompt for a task (SAME for both modes)
    ///            Only includes task name and description - tests are hidden
    /// @ai:effects pure
    fn build_prompt(&self, task: &Task) -> String {
        format!(
            "## Task: {}\n\n**Language:** {}\n\n{}\n\n\
             Please provide a complete implementation with all necessary types, \
             traits, and functions. Use proper error handling and include \
             appropriate documentation.",
            task.name,
            task.language.as_str(),
            task.description
        )
    }

    /// @ai:intent Create task context for execution
    /// @ai:effects pure
    fn create_task_context(&self, task: &Task, mode: PromptMode) -> TaskContext {
        TaskContext {
            task_id: task.id.clone(),
            mode: mode.as_str().to_string(),
            use_aicms_skill: mode == PromptMode::Aicms,
        }
    }

    /// @ai:intent Execute a single task once
    /// @ai:effects network
    async fn execute_once(
        &self,
        task: &Task,
        mode: PromptMode,
        repetition: u32,
    ) -> Result<ExecutionResult> {
        let prompt = self.build_prompt(task);
        let context = self.create_task_context(task, mode);

        let start = std::time::Instant::now();

        if self.run_config.dry_run {
            return Ok(ExecutionResult {
                task_id: task.id.clone(),
                mode,
                repetition,
                response: "[DRY RUN] No actual API call made".to_string(),
                input_tokens: 0,
                output_tokens: 0,
                execution_time_ms: 0,
            });
        }

        // Use baseline template as system prompt (same for both modes)
        // The difference is in the CLAUDE.md file for AICMS mode
        let response = self.client.send_message(&prompt, Some(&self.templates.baseline), &context).await?;
        let elapsed = start.elapsed();

        Ok(ExecutionResult {
            task_id: task.id.clone(),
            mode,
            repetition,
            response: response.content,
            input_tokens: response.input_tokens,
            output_tokens: response.output_tokens,
            execution_time_ms: elapsed.as_millis() as u64,
        })
    }

    /// @ai:intent Execute a task with all repetitions and modes
    /// @ai:effects network
    pub async fn execute_task(&self, task: &Task) -> Result<Vec<ExecutionResult>> {
        let mut results = Vec::new();

        for rep in 0..self.run_config.repetitions {
            for mode in [PromptMode::Baseline, PromptMode::Aicms] {
                tracing::info!(
                    "Executing {} (mode={}, rep={})",
                    task.id,
                    mode.as_str(),
                    rep
                );

                let result = self.execute_once(task, mode, rep).await?;
                results.push(result);
            }
        }

        Ok(results)
    }

    /// @ai:intent Execute all tasks
    /// @ai:effects network
    pub async fn execute_all(&self, tasks: &[Task]) -> Result<Vec<ExecutionResult>> {
        let mut all_results = Vec::new();

        for task in tasks {
            let results = self.execute_task(task).await?;
            all_results.extend(results);
        }

        Ok(all_results)
    }
}

/// @ai:intent Create executor from config
/// @ai:effects fs:read
pub fn create_executor<C: ClaudeClientTrait>(
    client: Arc<C>,
    config: &BenchmarkConfig,
) -> Result<BenchmarkExecutor<C>> {
    let templates = PromptTemplates::load(&config.paths.prompts_dir, &config.paths.skill_file)?;
    Ok(BenchmarkExecutor::new(client, templates, config.run.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::{Difficulty, Language, TaskCategory};
    use crate::runner::client::MockClaudeClient;

    fn create_test_task() -> Task {
        Task {
            id: "test-task".to_string(),
            name: "Test Task".to_string(),
            category: TaskCategory::Implement,
            language: Language::Rust,
            difficulty: Difficulty::Easy,
            description: "Implement a test function".to_string(),
        }
    }

    #[tokio::test]
    async fn test_dry_run_execution() {
        let client = Arc::new(MockClaudeClient::new("response".to_string()));
        let templates = PromptTemplates {
            baseline: "You are a coding assistant.".to_string(),
            aicms_skill: "skill".to_string(),
        };
        let run_config = RunConfig {
            repetitions: 1,
            dry_run: true,
            ..Default::default()
        };

        let executor = BenchmarkExecutor::new(client, templates, run_config);
        let task = create_test_task();

        let results = executor.execute_task(&task).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].response.contains("DRY RUN"));
    }

    #[test]
    fn test_strip_aicms_annotations() {
        let code = r#"//! @ai:module:intent User service
//! @ai:module:layer application

/// @ai:intent Create a new user
/// @ai:pre request.email must be valid
/// @ai:effects db:write
pub fn create(request: Request) -> User {
    todo!()
}

// Regular comment stays
fn helper() {}
"#;

        let stripped = strip_aicms_annotations(code);

        // AICMS annotations should be removed
        assert!(!stripped.contains("@ai:intent"));
        assert!(!stripped.contains("@ai:module"));
        assert!(!stripped.contains("@ai:pre"));
        assert!(!stripped.contains("@ai:effects"));

        // Code and regular comments should remain
        assert!(stripped.contains("pub fn create"));
        assert!(stripped.contains("fn helper"));
        assert!(stripped.contains("Regular comment stays"));
    }

    #[test]
    fn test_strip_annotations_python() {
        let code = r#"# @ai:module:intent User service
# @ai:intent Create user
# @ai:effects db:write
def create_user(request):
    pass

# Regular comment
def helper():
    pass
"#;

        let stripped = strip_aicms_annotations(code);

        assert!(!stripped.contains("@ai:intent"));
        assert!(stripped.contains("def create_user"));
        assert!(stripped.contains("Regular comment"));
    }
}
