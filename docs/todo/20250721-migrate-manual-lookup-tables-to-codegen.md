# Technical Project Plan: Migrate Manual Lookup Tables to Codegen

**Created**: 2025-07-21  
**Status**: Ready for implementation  
**Priority**: High (maintenance burden, violates TRUST-EXIFTOOL.md principles)

## Project Overview

**Goal**: Systematically migrate all manually maintained lookup tables to use the automated codegen system, eliminating maintenance burden and ensuring ExifTool compatibility.

**Problem**: Comprehensive analysis reveals 200+ manual lookup entries across multiple files that should be automatically generated from ExifTool source code, creating substantial maintenance debt with monthly ExifTool releases.

## Background & Context

### Why This Work is Needed

- **Violates CODEGEN.md**: Manual lookup tables drift from ExifTool source with monthly releases
- **High Maintenance Burden**: 1,500+ lines of manually maintained lookup code requiring constant updates
- **Translation Errors**: Manual Perl→Rust translation prone to mistakes that affect real-world parsing
- **Scale**: 15+ manual lookup functions with 5-50 entries each across multiple manufacturers
- **Explicit TODO**: Code comment in `src/implementations/nikon/tags/print_conv/basic.rs:46` specifically calls out this issue

### Related Documentation

- [CODEGEN.md](../CODEGEN.md) - Code generation framework and simple table extraction
- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - Must translate ExifTool exactly, not maintain manual copies
- [20250721-enhance-tag-structure-generator-for-subdirectories.md](20250721-enhance-tag-structure-generator-for-subdirectories.md) - Related Olympus work

## Technical Foundation

### Key Systems

- **Codegen Framework**: `codegen/src/generators/lookup_tables/standard.rs` - Simple table generator
- **Configuration**: `codegen/config/*/simple_table.json` - Per-module extraction configs
- **ExifTool Source**: `third-party/exiftool/lib/Image/ExifTool/*.pm` - Perl hash definitions
- **Generated Output**: `src/generated/*/` - Auto-generated lookup functions

### Current Architecture

```
Manual Lookup Tables (❌ Problem)
    ↓ 
src/implementations/nikon/tags/print_conv/*.rs
    ↓
15+ HashMap::new() manual initializations
    ↓
Monthly manual updates required

Target Architecture (✅ Solution)  
    ↓
ExifTool Nikon.pm %hash definitions
    ↓ (extractor + config)
Generated lookup functions in src/generated/Nikon_pm/
    ↓ (import + call)
PrintConv functions use generated tables
```

## Work Completed

### Comprehensive Audit ✅

**Files Analyzed**: All source files in `src/implementations/` and `src/generated/`

**Key Finding**: **No files falsely claim to be generated** - all files in `src/generated/` are legitimately auto-generated with proper headers.

**Legitimate Generated Files**: 104 files properly marked "Auto-generated" and "DO NOT EDIT MANUALLY"

### Problem Scope Identified ✅

**Primary Issues Found**:
1. **Nikon PrintConv Manual Tables** (Critical) - 15+ functions, 200+ entries
2. **EXIF PrintConv Hardcoded Logic** (Medium) - 7 functions in `print_conv.rs`
3. **Olympus Equipment Tags** (Medium) - Already documented in separate TPP

**No Issues Found**:
- MinoltaRaw and PanasonicRaw implementations are correctly using generated tables
- Other manufacturer implementations appear to follow proper patterns

## Remaining Tasks

### High Confidence Implementation Tasks

#### 1. Nikon PrintConv Migration (Critical Priority)

**Files to Migrate**:
- `src/implementations/nikon/tags/print_conv/basic.rs` (580 lines, 15+ functions)
- `src/implementations/nikon/tags/print_conv/advanced.rs` (~400 lines estimated)
- `src/implementations/nikon/tags/print_conv/af.rs` (~300 lines estimated)

**Manual Functions to Convert**:
```rust
// All these contain manual HashMap::new() initializations
- nikon_quality_conv() - 12 entries (VGA Basic, VGA Normal, etc.)
- nikon_white_balance_conv() - 10 entries (Auto, Preset, etc.)  
- nikon_iso_conv() - 32 entries (ISO mappings)
- nikon_af_area_mode_conv() - 30+ entries
- nikon_active_d_lighting_conv() - 8 entries
- nikon_image_optimization_conv() - 6 entries
- nikon_focus_mode_conv() - 8 entries
- nikon_metering_mode_conv() - 10 entries
- Plus 7+ additional manual functions
```

**Step-by-Step Implementation**:

1. **Find ExifTool Source Hashes**:
   ```bash
   # Search third-party/exiftool/lib/Image/ExifTool/Nikon.pm for:
   grep -n "nikonQuality\|nikonWhiteBalance\|nikonISO" third-party/exiftool/lib/Image/ExifTool/Nikon.pm
   ```

2. **Add to Codegen Config**: `codegen/config/Nikon_pm/simple_table.json`
   ```json
   {
     "tables": [
       {"hash_name": "%nikonQuality", "constant_name": "NIKON_QUALITY", "key_type": "u8"},
       {"hash_name": "%nikonWhiteBalance", "constant_name": "NIKON_WHITE_BALANCE", "key_type": "u8"},
       // ... add all 15+ manual lookup tables
     ]
   }
   ```

3. **Generate Code**: `make codegen`

4. **Update PrintConv Functions**:
   ```rust
   // Replace manual HashMap initialization
   use crate::generated::Nikon_pm::lookup_nikon_quality;
   
   pub fn nikon_quality_conv(value: u8) -> Option<&'static str> {
       lookup_nikon_quality(value)
   }
   ```

#### 2. EXIF PrintConv Hardcoded Tables (Medium Priority)

**File**: `src/implementations/print_conv.rs`

**Functions with Hardcoded match statements**:
- `resolutionunit_print_conv()` - 3 values
- `ycbcrpositioning_print_conv()` - 2 values  
- `flash_print_conv()` - ~25 values
- `colorspace_print_conv()` - ~7 values
- `whitebalance_print_conv()` - 2 values
- `meteringmode_print_conv()` - ~8 values
- `exposureprogram_print_conv()` - ~10 values

**Implementation**: Add to `codegen/config/Exif_pm/simple_table.json` and convert to generated lookups

#### 3. Nikon Tag Table Structures (Research Required)

**File**: `src/implementations/nikon/tags/tables.rs` (248 lines)

**Research Needed**: Determine if static tag table definitions can be generated from ExifTool Nikon.pm tag table structures, or if manual maintenance is appropriate for runtime performance.

### Medium Confidence Tasks (Need Research)

#### 4. Other Manufacturer Audit

**Research scope**: Canon, Sony, Olympus implementations for similar manual lookup patterns
**Estimated effort**: 4-8 hours analysis across all manufacturer modules
**Priority**: After Nikon migration complete

## Prerequisites

### Before Starting

- **Codegen Environment**: `make codegen` must run successfully
- **ExifTool Submodule**: Must be at correct commit for extraction  
- **Nikon Test Images**: Need sample NEF files for validation testing

### No Blocking Dependencies

This work can proceed independently - codegen framework is fully functional for simple table extraction.

## Testing Strategy

### Unit Tests

- Test each converted lookup function with known value mappings
- Validate unknown value handling (should return None or "Unknown")
- Compare generated vs manual lookup results on identical inputs

### Integration Tests

```bash
# 1. Generate new lookup tables
make codegen

# 2. Verify generated functions exist
ls src/generated/Nikon_pm/nikonquality.rs src/generated/Nikon_pm/nikonwhitebalance.rs

# 3. Test Nikon NEF processing
cargo run -- test-images/nikon/test.nef | grep -E "Quality|WhiteBalance|ISO"

# 4. Compare with ExifTool output
cargo run --bin compare-with-exiftool test-images/nikon/test.nef
```

### ExifTool Compatibility Validation

```bash
# Before and after comparison for regression testing
exiftool -j -struct -G test.nef > before.json
# ... perform migration ...  
cargo run -- test.nef > after.json
# Compare tag values are identical
```

## Success Criteria & Quality Gates

### Definition of Done

- [ ] All 15+ Nikon manual lookup functions converted to use generated tables
- [ ] All 7 EXIF hardcoded lookup functions converted to use generated tables  
- [ ] Manual HashMap initializations removed from PrintConv files
- [ ] `make precommit` passes with no regressions
- [ ] ExifTool compatibility maintained for all converted functions
- [ ] Generated lookup tables match ExifTool source exactly

### Quality Gates

- **Code Review**: Ensure generated tables match ExifTool Perl source exactly
- **Performance**: Generated LazyLock HashMap lookups should be O(1) like manual versions
- **Documentation**: Update function comments to reference generated tables
- **Testing**: All existing tests pass + new tests for edge cases

## Gotchas & Tribal Knowledge

### Technical Constraints

- **Trust ExifTool**: Generated lookups must match ExifTool output exactly, not "improved" versions
- **Key Types**: Ensure codegen key types (u8/u16/i32) match manual function parameter types
- **Value Format**: Some manual functions do custom string formatting - preserve in PrintConv wrapper

### ExifTool Source Patterns

- **Hash Definitions**: Look for `%hashName = (` patterns in ExifTool .pm files
- **Scoping**: Some hashes are `my %hash` (private) - codegen will need patching to access
- **Complex Entries**: Simple table extraction only handles primitive key→string mappings

### Implementation Details

- **Incremental Migration**: Convert one manual function at a time for safer testing
- **Preserve Function Signatures**: Keep existing PrintConv function signatures for compatibility
- **Import Patterns**: Use `use crate::generated::Module_pm::lookup_*` for clean imports

### Performance Considerations

- **LazyLock Pattern**: Generated tables use `std::sync::LazyLock<HashMap>` for O(1) lookup
- **Memory Usage**: Generated tables share string literals, no duplication
- **Compile Time**: Large generated files may slow compilation - modular output helps

### Avoiding Pitfalls

- **Don't Break Working Code**: MinoltaRaw/PanasonicRaw already use proper codegen - leave unchanged
- **Verify All Entries**: Manual translations often have subtle errors - validate against ExifTool
- **Handle Edge Cases**: Ensure "Unknown" fallback behavior matches manual implementations
- **Test Multiple Models**: Nikon has model-specific behaviors - test across camera models

### Future Maintenance

- **Monthly ExifTool Updates**: Generated tables automatically stay in sync
- **New Manufacturers**: Use this migration as template for future manufacturer modules  
- **Documentation Updates**: Update CODEGEN.md with lessons learned from this migration

## Implementation Notes

### Priority Order

1. **Nikon PrintConv Manual Functions** - Highest impact, explicit TODO in code
2. **EXIF Hardcoded PrintConv** - Medium impact, affects all manufacturers  
3. **Nikon Tag Tables** - Research required first
4. **Other Manufacturer Audit** - As-needed basis

### Risk Mitigation

- **Backup Plan**: Keep manual implementations commented out until validation complete
- **Rollback Strategy**: Generated table configs can be easily reverted if issues found
- **Testing First**: Validate small subset (2-3 functions) before migrating all

### Success Metrics

- **Lines of Code Reduced**: Target >1,000 lines of manual lookup code eliminated
- **Maintenance Effort**: Zero manual updates needed for ExifTool monthly releases
- **Accuracy**: 100% compatibility with ExifTool reference output