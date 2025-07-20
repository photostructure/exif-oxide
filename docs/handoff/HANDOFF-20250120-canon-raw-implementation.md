# Handoff: Canon RAW Implementation Progress

**Milestone**: 17d - Canon RAW Support

## Overview

This handoff document captures the current state of Canon RAW format implementation for CR2, CRW, and CR3 formats. The work is approximately 50% complete with major infrastructure in place.

## Current Status

### Completed Tasks ✅

1. **Canon Format Detection and CLI Integration**
   - Added CR2, CRW, CR3 support to `src/formats/mod.rs` extract_metadata function
   - CR2 correctly routes through TIFF processor (since CR2 is TIFF-based)
   - CRW and CR3 route through RAW processor
   - Files: `src/formats/mod.rs` lines 507-591

2. **Canon Offset Scheme Implementation**
   - Existing implementation in `src/implementations/canon/offset_schemes.rs`
   - Supports 4/6/16/28 byte offset variants based on camera model
   - Model detection patterns implemented (20D, 350D, REBEL XT, etc.)

3. **Canon RAW Handler Infrastructure**
   - Created `src/raw/formats/canon.rs` with CanonRawHandler
   - Registered in RAW processor (`src/raw/processor.rs`)
   - Format detection working in `src/raw/detector.rs`

4. **Generated Code Available**
   - Canon tag structure enum: `src/generated/Canon_pm/tag_structure.rs`
   - Conditional tags: `src/generated/Canon_pm/main_conditional_tags.rs`
   - Various inline PrintConv tables in `src/generated/Canon_pm/`
   - 84 Canon-specific tags with proper group hierarchy

### Pending Tasks ⏳

1. **Integrate Generated ProcessBinaryData Processors** (HIGH PRIORITY)
   - Canon has 169 ProcessBinaryData sections that need integration
   - Generated processors should be available via codegen
   - Need to map CanonDataType enum values to appropriate processors

2. **Handle Canon-specific IFDs and Maker Notes**
   - Implement proper Canon IFD processing in process_cr2()
   - Route Canon maker note tags through ProcessBinaryData handlers
   - Handle subdirectories for tags like CameraSettings, AFInfo, etc.

3. **Canon Color Data Processing**
   - Implement ColorData1-12 processing (conditional based on count)
   - Different camera generations use different ColorData formats

4. **Add CR2 to Compatibility Script**
   - Update `tools/generate_exiftool_json.sh` SUPPORTED_EXTENSIONS
   - Run `make compat-gen` to generate reference files
   - Ensure tests pass with `make compat-test`

5. **Write Comprehensive Tests**
   - Need test CR2 files from various Canon cameras
   - Test offset scheme detection for different models
   - Verify tag extraction matches ExifTool output

## Key Code Locations

### Core Implementation Files
- `src/raw/formats/canon.rs` - Main Canon RAW handler (partially complete)
- `src/implementations/canon/` - Existing Canon implementation modules
  - `offset_schemes.rs` - Offset detection (complete)
  - `binary_data.rs` - Binary data extraction functions
  - `tags.rs` - Tag name resolution
  - `af_info.rs` - AF Info processing
  - `mod.rs` - Canon coordinator

### Generated Code
- `src/generated/Canon_pm/` - All generated Canon tables
  - `tag_structure.rs` - CanonDataType enum with 84 tags
  - `main_conditional_tags.rs` - Conditional tag resolution
  - Various inline PrintConv tables

### Integration Points
- `src/formats/mod.rs` - CLI entry point (lines 507-591)
- `src/raw/processor.rs` - RAW processor registration
- `src/raw/detector.rs` - Format detection

## Issues Encountered and Solutions

### 1. Duplicate Code Issue
**Problem**: Started implementing offset management in canon.rs that already existed in implementations/canon/offset_schemes.rs
**Solution**: Import and use existing implementation instead of duplicating

### 2. CR2 Processing Path
**Problem**: CR2 is TIFF-based but needs special Canon handling
**Solution**: Route CR2 through TIFF processor which will call Canon maker note handlers

### 3. Missing Model Detection Codegen
**Problem**: No generated model detection patterns for Canon
**Solution**: Model detection is already implemented manually in offset_schemes.rs

## Technical Details to Know

### Canon Offset Schemes
Canon uses different offset schemes based on camera model:
- **4 bytes**: Default for most cameras
- **6 bytes**: 20D, 350D, REBEL XT, Kiss Digital N
- **16 bytes**: PowerShot, IXUS, IXY models
- **28 bytes**: FV-M30, Optura series

### CR2 Structure
```
CR2 File:
├── TIFF Header
├── IFD0 (Main Image)
├── IFD1 (Thumbnail)
├── EXIF IFD
│   └── Maker Note IFD (Canon-specific)
│       ├── Camera Settings (0x0001)
│       ├── Focal Length (0x0002)
│       ├── Shot Info (0x0003)
│       └── ... (80+ more tags)
├── Canon Color Data
└── RAW Image Data
```

### Trust ExifTool Principle
- Never "improve" or "optimize" ExifTool's logic
- Every quirk exists for a reason (usually specific camera bugs)
- Reference ExifTool source with file:line comments

## Next Steps

1. **Remove duplicate offset code** from canon.rs and use existing implementation
2. **Study Canon.pm ProcessBinaryData sections** to understand the 169 data processors
3. **Implement Canon IFD processing** in process_cr2() method
4. **Map CanonDataType enum** to appropriate binary data processors
5. **Test with real CR2 files** from various Canon cameras

## Refactoring Opportunities

1. **Consolidate Canon Implementations**
   - Currently split between `raw/formats/canon.rs` and `implementations/canon/`
   - Consider moving all Canon logic to one location

2. **Generate ProcessBinaryData Mappings**
   - The 169 ProcessBinaryData sections could be auto-generated
   - Would eliminate manual maintenance burden

3. **Unified Offset Management**
   - Canon offset schemes could be part of a larger offset management system
   - Other manufacturers (Sony, Panasonic) have similar patterns

4. **Conditional Tag System**
   - The conditional tag resolution in main_conditional_tags.rs is complex
   - Could benefit from a more unified expression evaluation system

## Success Criteria

- [ ] CLI can successfully read CR2 files and extract metadata
- [ ] Output matches `exiftool -j` for all Canon-specific tags
- [ ] All 84 CanonDataType tags are properly extracted
- [ ] Offset schemes work correctly for different camera models
- [ ] Tests pass for Canon sample files

## References

- ExifTool Canon.pm: 10,648 lines of Canon-specific processing
- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - Critical principle
- [MILESTONE-17d-Canon-RAW.md](../todo/MILESTONE-17d-Canon-RAW.md) - Original requirements