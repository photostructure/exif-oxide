# Testing

## Test Images

Unfortunately, the test images in `$REPO_ROOT/third-party/exiftool/t/images` are
stripped of their data payload, so they test _many_ aspects of metadata parsing,
but whenever possible, we'd _prefer_ to test against full-size out-of-camera
example files. If you cannot find a full-sized example that your current task
demands in `$REPO_ROOT/test-images`, **ask the user** and we'll add it to the
repository.

Two test image directories serve different purposes:

1. **`test-images/`** - Full-size camera files with complete data payloads

   - Use these for development and feature testing
   - Real photos from actual cameras
   - Contains full metadata and image data

2. **`third-party/exiftool/t/images/`** - ExifTool test suite
   - Stripped of image data (metadata only)
   - Tests edge cases and problematic files
   - Use for compatibility testing

When implementing a manufacturer's support, test against both:

- Full files in `test-images/` for realistic testing
- Edge cases in `third-party/exiftool/t/images/` for robustness

## ExifTool Compatibility Testing

The `tests/exiftool_compatibility_tests.rs` provides automated validation against ExifTool's reference output:

### How It Works

1. **Reference Generation**: `tools/generate_exiftool_json.sh` creates snapshots of ExifTool's JSON output for all test images
2. **Comparison Testing**: Tests run exif-oxide against the same images and compare JSON outputs
3. **Normalization Layer**: Handles ExifTool's presentation inconsistencies without changing core parsing logic

### Normalization System

ExifTool has inconsistent PrintConv formatting across manufacturer modules. The test normalization layer standardizes these for API consistency:

**Examples:**

- **FocalLength**: `24` → `"24 mm"`, `1.8` → `"1.8 mm"`, `"24.0 mm"` → `"24 mm"`
- **ExposureTime**: Preserves ExifTool's varied formats - fractions stay strings (`"1/400"`), whole seconds stay numbers (`4`)
- **FNumber**: `14.0` → `14` (cleaning precision while preserving JSON number type)

### Adding New Normalization Rules

When you discover ExifTool inconsistencies, add rules to `get_normalization_rules()`:

```rust
// For unit-based tags
rules.insert("TagName", NormalizationRule::UnitFormat {
    unit: "mm",
    decimal_places: Some(1)
});

// For cleaning numeric precision
rules.insert("TagName", NormalizationRule::CleanNumericPrecision {
    max_places: 1
});
```

This approach follows the updated TRUST-EXIFTOOL principle: preserve core parsing logic while standardizing inconsistent presentation layers.

### Running Compatibility Tests

```bash
# Generate ExifTool reference snapshots
make compat-gen

# Run compatibility tests
make compat-test
```

## Integration Test Patterns

### Test Helper Organization

Following Rust's testing best practices, exif-oxide uses a structured approach for integration tests that need to access internal functionality:

#### Feature-Gated Test Helpers

For integration tests that need to simulate internal state (like setting up EXIF data without parsing files), we use feature-gated helper methods:

```rust
// In src/lib.rs - enabled only during testing
#[cfg(any(test, feature = "test-helpers"))]
pub fn add_test_tag(&mut self, tag_id: u16, value: TagValue, namespace: &str, ifd_name: &str) {
    // Implementation that accesses private fields
}
```

#### Shared Test Helper Module

Integration tests share common functionality through `tests/common/mod.rs`:

```rust
// tests/common/mod.rs
pub fn create_camera_test_reader() -> ExifReader {
    create_test_reader_with_tags(vec![
        (0x829a, TagValue::String("50".to_string()), "EXIF", "ExifIFD"), // FocalLength
        (0x829d, TagValue::String("2.8".to_string()), "EXIF", "ExifIFD"), // FNumber
        // ... more test data
    ])
}

// tests/integration_test.rs
mod common;

#[test]
fn test_composite_building() {
    let mut reader = common::create_camera_test_reader();
    reader.build_composite_tags();
    // ... test logic
}
```

#### Running Tests with Helper Features

The project's `Makefile` automatically includes the `test-helpers` feature:

```bash
# Automatically runs with test-helpers feature
make test

# Manual execution with feature flag
cargo test --features test-helpers
```

### Benefits of This Approach

1. **Security**: Test helpers are only available when explicitly enabled via feature flag
2. **Clean API**: No pollution of production API with test-only methods
3. **Reusable**: Shared helpers eliminate code duplication across integration tests
4. **Standard**: Follows [Rust Book Chapter 11.3](https://doc.rust-lang.org/book/ch11-03-test-organization.html) recommendations

### When to Use Each Pattern

- **Unit tests**: Test private methods directly using `use super::*;` pattern
- **Integration tests**: Use shared helper modules and feature-gated public methods
- **Compatibility tests**: Use real files from `test-images/` directory

See the [Rust Book's Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html) for more details on idiomatic Rust testing patterns.

## Module Structure and Test Organization

### Unit Test Module Structure

For unit tests within library modules, follow these patterns to avoid clippy issues with `cargo clippy --fix`:

#### ✅ Correct Pattern - `#[cfg(test)]` Imports

```rust
// src/my_module.rs
use std::collections::HashMap;

// Put test imports at module level with cfg(test)
#[cfg(test)]
use crate::types::{TagValue, ProcessorType};
#[cfg(test)]
use crate::tiff_types::ByteOrder;

pub struct MyStruct {
    // implementation
}

#[test]
fn test_my_function() {
    // test using the imports above
    let value = TagValue::String("test".to_string());
}
```

#### ❌ Problematic Pattern - Nested Test Module

```rust
// This causes clippy --fix to incorrectly remove imports (!!)
#[cfg(test)]
mod tests {
    use super::*;  // clippy can't always resolve this correctly
    use crate::types::TagValue;  // clippy may mark as unused

    #[test]
    fn test_something() {
        // tests here
    }
}
```

### The Clippy Import Analysis Issue

When structuring test modules in Rust, you may encounter a situation where `cargo clippy --fix` incorrectly removes imports that are actually used in tests. This happens due to inconsistencies in how clippy analyzes imports across different compilation targets.

#### Root Cause

The issue occurs because:

1. **Different Analysis Contexts**: `cargo clippy` (regular linting) uses different target analysis than `cargo clippy --fix` (which includes `--all-targets` by default)
2. **Test Module Resolution**: When tests are in separate modules or files, clippy may not properly connect import usage across module boundaries
3. **Compilation Target Mismatch**: Test code is only compiled with `#[cfg(test)]`, but clippy's fix mode may analyze imports outside this context

#### Symptoms

- Clippy reports imports as "unused" but removing them causes compilation errors
- `cargo clippy --fix` removes imports that are clearly used in test functions
- Tests pass individually but fail after running clippy fix

#### Resolution Strategy

1. **Use Module-Level `#[cfg(test)]` Imports**:

   ```rust
   // Instead of nested test modules, use cfg-gated imports
   #[cfg(test)]
   use crate::types::TagValue;

   #[test]
   fn my_test() {
       let value = TagValue::String("test".to_string());
   }
   ```

2. **Avoid Nested Test Modules in Separate Files**:

   ```rust
   // In tests.rs - avoid this pattern:
   #[cfg(test)]
   mod tests {  // <- This can confuse clippy
       use super::*;
       // tests
   }

   // Use this instead:
   #[cfg(test)]
   use crate::my_module::MyStruct;

   #[test]
   fn my_test() {
       // tests
   }
   ```

3. **Verify Import Usage Context**:
   - Ensure imports are actually used in the same compilation context where they're declared
   - Check that `#[cfg(test)]` is applied consistently
   - Avoid mixing test and non-test imports in the same use statement

#### Prevention

- Follow Rust's standard testing patterns from [The Rust Book Chapter 11.3](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- Use `#[cfg(test)]` on imports rather than wrapping entire test modules
- Test your build process with `make fix` after restructuring test code
- When in doubt, use `cargo clippy --all-targets` for consistent analysis

This issue highlights the importance of understanding how Rust's compilation targets work with testing and how tools like clippy analyze code across different contexts.

## Avoid mocks and byte array snippets

- Avoid mocks and stubs where possible.

- Whenever possible, use integration tests that load actual files from
  `$REPO_ROOT/test-images`.
