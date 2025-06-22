# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Context

exif-oxide is a Rust reimplementation of Phil Harvey's ExifTool, focusing on:

- Performance (10-20x faster than Perl)
- Memory safety for untrusted files
- Embedded image extraction (thumbnails/previews)
- DateTime parsing intelligence from exiftool-vendored

## Development Philosophy

### 1. Respect ExifTool's Legacy

- Phil Harvey has been developing ExifTool for 25 years
- The Perl codebase contains invaluable camera-specific quirks and edge cases
- When in doubt, check how ExifTool handles it
- Maintain tag name and structure compatibility

### 2. Incremental Development

- Follow the spike plan in SPIKES.md
- Each spike should be independently testable
- Don't over-engineer early spikes - learn first, optimize later
- Document surprises and gotchas as you discover them

### 3. Testing Approach

- Use ExifTool's test images from t/images/ when possible
- Always test both byte orders (II and MM)
- Test with corrupted/malformed data
- Compare output with ExifTool for validation

## Technical Guidelines

### Binary Parsing

- Always bounds-check before reading
- Use byteorder crate for endian handling
- Prefer `get()` over indexing to avoid panics
- Maximum recursion depth for IFDs to prevent loops

### Error Handling

- Use thiserror for error types
- Provide context (offset, tag ID) in errors
- Continue parsing on non-fatal errors
- Collect warnings separately from errors

### Performance

- Benchmark against ExifTool regularly
- Profile before optimizing
- Consider memory-mapped files for large RAWs
- Lazy parsing - only decode requested tags

## ExifTool Compatibility Notes

### Tag Tables

- Tag IDs are hex numbers (0x10F = Make)
- Group hierarchy matters (0 = family, 1 = specific, 2 = category)
- Some tags have conditions based on other tag values
- PrintConv provides human-readable conversions

### Maker Notes

- Often use relative offsets from maker note start
- Some manufacturers encrypt or obfuscate data
- Model-specific variations are common
- Always preserve unknown data for write-back

### Binary Data

- ProcessBinaryData in ExifTool is the key pattern
- Handles both fixed and variable-length formats
- Negative indices count from end
- Some data needs bit-level parsing

## DateTime Heuristics

Key learnings from exiftool-vendored:

1. **Never trust single sources** - cross-reference multiple date fields
2. **GPS 0,0 means "unset"** not "off the coast of Africa"
3. **Video dates can be encoded in UTC** unless explicitly specified
4. **Nikon DST bug** - some models incorrectly handle daylight saving
5. **Subsecond data** - stored separately, various formats

## Common Pitfalls

### JPEG Parsing

- APP1 segments have 64KB size limit
- Multiple APP1 segments possible (EXIF + XMP)
- Segment length includes the length bytes
- Check for "Exif\0\0" signature

### IFD Parsing

- Next IFD offset can be -1 (0xFFFFFFFF) meaning "no next"
- Value fits inline if size × count ≤ 4 bytes
- Offsets are from TIFF header start, not file start
- Some cameras write invalid offsets

### String Handling

- EXIF strings are null-terminated
- But buffer may contain garbage after null
- UTF-8 not guaranteed - may need charset detection
- Some makers pad with spaces instead of nulls

## Development Workflow

1. **Before implementing**: Check ExifTool's implementation
2. **Write tests first**: Especially for edge cases
3. **Benchmark early**: Ensure we're actually faster
4. **Document quirks**: Add comments for non-obvious handling
5. **Validate compatibility**: Compare with ExifTool output

## Key ExifTool Files to Reference

When implementing features, check these ExifTool files in `../exiftool/`

- `lib/Image/ExifTool/Exif.pm` - Core EXIF handling
- `lib/Image/ExifTool/JPEG.pm` - JPEG segment parsing
- `lib/Image/ExifTool/Canon.pm` - Example manufacturer module
- `lib/Image/ExifTool.pm` - ProcessBinaryData and core functions
- `t/` - Test files show edge cases

## Future Considerations

### Table Generation

- Perl hash syntax is regular enough to parse
- Watch for special keys (PROCESS_PROC, WRITE_PROC)
- Conditions can be complex Perl expressions
- Some conversions are inline Perl code

### Write Support

- Must preserve unknown tags
- Maker notes often have checksums
- Some cameras require specific tag order
- In-place updates are risky - consider temp file

### Performance Optimizations

- SIMD for endian swapping (packed data)
- Parallel IFD processing (independent chains)
- String interning for common values
- Zero-copy for large binary data

## Commands

```bash
# Build and test
cargo build
cargo test

# Run specific test
cargo test test_canon_image

# Test with real images
cargo run --bin exif-oxide -- test-images/canon/Canon_T3i.JPG

# Debug EXIF parsing
cargo run --bin debug_exif -- test-images/canon/Canon_T3i.JPG

# Test thumbnail extraction
cargo run --bin test_canon_image
cargo test --test spike3

# Test XMP extraction
cargo run --example read_xmp test-images/canon/Canon_T3i.JPG
cargo run --example debug_xmp_extraction test-images/canon/Canon_T3i.JPG
cargo test --test spike4_xmp

# Debug JPEG segments (including XMP)
cargo run --example debug_jpeg_segments test-images/canon/Canon_T3i.JPG

# Benchmark vs ExifTool
cargo bench

# Test with ExifTool for comparison
./exiftool/exiftool -struct -json test.jpg > exiftool-output.json
cargo run --bin exif-oxide -- test.jpg > exif-oxide-output.json

# Generate tag tables (future)
cargo run --bin table_converter -- ../exiftool/lib/Image/ExifTool/Canon.pm

# Fuzz testing (future)
cargo fuzz run parse_jpeg
```

## Recent Achievements

### Spike 4: XMP Reading Phase 1 (COMPLETE)

- Successfully implemented XMP detection in JPEG APP1 segments
- Basic XML parsing for attribute-based properties
- Namespace registry with common namespaces (dc, xmp, tiff, exif, etc.)
- Simple key-value extraction from XMP packets
- Integration with existing JPEG parsing infrastructure
- Comprehensive test suite with multiple namespace support

### Spike 3: Binary Tag Extraction (COMPLETE)

- Successfully implemented IFD1 parsing for thumbnail extraction
- Cross-manufacturer thumbnail support (Canon, Nikon, Sony, Panasonic)
- Flexible format parsing handling Undefined vs U32 storage
- JPEG boundary detection with SOI/EOI marker trimming
- Performance under 8ms for thumbnail extraction
- Comprehensive test suite with real-world images

### Key Technical Breakthroughs

1. **XMP Packet Detection**: Proper handling of XMP signature in APP1 segments
2. **XML Parsing Strategy**: Using quick-xml for efficient streaming parse
3. **Namespace Handling**: Registry-based approach for expanding prefixes
4. **Integration Design**: XMP as parallel metadata stream to EXIF
5. **Offset Interpretation**: Discovered 12-byte headers before JPEG data in some cameras
6. **Format Flexibility**: Tags stored as Undefined format require numeric coercion
7. **Universal Extraction**: Same logic works across all major manufacturers
8. **JPEG Validation**: Proper SOI marker detection and EOI trimming

## Implementation Status

### Completed Components

- `core::jpeg` - JPEG APP1 segment extraction (EXIF and XMP)
- `core::ifd` - Complete IFD parsing with IFD0/IFD1 and endian support
- `core::types` - EXIF format definitions
- `core::endian` - Byte order handling
- `maker::canon` - Canon maker notes parsing
- `extract::thumbnail` - Thumbnail extraction from IFD1
- `extract::preview` - Preview image extraction framework
- `tables::mod` - Generated tag tables from ExifTool Perl modules
- `xmp::parser` - Basic XMP XML parsing for attributes
- `xmp::reader` - XMP extraction from JPEG files
- `xmp::types` - XMP data structures and namespace registry

### Key Implementation Decisions

1. **Direct binary parsing** - No nom dependency, transparent and efficient
2. **HashMap for tag storage** - Allows O(1) lookup by tag ID
3. **Separate JPEG and IFD parsing** - Clean separation of concerns
4. **Table-driven architecture** - Generated from ExifTool's Perl modules
5. **Flexible format parsing** - get_numeric_u32 handles various storage formats
6. **Memory-efficient extraction** - Zero-copy where possible for binary data
7. **Cross-manufacturer compatibility** - Same extraction logic works for all brands

### Testing with ExifTool Images

The ExifTool test suite in `exiftool/t/images/` provides excellent test coverage:

- Different manufacturers (Canon, Nikon, Fujifilm, etc.)
- Various byte orders (little-endian and big-endian)
- Edge cases and malformed data

Note: `ExifTool.jpg` contains FUJIFILM EXIF data but ExifTool reports Canon - this suggests ExifTool uses additional metadata sources beyond basic EXIF.

## Remember

The user is new to rust: be sure to write idiomatic, elegant rust and explain strategies as you go.

This project stands on the shoulders of giants:

- Phil Harvey's 25 years of ExifTool development
- The exiftool-vendored datetime heuristics
- The Rust community's excellent parsing libraries

Respect the complexity that cameras have introduced over decades. What seems like a bug might be a workaround for a specific camera model from 2003.
