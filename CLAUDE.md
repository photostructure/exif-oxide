# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Context

exif-oxide is a high-performance Rust implementation of Phil Harvey's [ExifTool](https://exiftool.org/), focusing on:

- Performance (10-20x faster than Perl)
- Memory safety for untrusted files
- Embedded image extraction (thumbnails/previews)
- DateTime parsing intelligence from [exiftool-vendored](https://github.com/photostructure/exiftool-vendored.js)

## Development Philosophy

### 1. Respect ExifTool's Legacy

- Phil Harvey has been developing ExifTool for 25 years
- The Perl codebase contains invaluable camera-specific quirks and edge cases
- We're not inventing anything here -- **how ExifTool handles it is the correct way**
- Maintain tag name and structure compatibility

### 2. Incremental Development

- Follow the spike/phase plan in SPIKES-*.md or TODO-*.md
- Each spike should be independently testable
- Don't over-engineer early spikes - learn first, optimize later
- Document surprises and gotchas as you discover them

### 3. Current Development Status (December 2024)

**✅ COMPLETED**: All core foundation spikes (1-6)  
**🔄 CURRENT**: Phase 1 - Multi-format read support (beyond JPEG)  
**⏳ NEXT**: Phase 2 - Maker note expansion for all manufacturers

**Key Limitation**: main.rs is hardcoded to JPEG despite having detection for 43 formats

### 4. Testing Approach

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

### All Core Spikes COMPLETE

**Status**: 6/6 core spikes complete, moving to multi-format development phases

**Spike 1: Basic EXIF Tag Reading (COMPLETE)**
- Core JPEG APP1 segment parsing 
- IFD structure parsing with both endianness support
- Make, Model, Orientation extraction working
- Full test coverage with real-world images

**Spike 1.5: Minimal Table Generation (COMPLETE)**
- Build-time code generation from ExifTool's Perl modules
- 496 EXIF tags parsed from Exif.pm with format and group info
- 34 Canon-specific tags from Canon.pm
- Static lookup tables with O(1) tag access

**Spike 2: Maker Note Parsing (COMPLETE)**
- Canon maker note IFD parsing in ExifIFD
- Manufacturer detection and dispatch system
- 28 Canon tags successfully extracted from test images
- Trait-based extensible architecture for other manufacturers

**Spike 3: Binary Tag Extraction (COMPLETE)**
- IFD1 thumbnail extraction across all manufacturers
- Canon preview image extraction from maker notes
- JPEG validation with SOI/EOI marker detection
- Memory-efficient streaming extraction <8ms performance
- Universal extraction logic working on Canon, Nikon, Sony, Panasonic

**Spike 4: XMP Reading Phases 1 & 2 (COMPLETE)**
- Complete XMP packet detection in JPEG APP1 segments
- Advanced XML parsing with hierarchical data structures
- RDF array support (Seq, Bag, Alt) with language alternatives
- UTF-16 encoding support for international content
- Namespace registry with dynamic expansion
- 39 comprehensive tests covering edge cases and error handling

**Spike 5: File Type Detection System (COMPLETE)**
- Universal format detection for 43 file formats with 100% ExifTool MIME compatibility
- Auto-generated magic number patterns from ExifTool's Perl source
- TIFF-based RAW format differentiation via Make/Model tag parsing
- Container format support (QuickTime, RIFF, MP4) with brand detection
- Sub-1ms performance using only first 1KB of data

**Spike 6: DateTime Intelligence (90% COMPLETE)**
- Multi-source datetime extraction from EXIF, XMP, GPS, and manufacturer tags
- GPS coordinate-based timezone inference with confidence scoring
- Manufacturer-specific quirk handling (Nikon DST, Canon formats, Apple variations)
- UTC delta calculation and multi-source validation framework
- Integration with public API (BasicExif struct extended with resolved_datetime field)

### Key Technical Breakthroughs

1. **Table-Driven Architecture**: Automated generation from ExifTool Perl sources
2. **Universal Binary Extraction**: Same logic across all camera manufacturers
3. **Advanced XMP Support**: Full hierarchical parsing beyond basic attributes
4. **Robust Error Handling**: Graceful degradation with malformed data
5. **Performance Optimization**: Sub-10ms parsing for typical JPEG files
6. **Cross-Manufacturer Compatibility**: Tested with Canon, Nikon, Sony, Fujifilm
7. **Memory Safety**: Bounds checking and zero-copy optimizations
8. **ExifTool Compatibility**: Direct translation of 25 years of accumulated knowledge
9. **Format Detection Excellence**: 43 formats detected with sub-1ms performance
10. **DateTime Intelligence**: GPS timezone inference and manufacturer quirk handling

## Implementation Status

### Completed Components

- `core::jpeg` - JPEG APP1 segment extraction (EXIF and XMP)
- `core::ifd` - Complete IFD parsing with IFD0/IFD1 and endian support
- `core::types` - EXIF format definitions with all data types
- `core::endian` - Byte order handling
- `maker::canon` - Canon maker notes parsing with 34 tags
- `maker::mod` - Manufacturer detection and dispatch system
- `extract::thumbnail` - Universal thumbnail extraction from IFD1
- `extract::preview` - Canon preview image extraction from maker notes
- `extract::mod` - Unified extraction API with largest preview selection
- `tables::mod` - Generated tag tables from ExifTool Perl modules (530 total tags)
- `xmp::parser` - Advanced XMP XML parsing with hierarchical structures
- `xmp::reader` - XMP extraction from JPEG files
- `xmp::types` - Complete XMP data structures with arrays and language support
- `xmp::namespace` - Dynamic namespace registry
- `error` - Comprehensive error handling for all components
- `detection::mod` - File type detection system supporting 43 formats
- `detection::magic_numbers` - Auto-generated magic number patterns from ExifTool  
- `detection::tiff_raw` - TIFF-based RAW format detection via manufacturer tags
- `datetime::mod` - DateTime intelligence framework with timezone inference
- `datetime::intelligence` - Multi-source datetime analysis and validation
- `datetime::gps_timezone` - GPS coordinate-based timezone lookup
- `datetime::quirks` - Manufacturer-specific datetime corrections

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

### Design for future updates

See doc/EXIFTOOL-SYNC.md