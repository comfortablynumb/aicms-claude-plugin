//! @ai:module:intent Define error types for the AICMS parser
//! @ai:module:layer domain
//! @ai:module:public_api Error, Result
//! @ai:module:stateless true

use std::path::PathBuf;
use thiserror::Error;

/// @ai:intent Unified error type for all AICMS parser operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to read file {path}: {source}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),

    #[error("Parse error at {file}:{line}: {message}")]
    Parse {
        file: PathBuf,
        line: usize,
        message: String,
    },

    #[error("Invalid annotation format: {0}")]
    InvalidAnnotation(String),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
