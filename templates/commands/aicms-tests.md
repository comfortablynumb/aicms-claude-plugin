# Generate Tests from AICMS Annotations

Generate unit tests for functions with AICMS annotations in the specified file or directory.

## Instructions

1. **Scan for annotated functions**: Find all functions with `@ai:*` annotations in the target file(s).

2. **Generate example-based tests**: For each `@ai:example` annotation:
   - Create a test case that calls the function with the specified inputs
   - Assert the return value matches the expected output
   - Handle floating-point comparisons with appropriate tolerance

3. **Generate edge case tests**: For each `@ai:edge_cases` annotation:
   - Create a test case for the described edge condition
   - Verify the expected behavior

4. **Generate property tests**: From `@ai:pre` and `@ai:post` annotations:
   - Generate property-based tests that verify preconditions are enforced
   - Generate tests that verify postconditions hold for valid inputs
   - Test that invalid inputs (violating preconditions) are handled appropriately

5. **Detect test framework**: Use the project's existing test framework:
   - **Rust**: `#[test]` or `#[tokio::test]` for async
   - **Python**: `pytest` or `unittest`
   - **TypeScript/JavaScript**: `jest`, `vitest`, or `mocha`
   - **Go**: standard `testing` package

6. **Output location**: Place tests in the appropriate test directory:
   - **Rust**: Same file with `#[cfg(test)]` module, or `tests/` directory
   - **Python**: `tests/` directory with `test_` prefix
   - **TypeScript**: `__tests__/` or `.test.ts` suffix

## Example Output

For a function like:
```rust
/// @ai:intent Calculate factorial
/// @ai:pre n >= 0
/// @ai:post result >= 1
/// @ai:example (5) -> 120
/// @ai:example (0) -> 1
fn factorial(n: u64) -> u64
```

Generate:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factorial_example_5() {
        assert_eq!(factorial(5), 120);
    }

    #[test]
    fn test_factorial_example_0() {
        assert_eq!(factorial(0), 1);
    }

    #[test]
    fn test_factorial_postcondition_result_ge_1() {
        for n in 0..=20 {
            assert!(factorial(n) >= 1, "postcondition failed: result >= 1 for n={}", n);
        }
    }
}
```

## Usage

```
/aicms-tests <file-or-directory>
```

Arguments:
- `$ARGUMENTS` - Path to the file or directory to generate tests for
