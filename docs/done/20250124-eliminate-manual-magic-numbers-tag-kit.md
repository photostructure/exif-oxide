# Technical Project Plan: Eliminate Manual Magic Numbers Using Enhanced Tag Kit

## ✅ PROJECT COMPLETE - July 24, 2025

All phases of this project have been successfully completed. The Canon binary data extraction functions now use the automated tag kit system instead of manually-ported lookup tables, eliminating the source of 100+ bugs and ensuring compliance with the Trust ExifTool principle.

## Project Overview

**High-level Goal**: Replace manually-ported lookup tables in Canon binary data extraction with automated tag kit extraction to eliminate the source of 100+ bugs caused by imprecise porting.

**Problem Statement**: Canon binary data extraction functions use manually-ported magic number lookup tables (e.g., `lookup_shot_info__white_balance()`), which violate the Trust ExifTool principle and have caused over 100 bugs due to translation errors, missing values, and maintenance drift from ExifTool releases.

## Background & Context

### Why This Work is Needed

- **100+ Bug History**: Manual porting of ExifTool lookup tables has caused significant bugs due to imprecise translation
- **Trust ExifTool Violation**: Manual magic number lists directly contradict [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md)
- **Maintenance Burden**: Monthly ExifTool releases require manual syncing of lookup tables
- **Error-Prone Process**: Manual translation introduces subtle bugs that are hard to detect and debug

### Related Documentation

- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - Core project principle against manual porting
- [CODEGEN.md](../CODEGEN.md) - Code generation system overview  
- [EXTRACTOR-GUIDE.md](../reference/EXTRACTOR-GUIDE.md) - Tag kit as primary extractor
- [20250122-tag-kit-migration-and-retrofit.md](20250122-tag-kit-migration-and-retrofit.md) - Previous tag kit migration work

## Technical Foundation

### Key Codebases

1. **Canon Binary Data Functions** (`src/implementations/canon/binary_data.rs`):
   - `extract_shot_info()` - Parses Canon ShotInfo binary block
   - `extract_panorama()` - Parses Canon Panorama binary block  
   - `extract_my_colors()` - Parses Canon MyColors binary block

2. **Tag Kit System** (`src/generated/Canon_pm/tag_kit/`):
   - Already configured to extract Canon ShotInfo, Panorama, MyColors tables
   - Contains automatically extracted PrintConv data as `PrintConvType::Simple`

3. **Tag Kit Extractor** (`codegen/extractors/tag_kit.pl`):
   - Extracts complete tag definitions with PrintConv implementations
   - Currently conservative: marks tags with RawConv as "Manual" even if PrintConv is simple

### APIs

- **Current Manual API**: `lookup_function_name(value) -> Option<&'static str>`
- **Target Tag Kit API**: `tag_kit::apply_print_conv(tag_id, value, evaluator, errors, warnings) -> TagValue`

### ExifTool Source References

- **ShotInfo**: `third-party/exiftool/lib/Image/ExifTool/Canon.pm:2717` (`%Image::ExifTool::Canon::ShotInfo`)
- **Panorama**: `third-party/exiftool/lib/Image/ExifTool/Canon.pm:2999` (`%Image::ExifTool::Canon::Panorama`) 
- **MyColors**: `third-party/exiftool/lib/Image/ExifTool/Canon.pm:3132` (`%Image::ExifTool::Canon::MyColors`)

## Work Completed

### Tag Kit System Infrastructure (100% Complete)

✅ **Tag Kit Extraction Already Working**: Canon ShotInfo, Panorama, MyColors tables are already configured and extracted by tag kit system in `codegen/config/Canon_pm/tag_kit.json`

✅ **PrintConv Data Generated**: Tag kit has already extracted the lookup data:
- **CameraType** (tag ID 26): `PrintConvType::Simple` with values `0 => "n/a", 248 => "EOS High-end", 250 => "Compact", 252 => "EOS Mid-range", 255 => "DV Camera"`
- **PanoramaDirection** (tag ID 5): `PrintConvType::Simple` with values `0 => "Left to Right", 1 => "Right to Left", etc.`
- **AutoExposureBracketing** (tag ID 16): `PrintConvType::Simple` with inline hash values
- **MyColorMode** (tag ID 2): `PrintConvType::Simple` with inline hash values  
- **WhiteBalance** (tag IDs 7, 8): `PrintConvType::Simple` with canonWhiteBalance reference

✅ **Binary Data Structure Preserved**: Binary data extraction functions correctly handle Canon's proprietary packed data formats following ExifTool's approach

### Analysis Completed

✅ **Validation Complete**: Confirmed tag kit is the correct tool for this job - handles 5/6 cases perfectly
✅ **Root Cause Identified**: Manual lookup functions are exactly the "lists of magic numbers" that TRUST-EXIFTOOL.md warns against

## Remaining Tasks

### Phase 1: Minor Tag Kit Enhancement (COMPLETED)

**Task**: Update tag kit extractor to handle AFPointsInFocus case

**Status**: COMPLETED - The AFPointsInFocus tag (ID 14 in CameraSettings, not ID 10 in ShotInfo) is already extracted as `PrintConvType::Simple(&PRINT_CONV_50)` with the expected lookup values.

**Note**: The document incorrectly stated AFPointsInFocus was tag ID 10 in ShotInfo. In ExifTool, AFPointsInFocus is:
- Tag ID 14 in CameraSettings table
- Tag ID 14 in ShotInfo table (different table, same ID)
- Tag ID 12 in AFInfo2 table

The tag kit system correctly extracts and handles all these variants.

### Phase 2: Binary Data Function Migration (COMPLETED)

**Task**: Replace 6 manual lookup function calls with tag kit calls

**Status**: COMPLETED - All manual lookup functions have been replaced with tag kit calls in `src/implementations/canon/binary_data.rs`:

1. **Updated implementations**:
   - WhiteBalance: Using `tag_kit::apply_print_conv(7, val, ...)` ✓
   - AFPointsInFocus: Using `tag_kit::apply_print_conv(14, val, ...)` ✓ (Note: corrected from ID 10 to ID 14)
   - AutoExposureBracketing: Using `tag_kit::apply_print_conv(16, val, ...)` ✓
   - CameraType: Using `tag_kit::apply_print_conv(26, val, ...)` ✓
   - PanoramaDirection: Using `tag_kit::apply_print_conv(5, val, ...)` ✓
   - MyColorMode: Using `tag_kit::apply_print_conv(2, val, ...)` ✓

2. **Binary Structure Preserved**: All binary data parsing logic remains intact - only PrintConv calls were replaced

### Phase 3: Remove Manual Infrastructure (COMPLETED)

**Task**: Clean up manually-ported lookup infrastructure

**Status**: COMPLETED - All manual lookup infrastructure has been removed:

1. **Inline imports removed**: No inline imports exist in `src/implementations/canon/binary_data.rs` ✓
2. **Codegen configs removed**: No `inline_printconv.json` exists in `codegen/config/Canon_pm/` ✓
3. **Generated files removed**: No inline lookup files exist in `src/generated/Canon_pm/` ✓
4. **Manual lookup functions removed**: All 6 lookup functions no longer exist in the codebase ✓

## Prerequisites

- Understanding of Rust module system and tag kit API patterns
- Familiarity with ExifTool's PrintConv concept  
- Read [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) before making changes

## Testing Strategy

### Unit Tests
- Existing tag kit tests provide coverage: `tests/tag_kit_integration.rs`
- Canon binary data function tests continue to pass with new implementation

### Integration Tests  
```bash
# Validate tag kit extraction works after enhancement
make codegen && cargo test tag_kit

# Ensure no output regressions with real images
cargo run --bin compare-with-exiftool test-images/canon.cr2
```

### Manual Testing
1. Run `make codegen` after tag kit enhancement - should complete without errors
2. Run `cargo check` after migration - should compile cleanly  
3. Test Canon files with changed PrintConv implementations show no differences vs ExifTool

## Success Criteria & Quality Gates

**Phase 1 Complete**:
- [x] AFPointsInFocus extracted as `PrintConvType::Simple` instead of `Manual` ✓
- [x] Tag kit enhancement doesn't break existing extractions ✓
- [x] `make codegen` completes successfully ✓

**Phase 2 Complete**:
- [x] All 6 manual lookup calls replaced with tag kit calls ✓
- [x] `cargo check` passes without compilation errors ✓
- [x] Canon binary data tests continue to pass ✓
- [x] No changes in output for Canon test images ✓

**Phase 3 Complete**:
- [x] Manual inline imports removed ✓
- [x] Generated inline lookup files deleted ✓
- [x] Zero references to removed lookup functions in codebase ✓

## Gotchas & Tribal Knowledge

### Tag Kit Function Signatures

Tag kit uses a different API pattern than manual lookups:
```rust
// Manual lookup (old)
if let Some(description) = lookup_shot_info__white_balance(val) {
    TagValue::string(description)
}

// Tag kit (new)  
use crate::expressions::ExpressionEvaluator;
let mut evaluator = ExpressionEvaluator::new();
let mut errors = Vec::new();
let mut warnings = Vec::new();
tag_kit::apply_print_conv(7, &TagValue::U8(val), &mut evaluator, &mut errors, &mut warnings)
```

### Binary Data vs Individual Tags

**Important Distinction**: These are Canon BinaryData tables (ShotInfo, Panorama, MyColors) with `FORMAT` and `FIRST_ENTRY`, not individual EXIF tags. The binary data extraction functions correctly handle Canon's proprietary binary parsing - we're only replacing the PrintConv lookup portion.

### ExifTool Fidelity

The binary data extraction approach mirrors ExifTool's architecture:
- ExifTool defines these as BinaryData tables with offset-based field extraction
- Our functions parse the same binary structures and apply PrintConv to individual fields
- This maintains faithful ExifTool translation while using automated PrintConv data

### RawConv vs PrintConv Separation

The tag kit enhancement separates two concerns:
- **RawConv**: Preprocessing logic (e.g., `$val==0 ? undef : $val`)  
- **PrintConv**: Display formatting (e.g., `0x3000 => 'None (MF)'`)

Many tags have complex RawConv but simple PrintConv - the enhancement extracts the simple PrintConv data regardless of RawConv complexity.

### Monthly ExifTool Updates

After this migration, Canon PrintConv data automatically updates with ExifTool releases:
1. Submodule update: `cd third-party/exiftool && git checkout v12.XX`
2. Regenerate: `make codegen`  
3. Zero manual intervention required

This eliminates the maintenance burden and translation errors that caused the original 100+ bugs.