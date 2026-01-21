//! @ai:module:intent Extract code blocks from Claude responses
//! @ai:module:layer application
//! @ai:module:public_api CodeExtractor, ExtractedCode, ExtractedFile
//! @ai:module:stateless true

use crate::corpus::Language;
use crate::evaluator::SourceFile;
use regex::Regex;

/// @ai:intent Extracted code from a response
#[derive(Debug, Clone)]
pub struct ExtractedCode {
    pub code: String,
    pub language: Option<Language>,
}

/// @ai:intent Extracted file with path from a multi-file response
#[derive(Debug, Clone)]
pub struct ExtractedFile {
    pub path: String,
    pub code: String,
    pub language: Option<Language>,
}

/// @ai:intent Trait for code extraction
pub trait CodeExtractorTrait: Send + Sync {
    /// @ai:intent Extract code blocks from response text
    fn extract(&self, response: &str, expected_lang: Language) -> Vec<ExtractedCode>;

    /// @ai:intent Extract the primary code block
    fn extract_primary(&self, response: &str, expected_lang: Language) -> Option<ExtractedCode>;

    /// @ai:intent Extract multiple files from a multi-file response
    fn extract_files(&self, response: &str, expected_lang: Language) -> Vec<ExtractedFile>;
}

/// @ai:intent Extracts code blocks from markdown-formatted responses
pub struct CodeExtractor {
    code_block_regex: Regex,
    /// Matches code blocks with file paths like ```rust:src/lib.rs
    file_code_block_regex: Regex,
    /// Matches file path comments like // file: src/lib.rs or # file: main.py
    file_marker_regex: Regex,
}

impl CodeExtractor {
    /// @ai:intent Create a new code extractor
    /// @ai:effects pure
    pub fn new() -> Self {
        Self {
            code_block_regex: Regex::new(r"```(\w*)\n([\s\S]*?)```").unwrap(),
            // Matches ```rust:path/to/file.rs or ```python:file.py
            file_code_block_regex: Regex::new(r"```(\w+):([^\n]+)\n([\s\S]*?)```").unwrap(),
            // Matches // file: path or # file: path at start of code block
            file_marker_regex: Regex::new(r"^(?://|#)\s*file:\s*(.+)$").unwrap(),
        }
    }

    /// @ai:intent Parse language identifier from code fence
    /// @ai:effects pure
    fn parse_language(lang_str: &str) -> Option<Language> {
        match lang_str.to_lowercase().as_str() {
            "rust" | "rs" => Some(Language::Rust),
            "python" | "py" => Some(Language::Python),
            "typescript" | "ts" | "javascript" | "js" => Some(Language::TypeScript),
            _ => None,
        }
    }

    /// @ai:intent Extract file path from first line comment if present
    /// @ai:effects pure
    fn extract_file_marker(&self, code: &str) -> Option<String> {
        let first_line = code.lines().next()?;

        self.file_marker_regex
            .captures(first_line)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().trim().to_string())
    }

    /// @ai:intent Remove file marker comment from code
    /// @ai:effects pure
    fn strip_file_marker(&self, code: &str) -> String {
        if self.extract_file_marker(code).is_some() {
            code.lines().skip(1).collect::<Vec<_>>().join("\n")
        } else {
            code.to_string()
        }
    }

    /// @ai:intent Convert extracted files to SourceFile format
    /// @ai:effects pure
    pub fn to_source_files(&self, files: &[ExtractedFile]) -> Vec<SourceFile> {
        files
            .iter()
            .map(|f| SourceFile {
                path: f.path.clone(),
                content: f.code.clone(),
            })
            .collect()
    }
}

impl Default for CodeExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeExtractorTrait for CodeExtractor {
    /// @ai:intent Extract all code blocks from response text
    /// @ai:effects pure
    fn extract(&self, response: &str, expected_lang: Language) -> Vec<ExtractedCode> {
        self.code_block_regex
            .captures_iter(response)
            .map(|cap| {
                let lang_str = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let code = cap.get(2).map(|m| m.as_str()).unwrap_or("");
                let language = Self::parse_language(lang_str);

                ExtractedCode {
                    code: code.trim().to_string(),
                    language,
                }
            })
            .filter(|ec| ec.language.is_none() || ec.language == Some(expected_lang))
            .collect()
    }

    /// @ai:intent Extract the primary (first matching) code block
    /// @ai:effects pure
    fn extract_primary(&self, response: &str, expected_lang: Language) -> Option<ExtractedCode> {
        let all = self.extract(response, expected_lang);

        all.into_iter()
            .find(|ec| ec.language == Some(expected_lang))
            .or_else(|| {
                self.extract(response, expected_lang)
                    .into_iter()
                    .find(|ec| ec.language.is_none() && !ec.code.is_empty())
            })
    }

    /// @ai:intent Extract multiple files from a multi-file response
    ///            Supports formats: ```rust:path/file.rs or // file: path comment
    /// @ai:effects pure
    fn extract_files(&self, response: &str, expected_lang: Language) -> Vec<ExtractedFile> {
        let mut files = Vec::new();

        // First try extracting with path in code fence: ```rust:src/lib.rs
        for cap in self.file_code_block_regex.captures_iter(response) {
            let lang_str = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let path = cap.get(2).map(|m| m.as_str().trim()).unwrap_or("");
            let code = cap.get(3).map(|m| m.as_str().trim()).unwrap_or("");
            let language = Self::parse_language(lang_str);

            if language.is_none() || language == Some(expected_lang) {
                files.push(ExtractedFile {
                    path: path.to_string(),
                    code: code.to_string(),
                    language,
                });
            }
        }

        // If no files with paths in fence, try file marker comments
        if files.is_empty() {
            for cap in self.code_block_regex.captures_iter(response) {
                let lang_str = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let code = cap.get(2).map(|m| m.as_str().trim()).unwrap_or("");
                let language = Self::parse_language(lang_str);

                if language.is_none() || language == Some(expected_lang) {
                    if let Some(path) = self.extract_file_marker(code) {
                        let clean_code = self.strip_file_marker(code);
                        files.push(ExtractedFile {
                            path,
                            code: clean_code,
                            language,
                        });
                    }
                }
            }
        }

        // Fallback: if still no files, extract all blocks and assign default paths
        if files.is_empty() {
            let blocks = self.extract(response, expected_lang);

            for (i, block) in blocks.into_iter().enumerate() {
                let ext = expected_lang.extension();
                let path = if i == 0 {
                    format!("main.{ext}")
                } else {
                    format!("file{i}.{ext}")
                };

                files.push(ExtractedFile {
                    path,
                    code: block.code,
                    language: block.language,
                });
            }
        }

        files
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rust_code() {
        let extractor = CodeExtractor::new();
        let response = r#"
Here's the implementation:

```rust
fn factorial(n: u64) -> u64 {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}
```
"#;

        let blocks = extractor.extract(response, Language::Rust);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].code.contains("factorial"));
        assert_eq!(blocks[0].language, Some(Language::Rust));
    }

    #[test]
    fn test_extract_primary_prefers_expected_language() {
        let extractor = CodeExtractor::new();
        let response = r#"
```python
def test(): pass
```

```rust
fn test() {}
```
"#;

        let primary = extractor.extract_primary(response, Language::Rust);
        assert!(primary.is_some());
        assert!(primary.unwrap().code.contains("fn test"));
    }

    #[test]
    fn test_extract_unlabeled_code_block() {
        let extractor = CodeExtractor::new();
        let response = r#"
```
fn test() {}
```
"#;

        let primary = extractor.extract_primary(response, Language::Rust);
        assert!(primary.is_some());
    }

    #[test]
    fn test_extract_files_with_path_in_fence() {
        let extractor = CodeExtractor::new();
        let response = r#"
Here are the files:

```rust:src/lib.rs
pub mod user;
pub mod repository;
```

```rust:src/user.rs
pub struct User {
    pub id: u64,
    pub name: String,
}
```
"#;

        let files = extractor.extract_files(response, Language::Rust);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, "src/lib.rs");
        assert_eq!(files[1].path, "src/user.rs");
        assert!(files[0].code.contains("pub mod user"));
        assert!(files[1].code.contains("struct User"));
    }

    #[test]
    fn test_extract_files_with_marker_comment() {
        let extractor = CodeExtractor::new();
        let response = r#"
```rust
// file: src/main.rs
fn main() {
    println!("Hello");
}
```

```rust
// file: src/lib.rs
pub fn hello() {}
```
"#;

        let files = extractor.extract_files(response, Language::Rust);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, "src/main.rs");
        assert_eq!(files[1].path, "src/lib.rs");
        // Marker comment should be stripped
        assert!(!files[0].code.contains("// file:"));
    }

    #[test]
    fn test_extract_files_fallback_no_paths() {
        let extractor = CodeExtractor::new();
        let response = r#"
```rust
fn main() {}
```

```rust
fn helper() {}
```
"#;

        let files = extractor.extract_files(response, Language::Rust);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, "main.rs");
        assert_eq!(files[1].path, "file1.rs");
    }
}
