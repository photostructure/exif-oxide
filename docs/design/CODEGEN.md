# exif-oxide Code Generation

This document describes how engineers can use and extend the code generation system for exif-oxide.

## Overview

Codegen extracts metadata definitions from ExifTool's Perl modules and generates Rust code for:

- Tag table definitions with runtime conversion references
- Simple lookup table extraction from manufacturer modules
- Reference lists for required implementations
- Type-safe generated code with graceful fallbacks

**Core Principle**: Complex logic is NOT generated - it's manually implemented with ExifTool source references.

## Critical Rule: Only Perl Parses Perl

**Use `require`/`use` in Perl scripts. NEVER use regex to parse Perl code.**

The Perl interpreter is the only competent Perl parser. Any attempt to parse Perl with regex will be brittle and cause future maintenance nightmares.

## Build Pipeline

1. **ExifTool Source** → 2. **Perl Extractors** → 3. **JSON** → 4. **Rust Codegen** → 5. **Generated Code**

```bash
# Full codegen pipeline
make codegen
```

## What Codegen Handles

### 1. Tag Tables with Runtime References

```rust
// Generated: No function stubs, just references
Tag {
    id: 0x0112,
    name: "Orientation",
    print_conv: Some("orientation_print_conv"), // Runtime reference
    value_conv: None,
}

// Runtime: Graceful fallback when implementation missing
fn apply_print_conv(tag: &Tag, value: &TagValue) -> String {
    match registry.get_print_conv(conv_name) {
        Some(conv_fn) => conv_fn(value),
        None => format!("{:?}", value), // Never panic!
    }
}
```

### 2. Simple Table Extraction

Automatically extracts primitive lookup tables from ExifTool manufacturer modules:

```perl
// Safe to extract ✅
%canonWhiteBalance = (
    0 => 'Auto',
    1 => 'Daylight',
    2 => 'Cloudy',
);

// Never extract ❌ (this needs to be manually ported)
%complexTable = (
    condition => '$$self{Model} =~ /regex/',
    subdirectory => { complex => 'structure' },
);
```

### 3. Reference Lists

Generated lists show what implementations are needed:

```rust
pub static REQUIRED_PRINT_CONV: &[&str] = &[
    "orientation_print_conv",
    "flash_print_conv",
    "canon_white_balance_print_conv",
];
```

## What Codegen Does NOT Handle

1. **Complex Perl Logic**: Multi-line conditions, evals, complex math
2. **Encryption/Decryption**: All crypto is manually implemented
3. **Manufacturer Quirks**: Error recovery and special cases
4. **Binary Processing Hooks**: Custom data parsing logic

## Mainstream Tag Filtering

Only generates code for tags meeting these criteria:

- **Frequency > 80%**: Tags in >80% of images
- **Mainstream flag**: Marked `mainstream: true` in TagMetadata.json
- **Critical dependencies**: Required by other mainstream tags

This reduces scope from ~15,000 tags to ~500-1000 essential tags.

## Development Workflow

### 1. Run Codegen

```bash
# Extract and generate all code
make codegen

# Output shows what's needed
Generated: 823 mainstream tag definitions
Complex conversions referenced: 234 (no stubs generated)
```

### 2. Find Missing Implementations

```bash
# Test on real images to see what's needed
cargo run -- photo.jpg --show-missing

# Output
Missing implementations:
- orientation_print_conv (EXIF:Orientation)
- canon_wb_lookup (Canon:WhiteBalance)
```

### 3. Implement What's Needed

Implement only the conversion functions actually used by your test images. Reference ExifTool source and register in implementation palette.

### 4. Validate

```bash
cargo test  # Compares against ExifTool reference output
```

## Simple Table Extraction Framework

### Adding New Simple Tables

**Step 1**: Identify candidate table in ExifTool modules:

```perl
%newCanonTable = (
    0 => 'Setting A',
    1 => 'Setting B',
    2 => 'Setting C',
);
```

**Step 2**: Add to `codegen/simple_tables.json`:

```json
{
  "module": "Canon.pm",
  "hash_name": "%newCanonTable",
  "output_file": "canon/new_table.rs",
  "constant_name": "NEW_CANON_TABLE",
  "key_type": "u8",
  "description": "Canon new setting names"
}
```

**Step 3**: Run extraction:

```bash
cd codegen && perl extract_simple_tables.pl > generated/simple_tables.json
cd codegen && cargo run  # Generates Rust code
```

**Step 4**: Use in PrintConv implementation:

```rust
use crate::generated::canon::new_table::lookup_new_canon_table;

pub fn canon_new_setting_print_conv(value: &TagValue) -> Result<String> {
    if let Some(setting) = value.as_u8() {
        if let Some(name) = lookup_new_canon_table(setting) {
            return Ok(name.to_string());
        }
    }
    Ok(format!("Unknown ({})", value))
}
```

### Table Selection Guidelines

**Include**:

- Simple hash tables with primitive keys/values
- No Perl variables or expressions
- High-value data (lens databases, mode settings)
- Tables with >10 entries

**Exclude**:

- Any Perl expressions in keys/values
- Nested structures or references
- Conditional logic
- Tables with <5 entries (manual easier)

The extraction framework automatically validates and skips complex tables.

### Generated Code Benefits

- **Type Safety**: Proper Rust types for all keys
- **Performance**: Fast HashMap lookups with LazyLock
- **Traceability**: Every entry references ExifTool source line
- **Maintenance**: Automatic updates with ExifTool releases

## ExifTool Update Workflow

When ExifTool releases a new version:

```bash
# 1. Update submodule
cd third-party/exiftool
git checkout v12.XX  # new version

# 2. Regenerate code
make codegen

# 3. Check what's new
# Output shows: X new mainstream tags, Y new processors

# 4. Implement missing pieces (if any)
# 5. Test and ship
cargo test
```

For minor updates that only add tags within existing processors, it's often just regenerate and ship.

## Key Tools and Commands

```bash
# Full codegen pipeline
make codegen

# Simple tables only
cd codegen && perl extract_simple_tables.pl > generated/simple_tables.json

# Test on real images
cargo run -- image.jpg --show-missing

# Run compatibility tests
make compat-test
```

## Related Documentation

- [API-DESIGN.md](API-DESIGN.md) - Generated API structure
- [IMPLEMENTATION-PALETTE.md](IMPLEMENTATION-PALETTE.md) - Manual implementation registration
- [ENGINEER-GUIDE.md](../ENGINEER-GUIDE.md) - Practical implementation guide
- [MILESTONE-CODEGEN-SIMPLE-TABLES.md](../milestones/MILESTONE-CODEGEN-SIMPLE-TABLES.md) - Complete simple tables implementation details
