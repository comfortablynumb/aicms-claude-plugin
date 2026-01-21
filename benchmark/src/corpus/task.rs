//! @ai:module:intent Task definitions for benchmark corpus
//! @ai:module:layer domain
//! @ai:module:public_api Task, TaskCategory, Language, Difficulty
//! @ai:module:stateless true

use serde::{Deserialize, Serialize};

/// @ai:intent Category of benchmark task
/// @ai:effects pure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskCategory {
    Implement,
    Bugfix,
    Refactor,
    Inference,
}

impl TaskCategory {
    /// @ai:intent Convert category to string representation
    /// @ai:effects pure
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskCategory::Implement => "implement",
            TaskCategory::Bugfix => "bugfix",
            TaskCategory::Refactor => "refactor",
            TaskCategory::Inference => "inference",
        }
    }
}

impl std::fmt::Display for TaskCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// @ai:intent Programming language for the task
/// @ai:effects pure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    Python,
    TypeScript,
}

impl Language {
    /// @ai:intent Convert language to string representation
    /// @ai:effects pure
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::Python => "python",
            Language::TypeScript => "typescript",
        }
    }

    /// @ai:intent Get file extension for this language
    /// @ai:effects pure
    pub fn extension(&self) -> &'static str {
        match self {
            Language::Rust => "rs",
            Language::Python => "py",
            Language::TypeScript => "ts",
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// @ai:intent Difficulty level of the task
/// @ai:effects pure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    /// @ai:intent Convert difficulty to string representation
    /// @ai:effects pure
    pub fn as_str(&self) -> &'static str {
        match self {
            Difficulty::Easy => "easy",
            Difficulty::Medium => "medium",
            Difficulty::Hard => "hard",
        }
    }
}

impl std::fmt::Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// @ai:intent A benchmark task definition
/// @ai:effects pure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub category: TaskCategory,
    pub language: Language,
    pub difficulty: Difficulty,
    /// Description shown to Claude - the only input for implement tasks
    pub description: String,
}

/// @ai:intent Raw task structure from TOML file
/// @ai:effects pure
#[derive(Debug, Deserialize)]
pub struct TaskFile {
    pub task: TaskMetadata,
}

/// @ai:intent Task metadata from TOML file
/// @ai:effects pure
#[derive(Debug, Deserialize)]
pub struct TaskMetadata {
    pub id: String,
    pub name: String,
    pub category: TaskCategory,
    pub language: Language,
    pub difficulty: Difficulty,
    pub description: String,
}

impl From<TaskFile> for Task {
    fn from(file: TaskFile) -> Self {
        Task {
            id: file.task.id,
            name: file.task.name,
            category: file.task.category,
            language: file.task.language,
            difficulty: file.task.difficulty,
            description: file.task.description,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_extension() {
        assert_eq!(Language::Rust.extension(), "rs");
        assert_eq!(Language::Python.extension(), "py");
        assert_eq!(Language::TypeScript.extension(), "ts");
    }

    #[test]
    fn test_category_as_str() {
        assert_eq!(TaskCategory::Implement.as_str(), "implement");
        assert_eq!(TaskCategory::Bugfix.as_str(), "bugfix");
    }
}
