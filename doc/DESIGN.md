# exif-oxide Design Document

## Overview

exif-oxide is a high-performance Rust implementation that provides ExifTool-compatible metadata extraction and manipulation. It leverages 25 years of Phil Harvey's ExifTool development while adding the performance and safety benefits of Rust, plus a decade of real-world datetime parsing heuristics from exiftool-vendored.

## Goals

1. **Performance**: Sub-10ms processing for typical JPEG files (vs ExifTool's 100-200ms)
2. **Compatibility**: Maintain tag name and structure compatibility with ExifTool
3. **Embedded Image Extraction**: First-class support for extracting thumbnails and previews
4. **DateTime Intelligence**: Incorporate proven timezone inference and parsing heuristics
5. **Maintainability**: Easy updates when ExifTool adds new camera support
6. **Safety**: Memory-safe handling of untrusted binary data

## Architecture

### Core Components

```
exif-oxide/
├── src/
│   ├── tables/              # Auto-generated from ExifTool
│   │   ├── mod.rs          # Table registry and lookup
│   │   ├── exif.rs         # Standard EXIF tags
│   │   ├── canon.rs        # Canon-specific tags
│   │   ├── nikon.rs        # Nikon-specific tags
│   │   └── ...             # Other manufacturers
│   │
│   ├── core/               # Core parsing logic
│   │   ├── mod.rs
│   │   ├── types.rs        # Core EXIF data types and formats
│   │   ├── endian.rs       # Byte order handling
│   │   ├── ifd.rs          # IFD structure parsing
│   │   ├── jpeg.rs         # JPEG segment parsing
│   │   ├── tiff.rs         # TIFF/RAW container parsing
│   │   └── binary_data.rs  # ProcessBinaryData port
│   │
│   ├── extract/            # Image extraction
│   │   ├── mod.rs
│   │   ├── thumbnail.rs    # EXIF thumbnail extraction
│   │   ├── preview.rs      # Maker note preview extraction
│   │   └── embedded.rs     # Generic embedded image handling
│   │
│   ├── datetime/           # DateTime parsing intelligence
│   │   ├── mod.rs
│   │   ├── parser.rs       # Core parsing with quirk handling
│   │   ├── timezone.rs     # GPS and multi-source TZ inference
│   │   └── quirks.rs       # Manufacturer-specific fixes
│   │
│   ├── value/              # Tag value types
│   │   ├── mod.rs
│   │   ├── types.rs        # EXIF data types
│   │   ├── convert.rs      # Value conversions
│   │   └── rational.rs     # Rational number handling
│   │
│   └── lib.rs              # Public API
│
├── tools/
│   ├── table_converter/    # Perl to Rust converter
│   │   ├── parser.rs       # Parse Perl tag tables
│   │   ├── generator.rs    # Generate Rust code
│   │   └── main.rs
│   │
│   └── sync_exiftool.rs    # Update from ExifTool releases
│
└── tests/
    ├── integration/        # End-to-end tests
    ├── images/            # Test images from ExifTool
    └── compatibility/     # ExifTool output comparison
```

### Data Flow

1. **File Reading**

   - Memory-mapped for large files
   - Buffered reading for small files
   - Format detection from magic bytes

2. **Format Parsing**

   - JPEG: APP1 segment extraction
   - TIFF/RAW: IFD chain navigation
   - Container formats: Seek to metadata

3. **Tag Processing**

   - Lazy evaluation - only parse requested tags
   - Binary data validation
   - Maker note decryption where needed

4. **Value Extraction**

   - Type-safe conversions
   - Multi-value handling
   - Composite tag calculation

5. **DateTime Enhancement**
   - Multi-source timezone inference
   - GPS coordinate-based TZ lookup
   - Manufacturer quirk corrections

### Key Design Decisions

#### 1. Table Generation Strategy

Rather than manually porting each tag table, we auto-generate Rust code from ExifTool's Perl modules:

```perl
# Input: Exif.pm (implemented in Spike 1.5)
%Image::ExifTool::Exif::Main = (
    0x10f => {
        Name => 'Make',
        Groups => { 2 => 'Camera' },
        Writable => 'string',
        WriteGroup => 'IFD0',
    },
    0x11a => {
        Name => 'XResolution',
        Writable => 'rational64u',
        WriteGroup => 'IFD0',
        Mandatory => 1,
    },
);
```

```rust
// Output: generated_tags.rs (actual implementation)
pub const EXIF_TAGS: &[(u16, TagInfo)] = &[
    (0x010f, TagInfo { 
        name: "Make", 
        format: ExifFormat::Ascii, 
        group: Some("Camera") 
    }),
    (0x011a, TagInfo { 
        name: "XResolution", 
        format: ExifFormat::Rational, 
        group: None 
    }),
    // ... 494 more tags
];

pub fn lookup_tag(tag_id: u16) -> Option<&'static TagInfo> {
    EXIF_TAGS.iter().find(|(id, _)| *id == tag_id).map(|(_, info)| info)
}
```

#### 2. Memory Management

- **Zero-copy where possible**: Use byte slices instead of allocating strings
- **Arena allocation**: Group related allocations to reduce fragmentation
- **Streaming for large data**: Don't load entire RAW files for metadata

#### 3. Error Handling

- **Graceful degradation**: Continue parsing despite errors
- **Warning accumulation**: Collect non-fatal issues
- **Detailed error context**: Include file offset and tag context

#### 4. Compatibility Layer

Provide both ExifTool-compatible and Rust-idiomatic APIs:

```rust
// ExifTool-compatible
let exif = ExifOxide::new();
exif.extract_info("photo.jpg")?;
let make = exif.get_value("Make")?; // Returns string

// Rust-idiomatic
let metadata = exif_oxide::read_file("photo.jpg")?;
let make: &str = metadata.get(tags::MAKE)?; // Type-safe
let preview: Vec<u8> = metadata.extract_preview()?;
```

## Performance Strategy

1. **Lazy Parsing**

   - Build tag index without parsing values
   - Only decode requested tags
   - Cache parsed values

2. **Parallel Processing**

   - Parse independent IFDs concurrently
   - Parallel extraction of multiple previews
   - Batch file processing with thread pool

3. **Memory Efficiency**

   - Mmap large files
   - Reuse buffers across operations
   - Smart string interning for common values

4. **SIMD Optimizations**
   - Endian swapping
   - UTF-16 to UTF-8 conversion
   - CRC calculations

## DateTime Intelligence

Incorporating exiftool-vendored's heuristics:

1. **Timezone Inference Priority**

   - Explicit timezone in tag
   - GPS coordinates → timezone database
   - UTC offset calculation from multiple timestamps
   - Video files default to UTC
   - Camera model-specific defaults

2. **Quirk Handling**

   - Nikon DST bug correction
   - GPS coordinate validation (0,0 = unset)
   - Apple format variations
   - Sub-second precision handling

3. **Validation**
   - Sanity check against file modification time
   - Cross-reference multiple date fields
   - Handle "0000:00:00" invalid dates

## Maintenance and Updates

1. **Automated Sync**

   ```bash
   # Check for ExifTool updates
   ./tools/sync_exiftool --check

   # Generate new tables
   ./tools/sync_exiftool --update

   # Run compatibility tests
   cargo test --features compat-tests
   ```

2. **Version Tracking**

   - Track ExifTool version in generated files
   - Changelog for table updates
   - Git tags for ExifTool version sync points

3. **Compatibility Testing**
   - Compare output with ExifTool
   - Regression tests for datetime heuristics
   - Performance benchmarks

## Security Considerations

1. **Input Validation**

   - Bounds checking on all reads
   - Maximum recursion depth for IFDs
   - File size limits
   - Malformed data handling

2. **Memory Safety**
   - No unsafe code in core parsing
   - Fuzzing-based testing
   - Address sanitizer in CI

## Implementation Status

### Completed (Spike 1)

- Basic JPEG segment parsing
- IFD structure parsing with endian support
- Core type system (ExifFormat, ExifValue)
- Make, Model, Orientation tag extraction
- Comprehensive test infrastructure

### Completed (Spike 1.5)

- **Table-driven Architecture**: Auto-generates 496 EXIF tags from ExifTool's Perl source
- **Complete Format Support**: Rational, SignedRational, all integer types, arrays
- **Build-time Code Generation**: Zero runtime overhead for tag lookup
- **Development Tooling**: `parse_exiftool_tags` binary for debugging
- **Comprehensive Testing**: 29 tests covering all format types and real images

### Completed (Spike 2)

- **Maker Note Architecture**: Manufacturer detection and dispatch system with trait-based parsers
- **Canon Maker Note Support**: Successfully parses Canon-specific tags (28/36 tags = 78% coverage)
- **ExifIFD Integration**: Extended IFD parser to handle sub-directories (critical for maker notes)
- **Table Generation Extension**: Canon.pm parsing integrated into build.rs (34 Canon tags)
- **Structural Tag Handling**: Special format overrides for tags like 0x8769 (ExifOffset)
- **Real-world Validation**: Tested with Canon1DmkIII.jpg and other ExifTool test images

### Implementation Insights

1. **Parser Architecture**

   - Direct parsing without nom proved sufficient for basic EXIF
   - Modular design allows easy addition of new formats
   - Separation of concerns: JPEG parsing vs IFD parsing

2. **Error Handling**

   - Continue on non-fatal errors (like ExifTool)
   - Provide context in errors (offset, tag ID)
   - Use Result<Option<T>> for "may not exist" vs "error"

3. **Table Generation Pipeline (Spike 1.5)**

   ```
   ExifTool Source → Regex Parser → Code Generator → Static Tables
        (Exif.pm)      (build.rs)      (build.rs)    (generated_tags.rs)
   ```

   - **Build-time Translation**: Perl → Rust conversion during compilation
   - **Zero Runtime Cost**: Generated code is just static arrays
   - **Comprehensive Coverage**: 496 tags with format and group information
   - **Development Tools**: `parse_exiftool_tags` for debugging and exploration

4. **Format Type System**

   ```rust
   pub enum ExifValue {
       Ascii(String),
       U8(u8), U16(u16), U32(u32),
       I16(i16), I32(i32),
       Rational(u32, u32),              // numerator, denominator
       SignedRational(i32, i32),
       U16Array(Vec<u16>), U32Array(Vec<u32>),
       RationalArray(Vec<(u32, u32)>),
       // ... arrays for all types
       Undefined(Vec<u8>),
   }
   ```

5. **Maker Note Architecture (Spike 2)**

   ```rust
   // Manufacturer detection and dispatch
   pub trait MakerNoteParser: Send + Sync {
       fn parse(&self, data: &[u8], byte_order: Endian, base_offset: usize) 
           -> Result<HashMap<u16, ExifValue>>;
       fn manufacturer(&self) -> &'static str;
   }
   
   // Tag prefixing to avoid conflicts
   let prefixed_tag = 0x8000 + canon_tag; // Canon tag 0x0001 becomes 0x8001
   ```

   **Critical Discovery**: Maker notes are typically stored in ExifIFD (tag 0x8769), not IFD0. This required extending the IFD parser to handle sub-directories and merge ExifIFD entries with the main IFD.

6. **Testing Strategy**
   - Unit tests with synthetic data for edge cases
   - Integration tests with ExifTool's test images
   - Table lookup validation for all generated tags
   - Real-world rational number parsing with Canon/Nikon images
   - Discovered discrepancies (e.g., ExifTool.jpg metadata)
   - Maker note validation with professional camera images (Canon1DmkIII.jpg)

## Future Extensibility

1. **Plugin System** for custom tags
2. **WASM Support** for browser usage
3. **Async API** for web services
4. **Streaming Parser** for large files
5. **Write Support** for all formats
