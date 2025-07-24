# Technical Project Plan: Fix KyoceraRaw Codegen Extraction Bug

## Project Overview

**High-level goal**: Fix a bug in the exif-oxide codegen system where KyoceraRaw tag_kit configurations are read correctly but silently skipped during the extraction phase.

**Problem statement**: The KyoceraRaw module's `tag_kit.json` configuration is being read and validated, but the Perl extraction never runs, preventing automatic generation of ISO lookup tables. This forces developers to maintain manual lookup functions instead of using the automated codegen system.

## Background & Context

**Why this work is needed**:
- Manual maintenance of ExifTool data is banned (see [CODEGEN.md](../CODEGEN.md) "Manual Porting Banned" section)
- KyoceraRaw ISO lookup function needs to be replaced with generated code
- Current bug prevents KyoceraRaw from benefiting from automatic ExifTool updates
- Other modules (Canon, Sony, Olympus, etc.) work correctly, making this an isolated bug

**Related design docs**:
- [CODEGEN.md](../CODEGEN.md) - Complete codegen system documentation
- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - Why manual porting is prohibited

## Technical Foundation

**Key codebases**:
- `codegen/src/extraction.rs` - Extraction orchestration
- `codegen/src/extractors/tag_kit.rs` - Tag kit extraction logic
- `codegen/extractors/tag_kit.pl` - Perl extraction script (works correctly)
- `codegen/config/KyoceraRaw_pm/tag_kit.json` - Configuration file

**Critical files**:
- `src/raw/utils.rs:39-58` - Manual `kyocera_iso_lookup` function to be replaced
- `third-party/exiftool/lib/Image/ExifTool/KyoceraRaw.pm:56-70` - Source data

**Systems to familiarize with**:
- Rust-orchestrated codegen pipeline (July 2025 architecture)
- Tag kit extraction workflow
- ExifTool perl module structure

## Work Completed

**Issue diagnosis**:
- ✅ Confirmed Perl extractor works correctly: `perl tag_kit.pl ../third-party/exiftool/lib/Image/ExifTool/KyoceraRaw.pm Main` produces valid JSON
- ✅ Identified extraction data matches manual function exactly
- ✅ Located bug in Rust orchestration: config is read but extraction phase skipped
- ✅ Confirmed other modules (Canon, Sony, etc.) work correctly with identical config structure

**Root cause identified**:
- ✅ Initial config parsing error: KyoceraRaw used "name" field instead of "table_name" 
- ✅ Path canonicalization error: "../third-party/..." path caused failures
- ✅ **Primary bug**: PrintConv HashMap generator only handled string values, not numeric values in ISO data

**Final fix implemented**:
- ✅ Fixed config parsing in `KyoceraRaw_pm/tag_kit.json` (table_name and path corrections)
- ✅ Enhanced generator in `codegen/src/generators/lookup_tables/mod.rs` to handle numeric PrintConv values
- ✅ DRY refactoring: Created shared `generate_print_conv_entry()` helper function to eliminate duplication
- ✅ Verified PRINT_CONV_0 HashMap now correctly populated with all 13 ISO mappings (7=>"25", 8=>"32", etc.)
- ✅ Updated `kyocera_iso_lookup()` function to use generated tag kit system instead of manual lookup

**Evidence of fix**:
```bash
# Generated file now exists and contains correct data:
$ cat src/generated/KyoceraRaw_pm/tag_kit/other.rs
static PRINT_CONV_0: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("7".to_string(), "25");
    map.insert("8".to_string(), "32");
    // ... all 13 ISO mappings
    map
});

# ISO tag correctly references the HashMap:
print_conv: PrintConvType::Simple(&PRINT_CONV_0),
```

## Project Completed ✅

All tasks have been successfully completed:

### Completed Work

1. ✅ **Fixed config parsing bugs**
   - Corrected KyoceraRaw config to use "table_name" instead of "name"
   - Fixed path resolution from "../third-party/..." to "third-party/..."

2. ✅ **Enhanced PrintConv generator**
   - Added support for numeric values (u64, i64, f64) in addition to strings
   - Generator now handles the ISO PrintConv data correctly (7=>25, 8=>32, etc.)

3. ✅ **DRY code refactoring**
   - Created shared `generate_print_conv_entry()` helper function
   - Eliminated duplication between `lookup_tables/mod.rs` and `tag_kit_modular.rs`

4. ✅ **Integration with tag kit system**
   - Updated `kyocera_iso_lookup()` to use generated tag kit system
   - Replaced manual lookup with automated codegen approach

### Regression Prevention
- ✅ **Systematic fix**: Enhanced generator handles numeric PrintConv values for all modules
- ✅ **Code quality**: DRY refactoring prevents future duplication bugs
- ✅ **Documentation**: This TPP captures the complete debugging process for future reference

## Prerequisites

- Familiarity with Rust debugging and logging
- Understanding of ExifTool Perl module structure
- Basic knowledge of JSON configuration processing

## Testing Strategy

**Debug validation**:
```bash
# Test manual extraction (should work):
cd codegen && perl extractors/tag_kit.pl ../third-party/exiftool/lib/Image/ExifTool/KyoceraRaw.pm Main

# Test full pipeline with debug logging:
RUST_LOG=debug make codegen 2>&1 | grep -A5 -B5 KyoceraRaw

# Verify fix:
ls -la codegen/generated/extract/tag_kits/*kyocera* # Should exist after fix
```

**Integration testing**:
- Verify generated lookup table matches manual function data
- Test replacement in `src/raw/utils.rs`
- Run `make precommit` to ensure no regressions

## Success Criteria & Quality Gates ✅

**Definition of done**:
- ✅ KyoceraRaw tag_kit extraction runs during `make codegen`
- ✅ Generated file `codegen/generated/extract/tag_kits/kyoceraraw__tag_kit.json` exists
- ✅ Generated Rust code `src/generated/KyoceraRaw_pm/tag_kit/` contains ISO lookup function
- ✅ Manual `kyocera_iso_lookup` function replaced with generated tag kit system
- ✅ No regression in other module extractions
- ✅ `make precommit` passes (all clippy warnings fixed)

**Completed reviews**: All changes implemented with systematic debugging approach and DRY refactoring

## Gotchas & Tribal Knowledge

**Critical insights**:
- **Perl extraction works perfectly** - bug is purely in Rust orchestration
- **Config syntax is correct** - identical to working modules like GPS (1 table)
- **Module appears in later phases** - validation and generation phases see KyoceraRaw correctly
- **Silent failure** - no error messages, extraction is simply skipped

**Investigation hints**:
- Focus on extraction phase batching in `extraction.rs`
- Compare single-table (GPS, KyoceraRaw) vs multi-table (Canon) processing
- Check if module name filtering/validation excludes KyoceraRaw
- Verify ExifTool file path resolution for KyoceraRaw.pm

**ExifTool data verification**:
```perl
# Expected ISO data in KyoceraRaw.pm:56-70
7 => 25, 8 => 32, 9 => 40, 10 => 50, 11 => 64, 12 => 80,
13 => 100, 14 => 125, 15 => 160, 16 => 200, 17 => 250, 
18 => 320, 19 => 400
```

**Testing the fix**:
After implementing the fix, the generated lookup should be usable as:
```rust
use crate::generated::KyoceraRaw_pm::tag_kit::lookup_iso_print_conv;
// Replace manual kyocera_iso_lookup with generated equivalent
```