//! @ai:module:intent AICMS parser library for extracting and validating annotations
//! @ai:module:layer infrastructure
//! @ai:module:public_api annotation, extractor, linter, parser, language, output, error
//! @ai:module:stateless true
//!
//! # AICMS Parser
//!
//! A library for parsing AICMS (AI-First Code Metadata Specification) annotations
//! from source files in multiple programming languages.
//!
//! ## Example
//!
//! ```rust,no_run
//! use aicms_parser::{extractor, linter, output};
//! use std::path::Path;
//!
//! // Extract annotations from a file
//! let parsed = extractor::extract_file(Path::new("src/lib.rs")).unwrap();
//! println!("{}", output::format_parsed_file(&parsed, output::OutputFormat::JsonPretty));
//!
//! // Lint a directory
//! let config = linter::LintConfig::strict();
//! let result = linter::lint_directory(Path::new("src"), &config).unwrap();
//! println!("{}", output::format_lint_result(&result, output::OutputFormat::Text));
//! ```

pub mod annotation;
pub mod diff;
pub mod error;
pub mod extractor;
pub mod language;
pub mod linter;
pub mod output;
pub mod parser;

pub use annotation::{
    Annotation, AnnotationLevel, FunctionAnnotations, Location, ModuleAnnotations, ParsedFile,
    ParsedProject,
};
pub use diff::{diff_files, diff_parsed, ChangeType, ContractChange, DiffResult};
pub use error::{Error, Result};
pub use extractor::extract_file;
pub use language::{detect_language, is_supported_file, Language};
pub use linter::{lint_directory, lint_file, LintConfig, LintIssue, LintResult, Severity};
pub use output::{format_diff_result, format_lint_result, format_parsed_file, to_json, OutputFormat};
