//! @ai:module:intent Validate required toolchain for benchmark execution
//! @ai:module:layer infrastructure
//! @ai:module:public_api ToolchainValidator, ToolchainStatus, LanguageTools
//! @ai:module:stateless true

use crate::corpus::Language;
use std::collections::HashSet;
use std::process::Command;

/// @ai:intent Tool requirements for each language
#[derive(Debug, Clone)]
pub struct LanguageTools {
    pub language: Language,
    pub compiler: &'static str,
    pub test_args: &'static [&'static str],
}

/// @ai:intent Status of toolchain validation
#[derive(Debug)]
pub struct ToolchainStatus {
    pub available_languages: HashSet<Language>,
    pub missing_tools: Vec<MissingTool>,
}

/// @ai:intent Information about a missing tool
#[derive(Debug)]
pub struct MissingTool {
    pub language: Language,
    pub tool_name: &'static str,
    pub install_hint: &'static str,
}

/// @ai:intent Validates that required tools are installed
pub struct ToolchainValidator;

impl ToolchainValidator {
    /// @ai:intent Get tool requirements for all supported languages
    /// @ai:effects pure
    fn get_language_tools() -> Vec<LanguageTools> {
        vec![
            LanguageTools {
                language: Language::Rust,
                compiler: "rustc",
                test_args: &["--version"],
            },
            LanguageTools {
                language: Language::Python,
                compiler: "python",
                test_args: &["--version"],
            },
            LanguageTools {
                language: Language::TypeScript,
                compiler: "tsc",
                test_args: &["--version"],
            },
        ]
    }

    /// @ai:intent Get install hint for a tool
    /// @ai:effects pure
    fn get_install_hint(tool: &str) -> &'static str {
        match tool {
            "rustc" => "Install Rust: https://rustup.rs/",
            "python" => "Install Python: https://www.python.org/downloads/",
            "tsc" => "Install TypeScript: npm install -g typescript",
            _ => "Check tool documentation for installation instructions",
        }
    }

    /// @ai:intent Check if a command is available on the system
    /// @ai:effects io
    fn is_tool_available(tool: &str, args: &[&str]) -> bool {
        Command::new(tool)
            .args(args)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// @ai:intent Validate all required tools and return status
    /// @ai:effects io
    pub fn validate() -> ToolchainStatus {
        let mut available_languages = HashSet::new();
        let mut missing_tools = Vec::new();

        for lang_tools in Self::get_language_tools() {
            if Self::is_tool_available(lang_tools.compiler, lang_tools.test_args) {
                available_languages.insert(lang_tools.language);
            } else {
                missing_tools.push(MissingTool {
                    language: lang_tools.language,
                    tool_name: lang_tools.compiler,
                    install_hint: Self::get_install_hint(lang_tools.compiler),
                });
            }
        }

        ToolchainStatus {
            available_languages,
            missing_tools,
        }
    }

    /// @ai:intent Log warnings for missing tools
    /// @ai:effects io
    pub fn log_warnings(status: &ToolchainStatus) {
        for missing in &status.missing_tools {
            tracing::warn!(
                "Tool '{}' not found - {} tasks will be skipped. {}",
                missing.tool_name,
                missing.language,
                missing.install_hint
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_language_tools_returns_all_languages() {
        let tools = ToolchainValidator::get_language_tools();
        assert_eq!(tools.len(), 3);
    }

    #[test]
    fn test_get_install_hint_known_tools() {
        assert!(ToolchainValidator::get_install_hint("rustc").contains("rustup"));
        assert!(ToolchainValidator::get_install_hint("python").contains("python.org"));
        assert!(ToolchainValidator::get_install_hint("tsc").contains("npm"));
    }

    #[test]
    fn test_is_tool_available_nonexistent() {
        assert!(!ToolchainValidator::is_tool_available(
            "nonexistent_tool_xyz",
            &["--version"]
        ));
    }
}
