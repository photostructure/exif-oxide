# Canon Lens Type Investigation & Implementation Plan

## Prerequisite reading

- doc/DESIGN.md
- doc/SYNC-DESIGN.md
- doc/SYNC-PRINTCONV-DESIGN.md

## Executive Summary

Most PrintConv are not functioning correctly:

```sh
mrm@speedy:~/src/exif-oxide$ cargo run -- -LensType -LensModel -RFLensType -json 
  "/home/mrm/src/exif-oxide/test-images/canon/canon_eos_r5_mark_ii_10.jpg" | fold | grep Lens
      Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
       Running `target/debug/exif-oxide -LensType -LensModel -RFLensType -json /home/mrm/src/exif-oxide/test-images/canon/canon_eos_r5_mark_ii_10.jpg`
  Warning: Skipping maker tag 0x4013 to avoid overflow
  Warning: Skipping maker tag 0x4008 to avoid overflow
  Warning: Skipping maker tag 0x4020 to avoid overflow
  Warning: Skipping maker tag 0x4021 to avoid overflow
  Warning: Skipping maker tag 0x4012 to avoid overflow
  Warning: Skipping maker tag 0x4019 to avoid overflow
  Warning: Skipping maker tag 0x4016 to avoid overflow
  Warning: Skipping maker tag 0x4025 to avoid overflow
  Warning: Skipping maker tag 0x4028 to avoid overflow
  Warning: Skipping maker tag 0x4011 to avoid overflow
  Warning: Skipping maker tag 0x404B to avoid overflow
  Warning: Skipping maker tag 0x402C to avoid overflow
  Warning: Skipping maker tag 0x4018 to avoid overflow
  Warning: Skipping maker tag 0x4010 to avoid overflow
  Warning: Skipping maker tag 0x4027 to avoid overflow
  Warning: Skipping maker tag 0x4001 to avoid overflow
  Warning: Skipping maker tag 0x4009 to avoid overflow
  Warning: Skipping maker tag 0x4015 to avoid overflow
      "LensType": "Undefined([2, 0, 93, 0, 3, 0, 0, 0, 0, 0, 144, 72, 255, 0, 0, 3
      "LensModel": "Undefined([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
  mrm@speedy:~/src/exif-oxide$ exiftool -LensType -LensModel -RFLensType -json "/home/mrm/src/exif-oxide/test-images/canon/canon_eos_r5_mark_ii_10.jpg"
  [{
    "SourceFile": "/home/mrm/src/exif-oxide/test-images/canon/canon_eos_r5_mark_ii_10.jpg",
    "LensType": "Canon RF 50mm F1.2L USM or other Canon RF Lens",
    "LensModel": "RF400mm F2.8 L IS USM",
    "RFLensType": "Canon RF 400mm F2.8L IS USM"
  }] 
```

 The issue involves multiple layers:

1. **Binary Data Extraction**: Canon stores lens information in ProcessBinaryData structures (CameraSettings, ShotInfo, etc.)
2. **PrintConv Mapping**: The sync system incorrectly maps `canonLensTypes` to `CanonLensType` instead of `CanonLensTypes`
3. **Tag Processing**: Canon tags in 0x4xxx range were being skipped due to overflow protection

## Current Status

### ✅ Completed
- Fixed IFD parser to handle Canon 0x4xxx tags without manufacturer prefix
- Updated `canon_tags.rs` to use `PrintConvId::None` for LensModel (0x0095) which is a string
- Identified that LensType (61182) comes from CameraSettings binary data at offset 0x0016
- Identified that RFLensType (289) comes from binary data at offset 0x003d

### ❌ Still Broken
- Canon ProcessBinaryData extraction not implemented for CameraSettings
- PrintConv extractor generates wrong mapping (CanonLensType vs CanonLensTypes)
- No actual lens type values being extracted from binary structures

## Root Cause Analysis

### 1. ProcessBinaryData Not Implemented

```
mrm@speedy:~/src/exif-oxide$ cargo run -- -LensType -LensModel -RFLensType -json "/home/mrm/src/exif-oxide/test-images/canon/canon_eos_r5_mark_ii_10.jpg" | fold | grep Lens
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/exif-oxide -LensType -LensModel -RFLensType -json /home/mrm/src/exif-oxide/test-images/canon/canon_eos_r5_mark_ii_10.jpg`
    "LensModel": "Unknown (Unknown (138))"
mrm@speedy:~/src/exif-oxide$ exiftool -LensType -LensModel -RFLensType -json "/home/mrm/src/exif-oxide/test-images/canon/canon_eos_r5_mark_ii_10.jpg"
[{
  "SourceFile": "/home/mrm/src/exif-oxide/test-images/canon/canon_eos_r5_mark_ii_10.jpg",
  "LensType": "Canon RF 50mm F1.2L USM or other Canon RF Lens",
  "LensModel": "RF400mm F2.8 L IS USM",
  "RFLensType": "Canon RF 400mm F2.8L IS USM"
}]
```

ExifTool output shows:
```
LensType = 61182  # From CameraSettings tag 0x0016
RFLensType = 289   # From another binary structure tag 0x003d
```

But exif-oxide shows:
```
"LensInfo": "Unknown (Unknown (30))",
"LensModel": "Undefined([0, 0, 0, 0, ...])"
```

The issue: Canon stores these values inside binary data structures that need ProcessBinaryData extraction.

### 2. PrintConv Mapping Bug

In `src/bin/exiftool_sync/extractors/printconv_tables.rs`:
```rust
fn generate_shared_printconv_id(&self, manufacturer: &str, table_name: &str) -> String {
    match table_name {
        "canonLensTypes" => format!("PrintConvId::{}LensType", manufacturer),  // WRONG!
        // Should be: "canonLensTypes" => "PrintConvId::CanonLensTypes",
```

This causes the extractor to generate `PrintConvId::CanonLensType` instead of `PrintConvId::CanonLensTypes`.

### 3. Canon Binary Data Structure

From ExifTool verbose output:
```
| | | | LensType = 61182
| | | | - Tag 0x0016 (2 bytes, int16u[1]):
| | | |     06d8: fe ee                                           [..]
```

This shows LensType is at offset 0x0016 within CameraSettings (tag 0x0001) binary data.

## Required Fixes

### 1. Fix PrintConv Extractor (High Priority)

**File**: `src/bin/exiftool_sync/extractors/printconv_tables.rs`

Change line ~372:
```rust
"canonLensTypes" => "PrintConvId::CanonLensTypes",
```

This is auto-generated code, so the fix needs to be in the extractor, not the generated file.

### 2. Implement Canon ProcessBinaryData (High Priority)

**New Sync Feature Required**: Extract CameraSettings binary data table

1. Check if `src/binary/formats/canon.rs` has CameraSettings table
2. If not, enhance `exiftool_sync extract binary-formats` to extract it
3. The table should include:
   - Offset 0x0016: LensType (u16)
   - Other CameraSettings fields

**Implementation Pattern** (following existing architecture):
```rust
// src/binary/formats/canon.rs
pub fn create_camerasettings_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("CameraSettings", ExifFormat::I16)
        .add_field(0x0016, "LensType", ExifFormat::U16, 1)
        // ... other fields
        .build()
}
```

### 3. Wire Up Binary Data Processing in Canon Parser

**File**: `src/maker/canon.rs`

The Canon parser needs to:
1. Detect CameraSettings tag (0x0001)
2. Apply ProcessBinaryData extraction
3. Store extracted values (like LensType) as proper tags
4. Apply PrintConv using CanonLensTypes lookup

## Research Needed

### 1. Full CameraSettings Structure

Need to extract the complete CameraSettings binary data table from ExifTool:
- Check `third-party/exiftool/lib/Image/ExifTool/Canon.pm` for CameraSettings definition
- Look for `ProcessBinaryData` tables
- Extract all field offsets and types

### 2. Other Binary Structures

RFLensType (289) comes from a different binary structure. Need to identify:
- Which Canon tag contains this structure
- The offset within that structure
- The complete structure definition

### 3. Comprehensive PrintConv Audit

The PrintConv system needs verification:
- Are other shared lookup tables affected by the same bug?
- Check mappings for: nikonLensTypes, pentaxLensTypes, sonyLensTypes
- Verify the complete PrintConv extraction and mapping flow

## Implementation Roadmap

### Phase 1: Quick Fixes (1-2 hours)
1. Fix `generate_shared_printconv_id` to map canonLensTypes correctly
2. Run `make sync` to regenerate tables
3. Test with Canon R5 Mark II image

### Phase 2: ProcessBinaryData Implementation (4-8 hours)
1. Research CameraSettings structure in ExifTool source
2. Enhance sync extractor to generate binary data tables for Canon
3. Implement binary data processing in Canon parser
4. Wire up extracted values to use proper PrintConv

### Phase 3: Comprehensive Solution (1-2 days)
1. Audit all manufacturer PrintConv mappings
2. Implement missing binary data structures
3. Add integration tests for lens type extraction
4. Document the complete ProcessBinaryData flow

## Test Cases

1. **Canon R5 Mark II**: 
   - Expected: LensType = "Canon RF 50mm F1.2L USM or other Canon RF Lens"
   - Expected: RFLensType = "Canon RF 400mm F2.8L IS USM"

2. **Other Canon cameras**: Need test images with various lens types

## Key Files to Modify

1. `src/bin/exiftool_sync/extractors/printconv_tables.rs` - Fix mapping bug
2. `src/bin/exiftool_sync/extractors/binary_formats.rs` - Add CameraSettings extraction
3. `src/maker/canon.rs` - Implement ProcessBinaryData handling
4. `src/binary/formats/canon.rs` - Generated binary tables (DO NOT EDIT directly)

## ExifTool Source References

Key files to study in `third-party/exiftool/`:
- `lib/Image/ExifTool/Canon.pm` - CameraSettings and other binary structures
- `lib/Image/ExifTool.pm` - ProcessBinaryData implementation (lines 6000+)
- `exiftool` - ConvertBinary function (lines 3891-3920)

## Success Criteria

1. `cargo run -- -LensModel -LensType -RFLensType test.jpg` shows human-readable lens names
2. Output matches ExifTool exactly
3. All Canon lens types from the 524-entry table work correctly
4. ProcessBinaryData extraction is table-driven and maintainable

## Notes for Next Engineer

1. The IFD parser fix for 0x4xxx tags is already done - don't revert it
2. The PrintConv system is table-driven - study `doc/SYNC-PRINTCONV-DESIGN.md`
3. Binary data extraction follows patterns in `src/core/binary_data.rs`
4. Always check ExifTool source - it has 25 years of camera quirks encoded
5. Run `make sync` after any extractor changes to regenerate tables

## Priority

This is **HIGH PRIORITY** because:
- Lens information is mainstream metadata (used by 85%+ of photos)
- It affects all Canon cameras
- The fix will establish patterns for other manufacturers
- ProcessBinaryData is critical for many other tags