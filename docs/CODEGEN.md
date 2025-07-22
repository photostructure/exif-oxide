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

```mermaid
flowchart
    A[ExifTool Source] --> B[Perl Extractors]
    B --> C[JSON]
    C --> D[Rust Codegen]
    D --> E[Generated Code]
    E -->|Function References| F[Runtime Registry]
    F -->|Manual Implementations| G[Manual Implementations]
    G -.-> F
```

### Build System

```
codegen/
├── src/                 # Rust orchestration (simplified architecture)
│   ├── main.rs         # Command-line interface and coordination
│   ├── discovery.rs    # Auto-discovery of module directories
│   ├── config/         # Configuration management
│   ├── extraction.rs   # Calls Perl extractors with explicit arguments
│   ├── patching.rs     # Atomic ExifTool patching with tempfile
│   ├── file_operations.rs # Atomic file I/O operations
│   ├── table_processor.rs # Table processing and validation
│   └── generators/     # Rust code generation
│       ├── lookup_tables/ # Lookup table generators (modular output)
│       ├── file_detection/ # File type detection
│       ├── data_sets/   # Boolean set generators
│       ├── tags.rs     # Tag table generator (semantic grouping)
│       └── composite_tags.rs # Composite tag generator
├── config/             # Source-file-based configuration
│   ├── Canon_pm/      # Canon.pm extractions
│   │   └── simple_table.json
│   ├── ExifTool_pm/   # ExifTool.pm extractions
│   │   ├── simple_table.json
│   │   ├── file_type_lookup.json
│   │   ├── boolean_set.json
│   │   └── regex_patterns.json
│   ├── Nikon_pm/      # Nikon.pm extractions
│   │   ├── simple_table.json
│   │   └── print_conv.json
│   └── [Other]_pm/    # Auto-discovered modules
├── extractors/        # Simple Perl scripts (explicit arguments)
│   ├── simple_table.pl     # Takes file path + hash name
│   ├── tag_tables.pl       # Tag definitions
│   ├── file_type_lookup.pl # File detection
│   ├── boolean_set.pl      # Boolean sets
│   └── regex_patterns.pl   # Pattern extraction
├── schemas/           # JSON schema validation
└── generated/         # Temporary extraction data (gitignored)
```

### Runtime System

```
src/
├── registry.rs                    # Function lookup registry
├── implementations/               # Manual conversion functions
│   ├── print_conv.rs             # PrintConv implementations
│   ├── value_conv.rs             # ValueConv implementations
│   └── [manufacturer]/          # Specialized processors
├── generated/                     # Generated lookup tables (modular structure)
│   ├── tags/                     # Semantic tag grouping (8 focused modules)
│   │   ├── core.rs               # Core EXIF tags (1,256 lines)
│   │   ├── camera.rs             # Camera-specific tags (376 lines)
│   │   ├── gps.rs                # GPS-related tags (365 lines)
│   │   ├── time.rs               # Time-related tags (123 lines)
│   │   ├── author.rs             # Author/copyright tags (46 lines)
│   │   ├── special.rs            # Special/mixed-group tags (46 lines)
│   │   ├── common.rs             # Shared types (33 lines)
│   │   └── mod.rs                # Re-exports and unified interface
│   ├── Canon_pm/                 # Canon.pm extractions (functional splitting)
│   │   ├── canonimagesize.rs     # Image size lookup
│   │   ├── canonwhitebalance.rs  # White balance lookup
│   │   ├── canonmodelid.rs       # Model ID lookup
│   │   ├── canonquality.rs       # Quality lookup
│   │   ├── picturestyles.rs      # Picture styles lookup
│   │   └── mod.rs                # Re-exports all Canon tables
│   ├── ExifTool_pm/              # ExifTool.pm extractions (6 functional modules)
│   │   ├── mimetype.rs           # MIME type lookup
│   │   ├── filetypeext.rs        # File type extension lookup
│   │   ├── weakmagic.rs          # Weak magic patterns
│   │   ├── createtypes.rs        # Create types
│   │   ├── processtype.rs        # Process types
│   │   ├── ispc.rs               # IsPC checks
│   │   └── mod.rs                # Re-exports all ExifTool tables
│   ├── XMP_pm/                   # XMP.pm extractions (5 functional modules)
│   │   ├── nsuri.rs              # Namespace URI lookup
│   │   ├── xmpns.rs              # XMP namespace lookup
│   │   ├── charname.rs           # Character name lookup
│   │   ├── charnum.rs            # Character number lookup
│   │   ├── stdxlatns.rs          # Standard translation namespace
│   │   └── mod.rs                # Re-exports all XMP tables
│   ├── Nikon_pm/                 # Nikon.pm extractions
│   │   ├── nikonlensids.rs       # Lens ID lookup
│   │   └── mod.rs                # Re-exports all Nikon tables
│   ├── Exif_pm/                  # Exif.pm extractions
│   │   ├── orientation.rs        # Orientation lookup
│   │   └── mod.rs                # Re-exports all Exif tables
│   ├── PNG_pm/                   # PNG.pm extractions (3 functional modules)
│   │   ├── isdatchunk.rs         # IsDatChunk checks
│   │   ├── istxtchunk.rs         # IsTxtChunk checks
│   │   ├── noleapfrog.rs         # NoLeapFrog checks
│   │   └── mod.rs                # Re-exports all PNG tables
│   └── file_types/               # File type detection
│       ├── file_type_lookup.rs   # File type lookup
│       ├── magic_number_patterns.rs # Magic number patterns
│       └── mod.rs                # Re-exports all file type tables
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
// Generated lookup table using std::sync::LazyLock (no external dependencies)
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
# File type magic numbers (may contain non-UTF-8 bytes)
%magicNumber = (
    JPEG => '\xff\xd8\xff',
    PNG  => '\x89PNG\r\n\x1a\n',
    BPG  => 'BPG\xfb',  # Contains raw byte 0xFB
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
- **Scalability**: Modular structure with semantic grouping and functional splitting
- **Binary Safety**: Proper handling of non-UTF-8 bytes in patterns
- **Minimal Dependencies**: Uses std::sync::LazyLock, no external crates for core functionality

### Extractor Selection Guide

The codegen system includes multiple specialized extractors, each serving a specific purpose. Following the "one-trick pony" principle, each extractor excels at one type of extraction.

#### Tag-Related Extractors

**Primary Extractor (Use This)**:
- **`tag_kit.pl`** - Extracts complete tag definitions with embedded PrintConv implementations
  - Purpose: Create self-contained "tag kits" with everything needed to process a tag
  - Target: Any ExifTool tag table (EXIF, GPS, manufacturer-specific)
  - Output: Tag ID, name, format, groups, AND PrintConv data together
  - Benefit: Eliminates tag ID/PrintConv mismatch bugs

**Deprecated Extractors (Being Replaced by Tag Kit)**:
- **`inline_printconv.pl`** - ⚠️ DEPRECATED - Extracts PrintConv hashes from tag definitions
  - Being replaced: Tag kit extracts the same data as part of complete bundles
- **`tag_tables.pl`** - ⚠️ DEPRECATED - Hardcoded to extract only EXIF/GPS tags
  - Being replaced: Tag kit is config-driven and works with any module
- **`tag_definitions.pl`** - ⚠️ DEPRECATED - Extracts tags with frequency filtering
  - Being replaced: Tag kit provides complete bundles instead of function references

#### Lookup Table Extractors

- **`simple_table.pl`** - Extracts standalone key-value lookup tables
  - Purpose: Static lookups not associated with specific tags
  - Example: Canon white balance names, Nikon lens IDs
  - When to use: Manufacturer-specific lookup tables referenced by multiple tags

- **`boolean_set.pl`** - Extracts boolean membership sets
  - Purpose: Fast membership testing (hash keys mapping to 1)
  - Example: `if ($isDatChunk{$chunk})`
  - When to use: ExifTool sets used for existence checks

#### Binary Data Extractors

- **`process_binary_data.pl`** - Extracts ProcessBinaryData table structures
  - Purpose: Binary data parsing table definitions
  - Target: Tables with `%binaryDataAttrs`
  - Output: Offset, format, and tag information for binary parsing

- **`runtime_table.pl`** - Generates runtime HashMap creation code
  - Purpose: Tables requiring runtime context (camera model, firmware)
  - Target: ProcessBinaryData tables with conditional logic
  - Output: Functions that create HashMaps at runtime

#### Specialized Extractors

- **`composite_tags.pl`** - Extracts composite tag definitions
  - Purpose: Tags calculated from other tags
  - Features: Dependencies, expressions, requirements

- **`conditional_tags.pl`** - Extracts complex conditional tag arrays
  - Purpose: Model/firmware-specific tag variations
  - Features: Condition parsing, count/format detection

- **`file_type_lookup.pl`** - Extracts file type detection structures
  - Purpose: File format identification
  - Output: Discriminated unions with aliases

- **`regex_patterns.pl`** - Extracts magic number patterns
  - Purpose: Binary file signature detection
  - Features: Non-UTF-8 byte handling

- **`model_detection.pl`** - Extracts camera model patterns
  - Purpose: Model-specific behavior detection

- **`offset_patterns.pl`** - Extracts offset calculation patterns
  - Purpose: Sony-specific complex offset schemes

- **`tag_table_structure.pl`** - Extracts manufacturer table structures
  - Purpose: Generate Rust enums for manufacturer tables

#### Quick Decision Guide

1. **Extracting tags with their PrintConvs?** → Use `tag_kit.pl`
2. **Extracting standalone lookup tables?** → Use `simple_table.pl`
3. **Extracting binary data structures?** → Use `process_binary_data.pl`
4. **Need runtime HashMap creation?** → Use `runtime_table.pl`
5. **Extracting boolean sets?** → Use `boolean_set.pl`
6. **Extracting composite tags?** → Use `composite_tags.pl`

For detailed extractor comparisons and examples, see [EXTRACTOR-GUIDE.md](../reference/EXTRACTOR-GUIDE.md).

### Non-UTF-8 Data Handling

The code generation system properly handles non-UTF-8 bytes found in ExifTool's binary patterns:

#### Pattern Escaping

The `escape_pattern_for_rust` function ensures all patterns are valid Rust string literals:

```rust
// Handles non-UTF-8 bytes like 0xFB in BPG magic number
fn escape_pattern_for_rust(pattern: &str) -> String {
    let mut escaped = String::new();

    for byte in pattern.bytes() {
        match byte {
            b'\\' => escaped.push_str("\\\\"),
            b'"' => escaped.push_str("\\\""),
            b'\n' => escaped.push_str("\\n"),
            b'\r' => escaped.push_str("\\r"),
            b'\t' => escaped.push_str("\\t"),
            // Non-ASCII or control characters become \xNN
            0x00..=0x1F | 0x7F..=0xFF => {
                escaped.push_str(&format!("\\x{:02x}", byte));
            }
            _ => escaped.push(byte as char),
        }
    }
    escaped
}
```

#### JSON Cleaning

When extracting patterns from ExifTool that contain raw bytes:

1. Perl extracts the data (may contain non-UTF-8)
2. Rust detects and cleans problematic bytes in JSON
3. Patterns are properly escaped for Rust string literals
4. Generated code contains valid UTF-8 with escape sequences

Example: BPG magic number `BPG\xfb` becomes `"BPG\\xfb"` in generated Rust code.

### Build Pipeline

The simplified build pipeline uses Rust orchestration with simple Perl scripts:

1. **Auto-discovery**: Rust scans `codegen/config/` directories for modules
2. **Configuration**: Each module has JSON configs with explicit `source` paths
3. **Patching**: Rust uses atomic file operations to temporarily patch ExifTool modules
4. **Extraction**: Rust calls Perl scripts with explicit file paths and hash names
5. **Generation**: Creates modular Rust code directly from individual JSON files
6. **Cleanup**: Makefile reverts ExifTool patches using git checkout

```bash
# Full pipeline
make codegen              # Complete build with schema validation
make -j4 codegen         # Parallel execution (faster)

# Development commands
make check-schemas       # Validate all configuration files
cd codegen && cargo run --release  # Run code generation directly
```

#### Architecture Improvements

**Old System** (pre-July 2025):

- Complex Makefile with parallel extraction logic
- Perl scripts read JSON configs and knew about structure
- Combined extraction → split-extractions → individual files
- Hardcoded module lists in Rust main.rs
- Path guessing logic for ExifTool modules

**New System** (July 2025):

- **Rust orchestrates everything** - Single `cargo run` handles full pipeline
- **Simple Perl scripts** - Take explicit file paths and hash names as arguments
- **Direct JSON output** - No split-extractions step needed
- **Auto-discovery** - Scans config directories, no hardcoded module lists
- **Atomic file operations** - Uses tempfile crate for safe ExifTool patching
- **Modular generated code** - Semantic grouping and functional splitting
- **Explicit source paths** - No path guessing, all configured in JSON

#### Why Patching is Required

ExifTool uses `my`-scoped lexical variables for many lookup tables (e.g., `my %canonWhiteBalance`). These variables are private to their module and cannot be accessed by external scripts. To extract these tables programmatically, we need to convert them to package variables (`our %canonWhiteBalance`) which are accessible via the symbol table.

**Patching Implementation**: Rust handles patching with atomic file operations:

- **Auto-discovery**: Reads all configurations to identify required variables per module
- **Streaming replacement**: Uses `tempfile::NamedTempFile::new_in()` for same-filesystem temp files
- **Atomic operations**: `temp_file.persist()` ensures safe replacement
- **Targeted patching**: Only patches variables actually needed by configurations
- **Safe cleanup**: Makefile uses `git checkout` to revert patches after generation
- **Error handling**: Graceful fallback if patching fails for specific variables

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

The simplified build system supports efficient development:

```bash
# Parallel execution (faster development)
make -j4 codegen

# Direct Rust execution (for debugging)
cd codegen && cargo run --release

# Schema validation
make check-schemas

# Syntax checking
make check-extractors

# Clean generated files
make clean
```

## Complete Command Reference

### Generation

```bash
make codegen              # Full pipeline with schema validation
make -j4 codegen         # Parallel execution
cd codegen && cargo run --release  # Direct Rust execution
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
make clean              # Clean all generated files
make check-extractors   # Check Perl script syntax
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
- **Generated Integration**: Source-file-based organization with functional splitting
- **Runtime Registry**: Zero-overhead function dispatch with graceful fallbacks  
- **Scalable Architecture**: Modular code generation supporting 300+ lookup tables
- **Semantic Organization**: Tags grouped by logical categories (core, camera, GPS, etc.)
- **Build Performance**: Smaller files improve IDE response and compilation speed

## Related Documentation

- [API-DESIGN.md](API-DESIGN.md) - Public API structure and TagValue design
- [PROCESSOR-DISPATCH.md](../guides/PROCESSOR-DISPATCH.md) - Advanced processor dispatch
- [ENGINEER-GUIDE.md](../ENGINEER-GUIDE.md) - Practical implementation guide  
- [ARCHITECTURE.md](../ARCHITECTURE.md) - High-level system overview
- [DEVELOPMENT-GUIDE.md](../guides/DEVELOPMENT-GUIDE.md) - Development workflow and best practices
