//! @ai:module:intent Configuration structs for benchmark system
//! @ai:module:layer infrastructure
//! @ai:module:public_api BenchmarkConfig, ApiConfig, RunConfig, FilterConfig
//! @ai:module:stateless true

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// @ai:intent Main configuration for the benchmark system
/// @ai:effects pure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub api: ApiConfig,
    pub run: RunConfig,
    pub paths: PathConfig,
}

/// @ai:intent API configuration for Claude client
/// @ai:effects pure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default)]
    pub temperature: f32,
    #[serde(default = "default_rate_limit")]
    pub requests_per_minute: u32,
}

/// @ai:intent Run configuration for benchmark execution
/// @ai:effects pure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConfig {
    #[serde(default = "default_repetitions")]
    pub repetitions: u32,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub filter: FilterConfig,
}

/// @ai:intent Path configuration for input/output directories
/// @ai:effects pure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfig {
    pub corpus_dir: PathBuf,
    pub prompts_dir: PathBuf,
    pub results_dir: PathBuf,
    pub skill_file: PathBuf,
    #[serde(default = "default_comparison_prompt")]
    pub comparison_prompt_file: PathBuf,
}

/// @ai:intent Filter configuration for selecting tasks
/// @ai:effects pure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilterConfig {
    pub categories: Option<Vec<String>>,
    pub languages: Option<Vec<String>>,
    pub difficulties: Option<Vec<String>>,
    pub task_ids: Option<Vec<String>>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            api: ApiConfig::default(),
            run: RunConfig::default(),
            paths: PathConfig::default(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            model: default_model(),
            max_tokens: default_max_tokens(),
            temperature: 0.0,
            requests_per_minute: default_rate_limit(),
        }
    }
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            repetitions: default_repetitions(),
            dry_run: false,
            filter: FilterConfig::default(),
        }
    }
}

impl Default for PathConfig {
    fn default() -> Self {
        Self {
            corpus_dir: PathBuf::from("corpus"),
            prompts_dir: PathBuf::from("prompts"),
            results_dir: PathBuf::from("results"),
            skill_file: PathBuf::from("../skills/aicms/SKILL.md"),
            comparison_prompt_file: default_comparison_prompt(),
        }
    }
}

fn default_comparison_prompt() -> PathBuf {
    PathBuf::from("prompts/comparison.md")
}

fn default_model() -> String {
    "claude-sonnet-4-20250514".to_string()
}

fn default_max_tokens() -> u32 {
    4096
}

fn default_rate_limit() -> u32 {
    60
}

fn default_repetitions() -> u32 {
    1
}

impl BenchmarkConfig {
    /// @ai:intent Load configuration from a TOML file
    /// @ai:pre path exists and is readable
    /// @ai:effects fs:read
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// @ai:intent Save configuration to a TOML file
    /// @ai:effects fs:write
    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

impl FilterConfig {
    /// @ai:intent Check if filter matches a task
    /// @ai:effects pure
    pub fn matches(&self, category: &str, language: &str, difficulty: &str, id: &str) -> bool {
        let category_match = self
            .categories
            .as_ref()
            .map(|c| c.iter().any(|cat| cat == category))
            .unwrap_or(true);

        let language_match = self
            .languages
            .as_ref()
            .map(|l| l.iter().any(|lang| lang == language))
            .unwrap_or(true);

        let difficulty_match = self
            .difficulties
            .as_ref()
            .map(|d| d.iter().any(|diff| diff == difficulty))
            .unwrap_or(true);

        let id_match = self
            .task_ids
            .as_ref()
            .map(|ids| ids.iter().any(|task_id| task_id == id))
            .unwrap_or(true);

        category_match && language_match && difficulty_match && id_match
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_matches_all_when_empty() {
        let filter = FilterConfig::default();
        assert!(filter.matches("implement", "rust", "easy", "test-task"));
    }

    #[test]
    fn test_filter_matches_specific_category() {
        let filter = FilterConfig {
            categories: Some(vec!["implement".to_string()]),
            ..Default::default()
        };
        assert!(filter.matches("implement", "rust", "easy", "test-task"));
        assert!(!filter.matches("bugfix", "rust", "easy", "test-task"));
    }

    #[test]
    fn test_filter_matches_multiple_criteria() {
        let filter = FilterConfig {
            categories: Some(vec!["implement".to_string()]),
            languages: Some(vec!["rust".to_string(), "python".to_string()]),
            ..Default::default()
        };
        assert!(filter.matches("implement", "rust", "easy", "test-task"));
        assert!(filter.matches("implement", "python", "medium", "other-task"));
        assert!(!filter.matches("implement", "typescript", "easy", "test-task"));
    }
}
