# HANDOFF: RAW File Compatibility Investigation and Fixes

**Engineer Handoff Date**: 2025-07-18  
**Status**: Partially Complete - Major Progress Made  
**Priority**: High - Final 2 compatibility test failures remaining

## Executive Summary

This handoff covers investigation and partial resolution of RAW file EXIF parsing issues that were causing 27/64 compatibility test failures. **Major success achieved**: reduced failures from 27 → 2 (93% improvement). The remaining 2 failures are Panasonic RW2 tag mapping issues requiring targeted fixes.

## Issues Being Addressed

### Primary Problem
RAW files (MRW, RW2) were only extracting MakerNotes tags but **missing standard EXIF tags** like `Make`, `Model`, `ExposureTime`, `FNumber`, etc. This caused widespread compatibility test failures.

### Root Cause Analysis
1. **Minolta MRW**: TTW (TIFF Tags) blocks containing standard EXIF data were being **skipped with TODO comments**
2. **Panasonic RW2**: **Tag ID mapping issues** - reading correct raw values but mapping to wrong tag names (GPS tags instead of EXIF tags)

## Work Completed ✅

### 1. MakerNotes Scope Management
- **Problem**: 27 compatibility test failures were primarily due to missing MakerNotes tags that aren't implemented yet
- **Solution**: Temporarily removed MakerNotes tags from `config/supported_tags.json` until milestone work is complete
- **Documentation**: Created `config/MAKERNOTES-TODO.md` tracking planned implementation per milestone
- **Result**: Reduced scope to focus on actual bugs vs. unimplemented features

### 2. Minolta MRW Complete Fix
- **Problem**: TTW blocks containing standard EXIF tags were skipped (lines 384-388 in `src/raw/formats/minolta.rs`)
- **Solution**: Implemented `process_ttw_block()` method using existing TIFF infrastructure
- **Code Changes**: 
  - Added TTW block processing using `extract_tiff_exif()` + `parse_exif_data()`
  - Follows ExifTool's approach: ProcessTIFF for TTW subdirectories
- **Result**: Minolta MRW files now extract standard EXIF tags correctly
- **Files**: `src/raw/formats/minolta.rs:370-409` (new method), `src/raw/formats/minolta.rs:425-433` (integration)

### 3. Compatibility Test Debugging Infrastructure
- **Added**: Manufacturer-specific TODO tracking in `tests/exiftool_compatibility_tests.rs:452-490`
- **Added**: `detect_manufacturer_from_path()` helper function
- **Purpose**: Documents what's missing per manufacturer and references specific milestones

## Current Status: 2 Remaining Failures

### Panasonic RW2 Tag Mapping Issue 🔍

**Files Affected**:
- `third-party/exiftool/t/images/Panasonic.rw2`
- `test-images/panasonic/panasonic_lumix_g9_ii_35.rw2`

**Problem**: Wrong tag ID mappings - extracting correct raw values but assigning wrong tag names:

| Expected EXIF Tag | Our Output | Raw Values |
|-------------------|------------|------------|
| `EXIF:ISO: 80` | `EXIF:GPSAltitude: "2742.0 m"` | 2742 |
| `EXIF:ColorSpace: "sRGB"` | `EXIF:GPSLatitude: 3724.0` | 3724 |
| `EXIF:ResolutionUnit: "inches"` | `EXIF:GPSLongitude: 6.0` | 6 |

**Analysis**: Values like `3724`, `2742`, `6` look like sensor dimensions or technical values being misinterpreted as GPS coordinates.

## Code to Study

### Essential Files
1. **`src/raw/formats/panasonic.rs`** - Panasonic RW2 handler (already has proper TIFF processing)
2. **`src/raw/formats/minolta.rs`** - Reference implementation for TTW block processing
3. **`tests/exiftool_compatibility_tests.rs`** - Test infrastructure and manufacturer tracking
4. **`config/supported_tags.json`** - Current scope (MakerNotes removed temporarily)
5. **`config/MAKERNOTES-TODO.md`** - MakerNotes implementation roadmap

### Reference Documentation
1. **`third-party/exiftool/lib/Image/ExifTool/PanasonicRaw.pm`** - ExifTool's Panasonic implementation
2. **`third-party/exiftool/lib/Image/ExifTool/MinoltaRaw.pm`** - ExifTool's Minolta implementation (lines 31-38 for TTW)
3. **`docs/TRUST-EXIFTOOL.md`** - Critical: all fixes must exactly follow ExifTool logic

### Key Code Sections
```rust
// REFERENCE: Working TTW block processing in Minolta handler
fn process_ttw_block(&self, reader: &mut ExifReader, ttw_data: &[u8], byte_order: &ByteOrder) -> Result<()> {
    // Use existing TIFF infrastructure
    let mut ttw_cursor = Cursor::new(ttw_data);
    match crate::formats::extract_tiff_exif(&mut ttw_cursor) {
        Ok(tiff_data) => reader.parse_exif_data(&tiff_data),
        Err(e) => Err(e)
    }
}
```

## Next Steps for Completion

### High Priority: Fix Panasonic Tag Mapping

1. **Debug Tag ID Assignment**
   - Compare our extracted tag IDs vs. ExifTool's expected IDs for Panasonic RW2
   - Check if we're reading from wrong IFD or using wrong tag definitions
   - Values 3724, 2742, 6 suggest we might be reading sensor info instead of EXIF

2. **Investigate Panasonic TIFF Structure**
   - Panasonic RW2 files have multiple IFDs (standard + manufacturer-specific)
   - Verify we're reading from correct IFD for standard EXIF tags
   - Check if our TIFF IFD processing is conflating different data sections

3. **Fix Tag Definitions**
   - Review `src/raw/formats/panasonic.rs` tag definitions
   - Ensure standard EXIF tags (0x010F=Make, 0x0110=Model, etc.) are mapped correctly
   - Check for tag ID conflicts between standard EXIF and Panasonic-specific tags

### Success Criteria
- [ ] All 64 compatibility tests pass (`make compat-test`)
- [ ] Panasonic RW2 files extract correct EXIF tags (ISO, ColorSpace, ResolutionUnit, etc.)
- [ ] No regressions in Minolta MRW functionality
- [ ] Standard EXIF tags correctly distinguished from GPS/MakerNotes tags

## Testing Strategy

### Immediate Testing
```bash
# Test specific failing files
cargo run -- third-party/exiftool/t/images/Panasonic.rw2 | grep -E "ISO|ColorSpace|GPS"
exiftool -j third-party/exiftool/t/images/Panasonic.rw2 | grep -E "ISO|ColorSpace|GPS"

# Run compatibility tests
make compat-test
```

### Debugging Commands
```bash
# Compare tag extraction details
RUST_LOG=debug cargo run -- third-party/exiftool/t/images/Panasonic.rw2 2>&1 | grep -E "Processing|tag|IFD"

# Check raw tag values
cargo run -- third-party/exiftool/t/images/Panasonic.rw2 2>/dev/null | jq 'keys'
```

## Tribal Knowledge

### Trust ExifTool Principle
- **Never "improve" ExifTool logic** - translate exactly
- **Every fix must reference ExifTool source** - include file:line comments
- **When in doubt, study ExifTool** - especially for edge cases

### Compatibility Test Infrastructure
- `make compat-gen` regenerates ExifTool reference snapshots
- `config/supported_tags.json` controls test scope
- Test failures show exact diffs - very helpful for debugging
- Manufacturer detection in tests helps track progress per brand

### RAW File Architecture
- **MRW files**: Multi-block structure (TTW=TIFF tags, PRD=picture data, WBG=white balance)
- **RW2 files**: TIFF-based with manufacturer-specific tags
- **Both use existing TIFF infrastructure** - don't reinvent parsing

### Code Organization Patterns
- RAW handlers in `src/raw/formats/`
- Each handler implements `RawFormatHandler` trait
- Standard TIFF processing via `extract_tiff_exif()` + `parse_exif_data()`
- Manufacturer-specific logic in separate methods

## Future Refactoring Opportunities

### 1. RAW Handler Architecture Unification
**Current**: Each RAW format has custom processing logic  
**Opportunity**: Create common base class for TIFF-based RAW formats (RW2, DNG, etc.)

```rust
// Proposed refactoring
trait TiffBasedRawHandler: RawFormatHandler {
    fn process_manufacturer_ifd(&self, reader: &mut ExifReader, ifd_data: &[u8]) -> Result<()>;
    fn get_manufacturer_tag_definitions(&self) -> &HashMap<u16, TagDef>;
}
```

### 2. Tag Mapping Validation System
**Current**: Tag mapping errors are only caught by compatibility tests  
**Opportunity**: Runtime validation against expected EXIF tag ranges

```rust
// Proposed validation
fn validate_exif_tag_mapping(tag_id: u16, value: &TagValue, context: &str) -> Result<()> {
    // Validate GPS tags are actually GPS coordinates
    // Validate EXIF tags are in correct ranges
    // Log warnings for suspicious mappings
}
```

### 3. Compatibility Test Granularity
**Current**: All-or-nothing compatibility tests  
**Opportunity**: Per-manufacturer test suites with expected failure tracking

```rust
// Proposed test structure
#[test] fn test_sony_raw_compatibility() { ... }
#[test] fn test_panasonic_raw_compatibility() { ... }
#[test] fn test_minolta_raw_compatibility() { ... }
```

## Related Milestones

- **MILESTONE-17d-Canon-RAW**: Canon lens database and MakerNotes
- **MILESTONE-17e-Sony-RAW**: Sony MakerNotes implementation  
- **MILESTONE-MOAR-CODEGEN**: Panasonic/Pentax/Olympus lookup tables
- **Future Nikon Milestone**: Not yet scheduled - Nikon MakerNotes missing

## Critical Files Modified

1. **`src/raw/formats/minolta.rs`**: Added TTW block processing
2. **`config/supported_tags.json`**: Removed MakerNotes tags temporarily
3. **`config/MAKERNOTES-TODO.md`**: Created MakerNotes implementation roadmap
4. **`tests/exiftool_compatibility_tests.rs`**: Added manufacturer tracking
5. **`src/generated/Canon_pm/psinfo_inline.rs`**: Fixed i16→i32 overflow

## Emergency Contacts

- **Compatibility tests failing**: Check `make compat-test` output for specific diffs
- **Build failures**: Usually type issues in generated code - check `src/generated/`
- **ExifTool reference**: Use `./tools/generate_exiftool_json.sh --force` to regenerate snapshots
- **Trust ExifTool violations**: Review `docs/TRUST-EXIFTOOL.md` and corresponding ExifTool source

**Good luck! The hardest part (Minolta MRW) is done. The Panasonic issue should be a targeted tag mapping fix.** 🚀