//! @ai:module:intent Compare annotations between two file versions for semantic changes
//! @ai:module:layer application
//! @ai:module:public_api diff_files, DiffResult, ContractChange, ChangeType
//! @ai:module:depends_on annotation, extractor
//! @ai:module:stateless true

use crate::annotation::{FunctionAnnotations, ParsedFile};
use crate::extractor::extract_file;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// @ai:intent Severity of a contract change
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Breaking,
    Notable,
    NonBreaking,
}

/// @ai:intent A single contract change detected between versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractChange {
    pub function_name: String,
    pub change_type: ChangeType,
    pub tag: String,
    pub description: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

/// @ai:intent Result of comparing two file versions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiffResult {
    pub file_path: String,
    pub changes: Vec<ContractChange>,
    pub breaking_count: usize,
    pub notable_count: usize,
    pub non_breaking_count: usize,
}

impl DiffResult {
    /// @ai:intent Check if there are any breaking changes
    pub fn has_breaking_changes(&self) -> bool {
        self.breaking_count > 0
    }

    /// @ai:intent Add a change and update counts
    fn add_change(&mut self, change: ContractChange) {
        match change.change_type {
            ChangeType::Breaking => self.breaking_count += 1,
            ChangeType::Notable => self.notable_count += 1,
            ChangeType::NonBreaking => self.non_breaking_count += 1,
        }
        self.changes.push(change);
    }
}

/// @ai:intent Compare two files and detect contract changes
/// @ai:effects fs:read
pub fn diff_files(old_path: &Path, new_path: &Path) -> Result<DiffResult> {
    let old_parsed = extract_file(old_path)?;
    let new_parsed = extract_file(new_path)?;

    Ok(diff_parsed(&old_parsed, &new_parsed))
}

/// @ai:intent Compare two parsed files
/// @ai:effects pure
pub fn diff_parsed(old: &ParsedFile, new: &ParsedFile) -> DiffResult {
    let mut result = DiffResult {
        file_path: new.path.display().to_string(),
        ..Default::default()
    };

    let old_funcs: std::collections::HashMap<&str, &FunctionAnnotations> = old
        .module
        .functions
        .iter()
        .map(|f| (f.name.as_str(), f))
        .collect();

    let new_funcs: std::collections::HashMap<&str, &FunctionAnnotations> = new
        .module
        .functions
        .iter()
        .map(|f| (f.name.as_str(), f))
        .collect();

    for (name, new_func) in &new_funcs {
        if let Some(old_func) = old_funcs.get(name) {
            compare_functions(&mut result, old_func, new_func);
        }
    }

    result
}

/// @ai:intent Compare annotations between two function versions
/// @ai:effects pure
fn compare_functions(
    result: &mut DiffResult,
    old: &FunctionAnnotations,
    new: &FunctionAnnotations,
) {
    let func_name = &new.name;

    // Compare @ai:pre (preconditions)
    compare_preconditions(result, func_name, &old.pre, &new.pre);

    // Compare @ai:post (postconditions)
    compare_postconditions(result, func_name, &old.post, &new.post);

    // Compare @ai:effects
    compare_effects(result, func_name, &old.effects, &new.effects);

    // Compare @ai:idempotent
    if old.idempotent != new.idempotent {
        if old.idempotent == Some(true) && new.idempotent != Some(true) {
            result.add_change(ContractChange {
                function_name: func_name.clone(),
                change_type: ChangeType::Breaking,
                tag: "@ai:idempotent".to_string(),
                description: "Function is no longer idempotent".to_string(),
                old_value: Some("true".to_string()),
                new_value: new.idempotent.map(|v| v.to_string()),
            });
        } else if new.idempotent == Some(true) {
            result.add_change(ContractChange {
                function_name: func_name.clone(),
                change_type: ChangeType::NonBreaking,
                tag: "@ai:idempotent".to_string(),
                description: "Function is now idempotent".to_string(),
                old_value: old.idempotent.map(|v| v.to_string()),
                new_value: Some("true".to_string()),
            });
        }
    }

    // Compare @ai:intent (notable change)
    if old.intent != new.intent && old.intent.is_some() && new.intent.is_some() {
        result.add_change(ContractChange {
            function_name: func_name.clone(),
            change_type: ChangeType::Notable,
            tag: "@ai:intent".to_string(),
            description: "Intent description changed".to_string(),
            old_value: old.intent.clone(),
            new_value: new.intent.clone(),
        });
    }

    // Compare @ai:confidence (notable if significant change)
    if let (Some(old_conf), Some(new_conf)) = (old.confidence, new.confidence) {
        let diff = (old_conf - new_conf).abs();

        if diff >= 0.1 {
            let change_type = if new_conf < old_conf {
                ChangeType::Notable
            } else {
                ChangeType::NonBreaking
            };

            result.add_change(ContractChange {
                function_name: func_name.clone(),
                change_type,
                tag: "@ai:confidence".to_string(),
                description: format!(
                    "Confidence {} from {:.2} to {:.2}",
                    if new_conf < old_conf {
                        "decreased"
                    } else {
                        "increased"
                    },
                    old_conf,
                    new_conf
                ),
                old_value: Some(format!("{:.2}", old_conf)),
                new_value: Some(format!("{:.2}", new_conf)),
            });
        }
    }

    // @ai:needs_review added (notable)
    if old.needs_review.is_none() && new.needs_review.is_some() {
        result.add_change(ContractChange {
            function_name: func_name.clone(),
            change_type: ChangeType::Notable,
            tag: "@ai:needs_review".to_string(),
            description: format!(
                "Review flag added: {}",
                new.needs_review.as_ref().unwrap()
            ),
            old_value: None,
            new_value: new.needs_review.clone(),
        });
    }

    // @ai:deprecated added (notable)
    if old.deprecated.is_none() && new.deprecated.is_some() {
        result.add_change(ContractChange {
            function_name: func_name.clone(),
            change_type: ChangeType::Notable,
            tag: "@ai:deprecated".to_string(),
            description: format!(
                "Function deprecated: {}",
                new.deprecated.as_ref().unwrap()
            ),
            old_value: None,
            new_value: new.deprecated.clone(),
        });
    }
}

/// @ai:intent Compare preconditions and detect strengthening (breaking) vs weakening (ok)
fn compare_preconditions(
    result: &mut DiffResult,
    func_name: &str,
    old_pre: &[String],
    new_pre: &[String],
) {
    // New preconditions added = BREAKING (stricter requirements)
    for new_cond in new_pre {
        if !old_pre.contains(new_cond) {
            result.add_change(ContractChange {
                function_name: func_name.to_string(),
                change_type: ChangeType::Breaking,
                tag: "@ai:pre".to_string(),
                description: "Precondition strengthened (new requirement added)".to_string(),
                old_value: None,
                new_value: Some(new_cond.clone()),
            });
        }
    }

    // Old preconditions removed = OK (less strict)
    for old_cond in old_pre {
        if !new_pre.contains(old_cond) {
            result.add_change(ContractChange {
                function_name: func_name.to_string(),
                change_type: ChangeType::NonBreaking,
                tag: "@ai:pre".to_string(),
                description: "Precondition weakened (requirement removed)".to_string(),
                old_value: Some(old_cond.clone()),
                new_value: None,
            });
        }
    }
}

/// @ai:intent Compare postconditions and detect weakening (breaking) vs strengthening (ok)
fn compare_postconditions(
    result: &mut DiffResult,
    func_name: &str,
    old_post: &[String],
    new_post: &[String],
) {
    // Old postconditions removed = BREAKING (weaker guarantee)
    for old_cond in old_post {
        if !new_post.contains(old_cond) {
            result.add_change(ContractChange {
                function_name: func_name.to_string(),
                change_type: ChangeType::Breaking,
                tag: "@ai:post".to_string(),
                description: "Postcondition weakened (guarantee removed)".to_string(),
                old_value: Some(old_cond.clone()),
                new_value: None,
            });
        }
    }

    // New postconditions added = OK (stronger guarantee)
    for new_cond in new_post {
        if !old_post.contains(new_cond) {
            result.add_change(ContractChange {
                function_name: func_name.to_string(),
                change_type: ChangeType::NonBreaking,
                tag: "@ai:post".to_string(),
                description: "Postcondition strengthened (new guarantee added)".to_string(),
                old_value: None,
                new_value: Some(new_cond.clone()),
            });
        }
    }
}

/// @ai:intent Compare effects and detect expansion (breaking) vs reduction (ok)
fn compare_effects(
    result: &mut DiffResult,
    func_name: &str,
    old_effects: &[String],
    new_effects: &[String],
) {
    let was_pure = old_effects.is_empty() || old_effects.contains(&"pure".to_string());
    let is_pure = new_effects.is_empty() || new_effects.contains(&"pure".to_string());

    // Pure -> not pure = BREAKING
    if was_pure && !is_pure {
        result.add_change(ContractChange {
            function_name: func_name.to_string(),
            change_type: ChangeType::Breaking,
            tag: "@ai:effects".to_string(),
            description: "Function is no longer pure (side effects added)".to_string(),
            old_value: Some("pure".to_string()),
            new_value: Some(new_effects.join(", ")),
        });
        return;
    }

    // Not pure -> pure = OK
    if !was_pure && is_pure {
        result.add_change(ContractChange {
            function_name: func_name.to_string(),
            change_type: ChangeType::NonBreaking,
            tag: "@ai:effects".to_string(),
            description: "Function is now pure (side effects removed)".to_string(),
            old_value: Some(old_effects.join(", ")),
            new_value: Some("pure".to_string()),
        });
        return;
    }

    // New effects added = BREAKING
    for new_effect in new_effects {
        if new_effect != "pure" && !old_effects.contains(new_effect) {
            result.add_change(ContractChange {
                function_name: func_name.to_string(),
                change_type: ChangeType::Breaking,
                tag: "@ai:effects".to_string(),
                description: format!("New side effect added: {}", new_effect),
                old_value: None,
                new_value: Some(new_effect.clone()),
            });
        }
    }

    // Effects removed = OK
    for old_effect in old_effects {
        if old_effect != "pure" && !new_effects.contains(old_effect) {
            result.add_change(ContractChange {
                function_name: func_name.to_string(),
                change_type: ChangeType::NonBreaking,
                tag: "@ai:effects".to_string(),
                description: format!("Side effect removed: {}", old_effect),
                old_value: Some(old_effect.clone()),
                new_value: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotation::{FunctionAnnotations, Location, ModuleAnnotations, ParsedFile};
    use std::path::PathBuf;

    fn create_test_file(functions: Vec<FunctionAnnotations>) -> ParsedFile {
        ParsedFile {
            path: PathBuf::from("test.rs"),
            language: "rust".to_string(),
            module: ModuleAnnotations {
                functions,
                ..Default::default()
            },
            raw_annotations: vec![],
        }
    }

    fn create_func(name: &str) -> FunctionAnnotations {
        FunctionAnnotations::new(name.to_string(), Location::default())
    }

    #[test]
    fn test_precondition_strengthened_is_breaking() {
        let mut old_func = create_func("test_fn");
        old_func.pre = vec!["x > 0".to_string()];

        let mut new_func = create_func("test_fn");
        new_func.pre = vec!["x > 0".to_string(), "x < 100".to_string()];

        let old_file = create_test_file(vec![old_func]);
        let new_file = create_test_file(vec![new_func]);

        let result = diff_parsed(&old_file, &new_file);

        assert_eq!(result.breaking_count, 1);
        assert!(result.changes.iter().any(|c| c.tag == "@ai:pre"
            && c.change_type == ChangeType::Breaking));
    }

    #[test]
    fn test_postcondition_weakened_is_breaking() {
        let mut old_func = create_func("test_fn");
        old_func.post = vec!["result >= 0".to_string()];

        let mut new_func = create_func("test_fn");
        new_func.post = vec![];

        let old_file = create_test_file(vec![old_func]);
        let new_file = create_test_file(vec![new_func]);

        let result = diff_parsed(&old_file, &new_file);

        assert_eq!(result.breaking_count, 1);
        assert!(result.changes.iter().any(|c| c.tag == "@ai:post"
            && c.change_type == ChangeType::Breaking));
    }

    #[test]
    fn test_effects_expanded_is_breaking() {
        let mut old_func = create_func("test_fn");
        old_func.effects = vec!["pure".to_string()];

        let mut new_func = create_func("test_fn");
        new_func.effects = vec!["db:write".to_string()];

        let old_file = create_test_file(vec![old_func]);
        let new_file = create_test_file(vec![new_func]);

        let result = diff_parsed(&old_file, &new_file);

        assert_eq!(result.breaking_count, 1);
    }
}
