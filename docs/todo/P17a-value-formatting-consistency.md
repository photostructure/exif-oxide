# Technical Project Plan: Value Formatting Consistency

## Project Overview

- **Goal**: Ensure all tag values match ExifTool's exact formatting conventions for numeric precision, string formatting, and output consistency
- **Problem**: Minor formatting differences causing compatibility test failures despite correct data extraction

## Background & Context

- ExifTool has specific formatting rules for different value types
- Numeric precision, string formatting, and data type representations must match exactly
- These differences affect user experience and downstream tool compatibility

## Technical Foundation

- **Key files**:
  - `src/value_conv.rs` - Value conversion implementations
  - `src/print_conv.rs` - Output formatting implementations  
  - `src/implementations/*/print_conv.rs` - Manufacturer-specific formatting
  - `src/generated/*/tag_kit/` - Generated PrintConv definitions

## Work Completed

- ✅ Basic value extraction infrastructure
- ✅ Some PrintConv implementations exist
- ⚙️ **PrintConv infrastructure needs systematic completion**

## Remaining Tasks

### High Priority - Core Formatting Issues

**Based on compatibility test failures:**

1. **Numeric Precision Consistency**
   - ExifTool: `"Software": 1.0` → exif-oxide: `"Software": "1.00"`
   - ExifTool: `"ShutterSpeedValue": 0` → exif-oxide: `"ShutterSpeedValue": 0.0`
   - **Fix**: Ensure integer values display as integers, floats match ExifTool precision

2. **Exposure Settings Formatting**
   - ExifTool: `"FNumber": 3.9` → exif-oxide: `"FNumber": [39, 10]` 
   - ExifTool: `"ExposureTime": "1/80"` → exif-oxide: `"ExposureTime": [1, 80]`
   - ExifTool: `"FocalLength": "17.5 mm"` → exif-oxide: `"FocalLength": [175, 10]`
   - **Fix**: Implement proper PrintConv for rational values

3. **Flash Mode Formatting**
   - ExifTool: `"Flash": "Off, Did not fire"` → exif-oxide: `"Flash": 16`
   - **Fix**: Implement flash lookup table and descriptive formatting

4. **Image Size Formatting**
   - ExifTool: `"ImageSize": "8x8"` → exif-oxide: `"ImageSize": "2048 1536"`
   - **Fix**: Ensure proper "WIDTHxHEIGHT" format for ImageSize composite

5. **Composite Aperture/ShutterSpeed**
   - ExifTool: `"Aperture": 3.9` → exif-oxide: `"Aperture": [39, 10]`
   - ExifTool: `"ShutterSpeed": "1/30"` → exif-oxide: `"ShutterSpeed": "1/30"`
   - **Fix**: Format composite tags consistently with EXIF counterparts

### Medium Priority - Manufacturer-Specific Formatting

1. **Canon Value Formatting**
   - Ensure Canon MakerNotes values match ExifTool formatting
   - LensType, CameraID, FileNumber formatting

2. **Nikon Value Formatting**  
   - Encrypted value handling and display
   - LensID calculation and formatting

3. **Sony Value Formatting**
   - SonyISO, SonyFNumber, SonyExposureTime formatting

### Implementation Strategy

**Phase 1: Fix Core EXIF PrintConv**
```rust
// Update src/exif/print_conv.rs
pub fn format_fnumber(rational: &[u32; 2]) -> String {
    let value = rational[0] as f64 / rational[1] as f64;
    if value.fract() == 0.0 {
        format!("{:.0}", value)  // "4" not "4.0"
    } else {
        format!("{:.1}", value)  // "3.9" not "3.90"
    }
}

pub fn format_exposure_time(rational: &[u32; 2]) -> String {
    let numerator = rational[0];
    let denominator = rational[1];
    
    if numerator == 1 {
        format!("1/{}", denominator)  // "1/80"
    } else {
        let value = numerator as f64 / denominator as f64;
        if value >= 0.3 {
            format!("{:.1}", value)   // "0.5" not "1/2"
        } else {
            format!("{}/{}", numerator, denominator)  // "1/200"
        }
    }
}

pub fn format_focal_length(rational: &[u32; 2]) -> String {
    let value = rational[0] as f64 / rational[1] as f64;
    format!("{:.1} mm", value)  // "17.5 mm"
}
```

**Phase 2: Flash Mode Lookup**
```rust
// Extract from ExifTool Flash.pm via codegen
pub fn format_flash_mode(value: u16) -> String {
    match value {
        0 => "No Flash",
        1 => "Fired",
        5 => "Fired, Return not detected",
        7 => "Fired, Return detected",
        8 => "On, Did not fire",
        16 => "Off, Did not fire",
        // ... complete lookup table from codegen
        _ => format!("Unknown ({})", value),
    }
}
```

**Phase 3: Composite Tag Formatting**
```rust
// Update src/composite_tags/implementations.rs
pub fn format_image_size(width: u32, height: u32) -> String {
    format!("{}x{}", width, height)  // "2048x1536" not "2048 1536"
}

pub fn format_megapixels(width: u32, height: u32) -> String {
    let mp = (width as f64 * height as f64) / 1_000_000.0;
    format!("{:.1}", mp)  // "3.1" not "3.145728"
}
```

## Prerequisites

- **P10a: EXIF Required Tags** - Core EXIF PrintConv infrastructure must exist
- **P12: Composite Required Tags** - Composite calculation framework needed
- **P20: Codegen Migration** - Flash lookup and other tables should be generated

## Testing Strategy

- **Compatibility Test Focus**: Run `make compat-force` and target formatting mismatches
- **Regression Testing**: Ensure formatting fixes don't break existing functionality
- **Cross-Manufacturer Testing**: Verify formatting consistency across Canon, Nikon, Sony files
- **Edge Case Testing**: Test with unusual values (zero denominators, very large/small numbers)

## Success Criteria & Quality Gates

### You are NOT done until this is done:

1. **Compatibility Test Pass Rate**:
   - [ ] Reduce formatting-related compatibility failures by >80%
   - [ ] No regressions in existing successful tag extractions

2. **Specific Tag Validation** (must be added to `config/supported_tags.json` and pass `make compat-force`):
   ```json
   Core EXIF formatting tags:
   - "EXIF:FNumber"          // Must show "3.9" not [39,10]
   - "EXIF:ExposureTime"     // Must show "1/80" not [1,80] 
   - "EXIF:FocalLength"      // Must show "17.5 mm" not [175,10]
   - "EXIF:Flash"            // Must show "Off, Did not fire" not 16
   - "EXIF:Software"         // Must show "1.0" not "1.00"
   
   Composite formatting tags:
   - "Composite:Aperture"    // Must show "3.9" not [39,10]
   - "Composite:ShutterSpeed"// Must show "1/30" not raw value
   - "Composite:ImageSize"   // Must show "2048x1536" not "2048 1536"
   - "Composite:Megapixels"  // Must show "3.1" not "3.145728"
   ```

3. **Validation Commands**:
   ```bash
   # After implementing fixes:
   make compat-force              # Regenerate reference files
   make compat-test | grep -c "❌"  # Count remaining failures
   
   # Target: <20 formatting-related failures remaining
   ```

4. **Manual Validation**:
   - Compare exif-oxide output with ExifTool for 10 representative files
   - Verify rational values display as formatted strings, not arrays
   - Confirm flash modes show descriptive text, not numeric codes

## Gotchas & Tribal Knowledge

### Formatting Rules from ExifTool

1. **Rational Formatting**:
   - Display as decimal when denominator results in simple decimal
   - Show as fraction for shutter speeds <0.3 seconds
   - Always include units for measurements (mm, seconds, etc.)

2. **Integer vs Float Display**:
   - Integer values should not show decimal points
   - Float values should use minimal precision (3.9 not 3.90)

3. **Composite Tag Consistency**:
   - Composite tags should format identically to their EXIF counterparts
   - ImageSize always uses "x" separator, never space

4. **Flash Mode Special Cases**:
   - Flash value is a bitmask with multiple flags
   - Must decode all flags to create descriptive text
   - "Off, Did not fire" is different from "No Flash"

### Error Patterns to Avoid

1. **Over-Precision**: Don't show more decimal places than ExifTool
2. **Under-Precision**: Don't round values ExifTool shows precisely  
3. **Unit Inconsistency**: Always include units where ExifTool does
4. **Format Mixing**: Don't mix rational arrays with formatted strings in output

### Dependencies

- Flash lookup tables must be generated via codegen (P20)
- EXIF PrintConv infrastructure must exist (P10a)
- Composite tag calculations must be working (P12)

---

**⚠️ Implementation Note**: This TPP focuses on OUTPUT FORMATTING only. The underlying data extraction should already be working - we're just fixing how values are presented to match ExifTool exactly.