//! @ai:module:intent TOML corpus loader for benchmark tasks
//! @ai:module:layer infrastructure
//! @ai:module:public_api CorpusLoader
//! @ai:module:stateless true

use crate::config::FilterConfig;
use crate::corpus::task::{Task, TaskFile};
use anyhow::{Context, Result};
use std::path::Path;
use walkdir::WalkDir;

/// @ai:intent Trait for loading task corpus
pub trait CorpusLoaderTrait: Send + Sync {
    /// @ai:intent Load all tasks from corpus directory
    fn load_all(&self, corpus_dir: &Path) -> Result<Vec<Task>>;

    /// @ai:intent Load tasks matching filter criteria
    fn load_filtered(&self, corpus_dir: &Path, filter: &FilterConfig) -> Result<Vec<Task>>;

    /// @ai:intent Load a single task by ID
    fn load_by_id(&self, corpus_dir: &Path, task_id: &str) -> Result<Option<Task>>;
}

/// @ai:intent Loads task definitions from TOML files
/// @ai:effects pure (stateless)
pub struct CorpusLoader;

impl CorpusLoader {
    /// @ai:intent Create a new corpus loader
    /// @ai:effects pure
    pub fn new() -> Self {
        Self
    }

    /// @ai:intent Parse a single task file
    /// @ai:pre path points to a valid TOML file
    /// @ai:effects fs:read
    fn parse_task_file(path: &Path) -> Result<Task> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read task file: {}", path.display()))?;

        let task_file: TaskFile = toml::from_str(&content)
            .with_context(|| format!("Failed to parse task file: {}", path.display()))?;

        Ok(task_file.into())
    }

    /// @ai:intent Find all TOML files in directory
    /// @ai:effects fs:read
    fn find_task_files(corpus_dir: &Path) -> Vec<std::path::PathBuf> {
        WalkDir::new(corpus_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "toml")
                    .unwrap_or(false)
            })
            .map(|e| e.path().to_path_buf())
            .collect()
    }
}

impl Default for CorpusLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl CorpusLoaderTrait for CorpusLoader {
    /// @ai:intent Load all tasks from corpus directory
    /// @ai:effects fs:read
    fn load_all(&self, corpus_dir: &Path) -> Result<Vec<Task>> {
        let files = Self::find_task_files(corpus_dir);
        let mut tasks = Vec::with_capacity(files.len());

        for path in files {
            match Self::parse_task_file(&path) {
                Ok(task) => tasks.push(task),
                Err(e) => {
                    tracing::warn!("Skipping invalid task file {}: {}", path.display(), e);
                }
            }
        }

        tasks.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(tasks)
    }

    /// @ai:intent Load tasks matching filter criteria
    /// @ai:effects fs:read
    fn load_filtered(&self, corpus_dir: &Path, filter: &FilterConfig) -> Result<Vec<Task>> {
        let all_tasks = self.load_all(corpus_dir)?;

        let filtered: Vec<Task> = all_tasks
            .into_iter()
            .filter(|task| {
                filter.matches(
                    task.category.as_str(),
                    task.language.as_str(),
                    task.difficulty.as_str(),
                    &task.id,
                )
            })
            .collect();

        Ok(filtered)
    }

    /// @ai:intent Load a single task by ID
    /// @ai:effects fs:read
    fn load_by_id(&self, corpus_dir: &Path, task_id: &str) -> Result<Option<Task>> {
        let all_tasks = self.load_all(corpus_dir)?;
        Ok(all_tasks.into_iter().find(|t| t.id == task_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_task(dir: &Path, name: &str, content: &str) {
        let path = dir.join(name);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let mut file = std::fs::File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_load_single_task() {
        let temp = TempDir::new().unwrap();
        let content = r#"
[task]
id = "test-task"
name = "Test Task"
category = "implement"
language = "rust"
difficulty = "easy"
description = "A test task"
"#;
        create_test_task(temp.path(), "test.toml", content);

        let loader = CorpusLoader::new();
        let tasks = loader.load_all(temp.path()).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "test-task");
    }

    #[test]
    fn test_load_filtered_by_language() {
        let temp = TempDir::new().unwrap();

        let rust_task = r#"
[task]
id = "rust-task"
name = "Rust Task"
category = "implement"
language = "rust"
difficulty = "easy"
description = "A rust task"
"#;

        let python_task = r#"
[task]
id = "python-task"
name = "Python Task"
category = "implement"
language = "python"
difficulty = "easy"
description = "A python task"
"#;

        create_test_task(temp.path(), "rust.toml", rust_task);
        create_test_task(temp.path(), "python.toml", python_task);

        let loader = CorpusLoader::new();
        let filter = FilterConfig {
            languages: Some(vec!["rust".to_string()]),
            ..Default::default()
        };

        let tasks = loader.load_filtered(temp.path(), &filter).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "rust-task");
    }
}
