# Complete Development Guide for exif-oxide

**🚨 CRITICAL: This guide assumes you've read [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - the fundamental law of this project.**

This guide covers all aspects of day-to-day development, from initial implementation to testing and ExifTool updates. Every workflow described here follows the "Trust ExifTool" principle.

## Section 1: Daily Development Workflow

> **Foundation:** All development follows [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - we translate ExifTool exactly, never "improve" it.

### 1.1 The Extract-Generate-Implement Cycle

Development follows a demand-driven approach: only implement what real images actually need.

```
Extract → Generate → Discover → Implement → Validate
   ↑                                            ↓
   └────────────────────────────────────────────┘
```

#### Step 1: Extract Phase

Extract tag definitions from ExifTool source:

```bash
make codegen
```

This parses ExifTool's Perl modules and extracts:

- Tag definitions
- PrintConv/ValueConv references
- Processor specifications
- Format patterns

#### Step 2: Generate Phase

Run code generation to create Rust code:

```bash
cd codegen && cargo run --release
```

Expected output:

```
Generated: 823 mainstream tag definitions (from 15,234 total)
Simple conversions implemented: 156
Complex conversions referenced: 234 (no stubs generated)
Custom processors identified: 47

Code is ready to compile and run!
Use --show-missing on actual images to see what's needed.
```

Generated files in `src/generated/`:

- `tags.rs` - Tag table definitions
- `conversion_refs.rs` - Required conversion lists
- `supported_tags.rs` - Currently supported tags
- `composite_tags.rs` - Composite tag definitions

#### Step 3: Discover Missing Implementations

##### Using --show-missing

Run on actual test images to discover what's needed:

```bash
cargo run -- test-images/Canon/Canon_T3i.jpg --show-missing
```

Output:

```
Missing Implementations for Canon_T3i.jpg
=========================================
HIGH PRIORITY (blocks common tags):
- orientation_lookup (PrintConv)
  Used by: EXIF:Orientation

MEDIUM PRIORITY:
- canon_ev_format (PrintConv)
  Used by: Canon:ExposureCompensation
- canon_wb_lookup (PrintConv)
  Used by: Canon:WhiteBalance
```

##### Prioritizing Work

Focus on:

1. **HIGH PRIORITY**: Tags that appear in most images
2. **Test Coverage**: Implementations needed by test suite
3. **User Requests**: Specific tags requested by users

#### Step 4: Implement What's Needed

##### Find ExifTool Source

Locate the implementation in ExifTool:

```bash
# Search for PrintConv
grep -r "orientation_lookup\|Orientation.*PrintConv" third-party/exiftool/lib/

# Find in specific module
less third-party/exiftool/lib/Image/ExifTool/Exif.pm
/Orientation
```

##### Create Implementation

Add to appropriate file:

```rust
// In src/implementations/print_conv.rs

/// EXIF Orientation PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:4912
pub fn orientation_print_conv(val: &TagValue) -> String {
    match val {
        TagValue::Integer(1) => "Horizontal (normal)".to_string(),
        TagValue::Integer(2) => "Mirror horizontal".to_string(),
        TagValue::Integer(3) => "Rotate 180".to_string(),
        TagValue::Integer(4) => "Mirror vertical".to_string(),
        TagValue::Integer(5) => "Mirror horizontal and rotate 270 CW".to_string(),
        TagValue::Integer(6) => "Rotate 90 CW".to_string(),
        TagValue::Integer(7) => "Mirror horizontal and rotate 90 CW".to_string(),
        TagValue::Integer(8) => "Rotate 270 CW".to_string(),
        _ => format!("{}", val),
    }
}
```

##### Register Implementation

Add to registry:

```rust
// In registry initialization
registry.register_print_conv(
    "exif_orientation_lookup",
    orientation_print_conv
);
```

#### Step 5: Validate

##### Run Tests

```bash
# Unit tests
cargo test

# Integration tests
make test

# Compatibility tests
make compat
```

##### Compare with ExifTool

```bash
# Generate reference output
exiftool -j test-images/Canon/Canon_T3i.jpg > expected.json

# Generate our output
cargo run -- test-images/Canon/Canon_T3i.jpg > actual.json

# Compare
diff expected.json actual.json
```

##### Update Supported Tags

If implementation is complete:

1. Add to `MILESTONE_COMPLETIONS` in `codegen/src/main.rs`
2. Regenerate: `cargo run -p codegen`
3. Verify in compatibility tests

### 1.2 Common Development Tasks

#### Adding a PrintConv

1. Find in ExifTool source
2. Add function to `src/implementations/print_conv.rs`
3. Register in `init_print_conv_registry()`
4. Test against real images

#### Adding a ValueConv

1. Find in ExifTool source
2. Add function to `src/implementations/value_conv.rs`
3. Register in `init_value_conv_registry()`
4. Ensure PrintConv still works with converted value

#### Adding a Manufacturer Processor

1. Create new file: `src/implementations/{manufacturer}.rs`
2. Implement processor function
3. Add module to `src/implementations/mod.rs`
4. Register in appropriate registry

#### Using Generated Lookup Tables

Generated tables integrate seamlessly with manual functions:

```rust
// Generated: Canon white balance lookup
use crate::generated::Canon_pm::lookup_canon_white_balance;

// Manual: PrintConv function using generated table
pub fn canon_wb_print_conv(value: &TagValue) -> TagValue {
    if let Some(wb) = value.as_u8() {
        if let Some(name) = lookup_canon_white_balance(wb) {
            return TagValue::string(name);
        }
    }
    TagValue::string(format!("Unknown ({value})"))
}
```

#### Adding Simple Extraction Types

**Step 1: Add to Configuration**

```json
// In codegen/config/Canon_pm/simple_table.json
{
  "description": "Canon.pm simple lookup tables",
  "tables": [
    {
      "hash_name": "%newCanonTable",
      "constant_name": "NEW_CANON_SETTING",
      "key_type": "u8",
      "description": "Canon new setting names"
    }
  ]
}
```

**Step 2: Generate and Use**

```bash
# Regenerate code (auto-patches ExifTool modules)
make codegen

# Use in implementation
use crate::generated::Canon_pm::lookup_new_canon_setting;
```

**Note**: The build system automatically patches ExifTool modules to expose `my`-scoped variables as package variables based on entries in configuration files. No manual patching is required.

### 1.3 Debugging Tips

```bash
# Enable trace logging
RUST_LOG=trace cargo run -- test.jpg

# Use ExifTool verbose mode for comparison
exiftool -v3 test.jpg

# Check specific tag extraction
cargo run -- test.jpg | jq '.["EXIF:Orientation"]'
```

### 1.4 Best Practices

1. **Always Reference ExifTool**: Include file and line numbers
2. **Test Edge Cases**: Include 0, negative, and invalid values
3. **Match Behavior Exactly**: Don't "improve" ExifTool's logic per [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md)
4. **Document Quirks**: Explain any non-obvious behavior
5. **Use Real Images**: Test against actual camera files

## Section 2: Testing Strategy

> **Principle:** Test against real files and ExifTool's actual behavior. Follow [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) for compatibility.

### 2.1 Test Images

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

**Important:** If you cannot find a full-sized example that your current task demands in `test-images/`, **ask the user** and we'll add it to the repository.

### 2.2 ExifTool Compatibility Testing

The `tests/exiftool_compatibility_tests.rs` provides automated validation against ExifTool's reference output:

#### How It Works

1. **Reference Generation**: `tools/generate_exiftool_json.sh` creates snapshots of ExifTool's JSON output for test images (only generates missing files by default, use `--force` to regenerate all)
2. **Comparison Testing**: Tests run exif-oxide against the same images and compare JSON outputs
3. **Normalization Layer**: Handles ExifTool's presentation inconsistencies without changing core parsing logic

#### Normalization System

ExifTool has inconsistent PrintConv formatting across manufacturer modules. The test normalization layer standardizes these for API consistency:

**Examples:**

- **FocalLength**: `24` → `"24 mm"`, `1.8` → `"1.8 mm"`, `"24.0 mm"` → `"24 mm"`
- **ExposureTime**: Preserves ExifTool's varied formats - fractions stay strings (`"1/400"`), whole seconds stay numbers (`4`)
- **FNumber**: `14.0` → `14` (cleaning precision while preserving JSON number type)

#### Adding New Normalization Rules

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

#### Running Compatibility Tests

```bash
# Generate missing ExifTool reference snapshots (incremental)
make compat-gen

# Force regenerate all ExifTool reference snapshots
make compat-gen-force

# Run compatibility tests
make compat-test
```

### 2.3 Integration Test Patterns

#### Test Helper Organization

Following Rust's testing best practices, exif-oxide uses a structured approach for integration tests that need to access internal functionality:

##### Feature-Gated Test Helpers

For integration tests that need to simulate internal state (like setting up EXIF data without parsing files), we use feature-gated helper methods:

```rust
// In src/lib.rs - enabled only during testing
#[cfg(any(test, feature = "test-helpers"))]
pub fn add_test_tag(&mut self, tag_id: u16, value: TagValue, namespace: &str, ifd_name: &str) {
    // Implementation that accesses private fields
}
```

##### Shared Test Helper Module

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

##### Running Tests with Helper Features

The project's `Makefile` automatically includes the `test-helpers` feature:

```bash
# Automatically runs with test-helpers feature
make test

# Manual execution with feature flag
cargo test --features test-helpers
```

##### Benefits of This Approach

1. **Security**: Test helpers are only available when explicitly enabled via feature flag
2. **Clean API**: No pollution of production API with test-only methods
3. **Reusable**: Shared helpers eliminate code duplication across integration tests
4. **Standard**: Follows [Rust Book Chapter 11.3](https://doc.rust-lang.org/book/ch11-03-test-organization.html) recommendations

##### When to Use Each Pattern

- **Unit tests**: Test private methods directly using `use super::*;` pattern
- **Integration tests**: Use shared helper modules and feature-gated public methods
- **Compatibility tests**: Use real files from `test-images/` directory

See the [Rust Book's Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html) for more details on idiomatic Rust testing patterns.

### 2.4 Module Structure and Test Organization

#### Unit Test Module Structure

For unit tests within library modules, follow these patterns to avoid clippy issues with `cargo clippy --fix`:

##### ✅ Correct Pattern - `#[cfg(test)]` Imports

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

##### ❌ Problematic Pattern - Nested Test Module

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

#### The Clippy Import Analysis Issue

When structuring test modules in Rust, you may encounter a situation where `cargo clippy --fix` incorrectly removes imports that are actually used in tests. This happens due to inconsistencies in how clippy analyzes imports across different compilation targets.

##### Root Cause

The issue occurs because:

1. **Different Analysis Contexts**: `cargo clippy` (regular linting) uses different target analysis than `cargo clippy --fix` (which includes `--all-targets` by default)
2. **Test Module Resolution**: When tests are in separate modules or files, clippy may not properly connect import usage across module boundaries
3. **Compilation Target Mismatch**: Test code is only compiled with `#[cfg(test)]`, but clippy's fix mode may analyze imports outside this context

##### Resolution Strategy

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

### 2.5 Avoid mocks and byte array snippets

- Avoid mocks and stubs where possible.
- Whenever possible, use integration tests that load actual files from `test-images/`.

## Section 3: ExifTool Update Workflow

> **Commitment:** We stay current with ExifTool's monthly releases to maintain compatibility per [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md).

### 3.1 Overview

ExifTool releases monthly updates with new parsers, file types, and bugfixes. This workflow ensures exif-oxide stays current with minimal manual effort.

### 3.2 Prerequisites

- ExifTool submodule at `third-party/exiftool/`
- Perl installed for extraction script
- Rust toolchain for building

### 3.3 Update Process

#### Step 1: Update ExifTool Submodule

```bash
cd third-party/exiftool
git fetch origin
git checkout v12.77  # new version tag
cd ../..
```

#### Step 2: Regenerate Tag Definitions

```bash
# Extract updated tag definitions from ExifTool
perl codegen/extract_tables.pl > codegen/generated/tag_tables.json

# Run code generation
cargo run -p codegen
```

#### Step 3: Review Changes

The codegen output will show what's new:

```
New in ExifTool 12.77:
- 3 new mainstream tags requiring implementation
- 1 new Canon processor variant
- 47 non-mainstream tags (ignored)

Missing implementations (priority order):
1. canon_new_lens_type (PrintConv) - 15 test images
2. nikon_z9_af_mode (PrintConv) - 8 test images
3. ProcessCanonCR3 (Processor) - 5 test images
```

#### Step 4: Assess Implementation Requirements

##### For Minor Updates (tags only)

If the update only adds new tags within existing processors:
- No manual implementation needed
- Proceed directly to testing

##### For Major Updates (new processors/conversions)

If new processors or complex conversions are needed:
- Implement missing pieces manually
- Reference ExifTool source code
- Test against provided images

#### Step 5: Implement Missing Pieces

For each missing implementation:

1. **Locate in ExifTool source**
   ```bash
   # Search for the implementation
   grep -r "canon_new_lens_type" third-party/exiftool/lib/
   ```

2. **Port to Rust**
   - Add to appropriate file in `src/implementations/`
   - Include ExifTool source reference
   - Match behavior exactly per [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md)

3. **Register implementation**
   - Add to registry in implementation file
   - Update MILESTONE_COMPLETIONS if needed

#### Step 6: Update Supported Tags

If you've added new working implementations:

```bash
# Update MILESTONE_COMPLETIONS in codegen/src/main.rs
# Then regenerate to update supported tags list
cargo run -p codegen
```

#### Step 7: Test Compatibility

```bash
# Run full test suite
make test

# Run compatibility tests
make compat

# Test specific new features
cargo run -p exif-oxide -- test-images/canon/new_model.jpg
```

#### Step 8: Commit Changes

```bash
# Stage generated code
git add src/generated/

# Stage implementations (if any)
git add src/implementations/

# Update ExifTool submodule reference
git add third-party/exiftool

# Commit with descriptive message
git commit -m "feat: update to ExifTool v12.77

- Add support for Canon new lens type
- Add Nikon Z9 AF mode support
- Update generated tag definitions"
```

### 3.4 Common Scenarios

#### Scenario 1: Tags-Only Update

Most common case - ExifTool adds tags but no new logic:

```bash
# Update submodule
cd third-party/exiftool && git checkout v12.77 && cd ../..

# Regenerate
perl codegen/extract_tables.pl > codegen/generated/tag_tables.json
cargo run -p codegen

# Test and ship
make test
git add -A && git commit -m "chore: update to ExifTool v12.77"
```

#### Scenario 2: New Manufacturer Support

When ExifTool adds a new camera manufacturer:

1. Check if it's mainstream (>80% frequency)
2. If yes, implement core processors
3. Start with basic tag extraction
4. Add PrintConv/ValueConv as needed

#### Scenario 3: New File Format

When ExifTool adds support for a new file format:

1. Check if format is in scope (image/video metadata)
2. Implement file detection
3. Add basic processor
4. Iterate based on test files

### 3.5 Troubleshooting

#### Extraction Script Fails

```bash
# Ensure you're in the right directory
pwd  # Should show /path/to/exif-oxide

# Check Perl modules
perl -e 'use lib "third-party/exiftool/lib"; use Image::ExifTool; print "OK\n"'
```

#### Codegen Errors

```bash
# Clean and rebuild
rm -rf codegen/target
cargo clean -p codegen
cargo build -p codegen
```

#### Test Failures

```bash
# Run specific test
cargo test canon_new_lens --no-fail-fast -- --nocapture

# Compare with ExifTool
exiftool -j test.jpg > expected.json
cargo run -- test.jpg > actual.json
diff expected.json actual.json
```

### 3.6 Best Practices

1. **Review ExifTool Changes**: Read ExifTool's Changes file for breaking changes
2. **Test Incrementally**: Test each new feature separately
3. **Document Issues**: Note any problematic tags in code comments
4. **Prioritize Mainstream**: Focus on tags with >80% usage frequency

## Section 4: Code Style & Best Practices

> **Foundation:** All code style decisions must preserve ExifTool compatibility per [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md).

### 4.1 Performance Considerations

- **Lazy Implementation**: Only implement what's actually used
- **Batch Discovery**: Run --show-missing on multiple images
- **Profile First**: Don't optimize without measurement

### 4.2 Generator Stubs (Future Feature)

Generate implementation stubs for specific images:

```bash
cargo run -p exif-oxide -- --generate-stubs Canon_T3i.jpg
```

Creates: `implementations/stubs/canon_t3i_stubs.rs` with skeleton functions.

### 4.3 Code Maintenance Practices

#### Watch for Codegen Opportunities

When reviewing or writing code, be vigilant for manually-maintained lookup tables that should be generated:

- **Red flag**: Any match statement or HashMap with >5 static entries mapping to strings
- **Red flag**: Hardcoded camera/lens names, white balance modes, or other manufacturer settings
- **Action**: Check if it came from ExifTool source (usually `%hashName = (...)`)
- **Solution**: Use the simple table extraction framework (see [CODEGEN.md](../CODEGEN.md))

Remember: Every manually-ported lookup table becomes a maintenance burden with monthly ExifTool updates.

#### File Size Guidelines

Keep source files under 500 lines for better maintainability:

- Files >500 lines should be refactored into focused modules
- The Read tool truncates at 2000 lines, hindering code analysis
- Smaller files improve code organization and tool effectiveness

### 4.4 Essential Reminders

- ExifTool compatibility is the #1 priority
- Don't innovate, translate
- Every quirk has a reason
- Test against real images
- Document ExifTool source references

## Quick Command Reference

### Generation
```bash
make codegen              # Full pipeline with schema validation
make -j4 codegen         # Parallel execution
cd codegen && cargo run --release  # Direct code generation
make check-schemas       # Validate configuration files
```

### Development
```bash
cargo run -- image.jpg --show-missing  # Find missing implementations
cargo run -- image.jpg > actual.json   # Test output
exiftool -j image.jpg > expected.json  # Reference output
```

### Testing
```bash
cargo test               # Full test suite
make compat-test        # ExifTool compatibility
make precommit          # Full validation including schema checks
```

### Incremental
```bash
make regen-tags         # Regenerate tag tables only
make regen-extract       # Regenerate lookup tables only
make clean              # Clean all generated files
```

**Remember:** If it seems weird, it's probably correct. Cameras are weird. Follow [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) always.