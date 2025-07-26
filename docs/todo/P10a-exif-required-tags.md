# Technical Project Plan: EXIF Required Tags Implementation

## Project Overview

- **Goal**: Ensure all standard EXIF tags marked as required in PhotoStructure are properly extracted
- **Problem**: Need comprehensive support for 36 EXIF tags that are marked as required in tag-metadata.json

## Background & Context

- EXIF tags are standardized across all camera manufacturers
- These 18 tags form the foundation of image metadata
- Most are already partially implemented but need verification

## Technical Foundation

- **Key files**:
  - `src/exif/tags.rs` - Tag definitions and creation
  - `src/exif/ifd.rs` - IFD parsing
  - `src/value_extraction.rs` - Value extraction from binary data
  - `src/generated/tags/` - Generated tag constants
- **Standards**: EXIF 2.32 specification

## Required EXIF Tags (36 total)

### Core Camera Settings (8 tags)
- **ApertureValue** (0x9202) - APEX aperture value
- **ExposureTime** (0x829A) - Shutter speed in seconds
  - Note: Sony cameras also write SonyExposureTime with potentially higher precision
- **FNumber** (0x829D) - F-stop value
  - Note: Sony cameras also write SonyFNumber with lens corrections
- **FocalLength** (0x920A) - Lens focal length in mm
- **ISO** (0x8827) - ISO sensitivity
  - Note: Sony cameras also write SonyISO with extended range info
- **ISOSpeed** (0x8833) - ISO speed ratings
- **ShutterSpeedValue** (0x9201) - APEX shutter speed
- **MaxApertureValue** (0x9205) - Smallest F number of lens

### Timestamps (7 tags)
- **CreateDate** (0x9004) - When image was created
- **DateTimeOriginal** (0x9003) - When photo was taken
- **DateTimeDigitized** (0x9004) - When digitized
- **ModifyDate** (0x0132) - File modification time
- **SubSecTime** (0x9290) - Subsecond timestamps
- **SubSecTimeDigitized** (0x9292) - Subsecond for digitized
- **DateTime** (0x0132) - File change date/time

### Image Properties (6 tags)
- **ImageWidth** (0x0100) - Image dimensions
- **ImageHeight** (0x0101) - Image dimensions
- **Orientation** (0x0112) - Rotation/flip info
- **ImageDescription** (0x010E) - User description
- **ExifImageWidth** (0xA002) - Valid image width
- **ExifImageHeight** (0xA003) - Valid image height

### Camera/Lens Information (6 tags)
- **Make** (0x010F) - Camera manufacturer
- **Model** (0x0110) - Camera model
- **Software** (0x0131) - Processing software
- **LensInfo** (0xA432) - Lens specification
- **LensMake** (0xA433) - Lens manufacturer
- **LensModel** (0xA434) - Lens model name

### GPS Information (5 tags)
- **GPSLatitude** (0x0002) - Latitude
- **GPSLongitude** (0x0004) - Longitude
- **GPSAltitude** (0x0006) - Altitude
- **GPSLatitudeRef** (0x0001) - North or South
- **GPSLongitudeRef** (0x0003) - East or West

### Other Required Tags (4 tags)
- **Copyright** (0x8298) - Copyright string
- **Artist** (0x013B) - Person who created the image
- **UserComment** (0x9286) - User comments
- **ExifVersion** (0x9000) - EXIF version

## Work Completed

- ✅ Basic IFD parsing infrastructure
- ✅ Value extraction for common types
- ✅ Tag namespace assignment
- ✅ Some tags already extracting (Make, Model, DateTime)
- ✅ **Tag Kit Migration** (2025-07-25)
  - Migrated EXIF and GPS modules from legacy tag_definitions.json to unified tag kit system
  - All tag lookups now use EXIF_PM_TAG_KITS and GPS_PM_TAG_KITS
  - PrintConv definitions embedded in tag kits (implementation pending)
  - ValueConv expressions defined (implementation pending)
- ✅ **GPS Coordinate Processing Restored** (2025-07-25)
  - GPS ToDegrees ValueConv already implemented in `src/implementations/value_conv.rs`
  - Added GPS coordinate PrintConv functions that return decimal degrees directly (no degree symbols)
  - Registered GPS PrintConv functions in implementation registry
  - Updated GPS processing in `tags.rs` to use manual registry for coordinates
  - GPS coordinates now properly convert rational arrays to decimal degrees and output as decimal values
- ✅ **APEX Value Conversions Implemented** (2025-07-25)
  - Added APEX ValueConv processing for ShutterSpeedValue, ApertureValue, and MaxApertureValue
  - Implemented `apex_shutter_speed_value_conv` and `apex_aperture_value_conv` functions
  - Added expression mapping in `tags.rs` for APEX conversions
  - APEX values now properly convert logarithmic values to linear values
- ✅ **Expression PrintConv Evaluation** (2025-07-25)
  - Implemented GPS expression PrintConv for GPSAltitude and GPSHPositioningError
  - Added direct expression evaluation in `tags.rs` for common GPS patterns
  - GPSAltitude now properly handles inf/undef values and adds "m" suffix
  - GPSHPositioningError adds "m" suffix to positioning error values
- ✅ **Additional GPS Tags Implementation** (2025-07-25)
  - Verified GPSSpeed (tag 13), GPSDestBearing (tag 24), and GPSImgDirection (tag 17) working correctly
  - All three tags properly defined in GPS tag kit with `PrintConvType::None` (raw rational values)
  - Rational parsing infrastructure correctly converts `[numerator, denominator]` to decimal values
  - Tags follow Trust ExifTool principle - simple rational64u format with no special processing
  - Implementation matches ExifTool behavior exactly (verified through research)

## Issues Discovered (2025-07-26)

### Critical PrintConv Pipeline Break 🚨

**MAJOR ISSUE**: Core Camera Settings showing raw rational values instead of human-readable formats:
- ExifTool: `"FNumber": 3.9` → exif-oxide: `"EXIF:FNumber": [39, 10]`
- ExifTool: `"FocalLength": "12.2 mm"` → exif-oxide: `"EXIF:FocalLength": [1220, 100]`
- ExifTool: `"ExposureTime": "1/80"` → exif-oxide: `"EXIF:ExposureTime": [1, 80]`

**Root Cause**: The tag kit system correctly extracts PrintConv definitions, but `apply_print_conv()` in `src/generated/Exif_pm/tag_kit/mod.rs:782-796` has TODO placeholders:
```rust
PrintConvType::Expression(expr) => {
    // TODO: Implement expression evaluation
    warnings.push(format!("Expression PrintConv not yet implemented..."));
    value.clone()  // Returns raw value
}
PrintConvType::Manual(func_name) => {
    // TODO: Look up in manual registry  
    warnings.push(format!("Manual PrintConv '{}' not found..."));
    value.clone()  // Returns raw value
}
```

**Research Conducted** (2025-07-26):
- ✅ Deeply studied 8 Core Camera Settings tags using 8 exiftool-researcher agents
- ✅ Examined tag kit definitions - correctly extracted from ExifTool with proper PrintConv types
- ✅ Verified all required manual implementations exist in `src/implementations/print_conv.rs`
- ✅ Verified all APEX value conversions exist in `src/implementations/value_conv.rs`
- ✅ Confirmed issue is in generated code connection, not individual function implementation
- ✅ Identified affected Core Camera Settings tags:
  - FNumber (33437): `PrintConvType::Manual("complex_expression_printconv")`
  - FocalLength (37386): `PrintConvType::Expression("sprintf(\"%.1f mm\",$val)")`
  - ApertureValue (37378): `PrintConvType::Expression("sprintf(\"%.1f\",$val)")` + ValueConv
  - MaxApertureValue (37381): `PrintConvType::Expression("sprintf(\"%.1f\",$val)")` + ValueConv
  - ExposureTime (33434): `PrintConvType::Manual("complex_expression_printconv")` (in datetime.rs)
  - ShutterSpeedValue (37377): `PrintConvType::Manual("complex_expression_printconv")` + ValueConv

**Critical Discovery**: ALL individual components are correctly implemented:
- Tag kit system extracts correct PrintConv definitions from ExifTool ✅
- Manual print conversion functions work correctly when called directly ✅
- APEX value conversion functions work correctly ✅
- The ONLY problem is the generated `apply_print_conv()` function has TODO placeholders ✅

## Current Status (2025-07-26)

**STATUS**: PrintConv pipeline broken - Core Camera Settings outputting raw values instead of human-readable formats

**WRONG APPROACH ATTEMPTED**: Initially tried to manually edit generated files in `src/generated/Exif_pm/tag_kit/mod.rs` - this violates the fundamental rule "DO NOT EDIT THE FILES THAT SAY DO NOT EDIT"

**CORRECT APPROACH IDENTIFIED**: Fix the code generation system to properly generate Expression and Manual PrintConv handling in the tag kit `apply_print_conv()` function

**LESSONS LEARNED**:
1. **Never assume from document status** - Document claimed "100% compatibility" but actual testing revealed critical gaps
2. **Always test actual output** - Real files showed raw [numerator, denominator] instead of formatted values
3. **Trust the modular architecture** - Individual components are working; only the connection layer is broken
4. **Code generation is the solution** - All pieces exist, just need proper generated wiring

## Remaining Tasks

### CRITICAL PRIORITY - Fix Tag Kit Code Generation

**IMMEDIATE TASK**: Fix the tag kit code generation system to properly handle Expression and Manual PrintConv types.

**Files to Study and Modify**:
1. **Tag Kit Generator**: `codegen/src/generators/tag_kit_modular.rs`
   - This generates the `apply_print_conv()` function with TODO placeholders
   - Need to enhance generator to produce working Expression and Manual PrintConv handling
   
2. **Tag Kit Template/Logic**: Look for template or generation logic that produces the `apply_print_conv()` function
   - Currently generates TODO placeholders for Expression and Manual cases
   - Should generate actual implementation that calls existing functions

**Required Implementation Approach**:
1. **Expression PrintConv Generation**:
   - Generate code to handle common sprintf patterns like `sprintf("%.1f mm",$val)`
   - Map `sprintf("%.1f mm",$val)` → `print_conv::focallength_print_conv()`
   - Map `sprintf("%.1f",$val)` → direct formatting for APEX values after ValueConv
   - Pattern matching approach: detect common sprintf formats and map to existing functions

2. **Manual PrintConv Generation**:
   - Generate code to map "complex_expression_printconv" to specific tag implementations  
   - Map FNumber (33437) → `print_conv::fnumber_print_conv()`
   - Map ExposureTime (33434) → `print_conv::exposuretime_print_conv()`
   - Map ShutterSpeedValue (37377) → `print_conv::exposuretime_print_conv()` (after ValueConv)
   - Tag-specific lookup: use tag ID to dispatch to correct function

3. **Integration with Existing Functions**:
   - ALL required PrintConv functions already exist in `src/implementations/print_conv.rs` ✅
   - ALL required ValueConv functions already exist in `src/implementations/value_conv.rs` ✅
   - Code generation just needs to connect tag kit definitions to these implementations

**Specific Implementation Strategy**:
   - Enhance `codegen/src/generators/tag_kit_modular.rs` to generate working `apply_print_conv()` 
   - Generate pattern matching for Expression types (sprintf patterns)
   - Generate tag-specific dispatch for Manual types (by tag ID)
   - Import existing functions from `crate::implementations::{print_conv, value_conv}`

**Alternative Approach** (if code generation is complex):
   - Modify the non-generated code in `src/exif/tags.rs` to override the tag kit's TODO results
   - Add fallback logic for Expression and Manual PrintConv types
   - This would be a temporary workaround until proper code generation is implemented

### Previous Work That Is Already Complete (Don't Redo)

1. **APEX Value Conversions** ✅ DONE
   - ApertureValue → FNumber conversion (APEX: FNumber = 2^(ApertureValue/2)) - implemented
   - ShutterSpeedValue → ExposureTime conversion (APEX: ExposureTime = 2^(-ShutterSpeedValue)) - implemented
   - MaxApertureValue conversion - implemented
   - APEX formulas per EXIF spec section 4.6.5 - working correctly

2. **Rational Value Handling** ✅ DONE
   - RATIONAL/SRATIONAL types properly extracted - working
   - Edge cases handled (0 denominator, overflow) - working
   - FocalLength, ExposureTime, FNumber all extracting correctly

3. **SubSecTime Processing** ✅ BASIC IMPLEMENTATION COMPLETE
   - Extract subsecond precision (ASCII string) - working
   - Basic timestamp handling complete
   - SubSecTimeDigitized working

4. **GPS Coordinate Processing** ✅ DONE
   - Convert GPS rational degrees/minutes/seconds to decimal - working
   - Handle GPSLatitudeRef/GPSLongitudeRef (N/S, E/W) - working
   - Process GPSAltitude with GPSAltitudeRef (above/below sea level) - working

### Medium Priority - Missing Tag Implementation

1. **Lens Information Tags**
   - LensInfo (0xA432) - Min/max focal length and aperture
   - LensMake (0xA433) - ASCII string
   - LensModel (0xA434) - ASCII string

2. **Additional Timestamps**
   - Proper handling of CreateDate vs DateTimeOriginal
   - ModifyDate extraction and formatting

3. **Standard Metadata**
   - Artist (0x013B) - Creator name
   - UserComment (0x9286) - May have character code prefix
   - ExifVersion (0x9000) - Usually "0230"

### Low Priority - String Encoding & Validation

1. **Copyright/Description Strings**
   - Handle various character encodings
   - Strip null terminators
   - Validate UTF-8

2. **GPS Processing Method**
   - GPSProcessingMethod tag support
   - Handle various encoding markers

3. **ISO Value Normalization**
   - Handle both ISO (0x8827) and ISOSpeed (0x8833)
   - Some cameras use ISOSpeedRatings array

## Prerequisites

- ✅ **Tag Kit Migration**: Complete [tag kit migration and retrofit](../done/20250723-tag-kit-migration-and-retrofit.md) for EXIF module
  - This ensures we're using the modern tag extraction system
  - Eliminates potential tag ID/PrintConv mismatches
- Verify RATIONAL type extraction working correctly
- Ensure all standard EXIF IFDs are being processed

## Testing Strategy

- Test with images from multiple manufacturers
- Verify APEX conversions match ExifTool output
- Check edge cases (missing values, invalid data)

## Success Criteria & Quality Gates

### You are NOT done until this is done:

1. **All Required EXIF Tags Extracting**:
   - [ ] 36 EXIF required tags from tag-metadata.json implemented and extracting correctly
   - [ ] Values match ExifTool output format exactly (no [rational] arrays in output)
   - [ ] PrintConv producing human-readable values for exposure settings

2. **Critical PrintConv Implementation** (addresses major compatibility failures):
   ```json
   Priority EXIF tags requiring PrintConv:
   - "EXIF:FNumber"         // Must show "3.9" not [39,10]
   - "EXIF:ExposureTime"    // Must show "1/80" not [1,80]  
   - "EXIF:FocalLength"     // Must show "17.5 mm" not [175,10]
   - "EXIF:Flash"           // Must show "Off, Did not fire" not 16
   - "EXIF:MeteringMode"    // Must show "Multi-segment" not 3
   - "EXIF:ExposureProgram" // Must show "Program AE" not 2
   - "EXIF:ResolutionUnit"  // Must show "inches" not 2
   - "EXIF:YCbCrPositioning"// Must show "Centered" not 1
   ```

3. **GPS Coordinate Processing**:
   - [ ] GPS coordinates converted to decimal degrees (not DMS arrays)
   - [ ] GPSLatitude/GPSLongitude showing signed decimal values
   - [ ] GPSAltitude with "m" suffix for meters

4. **Specific Tag Validation** (must be added to `config/supported_tags.json` and pass `make compat-force`):
   ```bash
   # All these tags must be uncommented/present in supported_tags.json:
   - "EXIF:ApertureValue"
   - "EXIF:ExposureTime" 
   - "EXIF:FNumber"
   - "EXIF:FocalLength"
   - "EXIF:Flash"
   - "EXIF:MeteringMode"
   - "EXIF:ExposureProgram"
   - "EXIF:ResolutionUnit"
   - "EXIF:YCbCrPositioning"
   - "EXIF:GPSLatitude"
   - "EXIF:GPSLongitude"
   - "EXIF:GPSAltitude"
   - "EXIF:GPSLatitudeRef"
   - "EXIF:GPSAltitudeRef"
   - "EXIF:SubSecTime"
   - "EXIF:SubSecTimeDigitized"
   - "EXIF:ImageDescription"
   - "EXIF:Copyright"
   ```

5. **Validation Commands**:
   ```bash
   # After implementing EXIF PrintConv:
   make compat-force                    # Regenerate reference files
   make compat-test | grep "EXIF:"      # Check EXIF tag compatibility
   
   # Target: All core EXIF tags showing formatted values, not raw data
   ```

6. **Manual Validation**:
   - Compare with ExifTool on 5+ camera files (Canon, Nikon, Sony, etc.)
   - Verify exposure settings show as formatted strings (e.g. "1/200", "f/2.8", "50.0 mm")
   - Confirm GPS coordinates show as decimal degrees (e.g. 34.05223, not DMS arrays)
   - Check flash modes show descriptive text ("Off, Did not fire" not numeric codes)

### Quality Gates Definition:
- **BLOCK P12, P13*, P17a until P10a PrintConv complete** - Other TPPs depend on EXIF foundation
- **Compatibility Test Threshold**: <10 EXIF-related failures in `make compat-test`
- **PrintConv Coverage**: All exposure settings (FNumber, ExposureTime, FocalLength) must show formatted strings

## Gotchas & Tribal Knowledge

### PrintConv Pipeline Issue (Current Problem)
- **DO NOT EDIT GENERATED FILES**: `src/generated/` files are regenerated by `make codegen`
- The tag kit system correctly extracts PrintConv definitions from ExifTool ✅
- ALL manual PrintConv functions already exist in `src/implementations/print_conv.rs` ✅
- ALL APEX ValueConv functions already exist in `src/implementations/value_conv.rs` ✅
- The ONLY problem is in the generated `apply_print_conv()` function having TODO placeholders
- **Solution**: Fix the code generator, not the generated code

### Code Generation System Deep Dive
- Tag kit extraction is working correctly - definitions include proper PrintConv types ✅
- Generator is in `codegen/src/generators/tag_kit_modular.rs` 
- The `apply_print_conv()` function is generated with TODO placeholders for Expression and Manual cases
- Current placeholders return `value.clone()` instead of applying conversions
- Need to enhance generator to produce actual implementation that:
  1. Pattern matches Expression sprintf formats to existing functions
  2. Maps Manual tag IDs to specific implementations
  3. Imports required functions from `crate::implementations::{print_conv, value_conv}`

### Trust ExifTool Principle
- Manual porting is BANNED - we've had 100+ bugs from transcription errors
- Always use codegen to extract ExifTool data automatically
- If something seems wrong with our output, first verify against ExifTool behavior
- Every conversion should have a comment pointing back to ExifTool source

### Core Camera Settings Research (Completed)
- **8 exiftool-researcher agents** studied each tag's ExifTool implementation in detail
- **All conversion formulas verified** against ExifTool source code
- **All manual implementations confirmed working** when tested in isolation
- **Problem isolated to code generation layer** - not individual implementations
- **Tag kit definitions confirmed correct** - proper PrintConv types extracted

### Development Anti-Patterns Learned
- **Never assume document status** - "100% compatibility" in docs ≠ actual output compatibility
- **Always test actual output first** - `cargo run -- test.jpg` reveals real issues faster than code review
- **Don't edit generated files** - Violates fundamental project principle, changes get overwritten
- **Use existing modular architecture** - Individual components work, focus on connection layer

### APEX Values (Already Working)
- APEX values use logarithmic scale (base 2)
- FNumber = 2^(ApertureValue/2)
- ExposureTime = 2^(-ShutterSpeedValue)
- Some cameras write both FNumber AND ApertureValue (prefer direct values)

### GPS Coordinates (Already Working)
- Stored as 3 RATIONAL values: degrees, minutes, seconds
- Decimal conversion: degrees + minutes/60 + seconds/3600
- Apply negative for South latitude or West longitude
- GPSAltitude can be negative (below sea level) based on GPSAltitudeRef

### Timestamp Handling (Already Working)
- DateTime format: "YYYY:MM:DD HH:MM:SS"
- SubSecTime is ASCII string, not numeric (e.g., "123" = 0.123 seconds)
- CreateDate (0x9004) is actually DateTimeDigitized in EXIF spec
- Some cameras don't set all timestamp fields

### Tag Location Priority (Already Working)
- Tags can appear in multiple IFDs (IFD0, ExifIFD, GPS IFD)
- Use first found, but GPS tags only valid in GPS IFD
- ImageWidth/Height in IFD0 may differ from ExifImageWidth/Height

### Character Encodings (Already Working)
- UserComment may start with encoding marker (e.g., "ASCII\0\0\0")
- Most strings are ASCII null-terminated
- Some cameras use UTF-8 without proper markers

## Detailed Implementation Guide for Next Engineer

### Key Files to Study and Understand

**Primary Code Generation File**:
- `codegen/src/generators/tag_kit_modular.rs` - Generates the broken `apply_print_conv()` function
  - Contains TODO placeholders that need to be replaced with actual implementation
  - Look for the `generate_apply_print_conv_function()` or similar function
  - Need to enhance to generate working Expression and Manual PrintConv handling

**Working Implementation Files** (DO NOT MODIFY - already correct):
- `src/implementations/print_conv.rs` - ALL required PrintConv functions exist and work
- `src/implementations/value_conv.rs` - ALL required APEX ValueConv functions exist and work
- `src/generated/Exif_pm/tag_kit/exif_specific.rs` - Tag definitions with correct PrintConv types

**Testing and Validation**:
- Use `cargo run --bin compare-with-exiftool test.jpg` to compare output with ExifTool
- Focus on FNumber, FocalLength, ExposureTime showing formatted values vs raw rationals

### Recommended Implementation Steps

1. **Study the Generator**: Examine `tag_kit_modular.rs` to understand current code generation pattern
2. **Identify TODO Location**: Find where Expression and Manual PrintConv cases generate TODO placeholders  
3. **Design Pattern Matching**: Create logic to:
   - Map `sprintf("%.1f mm",$val)` patterns to `focallength_print_conv()`
   - Map tag IDs to specific Manual PrintConv functions
4. **Generate Working Code**: Replace TODO placeholders with actual function calls
5. **Test and Validate**: Verify all 8 Core Camera Settings produce ExifTool-matching output

### Debugging and Validation Tools

```bash
# Test actual output against ExifTool
cargo run --bin compare-with-exiftool test.jpg EXIF:

# Look for PrintConv warnings in output  
cargo run -- test.jpg 2>&1 | grep "PrintConv"

# Regenerate tag kit code after changes
make codegen

# Full validation pipeline
make precommit
```

## Future Refactoring Considerations

**Code Generation System Improvements**:
1. **Template-Based Generation**: Consider using a template system for complex generated functions like `apply_print_conv()`
2. **Expression Evaluation**: Implement a Perl expression evaluator for more complex PrintConv patterns  
3. **Modular PrintConv**: Split large PrintConv functions into smaller, focused modules
4. **Testing Integration**: Generate unit tests for PrintConv functions automatically
5. **Documentation Generation**: Auto-generate documentation for tag kit definitions

**Architecture Improvements**:
1. **Registry Consolidation**: Consider unifying the manual registry and tag kit systems
2. **Error Handling**: Improve error propagation from PrintConv functions
3. **Performance**: Profile PrintConv function dispatch for hot paths
4. **Memory Usage**: Optimize LazyLock usage in generated lookup tables

**Long-term Maintenance**:
1. **ExifTool Updates**: Current approach will automatically handle monthly ExifTool releases
2. **Test Coverage**: Add integration tests for Core Camera Settings with real image files
3. **Performance Monitoring**: Track conversion pipeline performance as tag support grows