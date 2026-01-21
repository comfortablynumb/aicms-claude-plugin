# AICMS Semantic Diff

Detect breaking contract changes between two versions of a file or between staged changes and the current version.

## Instructions

When the user runs this command:

1. **Identify the comparison**:
   - If a file path is provided, compare `git diff` of staged changes vs HEAD
   - If two file paths are provided, compare them directly
   - If `--branch <name>` is specified, compare current branch vs target branch

2. **Extract annotations from both versions**:
   - Parse `@ai:pre`, `@ai:post`, `@ai:invariant`, `@ai:effects`, `@ai:idempotent`
   - Track function signatures as well

3. **Classify changes** using these categories:

   ### BREAKING Changes (🔴)
   - `@ai:pre` STRENGTHENED (new preconditions added, existing ones made stricter)
   - `@ai:post` WEAKENED (postconditions removed or made less strict)
   - `@ai:effects` EXPANDED (new side effects added)
   - `@ai:idempotent` changed from `true` to `false`
   - Function signature changed incompatibly

   ### NON-BREAKING Changes (🟢)
   - `@ai:pre` WEAKENED (preconditions relaxed - accepts more inputs)
   - `@ai:post` STRENGTHENED (stronger guarantees added)
   - `@ai:effects` REDUCED (fewer side effects)
   - `@ai:idempotent` changed from `false` to `true`

   ### NOTABLE Changes (🟡)
   - `@ai:intent` modified
   - `@ai:complexity` changed
   - `@ai:confidence` changed significantly
   - New `@ai:needs_review` added
   - `@ai:deprecated` added

4. **Output format**:

```
AICMS Semantic Diff: src/payment.rs

🔴 BREAKING CHANGES
  process_payment():
    - @ai:pre STRENGTHENED: "amount > 0" → "amount > 0 && amount < 10000"
      ⚠️ Callers may now fail with amounts >= 10000

    - @ai:effects EXPANDED: "db:write" → "db:write, network"
      ⚠️ Function now makes network calls

🟡 NOTABLE CHANGES
  validate_card():
    - @ai:intent MODIFIED:
      Old: "Validate credit card number"
      New: "Validate credit card number using Luhn algorithm"

    - @ai:confidence DECREASED: 0.95 → 0.75
      ⚠️ Consider reviewing implementation

🟢 NON-BREAKING CHANGES
  calculate_fee():
    - @ai:pre WEAKENED: "amount > 100" → "amount > 0"
      ✓ Function now accepts more inputs (backwards compatible)

Summary: 2 breaking, 2 notable, 1 non-breaking changes
```

## Change Detection Rules

### Precondition Changes (`@ai:pre`)
- **STRENGTHENED (BREAKING)**: New conditions added OR existing conditions made stricter
  - `amount > 0` → `amount > 0 && amount < 1000` (stricter)
  - No precondition → `user.is_authenticated()` (new requirement)
- **WEAKENED (OK)**: Conditions removed OR made less strict
  - `amount > 100` → `amount > 0` (accepts more)
  - `user.is_admin()` → removed (no longer required)

### Postcondition Changes (`@ai:post`)
- **WEAKENED (BREAKING)**: Conditions removed OR made less strict
  - `result.is_some()` → removed (no longer guaranteed)
  - `balance >= old(balance)` → `balance >= 0` (weaker guarantee)
- **STRENGTHENED (OK)**: Conditions added OR made stricter
  - No postcondition → `result >= 0` (new guarantee)

### Effect Changes (`@ai:effects`)
- **EXPANDED (BREAKING)**: New effects added
  - `pure` → `db:read` (now has side effects)
  - `db:read` → `db:read, network` (more effects)
- **REDUCED (OK)**: Effects removed
  - `network, db:write` → `db:write` (fewer effects)

## Usage Examples

```bash
# Compare staged changes vs HEAD
/aicms-diff src/payment.rs

# Compare two versions directly
/aicms-diff old_version.rs new_version.rs

# Compare branch vs main
/aicms-diff src/auth.rs --branch main

# Check all changed files in PR
/aicms-diff --all-staged
```

## Integration with PR Reviews

When reviewing a PR, use this command to:
1. Identify breaking contract changes early
2. Ensure contract changes are documented in PR description
3. Flag changes that need migration guides
4. Verify backwards compatibility claims
