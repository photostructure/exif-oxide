# Technical Project Plan: HEIC/HEIF Format Support Research

**Priority**: P14 - High-impact for PhotoStructure  
**Status**: Research Needed  
**Estimated Time**: 1-2 weeks research, 2-3 weeks implementation

## Project Overview

**Goal**: Research and implement HEIC/HEIF support including metadata extraction and binary data extraction for embedded images.

**Problem Statement**: HEIC/HEIF is increasingly common (default iPhone format since iOS 11) but has complex dual nature - containers can hold both single images and video sequences. PhotoStructure needs both metadata extraction and binary image extraction from these files.

## Background & Context

### Why This Work is Needed

- **Market Adoption**: Default format for iPhone photos since 2017
- **Storage Efficiency**: ~50% smaller than JPEG at same quality
- **Binary Extraction Critical**: HEIC files often contain multiple images (bursts, live photos)
- **Dual Nature Complexity**: Same file extension for both images and video sequences

### HEIC/HEIF Format Overview

- **Container**: Based on ISO Base Media File Format (ISO/IEC 14496-12)
- **Compression**: Uses HEVC (H.265) video codec for images
- **Structure**: Similar to MP4/QuickTime atom/box structure
- **Metadata**: Supports EXIF, XMP, and proprietary Apple metadata

## Technical Foundation

### ExifTool Implementation
- `third-party/exiftool/lib/Image/ExifTool/QuickTime.pm` - Handles HEIC as QuickTime variant
- `third-party/exiftool/lib/Image/ExifTool/HEIF.pm` - HEIF-specific handling (if exists)
- Examine how ExifTool detects image vs video sequences

### Key Standards
- ISO/IEC 23008-12:2017 (HEIF specification)
- ISO/IEC 14496-12 (ISO Base Media File Format)
- ITU-T H.265 (HEVC codec)

## Research Tasks

### Task 1: Format Structure Analysis (High Priority)

**Research Questions**:
1. How does ExifTool parse HEIC/HEIF files?
2. What's the box/atom structure?
3. How are images stored vs video sequences?
4. Where is EXIF/XMP metadata located?

**Deliverables**:
- Document box structure hierarchy
- Identify metadata location patterns
- Map to QuickTime atom parsing infrastructure

### Task 2: Image vs Video Detection (Critical)

**Research Questions**:
1. How to detect if HEIC contains single image vs video sequence?
2. What boxes/atoms indicate content type?
3. How does ExifTool make this distinction?

**Investigation**:
```bash
# Compare single image vs live photo HEIC
exiftool -v3 single_image.heic > single.txt
exiftool -v3 live_photo.heic > live.txt
diff single.txt live.txt
```

### Task 3: Binary Extraction Patterns (Critical for PhotoStructure)

**Research Questions**:
1. Where are image payloads stored?
2. How to extract primary image?
3. How to extract video poster frame?
4. What about burst sequences?

**Key Boxes to Investigate**:
- `mdat` - Media data (actual image bytes)
- `iloc` - Item location (offsets to images)
- `iref` - Item reference (relationships)
- `hdlr` - Handler type (pict vs vide)

### Task 4: Metadata Extraction

**Research Questions**:
1. Where is EXIF data stored? (Exif box?)
2. Where is XMP data? (uuid box?)
3. Apple-specific metadata location?
4. GPS and timestamp handling?

## Implementation Strategy

### Phase 1: Detection & Routing
- Add HEIC/HEIF detection to `src/formats/mod.rs`
- Route through QuickTime processor (since it's QuickTime-based)
- Implement image vs video detection

### Phase 2: Metadata Extraction
- Leverage existing QuickTime atom parsing
- Add HEIF-specific box handlers
- Extract EXIF/XMP from appropriate boxes

### Phase 3: Binary Extraction
- Implement `iloc` box parsing for image locations
- Extract primary image data from `mdat`
- Handle multiple images (burst, live photo frames)

## Prerequisites

1. **P15-MILESTONE-18-Video-Format-Support.md** - QuickTime infrastructure needed
2. **P16-MILESTONE-19-Binary-Data-Extraction.md** - Binary extraction framework
3. **Test images**: Need diverse HEIC samples:
   - Single image HEIC
   - Live Photo HEIC
   - Burst sequence HEIC
   - Portrait mode HEIC (with depth map)

## Testing Strategy

### Test Samples Needed
```
test-images/heic/
├── iphone_single_image.heic      # Basic single image
├── iphone_live_photo.heic        # Contains video
├── iphone_burst.heic              # Multiple images
├── iphone_portrait.heic           # With depth data
└── ipad_pro_raw.heic             # ProRAW format
```

### Validation
1. Compare metadata extraction with ExifTool
2. Verify binary extraction produces valid JPEG
3. Test image vs video detection accuracy
4. Performance comparison with ExifTool

## Success Criteria

1. **Accurate Detection**: Correctly identify HEIC/HEIF files
2. **Metadata Parity**: Extract same metadata as ExifTool
3. **Binary Extraction**: Successfully extract primary image
4. **Dual Nature Handling**: Correctly handle both image and video types
5. **Performance**: Within 10x of ExifTool performance

## Gotchas & Tribal Knowledge

### Apple Variations
- **Live Photos**: Contain both image and 3-second video
- **ProRAW**: DNG data wrapped in HEIC container
- **Depth Data**: Portrait mode includes depth map
- **Burst Photos**: Multiple full-res images in single file

### Technical Challenges
- **Fragmented mdat**: Image data may be split across multiple mdat boxes
- **Compression**: HEVC decoder not needed - extract compressed data
- **Tile-based**: Large images may be tiled
- **Edit Lists**: May contain non-destructive edits

### Implementation Notes
- Can likely reuse much QuickTime atom parsing code
- Focus on metadata and offset extraction, not HEVC decoding
- Binary extraction returns compressed HEIC tiles, not decoded pixels

## References

1. [ExifTool HEIC Documentation](https://exiftool.org/TagNames/QuickTime.html#HEIC)
2. [Apple HEIF Documentation](https://developer.apple.com/documentation/imageio/core_image_data_formats/heif)
3. [Nokia HEIF Reader/Writer](https://github.com/nokiatech/heif)
4. ISO/IEC 23008-12:2017 standard (paywalled)

## Next Steps

After research phase:
1. Create implementation plan based on findings
2. Prototype using existing QuickTime infrastructure
3. Add HEIF-specific box handlers as needed
4. Implement binary extraction for PhotoStructure use case