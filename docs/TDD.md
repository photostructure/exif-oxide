# Test-Driven Development and Bug Fixing

## Mandatory Bug-Fixing Workflow

When a bug or defect is discovered, you **MUST** follow this exact sequence:

### 1. Create a Breaking Test

Write a test that reproduces the issue:

- Clearly isolate the problematic behavior
- Use minimal test data that triggers the bug
- Give it a descriptive name explaining what should work

### 2. Validate the Test Explodes

Run the test to confirm it fails for the exact reason you expect:

- Must fail due to the bug, not test setup issues
- Verify failure mode matches the reported issue, (and isn't exposing yet another bug, or making an invalid assertion)

### 3. Address the Bug

Fix the underlying issue:

- Follow "Trust ExifTool" principle - check ExifTool's implementation
- Make minimal changes addressing root cause
- Include comments referencing ExifTool source when applicable

### 4. Validate the Test Passes

Confirm the fix works:

- Test now passes completely
- Run full test suite (`cargo t`) to ensure no regressions

## Example Workflow

```bash
# 1. Create breaking test
cargo t test_gps_parsing_near_equator
# Should fail with specific error

# 2. Fix the bug in implementation

# 3. Validate success
cargo t test_gps_parsing_near_equator  # Should pass
cargo t                                # Check for regressions
```

## Test Design Principles

- **Isolation**: One test per issue, minimal test data
- **Clarity**: Descriptive names, comments explaining the issue
- **Reproducibility**: Consistent, deterministic test data

## Integration with ExifTool

When fixing bugs:

1. Research ExifTool's behavior for this case
2. Compare implementations to find divergence
3. Trust ExifTool's logic - don't "improve" on it
4. Use comparison tools to verify alignment

## Test Organization for Large Test Suites

As test files grow beyond manageable size, follow these Rust conventions:

### Splitting Large Test Files

When `tests.rs` becomes too large, create a `tests/` subdirectory with organized modules:

```rust
// In your main module file
#[cfg(test)]
mod tests {
    mod basic_tests;
    mod edge_cases;
    mod integration_scenarios;
    
    // Can still have some tests directly here
    use super::*;
    
    #[test]
    fn quick_smoke_test() { ... }
}
```

Create separate test files:
- `tests/basic_tests.rs` - Core functionality tests
- `tests/edge_cases.rs` - Boundary conditions and error cases
- `tests/integration_scenarios.rs` - Complex multi-component tests

Each test file accesses the parent module:
```rust
use super::super::*; // Access parent module

#[test]
fn test_specific_functionality() { ... }
```

### Feature-Based Organization

Group tests by functionality being tested:

```rust
#[cfg(test)]
mod tests {
    mod parsing_tests;
    mod validation_tests;
    mod conversion_tests;
    mod error_handling_tests;
}
```

### Naming Conventions

- **Module names**: Use descriptive names indicating what's being tested
- **Test functions**: Prefix with `test_` and use snake_case
- **Organization**: One test file per major feature for maintainability
- **Shared utilities**: Create a `common` module for shared test helpers

### Best Practices

1. **Keep unit tests close to code** - Use `#[cfg(test)]` modules in the same file as implementation
2. **Feature-based grouping** - Organize by functionality, not arbitrary size limits
3. **Integration tests separate** - Use top-level `tests/` directory for black-box testing
4. **Maintain test isolation** - Each test should be independent and deterministic
