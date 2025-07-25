# exif-oxide Public API Design

**🚨 CRITICAL: This API design maintains ExifTool compatibility per [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md).**

This document describes the public API that the CLI and library consumers use to extract metadata from image files.

## Overview

The API is designed to provide:

- **Memory-efficient streaming** for large files and binary data
- **Compatibility with ExifTool** output formats
- **Type-safe tag values** with both logical and display representations
- **Graceful error handling** with partial results on failure

## Core API Structure

```rust
/// Main metadata reader - generated by codegen
pub struct ExifReader {
    // State management (see STATE-MANAGEMENT.md)
    processed: HashSet<DirectoryPath>,
    values: HashMap<String, TagValue>,
    data_members: HashMap<String, Value>,

    // Current processing context
    path: Vec<String>,
    base: u32,
    byte_order: ByteOrder,
}

impl ExifReader {
    /// Primary entry point for in-memory data
    pub fn read_metadata(
        &mut self,
        data: &[u8],
        options: ReadOptions,
    ) -> Result<Metadata, ExifError> {
        // Codegen produces this implementation:
        // 1. Detect file format using magic numbers
        // 2. Dispatch to appropriate processor
        // 3. Traverse metadata structure
        // 4. Apply conversions
        // 5. Return structured results
    }

    /// Streaming entry point (preferred)
    pub fn read_metadata_stream<R: Read + Seek>(
        &mut self,
        reader: R,
        options: ReadOptions,
    ) -> Result<Metadata, ExifError> {
        // Streaming implementation that doesn't load entire file
        // Binary tags return BinaryRef for separate streaming
    }

    /// Extract specific tags only
    pub fn read_tags(
        &mut self,
        data: &[u8],
        tag_names: &[String],
    ) -> Result<HashMap<String, TagValue>, ExifError> {
        let options = ReadOptions::default()
            .with_requested_tags(tag_names);
        let metadata = self.read_metadata(data, options)?;
        Ok(metadata.filtered_tags(tag_names))
    }

    /// Get reference to binary data for streaming
    pub fn get_binary_ref(
        &mut self,
        data: &[u8],
        tag_name: &str,
    ) -> Result<BinaryRef, ExifError> {
        // Returns reference for separate streaming
        // Does NOT load binary data into memory
    }

    /// Stream binary data separately
    pub fn stream_binary_tag<R: Read + Seek>(
        &self,
        reader: R,
        binary_ref: &BinaryRef,
    ) -> Result<impl Read, ExifError> {
        Ok(BinaryTagReader::new(reader, binary_ref))
    }
}
```

## Data Structures

### TagEntry - The Core Output Type

```rust
/// A single extracted metadata tag with both its converted value and display string.
/// 
/// This structure provides access to both the logical value (after ValueConv)
/// and the human-readable display string (after PrintConv), allowing consumers
/// to choose the most appropriate representation.
/// 
/// # Examples
/// 
/// ```
/// // A typical EXIF tag entry
/// TagEntry {
///     group: "EXIF".to_string(),
///     name: "FNumber".to_string(),
///     value: TagValue::F64(4.0),      // Post-ValueConv: 4/1 → 4.0
///     print: "4.0".to_string(),       // Post-PrintConv: formatted for display
/// }
/// 
/// // A tag with units in the display string
/// TagEntry {
///     group: "EXIF".to_string(),
///     name: "FocalLength".to_string(),
///     value: TagValue::F64(24.0),     // Numeric value
///     print: "24.0 mm".to_string(),   // Human-readable with units
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagEntry {
    /// Tag group name (e.g., "EXIF", "GPS", "Canon", "MakerNotes")
    /// 
    /// Groups follow ExifTool's naming conventions:
    /// - Main IFDs: "EXIF", "GPS", "IFD0", "IFD1"
    /// - Manufacturer: "Canon", "Nikon", "Sony", etc.
    /// - Sub-groups: "Canon::CameraSettings", etc.
    pub group: String,
    
    /// Tag name without group prefix (e.g., "FNumber", "ExposureTime")
    /// 
    /// Names match ExifTool's tag naming exactly for compatibility.
    pub name: String,
    
    /// The logical value after ValueConv processing.
    /// 
    /// This is the value you get with ExifTool's -# flag:
    /// - Rational values converted to floats (4/1 → 4.0)
    /// - APEX values converted to real units
    /// - Raw value if no ValueConv exists
    /// 
    /// # Examples
    /// 
    /// - FNumber: `TagValue::F64(4.0)` (from rational 4/1)
    /// - ExposureTime: `TagValue::F64(0.0005)` (from rational 1/2000)
    /// - Make: `TagValue::String("Canon")` (no ValueConv needed)
    pub value: TagValue,
    
    /// The display string after PrintConv processing.
    /// 
    /// This is the human-readable representation:
    /// - Numbers may be formatted ("4.0" not "4")
    /// - Units may be added ("24.0 mm")
    /// - Coded values decoded ("Rotate 90 CW" not "6")
    /// 
    /// If no PrintConv exists, this equals `value.to_string()`.
    /// 
    /// # ExifTool JSON Compatibility
    /// 
    /// When serializing to JSON, some numeric PrintConv results
    /// (like FNumber's "4.0") are encoded as JSON numbers, not strings.
    /// The CLI handles this compatibility layer.
    pub print: String,
}
```

### Other Data Types

```rust
/// Results structure
pub struct Metadata {
    pub tags: Vec<TagEntry>,
    pub errors: Vec<ExifError>,
    pub warnings: Vec<String>,
}

/// Tag value representation
pub enum TagValue {
    String(String),
    Integer(i64),
    Float(f64),
    Rational(i64, i64),
    Binary(BinaryRef),  // Streaming reference, not data
    Array(Vec<TagValue>),
}

/// Reference to binary data for streaming
pub struct BinaryRef {
    offset: u64,
    length: usize,
    format: BinaryFormat,  // JPEG, TIFF, etc.
}

/// Streaming reader for binary data
pub struct BinaryTagReader<R: Read + Seek> {
    reader: R,
    offset: u64,
    remaining: usize,
}

impl<R: Read + Seek> Read for BinaryTagReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // Efficient streaming implementation
    }
}
```

## Value and Print Conversion Strategy

### Overview

ExifTool has a two-stage conversion system that we faithfully reproduce:

1. **ValueConv**: Converts raw extracted data to logical values (e.g., rational → float, APEX → stops)
2. **PrintConv**: Converts logical values to human-readable strings (e.g., 4.0 → "f/4.0")

Our API exposes both conversions, allowing consumers to choose:
- `value` field: Post-ValueConv logical value (matches ExifTool -# output)
- `print` field: Post-PrintConv display string (matches normal ExifTool output)

### Conversion Pipeline

```
Raw Data → Format Extraction → ValueConv → value field
                                        ↓
                                   PrintConv → print field
```

### ExifTool Compatibility Notes

**PrintConv Quirks**:
- Some PrintConv functions return numeric strings that become JSON numbers (e.g., FNumber: "4.0" → 4.0)
- Others return formatted strings (e.g., FocalLength: 24 → "24.0 mm")
- We match these quirks exactly for compatibility

**-# Flag Behavior**:
- ExifTool's `-TagName#` syntax disables PrintConv for that tag
- Shows the ValueConv result (or raw value if no ValueConv)
- Our CLI emulates this behavior precisely

### Implementation Example

```rust
// In tag extraction
let (value, print) = self.apply_conversions(&raw_value, tag_def);

let entry = TagEntry {
    group: "EXIF".to_string(),
    name: tag_def.name.to_string(),
    value,  // Post-ValueConv (e.g., 4.0 as f64)
    print,  // Post-PrintConv (e.g., "4.0" or "f/4.0")
};
```

## CLI Output Behavior

The CLI chooses between value and print based on user flags:

```bash
# Normal output (uses print field)
exif-oxide image.jpg
{
  "EXIF:FNumber": 4.0,           # PrintConv returns "4.0", JSON encodes as number
  "EXIF:ExposureTime": "1/2000", # PrintConv returns string
  "EXIF:FocalLength": "24.0 mm"  # PrintConv returns string with units
}

# With -# flag (uses value field)
exif-oxide -FNumber# image.jpg
{
  "EXIF:FNumber": 4,             # ValueConv result (or raw if no ValueConv)
  "EXIF:ExposureTime": "1/2000", # Not requested with #, uses print
  "EXIF:FocalLength": "24.0 mm"  # Not requested with #, uses print
}
```

## Error Handling

The API provides graceful degradation:
- Missing implementations return raw values, never panic
- Partial results are returned even if some tags fail
- Errors are collected in `Metadata.errors`
- Warnings for non-fatal issues in `Metadata.warnings`

## Future Considerations

### Async Support

The current implementation is synchronous. Future async support would add:

```rust
pub async fn read_metadata_async<R: AsyncRead + AsyncSeek>(
    mut reader: R,
    options: ReadOptions,
) -> Result<Metadata, ExifError> {
    // Async implementation
}
```

### Write Support

Future write support would extend the API with:

```rust
pub fn write_metadata<W: Write + Seek>(
    &mut self,
    writer: W,
    changes: HashMap<String, TagValue>,
) -> Result<(), ExifError> {
    // Write implementation
}
```

## Related Documentation

- [STATE-MANAGEMENT.md](../STATE-MANAGEMENT.md) - How state is managed during processing
- [CODEGEN.md](CODEGEN.md) - Code generation and manual implementation system
