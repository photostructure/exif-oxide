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
/// Stateful EXIF reader for processing JPEG-embedded EXIF data
/// Reference: ExifTool lib/Image/ExifTool/Exif.pm ProcessExif function architecture
pub struct ExifReader {
    /// Extracted tag values by (tag_id, namespace) for context-aware storage
    pub(crate) extracted_tags: HashMap<(u16, String), TagValue>,
    /// Enhanced tag source tracking for conflict resolution
    pub(crate) tag_sources: HashMap<(u16, String), TagSourceInfo>,
    /// TIFF header information
    pub(crate) header: Option<TiffHeader>,
    /// Raw EXIF data buffer
    pub(crate) data: Vec<u8>,
    /// Parse errors (non-fatal, for graceful degradation)
    pub(crate) warnings: Vec<String>,

    // Stateful processing features
    /// PROCESSED hash for recursion prevention (ExifTool: $$self{PROCESSED})
    pub(crate) processed: HashMap<u64, String>,
    /// PATH stack for directory hierarchy tracking (ExifTool: $$self{PATH})
    pub(crate) path: Vec<String>,
    /// DataMember storage for tag dependencies (ExifTool: DataMember tags)
    pub(crate) data_members: HashMap<String, DataMemberValue>,
    /// Current base offset for pointer calculations
    pub(crate) base: u64,
    /// Processor dispatch configuration
    pub(crate) processor_dispatch: ProcessorDispatch,
    /// Computed composite tag values
    pub(crate) composite_tags: HashMap<String, TagValue>,
    // Additional fields for file type tracking, synthetic tags, etc.
}

impl ExifReader {
    /// Create new EXIF reader
    pub fn new() -> Self { /* ... */ }

    /// Parse EXIF data from JPEG APP1 segment after "Exif\0\0"
    /// Reference: ExifTool lib/Image/ExifTool/Exif.pm:6172 ProcessExif entry point
    pub fn parse_exif_data(&mut self, exif_data: &[u8]) -> Result<()> { /* ... */ }

    /// Get all extracted tags with their names (conversions already applied during extraction)
    /// Returns tags with group prefixes (e.g., "EXIF:Make", "GPS:GPSLatitude", "Composite:ImageSize")
    /// matching ExifTool's -G mode behavior
    pub fn get_all_tags(&self) -> HashMap<String, TagValue> { /* ... */ }

    /// Get all tags as TagEntry objects with both value and print representations
    /// This is the primary API that returns both ValueConv and PrintConv results
    pub fn get_all_tag_entries(&mut self) -> Vec<TagEntry> { /* ... */ }

    /// Build composite tags from extracted tags
    /// Reference: ExifTool lib/Image/ExifTool.pm BuildCompositeTags function
    pub fn build_composite_tags(&mut self) { /* ... */ }

    /// Get parsing warnings
    pub fn get_warnings(&self) -> &[String] { /* ... */ }

    /// Get TIFF header information
    pub fn get_header(&self) -> Option<&TiffHeader> { /* ... */ }
}
```

## Data Structures

### TagEntry - The Core Output Type

````rust
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

    /// ExifTool Group1 (subdirectory location)
    ///
    /// Identifies the specific IFD or subdirectory where the tag was found:
    /// - "IFD0" - Main image IFD
    /// - "ExifIFD" - EXIF subdirectory (tag 0x8769)
    /// - "GPS" - GPS subdirectory (tag 0x8825)
    /// - "InteropIFD" - Interoperability subdirectory (tag 0xa005)
    /// - "MakerNotes" - Manufacturer-specific subdirectory (tag 0x927c)
    pub group1: String,

    /// The display value after PrintConv processing.
    ///
    /// This can be either:
    /// - A string for human-readable formatting (e.g., "1/100", "24.0 mm", "Rotate 90 CW")
    /// - A numeric value for data that should remain numeric in JSON (e.g., ISO: 100, FNumber: 4.0)
    ///
    /// PrintConv functions decide the appropriate type based on the tag's semantics.
    /// If no PrintConv exists, this equals the original `value`.
    ///
    /// # Design Note
    ///
    /// This differs from ExifTool where PrintConv always returns strings.
    /// We chose this approach to avoid regex-based type guessing during JSON serialization.
    pub print: TagValue,
}
````

### Other Data Types

```rust
/// Configuration for filtering which tags to extract and how to format them
/// This struct controls both tag selection (filtering) and value formatting (PrintConv vs ValueConv)
#[derive(Debug, Clone, PartialEq)]
pub struct FilterOptions {
    pub requested_tags: Vec<String>,
    pub requested_groups: Vec<String>,
    pub group_all_patterns: Vec<String>,
    pub extract_all: bool,
    pub numeric_tags: HashSet<String>,
    pub glob_patterns: Vec<String>,
}

/// Represents extracted EXIF data from an image
/// This matches ExifTool's JSON output structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExifData {
    pub source_file: String,
    pub tags: IndexMap<String, TagValue>,
    pub entries: Vec<TagEntry>,
}

/// Tag value representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TagValue {
    String(String),
    Integer(i64),
    Float(f64),
    Rational(i64, i64),
    Binary(Vec<u8>),
    Array(Vec<TagValue>),
}

/// Enhanced tag source tracking for conflict resolution
/// Maps (tag_id, namespace) -> TagSourceInfo with namespace, priority, and processor context
#[derive(Debug, Clone, PartialEq)]
pub struct TagSourceInfo {
    pub namespace: String,
    pub ifd_name: String,
    pub processor: String,
}

/// TIFF header information
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TiffHeader {
    pub byte_order: ByteOrder,
    pub magic: u16,
    pub ifd0_offset: u32,
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

The CLI uses the TagEntry API to provide both value and print representations:

```bash
# Normal JSON output (uses print field from TagEntry)
exif-oxide image.jpg
{
  "EXIF:FNumber": 4.0,           # PrintConv may return numeric value
  "EXIF:ExposureTime": "1/2000", # PrintConv returns formatted string
  "EXIF:FocalLength": "24.0 mm"  # PrintConv adds units
}

# With numeric flag (uses value field from TagEntry)
exif-oxide --numeric FNumber image.jpg
{
  "EXIF:FNumber": 4.0,           # Raw ValueConv result
  "EXIF:ExposureTime": "1/2000", # Not requested with numeric, uses print
  "EXIF:FocalLength": "24.0 mm"  # Not requested with numeric, uses print
}
```

## Error Handling

The API provides graceful degradation:

- Missing implementations return raw values, never panic
- Partial results are returned even if some tags fail
- Errors are collected in `Metadata.errors`
- Warnings for non-fatal issues in `Metadata.warnings`

## Current Implementation Status

The API is currently focused on read-only metadata extraction with:

- **Complete EXIF parsing** for JPEG files with subdirectory support
- **Composite tag computation** following ExifTool's dependency resolution
- **Manufacturer-specific parsing** for Canon, Sony, Nikon maker notes
- **RAW format support** for several formats (RW2, ARW, NEF, etc.)
- **XMP and IPTC parsing** for additional metadata sources
- **ExifTool compatibility** in tag naming and value processing

### Future Considerations

- **Streaming API**: Currently all data is loaded into memory
- **Write Support**: Planned for future milestones
- **Async Support**: Not currently planned but could be added

## Unknown Tag Handling

exif-oxide follows ExifTool's default behavior by omitting tags marked as "Unknown" in the source. This provides cleaner output and matches what most users expect from metadata extraction tools.

### Current Behavior

**ExifTool (default)**:
```bash
$ exiftool image.jpg | grep "WB RGGB" | wc -l
10  # Shows only known/documented tags
```

**ExifTool with -u flag**:
```bash
$ exiftool -u image.jpg | grep "WB RGGB" | wc -l
25  # Shows all tags including Unknown ones
```

**exif-oxide**:
```bash
$ exif-oxide image.jpg | grep "WB_RGGB" | wc -l
10  # Shows only known tags, matching ExifTool default
```

### Why We Filter Unknown Tags

1. **Clean Output**: Most users don't need undocumented tag data
2. **ExifTool Compatibility**: Matches ExifTool's default behavior
3. **Reduced Noise**: Prevents cluttering output with experimental or manufacturer-internal tags
4. **Performance**: Slightly faster by skipping unknown tag processing

### Implementation

The tag_kit.pl extractor captures all tags including those with `Unknown => 1`, but we filter them during runtime:

```rust
// Skip tags marked as Unknown (matching ExifTool's default behavior)
if tag_name.contains("Unknown") {
    debug!("Skipping unknown {} tag: {}", manufacturer, tag_name);
    continue;
}
```

This simple string-based filter catches all tags with "Unknown" in their name, which ExifTool uses consistently for undocumented tags. The filtering is implemented in `src/exif/subdirectory_processing.rs` during binary data extraction.

### Current CLI Limitations

Our CLI currently does NOT support:
- No `-u` flag equivalent
- No `--show-unknown` flag
- Unknown tags are always filtered out

This matches ExifTool's default behavior but doesn't provide the option to show unknown tags.

### Future Considerations

We could add ExifTool `-u` equivalent functionality:
1. **Add CLI flag**: `--show-unknown` to include unknown tags
2. **Make filtering conditional**: Skip the "Unknown" check when flag is set
3. **Library API**: Add `include_unknown_tags` option to FilterOptions

## Related Documentation

- [CORE-ARCHITECTURE.md](../guides/CORE-ARCHITECTURE.md) - Core system architecture and offset management
- [CODEGEN.md](../CODEGEN.md) - Code generation and manual implementation system
- [PRINTCONV-VALUECONV-GUIDE.md](../guides/PRINTCONV-VALUECONV-GUIDE.md) - PrintConv/ValueConv implementation guide
- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - Critical principle for all implementation work
