# Infer AICMS Annotations from Code

Analyze existing unannotated code and propose AICMS annotations based on the implementation.

## Instructions

1. **Identify unannotated functions**: Find functions without `@ai:intent` annotations in the target file(s).

2. **Analyze each function** and infer:

   - **@ai:intent**: Describe what the function does based on:
     - Function name
     - Parameter names and types
     - Return type
     - Implementation logic

   - **@ai:pre**: Infer preconditions from:
     - Explicit validation/guard clauses
     - Assertions at function start
     - Type constraints (non-null, non-empty)
     - Implicit assumptions from usage patterns

   - **@ai:post**: Infer postconditions from:
     - Return statements
     - Type guarantees
     - State modifications

   - **@ai:effects**: Detect side effects:
     - `pure` if no external state access
     - `db:read/write` for database operations
     - `network` for HTTP/network calls
     - `fs:read/write` for file operations
     - `env` for environment variable access

   - **@ai:example**: Generate example inputs/outputs if possible from:
     - Existing tests
     - Default/typical usage patterns
     - Boundary values

   - **@ai:complexity**: Analyze time/space complexity from:
     - Loop structures
     - Recursive calls
     - Data structure operations

3. **Set confidence levels**:
   - Add `@ai:confidence` based on inference certainty
   - Lower confidence (0.5-0.7) for complex logic
   - Higher confidence (0.8-0.9) for straightforward functions

4. **Flag uncertainty**: Add `@ai:needs_review Inferred from implementation, verify business logic` when:
   - Business logic is unclear
   - Multiple interpretations possible
   - Complex control flow

5. **Output format**: Present the inferred annotations in a diff-like format:

```diff
+ /// @ai:intent Process a payment transaction
+ /// @ai:pre amount > 0
+ /// @ai:pre account.is_active()
+ /// @ai:post account.balance == old(account.balance) - amount
+ /// @ai:effects db:write, network
+ /// @ai:confidence 0.75
+ /// @ai:needs_review Inferred from implementation, verify business logic
  fn process_payment(account: &mut Account, amount: u64) -> Result<(), Error> {
```

6. **Do NOT modify files directly**: Present proposed annotations for review. User decides whether to apply them.

## Usage

```
/aicms-infer <file-or-directory>
```

Arguments:
- `$ARGUMENTS` - Path to the file or directory to analyze
