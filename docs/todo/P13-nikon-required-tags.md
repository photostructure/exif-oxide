# Technical Project Plan: Nikon Required Tags Implementation

## Project Overview

- **High-level goal**: Enable extraction of all "required: true" tags from Nikon JPEG and RAW (NEF/NRW) files
- **Problem statement**: Currently zero Nikon tags are extracted despite having basic infrastructure in place. Nikon's encryption system blocks access to most valuable metadata.

## Background & Context

- Nikon is ExifTool's largest module (14,191 lines) with the most sophisticated maker notes implementation
- 25-30 required tags need Nikon-specific implementation
- Unlike Canon, Nikon encrypts most maker notes data requiring serial number + shutter count keys
- Critical for PhotoStructure compatibility with Nikon cameras
- Nikon implementation presents significant complexity due to:
  1. **Extensive encryption** - Most valuable data encrypted using serial number and shutter count keys
  2. **Model-specific processing** - Over 30 different ShotInfo variants (D40, D80, D90, D3, D300, D700, D5000, D7000, D800, D850, Z6, Z7, Z9, etc.)
  3. **Complex subdirectories** - Multiple levels of nested binary data structures
  4. **Large lens database** - 618 lens IDs (55% more than Canon)

### Links to Related Design Docs

- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - Must follow ExifTool's implementation exactly
- [CODEGEN.md](../CODEGEN.md) - Leverage for model-specific tables
- [tag-metadata.json](../tag-metadata.json) - Required tags definition with frequencies

## Technical Foundation

### Key Codebases

- `third-party/exiftool/lib/Image/ExifTool/Nikon.pm` - Canonical implementation (14,191 lines)
- `src/implementations/nikon/` - Our Nikon module
- `codegen/config/Nikon_pm/` - Codegen configurations
- `src/generated/Nikon_pm/` - Generated lookup tables

### ExifTool Functions to Study

- `ProcessNikon()` - Main entry point with pre-scan logic
- `ProcessNikonEncrypted()` - Decryption implementation (lines 9084-9244)
- `LensIDConv()` - Complex lens identification system (8-parameter composite ID)
- Model-specific ShotInfo tables (30+ variants)

### Systems to Familiarize With

- Nikon Format1/2/3 detection (different offset schemes)
- Serial number and shutter count extraction for keys
- XOR-based encryption with model-specific constants
- Subdirectory and binary data processing
- Pre-scan requirement for key extraction

## Work Completed

### Infrastructure

- ✅ Basic module structure (`src/implementations/nikon/mod.rs`)
- ✅ Format detection for Nikon maker notes variants (Format1/2/3)
- ✅ Codegen extraction of 618 lens IDs (largest lens database)
- ✅ AF point lookup tables generated (105, 135, 153 point systems)
- ✅ Various lookup tables (NEF compression, metering modes, focus modes)
- ✅ Tag structure with 111 Nikon data types defined
- ✅ Module organization (encryption.rs, af.rs, lens.rs)
- ✅ `NikonEncryptionKeys` structure defined
- ✅ Offset calculation for different format versions

### Decisions Made

- Separate modules for encryption, AF, and lens handling
- Using codegen for static lookup tables
- Following ExifTool's exact tag naming
- Format-specific offset calculation matching ExifTool

### Issues Resolved

- Module structure matches ExifTool organization
- Codegen framework successfully extracts complex tables

## Required Tags Analysis

From `tag-metadata.json`, the following tags marked as `required: true` are Nikon-specific or populated by Nikon cameras:

### Core Camera Information

- **CameraID** (freq 0.068) - Unique camera identifier
- **Make** (freq 1.000) - "NIKON CORPORATION" or "NIKON"
- **Model** (freq 1.000) - Camera model name
- **InternalSerialNumber** (freq 0.150) - Different from visible serial
- **SerialNumber** (freq 0.130) - Camera body serial (tag 0x001d - CRITICAL for encryption)

### Date/Time Tags

- **DateTimeOriginal** (freq 0.970) - Original capture time
- **DateTimeUTC** (freq 0.007) - UTC timestamp

### Lens Information (Complex, Often Encrypted)

- **Lens** (freq 0.150) - Full lens description
- **LensID** (freq 0.200) - 8-byte composite ID requiring pattern matching
- **LensInfo** (freq 0.086) - Min/max focal length and aperture
- **LensModel** (freq 0.100) - From LensData (encrypted)
- **LensSpec** (freq 0.039) - Formatted lens specification
- **LensType** (freq 0.180) - From LensData (tag 0x0098, encrypted)
- **LensType2** (freq 0.000) - Z-mount specific
- **LensType3** (freq 0.000) - Additional Z-mount info
- **LensMake** (freq 0.022) - Usually "Nikon" or third-party

### Exposure Information (Often in Encrypted ShotInfo)

- **Aperture** (freq 0.850) - F-number used
- **ApertureValue** (freq 0.390) - APEX aperture value
- **ExposureTime** (freq 0.990) - In ShotInfo (encrypted)
- **FNumber** (freq 0.970) - In ShotInfo (encrypted)
- **FocalLength** (freq 0.950) - Multiple locations
- **ISO** (freq 0.890) - ISO, ISOSpeed, ISOInfo tags (may be encrypted)
- **ShutterSpeed** (freq 0.860) - Calculated from ExposureTime
- **ShutterSpeedValue** (freq 0.380) - APEX shutter speed
- **ShutterCount** (freq 0.041) - Tag 0x00a7 (CRITICAL for encryption)

### Image Properties

- **ImageHeight** (freq 1.000) - Image height in pixels
- **ImageWidth** (freq 1.000) - Image width in pixels
- **Rotation** (freq 0.059) - Auto-rotate info

### Other Required Tags

- **Country** (freq 0.010) - GPS/location country
- **City** (freq 0.010) - GPS/location city
- **FileNumber** (freq 0.130) - File number on camera
- **Rating** (freq 0.140) - Image rating (0-5)
- **Title** (freq 0.021) - Image title

## Remaining Tasks

### Phase 1: Enable Basic Unencrypted Tags (4-6 hours, High Confidence)

**Goal**: Extract unencrypted Nikon tags from main IFD
**Implementation**: Add entries to `src/generated/supported_tags.json`

1. Enable main Nikon IFD tags (0x0001-0x001c, 0x001e-0x00a6)
2. Implement basic tag extraction in `process_nikon_ifd()`
3. Add PrintConv for: CameraID, Quality, WhiteBalance, Sharpness, FocusMode
4. Test with sample Nikon JPEGs

**Expected yield**: 10-15 basic tags without encryption (Make, Model, DateTime, basic EXIF)

### Phase 2: Extract Encryption Keys (6-8 hours, High Confidence)

**Goal**: Extract serial number and shutter count for key generation
**Implementation**: Pre-scan phase in `process_maker_notes()`

1. Extract SerialNumber (tag 0x001d) - critical for key generation
2. Extract ShutterCount (tag 0x00a7) - second part of key
3. Handle alternative locations (0x00a8, 0x0247 for some models)
4. Store keys in ExifContext for later use
5. Validate key format per model (some serial numbers have special formats)

**Critical**: Keys must be extracted before processing other tags (two-pass requirement)

### Phase 3: Implement Decryption (8-12 hours, Research Required)

**Goal**: Decrypt ShotInfo sections using extracted keys
**Known approach**: Port `ProcessNikonEncrypted()` from Nikon.pm (lines 9084-9244)

- XOR decryption with model-specific constants
- Key generation: combine serial, count, and magic numbers
- Different algorithms for different camera generations

**Simplified algorithm from Nikon.pm**:

```perl
# Key generation pattern
$key = $serialNumber . $shutterCount . $modelConstant;
foreach $byte (@encryptedData) {
    $decrypted = $byte ^ $key[$index % length($key)];
}
```

**Unknown unknowns**:

- Exact byte ordering for key generation across models
- Edge cases in serial number formats
- Firmware version variations affecting encryption
- Z-series may use different key locations

**Research needed**:

1. Study lines 9084-9244 of Nikon.pm carefully
2. Map out model-specific magic constants
3. Create test harness comparing our decryption with ExifTool
4. Start with one modern model (suggest Z9 or D850)

### Phase 4: Model-Specific Processing (20-30 hours, Research Required)

**Goal**: Add handlers for major camera models
**Known requirements**: 30+ ShotInfo table variants

- Each model has unique offsets and tag definitions
- Must detect model and apply correct table
- Some models have multiple encrypted sections
- ShotInfoVersion determines which table to use ("0100", "0200", "0204", etc.)

**Implementation approach**:

```rust
match model {
    "NIKON D850" => process_shot_info_d850(data, offset),
    "NIKON Z 9" => process_shot_info_z9(data, offset),
    // ... 30+ more variants
}
```

**Consider**: Runtime table extraction for model-specific tables

- Add `runtime_table.json` configuration for Nikon
- Extract ShotInfo tables dynamically like Canon
- Would handle the 30+ model variations automatically

**Effort**: 2-3 hours per model × 10-15 priority models = 20-30 hours

### Phase 5: Binary Data Extraction (16-24 hours)

**Goal**: Extract data from decrypted sections
Major sections requiring binary data extraction:

1. **ShotInfo** (0x0091) - Camera settings, encrypted, model-specific
2. **ColorBalance** (0x0097) - White balance data, encrypted, versions 1/2/3/4/A/B/C
3. **LensData** (0x0098) - Lens information, encrypted, version-dependent
4. **AFInfo** (0x00b7) - Autofocus information
5. **VRInfo** (0x001f) - Vibration reduction data

**Effort**: 4-6 hours per section type

### Phase 6: Enable NEF/NRW Support (8-12 hours, Research Required)

**Current state**: Detection commented out in `raw_detector.rs`

- NEF format well-understood (TIFF-based like ORF, ARW)
- Standard structure: IFD0 (main), IFD1 (thumbnail), ExifIFD, MakerNote IFD
- Need to enable and test thoroughly

**Unknown**:

- NRW format differences from NEF
- Encryption differences in RAW files
- Preview extraction complexity
- NEF compression types (lossy, lossless, uncompressed)

## Prerequisites

- Complete Canon implementation as reference (mostly done) ✓
- Robust ProcessBinaryData framework (exists) ✓
- Runtime table extraction capability (needs enhancement for Nikon)
- Test images from multiple Nikon models with known metadata
- **Tag Kit Migration**: Complete [tag kit migration and retrofit](../done/20250723-tag-kit-migration-and-retrofit.md) for Nikon module
  - Currently no Nikon inline_printconv config exists, but when adding PrintConvs, use tag kit system

## Testing Strategy

### Unit Tests

- Encryption key extraction for each model
- Decryption validation against known values
- Lens ID resolution for all 618 lenses
- AF point decoding
- Model detection from ShotInfoVersion

### Integration Tests

- Compare output with ExifTool for test images
- Verify all required tags extracted
- Test with images from each major model series
- Handle missing encryption keys gracefully

### Manual Testing Steps

1. Run `cargo run --bin compare-with-exiftool [nikon.jpg]`
2. Verify encryption keys extracted correctly (`-v3` in ExifTool shows keys)
3. Check decrypted values match ExifTool
4. Validate lens identification
5. Test with both encrypted and unencrypted images

### Test Image Requirements

- Modern Z-mount: Z9, Z8, Z7II, Z6III samples
- Recent DSLR: D850, D780, D750, D500 samples
- Older models: D300, D700 (different encryption)
- Both JPEG and NEF files
- Images with known serial numbers for validation
- Third-party lens samples for edge cases

## Success Criteria & Quality Gates

### Definition of Done

- [ ] All required MakerNotes tags extracted from Nikon JPEGs
- [ ] Encryption/decryption matches ExifTool exactly
- [ ] Support for top 5 current Nikon models (Z9, Z8, Z7II, Z6III, D850)
- [ ] NEF file support enabled and tested
- [ ] Performance within 2x of ExifTool
- [ ] Zero regression in existing tests
- [ ] Clear error messages when decryption fails

### Quality Requirements

- Must handle missing encryption keys gracefully
- Error messages should guide users (e.g., "Cannot decrypt without serial number")
- No panic on malformed data
- Partial success acceptable (unencrypted tags even if decryption fails)

## Gotchas & Tribal Knowledge

### Known Edge Cases

1. **Serial Number Format**: Some models use different formats/locations
2. **Firmware Updates**: Can change encryption or tag locations
3. **Third-party Lenses**: May not match lens database exactly
4. **Missing Keys**: Some images lack serial/shutter count
5. **Coolpix Models**: Often don't use encryption

### Nikon Encryption System

- **Key Generation**: SerialNumber + ShutterCount + model-specific constants
- **XOR Algorithm**: Simple XOR but with complex key preparation
- **Model Constants**: Each camera has unique magic numbers
- **Version Detection**: Must detect encryption version before decrypting
- **Pre-scan Requirement**: MUST extract keys before processing other tags

### Lens Identification Complexity

- **8-Byte ID**: Composite of multiple parameters (not simple lookup)
- **Pattern Matching**: Same physical lens has multiple IDs
- **Teleconverters**: Modify the lens ID
- **Third-Party**: May report incorrect or generic IDs
- **Z-Mount**: Different ID scheme than F-mount

### Model-Specific Processing

- **ShotInfo Variants**: 30+ different table formats
- **Format Detection**: Must identify camera model first
- **Firmware Dependencies**: Same model may have format changes
- **Z-Series Differences**: Significant changes from DSLR era

### Technical Debt

- ExifTool's Nikon module has 25 years of accumulated fixes
- Many "magic" offsets exist for specific camera quirks
- Some decryption constants derived through reverse engineering

### Critical Insights

1. **Pre-scan is Mandatory**: Must extract keys before processing
2. **Model Detection First**: Everything depends on camera model
3. **Trust ExifTool**: If our output differs, we're wrong
4. **Start Small**: Get unencrypted tags working first
5. **Partial Success OK**: Better to extract some tags than none

### Nikon-Specific Patterns

- Tags 0x0001-0x00ff are mostly standard across models
- Tags 0x0100+ are often model-specific
- Encrypted sections typically start at known offsets
- ColorBalance format varies by camera generation
- Format1: Original DSLRs (D1, D100) - relative to maker note start
- Format2: Most DSLRs (D200-D850) - relative to TIFF header
- Format3: Recent mirrorless (Z series) - includes additional offset field

### Value Extraction Patterns

- **Encrypted First**: Most valuable data is encrypted
- **Multiple Locations**: Same tag in multiple places
- **Precedence**: Main IFD > Encrypted sections > Defaults
- **Format Variations**: Same tag may have different formats by model

### Performance Considerations

- **Decryption Overhead**: Adds 10-20% processing time
- **Pre-scan Impact**: Requires two passes through data
- **Caching Strategy**: Consider caching decrypted sections
- **Memory Usage**: Z9 has much larger metadata than older models
- **Large Tables**: ShotInfo can be several KB

## Implementation Timeline & Priorities

### Total Estimate: 40-60 hours for comprehensive support

**Recommended Priority Order**:

1. **Phase 1 & 2** (10-14 hours): Basic tags + key extraction - Quick wins
2. **Phase 3** (8-12 hours): Decryption for one model - Proves concept
3. **Phase 4** (partial, 6-9 hours): 3 popular models - Covers majority use
4. **Phase 6** (partial, 4-6 hours): Basic NEF support - RAW capability
5. **Phase 5** (as needed): Binary sections - Complete the implementation

**MVP Scope** (20-25 hours):

- Unencrypted tags working
- Decryption for D850/Z9
- Basic NEF support
- Would cover ~60% of Nikon users

## Recommendations

### Immediate Actions

1. **Study ExifTool's ProcessNikonEncrypted** thoroughly
2. **Start with Unencrypted Tags** to build confidence
3. **Focus on Popular Models** (D850, Z9 have highest usage)

### Long-term Strategy

1. **Leverage Codegen for Model Tables** - Each ShotInfo variant could be extracted
2. **Build Incremental Decryption** - Start with one model, expand systematically
3. **Consider Scope Reduction** - Support recent models first, add legacy later

## Comparison with Other Manufacturers

| Feature        | Nikon        | Canon        | Sony         | Olympus      |
| -------------- | ------------ | ------------ | ------------ | ------------ |
| Module Size    | 14,191 lines | 10,639 lines | 11,818 lines | 4,235 lines  |
| Tag Tables     | 135          | 107          | 139          | 15           |
| Lens Database  | 618 entries  | ~400 entries | ~500 entries | ~200 entries |
| Encryption     | Advanced     | None         | LFSR-based   | None         |
| Model Variants | 30+          | 20+          | 10+          | 5+           |
| AF Complexity  | Grid systems | Point-based  | Hybrid       | Simple       |

## Summary

Nikon implementation is significantly more complex than other manufacturers due to pervasive encryption, model-specific processing, and complex nested data structures. However, the existing codebase provides a solid foundation. Success requires methodical implementation starting with unencrypted tags, then building encryption support incrementally. Focus on recent camera models for maximum impact with minimum effort.
