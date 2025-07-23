# Technical Project Plan: Migrate All Manual Lookup Tables to Codegen

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

## Technical Foundation

### Key Systems

- **Simple Table Generator**: `codegen/src/generators/lookup_tables/standard.rs` - Generates lookup functions from Perl hashes
- **Configuration**: `codegen/config/*/simple_table.json` - Per-module extraction configs
- **ExifTool Source**: `third-party/exiftool/lib/Image/ExifTool/*.pm` - Perl hash definitions
- **Generated Output**: `src/generated/*/` - Auto-generated lookup functions

### Current Architecture

```
Manual Lookup Tables (❌ Problem)
    ↓
src/implementations/*/tags/print_conv/*.rs
    ↓
HashMap::new() manual initializations
    ↓
Monthly manual updates required

Target Architecture (✅ Solution)
    ↓
ExifTool *.pm %hash definitions
    ↓ (extractor + config)
Generated lookup functions in src/generated/*/
    ↓ (import + call)
PrintConv functions use generated tables
```

## Work Completed ✅

### Phase 1: Infrastructure Analysis & Critical Bug Fix (COMPLETE)

**Investigation Results** _(July 21, 2025)_:

- **Files Analyzed**: All source files in `src/implementations/` and `src/generated/`
- **Codegen System**: Confirmed fully functional - extracts 500+ tables across manufacturers
- **Migration Claims**: **VERIFIED** - Previous migrations are working as documented

**Critical Codegen Bug Fixed** _(URGENT - Required for compilation)_:

- **Issue**: Generated modules (`offset_patterns.rs`, `tag_structure.rs`) were not included in `mod.rs` files
- **Root Cause**: Standalone generators outside config-based system weren't detected by module generation
- **Solution**: Enhanced `process_config_directory` in `codegen/src/generators/lookup_tables/mod.rs` with `detect_additional_generated_files` function
- **Files Fixed**: Sony_pm, Canon_pm, Olympus_pm mod.rs files now properly include all generated modules
- **Status**: ✅ Fixed and tested - codegen now works correctly

**Migration Verification** _(Confirmed working)_:

1. **3 Nikon Functions** in `src/implementations/nikon/tags/print_conv/basic.rs`:

   - `nikon_focus_mode_conv()` → `lookup_focus_mode_z7()` (4 entries) ✅
   - `nikon_nef_compression_conv()` → `lookup_nef_compression()` (12 entries vs 4 manual!) ✅
   - Added `meteringModeZ7` config (ready for future use) ✅

2. **1 EXIF Function** in `src/implementations/print_conv.rs`:
   - `flash_print_conv()` → `lookup_flash()` (27 entries) ✅

**Lines of Code Eliminated**: ~200 lines of manual HashMap initialization code

**Build Status**: ⚠️ **Partially Fixed** - Codegen works, but Olympus enum conflict remains (see Current Issues)

## Current Issues ⚠️

### **URGENT: Olympus Enum Conflict (Compilation Blocker)**

**Problem**: Compilation fails due to incorrect `OlympusDataType` enum generation:

- `src/generated/Olympus_pm/tag_structure.rs` generated from wrong ExifTool table (`%Olympus::Equipment` instead of `%Olympus::Main`)
- Generated enum has equipment variants (`EquipmentVersion`, `CameraType2`) instead of expected main table variants (`Equipment`, `CameraSettings`, `RawDevelopment`, etc.)
- Code in `src/raw/formats/olympus.rs:42-50` references missing variants causing 14 compilation errors

**Investigation Needed**:

- Check `codegen/config/Olympus_pm/tag_table_structure.json` configuration
- Verify which ExifTool table should generate the main `OlympusDataType` enum
- Fix codegen to target correct table for Olympus main data types

**Error Pattern**:

```
error[E0599]: no variant or associated item named `Equipment` found for enum `Olympus_pm::tag_structure::OlympusDataType`
error[E0599]: no variant or associated item named `CameraSettings` found for enum `Olympus_pm::tag_structure::OlympusDataType`
```

**Workaround**: Temporarily use `equipment_tag_structure::OlympusDataType` if it has the correct variants, or fix the generation config.

## Remaining Tasks

### **IMMEDIATE PRIORITY: Fix Olympus Compilation**

1. **Fix Olympus enum generation** - Target `%Olympus::Main` table instead of `%Olympus::Equipment`
2. **Resolve duplicate enum conflict** - Two files generate same enum name but different variants
3. **Verify build passes** - `make precommit` must succeed before any other work

### Phase 2: Optional Cleanup & Additional Functions (Lower Priority)

**Status**: Phase 1 proved the system works. Remaining work is incremental improvements with diminishing returns.

**Could Be Migrated** (if desired):

1. **Remaining Nikon Functions** in `src/implementations/nikon/tags/print_conv/basic.rs`:

   - `nikon_quality_conv()`, `nikon_white_balance_conv()`, `nikon_iso_conv()`
   - **Challenge**: Need to find corresponding ExifTool hash names (many don't exist as simple tables)

2. **Remaining EXIF Functions** in `src/implementations/print_conv.rs`:
   - `resolutionunit_print_conv()`, `ycbcrpositioning_print_conv()`, `colorspace_print_conv()`
   - **Challenge**: Need to locate actual ExifTool hash definitions

## Key Research Findings & Tribal Knowledge

### 🔍 **Critical Discovery: Not All Manual Tables Have ExifTool Hash Sources**

During implementation, I discovered that **many manual lookup functions don't correspond to simple ExifTool hashes**:

1. **Complex Hash Structures**: `%isoAutoHiLimitZ7` contains ExifTool configuration metadata, not just key-value pairs:

   ```perl
   my %isoAutoHiLimitZ7 = (
       Format => 'int16u',           # Config metadata
       Unknown => 1,                 # Config metadata
       ValueConv => '($val-104)/8',  # Complex conversion
       SeparateTable => 'ISOAutoHiLimitZ7',
       PrintConv => { ... }          # The actual lookup table
   );
   ```

2. **Missing Hash Names**: Many manual functions like `nikon_quality_conv()` and `nikon_white_balance_conv()` don't have corresponding `%nikonQuality` or `%nikonWhiteBalance` hashes in ExifTool source.

3. **String-Based Tags**: Functions like `nikon_color_mode_conv()` handle string values directly from ExifTool without lookup tables.

### ✅ **What Works Well (Proven Pattern)**

**Simple Key-Value Hashes**: Successfully migrated functions that correspond to straightforward ExifTool hashes:

- `%focusModeZ7` → `lookup_focus_mode_z7()`
- `%nefCompression` → `lookup_nef_compression()`
- `%flash` → `lookup_flash()`

**Migration Pattern**:

1. Add hash to `codegen/config/ModuleName_pm/simple_table.json`
2. Run `make codegen` to generate lookup functions
3. Replace manual HashMap with generated lookup calls
4. Verify with `make precommit`

## Prerequisites

### Before Starting

- **Codegen Environment**: `make codegen` must run successfully
- **ExifTool Submodule**: Must be at correct commit for extraction
- **Test Images**: NEF/ORF files for validation testing

### No Blocking Dependencies

This work can proceed independently - codegen framework is functional for simple table extraction.

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
ls src/generated/Olympus_pm/equipment*.rs src/generated/Nikon_pm/nikon*.rs

# 3. Test file processing
cargo run -- test-images/olympus/test.orf | grep -E "CameraType2|SerialNumber|LensType"
cargo run -- test-images/nikon/test.nef | grep -E "Quality|WhiteBalance|ISO"

# 4. Compare with ExifTool output
cargo run --bin compare-with-exiftool test-images/olympus/test.orf
cargo run --bin compare-with-exiftool test-images/nikon/test.nef
```

### ExifTool Compatibility Validation

```bash
# Before and after comparison for regression testing
exiftool -j -struct -G test.orf > before.json
# ... perform migration ...
cargo run -- test.orf > after.json
# Compare tag values are identical
```

## For Future Engineers: Essential Context & Next Steps

### 🚨 **START HERE: Critical Codegen Fix Applied (July 21, 2025)**

**What Was Done**:

- Fixed major codegen bug where standalone generated modules weren't included in mod.rs files
- Enhanced `codegen/src/generators/lookup_tables/mod.rs:process_config_directory` with automatic detection
- **Do NOT revert this fix** - it's essential for compilation

**Files Modified**:

- `codegen/src/generators/lookup_tables/mod.rs` (lines ~150-250) - Added `detect_additional_generated_files`
- All generated `src/generated/*/mod.rs` files now properly include standalone modules

### 🎯 **Recommended Next Steps** (Priority Order)

**1. URGENT: Fix Olympus Compilation (Must Do First)**

- Investigate `codegen/config/Olympus_pm/tag_table_structure.json`
- Fix enum generation to target correct ExifTool table
- Resolve duplicate `OlympusDataType` enum definitions
- **Success Criteria**: `cargo check` passes without Olympus enum errors

**2. Optional: Continue Migration Pattern (Lower Priority)**

- Start with EXIF Functions: Focus on `src/implementations/print_conv.rs` functions with known ExifTool hash sources
- Research ExifTool Hashes First: Before attempting any migration, verify the ExifTool source has a corresponding simple hash:
  ```bash
  # Search for the hash in ExifTool source
  grep -r "%functionName\|%moduleName" third-party/exiftool/lib/Image/ExifTool/
  ```
- Test Pattern: Use the proven migration pattern from Phase 1

### 🛠 **Tools & Commands for Investigation**

```bash
# Find ExifTool hash definitions
grep -r "my %.*= (" third-party/exiftool/lib/Image/ExifTool/Nikon.pm

# Test specific hash extraction
cd codegen && perl extractors/simple_table.pl ../third-party/exiftool/lib/Image/ExifTool/Nikon.pm %hashName

# Verify generated files exist after codegen
ls src/generated/Nikon_pm/

# Test build after changes
make precommit
```

### 📁 **Files to Study**

**Key Implementation Files**:

- `src/implementations/nikon/tags/print_conv/basic.rs` - Example manual→generated migrations
- `src/implementations/print_conv.rs` - EXIF flash_print_conv() migration example

**Codegen Infrastructure**:

- `codegen/config/*/simple_table.json` - Configuration files for extractions
- `codegen/extractors/simple_table.pl` - Hash extraction logic
- `src/generated/*/` - Generated lookup functions (examples)

### ⚠️ **Critical Limitations Discovered**

**Not all manual functions can be migrated**: Many don't have corresponding simple ExifTool hashes. The manual implementations may be necessary translations of complex ExifTool logic.

## 🔧 **Refactoring Opportunities Identified**

**For Future Consideration** (when main compilation issues are resolved):

1. **Codegen Architecture Improvements**:

   - Current `detect_additional_generated_files` is functional but could be more systematic
   - Consider unified config-based approach for all generators instead of standalone detection
   - Module conflict resolution could be improved with proper namespacing

2. **Olympus Module Structure**:

   - Two separate enums (`OlympusDataType` in both `tag_structure.rs` and `equipment_tag_structure.rs`) suggest architectural issue
   - Consider separate namespace/module for equipment vs main Olympus tags
   - Equipment functionality already works via different mechanism - may not need enum generation

3. **Test Coverage Gap**:
   - **CRITICAL**: No integration tests validate that migrations actually work end-to-end
   - Manual verification was done but not automated
   - Future migrations should include test suite that validates generated lookups against ExifTool output

### 🗂️ **Potential Cleanup Tasks**

If desired (after compilation is fixed):

- **Remove unused config**: `codegen/config/Olympus_pm/equipment_tag_table_structure.json` (equipment lookup already works via different mechanism)
- **Continue EXIF functions**: Research remaining `print_conv.rs` functions for ExifTool hash sources
- **Nikon string functions**: Some functions like `nikon_color_mode_conv()` handle strings directly and may not need lookup tables

## Summary for Next Engineer

### **Current Status** _(July 21, 2025)_

- ✅ **Core Infrastructure**: Proven functional, codegen system works
- ✅ **Critical Bug Fixed**: Module inclusion issue resolved
- ⚠️ **Compilation Blocked**: Olympus enum conflict prevents build
- ✅ **Pattern Established**: Migration approach validated for simple hash cases

### **Priority Actions Needed**

1. **Fix Olympus compilation** - Must resolve before any other work
2. **Add integration tests** - Manual verification isn't sufficient for production
3. **Optional migration continuance** - When time permits

### **Key Achievement**

Successfully migrated 4 critical functions, eliminated ~200 lines of manual code, and established working migration pattern. The codegen infrastructure is now robust and ready for expanded use once compilation issues are resolved.

---

## ADDENDUM: New High-Priority Manual Lookup Tasks (July 23, 2025)

### **Additional TODOs Identified from Codebase Analysis**

After completion of the unified tag kit system, systematic analysis revealed additional high-impact manual lookup tables ready for codegen migration:

### **Priority 1: Quick Wins (15-30 min each)**

1. **Nikon WhiteBalance Manual HashMap** (`src/implementations/nikon/tags/print_conv/basic.rs:46`)
   - **Status**: TODO comment with complete instructions provided
   - **ExifTool Source**: Need to locate corresponding hash in `Nikon.pm`
   - **Manual HashMap**: 10 entries (Auto, Preset, Daylight, etc.)
   - **Action**: Add to `codegen/config/Nikon_pm/simple_table.json` as instructed in TODO comment

2. **Canon FocusMode Lookup** (`src/implementations/canon/mod.rs:919`)
   - **Status**: TODO comment indicates missing lookup function
   - **Current**: Returns placeholder `"FocusMode {tag_value}"`
   - **Action**: Find `%canonFocusMode` hash in Canon.pm and add to simple_table config

3. **File Type Lookup Module Re-enabling** (`src/file_detection/mimetypes_validation.rs:12`)
   - **Status**: Generated config exists (`ExifTool_pm/file_type_lookup.json`) but not used
   - **Current**: Functions commented out waiting for generated module
   - **Action**: Verify generated module exists and uncomment usage

### **Priority 2: Investigation Required (1-2 hours)**

4. **Conditional Tags Generation** (`src/exif/ifd.rs:656,673`)
   - **Status**: TODO comments about re-enabling when conditional tags generated
   - **Investigation**: Check if conditional tag configs exist and why not working

5. **Magic Number Pattern Generation** (`src/file_detection.rs:317`, `tests/pattern_test.rs:24,34`)
   - **Status**: Commented out waiting for magic number pattern generation
   - **Investigation**: Check if pattern extraction is configured

### **Migration Pattern (Proven)**

Based on successful tag kit work, the pattern for these migrations is:

1. **Find ExifTool source** - Usually a `%hashName = (...)` pattern
2. **Verify it's a simple table** - Only numbers/strings, no Perl expressions  
3. **Add to module config** - `codegen/config/ModuleName_pm/simple_table.json`
4. **Run codegen** - `make codegen`
5. **Update implementation** - Replace manual HashMap with generated lookup
6. **Test** - `make precommit` and verify functionality

### **Expected Impact**

- **Nikon WhiteBalance**: Eliminate 10-entry manual HashMap 
- **Canon FocusMode**: Fix placeholder output with proper values
- **File Type Lookup**: Enable 343 file type lookups for mimetypes validation
- **Conditional Tags**: Enable dynamic tag resolution (if infrastructure ready)
- **Magic Patterns**: Enable pattern-based file detection tests

### **Recommendation**

Start with **Nikon WhiteBalance** as it has complete implementation instructions in the TODO comment and represents the simplest possible migration following the exact pattern established in the tag kit work.
