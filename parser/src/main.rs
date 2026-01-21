//! @ai:module:intent CLI entry point for AICMS parser and linter
//! @ai:module:layer presentation
//! @ai:module:public_api main
//! @ai:module:depends_on linter, extractor, output

use aicms_parser::{
    diff, extractor, linter, output, LintConfig, OutputFormat,
};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "aicms")]
#[command(author, version, about = "AICMS - AI-First Code Metadata Specification tools")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lint source files for AICMS compliance
    Lint {
        /// Path to file or directory to lint
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Require @ai:intent on all functions
        #[arg(long, default_value = "true")]
        require_intent: bool,

        /// Require @ai:module:intent on all files
        #[arg(long, default_value = "false")]
        require_module_intent: bool,

        /// Warn on low confidence values
        #[arg(long, default_value = "true")]
        warn_low_confidence: bool,

        /// Confidence threshold for warnings
        #[arg(long, default_value = "0.7")]
        confidence_threshold: f32,

        /// Output format
        #[arg(long, short, value_enum, default_value = "text")]
        format: Format,
    },

    /// Extract annotations from source files
    Extract {
        /// Path to file or directory
        path: PathBuf,

        /// Output format
        #[arg(long, short, value_enum, default_value = "json-pretty")]
        format: Format,
    },

    /// Parse a file and show detected functions
    Parse {
        /// Path to file
        path: PathBuf,

        /// Output format
        #[arg(long, short, value_enum, default_value = "text")]
        format: Format,
    },

    /// Compare annotations between two file versions (semantic diff)
    Diff {
        /// Path to the old version of the file
        old_file: PathBuf,

        /// Path to the new version of the file
        new_file: PathBuf,

        /// Output format
        #[arg(long, short, value_enum, default_value = "text")]
        format: Format,

        /// Fail with exit code 1 if breaking changes are found
        #[arg(long, default_value = "false")]
        fail_on_breaking: bool,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Text,
    Json,
    JsonPretty,
}

impl From<Format> for OutputFormat {
    fn from(f: Format) -> Self {
        match f {
            Format::Text => OutputFormat::Text,
            Format::Json => OutputFormat::Json,
            Format::JsonPretty => OutputFormat::JsonPretty,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Lint {
            path,
            require_intent,
            require_module_intent,
            warn_low_confidence,
            confidence_threshold,
            format,
        } => {
            let config = LintConfig {
                require_intent,
                require_module_intent,
                require_effects_for_impure: false,
                warn_low_confidence,
                confidence_threshold,
            };

            let result = if path.is_file() {
                linter::lint_file(&path, &config)
            } else {
                linter::lint_directory(&path, &config)
            };

            match result {
                Ok(lint_result) => {
                    println!("{}", output::format_lint_result(&lint_result, format.into()));

                    if lint_result.passed() {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(1)
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    ExitCode::from(2)
                }
            }
        }

        Commands::Extract { path, format } => {
            if path.is_file() {
                match extractor::extract_file(&path) {
                    Ok(parsed) => {
                        println!("{}", output::format_parsed_file(&parsed, format.into()));
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ExitCode::from(2)
                    }
                }
            } else {
                eprintln!("Error: extract command requires a file path");
                ExitCode::from(2)
            }
        }

        Commands::Parse { path, format } => {
            match extractor::extract_file(&path) {
                Ok(parsed) => {
                    println!("{}", output::format_parsed_file(&parsed, format.into()));
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    ExitCode::from(2)
                }
            }
        }

        Commands::Diff {
            old_file,
            new_file,
            format,
            fail_on_breaking,
        } => {
            match diff::diff_files(&old_file, &new_file) {
                Ok(diff_result) => {
                    println!("{}", output::format_diff_result(&diff_result, format.into()));

                    if fail_on_breaking && diff_result.has_breaking_changes() {
                        ExitCode::from(1)
                    } else {
                        ExitCode::SUCCESS
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    ExitCode::from(2)
                }
            }
        }
    }
}
