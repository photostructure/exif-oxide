# Technical Project Plan: Composite Required Tags Implementation

## Project Overview

- **Goal**: Implement calculation of 29 composite tags marked as required in tag-metadata.json
- **Problem**: Composite tags require combining data from multiple sources and applying calculations

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
- **ImageSize** - "WIDTHxHEIGHT" string format - freq 1.000
- **ImageWidth** - Width from various sources - freq 1.000
- **ImageHeight** - Height from various sources - freq 1.000
- **Megapixels** - (Width × Height) / 1,000,000 - freq 1.000
- **CircleOfConfusion** - Based on sensor size - freq 0.030
- **ImageDataHash** - Hash of image pixel data
- **AvgBitrate** - Average bitrate for video - freq 0.0015

### Camera Settings (8 tags)
- **Aperture** - F-number formatted (e.g., "5.6") - freq 0.850
- **ShutterSpeed** - Formatted exposure time - freq 0.860
- **ISO** - Combined from various ISO tags - freq 0.890
- **LensID** - Lens identification - freq 0.200
- **LensType** - Lens type from MakerNotes - freq 0.180
- **Lens** - Full lens description - freq 0.150
- **LensSpec** - Formatted lens specification - freq 0.039
- **Rotation** - Effective rotation angle - freq 0.059

### Lens Calculations (4 tags)
- **FocalLength35efl** - 35mm equivalent focal length - freq 0.040
- **ScaleFactor35efl** - Crop factor - freq 0.020
- **HyperfocalDistance** - Hyperfocal distance calculation - freq 0.021
- **DOF** - Depth of field range - freq 0.140

### GPS (2 tags)
- **GPSDateTime** - Combined GPS date and time - freq 0.027
- **GPSPosition** - Combined lat/lon decimal - freq 0.034

### Timestamps (5 tags)
- **DateTimeOriginal** - Original capture time - freq 0.970
- **SubSecDateTimeOriginal** - With subseconds - freq 0.093
- **SubSecCreateDate** - Create date with subseconds - freq 0.090
- **SubSecModifyDate** - Modify date with subseconds - freq 0.090
- **SubSecMediaCreateDate** - Media create with subseconds - freq 0.000

### Other (3 tags)
- **FileNumber** - From filename pattern - freq 0.130
- **RegionInfoMP** - Microsoft Photo regions - freq 0.000
- **Duration** - Video duration - freq 0.002

## Work Completed

- ✅ Basic composite tag infrastructure exists
- ✅ Some composite tags already defined
- ⚠️ Need to verify calculations match ExifTool

## Remaining Tasks

### High Priority - Core Calculations

1. **ISO Consolidation** (freq 0.890)
   ```rust
   // Priority order: ISO, ISOSpeed, ISOSpeedRatings[0], PhotographicSensitivity
   // Handle manufacturer-specific tags:
   //   - Canon: CameraISO
   //   - Nikon: ISO (0x0002), ISOSpeed (0x0076), ISOInfo (in ShotInfo - encrypted)
   //   - Sony: SonyISO
   // Note: Nikon ISO values often require decryption
   ```

2. **ShutterSpeed Formatting** (freq 0.860)
   ```rust
   // Convert ExposureTime to human-readable
   // 0.004 → "1/250"
   // 0.3+ → "0.3" (not fraction)
   // 2.5 → "2.5"
   ```

3. **Aperture Formatting** (freq 0.850)
   ```rust
   // NO "f/" prefix per ExifTool (just the number)
   // From FNumber or calculate from ApertureValue
   // 2.8 → "2.8"
   ```

4. **Image Dimension Priority**
   - Check SubIFD3:ImageWidth/Height first (full resolution)
   - Then IFD0:ImageWidth/Height
   - Then ExifIFD:ExifImageWidth/Height

### Medium Priority - Lens Information

1. **LensID** (freq 0.200)
   - Lookup from manufacturer lens databases
   - Combine MakerNotes LensType with lens tables
   - Handle third-party lenses

2. **Lens** (freq 0.150)
   - Full descriptive name from LensID
   - Or construct from LensModel/LensMake
   - Include adapter info if present
   - **Nikon**: Requires decrypting LensData for full info
   - **Nikon**: Check teleconverter flags in lens ID bytes

3. **LensSpec** (freq 0.039)
   - Format: "18-55mm f/3.5-5.6" (zoom)
   - Format: "50mm f/1.8" (prime)
   - Extract from LensInfo tag or construct

### Advanced Calculations

1. **Depth of Field (DOF)** (freq 0.140)
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

3. **CircleOfConfusion** (freq 0.030)
   - Based on sensor diagonal
   - Default: diagonal / 1440
   - Override for specific formats

### GPS & Timestamps

1. **GPSDateTime** (freq 0.027)
   - Combine GPSDateStamp + GPSTimeStamp
   - Convert to standard format
   - Handle timezone (usually UTC)

2. **SubSec Timestamps**
   - Parse SubSecTime as fractional seconds
   - Append to main timestamp
   - Handle varying precision (1-3 digits typical)

### Special Cases

1. **Rotation** (freq 0.059)
   - From Orientation tag (1-8 → 0°, 90°, 180°, 270°)
   - Or from video rotation matrix
   - Account for camera orientation sensors

2. **FileNumber** (freq 0.130)
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

## Success Criteria

- All 29 composite tags calculating correctly
- Values match ExifTool output exactly
- Graceful handling of missing source data
- Efficient calculation (no redundant work)
- Proper precedence when multiple sources exist

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