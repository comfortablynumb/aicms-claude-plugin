//! @ai:module:intent Lint source files for AICMS compliance
//! @ai:module:layer application
//! @ai:module:public_api lint_file, lint_directory, LintResult, LintIssue, Severity
//! @ai:module:depends_on extractor, annotation, error
//! @ai:module:stateless true

use crate::annotation::{Location, ParsedFile};
use crate::error::Result;
use crate::extractor::extract_file;
use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;

/// @ai:intent Severity level for lint issues
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// @ai:intent A single lint issue found in the code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintIssue {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub location: Location,
    pub suggestion: Option<String>,
}

/// @ai:intent Configuration for the linter
#[derive(Debug, Clone, Default)]
pub struct LintConfig {
    pub require_intent: bool,
    pub require_module_intent: bool,
    pub require_effects_for_impure: bool,
    pub warn_low_confidence: bool,
    pub confidence_threshold: f32,
}

impl Default for Severity {
    fn default() -> Self {
        Self::Warning
    }
}

impl LintConfig {
    /// @ai:intent Create a strict lint configuration
    pub fn strict() -> Self {
        Self {
            require_intent: true,
            require_module_intent: true,
            require_effects_for_impure: true,
            warn_low_confidence: true,
            confidence_threshold: 0.7,
        }
    }
}

/// @ai:intent Result of linting a file or directory
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LintResult {
    pub files_checked: usize,
    pub functions_checked: usize,
    pub issues: Vec<LintIssue>,
    pub errors: usize,
    pub warnings: usize,
}

impl LintResult {
    /// @ai:intent Check if linting passed (no errors)
    pub fn passed(&self) -> bool {
        self.errors == 0
    }

    /// @ai:intent Merge another lint result into this one
    pub fn merge(&mut self, other: LintResult) {
        self.files_checked += other.files_checked;
        self.functions_checked += other.functions_checked;
        self.issues.extend(other.issues);
        self.errors += other.errors;
        self.warnings += other.warnings;
    }
}

/// @ai:intent Lint a single file
/// @ai:effects fs:read
pub fn lint_file(path: &Path, config: &LintConfig) -> Result<LintResult> {
    let parsed = extract_file(path)?;
    Ok(lint_parsed_file(&parsed, config))
}

/// @ai:intent Lint all supported files in a directory
/// @ai:effects fs:read
pub fn lint_directory(path: &Path, config: &LintConfig) -> Result<LintResult> {
    let mut result = LintResult::default();

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let file_path = entry.path();

        if crate::language::is_supported_file(file_path) {
            match lint_file(file_path, config) {
                Ok(file_result) => result.merge(file_result),
                Err(e) => {
                    result.issues.push(LintIssue {
                        severity: Severity::Error,
                        code: "E000".to_string(),
                        message: format!("Failed to parse file: {}", e),
                        location: Location::new(file_path.to_path_buf(), 0),
                        suggestion: None,
                    });
                    result.errors += 1;
                }
            }
        }
    }

    Ok(result)
}

/// @ai:intent Lint a parsed file
/// @ai:effects pure
fn lint_parsed_file(parsed: &ParsedFile, config: &LintConfig) -> LintResult {
    let mut result = LintResult {
        files_checked: 1,
        functions_checked: parsed.module.functions.len(),
        ..Default::default()
    };

    // Check module-level annotations
    if config.require_module_intent && parsed.module.intent.is_none() {
        result.issues.push(LintIssue {
            severity: Severity::Warning,
            code: "W001".to_string(),
            message: "Module missing @ai:module:intent annotation".to_string(),
            location: Location::new(parsed.path.clone(), 1),
            suggestion: Some("Add //! @ai:module:intent <description>".to_string()),
        });
        result.warnings += 1;
    }

    // Check function-level annotations
    for func in &parsed.module.functions {
        // Check for required intent
        if config.require_intent && func.intent.is_none() {
            result.issues.push(LintIssue {
                severity: Severity::Error,
                code: "E001".to_string(),
                message: format!("Function `{}` missing @ai:intent annotation", func.name),
                location: func.location.clone(),
                suggestion: Some(format!(
                    "Add /// @ai:intent <description> before `{}`",
                    func.name
                )),
            });
            result.errors += 1;
        }

        // Check for low confidence
        if config.warn_low_confidence {
            if let Some(conf) = func.confidence {
                if conf < config.confidence_threshold {
                    result.issues.push(LintIssue {
                        severity: Severity::Warning,
                        code: "W002".to_string(),
                        message: format!(
                            "Function `{}` has low confidence ({:.2} < {:.2})",
                            func.name, conf, config.confidence_threshold
                        ),
                        location: func.location.clone(),
                        suggestion: Some("Consider reviewing and improving confidence".to_string()),
                    });
                    result.warnings += 1;
                }
            }
        }

        // Check for needs_review flag
        if func.needs_review.is_some() {
            result.issues.push(LintIssue {
                severity: Severity::Info,
                code: "I001".to_string(),
                message: format!(
                    "Function `{}` flagged for review: {}",
                    func.name,
                    func.needs_review.as_ref().unwrap()
                ),
                location: func.location.clone(),
                suggestion: None,
            });
        }

        // Check for integration test requirement
        if func.test_integration.is_some() {
            result.issues.push(LintIssue {
                severity: Severity::Info,
                code: "I002".to_string(),
                message: format!(
                    "Function `{}` requires integration test: {}",
                    func.name,
                    func.test_integration.as_ref().unwrap()
                ),
                location: func.location.clone(),
                suggestion: None,
            });
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_lint_missing_intent() {
        let mut file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(
            file,
            r#"fn no_annotation() {{
    println!("hello");
}}"#
        )
        .unwrap();

        let config = LintConfig {
            require_intent: true,
            ..Default::default()
        };

        let result = lint_file(file.path(), &config).unwrap();

        assert_eq!(result.errors, 1);
        assert_eq!(result.issues[0].code, "E001");
    }

    #[test]
    fn test_lint_with_intent() {
        let mut file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(
            file,
            r#"/// @ai:intent Print hello
fn with_annotation() {{
    println!("hello");
}}"#
        )
        .unwrap();

        let config = LintConfig {
            require_intent: true,
            ..Default::default()
        };

        let result = lint_file(file.path(), &config).unwrap();

        assert_eq!(result.errors, 0);
    }
}
