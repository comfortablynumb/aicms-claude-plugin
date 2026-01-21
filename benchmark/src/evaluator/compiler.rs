//! @ai:module:intent Compilation checking for generated code
//! @ai:module:layer infrastructure
//! @ai:module:public_api CompilationChecker, CompilationResult
//! @ai:module:stateless true

use crate::corpus::Language;
use crate::evaluator::SourceFile;
use anyhow::Result;
use std::io::Write;
use std::process::Command;
use tempfile::TempDir;

/// @ai:intent Result of compilation check
#[derive(Debug, Clone)]
pub struct CompilationResult {
    pub success: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// @ai:intent Trait for compilation checking
#[allow(async_fn_in_trait)]
pub trait CompilationCheckerTrait: Send + Sync {
    /// @ai:intent Check if single-file code compiles
    fn check(&self, code: &str, language: Language) -> Result<CompilationResult>;

    /// @ai:intent Check if multi-file project compiles
    fn check_files(&self, files: &[SourceFile], language: Language) -> Result<CompilationResult>;

    /// @ai:intent Check if code in a directory compiles
    fn check_directory(&self, dir: &std::path::Path) -> Result<CompilationResult>;
}

/// @ai:intent Checks if generated code compiles
pub struct CompilationChecker;

impl CompilationChecker {
    /// @ai:intent Create a new compilation checker
    /// @ai:effects pure
    pub fn new() -> Self {
        Self
    }

    /// @ai:intent Check Rust code compilation
    /// @ai:effects fs:write, io
    fn check_rust(&self, code: &str) -> Result<CompilationResult> {
        let temp_dir = TempDir::new()?;
        let src_path = temp_dir.path().join("main.rs");

        let mut file = std::fs::File::create(&src_path)?;
        file.write_all(code.as_bytes())?;
        drop(file);

        let output = Command::new("rustc")
            .arg("--emit=metadata")
            .arg("--edition=2021")
            .arg("-o")
            .arg(temp_dir.path().join("out"))
            .arg(&src_path)
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let errors = extract_rust_messages(&stderr, "error");
        let warnings = extract_rust_messages(&stderr, "warning");

        Ok(CompilationResult {
            success: output.status.success(),
            errors,
            warnings,
        })
    }

    /// @ai:intent Check Python code compilation
    /// @ai:effects fs:write, io
    fn check_python(&self, code: &str) -> Result<CompilationResult> {
        let temp_dir = TempDir::new()?;
        let src_path = temp_dir.path().join("main.py");

        let mut file = std::fs::File::create(&src_path)?;
        file.write_all(code.as_bytes())?;
        drop(file);

        let output = Command::new("python")
            .arg("-m")
            .arg("py_compile")
            .arg(&src_path)
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let errors = if stderr.is_empty() {
            vec![]
        } else {
            vec![stderr.to_string()]
        };

        Ok(CompilationResult {
            success: output.status.success(),
            errors,
            warnings: vec![],
        })
    }

    /// @ai:intent Check TypeScript code compilation
    /// @ai:effects fs:write, io
    fn check_typescript(&self, code: &str) -> Result<CompilationResult> {
        let temp_dir = TempDir::new()?;
        let src_path = temp_dir.path().join("main.ts");

        let mut file = std::fs::File::create(&src_path)?;
        file.write_all(code.as_bytes())?;
        drop(file);

        let output = Command::new("tsc")
            .arg("--noEmit")
            .arg("--strict")
            .arg(&src_path)
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = format!("{}{}", stdout, stderr);

        let errors: Vec<String> = combined
            .lines()
            .filter(|l| l.contains("error"))
            .map(|l| l.to_string())
            .collect();

        Ok(CompilationResult {
            success: output.status.success(),
            errors,
            warnings: vec![],
        })
    }

    /// @ai:intent Check multi-file Rust project compilation using Cargo
    /// @ai:effects fs:write, io
    fn check_rust_files(&self, files: &[SourceFile]) -> Result<CompilationResult> {
        let temp_dir = TempDir::new()?;

        // Check if source files include a Cargo.toml
        let has_cargo_toml = files.iter().any(|f| {
            f.path == "Cargo.toml" || f.path.ends_with("/Cargo.toml") || f.path.ends_with("\\Cargo.toml")
        });

        // Write all source files first
        for source_file in files {
            // Cargo.toml goes at root, source files under src/
            let file_path = if source_file.path == "Cargo.toml"
                || source_file.path.ends_with("/Cargo.toml")
                || source_file.path.ends_with("\\Cargo.toml")
            {
                temp_dir.path().join("Cargo.toml")
            } else {
                let normalized_path = normalize_rust_path(&source_file.path);
                temp_dir.path().join(&normalized_path)
            };

            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            tracing::debug!("Writing file: {} -> {}", source_file.path, file_path.display());
            std::fs::write(&file_path, &source_file.content)?;
        }

        // Only create minimal Cargo.toml if none was provided
        if !has_cargo_toml {
            let cargo_toml = r#"[package]
name = "benchmark_project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
            std::fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml)?;
        }

        // Ensure src directory exists
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir)?;

        // Run cargo check
        let output = Command::new("cargo")
            .arg("check")
            .arg("--message-format=short")
            .current_dir(temp_dir.path())
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        tracing::debug!("Cargo check stdout: {}", stdout);
        tracing::debug!("Cargo check stderr: {}", stderr);

        let errors = extract_rust_messages(&stderr, "error");
        let warnings = extract_rust_messages(&stderr, "warning");

        Ok(CompilationResult {
            success: output.status.success(),
            errors,
            warnings,
        })
    }

    /// @ai:intent Check multi-file Python project compilation
    /// @ai:effects fs:write, io
    fn check_python_files(&self, files: &[SourceFile]) -> Result<CompilationResult> {
        let temp_dir = TempDir::new()?;
        let mut all_errors = Vec::new();

        // Write all source files
        for source_file in files {
            let file_path = temp_dir.path().join(&source_file.path);

            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
                // Create __init__.py for Python packages
                let init_path = parent.join("__init__.py");

                if !init_path.exists() {
                    std::fs::write(&init_path, "")?;
                }
            }

            std::fs::write(&file_path, &source_file.content)?;
        }

        // Check each Python file
        for source_file in files {
            let file_path = temp_dir.path().join(&source_file.path);

            let output = Command::new("python")
                .arg("-m")
                .arg("py_compile")
                .arg(&file_path)
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                all_errors.push(format!("{}: {}", source_file.path, stderr));
            }
        }

        Ok(CompilationResult {
            success: all_errors.is_empty(),
            errors: all_errors,
            warnings: vec![],
        })
    }

    /// @ai:intent Check multi-file TypeScript project compilation
    /// @ai:effects fs:write, io
    fn check_typescript_files(&self, files: &[SourceFile]) -> Result<CompilationResult> {
        let temp_dir = TempDir::new()?;

        // Write all source files
        for source_file in files {
            let file_path = temp_dir.path().join(&source_file.path);

            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&file_path, &source_file.content)?;
        }

        // Create tsconfig.json
        let tsconfig = r#"{
  "compilerOptions": {
    "strict": true,
    "noEmit": true,
    "esModuleInterop": true,
    "skipLibCheck": true
  },
  "include": ["**/*.ts"]
}"#;
        std::fs::write(temp_dir.path().join("tsconfig.json"), tsconfig)?;

        let output = Command::new("tsc")
            .arg("--noEmit")
            .current_dir(temp_dir.path())
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = format!("{}{}", stdout, stderr);

        let errors: Vec<String> = combined
            .lines()
            .filter(|l| l.contains("error"))
            .map(|l| l.to_string())
            .collect();

        Ok(CompilationResult {
            success: output.status.success(),
            errors,
            warnings: vec![],
        })
    }

    /// @ai:intent Check Rust code compilation in an existing directory
    /// @ai:effects io
    fn check_rust_directory(&self, dir: &std::path::Path) -> Result<CompilationResult> {
        // Run cargo check in the directory
        let output = Command::new("cargo")
            .arg("check")
            .arg("--message-format=short")
            .current_dir(dir)
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let errors = extract_rust_messages(&stderr, "error");
        let warnings = extract_rust_messages(&stderr, "warning");

        Ok(CompilationResult {
            success: output.status.success(),
            errors,
            warnings,
        })
    }

    /// @ai:intent Check Python code compilation in an existing directory
    /// @ai:effects io
    fn check_python_directory(&self, dir: &std::path::Path) -> Result<CompilationResult> {
        let mut all_errors = Vec::new();

        // Find and check all Python files
        check_python_files_recursive(dir, dir, &mut all_errors)?;

        Ok(CompilationResult {
            success: all_errors.is_empty(),
            errors: all_errors,
            warnings: vec![],
        })
    }

    /// @ai:intent Check TypeScript code compilation in an existing directory
    /// @ai:effects io
    fn check_typescript_directory(&self, dir: &std::path::Path) -> Result<CompilationResult> {
        let output = Command::new("tsc")
            .arg("--noEmit")
            .arg("--strict")
            .current_dir(dir)
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = format!("{}{}", stdout, stderr);

        let errors: Vec<String> = combined
            .lines()
            .filter(|l| l.contains("error"))
            .map(|l| l.to_string())
            .collect();

        Ok(CompilationResult {
            success: output.status.success(),
            errors,
            warnings: vec![],
        })
    }
}

/// @ai:intent Recursively check Python files in a directory
/// @ai:effects io
fn check_python_files_recursive(
    base: &std::path::Path,
    current: &std::path::Path,
    errors: &mut Vec<String>,
) -> Result<()> {
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();

            if !name.starts_with('.') && name != "__pycache__" && name != "venv" {
                check_python_files_recursive(base, &path, errors)?;
            }
        } else if path.extension().map_or(false, |e| e == "py") {
            let output = Command::new("python")
                .arg("-m")
                .arg("py_compile")
                .arg(&path)
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let relative = path.strip_prefix(base).unwrap_or(&path);
                errors.push(format!("{}: {}", relative.display(), stderr.trim()));
            }
        }
    }

    Ok(())
}

impl Default for CompilationChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// @ai:intent Detect language from files in a directory
/// @ai:effects fs:read
fn detect_language_from_directory(dir: &std::path::Path) -> Option<Language> {
    // Check for Cargo.toml (Rust)
    if dir.join("Cargo.toml").exists() {
        return Some(Language::Rust);
    }

    // Check for .rs files
    if has_files_with_extension(dir, "rs") {
        return Some(Language::Rust);
    }

    // Check for .py files
    if has_files_with_extension(dir, "py") {
        return Some(Language::Python);
    }

    // Check for .ts files
    if has_files_with_extension(dir, "ts") {
        return Some(Language::TypeScript);
    }

    None
}

/// @ai:intent Check if directory contains files with given extension
/// @ai:effects fs:read
fn has_files_with_extension(dir: &std::path::Path, ext: &str) -> bool {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() {
                if let Some(file_ext) = path.extension() {
                    if file_ext == ext {
                        return true;
                    }
                }
            } else if path.is_dir() {
                let name = path.file_name().unwrap_or_default().to_string_lossy();

                if !name.starts_with('.') && name != "target" && name != "__pycache__" {
                    if has_files_with_extension(&path, ext) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// @ai:intent Extract error or warning messages from Rust compiler output
/// @ai:effects pure
fn extract_rust_messages(output: &str, msg_type: &str) -> Vec<String> {
    output
        .lines()
        .filter(|line| line.contains(&format!("{msg_type}[")))
        .map(|line| line.to_string())
        .collect()
}

/// @ai:intent Normalize Rust file path to be under src/ directory
/// @ai:effects pure
fn normalize_rust_path(path: &str) -> String {
    // If path already starts with "src/", use as-is
    if path.starts_with("src/") || path.starts_with("src\\") {
        return path.to_string();
    }

    // Convert main.rs -> src/main.rs, lib.rs -> src/lib.rs, etc.
    format!("src/{}", path)
}

impl CompilationCheckerTrait for CompilationChecker {
    /// @ai:intent Check if code compiles for the given language
    /// @ai:effects fs:write, io
    fn check(&self, code: &str, language: Language) -> Result<CompilationResult> {
        match language {
            Language::Rust => self.check_rust(code),
            Language::Python => self.check_python(code),
            Language::TypeScript => self.check_typescript(code),
        }
    }

    /// @ai:intent Check if multi-file project compiles for the given language
    /// @ai:effects fs:write, io
    fn check_files(&self, files: &[SourceFile], language: Language) -> Result<CompilationResult> {
        if files.len() == 1 {
            return self.check(&files[0].content, language);
        }

        match language {
            Language::Rust => self.check_rust_files(files),
            Language::Python => self.check_python_files(files),
            Language::TypeScript => self.check_typescript_files(files),
        }
    }

    /// @ai:intent Check if code in an existing directory compiles
    /// @ai:effects io
    fn check_directory(&self, dir: &std::path::Path) -> Result<CompilationResult> {
        let language = detect_language_from_directory(dir).ok_or_else(|| {
            anyhow::anyhow!("Could not detect language in directory: {}", dir.display())
        })?;

        match language {
            Language::Rust => self.check_rust_directory(dir),
            Language::Python => self.check_python_directory(dir),
            Language::TypeScript => self.check_typescript_directory(dir),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_valid_code() {
        let checker = CompilationChecker::new();
        let code = "fn main() {}";

        let result = checker.check(code, Language::Rust).unwrap();
        assert!(result.success);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_rust_invalid_code() {
        let checker = CompilationChecker::new();
        let code = "fn main() { let x: i32 = \"not a number\"; }";

        let result = checker.check(code, Language::Rust).unwrap();
        assert!(!result.success);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_python_valid_code() {
        let checker = CompilationChecker::new();
        let code = "def main():\n    pass";

        let result = checker.check(code, Language::Python).unwrap();
        assert!(result.success);
    }
}
