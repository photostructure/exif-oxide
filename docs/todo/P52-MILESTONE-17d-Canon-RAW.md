# P52: Canon RAW Support

## Project Overview

**Goal**: Implement Canon RAW formats (CR2, CRW, and CR3) with comprehensive maker note processing

**Problem Statement**: Need Canon RAW support with focus on `required` tags in docs/tag-metadata.json. The infrastructure is solid and extracting Canon tags correctly, but needs output formatting fixes and coverage expansion.

**Current Status**: 60% Complete - Core infrastructure in place, integration work needed
- **54/232 Canon MakerNotes tags** (23% coverage vs. ExifTool's 232 tags)
- Major systems working: SHORT arrays, binary data extraction, PrintConv infrastructure
- Quality is much higher than initially documented

## Background & Context

**Why This Work Is Needed**:
- Canon is a major camera manufacturer requiring RAW format support
- Monthly ExifTool releases mean automated code generation is critical
- Complex offset schemes and validation requirements specific to Canon

**Complexity Context**:
- 10,648 lines in ExifTool Canon.pm
- 7 ProcessBinaryData sections (confirmed by source analysis)
- 84 Canon data types with generated lookup tables
- Multiple file formats: CR2 (TIFF-based), CRW, CR3

**Related Design Docs**:
- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - Fundamental principle
- [CODEGEN.md](../CODEGEN.md) - Code generation guide
- [ARCHITECTURE.md](../ARCHITECTURE.md) - High-level system overview

## Technical Foundation

**Key Codebases**:
- **ExifTool Canon.pm**: `third-party/exiftool/lib/Image/ExifTool/Canon.pm` (10,648 lines)
- **Implementation**: `src/implementations/canon/` - existing Canon modules
- **Generated Code**: `src/generated/Canon_pm/` - all generated Canon tables
- **RAW Handler**: `src/raw/formats/canon.rs` - main Canon RAW handler

**Critical Documentation**:
- ExifTool Canon.pm ProcessBinaryData sections (search `PROCESS_PROC.*ProcessBinaryData`)
- Canon offset schemes in MakerNotes.pm lines 1135-1141
- [MODULE_OVERVIEW.md](../../third-party/exiftool/doc/concepts/MODULE_OVERVIEW.md)
- [PROCESS_PROC.md](../../third-party/exiftool/doc/concepts/PROCESS_PROC.md)

**APIs & Systems**:
- Canon offset detection: `src/implementations/canon/offset_schemes.rs`
- Binary data extraction: `src/implementations/canon/binary_data.rs`
- Generated lookup tables: `src/generated/Canon_pm/*_inline.rs`
- Tag structure: `src/generated/Canon_pm/tag_structure.rs`

## Work Completed

### ✅ Major Infrastructure Achievements (January 2025)

**Build System & Code Cleanup**:
- **FIXED**: Compilation errors by removing duplicate offset code in `src/raw/formats/canon.rs`
- **FIXED**: Import errors and unused code warnings
- **SUCCESS**: Build passes without errors, all warnings resolved
- **CONFIRMED**: `make precommit` passes (includes codegen, format, build)

**Canon Offset Code Consolidation**:
- **REMOVED**: Duplicate `CanonOffsetManager` and `CanonOffsetConfig` types from `canon.rs`
- **USING**: Existing implementation in `src/implementations/canon/offset_schemes.rs`
- **BENEFIT**: Eliminated 50+ lines of duplicate code, single source of truth

**CR2 Compatibility Testing Setup**:
- **ADDED**: "cr2" to `tools/generate_exiftool_json.sh` SUPPORTED_EXTENSIONS
- **GENERATED**: Reference JSON files for 66 test images including 2 CR2 files
- **SUCCESS**: `make compat-gen` completes successfully, ready for testing

### ✅ Core Architecture Completed

**Canon Format Detection and CLI Integration**:
- Added CR2, CRW, CR3 support to `src/formats/mod.rs` extract_metadata function
- CR2 correctly routes through TIFF processor (since CR2 is TIFF-based)
- CRW and CR3 route through RAW processor
- Files: `src/formats/mod.rs` lines 507-591

**Canon Offset Scheme Implementation**:
- Existing implementation in `src/implementations/canon/offset_schemes.rs`
- Supports 4/6/16/28 byte offset variants based on camera model
- Model detection patterns implemented (20D, 350D, REBEL XT, etc.)

**Canon RAW Handler Infrastructure**:
- Created `src/raw/formats/canon.rs` with CanonRawHandler
- Registered in RAW processor (`src/raw/processor.rs`)
- Format detection working in `src/raw/detector.rs`

### ✅ Major Breakthrough (July 2025)

**ACTUAL STATUS**: Extracting **54 Canon MakerNotes tags** (23% coverage vs. ExifTool's 232 tags)

**KEY ACHIEVEMENTS**:
- ✅ **SHORT array extraction fix** - Root cause resolved, multi-value SHORT arrays now working
- ✅ **Binary data processors** - All major Canon sections integrated and extracting data
- ✅ **Main Canon table tags** - Core tags like CanonImageType, CanonFirmwareVersion working
- ✅ **PrintConv infrastructure** - Lookup table integration operational
- ✅ **Namespace issues** - Canon tags now correctly appear in "MakerNotes:" group

**WORKING BINARY DATA SECTIONS**:
- **CameraSettings (0x0001)**: 6 tags extracted (MacroMode, SelfTimer, Quality, FocusMode, etc.)
- **FocalLength (0x0002)**: 4 tags extracted (FocalType, FocalLength, FocalPlaneXSize, etc.)
- **ShotInfo (0x0004)**: 8 tags extracted (AutoISO, BaseISO, MeasuredEV, TargetAperture, etc.)
- **AFInfo2 (0x0026)**: Multiple AF-related tags
- **Panorama (0x0005)**: Integrated but needs testing

### Decision Rationale

**CR2 Processing Architecture**: Route CR2 through TIFF processor which calls Canon maker note handlers. This is correct because CR2 is TIFF-based but needs special Canon handling.

**Offset Management**: Canon uses absolute file offsets for maker note data, not relative offsets. Current offset handling in `find_canon_tag_data_with_full_access()` is correct.

**Generated Code Strategy**: Focus on using generated tables in `src/generated/Canon_pm/` instead of manual lookups to maintain compatibility with monthly ExifTool releases.

## 🛠️ Fixes Implemented (July 21, 2025)

### 1. SHORT Array Extraction (COMPLETED)

- Fixed `extract_short_array_value()` in `src/value_extraction.rs`
- Updated IFD parser in `src/exif/ifd.rs` to handle count > 1
- Canon binary data tags now extract as `TagValue::U16Array`

### 2. Canon PrintConv Integration (COMPLETED)

- **Fixed prefix issue**: `apply_camera_settings_print_conv()` now strips "MakerNotes:" prefix
- **Added** `apply_canon_main_table_print_conv()` for main Canon table tags
- **Working lookups**: CanonModelID, MacroMode, Quality (partially), SelfTimer

### 3. Namespace Assignment (COMPLETED)

- **Fixed** `create_tag_source_info()` in `src/exif/tags.rs`
- Canon tags now correctly appear in "MakerNotes:" group instead of "EXIF:"

### 4. Binary Data Integration (COMPLETED)

- All Canon binary data extractors integrated in `process_other_canon_binary_tags_with_reader()`
- FocalLength, ShotInfo, Panorama extractors now called and working

## High level guidance

- **Follow Trust ExifTool**: Study how ExifTool processes CR2 files exactly. See [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md)
- **Use Codegen**: Switch any manual lookup tables to generated code. See [CODEGEN.md](../CODEGEN.md)
- **Study ExifTool Sources**: [Canon.pm](../../third-party/exiftool/lib/Image/ExifTool/Canon.pm) and [module docs](../../third-party/exiftool/doc/modules/Canon.md)

## Overview

Canon RAW support with comprehensive maker note processing. The infrastructure is solid and extracting Canon tags correctly, but needs output formatting fixes and coverage expansion.

**Complexity Context**:

- 10,648 lines in ExifTool Canon.pm
- 7 ProcessBinaryData sections (confirmed by source analysis)
- 84 Canon data types with generated lookup tables
- Complex offset schemes and validation requirements

## 🔍 ACTUAL IMPLEMENTATION STATUS (July 20, 2025)

### What's Actually Working (Better Than Expected)

**✅ MakerNotes Processor Dispatch**:

- Canon detection correctly identifies "Canon::Main" processor
- Fallback system properly routes to `canon::process_canon_makernotes()`
- Binary data extraction infrastructure is functional

**✅ Binary Data Processing (Partially Working)**:

- Canon CameraSettings extraction: 6 tags (FocusMode, CanonFlashMode, Quality)
- Canon AFInfo processing: 13 tags (AFAreaXPositions, AFImageHeight, etc.)
- Synthetic tag ID generation and storage working correctly
- Generated lookup table integration functioning

**✅ Core Architecture**:

- Canon IFD parsing with proper offset handling
- TIFF-based maker note processing
- PrintConv application system operational

## 🎯 CRITICAL GAP ANALYSIS: 19/232 Tags (8.2% Coverage)

### Missing Main Canon Table Tags

**ExifTool's Canon::Main table** contains the primary Canon tags that should be extracted directly from the Canon IFD structure. Our implementation is missing most of these:


## Remaining Tasks

### Priority 1: Fix PrintConv Type Mismatches (HIGH CONFIDENCE - 2-4 hours)

**CRITICAL ISSUE**: PrintConv lookups are failing due to type mismatches between extracted values and lookup functions.

**Problem**: Binary data extractors return `I16` values, but lookup functions expect `u8`. Examples:
- `MacroMode`: Extracted as `I16(0)`, lookup expects `u8` → should show "Off" but shows raw value
- `Quality`: Extracted as `I16(0)`, lookup expects `i16` → should show "RAW" but shows 0
- `FocusMode`: Missing lookup function entirely

**Implementation Strategy**:
1. **Fix type conversions** in `apply_camera_settings_print_conv()` - convert `I16` to expected types
2. **Add missing lookup functions** - check what's available in generated Canon modules
3. **Test with real values** - ensure MacroMode(0)→"Off", Quality(0)→"RAW", etc.

**Files**: 
- `src/implementations/canon/mod.rs:apply_camera_settings_print_conv()` - Line 870+
- `src/generated/Canon_pm/camerasettings_inline.rs` - Available lookup functions
- `src/generated/Canon_pm/canonquality.rs` - Quality lookup specifically

**Expected Outcome**: Immediate improvement in human-readable output for existing 54 tags

### Priority 2: Integrate Generated ProcessBinaryData Processors (RESEARCH NEEDED)

**Problem**: 84 generated Canon data types exist but aren't connected to binary processing logic
- Canon has only 7 ProcessBinaryData sections in ExifTool (not 169 as initially thought)
- Generated processors should be available via codegen
- Need to map CanonDataType enum values to appropriate processors

**Key insight**: Focus on the subdirectory tags that have `has_subdirectory() == true`

**Research Required**: 
- Study ExifTool Canon.pm ProcessBinaryData sections
- Map subdirectory tags to appropriate binary processors
- Understand conditional processing requirements

### Priority 3: Implement Canon IFD Processing (RESEARCH NEEDED)

**Current State**: process_cr2() method is a stub that delegates to TIFF processor

**Tasks**:
- Replace TODO with actual Canon maker note processing
- Route Canon maker note tags through ProcessBinaryData handlers
- Handle subdirectories for tags like CameraSettings, AFInfo, etc.

**Pattern**: Study `src/implementations/canon/mod.rs::process_canon_makernotes()`

### Priority 4: Canon Color Data Processing (RESEARCH NEEDED)

**Goal**: Implement ColorData1-12 processing (conditional based on count)
- Different camera generations use different ColorData formats
- Requires model-specific conditional logic

### Priority 5: Write Comprehensive Tests (MEDIUM CONFIDENCE)

**Available test files**: 
- `test-images/canon/Canon_T3i.CR2`
- `third-party/exiftool/t/images/CanonRaw.cr2`

**Tasks**:
- Test offset scheme detection for different models
- Verify tag extraction matches ExifTool output
- Use `make compat-test` after implementing processing

## Prerequisites

**None identified** - Core infrastructure is complete and functional.

**Optional Dependencies**:
- If advanced ColorData processing is needed, may require camera model database
- Enhanced error handling could benefit from Canon-specific error types

All major dependencies (offset detection, binary data extraction, PrintConv infrastructure) are already implemented and working.

## Testing Strategy

### Unit Tests
- Test offset scheme detection for different camera models (20D, 350D, REBEL XT, etc.)
- Verify type conversion in PrintConv functions (I16 → u8/i16)
- Test synthetic tag ID generation and collision avoidance

### Integration Tests
- **Primary Test File**: `test-images/canon/Canon_T3i.CR2`
- **Additional Test File**: `third-party/exiftool/t/images/CanonRaw.cr2`
- Use `make compat-test` to verify tag extraction matches ExifTool output
- Test with multiple Canon camera models to verify offset schemes

### Manual Testing Steps
```bash
# Verify current Canon tag extraction
./target/release/exif-oxide test-images/canon/Canon_T3i.CR2 | grep '"MakerNotes:' | wc -l
# Should show 54 tags

# Check PrintConv status for specific tags
./target/release/exif-oxide test-images/canon/Canon_T3i.CR2 | grep -E "(Quality|MacroMode|SelfTimer)"

# Validate against ExifTool
cargo run --bin compare-with-exiftool test-images/canon/Canon_T3i.CR2 MakerNotes:

# Full debug logging for PrintConv issues
RUST_LOG=debug ./target/release/exif-oxide test-images/canon/Canon_T3i.CR2 2>&1 | grep -E "(PrintConv|apply_camera_settings_print_conv)"
```

### Reference JSON Generation
- **Command**: `make compat-gen` generates ExifTool reference files
- **Location**: Generated in `generated/exiftool-json/` directory
- **Coverage**: 66 test images including 2 CR2 files ready for testing

## Success Criteria & Quality Gates

### Current Achievement
- **54/232 Canon MakerNotes tags** (23% coverage) ✅
- **Build passes cleanly** with `make precommit` ✅
- **No duplicate code** between canon.rs and implementations/canon/ ✅
- **Generated code infrastructure** available and partially integrated ✅

### Completion Targets

**Phase 1: PrintConv Quality Improvement** (Highest ROI - URGENT)
- **Effort**: 2-4 hours
- **Outcome**: Immediate quality improvement - better human-readable output for existing 54 tags
- **Success**: MacroMode shows "Off", Quality shows "RAW", not raw numeric values

**Phase 2: Main Table Expansion** (High ROI)
- **Effort**: 4-6 hours
- **Outcome**: 54 → 75 tags (20+ new main table tags)
- **Success**: Core Canon metadata like FileNumber, OwnerName, DateStampMode extracted

**Phase 3: ColorData Model-Specific Processing** (Medium ROI)
- **Effort**: 8-12 hours
- **Outcome**: 75 → 120+ tags (45+ new ColorData tags)
- **Success**: Model-dependent color processing parameters extracted

**Final Success Criteria**:
- [ ] **CLI can successfully read CR2 files and extract metadata**
- [ ] **Output matches `exiftool -j` for Canon-specific tags**
- [ ] **Key subdirectory tags extracted**: CameraSettings, ShotInfo, AFInfo2
- [ ] **Offset schemes work correctly** for different camera models
- [ ] **Tests pass for Canon sample files** (`make compat-test`)
- [ ] **PrintConv provides human-readable output** for major tags
- [ ] **75%+ coverage target** (180+ tags) achievable with current architecture

### Quality Gates
- **Build**: `make precommit` must pass
- **Testing**: `make compat-test` shows reasonable Canon tag coverage
- **Code Quality**: Use generated lookup tables, not manual HashMap/match statements
- **ExifTool Compatibility**: Follow Trust ExifTool principle - no "improvements" to logic

## Gotchas & Tribal Knowledge

### Technical Discoveries

**Canon Main Table Structure**: ExifTool's Canon::Main table is simpler than initially thought. Most tags are direct value extractions, not complex binary data.

**Canon::Main Tag Categories**:
1. **Direct Value Tags** (~40 tags): Simple value extraction (strings, numbers)
   - Examples: CanonImageType, CanonFirmwareVersion, FileNumber, OwnerName
   - Implementation: Standard IFD tag extraction, no special processing needed

2. **SubDirectory Tags** (~7 tags): Point to binary data sections
   - Examples: CanonCameraSettings, CanonShotInfo, CanonFocalLength
   - Implementation: Already working via binary data extractors

3. **Conditional/Model Tags** (~5 tags): Model-dependent processing
   - Examples: ColorData variants, CameraInfo blocks
   - Implementation: Requires model detection logic

**CRITICAL INSIGHT**: The missing tags are mostly from category 1 (direct values) and subcategories of category 2 (binary data details). The core binary data extraction framework is already functional.

### Architecture Insights

**Two-Phase Processing System** (CRITICAL UNDERSTANDING):
- **Phase 1**: New processor registry in `src/processor_registry/` - No "Canon::Main" processor registered
- **Phase 2**: Fallback system calls `canon::process_canon_makernotes()` directly when registry lookup fails
- **KEY INSIGHT**: The fallback system works correctly - Canon processing happens in the legacy pathway

**Binary Data Integration Gap**: `process_canon_makernotes()` only calls 2 of 5 available binary data extractors:
- Available but unused: `extract_focal_length()`, `extract_shot_info()`, `extract_panorama()`, `extract_my_colors()`
- Integration pattern: Each extractor returns `Vec<(tag_name, tag_value)>` following CameraSettings pattern

### Critical Technical Details

**Offset Management**: Canon uses absolute file offsets for maker note data, not relative offsets. Current implementation in `find_canon_tag_data_with_full_access()` is correct - no additional offset adjustment needed.

**Canon Offset Schemes by Model**:
- **4 bytes**: Default for most cameras
- **6 bytes**: 20D, 350D, REBEL XT, Kiss Digital N
- **16 bytes**: PowerShot, IXUS, IXY models
- **28 bytes**: FV-M30, Optura series

**Type Conversion Issue** (URGENT FIX NEEDED):
```rust
// CURRENT (BROKEN): MacroMode extracted as I16(0)
TagValue::I16(0) → lookup_camera_settings__macro_mode(0_u8) → FAILS

// SOLUTION: Type conversion in PrintConv
if let Some(value) = tag_value.as_i16() {
    let u8_value = value as u8;  // Convert I16 → u8
    if let Some(result) = lookup_camera_settings__macro_mode(u8_value) {
        return TagValue::String(result.to_string());
    }
}
```

**Files That Should NOT Be Modified**:
- `src/value_extraction.rs` - SHORT array extraction is working
- `src/exif/ifd.rs` - IFD parsing correctly handles Canon structure
- `src/exif/tags.rs` - Namespace assignment is fixed
- Core binary data extractors in `src/implementations/canon/binary_data.rs` - These work

**Generated Module Issue**: `src/generated/Canon_pm/camerasettings_inline.rs` shows "Auto-generated from Olympus.pm" which suggests codegen mixed up modules. May explain missing Canon-specific lookup functions.

### What's Working (Don't Break This!)

1. **SHORT array extraction** - `src/value_extraction.rs` handles multi-value arrays correctly
2. **Binary data integration** - All major Canon sections are called and extracting data
3. **Canon IFD parsing** - Correctly handles Canon maker note structure
4. **Synthetic tag IDs** - Hash-based generation (0xC000-0xCFFF range) works reliably
5. **Main table extraction** - Basic Canon tags (ImageType, FirmwareVersion, ModelID) working
6. **Namespace assignment** - Canon tags correctly appear in "MakerNotes:" group

### Issues Encountered and Solutions

**✅ RESOLVED: Duplicate Code Issue**
- **Problem**: Started implementing offset management in canon.rs that already existed
- **Solution**: Removed duplicate code, imported existing implementation

**✅ RESOLVED: Build Errors**
- **Problem**: Compilation failed due to undefined types
- **Solution**: Removed duplicate offset management code, cleaned up imports

**✅ RESOLVED: CR2 Compatibility Testing**
- **Problem**: CR2 files not included in compatibility testing
- **Solution**: Added "cr2" to SUPPORTED_EXTENSIONS, generated reference files

**⚠️ ARCHITECTURE: ProcessBinaryData Integration Gap**
- **Problem**: 84 generated Canon data types exist but aren't connected to binary processing logic
- **Analysis**: Only 7 actual ProcessBinaryData sections in Canon.pm
- **Next Step**: Map subdirectory tags to appropriate binary processors

## Key Code Locations

### Core Implementation Files
- `src/raw/formats/canon.rs` - Main Canon RAW handler (partially complete)
- `src/implementations/canon/` - Existing Canon implementation modules
  - `offset_schemes.rs` - Offset detection (complete)
  - `binary_data.rs` - Binary data extraction functions
  - `tags.rs` - Tag name resolution
  - `af_info.rs` - AF Info processing
  - `mod.rs` - Canon coordinator

### Generated Code
- `src/generated/Canon_pm/` - All generated Canon tables
  - `tag_structure.rs` - CanonDataType enum with 84 tags
  - `main_conditional_tags.rs` - Conditional tag resolution
  - Various inline PrintConv tables

### Integration Points
- `src/formats/mod.rs` - CLI entry point (lines 507-591)
- `src/raw/processor.rs` - RAW processor registration
- `src/raw/detector.rs` - Format detection

### Working Code Architecture

**Main Processing Flow**:
1. `process_canon_makernotes()` in `src/implementations/canon/mod.rs:40+` - Entry point
2. `process_subdirectory()` - Extracts basic Canon IFD structure (main table tags)
3. `apply_canon_main_table_print_conv()` - Applies PrintConv to main table tags
4. `process_canon_binary_data_with_existing_processors()` - Processes binary data tags
5. `apply_camera_settings_print_conv()` - Applies PrintConv to binary data tags

**Current Working Binary Data Extractors** (all in `src/implementations/canon/binary_data.rs`):
- `extract_camera_settings()` - CameraSettings (0x0001) → 6 tags
- `extract_focal_length()` - FocalLength (0x0002) → 4 tags
- `extract_shot_info()` - ShotInfo (0x0004) → 8 tags
- `extract_panorama()` - Panorama (0x0005) → integrated but needs verification

**PrintConv Integration Points**:
- **Main table tags**: `apply_canon_main_table_print_conv()` at `src/implementations/canon/mod.rs:819+`
- **Binary data tags**: `apply_camera_settings_print_conv()` at `src/implementations/canon/mod.rs:870+`
- **Generated lookups**: `src/generated/Canon_pm/` modules with `lookup_*()` functions

### Available Test Files
- **`test-images/canon/Canon_T3i.CR2`** - Primary test file
- **`third-party/exiftool/t/images/CanonRaw.cr2`** - Additional test file
- **Reference JSON**: Generated in `generated/exiftool-json/` directory

### Essential Build Commands
- **`make precommit`** - Full build with codegen, format, test
- **`make compat-gen`** - Generate ExifTool reference files
- **`make compat-test`** - Run compatibility tests
- **`cargo build`** - Quick build check

---

**⚠️ IMPORTANT**: This TPP has been updated with handoff information from HANDOFF-20250120-canon-raw-implementation.md. The handoff document can now be safely deleted as all critical information has been preserved and integrated into this comprehensive technical project plan.
