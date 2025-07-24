# Technical Project Plan: Complete Panasonic RW2 Dimension Extraction

## Project Overview

**Goal**: Complete implementation of dimension extraction for Panasonic RW2 (RAW) files to match ExifTool's output exactly.

**Problem Statement**: ExifTool extracts dimensions from RW2 files using Panasonic-specific sensor border tags, but our current implementation fails due to incorrect TIFF header detection.

## Background & Context

- **User Need**: PhotoStructure requires reliable image dimensions for proper display and organization
- **Status**: 90% complete - sensor border calculation logic implemented, blocked on TIFF header recognition
- **Parent Project**: [dimension-required-tags.md](20250122-dimension-required-tags.md) - comprehensive dimension extraction across all image formats
- **Test File**: `test-images/panasonic/panasonic_lumix_g9_ii_35.rw2` (5776×4336 expected dimensions)

## Technical Foundation

### Key Documentation
- **CLAUDE.md**: Essential project guidelines and Trust ExifTool principle
- **TRUST-EXIFTOOL.md**: Core principle - copy ExifTool logic exactly, never "improve"
- **ExifTool PanasonicRaw.pm**: Lines 675-690 contain composite dimension calculation logic

### Key Codebases
- `src/raw/mod.rs::utils::extract_tiff_dimensions()`: Shared dimension extraction utility (lines 264-536)
- `src/formats/mod.rs`: Format dispatch logic (RW2 added at line 481)
- `third-party/exiftool/lib/Image/ExifTool/PanasonicRaw.pm`: Reference implementation

### ExifTool Reference Algorithm
```perl
# PanasonicRaw.pm:675-690
%Image::ExifTool::PanasonicRaw::Composite = (
    ImageWidth => {
        Require => {
            0 => 'IFD0:SensorLeftBorder',
            1 => 'IFD0:SensorRightBorder',
        },
        ValueConv => '$val[1] - $val[0]',
    },
    ImageHeight => {
        Require => {
            0 => 'IFD0:SensorTopBorder', 
            1 => 'IFD0:SensorBottomBorder',
        },
        ValueConv => '$val[1] - $val[0]',
    },
);
```

## Work Completed

### ✅ Research & Analysis
- **ExifTool Deep Dive**: Studied PanasonicRaw.pm to understand RW2 uses sensor border tags (0x04-0x07) instead of standard ImageWidth/ImageHeight
- **Verbose Output Analysis**: Confirmed test file has sensor borders: left=4, right=5780, top=4, bottom=4340
- **Expected Values**: Width = 5780-4 = 5776, Height = 4340-4 = 4336

### ✅ Implementation Framework
- **Sensor Border Detection**: Added tags 0x04-0x07 parsing in `extract_tiff_dimensions()` (lines 464-523)
- **Calculation Logic**: Implemented ExifTool's exact formula: width = right - left, height = bottom - top (lines 583-609)
- **Integration**: Added RW2 to TIFF processing branch in `formats/mod.rs:481`

### ✅ Code Structure
```rust
// Panasonic RW2 sensor border tags (implemented)
let mut sensor_top_border: Option<u16> = None;     // Tag 0x04
let mut sensor_left_border: Option<u16> = None;    // Tag 0x05  
let mut sensor_bottom_border: Option<u16> = None;  // Tag 0x06
let mut sensor_right_border: Option<u16> = None;   // Tag 0x07

// Calculation logic (implemented)
let panasonic_width = right - left;   // 5780 - 4 = 5776
let panasonic_height = bottom - top;  // 4340 - 4 = 4336
```

## Remaining Tasks

### 🚨 **CRITICAL BLOCKER** - Fix TIFF Header Detection

**Problem**: RW2 files use magic bytes `IIU\0` (not `II*\0`) so they're rejected by our TIFF parser.

**Evidence**:
```bash
$ hexdump -C test-images/panasonic/panasonic_lumix_g9_ii_35.rw2 | head -1
00000000  49 49 55 00 18 00 00 00  88 e7 74 d8 f8 25 1d 4d  |IIU.......t..%.M|
         ^^ ^^ ^^ ^^
         I  I  U  \0  (RW2 magic bytes)
```

**Current Code** (`src/raw/mod.rs:283`):
```rust
// FAILS for RW2 - only accepts II*\0 and MM\0*
let (is_little_endian, ifd0_offset) = match &data[0..4] {
    [0x49, 0x49, 0x2A, 0x00] => { // II*\0 (standard TIFF)
    [0x4D, 0x4D, 0x00, 0x2A] => { // MM\0* (standard TIFF)
    _ => {
        debug!("Invalid TIFF magic bytes in RAW file"); // ← RW2 hits this
        return Ok(());
    }
};
```

**Solution**: Add RW2 magic byte support:
```rust
let (is_little_endian, ifd0_offset) = match &data[0..4] {
    [0x49, 0x49, 0x2A, 0x00] => { /* II*\0 - standard TIFF */ }
    [0x4D, 0x4D, 0x00, 0x2A] => { /* MM\0* - standard TIFF */ }
    [0x49, 0x49, 0x55, 0x00] => { /* IIU\0 - Panasonic RW2 */ 
        // Same logic as little-endian TIFF
        if data.len() < 8 { return Ok(()); }
        let ifd0_offset = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
        (true, ifd0_offset)
    }
    _ => { /* Invalid magic bytes */ }
};
```

### 🔧 **Testing & Validation**

1. **Manual Testing**:
   ```bash
   RUST_LOG=debug cargo run -- test-images/panasonic/panasonic_lumix_g9_ii_35.rw2 | grep -E "(ImageWidth|ImageHeight)"
   # Should output: EXIF:ImageWidth: 5776, EXIF:ImageHeight: 4336
   ```

2. **ExifTool Comparison**:
   ```bash
   exiftool -j -G test-images/panasonic/panasonic_lumix_g9_ii_35.rw2 | grep -E "(ImageWidth|ImageHeight)"
   # Expected: "EXIF:ImageWidth": 5776, "EXIF:ImageHeight": 4336
   ```

3. **Compatibility Testing**:
   ```bash
   make compat  # Add RW2 test file to compat system
   ```

## Prerequisites

None - all dependencies already in place.

## Testing Strategy

### Unit Tests
- **Location**: `src/raw/mod.rs::tests` (existing test infrastructure)
- **Add**: Test cases for RW2 magic byte detection and sensor border calculation

### Integration Tests  
- **Compatibility Tests**: Add RW2 file to `make compat` system
- **Cross-verify**: Ensure dimensions match ExifTool exactly

### Manual Testing
```bash
# 1. Test dimension extraction
cargo run -- test-images/panasonic/panasonic_lumix_g9_ii_35.rw2

# 2. Verify against ExifTool
exiftool -j -G test-images/panasonic/panasonic_lumix_g9_ii_35.rw2 | grep ImageWidth

# 3. Check debug output  
RUST_LOG=debug cargo run -- test-images/panasonic/panasonic_lumix_g9_ii_35.rw2 2>&1 | grep -E "(Sensor|Calculated)"
```

## Success Criteria & Quality Gates

### ✅ **Definition of Done**
1. **Exact ExifTool Match**: Dimensions (5776×4336) output exactly as ExifTool
2. **Group Assignment**: Tags assigned to EXIF group (not File group)
3. **Debug Output**: Clear logging shows sensor border detection and calculation
4. **Integration**: RW2 files processed through TIFF branch successfully
5. **Compatibility**: `make compat` passes with RW2 test file
6. **Code Quality**: `make precommit` passes (clippy + tests)

### 🎯 **Expected Output**
```json
{
  "EXIF:ImageWidth": 5776,
  "EXIF:ImageHeight": 4336,
  "EXIF:SensorLeftBorder": 4,
  "EXIF:SensorRightBorder": 5780,
  "EXIF:SensorTopBorder": 4,
  "EXIF:SensorBottomBorder": 4340
}
```

## Gotchas & Tribal Knowledge

### 🚨 **Critical Insights**

1. **Trust ExifTool Principle**: Never attempt to "improve" logic - copy exactly, even if it seems inefficient
2. **RW2 is TIFF-variant**: Uses TIFF structure but with non-standard magic bytes `IIU\0`
3. **Sensor Borders ≠ Image Dimensions**: RW2 calculates dimensions from sensor crop area, not direct tags
4. **ExifTool Group Assignment**: Panasonic dimensions go to EXIF group (from TIFF tags), not File group

### 🔍 **Debug Techniques**

- **ExifTool Verbose**: `exiftool -v3 file.rw2` shows exact parsing steps
- **Hex Analysis**: `hexdump -C file.rw2 | head -3` reveals file structure
- **Our Debug**: `RUST_LOG=debug` shows dimension extraction steps

### ⚠️ **Known Edge Cases**

- **Multiple RW2 Variants**: Some RW2 files may have different internal structures
- **Byte Order**: RW2 appears to always use little-endian, but validate this assumption
- **Error Handling**: Gracefully handle files missing sensor border tags

### 📚 **Reference Links**

- **ExifTool RW2 Tags**: https://exiftool.org/TagNames/PanasonicRaw.html
- **TIFF Specification**: For understanding IFD structure
- **Trust ExifTool Doc**: `/docs/TRUST-EXIFTOOL.md` - fundamental project principle

### 🛠 **Implementation Notes**

- **File Location**: Core logic in `src/raw/mod.rs:283` (TIFF header detection)
- **Integration Point**: RW2 already added to format dispatch in `src/formats/mod.rs:481`
- **Shared Utility**: Reuses existing `extract_tiff_dimensions()` function
- **Code Style**: Follow existing patterns in the module, use comprehensive debug logging

### 📋 **Next Engineer Checklist**

1. [ ] Read TRUST-EXIFTOOL.md and understand the core principle
2. [ ] Study ExifTool's PanasonicRaw.pm lines 675-690 (composite dimensions)
3. [ ] Add RW2 magic bytes `IIU\0` to TIFF header detection
4. [ ] Test with the provided RW2 file to verify 5776×4336 output
5. [ ] Add to compatibility testing system
6. [ ] Verify `make precommit` passes

**Estimated Time**: 2-4 hours (mostly testing and validation)

The implementation is 90% complete - just needs the magic byte fix to unlock the existing sensor border calculation logic.