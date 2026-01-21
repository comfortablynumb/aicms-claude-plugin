# AICMS GitHub Action

Validate AICMS annotations and detect breaking contract changes in your CI/CD pipeline.

## Features

- **Lint annotations**: Ensure all functions have required `@ai:intent` annotations
- **Confidence checks**: Warn on functions with low confidence scores
- **Breaking change detection**: Detect breaking contract changes in PRs
- **Flexible output**: Text or JSON output formats

## Usage

### Basic Linting

```yaml
name: AICMS Validation

on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: aicms/check@v1
        with:
          path: 'src/'
          require-intent: 'true'
```

### Full Validation with Breaking Change Detection

```yaml
name: AICMS Validation

on:
  pull_request:
    branches: [main]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Required for diff

      - uses: aicms/check@v1
        with:
          path: 'src/'
          require-intent: 'true'
          require-module-intent: 'true'
          check-breaking-changes: 'true'
          base-branch: 'main'
```

## Inputs

| Input | Description | Default |
|-------|-------------|---------|
| `path` | Path to lint (file or directory) | `.` |
| `require-intent` | Require `@ai:intent` on all functions | `true` |
| `require-module-intent` | Require `@ai:module:intent` on all files | `false` |
| `warn-low-confidence` | Warn on low confidence scores | `true` |
| `confidence-threshold` | Minimum confidence threshold (0.0-1.0) | `0.7` |
| `fail-on-warning` | Fail if warnings are found | `false` |
| `check-breaking-changes` | Check for breaking changes in PR | `false` |
| `base-branch` | Base branch for breaking change detection | `main` |
| `output-format` | Output format: text, json, json-pretty | `text` |

## Outputs

| Output | Description |
|--------|-------------|
| `errors` | Number of errors found |
| `warnings` | Number of warnings found |
| `breaking-changes` | Number of breaking contract changes |
| `result` | Full lint result in JSON format |

## Breaking Change Detection

The action can detect these breaking contract changes:

**🔴 BREAKING Changes (fail the check)**
- `@ai:pre` strengthened (new requirements added)
- `@ai:post` weakened (guarantees removed)
- `@ai:effects` expanded (new side effects)
- `@ai:idempotent` changed from `true` to `false`

**🟡 NOTABLE Changes (warnings only)**
- `@ai:intent` modified
- `@ai:confidence` decreased significantly
- `@ai:deprecated` added

**🟢 NON-BREAKING Changes (allowed)**
- `@ai:pre` weakened (accepts more inputs)
- `@ai:post` strengthened (stronger guarantees)
- `@ai:effects` reduced (fewer side effects)

## Example Workflow Files

See the `examples/` directory for complete workflow templates:

- `basic-lint.yml` - Simple linting workflow
- `full-validation.yml` - Full validation with PR comments
