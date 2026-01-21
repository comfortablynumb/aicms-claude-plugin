//! @ai:module:intent CLI for AICMS benchmark system
//! @ai:module:layer presentation

use aicms_bench::{
    config::{BenchmarkConfig, FilterConfig, PathConfig},
    corpus::{CorpusLoader, CorpusLoaderTrait},
    evaluator::Evaluator,
    metrics::{MetricsAggregator, MetricsAggregatorTrait, TaskMetrics},
    report::ReportGenerator,
    runner::{create_executor, ClaudeClient, ClaudeCodeClient, MockClaudeClient},
    toolchain::ToolchainValidator,
};
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "aicms-bench")]
#[command(about = "AICMS benchmark system for measuring annotation effectiveness")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run benchmarks
    Run {
        /// Path to configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Filter by categories (comma-separated)
        #[arg(long)]
        categories: Option<String>,

        /// Filter by languages (comma-separated)
        #[arg(long)]
        languages: Option<String>,

        /// Filter by task IDs (comma-separated)
        #[arg(long)]
        tasks: Option<String>,

        /// Number of repetitions
        #[arg(short, long, default_value = "1")]
        repetitions: u32,

        /// Run without making API calls
        #[arg(long)]
        dry_run: bool,

        /// Use direct API instead of Claude Code CLI (requires ANTHROPIC_API_KEY)
        #[arg(long)]
        use_api: bool,

        /// Enable Claude-based comparison scoring (slower, uses Claude to score both implementations)
        #[arg(long)]
        compare: bool,

        /// Output directory for results
        #[arg(short, long, default_value = "results")]
        output: PathBuf,
    },

    /// Run comparison on existing results directory
    Compare {
        /// Path to results directory (e.g., results/2026-01-20_12-00-00)
        #[arg(short, long)]
        results_dir: PathBuf,

        /// Path to configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,
    },

    /// Generate reports from existing results
    Report {
        /// Path to results JSON file
        #[arg(short, long)]
        results: PathBuf,

        /// Output directory for reports
        #[arg(short, long, default_value = "reports")]
        output: PathBuf,
    },

    /// List available tasks
    List {
        /// Filter by category
        #[arg(long)]
        category: Option<String>,

        /// Filter by language
        #[arg(long)]
        language: Option<String>,
    },

    /// Validate corpus for errors
    Validate,

    /// Initialize default configuration
    Init {
        /// Output path for config file
        #[arg(short, long, default_value = "benchmark.toml")]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("aicms_bench=info".parse()?),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            config,
            categories,
            languages,
            tasks,
            repetitions,
            dry_run,
            use_api,
            compare,
            output,
        } => run_benchmarks(RunArgs {
            config,
            categories,
            languages,
            tasks,
            repetitions,
            dry_run,
            use_api,
            compare,
            output,
        })
        .await,
        Commands::Compare { results_dir, config } => run_comparison_only(results_dir, config),
        Commands::Report { results, output } => generate_reports(results, output),
        Commands::List { category, language } => list_tasks(category, language),
        Commands::Validate => validate(),
        Commands::Init { output } => init_config(output),
    }
}

struct RunArgs {
    config: Option<PathBuf>,
    categories: Option<String>,
    languages: Option<String>,
    tasks: Option<String>,
    repetitions: u32,
    dry_run: bool,
    use_api: bool,
    compare: bool,
    output: PathBuf,
}

/// @ai:intent Run benchmark suite
/// @ai:effects network, fs:write
async fn run_benchmarks(args: RunArgs) -> Result<()> {
    let mut config = load_or_default_config(args.config)?;

    config.run.repetitions = args.repetitions;
    config.run.dry_run = args.dry_run;
    config.run.filter = build_filter(args.categories, args.languages, args.tasks);

    let toolchain_status = ToolchainValidator::validate();
    ToolchainValidator::log_warnings(&toolchain_status);

    if toolchain_status.available_languages.is_empty() {
        tracing::error!("No language toolchains available. Cannot run benchmarks.");
        return Ok(());
    }

    tracing::info!("Loading corpus from {}", config.paths.corpus_dir.display());

    let loader = CorpusLoader::new();
    let all_tasks = loader.load_filtered(&config.paths.corpus_dir, &config.run.filter)?;

    let tasks: Vec<_> = all_tasks
        .into_iter()
        .filter(|task| toolchain_status.available_languages.contains(&task.language))
        .collect();

    if tasks.is_empty() {
        tracing::warn!("No tasks match the filter criteria (after excluding unavailable languages)");
        return Ok(());
    }

    tracing::info!("Found {} tasks to run", tasks.len());

    // Create output directory first so Claude runs inside it
    let timestamp = chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S");
    let output_dir = args.output.join(timestamp.to_string());
    std::fs::create_dir_all(&output_dir)?;
    tracing::info!("Output directory: {}", output_dir.display());

    let all_metrics = if config.run.dry_run {
        tracing::info!("Running in dry-run mode");
        let mock_client = Arc::new(MockClaudeClient::new(
            "Mock response with ```rust\nfn main() {}\n```".to_string(),
        ));
        let executor = create_executor(mock_client, &config)?;
        execute_tasks(&executor, &tasks).await?
    } else if args.use_api {
        tracing::info!("Using direct API (requires ANTHROPIC_API_KEY)");
        let client = Arc::new(ClaudeClient::new(config.api.clone())?);
        let executor = create_executor(client, &config)?;
        execute_tasks(&executor, &tasks).await?
    } else {
        tracing::info!("Using Claude Code CLI");
        let client = Arc::new(ClaudeCodeClient::new(output_dir.clone()));
        let executor = create_executor(client, &config)?;
        execute_tasks(&executor, &tasks).await?
    };

    let aggregator = MetricsAggregator::new();
    let mut results =
        aggregator.aggregate(&all_metrics.metrics, &tasks, &config.api.model, config.run.repetitions);

    // Load comparison prompt for saving with results
    let comparison_prompt = load_comparison_prompt(&config.paths.comparison_prompt_file)?;

    // Run Claude comparisons if enabled (only works with Claude Code CLI mode)
    if args.compare && !config.run.dry_run && !args.use_api {
        tracing::info!("Running Claude-based comparisons...");
        let comparisons = run_claude_comparisons(&config, &tasks, &output_dir)?;
        aggregator.add_claude_comparisons(&mut results, comparisons);
    } else if args.compare && args.use_api {
        tracing::warn!("Comparison not available with --use-api (no run directories)");
    }

    let reporter = ReportGenerator::new();
    reporter.generate_all(&results, &output_dir)?;

    // Save comparison prompt used
    reporter.save_comparison_prompt(&comparison_prompt, &output_dir)?;

    print_summary(&results);

    if let Some(ref stats) = results.claude_stats {
        print_claude_summary(stats, &results.claude_comparisons);
    }

    Ok(())
}

/// @ai:intent Run comparison only on existing results directory
/// @ai:effects network, fs:read, fs:write
fn run_comparison_only(results_dir: PathBuf, config_path: Option<PathBuf>) -> Result<()> {
    let config = load_or_default_config(config_path)?;

    // Validate directory structure
    let baseline_code_dir = results_dir.join("baseline").join("code");
    let aicms_code_dir = results_dir.join("aicms").join("code");

    if !baseline_code_dir.exists() || !aicms_code_dir.exists() {
        anyhow::bail!(
            "Invalid results directory structure. Expected:\n  {}/baseline/code/\n  {}/aicms/code/",
            results_dir.display(),
            results_dir.display()
        );
    }

    // Discover task IDs from the directory structure
    let tasks = discover_tasks_from_directory(&baseline_code_dir, &aicms_code_dir)?;

    if tasks.is_empty() {
        tracing::warn!("No tasks found with both baseline and aicms code. Nothing to compare.");
        return Ok(());
    }

    tracing::info!("Found {} tasks to compare", tasks.len());

    // Load comparison prompt
    let prompt_template = load_comparison_prompt(&config.paths.comparison_prompt_file)?;

    // Run comparisons
    let comparisons = run_comparison_on_discovered_tasks(&prompt_template, &tasks)?;

    // Print results
    if !comparisons.is_empty() {
        let stats = compute_comparison_stats(&comparisons);
        print_comparison_only_summary(&stats, &comparisons);
    }

    // Save comparison results
    save_comparison_results(&results_dir, &comparisons)?;

    Ok(())
}

/// @ai:intent Discover task IDs from existing code directories
/// @ai:effects fs:read
fn discover_tasks_from_directory(
    baseline_dir: &std::path::Path,
    aicms_dir: &std::path::Path,
) -> Result<Vec<DiscoveredTask>> {
    let mut tasks = Vec::new();
    let mut baseline_tasks = std::collections::HashSet::new();
    let mut aicms_tasks = std::collections::HashSet::new();

    // Collect baseline task IDs
    if baseline_dir.exists() {
        for entry in std::fs::read_dir(baseline_dir)? {
            let entry = entry?;

            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    baseline_tasks.insert(name.to_string());
                }
            }
        }
    }

    // Collect aicms task IDs
    if aicms_dir.exists() {
        for entry in std::fs::read_dir(aicms_dir)? {
            let entry = entry?;

            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    aicms_tasks.insert(name.to_string());
                }
            }
        }
    }

    // Find tasks that exist in both
    for task_id in baseline_tasks.intersection(&aicms_tasks) {
        tasks.push(DiscoveredTask {
            id: task_id.clone(),
            baseline_dir: baseline_dir.join(task_id),
            aicms_dir: aicms_dir.join(task_id),
        });
    }

    Ok(tasks)
}

/// @ai:intent Task discovered from directory structure
struct DiscoveredTask {
    id: String,
    baseline_dir: PathBuf,
    aicms_dir: PathBuf,
}

/// @ai:intent Run comparisons on discovered tasks
/// @ai:effects network, fs:read
fn run_comparison_on_discovered_tasks(
    prompt_template: &str,
    tasks: &[DiscoveredTask],
) -> Result<Vec<aicms_bench::metrics::TaskComparison>> {
    use aicms_bench::evaluator::{ClaudeScorer, ClaudeScorerTrait, CompilationChecker};
    use aicms_bench::metrics::TaskComparison;

    let scorer = ClaudeScorer::new(prompt_template.to_string());
    let compiler = CompilationChecker::new();
    let mut comparisons = Vec::new();
    let total = tasks.len();

    for (i, task) in tasks.iter().enumerate() {
        tracing::info!(
            "[{}/{}] Checking compilation for: {}",
            i + 1,
            total,
            task.id
        );

        // Check compilation for both implementations before comparing
        let baseline_compiles = check_directory_compiles(&compiler, &task.baseline_dir, "baseline", &task.id);
        let aicms_compiles = check_directory_compiles(&compiler, &task.aicms_dir, "aicms", &task.id);

        if !baseline_compiles || !aicms_compiles {
            tracing::warn!(
                "Skipping comparison for task {}: {} doesn't compile",
                task.id,
                if !baseline_compiles && !aicms_compiles {
                    "baseline and aicms"
                } else if !baseline_compiles {
                    "baseline"
                } else {
                    "aicms"
                }
            );
            continue;
        }

        // Build minimal spec from task ID
        let spec = format!("Task: {}\n\n(Task details from original corpus)", task.id);

        tracing::info!(
            "[{}/{}] Comparing implementations for: {}",
            i + 1,
            total,
            task.id
        );

        match scorer.compare_dirs(&spec, &task.baseline_dir, &task.aicms_dir) {
            Ok(comparison) => {
                comparisons.push(TaskComparison {
                    task_id: task.id.clone(),
                    comparison,
                });
            }
            Err(e) => {
                tracing::warn!("Failed to compare task {}: {}", task.id, e);
            }
        }
    }

    Ok(comparisons)
}

/// @ai:intent Compute comparison statistics
/// @ai:effects pure
fn compute_comparison_stats(
    comparisons: &[aicms_bench::metrics::TaskComparison],
) -> aicms_bench::metrics::ClaudeComparisonStats {
    let mut baseline_scores = Vec::new();
    let mut aicms_scores = Vec::new();
    let mut baseline_wins = 0;
    let mut aicms_wins = 0;
    let mut ties = 0;

    for comp in comparisons {
        baseline_scores.push(comp.comparison.baseline.overall as f64);
        aicms_scores.push(comp.comparison.aicms.overall as f64);

        match comp.comparison.winner.as_str() {
            "baseline" => baseline_wins += 1,
            "aicms" => aicms_wins += 1,
            _ => ties += 1,
        }
    }

    let avg_baseline = if baseline_scores.is_empty() {
        0.0
    } else {
        baseline_scores.iter().sum::<f64>() / baseline_scores.len() as f64
    };

    let avg_aicms = if aicms_scores.is_empty() {
        0.0
    } else {
        aicms_scores.iter().sum::<f64>() / aicms_scores.len() as f64
    };

    aicms_bench::metrics::ClaudeComparisonStats {
        avg_baseline_score: avg_baseline,
        avg_aicms_score: avg_aicms,
        baseline_wins,
        aicms_wins,
        ties,
    }
}

/// @ai:intent Print comparison-only summary
/// @ai:effects io
fn print_comparison_only_summary(
    stats: &aicms_bench::metrics::ClaudeComparisonStats,
    comparisons: &[aicms_bench::metrics::TaskComparison],
) {
    print_claude_summary(stats, comparisons);
}

/// @ai:intent Save comparison results to JSON file
/// @ai:effects fs:write
fn save_comparison_results(
    output_dir: &std::path::Path,
    comparisons: &[aicms_bench::metrics::TaskComparison],
) -> Result<()> {
    let output_path = output_dir.join("comparison_results.json");
    let json = serde_json::to_string_pretty(comparisons)?;
    std::fs::write(&output_path, json)?;
    tracing::info!("Comparison results saved to {}", output_path.display());
    Ok(())
}

/// @ai:intent Result of task execution
struct ExecutionData {
    metrics: Vec<TaskMetrics>,
}

/// @ai:intent Execute tasks and collect metrics
/// @ai:effects network
async fn execute_tasks<C: aicms_bench::runner::ClaudeClientTrait>(
    executor: &aicms_bench::runner::BenchmarkExecutor<C>,
    tasks: &[aicms_bench::corpus::Task],
) -> Result<ExecutionData> {
    let evaluator = Evaluator::new();
    let mut all_metrics = Vec::new();
    let total_tasks = tasks.len();

    for (index, task) in tasks.iter().enumerate() {
        let current = index + 1;
        tracing::info!("[{}/{}] Running task: {}", current, total_tasks, task.id);
        let executions = executor.execute_task(task).await?;

        for exec in &executions {
            let eval = evaluator.evaluate(task, exec)?;
            let metrics = TaskMetrics::from_evaluation(
                &eval,
                exec.input_tokens,
                exec.output_tokens,
                exec.execution_time_ms,
            );
            all_metrics.push(metrics);
        }
    }

    Ok(ExecutionData {
        metrics: all_metrics,
    })
}

/// @ai:intent Build task specification string for comparison
/// @ai:effects pure
fn build_task_spec(task: &aicms_bench::corpus::Task) -> String {
    format!(
        "Task: {}\n\nLanguage: {}\n\nDescription:\n{}",
        task.name,
        task.language.as_str(),
        task.description
    )
}

/// @ai:intent Load comparison prompt template from file
/// @ai:effects fs:read
fn load_comparison_prompt(path: &std::path::Path) -> Result<String> {
    if path.exists() {
        std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to load comparison prompt: {}", e))
    } else {
        tracing::info!(
            "Comparison prompt file not found at {:?}, using default",
            path
        );
        Ok(aicms_bench::evaluator::default_comparison_prompt())
    }
}

/// @ai:intent Run Claude comparisons for all tasks using the new directory structure
/// @ai:effects network, fs:read
fn run_claude_comparisons(
    config: &BenchmarkConfig,
    tasks: &[aicms_bench::corpus::Task],
    output_dir: &std::path::Path,
) -> Result<Vec<aicms_bench::metrics::TaskComparison>> {
    use aicms_bench::evaluator::{ClaudeScorer, ClaudeScorerTrait, CompilationChecker};
    use aicms_bench::metrics::TaskComparison;

    let prompt_template = load_comparison_prompt(&config.paths.comparison_prompt_file)?;
    let scorer = ClaudeScorer::new(prompt_template);
    let compiler = CompilationChecker::new();
    let mut comparisons = Vec::new();

    // New directory structure: {output_dir}/{mode}/code/{task_id}/
    let baseline_code_dir = output_dir.join("baseline").join("code");
    let aicms_code_dir = output_dir.join("aicms").join("code");

    // Find tasks that have both baseline and aicms directories
    let mut tasks_with_both = Vec::new();

    for task in tasks {
        let baseline_dir = baseline_code_dir.join(&task.id);
        let aicms_dir = aicms_code_dir.join(&task.id);

        let has_baseline = baseline_dir.exists();
        let has_aicms = aicms_dir.exists();

        if has_baseline && has_aicms {
            tasks_with_both.push((task, baseline_dir, aicms_dir));
        } else {
            let missing = match (has_baseline, has_aicms) {
                (false, false) => "baseline and aicms",
                (false, true) => "baseline",
                (true, false) => "aicms",
                _ => unreachable!(),
            };
            tracing::warn!(
                "Skipping comparison for task {}: missing {} directory",
                task.id,
                missing
            );
        }
    }

    let total = tasks_with_both.len();

    if total == 0 {
        tracing::warn!("No tasks have both baseline and aicms directories. Skipping comparisons.");
        return Ok(comparisons);
    }

    for (i, (task, baseline_dir, aicms_dir)) in tasks_with_both.iter().enumerate() {
        tracing::info!(
            "[{}/{}] Checking compilation for: {}",
            i + 1,
            total,
            task.id
        );

        // Check compilation for both implementations before comparing
        let baseline_compiles = check_directory_compiles(&compiler, baseline_dir, "baseline", &task.id);
        let aicms_compiles = check_directory_compiles(&compiler, aicms_dir, "aicms", &task.id);

        if !baseline_compiles || !aicms_compiles {
            tracing::warn!(
                "Skipping comparison for task {}: {} doesn't compile",
                task.id,
                if !baseline_compiles && !aicms_compiles {
                    "baseline and aicms"
                } else if !baseline_compiles {
                    "baseline"
                } else {
                    "aicms"
                }
            );
            continue;
        }

        let spec = build_task_spec(task);

        tracing::info!(
            "[{}/{}] Comparing implementations for: {}",
            i + 1,
            total,
            task.id
        );

        match scorer.compare_dirs(&spec, baseline_dir, aicms_dir) {
            Ok(comparison) => {
                comparisons.push(TaskComparison {
                    task_id: task.id.clone(),
                    comparison,
                });
            }
            Err(e) => {
                tracing::warn!("Failed to compare task {}: {}", task.id, e);
            }
        }
    }

    Ok(comparisons)
}

/// @ai:intent Check if directory compiles and log result
/// @ai:effects io
fn check_directory_compiles(
    compiler: &aicms_bench::evaluator::CompilationChecker,
    dir: &std::path::Path,
    mode: &str,
    task_id: &str,
) -> bool {
    use aicms_bench::evaluator::CompilationCheckerTrait;

    match compiler.check_directory(dir) {
        Ok(result) => {
            if result.success {
                tracing::debug!("{} ({}) compiles successfully", task_id, mode);
                true
            } else {
                tracing::warn!(
                    "{} ({}) compilation failed: {:?}",
                    task_id,
                    mode,
                    result.errors
                );
                false
            }
        }
        Err(e) => {
            tracing::warn!(
                "{} ({}) compilation check error: {}",
                task_id,
                mode,
                e
            );
            false
        }
    }
}

/// @ai:intent Print Claude comparison summary
/// @ai:effects io
fn print_claude_summary(
    stats: &aicms_bench::metrics::ClaudeComparisonStats,
    comparisons: &[aicms_bench::metrics::TaskComparison],
) {
    println!();
    println!("Claude Comparison Results");
    println!("=========================");
    println!();
    println!(
        "{:<25} {:>10} {:>10}",
        "", "Baseline", "AICMS"
    );
    println!("{}", "-".repeat(50));
    println!(
        "{:<25} {:>9.1} {:>9.1}",
        "Average Score:",
        stats.avg_baseline_score,
        stats.avg_aicms_score
    );
    println!();
    println!(
        "Wins: AICMS {} | Baseline {} | Ties {}",
        stats.aicms_wins, stats.baseline_wins, stats.ties
    );

    // Show detailed breakdown for each task
    for comp in comparisons {
        println!();
        println!("Task: {}", comp.task_id);
        println!("{}", "-".repeat(60));
        println!("Winner: {}", comp.comparison.winner.to_uppercase());
        println!("Summary: {}", comp.comparison.summary);
        println!();

        println!(
            "  {:<22} {:>10} {:>10}",
            "Aspect", "Baseline", "AICMS"
        );
        println!("  {}", "-".repeat(44));

        println!(
            "  {:<22} {:>10} {:>10}",
            "Overall:",
            comp.comparison.baseline.overall,
            comp.comparison.aicms.overall
        );
        println!(
            "  {:<22} {:>10} {:>10}",
            "Intent Match:",
            comp.comparison.baseline.intent_match.score,
            comp.comparison.aicms.intent_match.score
        );
        println!(
            "  {:<22} {:>10} {:>10}",
            "Edge Cases:",
            comp.comparison.baseline.edge_cases.score,
            comp.comparison.aicms.edge_cases.score
        );
        println!(
            "  {:<22} {:>10} {:>10}",
            "Code Quality:",
            comp.comparison.baseline.code_quality.score,
            comp.comparison.aicms.code_quality.score
        );
        println!(
            "  {:<22} {:>10} {:>10}",
            "Annotation Compliance:",
            comp.comparison.baseline.annotation_compliance.score,
            comp.comparison.aicms.annotation_compliance.score
        );

        // Show reasons for differences
        println!();
        println!("  Baseline reasons:");
        print_aspect_reason("Intent Match", &comp.comparison.baseline.intent_match.reason);
        print_aspect_reason("Edge Cases", &comp.comparison.baseline.edge_cases.reason);
        print_aspect_reason("Code Quality", &comp.comparison.baseline.code_quality.reason);
        print_aspect_reason("Annotations", &comp.comparison.baseline.annotation_compliance.reason);

        println!();
        println!("  AICMS reasons:");
        print_aspect_reason("Intent Match", &comp.comparison.aicms.intent_match.reason);
        print_aspect_reason("Edge Cases", &comp.comparison.aicms.edge_cases.reason);
        print_aspect_reason("Code Quality", &comp.comparison.aicms.code_quality.reason);
        print_aspect_reason("Annotations", &comp.comparison.aicms.annotation_compliance.reason);
    }

    println!();
}

/// @ai:intent Print a single aspect reason with wrapping
/// @ai:effects io
fn print_aspect_reason(aspect: &str, reason: &str) {
    println!("    {}: {}", aspect, reason);
}

/// @ai:intent Generate reports from results file
/// @ai:effects fs:read, fs:write
fn generate_reports(results_path: PathBuf, output_dir: PathBuf) -> Result<()> {
    let content = std::fs::read_to_string(&results_path)?;
    let results: aicms_bench::BenchmarkResults = serde_json::from_str(&content)?;

    let reporter = ReportGenerator::new();
    reporter.generate_all(&results, &output_dir)?;

    println!("Reports generated in {}", output_dir.display());
    Ok(())
}

/// @ai:intent List available tasks
/// @ai:effects fs:read
fn list_tasks(category: Option<String>, language: Option<String>) -> Result<()> {
    let config = BenchmarkConfig::default();
    let loader = CorpusLoader::new();

    let filter = FilterConfig {
        categories: category.map(|c| vec![c]),
        languages: language.map(|l| vec![l]),
        ..Default::default()
    };

    let tasks = loader.load_filtered(&config.paths.corpus_dir, &filter)?;

    println!("Available tasks ({}):", tasks.len());
    println!();
    println!("{:<30} {:<12} {:<12} {:<10}", "ID", "Category", "Language", "Difficulty");
    println!("{}", "-".repeat(70));

    for task in &tasks {
        println!(
            "{:<30} {:<12} {:<12} {:<10}",
            task.id,
            task.category.as_str(),
            task.language.as_str(),
            task.difficulty.as_str()
        );
    }

    Ok(())
}

/// @ai:intent Validate corpus tasks can be loaded
/// @ai:effects fs:read
fn validate() -> Result<()> {
    let config = BenchmarkConfig::default();
    let loader = CorpusLoader::new();
    let tasks = loader.load_all(&config.paths.corpus_dir)?;

    println!("Corpus validation passed!");
    println!("Total tasks: {}", tasks.len());

    for task in &tasks {
        println!("  - {} ({})", task.id, task.language);
    }

    Ok(())
}

/// @ai:intent Initialize default configuration file
/// @ai:effects fs:write
fn init_config(output: PathBuf) -> Result<()> {
    let config = BenchmarkConfig::default();
    config.save(&output)?;
    println!("Configuration saved to {}", output.display());
    Ok(())
}

/// @ai:intent Load configuration or use defaults
/// @ai:effects fs:read
fn load_or_default_config(path: Option<PathBuf>) -> Result<BenchmarkConfig> {
    match path {
        Some(p) => BenchmarkConfig::load(&p),
        None => {
            let default_path = PathBuf::from("benchmark.toml");

            if default_path.exists() {
                BenchmarkConfig::load(&default_path)
            } else {
                Ok(BenchmarkConfig {
                    paths: PathConfig {
                        corpus_dir: PathBuf::from("corpus"),
                        prompts_dir: PathBuf::from("prompts"),
                        results_dir: PathBuf::from("results"),
                        skill_file: PathBuf::from("../skills/aicms/SKILL.md"),
                        comparison_prompt_file: PathBuf::from("prompts/comparison.md"),
                    },
                    ..Default::default()
                })
            }
        }
    }
}

/// @ai:intent Build filter from CLI arguments
/// @ai:effects pure
fn build_filter(
    categories: Option<String>,
    languages: Option<String>,
    tasks: Option<String>,
) -> FilterConfig {
    FilterConfig {
        categories: categories.map(|s| s.split(',').map(|c| c.trim().to_string()).collect()),
        languages: languages.map(|s| s.split(',').map(|l| l.trim().to_string()).collect()),
        task_ids: tasks.map(|s| s.split(',').map(|t| t.trim().to_string()).collect()),
        difficulties: None,
    }
}

/// @ai:intent Print summary to console
/// @ai:effects io
fn print_summary(results: &aicms_bench::BenchmarkResults) {
    println!();
    println!("AICMS Benchmark Results");
    println!("=======================");
    println!();

    // Check for extraction failures
    let extraction_warnings = check_extraction_failures(&results.task_metrics);
    if !extraction_warnings.is_empty() {
        println!("Warnings:");
        for warning in &extraction_warnings {
            println!("  {}", warning);
        }
        println!();
    }

    println!(
        "{:<25} {:>10} {:>10} {:>10}",
        "", "Baseline", "AICMS", "Delta"
    );
    println!("{}", "-".repeat(60));
    println!(
        "{:<25} {:>9.1}% {:>9.1}% {:>+9.1}%",
        "Compilation rate:",
        results.overall.baseline.compilation_rate,
        results.overall.aicms.compilation_rate,
        results.overall.delta.compilation_rate
    );
    println!(
        "{:<25} {:>9.1}% {:>9.1}% {:>+9.1}%",
        "Test pass rate:",
        results.overall.baseline.avg_test_pass_rate,
        results.overall.aicms.avg_test_pass_rate,
        results.overall.delta.test_pass_rate
    );
    println!(
        "{:<25} {:>9.1}% {:>9.1}% {:>+9.1}%",
        "Lint compliance:",
        results.overall.baseline.avg_lint_compliance,
        results.overall.aicms.avg_lint_compliance,
        results.overall.delta.lint_compliance
    );
    println!();

    // Show lint issues if any
    print_lint_issues(&results.task_metrics);
}

/// @ai:intent Check for extraction failures and return warnings
/// @ai:effects pure
fn check_extraction_failures(metrics: &[aicms_bench::metrics::TaskMetrics]) -> Vec<String> {
    use std::collections::HashSet;

    let mut warnings = Vec::new();
    let mut tasks_missing_baseline = HashSet::new();
    let mut tasks_missing_aicms = HashSet::new();

    for m in metrics {
        if !m.code_extracted {
            match m.mode.as_str() {
                "baseline" => tasks_missing_baseline.insert(m.task_id.clone()),
                "aicms" => tasks_missing_aicms.insert(m.task_id.clone()),
                _ => false,
            };
        }
    }

    for task_id in &tasks_missing_baseline {
        warnings.push(format!(
            "Code extraction failed for {} (baseline) - metrics show 0%",
            task_id
        ));
    }

    for task_id in &tasks_missing_aicms {
        warnings.push(format!(
            "Code extraction failed for {} (aicms) - metrics show 0%",
            task_id
        ));
    }

    warnings
}

/// @ai:intent Print lint issues for each task/mode
/// @ai:effects io
fn print_lint_issues(metrics: &[aicms_bench::metrics::TaskMetrics]) {
    let issues_to_show: Vec<_> = metrics
        .iter()
        .filter(|m| !m.lint_issues.is_empty() && m.code_extracted)
        .collect();

    if issues_to_show.is_empty() {
        return;
    }

    println!("Lint Issues:");
    println!("{}", "-".repeat(60));

    for m in issues_to_show {
        println!("  {} ({}):", m.task_id, m.mode);

        for issue in &m.lint_issues {
            println!("    - {}", issue);
        }
    }

    println!();
}
