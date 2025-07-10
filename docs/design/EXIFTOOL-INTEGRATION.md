# ExifTool Integration: Code Generation & Implementation

**🚨 CRITICAL: All integration follows [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - we translate ExifTool exactly, never innovate.**

This document describes the complete system for integrating ExifTool's metadata extraction capabilities into exif-oxide through automated code generation and manual implementations.

## Overview

The ExifTool integration system uses a hybrid approach combining automated code generation with targeted manual implementations:

### Generated Components

- **Tag tables** with runtime conversion references
- **Lookup tables** from manufacturer modules (Canon white balance, Nikon lenses, etc.)
- **File type detection** patterns and discriminated unions
- **Reference lists** showing what manual implementations are needed

### Manual Components

- **Conversion functions** (PrintConv/ValueConv) with complex logic
- **Manufacturer processors** for binary data parsing
- **Encryption/decryption** algorithms
- **Format handlers** for variable-length data structures

**Core Principle**: Generated code provides data and structure; manual code provides ExifTool-equivalent logic. Complex logic is never generated - it's faithfully translated from ExifTool source.

## System Architecture

```
ExifTool Source → Perl Extractors → JSON → Rust Codegen → Generated Code
                                                      ↓
                Manual Implementations ←← Runtime Registry ←← Function References
```

### Build System

```
codegen/
├── patch_exiftool_modules.pl # Auto-patches ExifTool for variable access
├── config/               # Source-file-based configuration
│   ├── Canon_pm/        # Canon.pm extractions
│   │   ├── simple_table.json
│   │   └── print_conv.json
│   ├── ExifTool_pm/     # ExifTool.pm extractions
│   │   ├── simple_table.json
│   │   ├── file_type_lookup.json
│   │   └── boolean_set.json
│   └── Nikon_pm/        # Nikon.pm extractions
│       ├── simple_table.json
│       └── print_conv.json
├── schemas/             # JSON schema validation
│   ├── simple_table.json
│   ├── file_type_lookup.json
│   └── boolean_set.json
├── extractors/          # Perl scripts extract from ExifTool
│   ├── simple_table.pl       # Lookup tables
│   ├── tag_tables.pl    # Tag definitions
│   └── file_type_lookup.pl # File detection
├── src/generators/      # Rust code generation
│   ├── lookup_tables/   # Lookup table generators
│   ├── file_detection/  # File type detection
│   └── data_sets/       # Boolean set generators
│   ├── tags.rs         # Tag table generator
│   └── composite_tags.rs # Composite tag generator
└── generated/          # Extracted JSON data
```

### Runtime System

```
src/
├── registry.rs                    # Function lookup registry
├── implementations/               # Manual conversion functions
│   ├── print_conv.rs             # PrintConv implementations
│   ├── value_conv.rs             # ValueConv implementations
│   └── [manufacturer]/          # Specialized processors
├── generated/                     # Generated lookup tables
│   ├── Canon_pm/                 # Canon.pm extractions
│   │   └── mod.rs               # All Canon tables (direct generation)
│   ├── ExifTool_pm/             # ExifTool.pm extractions
│   │   └── mod.rs               # File type detection tables
│   └── Nikon_pm/                # Nikon.pm extractions
│       └── mod.rs               # All Nikon tables (direct generation)
└── processor_registry/           # Advanced processor architecture
    ├── traits.rs                # BinaryDataProcessor trait
    └── capability.rs            # Capability assessment
```

## Daily Development Workflow

### 1. Adding New PrintConv/ValueConv Functions

**Step 1: Identify Need**

```bash
# Test on real images to see what's missing
cargo run -- photo.jpg --show-missing

# Output shows:
# Missing implementations:
# - orientation_print_conv (EXIF:Orientation)
# - canon_wb_lookup (Canon:WhiteBalance)
```

**Step 2: Find ExifTool Source**

```perl
# Located in ExifTool source
%orientation = (
    1 => 'Horizontal (normal)',
    2 => 'Mirror horizontal',
    3 => 'Rotate 180',
    # ...
);
```

**Step 3: Implement Using Generated Tables**

```rust
/// EXIF Orientation PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:281-290 (%orientation hash)
/// Generated table: src/generated/Exif_pm/mod.rs
pub fn orientation_print_conv(val: &TagValue) -> TagValue {
    use crate::generated::Exif_pm::lookup_orientation;

    if let Some(orientation_val) = val.as_u8() {
        if let Some(description) = lookup_orientation(orientation_val) {
            return TagValue::string(description);
        }
    }
    TagValue::string(format!("Unknown ({val})"))
}
```

**Step 4: Register Function**

```rust
// In implementations/mod.rs
pub fn register_all_conversions() {
    registry::register_print_conv("orientation_print_conv", print_conv::orientation_print_conv);
}
```

**Step 5: Test**

```bash
# Verify against ExifTool
exiftool -j test.jpg > expected.json
cargo run -- test.jpg > actual.json
# Compare orientation values
```

### 2. Using Generated Lookup Tables

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

### 3. Adding Simple Extraction Types

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

## Code Generation System

### Direct Code Generation

The system generates simple, direct Rust code without macros:

```rust
// Generated lookup table (no macros)
pub static ORIENTATION: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(1, "Horizontal (normal)");
    map.insert(2, "Mirror horizontal");
    // ... more entries
    map
});

// Generated lookup function
pub fn lookup_orientation(key: u8) -> Option<&'static str> {
    ORIENTATION.get(&key).copied()
}
```

This approach prioritizes:

- **Readability**: Generated code looks like hand-written Rust
- **Debuggability**: Stack traces point to real code, not macro expansions
- **IDE Support**: Autocomplete and go-to-definition work perfectly
- **Simplicity**: No macro expertise needed to understand or modify

### Extraction Types

The system supports three extraction patterns:

#### Simple Tables

```perl
# Basic key-value lookups
%canonWhiteBalance = (
    0 => 'Auto',
    1 => 'Daylight',
    2 => 'Cloudy',
);
```

#### Regex Patterns

```perl
# File type magic numbers
%magicNumber = (
    JPEG => '\xff\xd8\xff',
    PNG  => '\x89PNG\r\n\x1a\n',
);
```

#### File Type Lookup

```perl
# Discriminated unions with aliases
%fileTypeLookup = (
    JPEG => { Description => 'JPEG image', Format => 'JPEG' },
    JPG  => 'JPEG',  # Alias
);
```

### Generated Code Benefits

- **Type Safety**: Proper Rust types for all keys
- **Performance**: LazyLock HashMap lookups with direct generation
- **Simplicity**: No macro complexity - generated code is plain Rust
- **Debugging**: Easy to read and debug generated functions
- **Traceability**: Every entry references ExifTool source
- **Maintenance**: Automatic updates with ExifTool releases
- **Integration**: Seamless use in manual functions via clean imports
- **Scalability**: Consolidated source-file-based organization

### Build Pipeline

The build pipeline automatically handles all necessary steps:

1. **Auto-patching**: Converts `my` variables to `our` in ExifTool modules
2. **Extraction**: Runs Perl extractors in parallel
3. **Generation**: Creates Rust code from extracted data

```bash
# Full pipeline with parallel execution
make codegen              # Full build (includes auto-patching and schema validation)
make -j4 codegen         # Parallel (4 jobs)

# Individual components
make extract             # Just lookup tables
make generated/tag_tables.json  # Just tag definitions
make check-schemas       # Validate all configuration files

# Incremental updates
make regen-extract       # Regenerate tables only
```

#### Why Patching is Required

ExifTool uses `my`-scoped lexical variables for many lookup tables (e.g., `my %canonWhiteBalance`). These variables are private to their module and cannot be accessed by external scripts. To extract these tables programmatically, we need to convert them to package variables (`our %canonWhiteBalance`) which are accessible via the symbol table.

**Auto-Patching Details**: The `patch_exiftool_modules.pl` script automatically:

- Reads `extract.json` to identify required variables
- Converts `my %varName` to `our %varName` in ExifTool modules
- Runs before extraction to ensure variables are accessible
- Tracks conversions to avoid redundant processing
- Only patches variables we actually need (based on `extract.json`)

## Manual Implementation System

### Runtime Registry

The system uses function-name based registration avoiding code generation overhead:

```rust
// Zero-cost function lookup
static GLOBAL_REGISTRY: LazyLock<Arc<RwLock<Registry>>> =
    LazyLock::new(|| Arc::new(RwLock::new(Registry::new())));

// O(1) dispatch
pub fn apply_print_conv(fn_name: &str, value: &TagValue) -> TagValue {
    if let Some(func) = GLOBAL_REGISTRY.read().unwrap().get_print_conv(fn_name) {
        func(value)
    } else {
        TagValue::string(format!("Unknown ({value})"))  // Graceful fallback
    }
}
```

### Function Patterns

#### PrintConv: Logical → Display

```rust
pub fn flash_print_conv(val: &TagValue) -> TagValue {
    // Returns TagValue for precise JSON control
    if let Some(flash_val) = val.as_u16() {
        // Complex flash decode logic from ExifTool
        TagValue::string(format_flash_description(flash_val))
    } else {
        TagValue::string(format!("Unknown ({val})"))
    }
}
```

#### ValueConv: Raw → Logical

```rust
pub fn fnumber_value_conv(val: &TagValue) -> Result<TagValue> {
    // Returns Result for error handling
    match val {
        TagValue::Rational(num, den) if *den != 0 => {
            Ok(TagValue::F64(*num as f64 / *den as f64))
        }
        _ => Ok(val.clone()),
    }
}
```

### Manufacturer Processors

Complex manufacturer-specific processing uses modular architecture:

```rust
// Canon module structure
src/implementations/canon/
├── af_info.rs          # AF Info processing
├── binary_data.rs      # Binary data extraction
├── offset_schemes.rs   # Offset detection
├── tags.rs             # Tag name resolution
└── tiff_footer.rs      # TIFF footer handling

// Example: Variable-length processing
pub fn process_canon_af_info(
    exif_reader: &mut crate::exif::ExifReader,
    data: &[u8],
) -> Result<()> {
    // ExifTool: lib/Image/ExifTool/Canon.pm AFInfo processing
    let mut offset = 0;

    // Extract size for variable-length array
    let af_info_size = u16::from_be_bytes([data[offset], data[offset + 1]]);
    offset += 2;

    // Process based on size - exact ExifTool translation
    // ... complex parsing logic

    Ok(())
}
```

## Advanced Processor Architecture

For ExifTool's 121+ ProcessBinaryData variants, the system provides a trait-based architecture:

### Capability Assessment

```rust
pub trait BinaryDataProcessor {
    fn assess_capability(&self, context: &ProcessorContext) -> ProcessorCapability;
    fn process(&self, context: &mut ProcessorContext) -> Result<()>;
}

pub enum ProcessorCapability {
    Perfect,      // Exact match - use this processor
    Good,         // Compatible but not optimal
    Fallback,     // Can handle but suboptimal
    Incompatible, // Cannot process
}
```

### Context-Rich Processing

```rust
pub struct ProcessorContext {
    pub tag_table: String,
    pub processor_name: String,
    pub data: Vec<u8>,
    pub metadata: HashMap<String, String>,
    pub exif_reader: &mut ExifReader,
}

// Example processor
impl BinaryDataProcessor for CanonCameraSettingsProcessor {
    fn assess_capability(&self, context: &ProcessorContext) -> ProcessorCapability {
        if context.tag_table == "Canon::Main" &&
           context.processor_name == "ProcessBinaryData" {
            ProcessorCapability::Perfect
        } else {
            ProcessorCapability::Incompatible
        }
    }
}
```

## System Extension

### Adding New Generator Types

For new ExifTool data patterns (like XMP namespaces):

1. **Create Perl Extractor**: `extractors/xmp_namespaces.pl`
2. **Add Input Schema**: `schemas/input.rs`
3. **Create Generator**: `generators/xmp_namespaces.rs`
4. **Wire into Main**: `main.rs`

The modular architecture makes extension straightforward.

### Error Handling

The system uses `thiserror` for idiomatic error management:

```rust
#[derive(Error, Debug)]
pub enum ExifError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid file format: {0}")]
    InvalidFormat(String),

    #[error("Registry error: {0}")]
    Registry(String),
}

pub type Result<T> = std::result::Result<T, ExifError>;
```

## Maintenance & Updates

### ExifTool Version Updates

```bash
# Update submodule to new ExifTool version
cd third-party/exiftool && git checkout v12.XX

# Regenerate all code
make codegen

# Test compatibility
cargo test

# Most updates require no manual intervention
```

### Build System Capabilities

The modular build system supports efficient development:

```bash
# Parallel execution (faster development)
make -j4 codegen

# Individual extractors (for debugging)
make generated/tag_tables.json
make extract

# Incremental regeneration (for rapid iteration)
make regen-tags
make regen-extract

# Syntax checking
make check-extractors
```

## Complete Command Reference

### Generation

```bash
make codegen              # Full pipeline with schema validation
make -j4 codegen         # Parallel execution
make extract             # Just lookup tables
make generated/tag_tables.json  # Just tag definitions
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

## Performance Characteristics

### Generated Code

- **Zero Runtime Cost**: LazyLock static tables with HashMap lookups
- **Type Safety**: Compile-time validation of all keys and values
- **Memory Efficiency**: Shared string literals, no duplication

### Manual Implementation

- **O(1) Function Dispatch**: HashMap-based registry lookup
- **Minimal Overhead**: Direct function calls after lookup
- **Graceful Degradation**: Never panics on missing implementations

### Build System

- **Parallel Extraction**: Multiple Perl extractors run concurrently
- **Incremental Updates**: Only regenerate changed components
- **Fast Iteration**: Simple table changes rebuild in seconds

## Current Capabilities

- **50+ Conversion Functions**: Core EXIF, GPS, and manufacturer PrintConv/ValueConv
- **Canon Support**: AF info, binary data, offset schemes, TIFF footer processing
- **Nikon Support**: AF processing, encryption, lens database, IFD handling
- **Sony Support**: Basic manufacturer-specific processing
- **File Detection**: Magic number patterns, MIME types, extension lookup
- **Generated Integration**: Source-file-based organization with clean imports
- **Runtime Registry**: Zero-overhead function dispatch with graceful fallbacks
- **Scalable Architecture**: Direct code generation supporting 300+ lookup tables

## Related Documentation

- [API-DESIGN.md](API-DESIGN.md) - Public API structure and TagValue design
- [PROCESSOR-PROC-DISPATCH.md](../PROCESSOR-PROC-DISPATCH.md) - Advanced processor dispatch
- [STATE-MANAGEMENT.md](../STATE-MANAGEMENT.md) - State management during processing
- [ENGINEER-GUIDE.md](../ENGINEER-GUIDE.md) - Practical implementation guide
- [ARCHITECTURE.md](../ARCHITECTURE.md) - High-level system overview
