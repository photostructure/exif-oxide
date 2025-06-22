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

# Benchmark vs ExifTool
cargo bench

# Test with ExifTool for comparison
exiftool -j test.jpg > exiftool-output.json
cargo run -- test.jpg > exif-oxide-output.json

# Generate tag tables (future)
cargo run --bin table_converter -- ../exiftool/lib/Image/ExifTool/Canon.pm

# Fuzz testing (future)
cargo fuzz run parse_jpeg
```

## Remember

This project stands on the shoulders of giants:
- Phil Harvey's 25 years of ExifTool development
- The exiftool-vendored datetime heuristics
- The Rust community's excellent parsing libraries

Respect the complexity that cameras have introduced over decades. What seems like a bug might be a workaround for a specific camera model from 2003.