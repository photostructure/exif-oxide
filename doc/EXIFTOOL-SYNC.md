# ExifTool Synchronization Guide

This guide documents how exif-oxide tracks knowledge from Phil Harvey's ExifTool and incorporates updates as ExifTool evolves.

**PLEASE NOTE**: this is a first-draft proposal. As we incorporate more of ExifTool's features, data, heuristics, and algorithms, we almost certainly will need to incrementally improve this document and tooling. Please keep this up to date!

**ALSO NOTE**: All the existing files in this project don't comply with this document yet -- we're working on it!

## Overview

exif-oxide leverages 25+ years of camera-specific knowledge from Phil Harvey's ExifTool while maintaining idiomatic Rust code. To efficiently incorporate ExifTool's ongoing updates, we use a simple source tracking system that makes it straightforward to identify which Rust files need updating when ExifTool changes.

## Source Tracking

### Module-Level Attribution

Each Rust file that implements ExifTool functionality includes doc attributes at the top:

```rust
//! Canon maker note parsing implementation

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Canon.pm"]
#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/CanonRaw.pm"]

// rest of implementation...
```

For auto-generated files:

```rust
// AUTO-GENERATED from ExifTool v12.65
// Source: lib/Image/ExifTool/Canon.pm (Main, PreviewImageInfo tables)
// Generated: 2024-12-22 by build.rs
// DO NOT EDIT - Regenerate with `cargo build`
```

### Why This Approach?

1. **Simple** - Just doc attributes at the top of each file
2. **Greppable** - Easy to find all files affected by a Perl module change
3. **Parseable** - Can be extracted programmatically when needed
4. **Self-documenting** - Shows up in rustdoc
5. **No separate files** - Attribution travels with the code

## Version Tracking

### Configuration File

The `exiftool-sync.toml` file tracks our synchronization status:

```toml
[exiftool]
version = "12.65"
git_commit = "e7d4a3b2c91f8e5d6a4b3c2d1e0f9a8b7c6d5e4f"
last_sync = "2024-12-22"
source_path = "../exiftool"

[sync_status]
fully_incorporated = ["12.60", "12.61", "12.62", "12.63"]
partially_incorporated = { "12.64" = ["Canon CR3", "Nikon Z9"] }
pending = ["12.65"]

[module_versions]
"src/maker/canon" = "12.63"
"src/maker/nikon" = "12.62"
"src/xmp" = "12.65"

# NEW: Algorithm extraction tracking
[extraction]
magic_numbers = { source = "ExifTool.pm:912-1027", last_extracted = "12.65", hash = "sha256:..." }
binary_formats = { source = "multiple", last_extracted = "12.64" }
datetime_patterns = { source = "multiple", last_extracted = "12.63" }
charset_mappings = { source = "ExifTool.pm:1056-1081", last_extracted = "12.65" }

[algorithm_tracking]
"ProcessBinaryData" = { version = "12.65", hash = "sha256:...", modules = ["ExifTool.pm"] }
"DateTime" = { version = "12.63", modules = ["ExifTool.pm", "Nikon.pm", "Canon.pm"] }
"MagicNumbers" = { version = "12.65", modules = ["ExifTool.pm"] }
```

## Synchronization Workflow

### When ExifTool Updates

1. **Run the sync tool**:
   ```bash
   cargo run --bin exiftool_sync diff 12.65 12.66
   ```

2. **See what needs attention**:
   ```
   CHANGED FILES WITH IMPLEMENTATIONS:
   Canon.pm (3 changes) → impacts:
     - src/maker/canon.rs
     - src/tables/canon.rs [AUTO-GENERATED]
   
   Exif.pm (1 new tag) → impacts:
     - src/tables/exif.rs [AUTO-GENERATED]
   
   CHANGED FILES WITHOUT IMPLEMENTATIONS:
   PDF.pm (12 changes) → no impact (not implemented)
   QuickTime.pm (5 changes) → no impact (not implemented)
   ```

3. **Take action**:
   - For `[AUTO-GENERATED]` files: Run `cargo build` to regenerate
   - For manual implementations: Review the Perl diff and update accordingly
   - Add a simple version note if the change is significant:
     ```rust
     // v12.66: Fixed Canon CR3 preview extraction
     ```

4. **Update version tracking**:
   ```toml
   [exiftool]
   version = "12.66"
   last_sync = "2024-12-23"
   ```

### Adding New ExifTool Features

1. **Check ExifTool source** for the feature you want to implement
2. **Add doc attribute** to your Rust file:
   ```rust
   #![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/NewFeature.pm"]
   ```
3. **Implement the feature** in idiomatic Rust
4. **Test against ExifTool** output for compatibility

## Testing Standards

### ExifTool Compatibility Tests

Validate against ExifTool output:

```rust
#[test]
fn test_canon_preview_extraction() {
    // Test against known ExifTool output
    let image = include_bytes!("../test-images/canon/Canon_T3i.JPG");
    let preview = extract_preview(image).unwrap();
    
    // These values come from ExifTool v12.65 output
    assert_eq!(preview.len(), 123456);
}
```

For systematic validation:

```bash
# Generate ExifTool output for comparison
exiftool -json -b test.jpg > expected.json

# Run our implementation
cargo test test_compatibility
```

## Code Review Checklist

When reviewing PRs with ExifTool-derived code:

- [ ] Doc attributes present at top of file listing ExifTool sources
- [ ] Tests validate against ExifTool output
- [ ] Version updated in `exiftool-sync.toml` if syncing changes
- [ ] Generated code includes version headers

## Best Practices

### DO:
- Add `EXIFTOOL-SOURCE` doc attributes when implementing ExifTool features
- Test output against ExifTool for compatibility
- Document significant deviations from ExifTool behavior
- Run sync tool when updating to new ExifTool versions
- Keep the implementation idiomatic Rust

### DON'T:
- Copy ExifTool code verbatim without understanding
- Implement features without checking ExifTool first
- Skip compatibility testing
- Over-complicate attribution (keep it simple)

## Directory Structure

```
exif-oxide/
├── exiftool-sync.toml        # Simple version tracking
├── exiftool/                 # Git submodule of ExifTool
└── src/
    ├── maker/
    │   ├── canon.rs          # Has #![doc = "EXIFTOOL-SOURCE: ..."] at top
    │   └── nikon.rs          # Has #![doc = "EXIFTOOL-SOURCE: ..."] at top
    ├── tables/               # AUTO-GENERATED files
    └── bin/
        └── exiftool_sync.rs  # Sync tooling that finds impacts
```

## Updating ExifTool

To update the ExifTool reference:

```bash
# Update ExifTool submodule
cd exiftool
git fetch
git checkout v12.66
cd ..

# Check what needs updating
cargo run --bin exiftool_sync diff 12.65 12.66

# Regenerate tables
cargo build  # Triggers build.rs

# Run compatibility tests
cargo test --features exiftool-compat
```

## Algorithm Extraction Guide

### Magic Number Extraction

To extract ExifTool's file type detection patterns:

```bash
# Extract magic numbers and generate Rust code
cargo run --bin exiftool_sync extract magic-numbers

# This parses %magicNumber from ExifTool.pm and generates:
# - src/tables/magic_numbers.rs
# - src/detection/patterns.rs
```

The extraction process:
1. Parses Perl regex patterns from `%magicNumber` hash
2. Converts to Rust-compatible patterns
3. Preserves weak detection flags and test lengths
4. Generates lookup tables with proper attribution

### ProcessBinaryData Extraction

ExifTool's binary data parsing is spread across many modules:

```bash
# Scan all modules for binary data formats
cargo run --bin exiftool_sync extract binary-formats

# Generates:
# - src/binary/formats/*.rs (format-specific processors)
# - src/binary/tables.rs (lookup tables)
```

Key challenges addressed:
- Variable-length formats (pstring, var_int32u)
- Negative indices (from end of data)
- Conditional processing based on other tag values
- Model-specific variations

### DateTime Pattern Extraction

DateTime handling involves multiple sources:

```bash
# Extract datetime patterns and quirks
cargo run --bin exiftool_sync extract datetime-patterns

# Scans for:
# - Date format patterns
# - Timezone handling
# - Manufacturer quirks (Nikon DST, etc.)
# - GPS timezone inference
```

### Tracking Algorithm Changes

When ExifTool updates affect core algorithms:

1. **Identify Changes**: 
   ```bash
   cargo run --bin exiftool_sync diff 12.65 12.66 --algorithm ProcessBinaryData
   ```

2. **Update Hash References**:
   - The tool calculates SHA256 of algorithm implementations
   - Detects when core logic changes between versions

3. **Re-extract if Needed**:
   ```bash
   cargo run --bin exiftool_sync extract --force --component binary-formats
   ```

## Questions?

For questions about the sync process:
1. Check existing module `EXIFTOOL_ATTRIBUTION.md` files for examples
2. Run `cargo run --bin exiftool_sync help` for tool documentation
3. Review recent PRs that incorporated ExifTool updates
4. See `doc/SPIKES-20250623.md` for detailed extraction plans

Remember: ExifTool represents 25+ years of camera-specific knowledge. Proper attribution ensures we can continue benefiting from Phil Harvey's ongoing work while building our Rust implementation.