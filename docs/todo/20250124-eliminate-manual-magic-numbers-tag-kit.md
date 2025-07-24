# Technical Project Plan: Eliminate Manual Magic Numbers Using Enhanced Tag Kit

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

### Phase 1: Minor Tag Kit Enhancement (High Confidence)

**Task**: Update tag kit extractor to handle AFPointsInFocus case

**Issue**: AFPointsInFocus currently marked as `PrintConvType::Manual` because it has `RawConv => '$val==0 ? undef : $val'`, but its PrintConv is actually a simple hash.

**Implementation**:
1. **Modify `codegen/extractors/tag_kit.pl`**:
   ```perl
   # In extract_print_conv() function  
   # Separate RawConv concerns from PrintConv assessment
   # Extract simple PrintConv hashes even when RawConv exists
   ```

2. **Target Result**: AFPointsInFocus becomes `PrintConvType::Simple` with values `0x3000 => 'None (MF)', 0x3001 => 'Right', etc.`

### Phase 2: Binary Data Function Migration (High Confidence)

**Task**: Replace 6 manual lookup function calls with tag kit calls

**Implementation**:
1. **Update `src/implementations/canon/binary_data.rs`**:
   ```rust
   // Replace these manual calls:
   lookup_shot_info__white_balance(val)           → tag_kit::apply_print_conv(7, val, ...)
   lookup_shot_info__a_f_points_in_focus(val)    → tag_kit::apply_print_conv(10, val, ...)  
   lookup_shot_info__auto_exposure_bracketing(val) → tag_kit::apply_print_conv(16, val, ...)
   lookup_shot_info__camera_type(val)            → tag_kit::apply_print_conv(26, val, ...)
   lookup_panorama__panorama_direction(val)      → tag_kit::apply_print_conv(5, val, ...)
   lookup_my_colors__my_color_mode(val)          → tag_kit::apply_print_conv(2, val, ...)
   ```

2. **Keep Binary Structure**: Preserve all binary data parsing logic - only replace PrintConv calls

### Phase 3: Remove Manual Infrastructure (High Confidence)

**Task**: Clean up manually-ported lookup infrastructure

**Implementation**:
1. **Remove inline imports** in `src/implementations/canon/binary_data.rs`:
   - `use crate::generated::Canon_pm::shotinfo_inline::*;`
   - `use crate::generated::Canon_pm::panorama_inline::*;`
   - `use crate::generated::Canon_pm::mycolors_inline::*;`

2. **Remove codegen configs**: Delete `codegen/config/Canon_pm/inline_printconv.json` (if exists)

3. **Remove generated files**: Delete generated inline lookup files

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
- [ ] AFPointsInFocus extracted as `PrintConvType::Simple` instead of `Manual`
- [ ] Tag kit enhancement doesn't break existing extractions
- [ ] `make codegen` completes successfully

**Phase 2 Complete**:
- [ ] All 6 manual lookup calls replaced with tag kit calls
- [ ] `cargo check` passes without compilation errors
- [ ] Canon binary data tests continue to pass
- [ ] No changes in `compare-with-exiftool` output for Canon test images

**Phase 3 Complete**:
- [ ] Manual inline imports removed 
- [ ] Generated inline lookup files deleted
- [ ] `make precommit` passes completely
- [ ] Zero references to removed lookup functions in codebase

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