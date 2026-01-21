//! @ai:module:intent Parse source files and extract comment blocks
//! @ai:module:layer application
//! @ai:module:public_api parse_file, CommentBlock
//! @ai:module:depends_on language, error
//! @ai:module:stateless true

use crate::error::{Error, Result};
use crate::language::{detect_language, Language};
use regex::Regex;
use std::path::Path;

/// @ai:intent Represents a block of consecutive comments
#[derive(Debug, Clone)]
pub struct CommentBlock {
    pub lines: Vec<CommentLine>,
    pub start_line: usize,
    pub end_line: usize,
}

/// @ai:intent Represents a single comment line
#[derive(Debug, Clone)]
pub struct CommentLine {
    pub line_number: usize,
    pub content: String,
    pub is_doc_comment: bool,
}

/// @ai:intent Parsed source file with extracted comments
#[derive(Debug)]
pub struct ParsedSource {
    pub language: Language,
    pub comment_blocks: Vec<CommentBlock>,
    pub function_locations: Vec<FunctionLocation>,
}

/// @ai:intent Location of a function definition in source
#[derive(Debug, Clone)]
pub struct FunctionLocation {
    pub name: String,
    pub line: usize,
    pub preceding_comment_block: Option<usize>,
}

/// @ai:intent Parse a source file and extract comment blocks
/// @ai:pre path exists and is readable
/// @ai:post result contains all comment blocks and function locations
/// @ai:effects fs:read
pub fn parse_file(path: &Path) -> Result<ParsedSource> {
    let language = detect_language(path)
        .ok_or_else(|| Error::UnsupportedFileType(path.display().to_string()))?;

    let content = std::fs::read_to_string(path).map_err(|e| Error::FileRead {
        path: path.to_path_buf(),
        source: e,
    })?;

    let comment_blocks = extract_comment_blocks(&content, language);
    let function_locations = extract_function_locations(&content, language, &comment_blocks);

    Ok(ParsedSource {
        language,
        comment_blocks,
        function_locations,
    })
}

/// @ai:intent Extract all comment blocks from source content
/// @ai:effects pure
fn extract_comment_blocks(content: &str, language: Language) -> Vec<CommentBlock> {
    let style = language.comment_style();
    let mut blocks = Vec::new();
    let mut current_block: Option<CommentBlock> = None;

    for (line_idx, line) in content.lines().enumerate() {
        let line_number = line_idx + 1;
        let trimmed = line.trim();

        if let Some(comment) = extract_single_line_comment(trimmed, &style) {
            let is_doc = is_doc_comment(trimmed, &style);

            let comment_line = CommentLine {
                line_number,
                content: comment,
                is_doc_comment: is_doc,
            };

            match &mut current_block {
                Some(block) => {
                    block.lines.push(comment_line);
                    block.end_line = line_number;
                }
                None => {
                    current_block = Some(CommentBlock {
                        lines: vec![comment_line],
                        start_line: line_number,
                        end_line: line_number,
                    });
                }
            }
        } else if !trimmed.is_empty() {
            if let Some(block) = current_block.take() {
                blocks.push(block);
            }
        }
    }

    if let Some(block) = current_block {
        blocks.push(block);
    }

    blocks
}

/// @ai:intent Extract comment content from a single line
/// @ai:effects pure
fn extract_single_line_comment(line: &str, style: &crate::language::CommentStyle) -> Option<String> {
    for prefix in style.doc_line.iter().chain(style.single_line.iter()) {
        if line.starts_with(prefix) {
            let content = line[prefix.len()..].trim();
            return Some(content.to_string());
        }
    }

    if let (Some(start), Some(end)) = (style.block_start, style.block_end) {
        if line.starts_with(start) && line.ends_with(end) && line.len() > start.len() + end.len() {
            let content = &line[start.len()..line.len() - end.len()];
            return Some(content.trim().to_string());
        }

        if let Some(prefix) = style.block_line_prefix {
            let trimmed = line.trim_start();
            if trimmed.starts_with(prefix) {
                return Some(trimmed[prefix.len()..].trim().to_string());
            }
        }
    }

    None
}

/// @ai:intent Check if a line is a documentation comment
/// @ai:effects pure
fn is_doc_comment(line: &str, style: &crate::language::CommentStyle) -> bool {
    style.doc_line.iter().any(|prefix| line.starts_with(prefix))
}

/// @ai:intent Extract function locations from source content
/// @ai:effects pure
fn extract_function_locations(
    content: &str,
    language: Language,
    comment_blocks: &[CommentBlock],
) -> Vec<FunctionLocation> {
    let pattern = get_function_pattern(language);
    let re = Regex::new(&pattern).expect("Invalid regex pattern");

    let mut locations = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        let line_number = line_idx + 1;

        if let Some(captures) = re.captures(line) {
            let name = captures
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let preceding_block = find_preceding_comment_block(line_number, comment_blocks);

            locations.push(FunctionLocation {
                name,
                line: line_number,
                preceding_comment_block: preceding_block,
            });
        }
    }

    locations
}

/// @ai:intent Get regex pattern for function definitions in a language
/// @ai:effects pure
fn get_function_pattern(language: Language) -> String {
    match language {
        Language::Rust => r"^\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)".to_string(),
        Language::Python => r"^\s*(?:async\s+)?def\s+(\w+)".to_string(),
        Language::TypeScript | Language::JavaScript => {
            r"^\s*(?:export\s+)?(?:async\s+)?function\s+(\w+)".to_string()
        }
        Language::Go => r"^\s*func\s+(?:\([^)]*\)\s+)?(\w+)".to_string(),
        Language::Java => {
            r"^\s*(?:public|private|protected)?\s*(?:static\s+)?(?:\w+\s+)+(\w+)\s*\(".to_string()
        }
        Language::C | Language::Cpp => r"^\s*(?:\w+\s+)+(\w+)\s*\(".to_string(),
    }
}

/// @ai:intent Find the comment block immediately preceding a line
/// @ai:effects pure
fn find_preceding_comment_block(line: usize, blocks: &[CommentBlock]) -> Option<usize> {
    for (idx, block) in blocks.iter().enumerate() {
        if block.end_line == line - 1 || block.end_line == line - 2 {
            return Some(idx);
        }
    }
    None
}

impl CommentBlock {
    /// @ai:intent Check if this block contains any @ai: annotations
    /// @ai:effects pure
    pub fn has_ai_annotations(&self) -> bool {
        self.lines.iter().any(|l| l.content.contains("@ai:"))
    }

    /// @ai:intent Get all lines containing @ai: annotations
    /// @ai:effects pure
    pub fn ai_annotation_lines(&self) -> Vec<&CommentLine> {
        self.lines
            .iter()
            .filter(|l| l.content.contains("@ai:"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rust_comment() {
        let style = Language::Rust.comment_style();
        assert_eq!(
            extract_single_line_comment("/// @ai:intent Test", &style),
            Some("@ai:intent Test".to_string())
        );
    }

    #[test]
    fn test_extract_python_comment() {
        let style = Language::Python.comment_style();
        assert_eq!(
            extract_single_line_comment("# @ai:intent Test", &style),
            Some("@ai:intent Test".to_string())
        );
    }
}
