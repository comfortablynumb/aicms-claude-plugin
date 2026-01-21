//! @ai:module:intent Claude Code CLI client for benchmark execution
//! @ai:module:layer infrastructure
//! @ai:module:public_api ClaudeCodeClient
//! @ai:module:stateless true

use crate::runner::client::{ClaudeClientTrait, ClaudeResponse, TaskContext};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::Write;

/// @ai:intent Client that uses Claude Code CLI instead of direct API
pub struct ClaudeCodeClient {
    model: Option<String>,
    /// Base output directory (results/{timestamp}/)
    output_dir: PathBuf,
    /// Path to the AICMS skill file
    skill_file: PathBuf,
}

impl ClaudeCodeClient {
    /// @ai:intent Create a new Claude Code CLI client with output directory
    /// @ai:effects fs:write
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            model: None,
            output_dir,
            skill_file: PathBuf::from("../skills/aicms/SKILL.md"),
        }
    }

    /// @ai:intent Create client with specific model and output directory
    /// @ai:effects fs:write
    pub fn with_model(model: String, output_dir: PathBuf) -> Self {
        Self {
            model: Some(model),
            output_dir,
            skill_file: PathBuf::from("../skills/aicms/SKILL.md"),
        }
    }

    /// @ai:intent Set the skill file path
    /// @ai:effects pure
    pub fn with_skill_file(mut self, path: PathBuf) -> Self {
        self.skill_file = path;
        self
    }

    /// @ai:intent Get the code directory for a mode (baseline/aicms)
    /// @ai:effects pure
    fn get_code_dir(&self, mode: &str) -> PathBuf {
        self.output_dir.join(mode).join("code")
    }

    /// @ai:intent Get the report directory for a mode (baseline/aicms)
    /// @ai:effects pure
    fn get_report_dir(&self, mode: &str) -> PathBuf {
        self.output_dir.join(mode).join("report")
    }

    /// @ai:intent Create fresh directories for this run (code and report)
    /// @ai:effects fs:write
    fn create_run_dirs(&self, task_id: &str, mode: &str) -> Result<(PathBuf, PathBuf)> {
        let code_dir = self.get_code_dir(mode).join(task_id);
        let report_dir = self.get_report_dir(mode).join(task_id);

        // Clean up if exists from previous run
        if code_dir.exists() {
            std::fs::remove_dir_all(&code_dir)?;
        }
        std::fs::create_dir_all(&code_dir)?;

        if report_dir.exists() {
            std::fs::remove_dir_all(&report_dir)?;
        }
        std::fs::create_dir_all(&report_dir)?;

        Ok((code_dir, report_dir))
    }

    /// @ai:intent Create CLAUDE.md file for AICMS mode that imports the skill
    /// @ai:effects fs:write
    fn create_aicms_claude_md(&self, code_dir: &PathBuf) -> Result<()> {
        // Get absolute path to skill file
        let skill_path = if self.skill_file.is_absolute() {
            self.skill_file.clone()
        } else {
            std::env::current_dir()?.join(&self.skill_file)
        };

        let skill_path_str = skill_path.to_string_lossy().replace('\\', "/");

        let claude_md_content = format!(
            "# AICMS Benchmark\n\n\
             @{}\n\n\
             When working in this codebase:\n\
             - Read and respect existing @ai:* annotations\n\
             - Add annotations to new functions you create\n\
             - Validate implementations against their specs\n\
             - Flag discrepancies between intent and implementation\n",
            skill_path_str
        );

        std::fs::write(code_dir.join("CLAUDE.md"), claude_md_content)?;
        Ok(())
    }

    /// @ai:intent Read all source files from directory recursively
    /// @ai:effects fs:read
    fn collect_generated_files(&self, dir: &PathBuf) -> Result<Vec<(String, String)>> {
        let mut files = Vec::new();
        self.collect_files_recursive(dir, dir, &mut files)?;
        Ok(files)
    }

    /// @ai:intent Recursively collect files from directory
    /// @ai:effects fs:read
    fn collect_files_recursive(
        &self,
        base: &PathBuf,
        current: &PathBuf,
        files: &mut Vec<(String, String)>,
    ) -> Result<()> {
        if !current.is_dir() {
            return Ok(());
        }

        for entry in std::fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Skip hidden directories and target/
                let name = path.file_name().unwrap_or_default().to_string_lossy();

                if !name.starts_with('.') && name != "target" {
                    self.collect_files_recursive(base, &path, files)?;
                }
            } else {
                // Include source files, exclude CLAUDE.md
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                let ext = path.extension().unwrap_or_default().to_string_lossy();

                if name == "CLAUDE.md" {
                    continue;
                }

                if matches!(ext.as_ref(), "rs" | "py" | "ts" | "js" | "toml" | "json") {
                    let relative = path.strip_prefix(base).unwrap_or(&path);
                    let content = std::fs::read_to_string(&path)?;
                    files.push((relative.to_string_lossy().to_string(), content));
                }
            }
        }

        Ok(())
    }

    /// @ai:intent Format collected files as markdown code blocks
    /// @ai:effects pure
    fn format_files_as_markdown(&self, files: &[(String, String)], language: &str) -> String {
        files
            .iter()
            .map(|(path, content)| {
                format!("```{}:{}\n{}\n```", language, path, content)
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

impl ClaudeClientTrait for ClaudeCodeClient {
    /// @ai:intent Send a message using Claude Code CLI in agentic mode
    /// @ai:effects io, fs:write, fs:read
    async fn send_message(
        &self,
        prompt: &str,
        _system: Option<&str>,
        context: &TaskContext,
    ) -> Result<ClaudeResponse> {
        // Create fresh directories for code and reports
        let (code_dir, report_dir) = self.create_run_dirs(&context.task_id, &context.mode)?;

        // For AICMS mode, create CLAUDE.md that imports the skill
        if context.use_aicms_skill {
            self.create_aicms_claude_md(&code_dir)?;
            tracing::info!("Created CLAUDE.md with AICMS skill import");
        }

        // Detect language from prompt
        let language = detect_language(prompt);

        // Build the prompt (SAME for both modes - no system prompt difference)
        let full_prompt = build_prompt(prompt);

        let mut cmd = Command::new("claude");

        // Run in agentic mode with stdin prompt
        cmd.arg("--print");
        cmd.arg("--verbose");

        // Bypass all permissions so Claude can run cargo test, etc.
        cmd.arg("--dangerously-skip-permissions");

        // Skip user's home settings to avoid influencing generation
        cmd.arg("--setting-sources").arg("project,local");

        // Add model flag if specified
        if let Some(ref model) = self.model {
            cmd.arg("--model").arg(model);
        }

        // Run from the code directory
        cmd.current_dir(&code_dir);

        // Set up stdin for the prompt
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        tracing::info!(
            "Running Claude in {} mode (skill={}) in {}",
            context.mode,
            context.use_aicms_skill,
            code_dir.display()
        );

        let mut child = cmd
            .spawn()
            .context("Failed to execute claude CLI. Is Claude Code installed?")?;

        // Write prompt to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(full_prompt.as_bytes())
                .context("Failed to write prompt to claude stdin")?;
        }

        let output = child
            .wait_with_output()
            .context("Failed to wait for claude process")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Save interaction log in report directory
        let log_path = report_dir.join("_claude_interaction.log");
        let log_content = format!(
            "=== MODE ===\n{} (use_aicms_skill={})\n\n\
             === PROMPT ===\n{}\n\n\
             === STDOUT ===\n{}\n\n\
             === STDERR ===\n{}\n\n\
             === EXIT CODE ===\n{:?}",
            context.mode, context.use_aicms_skill, full_prompt, stdout, stderr, output.status.code()
        );
        std::fs::write(&log_path, &log_content).ok();
        tracing::info!("Saved interaction log to {}", log_path.display());

        if !output.status.success() {
            tracing::warn!(
                "Claude CLI returned non-zero exit code: {:?}",
                output.status.code()
            );
            tracing::warn!("stderr: {}", stderr);
        }

        // Collect all generated files from the code directory
        let generated_files = self.collect_generated_files(&code_dir)?;

        tracing::info!(
            "Collected {} files from {}: {:?}",
            generated_files.len(),
            code_dir.display(),
            generated_files.iter().map(|(p, _)| p).collect::<Vec<_>>()
        );

        // Also log stdout preview if no files generated
        if generated_files.is_empty() {
            let preview = truncate_string(&stdout, 500);
            tracing::warn!("No files generated. stdout preview:\n{}", preview);
        }

        // Format as markdown code blocks for the evaluator
        let content = if generated_files.is_empty() {
            // If no files were generated, return Claude's stdout (might contain code blocks)
            stdout
        } else {
            self.format_files_as_markdown(&generated_files, language)
        };

        // Estimate tokens
        let estimated_input_tokens = (full_prompt.len() / 4) as u32;
        let estimated_output_tokens = (content.len() / 4) as u32;

        Ok(ClaudeResponse {
            content,
            input_tokens: estimated_input_tokens,
            output_tokens: estimated_output_tokens,
            stop_reason: "end_turn".to_string(),
        })
    }
}

/// @ai:intent Detect programming language from prompt text
/// @ai:effects pure
fn detect_language(prompt: &str) -> &'static str {
    if prompt.contains("Rust") || prompt.contains("rust") {
        "rust"
    } else if prompt.contains("Python") || prompt.contains("python") {
        "python"
    } else if prompt.contains("TypeScript") || prompt.contains("typescript") {
        "typescript"
    } else {
        "rust" // default
    }
}

/// @ai:intent Build prompt with instructions (SAME for both modes)
/// @ai:effects pure
fn build_prompt(prompt: &str) -> String {
    format!(
        "{}\n\n---\n\n\
         Write all files to the current directory. \
         Create a proper project structure (e.g., src/ for Rust, proper modules for Python). \
         Include comprehensive tests. Run the tests to verify they pass.",
        prompt
    )
}

/// @ai:intent Truncate string with ellipsis if too long
/// @ai:effects pure
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len])
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_client_creation() {
        let temp = TempDir::new().unwrap();
        let client = ClaudeCodeClient::new(temp.path().to_path_buf());
        assert!(client.model.is_none());

        let temp2 = TempDir::new().unwrap();
        let client = ClaudeCodeClient::with_model("sonnet".to_string(), temp2.path().to_path_buf());
        assert_eq!(client.model, Some("sonnet".to_string()));
    }

    #[test]
    fn test_format_files_as_markdown() {
        let temp = TempDir::new().unwrap();
        let client = ClaudeCodeClient::new(temp.path().to_path_buf());
        let files = vec![
            ("src/lib.rs".to_string(), "pub mod user;".to_string()),
            ("src/user.rs".to_string(), "pub struct User {}".to_string()),
        ];

        let markdown = client.format_files_as_markdown(&files, "rust");

        assert!(markdown.contains("```rust:src/lib.rs"));
        assert!(markdown.contains("pub mod user;"));
        assert!(markdown.contains("```rust:src/user.rs"));
        assert!(markdown.contains("pub struct User {}"));
    }

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language("Write a Rust function"), "rust");
        assert_eq!(detect_language("Write a rust function"), "rust");
        assert_eq!(detect_language("Write a Python function"), "python");
        assert_eq!(detect_language("Write a python function"), "python");
        assert_eq!(detect_language("Write a TypeScript function"), "typescript");
        assert_eq!(detect_language("Write a typescript function"), "typescript");
        assert_eq!(detect_language("Write a function"), "rust"); // default
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(truncate_string("this is a long string", 10), "this is a ...");
    }

    #[test]
    fn test_directory_paths() {
        let temp = TempDir::new().unwrap();
        let client = ClaudeCodeClient::new(temp.path().to_path_buf());

        let code_dir = client.get_code_dir("baseline");
        assert!(code_dir.ends_with("baseline/code"));

        let report_dir = client.get_report_dir("aicms");
        assert!(report_dir.ends_with("aicms/report"));
    }

    #[test]
    fn test_build_prompt_same_for_both_modes() {
        let prompt = "## Task: Test\n\nDescription";
        let result = build_prompt(prompt);

        // Should contain the prompt
        assert!(result.contains("## Task: Test"));
        assert!(result.contains("Description"));
        // Should contain instructions
        assert!(result.contains("Write all files to the current directory"));
    }
}
