//! @ai:module:intent Format output for different formats (JSON, text)
//! @ai:module:layer infrastructure
//! @ai:module:public_api OutputFormat, format_lint_result, format_parsed_file
//! @ai:module:depends_on linter, annotation
//! @ai:module:stateless true

use crate::annotation::ParsedFile;
use crate::diff::{ChangeType, DiffResult};
use crate::linter::{LintResult, Severity};
use colored::Colorize;
use serde::Serialize;

/// @ai:intent Output format options
#[derive(Debug, Clone, Copy, Default)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    JsonPretty,
}

/// @ai:intent Format lint results as a string
/// @ai:effects pure
pub fn format_lint_result(result: &LintResult, format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string(result).unwrap_or_default(),
        OutputFormat::JsonPretty => {
            serde_json::to_string_pretty(result).unwrap_or_default()
        }
        OutputFormat::Text => format_lint_result_text(result),
    }
}

/// @ai:intent Format lint results as human-readable text
/// @ai:effects pure
fn format_lint_result_text(result: &LintResult) -> String {
    let mut output = String::new();

    for issue in &result.issues {
        let severity_str = match issue.severity {
            Severity::Error => "ERROR".red().bold(),
            Severity::Warning => "WARN".yellow().bold(),
            Severity::Info => "INFO".blue(),
        };

        let location = format!(
            "{}:{}",
            issue.location.file.display(),
            issue.location.line
        );

        output.push_str(&format!(
            "{} {} - {} ({})\n",
            severity_str,
            location.dimmed(),
            issue.message,
            issue.code.dimmed()
        ));

        if let Some(suggestion) = &issue.suggestion {
            output.push_str(&format!("  {} {}\n", "hint:".cyan(), suggestion));
        }
    }

    output.push('\n');
    output.push_str(&format!(
        "Checked {} files, {} functions\n",
        result.files_checked, result.functions_checked
    ));

    if result.errors > 0 {
        output.push_str(&format!(
            "{} errors, {} warnings\n",
            result.errors.to_string().red().bold(),
            result.warnings.to_string().yellow()
        ));
    } else if result.warnings > 0 {
        output.push_str(&format!(
            "{} {} warnings\n",
            "OK".green().bold(),
            result.warnings.to_string().yellow()
        ));
    } else {
        output.push_str(&format!("{} No issues found\n", "OK".green().bold()));
    }

    output
}

/// @ai:intent Format parsed file as JSON
/// @ai:effects pure
pub fn format_parsed_file(file: &ParsedFile, format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string(file).unwrap_or_default(),
        OutputFormat::JsonPretty => serde_json::to_string_pretty(file).unwrap_or_default(),
        OutputFormat::Text => format_parsed_file_text(file),
    }
}

/// @ai:intent Format parsed file as human-readable text
/// @ai:effects pure
fn format_parsed_file_text(file: &ParsedFile) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "{} ({})\n",
        file.path.display().to_string().bold(),
        file.language
    ));

    if let Some(intent) = &file.module.intent {
        output.push_str(&format!("  Module: {}\n", intent));
    }

    if let Some(layer) = &file.module.layer {
        output.push_str(&format!("  Layer: {}\n", layer));
    }

    output.push_str(&format!("\n  Functions ({}):\n", file.module.functions.len()));

    for func in &file.module.functions {
        output.push_str(&format!("    {} (line {})\n", func.name.cyan(), func.location.line));

        if let Some(intent) = &func.intent {
            output.push_str(&format!("      intent: {}\n", intent));
        }

        if !func.effects.is_empty() {
            output.push_str(&format!("      effects: {}\n", func.effects.join(", ")));
        }

        if let Some(confidence) = func.confidence {
            output.push_str(&format!("      confidence: {:.2}\n", confidence));
        }
    }

    output
}

/// @ai:intent Format any serializable value as JSON
/// @ai:effects pure
pub fn to_json<T: Serialize>(value: &T, pretty: bool) -> String {
    if pretty {
        serde_json::to_string_pretty(value).unwrap_or_default()
    } else {
        serde_json::to_string(value).unwrap_or_default()
    }
}

/// @ai:intent Format diff results as a string
/// @ai:effects pure
pub fn format_diff_result(result: &DiffResult, format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string(result).unwrap_or_default(),
        OutputFormat::JsonPretty => serde_json::to_string_pretty(result).unwrap_or_default(),
        OutputFormat::Text => format_diff_result_text(result),
    }
}

/// @ai:intent Format diff results as human-readable text
/// @ai:effects pure
fn format_diff_result_text(result: &DiffResult) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "AICMS Semantic Diff: {}\n\n",
        result.file_path.bold()
    ));

    // Group changes by type
    let breaking: Vec<_> = result
        .changes
        .iter()
        .filter(|c| c.change_type == ChangeType::Breaking)
        .collect();

    let notable: Vec<_> = result
        .changes
        .iter()
        .filter(|c| c.change_type == ChangeType::Notable)
        .collect();

    let non_breaking: Vec<_> = result
        .changes
        .iter()
        .filter(|c| c.change_type == ChangeType::NonBreaking)
        .collect();

    // Breaking changes
    if !breaking.is_empty() {
        output.push_str(&format!("{}\n", "🔴 BREAKING CHANGES".red().bold()));

        for change in breaking {
            output.push_str(&format!("  {}():\n", change.function_name.cyan()));
            output.push_str(&format!("    - {} {}\n", change.tag.yellow(), change.description));

            if let Some(old) = &change.old_value {
                output.push_str(&format!("      Old: {}\n", old.dimmed()));
            }

            if let Some(new) = &change.new_value {
                output.push_str(&format!("      New: {}\n", new));
            }

            output.push('\n');
        }
    }

    // Notable changes
    if !notable.is_empty() {
        output.push_str(&format!("{}\n", "🟡 NOTABLE CHANGES".yellow().bold()));

        for change in notable {
            output.push_str(&format!("  {}():\n", change.function_name.cyan()));
            output.push_str(&format!("    - {} {}\n", change.tag.yellow(), change.description));

            if let Some(old) = &change.old_value {
                output.push_str(&format!("      Old: {}\n", old.dimmed()));
            }

            if let Some(new) = &change.new_value {
                output.push_str(&format!("      New: {}\n", new));
            }

            output.push('\n');
        }
    }

    // Non-breaking changes
    if !non_breaking.is_empty() {
        output.push_str(&format!("{}\n", "🟢 NON-BREAKING CHANGES".green().bold()));

        for change in non_breaking {
            output.push_str(&format!("  {}():\n", change.function_name.cyan()));
            output.push_str(&format!(
                "    - {} {} {}\n",
                change.tag.yellow(),
                change.description,
                "✓".green()
            ));

            if let Some(old) = &change.old_value {
                output.push_str(&format!("      Old: {}\n", old.dimmed()));
            }

            if let Some(new) = &change.new_value {
                output.push_str(&format!("      New: {}\n", new));
            }

            output.push('\n');
        }
    }

    // Summary
    output.push_str(&format!(
        "Summary: {} breaking, {} notable, {} non-breaking changes\n",
        if result.breaking_count > 0 {
            result.breaking_count.to_string().red().bold().to_string()
        } else {
            "0".to_string()
        },
        if result.notable_count > 0 {
            result.notable_count.to_string().yellow().to_string()
        } else {
            "0".to_string()
        },
        result.non_breaking_count
    ));

    output
}
