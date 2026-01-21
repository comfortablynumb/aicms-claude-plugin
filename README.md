# AICMS: AI-First Code Metadata Specification

A language-agnostic standard for embedding AI-consumable metadata in source code. AICMS helps AI coding assistants understand, generate, verify, and maintain code more effectively.

## Why AICMS?

When AI assistants generate code, they often miss the **why** behind the code. AICMS solves this by embedding structured metadata directly in your source files:

| Problem | AICMS Solution |
|---------|----------------|
| AI doesn't understand function purpose | `@ai:intent` describes what it should do |
| Generated code lacks input validation | `@ai:pre` defines preconditions to enforce |
| No guarantees about return values | `@ai:post` specifies postconditions |
| Missing test cases | `@ai:example` provides executable specs |
| Unknown side effects | `@ai:effects` declares purity and I/O |
| No confidence tracking | `@ai:confidence` rates implementation trust |

**Result**: AI generates better code, catches its own mistakes, and produces implementations that match your intent.

## Installation

### Claude Code Plugin (Recommended)

```bash
# Add the AICMS marketplace
/plugin marketplace add aicms/aicms

# Install the plugin
/plugin install aicms
```

**What you get:**
- Claude automatically understands `@ai:*` annotations
- 5 slash commands: `/aicms:implement`, `/aicms:infer`, `/aicms:tests`, `/aicms:contracts`, `/aicms:diff`
- Optional pre-commit hooks for validation

### Manual Installation

```bash
# 1. Copy the skill to your project
mkdir -p .claude/skills/aicms
cp path/to/aicms/skills/aicms/SKILL.md .claude/skills/aicms/SKILL.md

# 2. Add to your CLAUDE.md
echo "@.claude/skills/aicms/SKILL.md" >> CLAUDE.md
```

## Quick Start

### 1. Add Project Constraints (Optional)

Place `@ai:project:*` tags in your **main entrypoint file** (`main.rs`, `main.py`, `index.ts`, etc.):

```rust
// main.rs
//! @ai:project:max_function_lines 40
//! @ai:project:max_params 4
//! @ai:project:require_interface_for_deps true
//! @ai:project:error_strategy Result
//! @ai:project:no_panic true

fn main() {
    // ...
}
```

### 2. Add Module Metadata

Place `@ai:module:*` tags in your **module entrypoint file** (`mod.rs`, `__init__.py`, `index.ts`, etc.):

```rust
// src/auth/mod.rs
//! @ai:module:intent Handle user authentication and session management
//! @ai:module:layer application
//! @ai:module:public_api AuthService, Session, AuthError
//! @ai:module:depends_on domain::user, infrastructure::token_store
//! @ai:module:stateless true

pub mod service;
pub mod session;
pub mod error;
```

### 3. Annotate Functions

```rust
/// @ai:intent Calculate compound interest for a loan
/// @ai:pre principal > 0, rate >= 0, years >= 0
/// @ai:post result >= principal
/// @ai:example (1000.0, 0.05, 10) -> 1628.89
/// @ai:effects pure
fn calculate_compound_interest(principal: f64, rate: f64, years: u32) -> f64 {
    principal * (1.0 + rate).powi(years as i32)
}
```

## Annotation Levels

AICMS operates at three levels:

### Project Level (`@ai:project:*`)

**Location:** Main entrypoint file (`main.rs`, `main.py`, `main.go`, `index.ts`, `App.java`)

Defines codebase-wide constraints that Claude enforces when generating code:

| Tag | Default | Description |
|-----|---------|-------------|
| `@ai:project:max_function_lines` | 50 | Maximum lines per function |
| `@ai:project:max_file_lines` | 500 | Maximum lines per file |
| `@ai:project:max_params` | 4 | Maximum function parameters |
| `@ai:project:require_interface_for_deps` | true | Dependencies behind traits/interfaces |
| `@ai:project:error_strategy` | Result | Error handling: `Result`, `exceptions`, `error_codes` |
| `@ai:project:no_panic` | false | Disallow panic/unwrap in production |

### Module Level (`@ai:module:*`)

**Location:** Module entrypoint file (`mod.rs`, `__init__.py`, `index.ts`, `package-info.java`)

Describes the purpose and constraints of a module:

| Tag | Description |
|-----|-------------|
| `@ai:module:intent` | Single sentence describing module's responsibility |
| `@ai:module:layer` | Architecture layer: `domain`, `application`, `infrastructure`, `presentation` |
| `@ai:module:public_api` | Comma-separated list of exported items |
| `@ai:module:depends_on` | Modules this depends on |
| `@ai:module:stateless` | Module has no global mutable state |

### Function Level (`@ai:*`)

**Location:** Immediately before the function definition

| Tag | Required | Description |
|-----|----------|-------------|
| `@ai:intent` | Yes | Natural language purpose |
| `@ai:pre` | No | Preconditions (comma-separated expressions) |
| `@ai:post` | No | Postconditions (`result` = return value) |
| `@ai:example` | No | Test cases: `(args) -> expected` |
| `@ai:effects` | No | Side effects: `pure`, `io`, `db:read`, `db:write`, `network`, `fs:read`, `fs:write` |
| `@ai:idempotent` | No | Safe to call multiple times: `true`/`false` |
| `@ai:confidence` | No | AI's confidence: `0.0-1.0` |
| `@ai:needs_review` | No | Flags for human review |
| `@ai:complexity` | No | Time/space complexity: `O(n)`, `O(n log n)` |

## Language Examples

### Rust

```rust
// main.rs - Project constraints
//! @ai:project:max_function_lines 40
//! @ai:project:require_interface_for_deps true

// src/payment/mod.rs - Module metadata
//! @ai:module:intent Process payments and handle transactions
//! @ai:module:layer application
//! @ai:module:public_api PaymentService, Transaction, PaymentError

// src/payment/service.rs - Function annotations
/// @ai:intent Process a payment transaction
/// @ai:pre amount > 0, account.is_active()
/// @ai:post account.balance == old(account.balance) - amount
/// @ai:effects db:write, network
/// @ai:idempotent false
pub fn process_payment(account: &mut Account, amount: u64) -> Result<Receipt, Error> {
    // implementation
}
```

### Python

```python
# main.py - Project constraints
# @ai:project:max_function_lines 40
# @ai:project:error_strategy exceptions

# src/payment/__init__.py - Module metadata
# @ai:module:intent Process payments and handle transactions
# @ai:module:layer application
# @ai:module:public_api PaymentService, Transaction, PaymentError

# src/payment/service.py - Function annotations
# @ai:intent Process a payment transaction
# @ai:pre amount > 0 and account.is_active
# @ai:post account.balance == old_balance - amount
# @ai:effects db:write, network
def process_payment(account: Account, amount: int) -> Receipt:
    # implementation
```

### TypeScript

```typescript
// index.ts - Project constraints
/**
 * @ai:project:max_function_lines 40
 * @ai:project:error_strategy exceptions
 */

// src/payment/index.ts - Module metadata
/**
 * @ai:module:intent Process payments and handle transactions
 * @ai:module:layer application
 * @ai:module:public_api PaymentService, Transaction, PaymentError
 */

// src/payment/service.ts - Function annotations
/**
 * @ai:intent Process a payment transaction
 * @ai:pre amount > 0 && account.isActive
 * @ai:post account.balance === oldBalance - amount
 * @ai:effects db:write, network
 */
export function processPayment(account: Account, amount: number): Receipt {
    // implementation
}
```

## Slash Commands

| Command | Description |
|---------|-------------|
| `/aicms:implement <path>` | Generate implementation for stub functions with specs |
| `/aicms:infer <path>` | Propose annotations for unannotated code |
| `/aicms:tests <path>` | Generate tests from `@ai:example` annotations |
| `/aicms:contracts <path>` | Generate runtime assertion wrappers |
| `/aicms:diff <path>` | Detect breaking contract changes |

### Spec-First Development

Write the spec, let Claude implement:

```rust
/// @ai:intent Sort a list using merge sort algorithm
/// @ai:pre list.len() > 0
/// @ai:post result.is_sorted(), result.len() == list.len()
/// @ai:example ([3, 1, 4, 1, 5]) -> [1, 1, 3, 4, 5]
/// @ai:complexity O(n log n)
/// @ai:effects pure
fn merge_sort<T: Ord + Clone>(list: Vec<T>) -> Vec<T> {
    todo!()  // Claude implements this
}
```

Run `/aicms:implement src/sorting.rs` and Claude generates the implementation.

## CLI Tool

For CI/CD integration, use the Rust parser:

```bash
# Build
cd parser && cargo build --release

# Lint for compliance
aicms lint src/
aicms lint --require-intent --require-module-intent src/

# Extract annotations to JSON
aicms extract src/math.rs --format json-pretty

# Detect breaking changes
aicms diff old.rs new.rs --fail-on-breaking
```

## GitHub Action

```yaml
- uses: aicms/check@v1
  with:
    path: 'src/'
    require-intent: 'true'
    check-breaking-changes: 'true'
```

## Benefits

| Benefit | How |
|---------|-----|
| **Better code generation** | AI understands intent, not just syntax |
| **Automatic test generation** | Tests from `@ai:example` annotations |
| **Contract validation** | Catch spec/implementation mismatches |
| **Breaking change detection** | Semantic diff for contract changes |
| **Provenance tracking** | Know who wrote what and confidence level |
| **Spec-first development** | Write specs, AI implements |

## Project Structure

```
aicms/
├── .claude-plugin/           # Plugin configuration
│   ├── plugin.json
│   └── marketplace.json
├── skills/aicms/SKILL.md     # Core skill (teaches Claude AICMS)
├── commands/                 # Slash commands
├── hooks/                    # Pre-commit hooks
├── parser/                   # CLI tool for CI/CD
├── examples/                 # Annotated examples
└── integrations/             # GitHub Action
```

## Links

- [Full Specification](skills/aicms/SKILL.md)
- [Roadmap](doc/ROADMAP.md)
- [GitHub Action](integrations/github-action/README.md)
