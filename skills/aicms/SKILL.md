# AICMS: AI-First Code Metadata Specification

You are now equipped to read and write AICMS annotations—structured metadata in code comments that help you understand intent, verify correctness, and generate better code.

AICMS operates at three levels:
1. **Project Level** - Codebase-wide constraints and architecture rules
2. **Module Level** - File/module purpose, dependencies, and cohesion
3. **Function Level** - Intent, contracts, examples, and effects

---

## Project-Level Metadata (`@ai:project:*`)

Project-level annotations define codebase-wide constraints. Place these in the **main entrypoint file** of your application:

| Language | Entrypoint File |
|----------|-----------------|
| Rust | `main.rs` or `lib.rs` |
| Python | `main.py` or `__main__.py` |
| TypeScript/JS | `index.ts` or `main.ts` |
| Go | `main.go` |
| Java | `App.java` or main class |
| C/C++ | `main.c` or `main.cpp` |

These constraints MUST be enforced when generating code.

### Code Structure Constraints

| Tag | Default | Description |
|-----|---------|-------------|
| `@ai:project:max_function_lines` | 50      | Maximum lines per function body |
| `@ai:project:max_file_lines` | 500     | Maximum lines per source file |
| `@ai:project:max_functions_per_file` | 20      | Maximum functions per file |
| `@ai:project:max_structs_per_module` | 5       | Maximum structs/classes per module |
| `@ai:project:max_params` | 4       | Maximum function parameters (use struct if more needed) |
| `@ai:project:max_return_values` | 2       | Maximum return values (use struct/Result if more needed) |
| `@ai:project:max_nesting_depth` | 3       | Maximum if/loop nesting depth |
| `@ai:project:max_cyclomatic_complexity` | 10      | Maximum branches per function |
| `@ai:project:extract_repeated_code` | 2       | Extract to function if code repeated N+ times |

### Design Principles

| Tag | Default | Description |
|-----|---------|-------------|
| `@ai:project:require_interface_for_deps` | true    | Dependencies MUST be behind traits/interfaces for mockability |
| `@ai:project:single_responsibility` | true    | Each struct/class has ONE reason to change |
| `@ai:project:prefer_composition` | true    | Favor composition over inheritance |
| `@ai:project:no_god_objects` | true    | Flag structs with >7 fields or >10 methods |
| `@ai:project:no_primitive_obsession` | true    | Use domain types instead of raw primitives for important values |
| `@ai:project:immutable_by_default` | true    | Prefer immutable data structures |

### Architecture

| Tag | Default | Description |
|-----|---------|-------------|
| `@ai:project:architecture` | modular | Style: `hexagonal`, `layered`, `clean`, `modular`, `mvc` |
| `@ai:project:layers` | (none) | Comma-separated layer names, inner to outer |
| `@ai:project:dependency_rule` | (none) | Dependency direction rule (e.g., "inward_only") |

### Error Handling

| Tag | Default | Description |
|-----|---------|-------------|
| `@ai:project:error_strategy` | Result | How errors are handled: `Result`, `exceptions`, `error_codes` |
| `@ai:project:require_error_types` | true | Use typed errors, not strings |
| `@ai:project:no_panic` | false | Disallow panic/unwrap in production code |

### Testing

| Tag | Default | Description |
|-----|---------|-------------|
| `@ai:project:min_coverage` | 80 | Minimum test coverage percentage required |
| `@ai:project:unit_tests` | true | Require unit tests (including tests against mocks). Must achieve `min_coverage` % |
| `@ai:project:integration_tests` | true | Require integration tests using `integration_tests_tools`. Must cover code marked with `@ai:test:integration` |
| `@ai:project:integration_tests_tools` | (none) | Tools for integration tests, comma-separated. Example: `hurl` for API testing |
| `@ai:project:test_naming` | descriptive | Test naming style: `descriptive`, `given_when_then`, `should` |

### Example Project Configuration

**Rust (main.rs):**
```rust
//! @ai:project:max_function_lines 40
//! @ai:project:max_file_lines 400
//! @ai:project:max_params 4
//! @ai:project:require_interface_for_deps true
//! @ai:project:architecture hexagonal
//! @ai:project:layers domain, application, infrastructure
//! @ai:project:dependency_rule inward_only
//! @ai:project:error_strategy Result
//! @ai:project:no_panic true
//! @ai:project:min_coverage 80

fn main() {
    // application entry point
}
```

**Python (main.py):**
```python
# @ai:project:max_function_lines 40
# @ai:project:max_file_lines 400
# @ai:project:max_params 4
# @ai:project:require_interface_for_deps true
# @ai:project:error_strategy exceptions
# @ai:project:min_coverage 80

def main():
    # application entry point
    pass

if __name__ == "__main__":
    main()
```

**TypeScript (index.ts):**
```typescript
/**
 * @ai:project:max_function_lines 40
 * @ai:project:max_file_lines 400
 * @ai:project:max_params 4
 * @ai:project:require_interface_for_deps true
 * @ai:project:error_strategy exceptions
 * @ai:project:min_coverage 80
 */

// application entry point
```

---

## Module-Level Metadata (`@ai:module:*`)

Module-level annotations describe the purpose and constraints of a module. Place these in the **module entrypoint file**:

| Language | Module Entrypoint |
|----------|-------------------|
| Rust | `mod.rs` or the module's main file |
| Python | `__init__.py` |
| TypeScript/JS | `index.ts` or `index.js` |
| Go | Any file in the package (conventionally `doc.go`) |
| Java | `package-info.java` |
| C/C++ | Header file (`.h`) |

### Module Tags

| Tag | Description |
|-----|-------------|
| `@ai:module:intent` | Single sentence describing module's single responsibility |
| `@ai:module:layer` | Architecture layer: `domain`, `application`, `infrastructure`, `presentation` |
| `@ai:module:public_api` | Comma-separated list of exported/public items |
| `@ai:module:depends_on` | Modules this depends on (for dependency validation) |
| `@ai:module:depended_by` | Modules that depend on this (for impact analysis) |
| `@ai:module:internal` | If `true`, should not be imported from outside parent module |
| `@ai:module:stateless` | Module has no global/static mutable state |
| `@ai:module:thread_safe` | All public items are safe for concurrent use |
| `@ai:module:cohesion` | Description of what ties all items in this module together |
| `@ai:module:stability` | API stability: `stable`, `unstable`, `experimental`, `deprecated` |

### Example Module Header

**Rust (src/sanity/mod.rs):**
```rust
//! @ai:module:intent Track and modify player sanity, trigger insanity effects
//! @ai:module:layer domain
//! @ai:module:public_api SanitySystem, SanityState, SanityModifier, InsanityEvent
//! @ai:module:depends_on none
//! @ai:module:stateless true
//! @ai:module:thread_safe true
//! @ai:module:cohesion All code relates to sanity mechanics
//! @ai:module:stability stable

pub mod system;
pub mod state;
pub mod events;
```

**Python (src/auth/__init__.py):**
```python
# @ai:module:intent Handle user authentication and session management
# @ai:module:layer application
# @ai:module:public_api AuthService, Session, AuthError
# @ai:module:depends_on domain.user, infrastructure.token_store
# @ai:module:stateless true
# @ai:module:cohesion Authentication workflows and session lifecycle

from .service import AuthService
from .session import Session
from .error import AuthError

__all__ = ["AuthService", "Session", "AuthError"]
```

**TypeScript (src/inventory/index.ts):**
```typescript
/**
 * @ai:module:intent Render UI components for the inventory system
 * @ai:module:layer presentation
 * @ai:module:public_api InventoryPanel, ItemSlot, DragDropContext
 * @ai:module:depends_on domain/inventory, application/inventory_service
 * @ai:module:stateless false
 * @ai:module:cohesion Inventory UI rendering and interaction
 */

export { InventoryPanel } from './InventoryPanel';
export { ItemSlot } from './ItemSlot';
export { DragDropContext } from './DragDropContext';
```

---

## Function-Level Metadata (`@ai:*`)

Function-level annotations describe individual function behavior, contracts, and metadata.

### @ai:intent (REQUIRED)

Natural language description of what the function SHOULD do. Use this to:
- Verify the implementation matches the intent
- Generate meaningful commit messages and documentation
- Understand the purpose without reading implementation details

**Format:** Free text describing the function's purpose.

### @ai:pre (Preconditions)

Conditions that MUST be true before the function executes.
- Validate inputs against these when reviewing code
- Generate test cases that violate preconditions (expect errors)
- If preconditions are not enforced, flag as potential bug

**Format:** Comma-separated boolean expressions using parameter names.

**Special syntax:**
- `param != null` - Parameter must not be null/None/nil
- `param.len() > 0` - Collection must not be empty
- `param > 0` - Numeric constraint
- `param in [a, b, c]` - Value must be one of the listed options

### @ai:post (Postconditions)

Conditions that MUST be true after the function returns.
- `result` refers to the return value
- `old(x)` refers to the value of x before execution
- Use these to verify implementation correctness

**Format:** Comma-separated boolean expressions.

**Special syntax:**
- `result > 0` - Return value constraint
- `result.len() == old(input.len())` - Length preservation
- `result != null` - Non-null return guarantee
- `old(x) + old(y) == x + y` - Conservation invariant

### @ai:invariant

Conditions that MUST remain true throughout execution. Useful for loops and stateful operations.

**Format:** Boolean expression that holds at all times.

### @ai:example

Concrete input/output pairs. These are executable specifications.

**Format:** `(args) -> expected` where args match parameter order.

**Rules:**
- If implementation doesn't match examples, flag the discrepancy
- Use these as basis for generated unit tests
- Multiple examples can be provided on separate lines

**Examples:**
```
@ai:example (5, 3) -> 8
@ai:example (0, 0) -> 0
@ai:example (-1, 1) -> 0
```

### @ai:effects

Side effects the function may have. Critical for understanding function purity and composability.

**Valid values:**
- `pure` - No side effects (same input = same output, no external state changes)
- `io` - General I/O operations
- `db:read` - Reads from database
- `db:write` - Writes to database
- `network` - Makes network calls
- `fs:read` - Reads from file system
- `fs:write` - Writes to file system
- `env` - Reads environment variables
- `state:read` - Reads from shared/global state
- `state:write` - Modifies shared/global state
- `random` - Uses random number generation
- `time` - Depends on current time

**Multiple effects:** Comma-separated, e.g., `@ai:effects db:read, network`

### @ai:idempotent

Indicates whether the function is safe to call multiple times with the same arguments without changing the result beyond the first call.

**Format:** `true` or `false`

### @ai:confidence

A 0.0-1.0 score indicating trust in the implementation.

**Interpretation:**
- `< 0.5` - Low confidence, needs significant review
- `0.5-0.7` - Moderate confidence, review recommended
- `0.7-0.9` - Standard confidence, normal review
- `> 0.9` - High confidence, well-tested

### @ai:needs_review

Explicit flag that human review is required. Always include a reason.

**Format:** Free text explaining why review is needed.

**Common reasons:**
- Security-sensitive logic
- Complex business rules
- Edge cases not fully handled
- Performance implications

### @ai:author

Provenance tracking - who wrote this code.

**Format:**
- `claude-4` / `claude-opus-4` / `gpt-4` - AI model name
- `human:email@example.com` - Human author
- `claude-4, verified:human:alice@example.com` - AI-written, human-verified

### @ai:verified

Verification status indicating what validation has been performed.

**Format:**
- `human:email:YYYY-MM-DD` - Human reviewed on date
- `tests:passing` - All tests pass
- `tests:coverage:95%` - Test coverage level
- `formal:dafny` / `formal:verus` - Formal verification tool used

### @ai:assumes

Implicit assumptions not captured in preconditions. Documents things the code relies on but doesn't check.

**Format:** Free text describing assumptions.

**Examples:**
- Database connection is available
- User is authenticated
- Input is UTF-8 encoded

### @ai:context

Required context to understand the function. Points to related code or documentation.

**Format:** File paths, module names, or documentation links.

### @ai:related

Related functions that work together with this one.

**Format:** Function names, optionally with file paths.

### @ai:deprecated

Marks the function as deprecated with replacement information.

**Format:** Replacement function name or removal date.

### @ai:complexity

Time and/or space complexity.

**Format:** Big-O notation.

**Examples:**
- `O(n)`
- `O(n log n) time, O(n) space`
- `O(1) amortized`

### @ai:edge_cases

Known edge cases with their expected behavior.

**Format:** `condition -> behavior`

**Examples:**
```
@ai:edge_cases empty list -> returns empty list
@ai:edge_cases single element -> returns unchanged
@ai:edge_cases all duplicates -> returns single element
```

### @ai:override

Override a project-level constraint for this specific function.

**Format:** `@ai:override:<constraint_name> <value>`

**Example:**
```rust
/// @ai:intent Parse complex shader source with many edge cases
/// @ai:override:max_function_lines 80
fn parse_shader(source: &str) -> Result<Shader, ParseError> {
```

### @ai:test:integration

Marks a function as requiring integration test coverage. When `@ai:project:integration_tests` is true, functions with this annotation MUST have corresponding integration tests using the tools specified in `@ai:project:integration_tests_tools`.

Use this for code that:
- Interacts with external services (databases, APIs, file systems)
- Has side effects that can only be verified in a real environment
- Implements critical business workflows

**Format:** Optional description of what integration test should verify.

**Example:**
```rust
/// @ai:intent Process payment through external payment gateway
/// @ai:effects network, db:write
/// @ai:test:integration Verify payment is processed and recorded correctly
async fn process_payment(payment: Payment) -> Result<Receipt, PaymentError> {
```

---

## Writing AICMS Annotations

When generating or modifying code, ADD appropriate annotations following these guidelines:

### Required Annotations

1. **ALWAYS add `@ai:intent`** - Every function must have a clear purpose statement.
2. **ALWAYS add `@ai:module:intent`** - Every new module/file must have a purpose.

### Recommended Annotations

3. **Add `@ai:pre`** when:
   - Function has input validation requirements
   - Parameters have constraints (non-null, positive, valid range)
   - Function behavior is undefined for certain inputs

4. **Add `@ai:post`** when:
   - Function makes guarantees about return value
   - State is modified in predictable ways
   - Relationship exists between input and output

5. **Add `@ai:example`** - Include at least one concrete test case. More examples for:
   - Complex functions
   - Functions with multiple code paths
   - Edge case handling

6. **Add `@ai:effects`** when:
   - Function has ANY side effects (omit only for pure functions)
   - Function interacts with external systems

7. **Add `@ai:confidence`** reflecting your certainty:
   - Be honest about uncertainty
   - Lower confidence for complex logic, concurrency, security
   - Higher confidence for simple, well-understood patterns

8. **Add `@ai:needs_review`** for:
   - Security-sensitive code (auth, crypto, permissions)
   - Complex business logic
   - Performance-critical sections
   - Code with known limitations

### Annotation Placement

- **Project-level:** In `CLAUDE.md` under an AICMS section
- **Module-level:** At the top of the file in module documentation
- **Function-level:** Immediately before the function definition

---

## Code Generation Best Practices

When generating code, follow these principles to produce robust, portable, and maintainable implementations.

### Standard Library First

**ALWAYS prefer standard library implementations over external packages/crates/modules.**

Before adding any external dependency, ask:
1. Can this be done with the standard library?
2. Is the dependency truly necessary, or just convenient?
3. Does the benefit outweigh the cost of the additional dependency?

**Examples by language:**

| Need | Avoid | Prefer |
|------|-------|--------|
| UUIDs | External UUID library | Standard library random + formatting, or simple string IDs |
| JSON | External JSON library (when std has one) | Standard library JSON (Python: `json`, Go: `encoding/json`) |
| Date/Time | External date library | Standard library time (`chrono` → `std::time` in Rust, `datetime` in Python) |
| HTTP client | Heavy frameworks | Standard library HTTP (Go: `net/http`, Python: `urllib`) |
| String utils | External string libraries | Built-in string methods |

### Native Error Handling

**Use the language's native mechanism for returning results with possible errors.**

Every language has an idiomatic way to handle operations that can fail. Use these patterns:

| Language | Error Handling Pattern |
|----------|----------------------|
| Rust | `Result<T, E>` - Return `Ok(value)` or `Err(error)` |
| Go | `(T, error)` - Return value and error as tuple |
| Python | Raise exceptions, or return `Optional[T]` for expected failures |
| TypeScript | Return `T \| Error`, use `Result<T, E>` pattern, or throw exceptions |
| Java | Throw checked/unchecked exceptions, or return `Optional<T>` |
| C | Return error codes with output parameters |

**Rules:**
- Define custom error types that capture failure context
- Implement standard error traits/interfaces (e.g., `std::error::Error` in Rust, `error` interface in Go)
- Propagate errors with context rather than swallowing them
- Never use strings as error types when a structured error type is possible

### Self-Contained Code

**Generate code that compiles and runs without requiring external package installation when feasible.**

This means:
1. Avoid dependencies for simple functionality
2. Implement common utilities inline rather than importing
3. Use language primitives over framework conveniences
4. Generate complete, working code that can be tested immediately

**When dependencies ARE appropriate:**
- Complex cryptographic operations (don't roll your own crypto)
- Database drivers (these are inherently external)
- Protocol implementations (gRPC, protobuf)
- Functionality that would require >100 lines to implement correctly

### Minimal Dependency Principle

When external dependencies are truly needed:

1. **Choose wisely:** Pick well-maintained, focused libraries over large frameworks
2. **Version carefully:** Use stable versions, avoid pre-release
3. **Document why:** Add a comment explaining why the dependency is needed
4. **Consider alternatives:** List alternatives you considered and rejected

**Example (Rust):**
```rust
// Using serde for JSON because:
// - Standard library has no JSON support
// - Manual parsing would be error-prone and verbose
// - serde is the de-facto standard with excellent performance
use serde::{Deserialize, Serialize};
```

---

## Enforcement Behavior

### Project Constraint Enforcement

When generating code, ALWAYS check and enforce project-level constraints:

1. **Before writing a function:**
   - Will it exceed `max_function_lines`? → Split into smaller functions
   - Will it exceed `max_params`? → Create a params/config struct
   - Does it need external dependencies? → Create a trait/interface first

2. **Before writing a file:**
   - Will it exceed `max_file_lines`? → Split into multiple modules
   - Will it exceed `max_functions_per_file`? → Split by responsibility
   - Will it exceed `max_structs_per_module`? → Create submodules

3. **When adding dependencies:**
   - Is `require_interface_for_deps` true? → Define trait, accept trait as parameter
   - Does it violate `dependency_rule`? → Restructure or use dependency injection

### Self-Correction Protocol

When a constraint would be violated:

1. **Self-correct BEFORE presenting code:**
   ```
   // Instead of one 80-line function, split into:
   fn process_input(input: &Input) -> ProcessedData { ... }  // 25 lines
   fn validate_data(data: &ProcessedData) -> Result<()> { ... }  // 20 lines
   fn transform_output(data: ProcessedData) -> Output { ... }  // 20 lines
   ```

2. **Report the correction:**
   ```
   Note: Split `process_game_logic` (would be 75 lines) into 3 functions
   to respect @ai:project:max_function_lines (40):
   - process_input() - Parse and validate input
   - update_state() - Apply game logic
   - prepare_output() - Format results
   ```

3. **If constraint cannot be reasonably met:**
   ```
   WARNING: Function `parse_complex_grammar` requires 65 lines for
   correctness. Recommend either:
   1. Add @ai:override:max_function_lines 65 for this function
   2. Increase project max_function_lines to 65
   Current implementation uses the minimum viable line count.
   ```

### Interface Enforcement

When `@ai:project:require_interface_for_deps` is true:

**Wrong:**
```rust
pub struct GameEngine {
    renderer: WgpuRenderer,  // Concrete dependency - NOT ALLOWED
    audio: KiraAudioSystem,  // Concrete dependency - NOT ALLOWED
}
```

**Correct:**
```rust
pub struct GameEngine<R: Renderer, A: AudioSystem> {
    renderer: R,  // Trait dependency - mockable
    audio: A,     // Trait dependency - mockable
}

// Traits defined for dependencies
pub trait Renderer { ... }
pub trait AudioSystem { ... }
```

---

## Validation Behavior

When asked to review or modify annotated code, perform these checks:

### 1. Project Constraint Validation
- Does the code respect all `@ai:project:*` constraints?
- Are dependencies properly abstracted behind interfaces?
- Does the architecture follow the declared `@ai:project:architecture`?

### 2. Module Validation
- Does `@ai:module:intent` match what the module actually contains?
- Are all `@ai:module:public_api` items actually public?
- Do dependencies match `@ai:module:depends_on`?
- If `@ai:module:stateless` is true, verify no mutable statics

### 3. Intent Verification
- Does the implementation actually do what `@ai:intent` describes?
- Are there code paths that violate the intent?

### 4. Precondition Enforcement
- Are `@ai:pre` conditions enforced at the start of the function?
- Does the function fail gracefully when preconditions are violated?
- If preconditions are NOT enforced, flag this as: `WARNING: Precondition not enforced: {condition}`

### 5. Postcondition Verification
- Do ALL code paths satisfy the `@ai:post` conditions?
- Are there edge cases where postconditions might fail?

### 6. Example Validation
- Mentally execute each `@ai:example` case
- If any example doesn't match the implementation, report: `DISCREPANCY: Example (args) -> expected, but implementation returns actual`

### 7. Effects Verification
- Does `@ai:effects` accurately describe ALL side effects?
- If a function marked `pure` has side effects, flag it
- If a function has undeclared side effects, flag it

### 8. Discrepancy Reporting
When ANY mismatch is found between annotations and implementation:
1. Report the specific discrepancy
2. Quote the annotation and the conflicting code
3. Suggest whether to fix the code or update the annotation
4. Do NOT silently ignore mismatches

---

## Language-Specific Formats

Use the native comment syntax for each language:

### Rust
```rust
//! @ai:module:intent Handle player movement and physics
//! @ai:module:layer domain
//! @ai:module:public_api MovementSystem, MovementState, MovementConfig
//! @ai:module:stateless true

/// @ai:intent Calculate the factorial of a number
/// @ai:pre n >= 0
/// @ai:post result >= 1
/// @ai:example (5) -> 120
/// @ai:effects pure
fn factorial(n: u64) -> u64 {
    // implementation
}
```

### Python
```python
# @ai:module:intent Handle player movement and physics
# @ai:module:layer domain
# @ai:module:public_api MovementSystem, MovementState, MovementConfig
# @ai:module:stateless true

# @ai:intent Calculate the factorial of a number
# @ai:pre n >= 0
# @ai:post result >= 1
# @ai:example (5) -> 120
# @ai:effects pure
def factorial(n: int) -> int:
    # implementation
```

### TypeScript / JavaScript
```typescript
/**
 * @ai:module:intent Handle player movement and physics
 * @ai:module:layer domain
 * @ai:module:public_api MovementSystem, MovementState, MovementConfig
 * @ai:module:stateless true
 */

/**
 * @ai:intent Calculate the factorial of a number
 * @ai:pre n >= 0
 * @ai:post result >= 1
 * @ai:example (5) -> 120
 * @ai:effects pure
 */
function factorial(n: number): number {
    // implementation
}
```

### Go
```go
// @ai:module:intent Handle player movement and physics
// @ai:module:layer domain
// @ai:module:public_api MovementSystem, MovementState, MovementConfig
// @ai:module:stateless true

// @ai:intent Calculate the factorial of a number
// @ai:pre n >= 0
// @ai:post result >= 1
// @ai:example (5) -> 120
// @ai:effects pure
func Factorial(n uint64) uint64 {
    // implementation
}
```

### Java
```java
/**
 * @ai:module:intent Handle player movement and physics
 * @ai:module:layer domain
 * @ai:module:public_api MovementSystem, MovementState, MovementConfig
 * @ai:module:stateless true
 */
package com.game.movement;

/**
 * @ai:intent Calculate the factorial of a number
 * @ai:pre n >= 0
 * @ai:post result >= 1
 * @ai:example (5) -> 120
 * @ai:effects pure
 */
public long factorial(int n) {
    // implementation
}
```

### C / C++
```c
// @ai:module:intent Handle player movement and physics
// @ai:module:layer domain
// @ai:module:public_api MovementSystem, MovementState, MovementConfig
// @ai:module:stateless true

// @ai:intent Calculate the factorial of a number
// @ai:pre n >= 0
// @ai:post result >= 1
// @ai:example (5) -> 120
// @ai:effects pure
uint64_t factorial(uint64_t n) {
    // implementation
}
```

---

## Multi-Line Annotations

For complex specifications, continue on subsequent lines:

```rust
/// @ai:intent Transfer funds between two accounts, ensuring atomicity
///            and logging the transaction for audit purposes
/// @ai:pre from.balance >= amount, amount > 0, from != to
/// @ai:post from.balance == old(from.balance) - amount,
///          to.balance == old(to.balance) + amount
/// @ai:effects db:write, state:write
/// @ai:needs_review Security-critical: verify permission checks
```

---

## Spec-First Development

When a function has annotations but no implementation (contains `todo!()`, `pass`, `throw new Error("Not implemented")`), treat the annotations as a specification to implement against:

1. Read all `@ai:*` annotations
2. Check project constraints before implementing
3. Generate implementation that satisfies all constraints
4. Verify implementation against `@ai:example` cases
5. Ensure implementation respects `@ai:project:*` limits
6. Set `@ai:confidence` based on complexity and test coverage
7. Add `@ai:needs_review` if any uncertainty exists
