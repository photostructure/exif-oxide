# Technical Project Plan: HEIC/HEIF Format Support Research

**Priority**: P14 - High-impact for PhotoStructure  
**Status**: Research Complete ✅  
**Estimated Time**: 1-2 weeks research (COMPLETED), 2-3 weeks implementation  
**Research Completed**: July 2025

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

### ExifTool Implementation ✅ RESEARCHED

- **`third-party/exiftool/lib/Image/ExifTool/QuickTime.pm`** - Handles HEIC as QuickTime variant
  - **Key Functions**: `HandleItemInfo()` (line 9226+), `ParseItemLocation()` (line 9014)
  - **Detection Logic**: ftyp brands (lines 119-126), handler types (lines 8319-8323)
  - **Item Processing**: iloc parsing, mdat/idat data extraction, multi-image support
- **`third-party/exiftool/lib/Image/ExifTool/HEIF.pm`** - ❌ **DOES NOT EXIST**
  - **Research Finding**: No separate HEIF module - all processing through QuickTime.pm
  - **Architecture**: HEIF uses same ISOBMFF foundation as QuickTime/MP4
- **Image vs Video Detection**: ✅ **FULLY DOCUMENTED**
  - **Three-tier system**: ftyp brand → handler type → structural analysis
  - **Live Photo detection**: Apple metadata keys (lines 6650-6716)
  - **Motion Photo support**: Samsung mpvd atoms (lines 950-956)

### Key Standards

- ISO/IEC 23008-12:2017 (HEIF specification)
- ISO/IEC 14496-12 (ISO Base Media File Format)
- ITU-T H.265 (HEVC codec)

## Research Tasks

### Task 1: Format Structure Analysis (High Priority) ✅ COMPLETED

**Research Questions**:

1. **How does ExifTool parse HEIC/HEIF files?**

   - **Answer**: ExifTool processes HEIC/HEIF files through `QuickTime.pm` as QuickTime-variant containers using the same atom/box parsing infrastructure. Files are detected via `ftyp` atom brands (`heic`, `hevc`, `mif1`, `msf1`, `heix`) and routed through the QuickTime processing pipeline with HEIC-specific handling in the `HandleItemInfo()` function (QuickTime.pm:9226+).

2. **What's the box/atom structure?**

   - **Answer**: HEIC uses ISO Base Media File Format (ISOBMFF) structure identical to QuickTime/MP4:
     ```
     ftyp (heic/heif/mif1/msf1) - File type identification
     ├── meta (metadata container)
     │   ├── hdlr (handler = 'pict' for images)
     │   ├── pitm (primary item reference)
     │   ├── iinf (item information)
     │   ├── iloc (item location - offsets to data)
     │   ├── iprp (item properties)
     │   └── iref (item references)
     ├── mdat/idat (media/item data - actual image bytes)
     └── uuid (XMP/other metadata)
     ```

3. **How are images stored vs video sequences?**

   - **Answer**: Images use **item-based storage** (items referenced by ID in `iloc` box) while video sequences use **track-based storage** (traditional QuickTime media tracks). HEIC images are stored as items with image data in `mdat` or `idat` containers, while video uses track atoms with media data in `mdat`.

4. **Where is EXIF/XMP metadata located?**
   - **Answer**:
     - **EXIF**: Stored in item-based containers, processed by `HandleItemInfo()` with TIFF processing
     - **XMP**: Stored in UUID boxes (`\xbe\x7a\xcf\xcb...` UUID) or UserData `XMP_` atoms
     - **Apple metadata**: Stored in `mdta` handler tags with dynamic key definitions

**Deliverables**:

- ✅ Document box structure hierarchy (see above)
- ✅ Identify metadata location patterns (EXIF=items, XMP=UUID, Apple=mdta)
- ✅ Map to QuickTime atom parsing infrastructure (same parsing, HEIC-specific item handling)

### Task 2: Image vs Video Detection (Critical) ✅ COMPLETED

**Research Questions**:

1. **How to detect if HEIC contains single image vs video sequence?**

   - **Answer**: ExifTool uses a **two-tier detection system**:
     - **Primary**: `ftyp` atom major brand (`heic` = single image, `hevc` = image sequence)
     - **Secondary**: Handler types in `hdlr` atoms (`pict` = picture content, `vide` = video content)
     - **Tertiary**: Duration analysis (zero duration = still image, non-zero = sequence/motion)

2. **What boxes/atoms indicate content type?**

   - **Answer**: Key indicators (QuickTime.pm:119-123, 8319-8322):

     ```perl
     # ftyp brands
     'heic' => 'High Efficiency Image Format HEVC still image (.HEIC)'
     'hevc' => 'High Efficiency Image Format HEVC sequence (.HEICS)'

     # hdlr handler types
     pict => 'Picture'      # HEIC images
     vide => 'Video Track'  # Video content
     ```

3. **How does ExifTool make this distinction?**
   - **Answer**: Multi-layered detection process:
     1. **Read `ftyp` major brand** for initial classification
     2. **Check `hdlr` handler types** for content confirmation
     3. **Analyze duration/timing** for motion detection
     4. **Special case detection** for Live Photos (motion photo metadata)
     5. **Track structure analysis** for mixed content scenarios

**Investigation Results**:

- ✅ Detection logic found in QuickTime.pm:119-126 (MIME mapping), 8319-8322 (handler types)
- ✅ Live Photo detection via Apple metadata (lines 6650-6716)
- ✅ Samsung Motion Photos handled via `mpvd` atoms (lines 950-956)

### Task 3: Binary Extraction Patterns (Critical for PhotoStructure) ✅ COMPLETED

**Research Questions**:

1. **Where are image payloads stored?**

   - **Answer**: Image data stored in **mdat** (media data) or **idat** (item data) containers, with locations specified by **iloc** (item location) box. The `ConstructionMethod` field determines storage type: 0=file offset, 1=idat container, 2=item reference.

2. **How to extract primary image?**

   - **Answer**: Primary image extraction process (QuickTime.pm:9226+ `HandleItemInfo()`):
     1. **Primary Item ID** from `pitm` atom identifies main image
     2. **Item Location** from `iloc` provides offset/length data with extents array
     3. **Binary Assembly** combines multiple extents into single buffer
     4. **Format Detection** recognizes EXIF, XMP, JPEG, and raw image data types

3. **How to extract video poster frame?**

   - **Answer**: Video frame extraction uses **track-based processing**:
     - **Handler Context**: `$$et{HandlerType}` determines processing path
     - **Sample Description**: Video codec info in `stsd` atoms
     - **Thumbnail Extraction**: Preview images identified by dimensions (lines 9371-9379)
     - **Track Selection**: `vide` handler indicates video tracks vs `pict` for images

4. **What about burst sequences?**
   - **Answer**: Burst sequence handling:
     - **Multi-Image Processing**: `iref` box defines relationships between images
     - **Item Association**: Properties linked to multiple items via `ipma` box
     - **Document Numbers**: Sub-documents created for non-primary items (lines 9399-9405)
     - **Sequential Processing**: Items processed in sorted order for consistency

**Key Boxes Implementation**:

- ✅ **`iloc`** - Parsed by `ParseItemLocation()` (line 9014) with extent processing
- ✅ **`mdat/idat`** - Binary data extraction with fragmentation support
- ✅ **`iref`** - Item relationships for derived/related images (line 2836)
- ✅ **`hdlr`** - Handler type routing (`pict` vs `vide`) (lines 8320-8323)

### Task 4: Metadata Extraction ✅ COMPLETED

**Research Questions**:

1. **Where is EXIF data stored? (Exif box?)**

   - **Answer**: EXIF data is stored in **item-based containers** (not traditional Exif boxes). Processing occurs in `HandleItemInfo()` (QuickTime.pm:9345-9367) with TIFF processing after header validation. EXIF items are identified by content type mapping and processed with `Image::ExifTool::ProcessTIFF`.

2. **Where is XMP data? (uuid box?)**

   - **Answer**: XMP stored in **UUID boxes** with specific UUID `\xbe\x7a\xcf\xcb\x97\xa9\x42\xe8\x9c\x71\x99\x94\x91\xe3\xaf\xac` (QuickTime.pm:681-690). Alternative storage in UserData `XMP_` atoms. XMP can also be in `application/rdf+xml` MIME-type items within the item structure.

3. **Apple-specific metadata location?**

   - **Answer**: Apple metadata stored in **`mdta` handler** (Metadata Tags) with dynamic key definitions (QuickTime.pm:9659-9686). Key Apple tags include:
     ```
     com.apple.quicktime.live-photo-info → LivePhotoInfo
     com.apple.quicktime.video-orientation → VideoOrientation
     com.apple.quicktime.scene-illuminance → SceneIlluminance
     com.apple.proapps.exif.{Exif}.* → ProRes RAW EXIF mappings
     ```

4. **GPS and timestamp handling?**
   - **Answer**: Multiple GPS/timestamp storage locations:
     - **ISO 6709 GPS**: `\xa9xyz` atom with `ConvertISO6709` processing (lines 1616-1623)
     - **3GPP Location**: Complex structured GPS in `loci` atoms (lines 1733-1793)
     - **Timestamps**: `\xa9day` for ContentCreateDate with ISO 8601 conversion (lines 1566-1570)
     - **Apple Timestamps**: Via `mdta` handler for creation time
     - **GPS Track Data**: Extracted with `ExtractEmbedded` option from timed metadata

## Implementation Strategy ✅ RESEARCH-BASED PLAN

**Key Research Finding**: ExifTool processes HEIC/HEIF entirely through `QuickTime.pm` - no separate HEIF module exists. HEIC uses item-based architecture within ISOBMFF containers.

### Phase 1: Detection & Routing ✅ PLANNED

- **Add HEIC/HEIF detection** to `src/formats/mod.rs` using ftyp atom major brands:
  - `heic` → Single still image
  - `hevc` → Image sequence/video
  - `mif1`/`msf1` → HEIF variants
  - `heix` → Canon variant
- **Route through QuickTime processor** (confirmed: same ISOBMFF foundation as MP4)
- **Implement three-tier detection**:
  1. ftyp brand analysis (primary)
  2. hdlr handler type verification (`pict` vs `vide`)
  3. Structural analysis (duration, Apple metadata)

### Phase 2: Metadata Extraction ✅ PLANNED

- **Leverage QuickTime atom parsing** infrastructure if available
- **Implement item-based EXIF extraction** using `HandleItemInfo()` equivalent:
  - Parse iloc (item location) boxes for data offsets
  - Extract EXIF from item containers (not traditional Exif boxes)
  - Handle multiple EXIF header formats
- **Add XMP extraction** from UUID boxes (`be7a-cfcb-97a9-42e8...` UUID)
- **Implement Apple mdta handler** for Live Photo and Apple-specific metadata

### Phase 3: Binary Extraction ✅ PLANNED

- **Implement iloc box parsing** (`ParseItemLocation()` equivalent) for PhotoStructure:
  - Handle three construction methods (file offset, idat container, item reference)
  - Support fragmented data across multiple extents
  - Calculate base offsets considering construction method
- **Extract primary image data** from mdat/idat containers using pitm (primary item) reference
- **Handle multi-image files**:
  - Burst sequences (multiple items with relationships)
  - Live Photos (image + video components)
  - Document numbering for multiple images

## Prerequisites ✅ RESEARCH-INFORMED

1. **P15-MILESTONE-18-Video-Format-Support.md** - QuickTime atom/box parsing infrastructure needed
   - **Research Finding**: Can reuse QuickTime parsing with HEIC-specific item handling
   - **Required**: ftyp, meta, pitm, iinf, iloc, iprp, iref atom parsers
2. **P16-MILESTONE-19-Binary-Data-Extraction.md** - Binary extraction framework for PhotoStructure
   - **Research Finding**: Critical for iloc→mdat/idat binary location resolution
   - **Required**: Extent-based data assembly, fragmentation handling
3. **Test images**: Need diverse HEIC samples (CONFIRMED CRITICAL):
   - Single image HEIC (ftyp: heic, hdlr: pict)
   - Live Photo HEIC (Apple metadata present)
   - Burst sequence HEIC (multiple items, ftyp: hevc)
   - Portrait mode HEIC (with depth map items)
   - Samsung Motion Photo HEIC (mpvd atoms)
   - Canon HEIC variants (heix brand)

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

## Success Criteria ✅ RESEARCH-VALIDATED

1. **Accurate Detection**: Correctly identify HEIC/HEIF files using three-tier system
   - ✅ **ftyp brand detection** (heic/hevc/mif1/msf1/heix)
   - ✅ **Handler type validation** (pict vs vide)
   - ✅ **Content type classification** (still/sequence/Live Photo/video)
2. **Metadata Parity**: Extract same metadata as ExifTool
   - ✅ **EXIF from items** (not traditional Exif boxes)
   - ✅ **XMP from UUID boxes** (specific UUID identifier)
   - ✅ **Apple metadata** (mdta handler with dynamic keys)
3. **Binary Extraction**: Successfully extract primary image for PhotoStructure
   - ✅ **Primary item identification** (pitm box)
   - ✅ **iloc-based data location** (extent assembly)
   - ✅ **mdat/idat container handling** (construction method awareness)
4. **Dual Nature Handling**: Correctly handle image vs video content
   - ✅ **Live Photo detection** (Apple metadata + dual content)
   - ✅ **Motion Photo support** (Samsung mpvd atoms)
   - ✅ **Video sequence handling** (vide handler type)
5. **Performance**: Within 10x of ExifTool performance
   - ✅ **Fast scan mode** (skip mdat/idat for metadata-only)
   - ✅ **Selective extraction** (process only requested items)
   - ✅ **Memory efficiency** (streaming reads for large data)

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

## Research Deliverables ✅ COMPLETED

**Comprehensive Documentation Created**:

1. **`third-party/exiftool/doc/concepts/HEIC_HEIF_PROCESSING.md`** - Complete format processing guide
2. **`third-party/exiftool/doc/concepts/HEIC_BINARY_EXTRACTION.md`** - Binary data extraction specifics
3. **`third-party/exiftool/doc/concepts/HEIC_METADATA_EXTRACTION.md`** - Metadata location and extraction patterns
4. **`third-party/exiftool/doc/concepts/HEIC_IMAGE_VIDEO_DETECTION.md`** - Detection algorithms and edge cases

**All Research Questions Answered In-Line** ✅

## Next Steps - Implementation Phase

**Immediate Next Actions**:

1. **Create detailed implementation plan** based on research findings
2. **Begin QuickTime infrastructure assessment** - audit existing atom parsing capabilities
3. **Implement ftyp-based HEIC detection** in `src/formats/mod.rs`
4. **Prototype item-based metadata extraction** using research-identified patterns
5. **Develop iloc parser** for PhotoStructure binary extraction requirements
6. **Create test harness** with diverse HEIC samples for validation

**Implementation Priority Order** (based on PhotoStructure needs):

1. **Detection & Routing** (enables format recognition)
2. **Basic Metadata Extraction** (EXIF/XMP parity with ExifTool)
3. **Binary Extraction** (primary image extraction for PhotoStructure)
4. **Advanced Features** (Live Photos, burst sequences, multi-image)
