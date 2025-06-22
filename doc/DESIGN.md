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
│   │   ├── reader.rs       # Binary data reading with endian support
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
# Input: Canon.pm
%Image::ExifTool::Canon::Main = (
    GROUPS => { 0 => 'MakerNotes', 2 => 'Camera' },
    0x1 => {
        Name => 'CanonCameraSettings',
        SubDirectory => { TagTable => 'Image::ExifTool::Canon::CameraSettings' },
    },
);
```

```rust
// Output: canon.rs (generated)
lazy_static! {
    pub static ref CANON_MAIN: TagTable = TagTable {
        groups: &[("MakerNotes", 0), ("Camera", 2)],
        tags: &[
            (0x1, Tag {
                name: "CanonCameraSettings",
                tag_type: TagType::SubDirectory("Canon::CameraSettings"),
                ..Default::default()
            }),
        ],
    };
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

## Future Extensibility

1. **Plugin System** for custom tags
2. **WASM Support** for browser usage
3. **Async API** for web services
4. **Streaming Parser** for large files
5. **Write Support** for all formats