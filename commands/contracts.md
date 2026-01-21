# AICMS Runtime Contract Validation

Generate runtime assertion wrappers for functions with @ai:pre and @ai:post annotations.

## Instructions

When the user runs this command on a file or function:

1. **Parse the file** to find functions with `@ai:pre` and/or `@ai:post` annotations

2. **For each annotated function**, generate a wrapper that:
   - Checks `@ai:pre` conditions before calling the original function
   - Captures `old()` values for postcondition checks
   - Calls the original function
   - Checks `@ai:post` conditions after the function returns
   - In Rust: wrap with `#[cfg(debug_assertions)]`
   - In other languages: use appropriate debug/development guards

3. **Generate the wrapper code** following language conventions:

### Rust Example

For this annotated function:
```rust
/// @ai:intent Transfer funds between accounts
/// @ai:pre from.balance >= amount
/// @ai:pre amount > 0
/// @ai:post from.balance + to.balance == old(from.balance + to.balance)
fn transfer(from: &mut Account, to: &mut Account, amount: u64) -> Result<(), Error> {
    // implementation
}
```

Generate this wrapper:
```rust
#[cfg(debug_assertions)]
pub fn transfer_checked(from: &mut Account, to: &mut Account, amount: u64) -> Result<(), Error> {
    // @ai:pre from.balance >= amount
    assert!(from.balance >= amount, "AICMS pre-condition failed: from.balance >= amount");
    // @ai:pre amount > 0
    assert!(amount > 0, "AICMS pre-condition failed: amount > 0");

    // Capture old values for postconditions
    let old_total = from.balance + to.balance;

    let result = transfer(from, to, amount);

    // @ai:post from.balance + to.balance == old(from.balance + to.balance)
    assert!(from.balance + to.balance == old_total, "AICMS post-condition failed: from.balance + to.balance == old(from.balance + to.balance)");

    result
}

#[cfg(not(debug_assertions))]
pub use transfer as transfer_checked;
```

### Python Example

```python
def transfer_checked(from_acc: Account, to_acc: Account, amount: int) -> None:
    """Wrapper with runtime contract validation (debug only)."""
    if __debug__:
        # @ai:pre from_acc.balance >= amount
        assert from_acc.balance >= amount, "AICMS pre-condition failed: from_acc.balance >= amount"
        # @ai:pre amount > 0
        assert amount > 0, "AICMS pre-condition failed: amount > 0"

        # Capture old values
        old_total = from_acc.balance + to_acc.balance

    transfer(from_acc, to_acc, amount)

    if __debug__:
        # @ai:post
        assert from_acc.balance + to_acc.balance == old_total, "AICMS post-condition failed"
```

### TypeScript Example

```typescript
function transferChecked(from: Account, to: Account, amount: number): void {
    if (process.env.NODE_ENV !== 'production') {
        // @ai:pre from.balance >= amount
        console.assert(from.balance >= amount, "AICMS pre-condition failed: from.balance >= amount");
        // @ai:pre amount > 0
        console.assert(amount > 0, "AICMS pre-condition failed: amount > 0");
    }

    const oldTotal = from.balance + to.balance;

    transfer(from, to, amount);

    if (process.env.NODE_ENV !== 'production') {
        // @ai:post
        console.assert(from.balance + to.balance === oldTotal, "AICMS post-condition failed");
    }
}
```

## Output Options

1. **Inline** - Add wrapper functions to the same file
2. **Separate file** - Create `*_contracts.rs` / `*_contracts.py` file
3. **Test file** - Generate as test helpers

## Usage

```
/project:aicms-contracts src/banking.rs
/project:aicms-contracts src/auth.py --output separate
/project:aicms-contracts src/payment.ts --inline
```
