# Technical Project Plan: Composite Required Tags Implementation

## Project Overview

- **Goal**: Implement calculation of 29 composite tags marked as required in tag-metadata.json
- **Problem**: Composite tags require combining data from multiple sources and applying calculations

## MANDATORY READING

These are relevant, mandatory, prerequisite reading for every task:

- [@CLAUDE.md](CLAUDE.md)
- [@docs/TRUST-EXIFTOOL.md](docs/TRUST-EXIFTOOL.md).

## DO NOT BLINDLY FOLLOW THIS PLAN

Building the wrong thing (because you made an assumption or misunderstood something) is **much** more expensive than asking for guidance or clarity.

The authors tried their best, but also assume there will be aspects of this plan that may be odd, confusing, or unintuitive to you. Communication is hard!

**FIRSTLY**, follow and study **all** referenced source and documentation. Ultrathink, analyze, and critique the given overall TPP and the current task breakdown.

If anything doesn't make sense, or if there are alternatives that may be more optimal, ask clarifying questions. We all want to drive to the best solution and are delighted to help clarify issues and discuss alternatives. DON'T BE SHY!

## KEEP THIS UPDATED

This TPP is a living document. **MAKE UPDATES AS YOU WORK**. Be concise. Avoid lengthy prose!

**What to Update:**

- 🔍 **Discoveries**: Add findings with links to source code/docs (in relevant sections)
- 🤔 **Decisions**: Document WHY you chose approach A over B (in "Work Completed")
- ⚠️ **Surprises**: Note unexpected behavior or assumptions that were wrong (in "Gotchas")
- ✅ **Progress**: Move completed items from "Remaining Tasks" to "Work Completed"
- 🚧 **Blockers**: Add new prerequisites or dependencies you discover

**When to Update:**

- After each research session (even if you found nothing - document that!)
- When you realize the original approach won't work
- When you discover critical context not in the original TPP
- Before context switching to another task

**Keep the content tight**

- If there were code examples that are now implemented, replace the code with a link to the final source.
- If there is a lengthy discussion that resulted in failure or is now better encoded in source, summarize and link to the final source.
- Remember: the `ReadTool` doesn't love reading files longer than 500 lines, and that can cause dangerous omissions of context.

The Engineers of Tomorrow are interested in your discoveries, not just your final code!


## Background & Context

- Composite tags are calculated from other tags, not directly extracted
- Essential for user-friendly display and advanced features
- Must match ExifTool's calculation methods exactly

## Technical Foundation

- **Key files needed**:
  - `src/composite_tags.rs` - Composite tag calculations
  - `src/generated/composite_tags.rs` - Generated composite definitions
- **ExifTool reference**: Composite.pm module

## Required Composite Tags (29 total)

### Image Properties (7 tags)
- **ImageSize** - "WIDTHxHEIGHT" string format
- **ImageWidth** - Width from various sources
- **ImageHeight** - Height from various sources
- **Megapixels** - (Width × Height) / 1,000,000
- **CircleOfConfusion** - Based on sensor size
- **ImageDataHash** - Hash of image pixel data
- **AvgBitrate** - Average bitrate for video

### Camera Settings (8 tags)
- **Aperture** - F-number formatted (e.g., "5.6")
- **ShutterSpeed** - Formatted exposure time
- **ISO** - Combined from various ISO tags
- **LensID** - Lens identification
- **LensType** - Lens type from MakerNotes
- **Lens** - Full lens description
- **LensSpec** - Formatted lens specification
- **Rotation** - Effective rotation angle

### Lens Calculations (4 tags)
- **FocalLength35efl** - 35mm equivalent focal length
- **ScaleFactor35efl** - Crop factor
- **HyperfocalDistance** - Hyperfocal distance calculation
- **DOF** - Depth of field range

### GPS (2 tags)
- **GPSDateTime** - Combined GPS date and time
- **GPSPosition** - Combined lat/lon decimal

### Timestamps (5 tags)
- **DateTimeOriginal** - Original capture time
- **SubSecDateTimeOriginal** - With subseconds
- **SubSecCreateDate** - Create date with subseconds
- **SubSecModifyDate** - Modify date with subseconds
- **SubSecMediaCreateDate** - Media create with subseconds

### Other (3 tags)
- **FileNumber** - From filename pattern
- **RegionInfoMP** - Microsoft Photo regions
- **Duration** - Video duration

## Work Completed

### Infrastructure ✅
- ✅ Basic composite tag infrastructure exists
- ✅ Composite tags generation and dispatch system
- ✅ Multi-pass dependency resolution algorithm

### Phase 1: Core Essential Tags ✅ (July 25, 2025)
- ✅ **ISO** - Priority-based consolidation from multiple ISO sources
- ✅ **ImageWidth** - Width dimension with proper precedence (SubIFD3 > IFD0 > ExifIFD)
- ✅ **ImageHeight** - Height dimension with proper precedence
- ✅ **Rotation** - EXIF Orientation tag converted to degrees (0°, 90°, 180°, 270°)

### Phase 2: GPS Consolidation ✅ (July 25, 2025)
- ✅ **GPSDateTime** - Combined GPS date/time stamps to UTC format
- ✅ **GPSLatitude** - Raw GPS coordinates to signed decimal degrees
- ✅ **GPSLongitude** - Raw GPS coordinates to signed decimal degrees

### Phase 3: SubSec Timestamps ✅ (July 25, 2025)
- ✅ **SubSecCreateDate** - EXIF CreateDate with subseconds and timezone
- ✅ **SubSecModifyDate** - EXIF ModifyDate with subseconds and timezone
- ✅ **SubSecMediaCreateDate** - Media create date with subseconds

### Phase 6: Dynamic Dependency Resolution ✅ (July 28, 2025)
- ✅ **ExifTool-compatible dynamic tag resolution** - Implements ExifTool's BuildCompositeTags algorithm (lib/Image/ExifTool.pm:3977-4055)
- ✅ **Multi-pass composite-to-composite dependencies** - Handles complex dependency graphs like LightValue requiring Aperture, ShutterSpeed, ISO
- ✅ **Manual tag computation bridge** - Allows enhanced consolidation (ISO from multiple sources) while maintaining ExifTool compatibility
- ✅ **LightValue composite implementation** - Exact ExifTool CalculateLV function (lib/Image/ExifTool/Exif.pm:5319-5330)
- ✅ **Validated with real images** - LightValue: 14.96 computed correctly using dynamic ISO resolution

### Quality Assurance ✅ (July 25, 2025)
- ✅ All implementations include ExifTool source file and line number references
- ✅ Comprehensive testing with `make precommit` - all tests passing
- ✅ Full compliance with Trust ExifTool principle
- ✅ 11 critical composite tags successfully implemented and validated

## Remaining Tasks

### Phase 4: Lens System (Medium Priority)
**Status**: Ready for implementation

- **Lens** - Full lens description from manufacturer databases
- **LensID** - Lens identification from MakerNotes
- **LensSpec** - Formatted lens specification (e.g., "18-55mm f/3.5-5.6")
- **LensType** - Lens type from MakerNotes

### Phase 5: Media Tags & Advanced Features (Medium Priority)
**Status**: Ready for implementation

- **Duration** - Video duration calculation
- **ScaleFactor35efl** - Complete sensor size calculation (basic version exists, needs enhancement)

### Implementation Notes from Completed Phases

1. **ISO Consolidation** ✅ COMPLETED
   - Implemented priority order: ISO, ISOSpeed, ISOSpeedRatings[0], PhotographicSensitivity
   - Manufacturer-specific tags: Canon CameraISO, Nikon ISO2, Sony SonyISO
   - ExifTool: lib/Image/ExifTool/Canon.pm:9792-9806, lib/Image/ExifTool/Exif.pm:2116-2124

2. **Image Dimension Priority** ✅ COMPLETED
   - Implemented precedence: SubIFD3 > IFD0 > ExifIFD
   - ExifTool: lib/Image/ExifTool/Exif.pm:725-745 (ImageWidth), 746-766 (ImageHeight)

3. **GPS Coordinate Processing** ✅ COMPLETED
   - GPS coordinates converted from DMS to signed decimal degrees
   - Hemisphere handling: South/West negative
   - ExifTool: lib/Image/ExifTool/GPS.pm:353-390

4. **SubSec Timestamp Processing** ✅ COMPLETED
   - SubSec values normalized to fractional seconds
   - Timezone offset integration
   - ExifTool: lib/Image/ExifTool/Exif.pm:4930-4950

### Medium Priority - Lens Information

1. **LensID**
   - Lookup from manufacturer lens databases
   - Combine MakerNotes LensType with lens tables
   - Handle third-party lenses

2. **Lens**
   - Full descriptive name from LensID
   - Or construct from LensModel/LensMake
   - Include adapter info if present
   - **Nikon**: Requires decrypting LensData for full info
   - **Nikon**: Check teleconverter flags in lens ID bytes

3. **LensSpec**
   - Format: "18-55mm f/3.5-5.6" (zoom)
   - Format: "50mm f/1.8" (prime)
   - Extract from LensInfo tag or construct

### Advanced Calculations

1. **Depth of Field (DOF)**
   ```rust
   // Requires: FocalLength, Aperture, FocusDistance, CircleOfConfusion
   // Near = (H × D) / (H + D - f)
   // Far = (H × D) / (H - D + f)
   // H = Hyperfocal distance
   ```

2. **35mm Equivalent Calculations**
   - **FocalLength35efl**: FocalLength × ScaleFactor35efl
   - **ScaleFactor35efl**: 43.27 / SensorDiagonal
   - Need sensor size from camera database

3. **CircleOfConfusion**
   - Based on sensor diagonal
   - Default: diagonal / 1440
   - Override for specific formats

### GPS & Timestamps

1. **GPSDateTime**
   - Combine GPSDateStamp + GPSTimeStamp
   - Convert to standard format
   - Handle timezone (usually UTC)

2. **SubSec Timestamps**
   - Parse SubSecTime as fractional seconds
   - Append to main timestamp
   - Handle varying precision (1-3 digits typical)

### Special Cases

1. **Rotation**
   - From Orientation tag (1-8 → 0°, 90°, 180°, 270°)
   - Or from video rotation matrix
   - Account for camera orientation sensors

2. **FileNumber**
   - Extract from filename: DSC_(\d+), IMG_(\d+), _MG_(\d+)
   - Handle camera-specific patterns
   - Return numeric portion only

## Prerequisites

- All source tags must be available
- Understanding of ExifTool's calculation methods
- Proper PrintConv formatting

## Testing Strategy

- Compare all composite values with ExifTool output
- Test edge cases (missing data, invalid values)
- Verify formatting matches exactly
- Test with various camera models

## Success Criteria & Quality Gates

### You are NOT done until this is done:

1. **All Required Composite Tags Implemented**:
   - [ ] 29 composite tags from tag-metadata.json calculating correctly
   - [ ] Values match ExifTool output exactly (critical formatting requirements)
   - [ ] Graceful handling of missing source data

2. **Critical Composite Formatting Issues** (addresses major compatibility failures):
   ```json
   Priority composite tags with formatting problems:
   - "Composite:Aperture"      // Must show "3.9" not [39,10]
   - "Composite:ShutterSpeed"  // Must show "1/30" not raw value
   - "Composite:ImageSize"     // Must show "2048x1536" not "2048 1536"  
   - "Composite:Megapixels"    // Must show "3.1" not "3.145728"
   - "Composite:ISO"           // Must show consolidated ISO value
   ```

3. **Missing Composite Calculations** (found in compatibility failures):
   ```json
   Currently missing composite tags:
   - "Composite:SubSecCreateDate"      // EXIF CreateDate + SubSecTime
   - "Composite:SubSecDateTimeOriginal"// EXIF DateTimeOriginal + SubSecTimeOriginal  
   - "Composite:SubSecModifyDate"      // EXIF ModifyDate + SubSecTime
   - "Composite:GPSPosition"           // "lat lon" decimal format
   - "Composite:GPSDateTime"           // Combined GPS date/time
   ```

4. **Specific Tag Validation** (must be added to `config/supported_tags.json` and pass `make compat-force`):
   ```bash
   # All these composite tags must be present and working:
   - "Composite:Aperture"
   - "Composite:ShutterSpeed"  
   - "Composite:ImageSize"
   - "Composite:Megapixels"
   - "Composite:ISO"
   - "Composite:Rotation"
   - "Composite:SubSecCreateDate"
   - "Composite:SubSecDateTimeOriginal"
   - "Composite:SubSecMediaCreateDate"
   - "Composite:SubSecModifyDate"
   - "Composite:GPSDateTime"
   - "Composite:GPSPosition"
   ```

5. **Validation Commands**:
   ```bash
   # After implementing composite calculations:
   make compat-force                      # Regenerate reference files
   make compat-test | grep "Composite:"   # Check composite compatibility
   
   # Target: All composite tags showing formatted values matching ExifTool
   ```

6. **Manual Validation** (Test with Multiple File Types):
   - **JPEG with EXIF**: Verify ImageSize, Megapixels, Aperture calculations
   - **GPS-enabled images**: Confirm GPSPosition, GPSDateTime composites
   - **Various cameras**: Test ISO consolidation from multiple sources
   - **SubSec precision**: Verify timestamp composites include subseconds

### Prerequisites & Dependencies:
- **MUST WAIT for P10a completion** - Composite tags depend on EXIF source data being properly formatted
- **P14b GPS Processing** - GPS composite tags require GPS destination processing
- Source tags (EXIF, GPS, MakerNotes) must be extracting correctly

### Quality Gates Definition:
- **Compatibility Test Threshold**: <5 Composite-related failures in `make compat-test`
- **Format Consistency**: Composite:Aperture must match EXIF:FNumber formatting exactly
- **ImageSize Format**: Must use "WIDTHxHEIGHT" format (2048x1536), never space-separated

## Gotchas & Tribal Knowledge

### Formatting Rules
- **ShutterSpeed**: Values ≥ 0.3 seconds show as decimal, not fraction
- **Aperture**: NO "f/" prefix (ExifTool style)
- **ISO**: Prefer standard ISO tag, fallback to manufacturer-specific
- **ImageSize**: Format as "4000x3000" not "4000 x 3000"

### Calculation Specifics
- **Megapixels**: Round to 1 decimal place (16.1 not 16.12)
- **DOF**: Returns "inf" for infinity, handle gracefully
- **CircleOfConfusion**: Camera-specific overrides exist
- **FocalLength35efl**: May already be provided by some cameras

### Precedence Rules
- **ImageWidth/Height**: SubIFD3 > IFD0 > ExifIFD
- **DateTimeOriginal**: EXIF > XMP > QuickTime
- **GPS**: Prefer decimal over degrees/minutes/seconds
- **Lens Info**: LensID > LensModel > constructed from LensType

### Special Cases
- **LensSpec**: Prime lenses show single aperture value
- **GPS Sign**: South latitude and West longitude are negative
- **SubSec**: Can be 1-6 digits, normalize to seconds
- **Rotation**: Video rotation matrix overrides EXIF Orientation
- **FileNumber**: Must handle leading zeros (preserve or strip?)

### Manufacturer Quirks
- Canon stores ISO in MakerNotes:CameraISO
- Nikon ISO locations: ISO, ISOSpeed, ISOInfo (all may be encrypted)
- Nikon LensID: 8-byte composite requiring pattern matching
- Nikon lens data often encrypted in LensData (0x0098)
- Sony uses different tag names for similar data
- Olympus LensType needs lookup table (olympusLensTypes)