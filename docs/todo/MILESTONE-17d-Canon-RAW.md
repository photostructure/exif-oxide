# Milestone 17d: Canon RAW Support

**Goal**: Implement Canon RAW formats (CR2 required, CRW/CR3 optional)  
**Status**: 85% Complete - Core infrastructure working, tag extraction active, output issues remain  

## High level guidance

- **Follow Trust ExifTool**: Study how ExifTool processes CR2 files exactly. See [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md)
- **Use Codegen**: Switch any manual lookup tables to generated code. See [EXIFTOOL-INTEGRATION.md](../design/EXIFTOOL-INTEGRATION.md)
- **Study ExifTool Sources**: [Canon.pm](../../third-party/exiftool/lib/Image/ExifTool/Canon.pm) and [module docs](../../third-party/exiftool/doc/modules/Canon.md)

## Overview

Canon RAW support with comprehensive maker note processing. The infrastructure is solid and extracting Canon tags correctly, but needs output formatting fixes and coverage expansion.

**Complexity Context**:
- 10,648 lines in ExifTool Canon.pm
- 7 ProcessBinaryData sections (confirmed by source analysis)
- 84 Canon data types with generated lookup tables
- Complex offset schemes and validation requirements

## 🎯 Current Status

### ✅ Completed Infrastructure (85%)

1. **Canon IFD Parsing** - `find_canon_tag_data()` correctly extracts Canon maker note tags
2. **Binary Data Processing** - 5 of 7 ProcessBinaryData sections implemented:
   - CameraSettings (0x0001) - 6 tags extracted
   - FocalLength (0x0002) - 4 tags extracted  
   - ShotInfo (0x0004) - 8 tags extracted
   - AFInfo (0x0012) - ProcessSerialData with variable arrays
   - AFInfo2 (0x0026) - ProcessSerialData with proper offset handling
   - Panorama (0x0005) - 2 tags extracted with generated lookups
   - MyColors (0x001d) - 1 tag extracted with validation

3. **Generated Code Integration** - Using lookup tables from `src/generated/Canon_pm/*_inline.rs`
4. **Tag Naming** - Fixed duplicate prefix issue (`MakerNotes:MakerNotes:` → `MakerNotes:`)
5. **Offset Management** - Absolute vs relative offset handling working correctly
6. **PrintConv System** - Generated lookup tables applied successfully

### 🔧 Remaining Critical Issues

1. **OUTPUT FORMATTING BUG** (BLOCKER)
   - **Issue**: Canon tags stored correctly but not appearing in JSON output
   - **Evidence**: Debug shows "Stored Canon tag MakerNotes:MacroMode" but validation finds 0 tags
   - **Root Cause**: Synthetic tag IDs (0xC000+) not mapping back to stored names in output generation
   - **Impact**: 0 exif-oxide tags vs 232 ExifTool tags in validation

2. **Missing Canon Coverage** (200+ tags)
   - **Current**: Only ProcessBinaryData tags extracted
   - **Missing**: Main Canon table tags (strings, simple values, complex subdirectories)
   - **Examples**: CanonImageType, FirmwareVersion, OwnerName, SerialNumber, LensType, ModelID

3. **Manual Lookup Tables** (TRUST-EXIFTOOL violation)
   - **Found**: Hardcoded PrintConv tables in `binary_data.rs` CameraSettings processing
   - **Should Use**: Generated functions from `canonlenstypes.rs`, `canonmodelid.rs`, `canonquality.rs`

## 🚨 Immediate Tasks (Priority Order)

### 1. **CRITICAL: Fix Synthetic Tag Output** 
**Problem**: Canon tags stored with synthetic IDs but not appearing in final JSON
**Solution**: Debug and fix tag name resolution in output generation
**Location**: Check how synthetic_tag_names HashMap is used in JSON output
**Test**: Validation should show Canon tags instead of 0 tags

### 2. **HIGH: Replace Manual Lookups with Codegen**
**Violations Found**:
- `apply_camera_settings_print_conv()` - manually coded, should use generated tables
- Need to use: `lookup_canon_lens_type()`, `lookup_canon_model_id()`, `lookup_canon_quality()`
**Files**: `src/implementations/canon/binary_data.rs`, generated lookup functions

### 3. **HIGH: Add Missing Canon Main Table Tags**
**Missing**: 230+ Canon tags that ExifTool extracts
**Examples**: Simple string/value tags from Canon main table
**Reference**: `third-party/exiftool/lib/Image/ExifTool/Canon.pm` %Canon::Main table
**Implementation**: Add processing for non-ProcessBinaryData Canon tags

## 🧠 Critical Tribal Knowledge

### ExifTool Canon Implementation Facts
- **ProcessBinaryData Count**: 7 sections total (not 169 as originally estimated)
- **Binary Data Formats**: CameraSettings/ShotInfo use `int16s` (signed), FocalLength uses `int16u` (unsigned)
- **Offset Handling**: Canon IFD offsets are **absolute file offsets**, not relative to maker note
- **Validation**: MyColors section includes size validation (first 16-bit value = data length)

### Key Code Patterns
1. **Binary Data Extraction**: Follow exact pattern from `extract_camera_settings()`
2. **Generated Lookups**: Always use `crate::generated::Canon_pm::*_inline::lookup_*()`  
3. **Tag Naming**: Functions return `"MakerNotes:TagName"` - no additional prefix needed
4. **Synthetic IDs**: Range 0xC000-0xCFFF with hash-based generation for uniqueness

### Offset Management (CRITICAL)
```rust
// CORRECT: Use absolute file offsets for Canon IFD data
find_canon_tag_data_with_full_access(full_data, maker_note_data, maker_note_offset, tag_id)

// WRONG: Relative offsets won't work for large data sections
find_canon_tag_data(maker_note_data, tag_id) 
```

## 📚 Essential References

### ExifTool Source (GOSPEL)
- **Canon.pm**: `third-party/exiftool/lib/Image/ExifTool/Canon.pm`
  - Main table: Lines ~500-800
  - ProcessBinaryData sections: Search `PROCESS_PROC.*ProcessBinaryData`
  - PrintConv tables: Individual table definitions

### Implementation Files
- **Main Coordinator**: `src/implementations/canon/mod.rs`
- **Binary Extraction**: `src/implementations/canon/binary_data.rs` 
- **AF Processing**: `src/implementations/canon/af_info.rs`
- **Generated Lookups**: `src/generated/Canon_pm/*_inline.rs`
- **Tag Structure**: `src/generated/Canon_pm/tag_structure.rs`

### Testing & Validation
- **Test Images**: `test-images/canon/Canon_T3i.CR2`
- **Validation**: `cargo run --bin compare-with-exiftool image.cr2 MakerNotes:`
- **Debug**: `RUST_LOG=debug ./target/release/exif-oxide image.cr2`

## 🔍 Debugging Commands

```bash
# Check Canon tag extraction
RUST_LOG=debug ./target/release/exif-oxide test-images/canon/Canon_T3i.CR2 2>&1 | grep "Stored Canon tag"

# Validate against ExifTool  
cargo run --bin compare-with-exiftool test-images/canon/Canon_T3i.CR2 MakerNotes:

# Check JSON output
./target/release/exif-oxide test-images/canon/Canon_T3i.CR2 | grep -E "MakerNotes:"
```

## ✅ Success Criteria

1. **Tag Count**: Match ExifTool's ~232 Canon MakerNotes tags (currently 0 visible)
2. **Core Tags**: All ProcessBinaryData sections extracting tags correctly 
3. **Generated Code**: No manual lookup tables - all using generated functions
4. **Validation**: `compare-with-exiftool` shows matching tag counts and values
5. **Performance**: `make precommit` passes without errors

## 🎯 Completion Estimate

- **Output Fix**: 2-4 hours (critical path)
- **Codegen Migration**: 1-2 hours (cleanup)  
- **Missing Tags**: 4-8 hours (main implementation work)
- **Total Remaining**: 1-2 days

**Next Milestone**: 17e - Sony ARW (advanced offset management patterns)

## 🔄 Handoff Notes

The Canon implementation has **solid foundations** with working IFD parsing, binary data extraction, and generated lookup integration. The critical blocker is output formatting - Canon tags are being extracted and stored correctly but not appearing in the final JSON due to synthetic tag ID resolution issues.

**Start Here**: Debug the synthetic tag output issue first - this will immediately show dramatic improvement in validation results and unlock visibility into the working Canon tag extraction.