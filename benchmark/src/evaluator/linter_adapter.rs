//! @ai:module:intent AICMS linter integration for annotation validation
//! @ai:module:layer infrastructure
//! @ai:module:public_api LinterAdapter, LintResult, LintIssue
//! @ai:module:stateless true

use regex::Regex;

/// @ai:intent Result of linting AICMS annotations
#[derive(Debug, Clone)]
pub struct LintResult {
    pub issues: Vec<LintIssue>,
    pub annotation_count: u32,
    pub valid_annotation_count: u32,
}

impl LintResult {
    /// @ai:intent Calculate compliance rate
    /// @ai:effects pure
    pub fn compliance_rate(&self) -> f64 {
        if self.annotation_count == 0 {
            0.0
        } else {
            (self.valid_annotation_count as f64 / self.annotation_count as f64) * 100.0
        }
    }
}

/// @ai:intent A linting issue found in code
#[derive(Debug, Clone)]
pub struct LintIssue {
    pub severity: Severity,
    pub message: String,
    pub line: Option<u32>,
}

/// @ai:intent Severity level of a lint issue
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// @ai:intent Trait for linting AICMS annotations
pub trait LinterAdapterTrait: Send + Sync {
    /// @ai:intent Lint code for AICMS annotation issues
    fn lint(&self, code: &str) -> LintResult;
}

/// @ai:intent AICMS linter for validating annotations
pub struct LinterAdapter {
    annotation_regex: Regex,
    valid_tags: Vec<&'static str>,
}

impl LinterAdapter {
    /// @ai:intent Create a new linter adapter
    /// @ai:effects pure
    pub fn new() -> Self {
        Self {
            annotation_regex: Regex::new(r"@ai:(\w+(?::\w+)*)(?:\s+(.*))?").unwrap(),
            valid_tags: vec![
                "intent",
                "pre",
                "post",
                "invariant",
                "example",
                "effects",
                "idempotent",
                "retry_safe",
                "confidence",
                "needs_review",
                "author",
                "verified",
                "assumes",
                "context",
                "related",
                "deprecated",
                "complexity",
                "edge_cases",
                "override",
                "constraint",  // Alias for pre, commonly generated
                "test:integration",
                "module:intent",
                "module:layer",
                "module:bounded_context",
                "module:public_api",
                "module:depends_on",
                "module:depended_by",
                "module:internal",
                "module:stateless",
                "module:thread_safe",
                "module:cohesion",
                "module:stability",
                "project:max_function_lines",
                "project:max_file_lines",
                "project:max_functions_per_file",
                "project:max_structs_per_module",
                "project:max_params",
                "project:max_return_values",
                "project:max_nesting_depth",
                "project:max_cyclomatic_complexity",
                "project:extract_repeated_code",
                "project:require_interface_for_deps",
                "project:single_responsibility",
                "project:prefer_composition",
                "project:no_god_objects",
                "project:no_primitive_obsession",
                "project:immutable_by_default",
                "project:architecture",
                "project:layers",
                "project:dependency_rule",
                "project:error_strategy",
                "project:require_error_types",
                "project:no_panic",
                "project:min_coverage",
                "project:unit_tests",
                "project:integration_tests",
                "project:integration_tests_tools",
                "project:test_naming",
            ],
        }
    }

    /// @ai:intent Check if a tag is valid
    /// @ai:effects pure
    fn is_valid_tag(&self, tag: &str) -> bool {
        self.valid_tags.contains(&tag) || tag.starts_with("override:")
    }

    /// @ai:intent Validate effects value
    /// @ai:effects pure
    fn validate_effects(&self, value: &str) -> Option<LintIssue> {
        let valid_effects = [
            "pure",
            "io",
            "db:read",
            "db:write",
            "network",
            "fs:read",
            "fs:write",
            "env",
            "state:read",
            "state:write",
            "random",
            "time",
        ];

        for effect in value.split(',').map(|s| s.trim()) {
            if !valid_effects.contains(&effect) {
                return Some(LintIssue {
                    severity: Severity::Warning,
                    message: format!("Unknown effect: {}", effect),
                    line: None,
                });
            }
        }

        None
    }

    /// @ai:intent Validate confidence value
    /// @ai:effects pure
    fn validate_confidence(&self, value: &str) -> Option<LintIssue> {
        match value.parse::<f64>() {
            Ok(v) if (0.0..=1.0).contains(&v) => None,
            Ok(v) => Some(LintIssue {
                severity: Severity::Error,
                message: format!("Confidence must be between 0.0 and 1.0, got {}", v),
                line: None,
            }),
            Err(_) => Some(LintIssue {
                severity: Severity::Error,
                message: format!("Invalid confidence value: {}", value),
                line: None,
            }),
        }
    }
}

impl Default for LinterAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl LinterAdapterTrait for LinterAdapter {
    /// @ai:intent Lint code for AICMS annotation issues
    /// @ai:effects pure
    fn lint(&self, code: &str) -> LintResult {
        let mut issues = Vec::new();
        let mut annotation_count = 0u32;
        let mut valid_count = 0u32;
        let mut has_intent = false;

        for (line_num, line) in code.lines().enumerate() {
            for cap in self.annotation_regex.captures_iter(line) {
                annotation_count += 1;
                let tag = &cap[1];
                let value = cap.get(2).map(|m| m.as_str().trim()).unwrap_or("");

                if !self.is_valid_tag(tag) {
                    issues.push(LintIssue {
                        severity: Severity::Error,
                        message: format!("Unknown annotation tag: @ai:{}", tag),
                        line: Some(line_num as u32 + 1),
                    });
                    continue;
                }

                if tag == "intent" || tag == "module:intent" {
                    has_intent = true;

                    if value.is_empty() {
                        issues.push(LintIssue {
                            severity: Severity::Error,
                            message: "Intent annotation must have a value".to_string(),
                            line: Some(line_num as u32 + 1),
                        });
                        continue;
                    }
                }

                if tag == "effects" {
                    if let Some(issue) = self.validate_effects(value) {
                        issues.push(LintIssue {
                            line: Some(line_num as u32 + 1),
                            ..issue
                        });
                        continue;
                    }
                }

                if tag == "confidence" {
                    if let Some(issue) = self.validate_confidence(value) {
                        issues.push(LintIssue {
                            line: Some(line_num as u32 + 1),
                            ..issue
                        });
                        continue;
                    }
                }

                valid_count += 1;
            }
        }

        if annotation_count > 0 && !has_intent {
            issues.push(LintIssue {
                severity: Severity::Warning,
                message: "Missing @ai:intent annotation (required for all functions)".to_string(),
                line: None,
            });
        }

        LintResult {
            issues,
            annotation_count,
            valid_annotation_count: valid_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lint_valid_annotations() {
        let linter = LinterAdapter::new();
        let code = r#"
/// @ai:intent Calculate factorial
/// @ai:pre n >= 0
/// @ai:effects pure
fn factorial(n: u64) -> u64 { 1 }
"#;

        let result = linter.lint(code);
        assert_eq!(result.annotation_count, 3);
        assert_eq!(result.valid_annotation_count, 3);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_lint_invalid_tag() {
        let linter = LinterAdapter::new();
        let code = "/// @ai:invalid_tag something";

        let result = linter.lint(code);
        assert_eq!(result.issues.len(), 2); // invalid tag + missing intent
        assert!(result.issues[0].message.contains("Unknown annotation"));
    }

    #[test]
    fn test_lint_invalid_confidence() {
        let linter = LinterAdapter::new();
        let code = r#"
/// @ai:intent Test
/// @ai:confidence 1.5
"#;

        let result = linter.lint(code);
        assert!(result.issues.iter().any(|i| i.message.contains("0.0 and 1.0")));
    }

    #[test]
    fn test_compliance_rate() {
        let result = LintResult {
            issues: vec![],
            annotation_count: 10,
            valid_annotation_count: 8,
        };
        assert!((result.compliance_rate() - 80.0).abs() < 0.01);
    }
}
