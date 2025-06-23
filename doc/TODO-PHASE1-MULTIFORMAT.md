# Phase 1: Multi-Format Read Support

**Goal**: Support reading metadata from all 43 currently-detected file formats, not just JPEG.

**Duration**: 2-3 weeks

**Dependencies**: Spike 6 (DateTime Intelligence) completion

## IMMEDIATE (Critical Path - 1 week)

### 1. Core Parser Extension  
**Context**: main.rs is hardcoded to JPEG despite having detection for 43 formats.

**Files to modify**:
- `src/core/tiff.rs` (new) - TIFF/IFD parsing for RAW formats
- `src/core/heif.rs` (new) - HEIF/HEIC container parsing  
- `src/core/png.rs` (new) - PNG chunk parsing for metadata
- `src/lib.rs` - Add format-agnostic API
- `src/main.rs` - Replace JPEG-only logic with format dispatch

**Implementation pattern**:
```rust
// Follow the existing pattern in jpeg.rs:
pub fn find_metadata_segment(file: &mut File, format: FileType) -> Result<Option<MetadataSegment>> {
    match format {
        FileType::JPEG => jpeg::find_exif_segment(file),
        FileType::TIFF | FileType::CR2 | FileType::NEF => tiff::find_ifd_data(file),
        FileType::HEIF | FileType::HEIC => heif::find_exif_box(file),
        FileType::PNG => png::find_exif_chunk(file),
        // ... 
    }
}
```

**Testing command**: `cargo test multiformat && cargo run -- test-images/various-formats/`

### 2. TIFF-based RAW Format Support
**Context**: 16+ RAW formats (CR2, NEF, ARW, etc.) all use TIFF structure but need format-specific handling.

**Reference implementation**: Review `src/detection/tiff_raw.rs` for manufacturer detection patterns.

**Files to create**:
- `src/core/tiff.rs` - Universal TIFF/IFD parser
- Tests with ExifTool images: `Canon.tif`, `Nikon.tif`, `Sony.arw`

**Key challenge**: Handle manufacturer-specific IFD offsets and structures.

## SHORT-TERM (Enhanced support - 1 week)

### 3. Container Format Parsers
**Context**: AVI, MOV, MP4, WebP use container formats (RIFF, QuickTime) with embedded metadata.

**Reference pattern**: Follow `src/detection/mod.rs` QuickTime brand detection logic.

**Files to create**:
- `src/core/containers/riff.rs` - RIFF container parsing (AVI, WebP)
- `src/core/containers/quicktime.rs` - QuickTime/MP4 atom parsing
- `src/core/containers/mod.rs` - Container dispatch logic

**Implementation approach**:
1. Parse container structure to locate metadata atoms/chunks
2. Extract EXIF data from located positions  
3. Handle format-specific quirks (WebP VP8X chunk, MP4 meta atom)

### 4. PNG Metadata Support
**Context**: PNG stores EXIF in specific chunks, different from JPEG APP1 segments.

**Files to create**:
- `src/core/png.rs` - PNG chunk parsing for eXIf chunk

**PNG-specific handling**:
- Find eXIf chunk (contains raw EXIF data)
- Extract and parse IFD structure from chunk data
- Handle PNG-specific metadata chunks (tEXt, iTXt)

## MEDIUM-TERM (Comprehensive coverage - 1 week)

### 5. Integration & Format Dispatch
**Context**: Unify all format parsers into single API that works across all 43 detected formats.

**Files to modify**:
- `src/lib.rs` - Replace `read_basic_exif()` with `read_metadata()` 
- `src/main.rs` - Remove JPEG assumptions, use format detection
- Add comprehensive integration tests

**New public API**:
```rust
pub fn read_metadata<P: AsRef<Path>>(path: P) -> Result<Metadata> {
    let format = detect_file_type(&path)?;
    let metadata_segment = find_metadata_segment(&path, format)?;
    // Parse based on format...
}
```

### 6. Performance Optimization
**Context**: Ensure multi-format support doesn't slow down common JPEG use case.

**Optimization targets**:
- Lazy format detection (only read 1KB for detection)
- Format-specific fast paths (JPEG bypass container parsing)
- Memory-efficient container parsing (streaming vs loading entire file)

**Benchmarks to add**:
```bash
# Existing JPEG performance should not regress
cargo bench jpeg_parsing

# New formats should be reasonable
cargo bench tiff_parsing 
cargo bench heif_parsing
```

## LONG-TERM (Production polish - ongoing)

### 7. Comprehensive Format Testing
**Context**: Validate against ExifTool test suite for format compatibility.

**Test matrix**:
- All 43 detected formats with ExifTool comparison
- Error handling for malformed files
- Performance benchmarks vs ExifTool

**ExifTool test images to use**:
- `exiftool/t/images/` directory has comprehensive format coverage
- Focus on formats with significant metadata (not just magic number detection)

### 8. Error Handling & Edge Cases
**Context**: Robust handling of malformed files and missing metadata.

**Edge cases to handle**:
- Files with correct magic numbers but corrupted metadata
- Formats with multiple metadata locations (JPEG + EXIF + XMP)
- Container formats with nested structures
- Large files requiring streaming parsing

## Technical Notes

### Format Priority Order
1. **High Priority**: JPEG, TIFF, HEIF, PNG (90% of use cases)
2. **Medium Priority**: CR2, NEF, ARW, MP4, MOV (professional workflows)  
3. **Lower Priority**: Remaining RAW formats, video containers

### Parser Architecture
- **Follow existing patterns**: jpeg.rs structure, error handling, API design
- **Reuse detection logic**: Don't duplicate format detection, use existing `src/detection/`
- **Maintain performance**: JPEG parsing speed should not regress

### ExifTool Compatibility  
- **Format support**: Match ExifTool's format coverage where we have detection
- **Metadata extraction**: Same tags extracted from same locations
- **Error behavior**: Graceful degradation like ExifTool

## Success Criteria
- [ ] All 43 detected formats can be parsed (even if just basic metadata)
- [ ] JPEG performance does not regress
- [ ] 95%+ compatibility with ExifTool for supported formats
- [ ] Clean, consistent API across all formats
- [ ] Comprehensive test coverage with real-world files