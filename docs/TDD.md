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
