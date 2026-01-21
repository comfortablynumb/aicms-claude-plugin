//! @ai:module:intent Define language-specific comment formats
//! @ai:module:layer domain
//! @ai:module:public_api Language, detect_language
//! @ai:module:stateless true

use std::path::Path;

/// @ai:intent Represents a supported programming language with its comment syntax
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    Python,
    TypeScript,
    JavaScript,
    Go,
    Java,
    C,
    Cpp,
}

/// @ai:intent Comment style configuration for a language
#[derive(Debug, Clone)]
pub struct CommentStyle {
    pub single_line: Vec<&'static str>,
    pub doc_line: Vec<&'static str>,
    pub block_start: Option<&'static str>,
    pub block_end: Option<&'static str>,
    pub block_line_prefix: Option<&'static str>,
}

impl Language {
    /// @ai:intent Get the comment style for this language
    /// @ai:effects pure
    pub fn comment_style(&self) -> CommentStyle {
        match self {
            Language::Rust => CommentStyle {
                single_line: vec!["//"],
                doc_line: vec!["///", "//!"],
                block_start: Some("/*"),
                block_end: Some("*/"),
                block_line_prefix: Some("*"),
            },
            Language::Python => CommentStyle {
                single_line: vec!["#"],
                doc_line: vec!["#"],
                block_start: Some("\"\"\""),
                block_end: Some("\"\"\""),
                block_line_prefix: None,
            },
            Language::TypeScript | Language::JavaScript => CommentStyle {
                single_line: vec!["//"],
                doc_line: vec!["//"],
                block_start: Some("/*"),
                block_end: Some("*/"),
                block_line_prefix: Some("*"),
            },
            Language::Go => CommentStyle {
                single_line: vec!["//"],
                doc_line: vec!["//"],
                block_start: Some("/*"),
                block_end: Some("*/"),
                block_line_prefix: Some("*"),
            },
            Language::Java => CommentStyle {
                single_line: vec!["//"],
                doc_line: vec!["//"],
                block_start: Some("/*"),
                block_end: Some("*/"),
                block_line_prefix: Some("*"),
            },
            Language::C | Language::Cpp => CommentStyle {
                single_line: vec!["//"],
                doc_line: vec!["//"],
                block_start: Some("/*"),
                block_end: Some("*/"),
                block_line_prefix: Some("*"),
            },
        }
    }

    /// @ai:intent Get file extensions for this language
    /// @ai:effects pure
    pub fn extensions(&self) -> &[&str] {
        match self {
            Language::Rust => &["rs"],
            Language::Python => &["py", "pyi"],
            Language::TypeScript => &["ts", "tsx"],
            Language::JavaScript => &["js", "jsx", "mjs"],
            Language::Go => &["go"],
            Language::Java => &["java"],
            Language::C => &["c", "h"],
            Language::Cpp => &["cpp", "cc", "cxx", "hpp", "hh", "hxx"],
        }
    }

    /// @ai:intent Get language name as string
    /// @ai:effects pure
    pub fn name(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::Python => "python",
            Language::TypeScript => "typescript",
            Language::JavaScript => "javascript",
            Language::Go => "go",
            Language::Java => "java",
            Language::C => "c",
            Language::Cpp => "cpp",
        }
    }
}

/// @ai:intent Detect the programming language from a file path
/// @ai:pre path is a valid file path
/// @ai:post result is Some if extension is recognized
/// @ai:example ("test.rs") -> Some(Rust)
/// @ai:example ("test.py") -> Some(Python)
/// @ai:example ("test.txt") -> None
/// @ai:effects pure
pub fn detect_language(path: &Path) -> Option<Language> {
    let ext = path.extension()?.to_str()?;

    let all_languages = [
        Language::Rust,
        Language::Python,
        Language::TypeScript,
        Language::JavaScript,
        Language::Go,
        Language::Java,
        Language::C,
        Language::Cpp,
    ];

    for lang in all_languages {
        if lang.extensions().contains(&ext) {
            return Some(lang);
        }
    }

    None
}

/// @ai:intent Check if a file should be parsed based on extension
/// @ai:effects pure
pub fn is_supported_file(path: &Path) -> bool {
    detect_language(path).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_rust() {
        assert_eq!(
            detect_language(Path::new("test.rs")),
            Some(Language::Rust)
        );
    }

    #[test]
    fn test_detect_python() {
        assert_eq!(
            detect_language(Path::new("test.py")),
            Some(Language::Python)
        );
    }

    #[test]
    fn test_detect_typescript() {
        assert_eq!(
            detect_language(Path::new("test.ts")),
            Some(Language::TypeScript)
        );
    }

    #[test]
    fn test_unsupported() {
        assert_eq!(detect_language(Path::new("test.txt")), None);
    }
}
