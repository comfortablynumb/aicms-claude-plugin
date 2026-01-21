//! @ai:module:intent Execute tests against generated code
//! @ai:module:layer infrastructure
//! @ai:module:public_api TestRunner, TestResult
//! @ai:module:stateless true

use crate::corpus::Language;
use crate::evaluator::SourceFile;
use anyhow::Result;
use std::io::Write;
use std::process::Command;
use tempfile::TempDir;

/// @ai:intent Result of running tests
#[derive(Debug, Clone)]
pub struct TestResult {
    pub passed: u32,
    pub failed: u32,
    pub total: u32,
    pub output: String,
}

impl TestResult {
    /// @ai:intent Calculate pass rate as percentage
    /// @ai:effects pure
    pub fn pass_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.passed as f64 / self.total as f64) * 100.0
        }
    }
}

/// @ai:intent Trait for test execution
pub trait TestRunnerTrait: Send + Sync {
    /// @ai:intent Run tests against single-file generated code
    fn run(&self, code: &str, test_code: &str, language: Language) -> Result<TestResult>;

    /// @ai:intent Run tests against multi-file generated code
    fn run_files(
        &self,
        source_files: &[SourceFile],
        test_files: &[SourceFile],
        language: Language,
    ) -> Result<TestResult>;
}

/// @ai:intent Executes tests for generated code
pub struct TestRunner;

impl TestRunner {
    /// @ai:intent Create a new test runner
    /// @ai:effects pure
    pub fn new() -> Self {
        Self
    }

    /// @ai:intent Run Rust tests
    /// @ai:effects fs:write, io
    fn run_rust(&self, code: &str, test_code: &str) -> Result<TestResult> {
        let temp_dir = TempDir::new()?;
        let src_path = temp_dir.path().join("main.rs");

        let combined = format!("{code}\n\n#[cfg(test)]\nmod tests {{\n    use super::*;\n{test_code}\n}}");

        let mut file = std::fs::File::create(&src_path)?;
        file.write_all(combined.as_bytes())?;
        drop(file);

        let output = Command::new("rustc")
            .arg("--test")
            .arg("--edition=2021")
            .arg("-o")
            .arg(temp_dir.path().join("test_bin"))
            .arg(&src_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Ok(TestResult {
                passed: 0,
                failed: 1,
                total: 1,
                output: format!("Compilation failed: {}", stderr),
            });
        }

        let test_output = Command::new(temp_dir.path().join("test_bin")).output()?;

        let stdout = String::from_utf8_lossy(&test_output.stdout);
        parse_rust_test_output(&stdout)
    }

    /// @ai:intent Run Python tests
    /// @ai:effects fs:write, io
    fn run_python(&self, code: &str, test_code: &str) -> Result<TestResult> {
        let temp_dir = TempDir::new()?;
        let src_path = temp_dir.path().join("test_main.py");

        let combined = format!("{code}\n\nimport unittest\n\nclass TestGenerated(unittest.TestCase):\n{test_code}\n\nif __name__ == '__main__':\n    unittest.main(verbosity=2)");

        let mut file = std::fs::File::create(&src_path)?;
        file.write_all(combined.as_bytes())?;
        drop(file);

        let output = Command::new("python").arg(&src_path).output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        parse_python_test_output(&stderr)
    }

    /// @ai:intent Run TypeScript tests (using ts-node and basic assertions)
    /// @ai:effects fs:write, io
    fn run_typescript(&self, code: &str, test_code: &str) -> Result<TestResult> {
        let temp_dir = TempDir::new()?;
        let src_path = temp_dir.path().join("test.ts");

        let combined = format!("{code}\n\n{test_code}");

        let mut file = std::fs::File::create(&src_path)?;
        file.write_all(combined.as_bytes())?;
        drop(file);

        let output = Command::new("npx")
            .arg("ts-node")
            .arg(&src_path)
            .output()?;

        let success = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        Ok(TestResult {
            passed: if success { 1 } else { 0 },
            failed: if success { 0 } else { 1 },
            total: 1,
            output: format!("{}{}", stdout, stderr),
        })
    }

    /// @ai:intent Run multi-file Rust tests using Cargo
    /// @ai:effects fs:write, io
    fn run_rust_files(
        &self,
        source_files: &[SourceFile],
        test_files: &[SourceFile],
    ) -> Result<TestResult> {
        let temp_dir = TempDir::new()?;

        // Check if source files include a Cargo.toml
        let has_cargo_toml = source_files.iter().any(|f| {
            f.path == "Cargo.toml"
                || f.path.ends_with("/Cargo.toml")
                || f.path.ends_with("\\Cargo.toml")
        });

        // Create src directory and write source files
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir)?;

        for source_file in source_files {
            // Cargo.toml goes at root, source files under src/
            let file_path = if source_file.path == "Cargo.toml"
                || source_file.path.ends_with("/Cargo.toml")
                || source_file.path.ends_with("\\Cargo.toml")
            {
                temp_dir.path().join("Cargo.toml")
            } else {
                temp_dir.path().join(&source_file.path)
            };

            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&file_path, &source_file.content)?;
        }

        // Only create minimal Cargo.toml if none was provided
        if !has_cargo_toml {
            let cargo_toml = r#"[package]
name = "benchmark_project"
version = "0.1.0"
edition = "2021"

[dependencies]

[dev-dependencies]
"#;
            std::fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml)?;
        }

        // Create tests directory and write test files
        let tests_dir = temp_dir.path().join("tests");
        std::fs::create_dir_all(&tests_dir)?;

        for test_file in test_files {
            let file_path = tests_dir.join(&test_file.path);

            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&file_path, &test_file.content)?;
        }

        // Run cargo test
        let output = Command::new("cargo")
            .arg("test")
            .arg("--")
            .arg("--test-threads=1")
            .current_dir(temp_dir.path())
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        parse_rust_test_output(&stdout)
    }

    /// @ai:intent Run multi-file Python tests using pytest
    /// @ai:effects fs:write, io
    fn run_python_files(
        &self,
        source_files: &[SourceFile],
        test_files: &[SourceFile],
    ) -> Result<TestResult> {
        let temp_dir = TempDir::new()?;

        // Write source files
        for source_file in source_files {
            let file_path = temp_dir.path().join(&source_file.path);

            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
                let init_path = parent.join("__init__.py");

                if !init_path.exists() {
                    std::fs::write(&init_path, "")?;
                }
            }

            std::fs::write(&file_path, &source_file.content)?;
        }

        // Write test files
        for test_file in test_files {
            let file_path = temp_dir.path().join(&test_file.path);

            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&file_path, &test_file.content)?;
        }

        // Run pytest
        let output = Command::new("python")
            .arg("-m")
            .arg("pytest")
            .arg("-v")
            .current_dir(temp_dir.path())
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        parse_pytest_output(&stdout, &stderr)
    }

    /// @ai:intent Run multi-file TypeScript tests
    /// @ai:effects fs:write, io
    fn run_typescript_files(
        &self,
        source_files: &[SourceFile],
        test_files: &[SourceFile],
    ) -> Result<TestResult> {
        let temp_dir = TempDir::new()?;

        // Write all files
        for source_file in source_files {
            let file_path = temp_dir.path().join(&source_file.path);

            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&file_path, &source_file.content)?;
        }

        for test_file in test_files {
            let file_path = temp_dir.path().join(&test_file.path);

            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&file_path, &test_file.content)?;
        }

        // Create tsconfig.json
        let tsconfig = r#"{
  "compilerOptions": {
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "outDir": "./dist"
  },
  "include": ["**/*.ts"]
}"#;
        std::fs::write(temp_dir.path().join("tsconfig.json"), tsconfig)?;

        // Run test file with ts-node (assuming first test file is the entry)
        let test_entry = test_files
            .first()
            .map(|f| f.path.clone())
            .unwrap_or_else(|| "test.ts".to_string());

        let output = Command::new("npx")
            .arg("ts-node")
            .arg(&test_entry)
            .current_dir(temp_dir.path())
            .output()?;

        let success = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        Ok(TestResult {
            passed: if success { 1 } else { 0 },
            failed: if success { 0 } else { 1 },
            total: 1,
            output: format!("{}{}", stdout, stderr),
        })
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// @ai:intent Parse Rust test output for pass/fail counts
/// @ai:effects pure
fn parse_rust_test_output(output: &str) -> Result<TestResult> {
    let mut passed = 0u32;
    let mut failed = 0u32;

    for line in output.lines() {
        if line.contains("test result:") {
            let parts: Vec<&str> = line.split_whitespace().collect();

            for (i, part) in parts.iter().enumerate() {
                if *part == "passed;" && i > 0 {
                    passed = parts[i - 1].parse().unwrap_or(0);
                }

                if *part == "failed;" && i > 0 {
                    failed = parts[i - 1].parse().unwrap_or(0);
                }
            }
        }
    }

    Ok(TestResult {
        passed,
        failed,
        total: passed + failed,
        output: output.to_string(),
    })
}

/// @ai:intent Parse Python unittest output for pass/fail counts
/// @ai:effects pure
fn parse_python_test_output(output: &str) -> Result<TestResult> {
    let mut passed = 0u32;
    let mut failed = 0u32;

    for line in output.lines() {
        if line.starts_with("Ran ") && line.contains("test") {
            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.len() >= 2 {
                let total: u32 = parts[1].parse().unwrap_or(0);

                if output.contains("OK") {
                    passed = total;
                } else if output.contains("FAILED") {
                    for l in output.lines() {
                        if l.contains("failures=") {
                            if let Some(num) = l
                                .split("failures=")
                                .nth(1)
                                .and_then(|s| s.split(')').next())
                            {
                                failed = num.parse().unwrap_or(0);
                            }
                        }
                    }
                    passed = total.saturating_sub(failed);
                }
            }
        }
    }

    Ok(TestResult {
        passed,
        failed,
        total: passed + failed,
        output: output.to_string(),
    })
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

/// @ai:intent Parse pytest output for pass/fail counts
/// @ai:effects pure
fn parse_pytest_output(stdout: &str, stderr: &str) -> Result<TestResult> {
    let combined = format!("{stdout}\n{stderr}");
    let mut passed = 0u32;
    let mut failed = 0u32;

    // Look for pytest summary line like "5 passed, 2 failed"
    for line in combined.lines() {
        if line.contains("passed") || line.contains("failed") {
            // Parse "X passed"
            if let Some(pos) = line.find("passed") {
                let before = &line[..pos];

                if let Some(num_str) = before.split_whitespace().last() {
                    passed = num_str.parse().unwrap_or(0);
                }
            }

            // Parse "X failed"
            if let Some(pos) = line.find("failed") {
                let before = &line[..pos];

                if let Some(num_str) = before.split(',').last() {
                    if let Some(n) = num_str.split_whitespace().last() {
                        failed = n.parse().unwrap_or(0);
                    }
                }
            }
        }
    }

    Ok(TestResult {
        passed,
        failed,
        total: passed + failed,
        output: combined,
    })
}

impl TestRunner {
    /// @ai:intent Run Claude's own tests embedded in the generated code
    /// @ai:effects fs:write, io
    pub fn run_own_tests(&self, source_files: &[SourceFile], language: Language) -> Result<TestResult> {
        match language {
            Language::Rust => self.run_rust_own_tests(source_files),
            Language::Python => self.run_python_own_tests(source_files),
            Language::TypeScript => self.run_typescript_own_tests(source_files),
        }
    }

    /// @ai:intent Run Rust's built-in tests with coverage
    /// @ai:effects fs:write, io
    fn run_rust_own_tests(&self, source_files: &[SourceFile]) -> Result<TestResult> {
        let temp_dir = TempDir::new()?;

        // Check if source files include a Cargo.toml
        let has_cargo_toml = source_files.iter().any(|f| {
            f.path == "Cargo.toml"
                || f.path.ends_with("/Cargo.toml")
                || f.path.ends_with("\\Cargo.toml")
        });

        // Write source files first
        for source_file in source_files {
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

            tracing::debug!(
                "Writing test file: {} -> {}",
                source_file.path,
                file_path.display()
            );
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

        // Run cargo test
        let output = Command::new("cargo")
            .arg("test")
            .arg("--")
            .arg("--test-threads=1")
            .current_dir(temp_dir.path())
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        tracing::debug!("Cargo test stdout: {}", stdout);
        tracing::debug!("Cargo test stderr: {}", stderr);

        if !output.status.success() && stdout.is_empty() {
            return Ok(TestResult {
                passed: 0,
                failed: 1,
                total: 1,
                output: format!("Build/test failed: {}", stderr),
            });
        }

        parse_rust_test_output(&stdout)
    }

    /// @ai:intent Run Python's pytest on generated code
    /// @ai:effects fs:write, io
    fn run_python_own_tests(&self, source_files: &[SourceFile]) -> Result<TestResult> {
        let temp_dir = TempDir::new()?;

        // Write source files
        for source_file in source_files {
            let file_path = temp_dir.path().join(&source_file.path);

            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;

                // Create __init__.py for packages
                let init_path = parent.join("__init__.py");
                if !init_path.exists() {
                    std::fs::write(&init_path, "")?;
                }
            }

            std::fs::write(&file_path, &source_file.content)?;
        }

        // Run pytest
        let output = Command::new("python")
            .arg("-m")
            .arg("pytest")
            .arg("-v")
            .current_dir(temp_dir.path())
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        parse_pytest_output(&stdout, &stderr)
    }

    /// @ai:intent Run TypeScript tests
    /// @ai:effects fs:write, io
    fn run_typescript_own_tests(&self, source_files: &[SourceFile]) -> Result<TestResult> {
        let temp_dir = TempDir::new()?;

        // Write source files
        for source_file in source_files {
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
    "esModuleInterop": true,
    "skipLibCheck": true,
    "outDir": "./dist"
  },
  "include": ["**/*.ts"]
}"#;
        std::fs::write(temp_dir.path().join("tsconfig.json"), tsconfig)?;

        // Try to find and run test file
        let test_file = source_files
            .iter()
            .find(|f| f.path.contains("test"))
            .or_else(|| source_files.first());

        if let Some(test) = test_file {
            let output = Command::new("npx")
                .arg("ts-node")
                .arg(&test.path)
                .current_dir(temp_dir.path())
                .output()?;

            let success = output.status.success();
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            Ok(TestResult {
                passed: if success { 1 } else { 0 },
                failed: if success { 0 } else { 1 },
                total: 1,
                output: format!("{}{}", stdout, stderr),
            })
        } else {
            Ok(TestResult {
                passed: 0,
                failed: 0,
                total: 0,
                output: "No test files found".to_string(),
            })
        }
    }
}

impl TestRunnerTrait for TestRunner {
    /// @ai:intent Run tests against generated code
    /// @ai:effects fs:write, io
    fn run(&self, code: &str, test_code: &str, language: Language) -> Result<TestResult> {
        match language {
            Language::Rust => self.run_rust(code, test_code),
            Language::Python => self.run_python(code, test_code),
            Language::TypeScript => self.run_typescript(code, test_code),
        }
    }

    /// @ai:intent Run tests against multi-file generated code
    /// @ai:effects fs:write, io
    fn run_files(
        &self,
        source_files: &[SourceFile],
        test_files: &[SourceFile],
        language: Language,
    ) -> Result<TestResult> {
        // If single files, fall back to simple run
        if source_files.len() == 1 && test_files.len() == 1 {
            return self.run(&source_files[0].content, &test_files[0].content, language);
        }

        match language {
            Language::Rust => self.run_rust_files(source_files, test_files),
            Language::Python => self.run_python_files(source_files, test_files),
            Language::TypeScript => self.run_typescript_files(source_files, test_files),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rust_output() {
        let output = "test result: ok. 3 passed; 1 failed; 0 ignored";
        let result = parse_rust_test_output(output).unwrap();
        assert_eq!(result.passed, 3);
        assert_eq!(result.failed, 1);
    }

    #[test]
    fn test_pass_rate_calculation() {
        let result = TestResult {
            passed: 7,
            failed: 3,
            total: 10,
            output: String::new(),
        };
        assert!((result.pass_rate() - 70.0).abs() < 0.01);
    }
}
