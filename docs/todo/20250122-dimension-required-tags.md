# Technical Project Plan: File System Required Tags Implementation

## Project Overview

- **Goal**: Ensure all image dimension tags required by PhotoStructure are properly extracted across all supported formats

### 🎯 **Current Progress Status** (July 23, 2025)

**✅ Core Web Formats Completed (4/6)**:
- ✅ **JPEG** - Full dimension extraction from SOF markers (numeric output ✅)
- ✅ **PNG** - Full dimension extraction from IHDR chunks (numeric output ✅)
- ✅ **GIF** - Full dimension extraction from Logical Screen Descriptor (numeric output ✅)
- ✅ **TIFF** - COMPLETED via existing TIFF processing (numeric output ✅)
- 🔲 **SVG** - Requires XML parsing
- 🔲 **WebP** - RIFF-based format

**✅ RAW Formats Completed (2/30+)**:
- ✅ **Canon CR2** - TIFF-based extraction (numeric output ✅)
- ✅ **Sony ARW** - Enhanced SubIFD support (numeric output ✅)

**📊 Overall Status**: Excellent progress with 4/6 core web formats complete (JPEG + PNG + GIF + TIFF), AVIF modern format, and major RAW formats (Canon + Sony). All dimension tags now output as numbers matching ExifTool format. WebP and SVG remain for full web format coverage.

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

## Required File System Tags (PhotoStructure Integration Requirements)

### Core Media Properties (CRITICAL)
- **ImageWidth** ✅ JPEG complete, ✅ PNG complete, ✅ GIF complete, ✅ RAW complete (Sony ARW, Canon CR2), needs remaining PhotoStructure formats below
- **ImageHeight** ✅ JPEG complete, ✅ PNG complete, ✅ GIF complete, ✅ RAW complete (Sony ARW, Canon CR2), needs remaining PhotoStructure formats below

### Extended Properties (HIGH VALUE)
- **BitsPerSample** ✅ JPEG complete, needs RAW/TIFF/PNG/WebP/Video
- **ColorComponents** ✅ JPEG complete, needs RAW/TIFF/PNG/WebP/Video
- **YCbCrSubSampling** ✅ JPEG complete, needs RAW when applicable
- **EncodingProcess** ✅ JPEG complete, needs RAW when applicable

## PhotoStructure Format Requirements

All formats below require ImageWidth/ImageHeight extraction for PhotoStructure integration:

### **SharpImageFiletypes** (Standard Web Images)
- ✅ **JPEG** (`image/jpeg`: jpg, jpeg, jpe, jfif, jfi) - COMPLETED
- ✅ **PNG** (`image/png`: png) - COMPLETED (July 23, 2025)
- ✅ **GIF** (`image/gif`: gif) - COMPLETED (July 23, 2025)
- ✅ **TIFF** (`image/tiff`: tif, tiff) - COMPLETED (July 23, 2025)
- 🔲 **WebP** (`image/webp`: webp) - RIFF-based format

### **HeifFiletypes** (Modern Efficient Formats)
- ✅ **AVIF** (`image/avif`: avif) - COMPLETED (July 23, 2025)
- ⚠️ **HEIC** (`image/heic`: heic) - BASIC IMPLEMENTATION (July 24, 2025) - extracts first ispe box, needs primary item detection for exact ExifTool match
- ⚠️ **HEIF** (`image/heif`: heif) - BASIC IMPLEMENTATION (July 24, 2025) - extracts first ispe box, needs primary item detection for exact ExifTool match

### **RawImageFiletypes** (Camera RAW Formats)
#### ✅ **COMPLETED** (2/30+ formats)
- ✅ **Canon CR2** (`image/x-canon-cr2`: cr2) - COMPLETED (numeric output ✅)
- ✅ **Sony ARW** (`image/x-sony-arw`: arw) - COMPLETED (numeric output ✅)

#### 🔲 **HIGH PRIORITY** (Common formats)
- 🔲 **Adobe DNG** (`image/x-adobe-dng`: dng) - Universal RAW standard
- 🔲 **Canon CR3** (`image/x-canon-cr3`: cr3) - Modern Canon RAW
- 🔲 **Nikon NEF** (`image/x-nikon-nef`: nef) - Major manufacturer
- 🔲 **Fuji RAF** (`image/x-fuji-raf`: raf) - Popular mirrorless
- 🔲 **Olympus ORF** (`image/x-olympus-orf`: orf) - Micro Four Thirds
- 🔲 **Panasonic RW2** (`image/x-panasonic-rw2`: rw2) - Micro Four Thirds

#### 🔲 **MEDIUM PRIORITY** (Less common)
- 🔲 **Canon CRW** (`image/x-canon-crw`: crw) - Legacy Canon
- 🔲 **Epson ERF** (`image/x-epson-erf`: erf) 
- 🔲 **Hasselblad 3FR** (`image/x-hasselblad-3fr`: 3fr) - Medium format
- 🔲 **Kodak formats** (`image/x-kodak-dcr`: dcr, `image/x-kodak-k25`: k25, `image/x-kodak-kdc`: kdc)
- 🔲 **Mamiya MEF** (`image/x-mamiya-mef`: mef) - Medium format
- 🔲 **Minolta MRW** (`image/x-minolta-mrw`: mrw)
- 🔲 **Nikon NRW** (`image/x-nikon-nrw`: nrw) - Compact RAW
- 🔲 **Panasonic RWL** (`image/x-panasonic-raw`: rwl) 
- 🔲 **Pentax PEF** (`image/x-pentax-pef`: pef, dng)
- 🔲 **Samsung SRW** (`image/x-samsung-srw`: srw)
- 🔲 **Sigma X3F** (`image/x-sigma-x3f`: x3f) - Foveon sensor
- 🔲 **Sony SR2/SRF** (`image/x-sony-sr2`: sr2, `image/x-sony-srf`: srf) - Legacy Sony
- 🔲 **Generic RAW** (`image/x-raw`: raw) - LEICA and Panasonic

### **VideoFiletypes** (Video Containers)
- 🔲 **MP4** (`video/mp4`: mp4, insv) - Most common
- 🔲 **QuickTime MOV** (`video/quicktime`: mov, qt) - Apple format
- 🔲 **AVI** (`video/x-msvideo`: avi) - Legacy Windows
- 🔲 **MKV** (`video/mkv`: mkv) - Open container
- 🔲 **WebM** (`video/webm`: webm) - Web standard
- 🔲 **MPEG** (`video/mpeg`: m2v, mpeg, mpg) - Legacy standard
- 🔲 **3GPP** (`video/3gpp`: 3gp, 3gpp, `video/3gpp2`: 3g2) - Mobile
- 🔲 **M2TS** (`video/mp2t`: mts, ts) - Broadcast
- 🔲 **M4V** (`video/x-m4v`: m4v) - iTunes format
- 🔲 **WMV** (`video/x-ms-wmv`: wmv, `video/x-ms-asf`: asf) - Microsoft
- 🔲 **MNG** (`video/x-mng`: mng) - Animated PNG

## ✅ COMPLETED: Core Image Format Support

### JPEG Implementation ✅ **COMPLETED**

Successfully implemented all dimension tags for JPEG files by parsing SOF (Start of Frame) markers:

- **Location**: `src/formats/jpeg.rs` - `parse_sof_data()` and `scan_jpeg_segments()`
- **Method**: Extract from SOF0-SOF15 markers (0xC0-0xCF except DHT/JPGA/DAC)
- **ExifTool Reference**: `lib/Image/ExifTool.pm:7321-7336`
- **Binary Format**: `unpack('Cn2C', data)` - precision, height, width, components
- **Testing**: Verified with Nikon Z8 (8256×5504) and Canon T3i (5184×3456)

### PNG Implementation ✅ **COMPLETED** (July 23, 2025)

Successfully implemented PNG dimension extraction by parsing IHDR chunks:

- **Location**: `src/formats/png.rs` - `parse_png_ihdr()` and `create_png_tag_entries()`
- **Method**: Extract from PNG IHDR chunk (first chunk after PNG signature)
- **ExifTool Reference**: `lib/Image/ExifTool/PNG.pm:387-423` (ImageHeader table)
- **Binary Format**: Width(u32be), Height(u32be), BitDepth(u8), ColorType(u8), etc.
- **Tags Extracted**: PNG:ImageWidth, PNG:ImageHeight, PNG:BitDepth, PNG:ColorType, PNG:Compression, PNG:Filter, PNG:Interlace  
- **Testing**: Verified with 3 PNG files including `test-images/example.png` (1130×726 Palette)
- **Integration**: Added PNG case to `src/formats/mod.rs:758` format dispatch
- **Group Assignment**: PNG tags use "PNG" group (not "File" group like JPEG)
- **Compatibility**: Added PNG support to test infrastructure and `config/supported_tags.json`

### GIF Implementation ✅ **COMPLETED** (July 23, 2025)

Successfully implemented GIF dimension extraction by parsing Logical Screen Descriptor:

- **Location**: `src/formats/gif.rs` - `parse_gif_screen_descriptor()` and `create_gif_tag_entries()`
- **Method**: Extract from GIF Logical Screen Descriptor (7 bytes after 6-byte signature)
- **ExifTool Reference**: `lib/Image/ExifTool/GIF.pm:105-138` (Screen table)
- **Binary Format**: Width(u16le), Height(u16le), Flags(u8), Background(u8), PixelAspectRatio(u8)
- **Tags Extracted**: GIF:ImageWidth, GIF:ImageHeight, GIF:HasColorMap, GIF:ColorResolutionDepth, GIF:BitsPerPixel, GIF:BackgroundColor, GIF:PixelAspectRatio
- **Testing**: Verified with `test-images/example.gif` (663×475 animated GIF with 256-color palette)
- **Integration**: Added GIF case to `src/formats/mod.rs:800` format dispatch  
- **Group Assignment**: GIF tags use "GIF" group (following ExifTool's group assignment)
- **ExifTool Compliance**: Matches ExifTool output exactly for all extracted tags (dimensions as numbers)

## ✅ COMPLETED: RAW Format Implementation (July 23, 2025)

Successfully implemented dimension extraction for major RAW formats using shared TIFF utility:

- **Location**: `src/raw/mod.rs::utils::extract_tiff_dimensions()` (lines 112-474)
- **Method**: Extract from TIFF IFD0 tags 0x0100 (ImageWidth) and 0x0101 (ImageHeight) with SubIFD fallback
- **Architecture**: Shared utility called from both RAW handlers and TIFF branch for comprehensive coverage
- **Sony ARW**: Enhanced with SubIFD support (tag 0x014a) to handle Sony's non-standard dimension storage
- **Canon CR2**: Uses standard TIFF IFD0 tags, works through shared utility
- **Integration**: Integrated in `formats/mod.rs` TIFF branch for ARW/CR2/NEF/NRW formats
- **Testing**: Verified Sony A7C II (7040×4688) and Canon T3i (5184×3456) match ExifTool exactly
- **ExifTool Compliance**: Uses identical algorithm to ExifTool's Exif.pm:460-473 implementation
- **✅ DIMENSION SERIALIZATION FIX**: Fixed TagValue type bug - dimensions now output as numbers (`TagValue::U32`) instead of strings (`TagValue::String`), matching ExifTool's JSON format exactly (lines 521, 527)

## ✅ COMPLETED: AVIF Format Implementation (July 23, 2025)

Successfully implemented AVIF (AV1 Image File Format) dimension extraction following ExifTool's exact implementation:

- **Location**: `src/formats/avif.rs` - Complete AVIF processing module with ISO Base Media File Format box parsing
- **Method**: Extract dimensions from 'ispe' (Image Spatial Extent) box within AVIF's ISO container structure
- **ExifTool Reference**: `lib/Image/ExifTool/QuickTime.pm:2946-2959` (ispe box processing)
- **Box Hierarchy**: Navigates `ftyp` → `meta` → `iprp` → `ipco` → `ispe` box structure exactly as ExifTool
- **Binary Format**: Parses ispe box with version/flags validation and big-endian 32-bit width/height extraction
- **Tags Extracted**: File:ImageWidth, File:ImageHeight (corrected to match ExifTool's actual group assignment)
- **Integration**: Added "MOV" format case to `src/formats/mod.rs` with AVIF file_type detection  
- **File Detection**: Enhanced magic byte detection for ISO Base Media File Format with 'avif' brand detection
- **Group Assignment**: AVIF image dimensions assigned to "File" group (matching ExifTool's actual behavior)
- **ExifTool Compliance**: Uses identical box parsing logic and dimension extraction algorithm as ExifTool
- **Testing**: Comprehensive unit tests for box parsing, ispe box processing, and tag entry creation
- **Architecture**: Reusable ISO Base Media File Format parser for future HEIF/MP4/MOV support

## ⚠️ BASIC IMPLEMENTATION: HEIC/HEIF Format Support (July 24, 2025)

Implemented basic HEIC/HEIF dimension extraction with important limitations:

- **Location**: `src/formats/mod.rs:887-941` - HEIC/HEIF processing within MOV format dispatcher
- **Method**: Reuses AVIF's ispe box parsing infrastructure (`avif::extract_avif_dimensions`)
- **ExifTool Reference**: `lib/Image/ExifTool/QuickTime.pm:2946-2959` (ispe box structure identical to AVIF)
- **Tags Extracted**: File:ImageWidth, File:ImageHeight (matching ExifTool's group assignment)
- **✅ Working**: Basic dimension extraction from HEIC/HEIF files
- **⚠️ LIMITATION**: Extracts first ispe box found, which may be a thumbnail instead of main image
- **🔴 MISSING**: ExifTool's primary item detection logic (pitm/iinf/ipma box processing)

### Required for Complete ExifTool Compatibility

To match ExifTool's exact behavior, the following ExifTool logic must be implemented:

1. **Primary Item Detection** (`lib/Image/ExifTool/QuickTime.pm:3550-3557`)
   - Parse `pitm` box to extract primary item ID (version 0: 16-bit, version 1+: 32-bit)
   - Store primary item ID in processing context

2. **Item Information Processing** (`lib/Image/ExifTool/QuickTime.pm:3730-3740`)  
   - Parse `iinf` box to build item information map
   - Track item types, content types, and relationships

3. **Property Association Logic** (`lib/Image/ExifTool/QuickTime.pm:10320-10380`)
   - Parse `ipma` box to link items with properties in `ipco` container
   - Build association maps between item IDs and property indices

4. **DOC_NUM Logic** (`lib/Image/ExifTool/QuickTime.pm:6450-6460`)
   - Only extract dimensions from ispe boxes associated with primary item
   - Skip ispe boxes for thumbnails, previews, or sub-documents
   - Implement: `unless ($$self{DOC_NUM}) { $self->FoundTag(ImageWidth => $dim[0]); }`

### Current Test Results

- **IMG_9757.heic**: Extracts 512x512 (thumbnail) instead of ExifTool's 4032x3024 (main image)
- **IMG_9811.heic**: Extracts 512x512 (thumbnail) instead of ExifTool's 4032x3024 (main image)
- **Detection**: ✅ Correctly identifies HEIC/HEIF files and processes ispe box structure
- **Group Assignment**: ✅ Creates File:ImageWidth and File:ImageHeight tags matching ExifTool

## Implementation Roadmap

### **Phase 1: Core Web Image Formats** (IMMEDIATE PRIORITY)

These formats are essential for web-based photo management and should be implemented first:

#### 1. **PNG** ✅ **COMPLETED** (July 23, 2025)
   - **STATUS**: ✅ **IMPLEMENTATION COMPLETE AND TESTED**
   - **Method**: Parse PNG IHDR chunk (first chunk after PNG signature)
   - **Implementation**: Added PNG processing module `src/formats/png.rs` with IHDR chunk parser
   - **Integration**: Added PNG case to format dispatch in `src/formats/mod.rs:758`
   - **Testing**: Verified with 3 PNG files: `test-images/example.png` (1130×726), `test-images/example-original.png` (1130×726), `third-party/exiftool/t/images/PNG.png` (16×16)
   - **Tags Extracted**: PNG:ImageWidth, PNG:ImageHeight, PNG:BitDepth, PNG:ColorType, PNG:Compression, PNG:Filter, PNG:Interlace
   - **ExifTool Compliance**: ✅ Uses identical algorithm to ExifTool's PNG.pm:387-423 (ImageHeader processing)
   - **Group Assignment**: PNG tags assigned to "PNG" group (not "File" group like JPEG)

#### 2. **GIF** (HIGH PRIORITY)
   - **Why Important**: Still widely used, simple header format
   - **Method**: Parse GIF header (Screen Descriptor)
   - **Format**: Width/Height in logical screen descriptor (bytes 6-9)
   - **ExifTool reference**: `lib/Image/ExifTool/GIF.pm`

#### 3. **WebP** (HIGH PRIORITY) 
   - **Why Important**: Modern web format, growing adoption
   - **Method**: Parse WebP VP8/VP8L/VP8X chunk headers  
   - **ExifTool reference**: `lib/Image/ExifTool/RIFF.pm` (WebP is RIFF-based)

#### 4. **TIFF** ✅ **COMPLETED** (July 23, 2025)
   - **STATUS**: ✅ **IMPLEMENTATION COMPLETE AND TESTED**
   - **Method**: Extract from TIFF IFD tags 0x0100 (ImageWidth) and 0x0101 (ImageLength)
   - **Implementation**: Uses existing TIFF processing via ExifReader in `src/formats/mod.rs:456-472`
   - **Testing**: Verified with `ExifTool.tif` (160×120) and `GeoTiff.tif` (25×24)
   - **Tags Extracted**: EXIF:ImageWidth, EXIF:ImageHeight
   - **ExifTool Compliance**: ✅ Values and group assignment match ExifTool exactly
   - **Group Assignment**: EXIF tags use "EXIF" group (IFD0 tags processed by standard EXIF reader)

#### 5. **SVG** (LOWER PRIORITY)
   - **Method**: Parse XML `<svg>` element width/height attributes
   - **Complexity**: Requires XML parsing, viewBox calculations
   - **Note**: Dimensions may be in various units (px, %, em, etc.)

### **Phase 2: Modern Efficient Formats** (HIGH PRIORITY)

#### 1. **HEIC/HEIF/AVIF** (Apple and modern formats)
   - **Why Important**: Default format for modern iPhones, growing AVIF adoption
   - **Method**: Parse HEIF container, find primary image dimensions
   - **Complexity**: Complex container format, may need specialized library support
   - **ExifTool reference**: `lib/Image/ExifTool/QuickTime.pm` (HEIF uses ISO Base Media File Format)

### **Phase 3: Priority RAW Formats** (MEDIUM-HIGH PRIORITY)

#### 1. **Adobe DNG** (HIGHEST RAW PRIORITY)
   - **Why First**: Universal RAW standard, TIFF-based
   - **Implementation**: Should use existing `extract_tiff_dimensions()` utility

#### 2. **Canon CR3** (HIGH RAW PRIORITY) 
   - **Why Important**: Modern Canon cameras (2018+)
   - **Method**: Canon's newer container format
   - **ExifTool reference**: `lib/Image/ExifTool/Canon.pm`

#### 3. **Nikon NEF** (HIGH RAW PRIORITY)
   - **Implementation**: Should use existing `extract_tiff_dimensions()` utility
   - **ExifTool reference**: `lib/Image/ExifTool/Nikon.pm`

#### 4. **Fuji RAF, Olympus ORF, Panasonic RW2** (MEDIUM RAW PRIORITY)
   - **Why Important**: Popular mirrorless manufacturers
   - **Implementation**: Use existing TIFF-based utility with manufacturer-specific enhancements

### **Phase 4: Video Formats** (SPECIALIZED PRIORITY)

#### 1. **MP4/MOV** (HIGHEST VIDEO PRIORITY)
   - **Why First**: Most common video containers
   - **Method**: Parse container metadata, find video track dimensions
   - **ExifTool reference**: `lib/Image/ExifTool/QuickTime.pm`

#### 2. **Other Video Containers** (LOWER VIDEO PRIORITY)
   - **AVI, MKV, WebM, etc.**: Each requires container-specific parsing
   - **Complexity**: High - video containers are complex, may need ffmpeg integration

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
- **✅ ACHIEVED**: Dimensions output as numbers (not strings) matching ExifTool JSON format
- Dimensions available even with corrupted/missing EXIF data
- Proper error handling for malformed files
- **✅ ACHIEVED**: All compatibility tests pass (`make compat`)

## Engineer Handoff: Critical Knowledge for Next Implementation (July 23, 2025)

### **Completed Work Summary**

✅ **JPEG Dimensions**: Complete via SOF marker parsing in `src/formats/jpeg.rs`  
✅ **Canon CR2 Dimensions**: Complete via shared TIFF utility in `src/raw/formats/canon.rs`  
🔄 **Sony ARW Dimensions**: ~90% complete, shared utility exists, needs final integration  

### **Shared Infrastructure Created**

**Key Innovation**: `src/raw/mod.rs::utils::extract_tiff_dimensions()` (lines 233-370)
- **Purpose**: Extract ImageWidth/ImageHeight from any TIFF-based file (ARW, CR2, NEF, etc.)  
- **Features**: Handles both IFD0 and SubIFD locations, byte order aware, error resilient
- **Integration**: Called from both RAW handlers and TIFF branch for dual coverage
- **Testing**: Verified working with Canon CR2, enhanced for Sony ARW SubIFD support

### **Architecture Discoveries (CRITICAL)**

1. **ARW Processing Path**: ARW files go through `formats/mod.rs` TIFF branch (line 474), NOT RAW branch
   - **File Detection**: `file sony.arw` shows "TIFF image data" - explains routing
   - **Integration Point**: Add `extract_tiff_dimensions()` call after `parse_exif_data()` succeeds

2. **Group Assignment Strategy**:
   - **JPEG/PNG**: Create `File:ImageWidth/Height` (from file structure)
   - **TIFF/RAW**: Create `EXIF:ImageWidth/Height` (from TIFF IFD tags)
   - **ExifTool Compliance**: Match ExifTool's group assignment exactly

3. **Dual Coverage Pattern**: Many formats get processed twice (RAW + TIFF branches)
   - **Benefit**: Redundancy ensures dimensions are extracted
   - **Implementation**: Both paths should call same utility function

### **Research Revelations**

1. **TIFF Dimension Complexity**: Sony ARW often stores dimensions in SubIFD (tag 0x014A), not IFD0
   - **Solution**: Enhanced `extract_tiff_dimensions()` to check both locations
   - **Debugging**: Added comprehensive logging for IFD entry scanning

2. **Canon vs Sony**: Canon CR2 uses standard IFD0 tags, Sony ARW uses various locations
   - **Shared Utility**: One function handles both via fallback logic
   - **Performance**: Early exit when both dimensions found

3. **ExifTool Reference Pattern**: Always find the exact ExifTool source location
   - **TIFF Dimensions**: `lib/Image/ExifTool/Exif.pm:460-473` (tags 0x0100/0x0101)
   - **JPEG SOF**: `lib/Image/ExifTool.pm:7321-7336` (SOF marker parsing)

### **Next Engineer Success Criteria**

**For PNG Implementation**:
1. ✅ **File:ImageWidth** and **File:ImageHeight** extracted from IHDR chunk
2. ✅ **Values match ExifTool exactly**: `exiftool -j -G test.png | grep ImageWidth`
3. ✅ **Error handling** for malformed/truncated PNG files
4. ✅ **Integration** in `formats/mod.rs` PNG branch (create if needed)
5. ✅ **Testing** with multiple PNG files from `test-images/`

### **Code Locations to Study**

**Essential Files**:
- `src/formats/mod.rs:280-342` - JPEG dimension extraction pattern (reference implementation)
- `src/raw/mod.rs:233-370` - Shared TIFF dimension utility (reusable pattern)
- `src/formats/jpeg.rs:parse_sof_data()` - Binary parsing example
- `third-party/exiftool/lib/Image/ExifTool/PNG.pm` - ExifTool PNG implementation

**Integration Points**:
- `src/formats/mod.rs:227` - Format dispatch logic (add PNG case)
- `src/file_detection.rs` - File type detection (PNG already supported)

### **Testing Strategy**

**Development Testing**:
```bash
# Test specific file
cargo run -- test-images/png/sample.png | grep -E "(ImageWidth|ImageHeight)"

# Compare with ExifTool
exiftool -j -G test-images/png/sample.png | grep -E "(ImageWidth|ImageHeight)"
```

**Validation Requirements**:
- Values must match ExifTool exactly (numeric values may be strings in our output)
- Must handle corrupted/truncated files gracefully
- Must create File: group tags (not EXIF: group)

### **Known Issues & Workarounds**

1. **Byte Order Confusion**: Always check if format uses big-endian or little-endian
   - **PNG**: Always big-endian (network byte order)
   - **TIFF**: Read header to determine (II=little, MM=big)

2. **Group Assignment**: Different formats assign to different groups
   - **File Structure**: Use File: group (JPEG SOF, PNG IHDR)
   - **Metadata Tags**: Use EXIF: group (TIFF IFD tags)

3. **Binary Parsing**: Use exact ExifTool unpack patterns
   - **PNG IHDR**: `unpack('N2')` = two 32-bit big-endian values
   - **Validation**: Always validate offsets and lengths before reading

### **Future Refactoring Considerations**

1. **Binary Parsing Utilities**: Create shared functions for common patterns
   - `read_u32_be()`, `read_u32_le()` helpers
   - TIFF IFD entry parsing utilities
   - Chunk/segment scanning patterns

2. **Error Handling**: Standardize dimension extraction error types
   - **Graceful Degradation**: Missing dimensions shouldn't fail entire processing
   - **Debug Logging**: Comprehensive logging for troubleshooting

3. **Testing Infrastructure**: Create dimension extraction test suite
   - **Format Coverage**: Test files for each major format
   - **Edge Cases**: Corrupted, minimal, and unusual files
   - **ExifTool Comparison**: Automated comparison testing

### **Immediate Next Steps (for PNG)**

1. **Research**: Study `third-party/exiftool/lib/Image/ExifTool/PNG.pm` IHDR parsing
2. **Implementation**: Add PNG case to `formats/mod.rs` format dispatch
3. **Binary Parsing**: Create PNG IHDR chunk parser (8-byte signature + 13-byte IHDR)
4. **Testing**: Verify against ExifTool with multiple PNG files
5. **Documentation**: Update this document with PNG completion status

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