# AICMS Benchmark System

An automated benchmark system for measuring AICMS (AI-First Code Metadata Specification) effectiveness by running Claude on coding tasks with and without AICMS context.

## Features

- **Comparative benchmarking**: Run tasks with baseline prompts vs AICMS-aware prompts
- **Multi-language support**: Rust, Python, TypeScript
- **Multiple task categories**: Implementation, bugfix, refactor, inference
- **Automated evaluation**: Compilation checking, test execution, example validation, lint compliance
- **Comprehensive reporting**: JSON data, Markdown summaries, PNG charts

## Installation

```bash
cd benchmark
cargo build --release
```

## Usage

### Run Benchmarks

```bash
# Run all tasks
aicms-bench run

# Run with specific configuration
aicms-bench run --config benchmark.toml

# Filter by category
aicms-bench run --categories implement,bugfix

# Filter by language
aicms-bench run --languages rust,python

# Run specific tasks
aicms-bench run --tasks impl-rust-factorial,impl-rust-fibonacci

# Dry run (no API calls)
aicms-bench run --dry-run

# Use direct API instead of Claude Code CLI (requires ANTHROPIC_API_KEY)
aicms-bench run --use-api

# Multiple repetitions for statistical validity
aicms-bench run --repetitions 3

# Run with comparison scoring (uses Claude to compare implementations)
aicms-bench run --compare
```

### Run Comparison Only

Run comparison on existing benchmark results without re-generating code:

```bash
# Compare implementations in an existing results directory
aicms-bench compare --results-dir results/2026-01-19_12-00-00

# With custom configuration
aicms-bench compare --results-dir results/2026-01-19_12-00-00 --config benchmark.toml
```

### Generate Reports

```bash
# Generate reports from existing results
aicms-bench report --results results/2026-01-19_12-00-00/results.json --output reports/
```

### List Tasks

```bash
# List all tasks
aicms-bench list

# Filter by category
aicms-bench list --category implement

# Filter by language
aicms-bench list --language rust
```

### Validate Corpus

```bash
# Check corpus for errors
aicms-bench validate
```

### Initialize Configuration

```bash
# Create default config file
aicms-bench init --output benchmark.toml
```

## Configuration

Create a `benchmark.toml` file:

```toml
[api]
model = "claude-sonnet-4-20250514"
max_tokens = 4096
temperature = 0.0
requests_per_minute = 60

[run]
repetitions = 1
dry_run = false

[paths]
corpus_dir = "corpus"
prompts_dir = "prompts"
results_dir = "results"
skill_file = "../skills/aicms/SKILL.md"
```

## Environment Variables

- `ANTHROPIC_API_KEY`: Only required when using `--use-api` flag

## Claude Code CLI (Default)

By default, the benchmark uses Claude Code CLI (`claude` command), which means:
- No API key required
- Uses your existing Claude Code authentication
- Runs benchmarks through the same interface you use interactively
- **Ignores user's `~/.claude/CLAUDE.md`** to avoid influencing code generation

```bash
# Default: uses Claude Code CLI
aicms-bench run --tasks impl-rust-factorial
```

## Fair Comparison

When running comparisons (`--compare` flag or `compare` command), the system ensures fair evaluation:

1. **Compilation verification**: Both implementations must compile successfully before comparison. Tasks where either implementation fails to compile are skipped.
2. **Ignores `@ai:*` annotations**: The comparison prompt instructs Claude to ignore all AICMS annotations when scoring, focusing only on the actual code implementation
3. **Isolated generation**: Code generation uses `--setting-sources project,local` to exclude user-level settings from influencing results
4. **Identical task specs**: Both baseline and AICMS modes receive the same task description

## Direct API Mode

If you prefer to use the Anthropic API directly (useful for automation or CI/CD):

```bash
# Use direct API (requires ANTHROPIC_API_KEY environment variable)
aicms-bench run --use-api --tasks impl-rust-factorial
```

## Task Corpus

The benchmark includes 30+ tasks across categories:

| Category   | Description                              | Count |
|------------|------------------------------------------|-------|
| Implement  | Implement functions from AICMS specs     | 18    |
| Bugfix     | Find and fix bugs in annotated code      | 8     |
| Refactor   | Refactor code while maintaining behavior | 6     |
| Inference  | Add AICMS annotations to unannotated code| 6     |

### Task Format

Tasks are defined in TOML files:

```toml
[task]
id = "impl-rust-factorial"
name = "Factorial Implementation"
category = "implement"
language = "rust"
difficulty = "easy"
description = "Implement the factorial function..."

[input]
code = """
/// @ai:intent Calculate the factorial
/// @ai:example (5) -> 120
fn factorial(n: u64) -> u64 { todo!() }
"""
test_code = "..."

[expected]
compiles = true
[[expected.example_cases]]
inputs = "5"
expected_output = "120"
```

## Metrics

| Metric               | Description                                |
|----------------------|--------------------------------------------|
| Compilation rate     | Percentage of code that compiles           |
| Test pass rate       | Percentage of tests passed                 |
| Example satisfaction | Percentage of @ai:example cases satisfied  |
| Lint compliance      | Percentage of valid AICMS annotations      |
| Annotation quality   | Quality score for inferred annotations     |

## Output

Results are saved to `results/<timestamp>/` with the following structure:

```
results/2026-01-19_12-00-00/
├── baseline/
│   ├── code/                    # Generated code for baseline mode
│   │   ├── impl-rust-factorial/
│   │   │   ├── src/
│   │   │   │   └── lib.rs
│   │   │   └── Cargo.toml
│   │   └── impl-rust-fibonacci/
│   │       └── ...
│   └── report/                  # Logs and interaction records
│       ├── impl-rust-factorial/
│       │   └── _claude_interaction.log
│       └── impl-rust-fibonacci/
│           └── _claude_interaction.log
├── aicms/
│   ├── code/                    # Generated code for AICMS mode
│   │   └── ...
│   └── report/                  # Logs and interaction records
│       └── ...
├── results.json                 # Complete benchmark data
├── results.md                   # Human-readable summary
├── comparison.png               # Overall comparison chart
├── by_language.png              # Language breakdown chart
├── by_difficulty.png            # Difficulty breakdown chart
├── comparison_prompt.md         # Prompt used for comparison
└── comparison_results.json      # Detailed comparison results (if --compare used)
```

## Architecture

```
benchmark/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Library exports
│   ├── config.rs        # Configuration types
│   ├── corpus/          # Task definitions and loader
│   ├── runner/          # Claude API client and executor
│   ├── evaluator/       # Compilation, tests, examples, linting
│   ├── metrics/         # Aggregation and statistics
│   └── report/          # JSON, Markdown, chart generation
├── corpus/              # Task TOML files
├── prompts/             # Baseline and AICMS prompts
└── results/             # Output directory
```

## Adding New Tasks

1. Create a TOML file in the appropriate corpus directory
2. Define task metadata, input code, and expected results
3. Run `aicms-bench validate` to check for errors
4. Run the benchmark to include the new task

## License

MIT
