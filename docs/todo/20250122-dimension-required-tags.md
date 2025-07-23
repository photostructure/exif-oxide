# Technical Project Plan: File System Required Tags Implementation

## Project Overview

- **Goal**: Ensure all image dimension tags required by PhotoStructure are properly extracted across all supported formats

## Background & Context

PhotoStructure requires reliable image dimensions for proper display and organization. These dimensions must be extracted directly from file structure (not just EXIF) to work even with corrupted metadata.

## Technical Foundation

Study the entirety of the documentation, and study referenced relevant source code.

- [CLAUDE.md](CLAUDE.md)
- [TRUST-EXIFTOOL.md](docs/TRUST-EXIFTOOL.md) -- follow their dimension extraction algorithm **precisely**.
- [CODEGEN.md](docs/CODEGEN.md) -- if there's any tabular data, or perl code that you think we could automatically extract and use, **strongly** prefer that to any manual porting effort.

- **Key files**:
  - `src/formats/mod.rs` - Format-specific processing
  - `src/formats/jpeg.rs` - JPEG SOF parsing (✅ COMPLETED)
  - `src/raw/` - RAW format processing

## Required File System Tags

### Core Media Properties (CRITICAL)
- **ImageWidth** ✅ JPEG complete, needs RAW/TIFF/PNG/WebP/Video
- **ImageHeight** ✅ JPEG complete, needs RAW/TIFF/PNG/WebP/Video

### Extended Properties (HIGH VALUE)
- **BitsPerSample** ✅ JPEG complete, needs RAW/TIFF/PNG/WebP/Video
- **ColorComponents** ✅ JPEG complete, needs RAW/TIFF/PNG/WebP/Video
- **YCbCrSubSampling** ✅ JPEG complete, needs RAW when applicable
- **EncodingProcess** ✅ JPEG complete, needs RAW when applicable

## ✅ COMPLETED: JPEG Implementation

Successfully implemented all dimension tags for JPEG files by parsing SOF (Start of Frame) markers:

- **Location**: `src/formats/jpeg.rs` - `parse_sof_data()` and `scan_jpeg_segments()`
- **Method**: Extract from SOF0-SOF15 markers (0xC0-0xCF except DHT/JPGA/DAC)
- **ExifTool Reference**: `lib/Image/ExifTool.pm:7321-7336`
- **Binary Format**: `unpack('Cn2C', data)` - precision, height, width, components
- **Testing**: Verified with Nikon Z8 (8256×5504) and Canon T3i (5184×3456)

## Remaining Tasks

### High Priority - RAW Format Support

RAW formats store dimensions in TIFF-like structures but locations vary by manufacturer:

#### 1. **Sony ARW** (Priority: HIGH - PhotoStructure common) - IN PROGRESS
   - **STATUS**: 90% complete - shared utility created, processing path identified, needs final integration
   - **Key Discovery**: ARW files are processed through TIFF branch in formats/mod.rs, NOT RAW branch
   - **Implementation Created**: `raw::utils::extract_tiff_dimensions()` shared utility (src/raw/mod.rs:112-298)
   - **Expected Output**: EXIF:ImageWidth=7040, EXIF:ImageHeight=4688 (from TIFF IFD0 tags 0x0100/0x0101)
   - **Test File**: `test-images/sony/sony_a7c_ii_02.arw`
   - **REMAINING TASK**: Add `crate::raw::utils::extract_tiff_dimensions(&mut exif_reader, &tiff_data)?;` 
     call in formats/mod.rs TIFF branch (line ~469) after successful `exif_reader.parse_exif_data(&tiff_data)`
   - **Verification**: Output should match ExifTool: `exiftool -j -G test-images/sony/sony_a7c_ii_02.arw | grep ImageWidth`

#### 2. **Canon CR2** (Priority: HIGH - PhotoStructure common) - PARALLEL DEVELOPMENT
   - **STATUS**: Being worked on in concurrent session - shared utility ready
   - **Implementation Available**: Same `raw::utils::extract_tiff_dimensions()` utility can be used
   - **Architecture Issue**: Canon handler in src/raw/formats/canon.rs needs `data` parameter access
   - **Integration Point**: Canon can use same TIFF branch integration as Sony ARW
   - **Note**: Current Canon implementation attempts to call dimension extraction but lacks data parameter

#### 3. **Nikon NEF** (Priority: MEDIUM)
   - **Research needed**: NEF structure and dimension storage
   - **Likely location**: TIFF IFD tags or Nikon MakerNotes
   - **ExifTool reference**: `lib/Image/ExifTool/Nikon.pm`
   - **Test with**: Any NEF files in test-images

#### 4. **Other RAW formats** (Priority: MEDIUM)
   - ORF (Olympus), RW2 (Panasonic), MRW (Minolta), RWL
   - Follow same pattern: research ExifTool source, find dimension storage, extract

### Medium Priority - Additional Formats

#### 5. **TIFF Files** (Priority: MEDIUM)
   - **Method**: Extract from TIFF IFD tags 0x0100 (ImageWidth) and 0x0101 (ImageLength)
   - **Location**: Should work in existing TIFF processing in `src/formats/mod.rs`
   - **Implementation**: Likely just needs to extract these specific tags to File group

#### 6. **PNG Files** (Priority: MEDIUM)
   - **Method**: Parse PNG IHDR chunk (first chunk after PNG signature)
   - **Format**: Width (4 bytes), Height (4 bytes), bit depth, color type, etc.
   - **ExifTool reference**: `lib/Image/ExifTool/PNG.pm`
   - **Implementation**: Add PNG format detection and IHDR parsing

#### 7. **WebP Files** (Priority: LOW)
   - **Method**: Parse WebP VP8/VP8L/VP8X chunk headers
   - **ExifTool reference**: `lib/Image/ExifTool/RIFF.pm` (WebP is RIFF-based)
   - **Implementation**: Add WebP format detection and chunk parsing

#### 8. **Video Files** (Priority: LOW - if PhotoStructure needs it)
   - **Method**: Parse container headers (MP4, MOV, AVI, etc.)
   - **Complexity**: High - each container format is different
   - **ExifTool reference**: Various modules (QuickTime.pm, RIFF.pm, etc.)

## Implementation Guide

### Pattern for Adding New Format Support

1. **Research ExifTool Implementation**
   - Find relevant module: `third-party/exiftool/lib/Image/ExifTool/[Format].pm`
   - Look for dimension extraction logic
   - Note any special cases or format quirks

2. **Add Format Detection** (if needed)
   - Update `src/file_detection.rs` with magic bytes/file signatures
   - Add to `src/formats/detection.rs` format enum

3. **Add Dimension Extraction**
   - For RAW: Update existing RAW processor in `src/raw/`
   - For others: Add format-specific parsing in `src/formats/`
   - Extract to File group tags (not EXIF group)

4. **Add TagEntry Creation**
   - Follow pattern in `src/formats/mod.rs` JPEG section (lines 285-341)
   - Use `TagValue::String(dimension.to_string())` for numbers
   - Set `group: "File"` and `group1: "File"`

5. **Testing**
   - Test with `cargo run -- [test_file]`
   - Compare with `exiftool -j -struct -G [test_file]`
   - Ensure File:ImageWidth and File:ImageHeight match exactly

### Key ExifTool References by Format

- **JPEG**: `lib/Image/ExifTool.pm:7321-7336` (SOF parsing)
- **TIFF**: `lib/Image/ExifTool/Exif.pm` tags 0x0100, 0x0101
- **PNG**: `lib/Image/ExifTool/PNG.pm` IHDR chunk parsing
- **Sony ARW**: `lib/Image/ExifTool/Sony.pm`
- **Canon CR2**: `lib/Image/ExifTool/Canon.pm` 
- **Nikon NEF**: `lib/Image/ExifTool/Nikon.pm`
- **WebP**: `lib/Image/ExifTool/RIFF.pm` (WebP is RIFF container)

### Critical TIFF Tag IDs

- **0x0100**: ImageWidth (TIFF standard)
- **0x0101**: ImageLength/ImageHeight (TIFF standard)  
- **0x0102**: BitsPerSample
- **0x0115**: SamplesPerPixel (similar to ColorComponents)
- **0x0212**: YCbCrSubSampling

## Testing Strategy

### Verify with Real Files
- Use files from `test-images/` directory
- Compare all File: group tags with ExifTool: `exiftool -j -struct -G [file]`
- Test edge cases: corrupted EXIF, thumbnail-only files, unusual dimensions

### Compatibility Testing
- Run `make compat` to verify no regressions
- Ensure dimensions work even when EXIF tags are missing
- Test across different file sizes (small thumbnails, large originals)

## Success Criteria

- **File:ImageWidth** and **File:ImageHeight** extracted for all major formats
- Values match ExifTool's `exiftool -j -struct -G` output exactly
- Dimensions available even with corrupted/missing EXIF data
- Proper error handling for malformed files
- All compatibility tests pass (`make compat`)

## Gotchas & Tribal Knowledge

### Image Dimensions
- **JPEG**: ✅ Read from SOF0-SOF15 markers, not EXIF (completed)
- **TIFF/RAW**: Read from IFD0 tags 0x0100/0x0101, may be in MakerNotes  
- **PNG**: Parse IHDR chunk (8 bytes after PNG signature)
- **WebP**: Parse VP8/VP8L/VP8X chunk headers
- **Orientation**: File dimensions are ALWAYS pre-rotation values (raw sensor dimensions)

### RAW Format Specifics
- **Sony ARW**: TIFF-based, dimensions in IFD0 tags 0x0100/0x0101 - **CRITICAL: Processed via TIFF branch, not RAW**
- **Canon CR2**: TIFF-based, uses standard TIFF tags - **Same processing path as ARW**
- **Nikon NEF**: TIFF-based, may use Nikon-specific MakerNote tags
- **All RAW**: May have multiple dimension sets (sensor, preview, thumbnail) - use largest

### Architecture Discovery (CRITICAL)
- **ARW Processing Path**: ARW files are detected as TIFF format and processed through formats/mod.rs TIFF branch (line 454), NOT through RAW processor
- **File Detection**: `file test-images/sony/sony_a7c_ii_02.arw` shows "TIFF image data" - explains processing path
- **Integration Point**: Dimension extraction must be added to TIFF branch, not RAW handlers
- **Shared Utility**: `raw::utils::extract_tiff_dimensions()` works for both Sony ARW and Canon CR2

### TIFF Endianness
- **"II"** = Little-endian (Intel byte order)
- **"MM"** = Big-endian (Motorola byte order)
- Must read TIFF header first to determine byte order for dimension tags

### Binary Data Parsing
- **JPEG SOF**: `unpack('Cn2C')` = precision(u8), height(u16be), width(u16be), components(u8)
- **PNG IHDR**: Width(u32be), Height(u32be), BitDepth(u8), ColorType(u8), etc.
- **TIFF IFD**: Tag entries are 12 bytes, value depends on data type and count

### Performance Notes
- Dimension extraction should be fast (file header parsing only)
- Don't parse entire EXIF structure just for dimensions
- Cache parsed dimensions to avoid re-reading file headers