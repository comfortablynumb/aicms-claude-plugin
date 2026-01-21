//! @ai:module:intent Extract structured annotations from parsed comments
//! @ai:module:layer application
//! @ai:module:public_api extract_annotations, extract_file
//! @ai:module:depends_on annotation, parser, error
//! @ai:module:stateless true

use crate::annotation::{
    Annotation, AnnotationLevel, FunctionAnnotations, Location, ModuleAnnotations, ParsedFile,
};
use crate::error::Result;
use crate::parser::{parse_file, CommentBlock, ParsedSource};
use regex::Regex;
use std::path::Path;

/// @ai:intent Extract all annotations from a source file
/// @ai:pre path exists and is a supported file type
/// @ai:effects fs:read
pub fn extract_file(path: &Path) -> Result<ParsedFile> {
    let parsed = parse_file(path)?;
    let (module, raw_annotations) = extract_from_parsed(&parsed, path);

    Ok(ParsedFile {
        path: path.to_path_buf(),
        language: parsed.language.name().to_string(),
        module,
        raw_annotations,
    })
}

/// @ai:intent Extract annotations from parsed source
/// @ai:effects pure
fn extract_from_parsed(parsed: &ParsedSource, path: &Path) -> (ModuleAnnotations, Vec<Annotation>) {
    let mut module = ModuleAnnotations {
        file: path.to_path_buf(),
        ..Default::default()
    };
    let mut raw_annotations = Vec::new();

    // Extract module-level annotations from the first comment block
    if let Some(first_block) = parsed.comment_blocks.first() {
        if first_block.has_ai_annotations() {
            extract_module_annotations(first_block, path, &mut module, &mut raw_annotations);
        }
    }

    // Extract function-level annotations
    for func_loc in &parsed.function_locations {
        let mut func_annot = FunctionAnnotations::new(
            func_loc.name.clone(),
            Location::new(path.to_path_buf(), func_loc.line),
        );

        if let Some(block_idx) = func_loc.preceding_comment_block {
            if let Some(block) = parsed.comment_blocks.get(block_idx) {
                extract_function_annotations(block, path, &mut func_annot, &mut raw_annotations);
            }
        }

        module.functions.push(func_annot);
    }

    (module, raw_annotations)
}

/// @ai:intent Extract module-level annotations from a comment block
/// @ai:effects pure
fn extract_module_annotations(
    block: &CommentBlock,
    path: &Path,
    module: &mut ModuleAnnotations,
    raw: &mut Vec<Annotation>,
) {
    let re = Regex::new(r"@ai:module:(\w+)\s+(.*)").expect("Invalid regex");

    for line in &block.lines {
        if let Some(captures) = re.captures(&line.content) {
            let tag = captures.get(1).unwrap().as_str();
            let value = captures.get(2).unwrap().as_str().trim();

            let annotation = Annotation {
                level: AnnotationLevel::Module,
                tag: format!("module:{}", tag),
                value: value.to_string(),
                location: Location::new(path.to_path_buf(), line.line_number),
            };
            raw.push(annotation);

            apply_module_annotation(module, tag, value);
        }
    }
}

/// @ai:intent Apply a parsed annotation to the module struct
/// @ai:effects pure
fn apply_module_annotation(module: &mut ModuleAnnotations, tag: &str, value: &str) {
    match tag {
        "intent" => module.intent = Some(value.to_string()),
        "layer" => module.layer = Some(value.to_string()),
        "public_api" => {
            module.public_api = value.split(',').map(|s| s.trim().to_string()).collect();
        }
        "depends_on" => {
            module.depends_on = value.split(',').map(|s| s.trim().to_string()).collect();
        }
        "depended_by" => {
            module.depended_by = value.split(',').map(|s| s.trim().to_string()).collect();
        }
        "internal" => module.internal = Some(value == "true"),
        "stateless" => module.stateless = Some(value == "true"),
        "thread_safe" => module.thread_safe = Some(value == "true"),
        "cohesion" => module.cohesion = Some(value.to_string()),
        "stability" => module.stability = Some(value.to_string()),
        _ => {}
    }
}

/// @ai:intent Extract function-level annotations from a comment block
/// @ai:effects pure
fn extract_function_annotations(
    block: &CommentBlock,
    path: &Path,
    func: &mut FunctionAnnotations,
    raw: &mut Vec<Annotation>,
) {
    let re_standard = Regex::new(r"@ai:(\w+)\s*(.*)").expect("Invalid regex");
    let re_override = Regex::new(r"@ai:override:(\w+)\s+(.*)").expect("Invalid regex");
    let re_test = Regex::new(r"@ai:test:(\w+)\s*(.*)").expect("Invalid regex");

    for line in &block.lines {
        // Check for override annotations first
        if let Some(captures) = re_override.captures(&line.content) {
            let constraint = captures.get(1).unwrap().as_str();
            let value = captures.get(2).unwrap().as_str().trim();

            raw.push(Annotation {
                level: AnnotationLevel::Function,
                tag: format!("override:{}", constraint),
                value: value.to_string(),
                location: Location::new(path.to_path_buf(), line.line_number),
            });

            func.overrides.push((constraint.to_string(), value.to_string()));
            continue;
        }

        // Check for test annotations
        if let Some(captures) = re_test.captures(&line.content) {
            let test_type = captures.get(1).unwrap().as_str();
            let value = captures.get(2).unwrap().as_str().trim();

            raw.push(Annotation {
                level: AnnotationLevel::Test,
                tag: format!("test:{}", test_type),
                value: value.to_string(),
                location: Location::new(path.to_path_buf(), line.line_number),
            });

            if test_type == "integration" {
                func.test_integration = Some(value.to_string());
            }
            continue;
        }

        // Check for standard annotations
        if let Some(captures) = re_standard.captures(&line.content) {
            let tag = captures.get(1).unwrap().as_str();

            // Skip if this is a module annotation
            if tag.starts_with("module:") {
                continue;
            }

            let value = captures.get(2).unwrap().as_str().trim();

            raw.push(Annotation {
                level: AnnotationLevel::Function,
                tag: tag.to_string(),
                value: value.to_string(),
                location: Location::new(path.to_path_buf(), line.line_number),
            });

            apply_function_annotation(func, tag, value);
        }
    }
}

/// @ai:intent Apply a parsed annotation to the function struct
/// @ai:effects pure
fn apply_function_annotation(func: &mut FunctionAnnotations, tag: &str, value: &str) {
    match tag {
        "intent" => func.intent = Some(value.to_string()),
        "pre" => func.pre.push(value.to_string()),
        "post" => func.post.push(value.to_string()),
        "invariant" => func.invariant = Some(value.to_string()),
        "example" => func.examples.push(value.to_string()),
        "effects" => {
            func.effects = value.split(',').map(|s| s.trim().to_string()).collect();
        }
        "idempotent" => func.idempotent = Some(value == "true"),
        "confidence" => {
            if let Ok(conf) = value.parse::<f32>() {
                func.confidence = Some(conf);
            }
        }
        "needs_review" => func.needs_review = Some(value.to_string()),
        "author" => func.author = Some(value.to_string()),
        "verified" => func.verified = Some(value.to_string()),
        "assumes" => func.assumes = Some(value.to_string()),
        "context" => func.context = Some(value.to_string()),
        "related" => {
            func.related = value.split(',').map(|s| s.trim().to_string()).collect();
        }
        "deprecated" => func.deprecated = Some(value.to_string()),
        "complexity" => func.complexity = Some(value.to_string()),
        "edge_cases" => func.edge_cases.push(value.to_string()),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_extract_rust_file() {
        let mut file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(
            file,
            r#"//! @ai:module:intent Test module
//! @ai:module:layer domain

/// @ai:intent Test function
/// @ai:pre x > 0
/// @ai:effects pure
fn test_func(x: i32) -> i32 {{
    x
}}"#
        )
        .unwrap();

        let result = extract_file(file.path()).unwrap();

        assert_eq!(result.module.intent, Some("Test module".to_string()));
        assert_eq!(result.module.layer, Some("domain".to_string()));
        assert_eq!(result.module.functions.len(), 1);

        let func = &result.module.functions[0];
        assert_eq!(func.name, "test_func");
        assert_eq!(func.intent, Some("Test function".to_string()));
        assert_eq!(func.pre, vec!["x > 0".to_string()]);
        assert_eq!(func.effects, vec!["pure".to_string()]);
    }
}
