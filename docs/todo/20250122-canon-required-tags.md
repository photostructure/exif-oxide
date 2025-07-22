# Technical Project Plan: Canon Required Tags Implementation

## Project Overview

- **Goal**: Implement support for all Canon-specific required tags from PhotoStructure's tag-metadata.json
- **Problem**: Currently supporting 54/232 Canon MakerNotes tags (23%), need to prioritize required tags for PhotoStructure compatibility

## Background & Context

- PhotoStructure has identified 124 "required" tags across all manufacturers
- 10 tags are explicitly Canon-specific, plus ~19 MakerNotes tags that Canon populates
- Current Canon implementation extracts 54 tags but missing critical required ones

## Technical Foundation

- **Key files**:
  - `src/implementations/canon/mod.rs` - Main Canon processor
  - `src/implementations/canon/binary_data.rs` - Binary data extractors
  - `src/generated/Canon_pm/` - Generated lookup tables
  - `third-party/exiftool/lib/Image/ExifTool/Canon.pm` - ExifTool source
- **Documentation**: 
  - `docs/todo/MILESTONE-17d-Canon-RAW.md` - Current Canon status
  - `third-party/exiftool/doc/modules/Canon.md` - Canon module overview

## Work Completed

- ✅ SHORT array extraction fixed
- ✅ Binary data processors integrated (CameraSettings, FocalLength, ShotInfo, AFInfo2)
- ✅ Namespace assignment fixed (Canon tags now in "MakerNotes:" group)
- ✅ Basic PrintConv infrastructure working
- ✅ 54 Canon MakerNotes tags extracting

## Remaining Tasks

### High Priority - Canon-Specific Required Tags (10 tags)

1. **FileNumber** (0x0008)
   - Already in Canon::Main table
   - Add to `apply_canon_main_table_print_conv()`
   - No special processing needed

2. **InternalSerialNumber** (0x0096)
   - Part of Canon::Main table
   - May need decoding via `ProcessSerialData()`
   - Check Canon.pm line ~2700 for format

3. **LensType** (from LensInfo binary data)
   - Complex lens ID calculation
   - Already have `canonLensTypes` lookup table generated
   - Need to integrate with teleconverter detection

4. **CameraID** (various locations)
   - Model-specific extraction
   - Check CameraInfo tables for each model

5. **AFPointAreaExpansion**, **PreviewButton**, **AssignFuncButton**, **MenuButtonReturn**
   - From CustomFunctions binary data sections
   - Need model-specific CustomFunctions processors

6. **ZoneMatching**
   - Part of processing/color data
   - Check ColorData sections

### Medium Priority - Universal MakerNotes Tags Canon Provides

These tags from MakerNotes group that Canon cameras populate:

**High Frequency Tags (>50%):**
- **ExposureTime** (freq 0.990) - In ShotInfo/CameraSettings
- **FNumber** (freq 0.970) - In CameraSettings/ShotInfo
- **FocalLength** (freq 0.950) - Already extracting
- **ISO** (freq 0.890) - BaseISO, AutoISO already extracting
- **ShutterSpeed** (freq 0.860) - Need APEX conversion
- **Aperture** (freq 0.850) - Already extracting
- **ApertureValue** (freq 0.390) - APEX value
- **ShutterSpeedValue** (freq 0.380) - APEX value

**Lens Information:**
- **LensID** (freq 0.200) - Complex calculation with teleconverter
- **LensType** (freq 0.180) - From binary data
- **Lens** (freq 0.150) - Full lens description
- **LensModel** (freq 0.100) - Lens model name
- **LensInfo** (freq 0.086) - Min/max focal length and aperture
- **LensSpec** (freq 0.039) - Formatted specification
- **LensMake** (freq 0.022) - Usually "Canon"

**Other Required Tags:**
- **SerialNumber** (freq 0.130) - Camera body serial
- **InternalSerialNumber** (freq 0.150) - Internal ID
- **FileNumber** (freq 0.130) - Image counter
- **CameraID** (freq 0.068) - Camera-specific ID
- **Categories** (freq 0.051) - Tag categories
- **Title** (freq 0.021) - Image title
- **City** (freq 0.010) - Location city
- **Country** (freq 0.010) - Location country
- **DateTimeUTC** (freq 0.007) - UTC timestamp

### Low Priority - Standard Tags Canon Writes

Canon cameras write these standard EXIF tags:
- **Copyright** (0x8298)
- **ImageDescription** (0x010E)
- **XPKeywords**, **XPSubject**, **XPTitle** (Windows XP tags)

## Prerequisites

- Fix PrintConv type mismatches (I16 → u8 conversions)
- Verify generated Canon lookup tables are correct (not Olympus)
- Model detection for camera-specific processing

## Testing Strategy

- Use `Canon_T3i.CR2` test image
- Compare with ExifTool output: `cargo run --bin compare-with-exiftool`
- Verify required tags present in output
- Check PrintConv human-readable values

## Success Criteria

- All 10 Canon-specific required tags extracting correctly
- PrintConv producing human-readable values
- Namespace correctly set to "MakerNotes:"
- Pass compatibility tests with PhotoStructure

## Gotchas & Tribal Knowledge

### Canon-Specific Quirks
- **Absolute Offsets**: Canon uses absolute file offsets, not relative to MakerNote start
- **Binary Data Format**: Most tags in binary data sections (CameraSettings, ShotInfo, etc.)
- **SHORT Arrays**: Canon stores many values as arrays of 16-bit integers
- **Model Dependencies**: Processing varies significantly between camera generations

### Lens Identification
- **LensType Calculation**: Complex formula involving focal length and teleconverter flags
- **Teleconverter Detection**: Check bit flags in lens data to detect 1.4x/2x converters
- **Third-Party Lenses**: May report incorrect or generic lens types
- **RF vs EF Lenses**: Different ID ranges for mirrorless RF mount

### Value Extraction Issues
- **Type Mismatches**: Binary extractors return I16 but PrintConv lookups expect u8
- **APEX Conversions**: ShutterSpeed = 2^(-ShutterSpeedValue), Aperture = 2^(ApertureValue/2)
- **Multiple Locations**: Same tag may appear in CameraSettings, ShotInfo, and Main table
- **Precedence**: Use first found value, typically Main > CameraSettings > ShotInfo

### Special Processing
- **SerialNumber**: May need decoding via ProcessSerialData()
- **ISO**: Check multiple locations (BaseISO, AutoISO, ISO tags)
- **CustomFunctions**: Format varies by camera model group
- **ColorData**: Different versions for different camera generations