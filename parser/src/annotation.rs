//! @ai:module:intent Define data structures for AICMS annotations
//! @ai:module:layer domain
//! @ai:module:public_api Annotation, AnnotationType, FunctionAnnotations, ModuleAnnotations, Location
//! @ai:module:stateless true

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// @ai:intent Represents a source code location
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Location {
    pub file: PathBuf,
    pub line: usize,
    pub column: Option<usize>,
}

impl Default for Location {
    fn default() -> Self {
        Self {
            file: PathBuf::new(),
            line: 0,
            column: None,
        }
    }
}

/// @ai:intent Categorizes annotation types by level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationLevel {
    Project,
    Module,
    Function,
    Test,
}

/// @ai:intent Represents a single parsed annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub level: AnnotationLevel,
    pub tag: String,
    pub value: String,
    pub location: Location,
}

/// @ai:intent Collection of annotations for a function
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FunctionAnnotations {
    pub name: String,
    pub location: Location,
    pub intent: Option<String>,
    pub pre: Vec<String>,
    pub post: Vec<String>,
    pub invariant: Option<String>,
    pub examples: Vec<String>,
    pub effects: Vec<String>,
    pub idempotent: Option<bool>,
    pub confidence: Option<f32>,
    pub needs_review: Option<String>,
    pub author: Option<String>,
    pub verified: Option<String>,
    pub assumes: Option<String>,
    pub context: Option<String>,
    pub related: Vec<String>,
    pub deprecated: Option<String>,
    pub complexity: Option<String>,
    pub edge_cases: Vec<String>,
    pub overrides: Vec<(String, String)>,
    pub test_integration: Option<String>,
}

/// @ai:intent Collection of annotations for a module/file
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModuleAnnotations {
    pub file: PathBuf,
    pub intent: Option<String>,
    pub layer: Option<String>,
    pub public_api: Vec<String>,
    pub depends_on: Vec<String>,
    pub depended_by: Vec<String>,
    pub internal: Option<bool>,
    pub stateless: Option<bool>,
    pub thread_safe: Option<bool>,
    pub cohesion: Option<String>,
    pub stability: Option<String>,
    pub functions: Vec<FunctionAnnotations>,
}

/// @ai:intent Complete parsed result for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedFile {
    pub path: PathBuf,
    pub language: String,
    pub module: ModuleAnnotations,
    pub raw_annotations: Vec<Annotation>,
}

/// @ai:intent Complete parsed result for a project
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParsedProject {
    pub files: Vec<ParsedFile>,
    pub total_functions: usize,
    pub annotated_functions: usize,
    pub functions_missing_intent: Vec<Location>,
}

impl FunctionAnnotations {
    /// @ai:intent Create a new FunctionAnnotations with just the name and location
    pub fn new(name: String, location: Location) -> Self {
        Self {
            name,
            location,
            ..Default::default()
        }
    }

    /// @ai:intent Check if the function has the required @ai:intent annotation
    pub fn has_intent(&self) -> bool {
        self.intent.is_some()
    }

    /// @ai:intent Check if the function has any annotations at all
    pub fn is_annotated(&self) -> bool {
        self.intent.is_some()
            || !self.pre.is_empty()
            || !self.post.is_empty()
            || !self.examples.is_empty()
            || !self.effects.is_empty()
    }
}

impl Location {
    /// @ai:intent Create a new Location
    pub fn new(file: PathBuf, line: usize) -> Self {
        Self {
            file,
            line,
            column: None,
        }
    }
}
