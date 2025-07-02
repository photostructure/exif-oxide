# exif-oxide Implementation Palette

This document describes the organization and patterns for manual implementations in exif-oxide, which handle complex logic that cannot be automatically generated from ExifTool source.

## Overview

The Implementation Palette is a collection of manually-written Rust code that:

- Implements complex conversion logic from ExifTool
- Handles manufacturer-specific processing
- Provides format parsing for variable-length data
- Manages encryption/decryption algorithms

Every manual implementation includes references to the specific ExifTool source code it's based on.

## Current Implementation Structure

**Note**: The documented ideal structure differs from current implementation. Current structure:

```
src/implementations/
├── mod.rs          # Module exports
├── print_conv.rs   # PrintConv implementations
├── value_conv.rs   # ValueConv implementations
├── canon.rs        # Canon-specific processors
└── sony.rs         # Sony-specific processors
```

**Planned structure** (from ARCHITECTURE.md):

```
implementations/
├── registry.rs # Maps signatures/keys to implementations
├── process/
│   ├── mod.rs # ProcessProc trait and dispatch
│   ├── exif.rs # ProcessExif implementation
│   ├── binary_data.rs # ProcessBinaryData with simple patterns
│   ├── xmp.rs # ProcessXMP implementation
│   └── manufacturers/
│       ├── canon/
│       │   ├── mod.rs
│       │   └── serial_data.rs # ProcessSerialData
│       ├── nikon/
│       │   ├── mod.rs
│       │   └── encrypted.rs # ProcessNikonEncrypted
│       └── sony/
│           └── encrypted.rs # Sony encryption
├── conversions/
│   ├── print_conv/
│   │   ├── lookups.rs # Simple hash lookups
│   │   ├── bitwise.rs # BITMASK operations
│   │   └── format.rs # sprintf-style formatting
│   ├── value_conv/
│   │   ├── math.rs # Mathematical conversions
│   │   └── binary.rs # Binary data handling
│   └── conditions/
│       └── model.rs # Model/firmware conditions
├── formats/
│   ├── simple.rs # int16u[10], string[20]
│   └── variable.rs # string[$val{3}] ONLY
├── crypto/
│   ├── nikon.rs # Nikon decryption algorithm
│   └── sony.rs # Sony decryption algorithm
└── error/
    └── classification.rs # MINOR_ERRORS system port
```

## Indexing Schemes

The palette uses multiple indexing strategies to map from ExifTool definitions to Rust implementations:

### 1. Signature-Based (for PrintConv/ValueConv)

```rust
registry.register_print_conv(
    "{ 0 => 'None', 1 => 'Horizontal (normal)', ... }",
    implementations::print_conv::orientation
);
```

The signature is the ExifTool Perl hash/expression, mapped to the Rust implementation.

### 2. Key-Based (for complex implementations)

```rust
registry.register_processor(
    ProcessorKey {
        table: "Canon::Main",
        tag: Some(0x5),
        name: "ProcessSerialData",
    },
    implementations::process::canon::serial_data::process
);
```

Complex processors are indexed by table, tag ID, and processor name.

### 3. Pattern-Based (for formats)

```rust
registry.register_format_pattern(
    "string[$val{*}]",
    implementations::formats::variable::string_from_val
);
```

Variable format patterns are matched to format parsing functions.

## Manual Implementation Pattern

Each manual implementation follows this pattern:

```rust
/// Canon Serial/AF Data Processing
///
/// Handles variable-length serial data based on NumAFPoints.
/// ExifTool: lib/Image/ExifTool/Canon.pm:6337 ProcessSerialData
pub fn process_serial_data(
    reader: &mut ExifReader,
    data: &[u8],
    table: &TagTable,
) -> Result<()> {
    // Manual port of ProcessSerialData logic
    // References specific line numbers in ExifTool

    // Get NumAFPoints from DataMember
    let num_points = reader.data_members.get("NumAFPoints")
        .and_then(|v| v.as_u16())
        .unwrap_or(0);

    // Complex logic ported from Perl...
}
```

**Key Requirements**:

1. **Documentation**: Clear description of what the function does
2. **ExifTool Reference**: Exact file and line numbers in ExifTool source
3. **Faithful Translation**: Logic matches ExifTool exactly, no "improvements"
4. **Error Handling**: Graceful degradation on invalid data

## Handling ProcessBinaryData

### Supported Format Patterns

Only these simple patterns are handled by codegen:

1. **Fixed Arrays**: `int16u[10]`, `string[32]`
2. **Simple Variables**: `string[$val{3}]` where tag 3 has the length

### Format Handler Example

```rust
// In implementations/formats/variable.rs
pub fn parse_string_from_val(
    data: &[u8],
    offset: usize,
    val_ref: usize,
    ctx: &Context,
) -> Result<(String, usize)> {
    // Get length from previously extracted value
    let length = ctx.get_value(val_ref)
        .and_then(|v| v.as_usize())
        .ok_or("Missing length value")?;

    // Validate bounds
    if offset + length > data.len() {
        return Err("String extends beyond data");
    }

    // Extract string
    let bytes = &data[offset..offset + length];
    let string = String::from_utf8_lossy(bytes).into_owned();

    Ok((string, length))
}
```

### Complex Format Handling

For complex formats, entire tag tables are manually implemented:

```rust
// When Format contains Hook or complex expressions
pub fn process_sony_camera_settings(
    reader: &mut ExifReader,
    data: &[u8],
) -> Result<()> {
    // Entire manual implementation
    // No attempt to parse Perl Hook code
}
```

## Common Implementation Patterns

### PrintConv Implementations

```rust
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

### ValueConv Implementations

```rust
pub fn fnumber_value_conv(val: &TagValue) -> TagValue {
    match val {
        TagValue::Rational(num, den) if *den != 0 => {
            TagValue::Float(*num as f64 / *den as f64)
        }
        _ => val.clone(),
    }
}
```

### Manufacturer-Specific Processors

```rust
// Canon AF Info processing with variable-length arrays
pub fn process_canon_af_info(
    reader: &mut ExifReader,
    data: &[u8],
    _table: &TagTable,
) -> Result<()> {
    // ExifTool: Canon.pm:6520 AFInfo2
    
    let mut offset = 0;
    
    // Tag 0: AFInfoSize
    let size = u16::from_be_bytes([data[offset], data[offset + 1]]);
    reader.add_tag("Canon:AFInfoSize", TagValue::Integer(size as i64));
    offset += 2;
    
    // Tag 1: AFAreaMode
    let mode = data[offset + 1];
    reader.add_tag("Canon:AFAreaMode", TagValue::Integer(mode as i64));
    offset += 2;
    
    // Tag 2: NumAFPoints - stored as DataMember
    let num_points = u16::from_be_bytes([data[offset], data[offset + 1]]);
    reader.data_members.insert("NumAFPoints".to_string(), num_points);
    reader.add_tag("Canon:NumAFPoints", TagValue::Integer(num_points as i64));
    offset += 2;
    
    // Variable-length arrays based on NumAFPoints
    // ... complex parsing logic
    
    Ok(())
}
```

## Error Handling Pattern

```rust
pub enum ErrorLevel {
    /// Fatal - stop processing
    Fatal,
    /// Minor - continue but warn (ExifTool's MINOR_ERRORS)
    Minor,
    /// Warning - informational only
    Warning,
}

pub struct ExifError {
    pub level: ErrorLevel,
    pub message: String,
    pub context: ErrorContext,
}

pub struct ErrorContext {
    pub path: Vec<String>,      // IFD0.ExifIFD.MakerNotes
    pub tag: Option<String>,
    pub offset: Option<usize>,
}

/// Idiomatic Rust error types using thiserror
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExifError {
    #[error("Invalid JPEG marker {marker:#x} at offset {offset:#x}")]
    InvalidMarker { marker: u8, offset: usize },

    #[error("Tag {tag} requires format {required} but found {found}")]
    InvalidFormat { tag: String, required: String, found: String },

    #[error("Missing processor implementation: {0}")]
    MissingProcessor(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Adding New Implementations

### Step 1: Identify Need

Use the `--show-missing` flag to identify what implementations are needed:

```bash
cargo run -p exif-oxide -- --show-missing test.jpg
```

### Step 2: Find ExifTool Source

Locate the implementation in ExifTool source:
- PrintConv: Usually in the tag definition
- ValueConv: In tag definition or separate function
- Processors: In manufacturer-specific modules

### Step 3: Implement

Create the function following the patterns above:
- Include documentation
- Reference ExifTool source
- Match behavior exactly
- Handle edge cases

### Step 4: Register

Add to the appropriate registry:
- PrintConv → `print_conv.rs`
- ValueConv → `value_conv.rs`
- Processors → manufacturer-specific file

### Step 5: Test

Verify against ExifTool output:
```bash
# Compare outputs
exiftool -j test.jpg > expected.json
cargo run -p exif-oxide -- test.jpg > actual.json
```

## Future Work

1. **Reorganize** to match planned structure when complexity requires it
2. **Add crypto** implementations for Nikon/Sony encryption
3. **Implement conditions** for model-specific behavior
4. **Add format parsers** for more variable-length patterns

## Related Documentation

- [CODEGEN-STRATEGY.md](CODEGEN-STRATEGY.md) - What's generated vs manual
- [PROCESSOR-PROC-DISPATCH.md](../PROCESSOR-PROC-DISPATCH.md) - Processor dispatch design
- [STATE-MANAGEMENT.md](../STATE-MANAGEMENT.md) - How state is managed during processing