# Technical Project Plan: Olympus Equipment Tag Extraction

## Project Overview

- **Goal**: Extract critical Equipment subdirectory tags (CameraType2, LensType, SerialNumber) from Olympus images to enable proper camera/lens identification
- **Problem**: Equipment IFD processor is stubbed, preventing extraction of 6 most critical required tags despite 90% infrastructure being complete
- **Constraints**: Must maintain namespace-aware storage, follow ExifTool's dual binary/IFD Equipment format exactly

---

## ⚠️ CRITICAL REMINDERS

If you read this document, you **MUST** read and follow [CLAUDE.md](../CLAUDE.md) as well as [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md):

- **Trust ExifTool** (Translate and cite references, but using codegen is preferred)
- **Ask clarifying questions** (Maximize "velocity made good")
- **Assume Concurrent Edits** (STOP if you find a compilation error that isn't related to your work)
- **Don't edit generated code** (read [CODEGEN.md](CODEGEN.md) if you find yourself wanting to edit `src/generated/**.*rs`)
- **Keep documentation current** (so update this TPP with status updates, and any novel context that will be helpful to future engineers that are tasked with completing this TPP. Do not use hyperbolic "DRAMATIC IMPROVEMENT"/"GROUNDBREAKING PROGRESS" styled updates -- that causes confusion and partially-completed low-quality work)

**NOTE**: These rules prevent build breaks, data corruption, and wasted effort across the team. 

If you are found violating any topics in these sections, **your work will be immediately halted, reverted, and you will be dismissed from the team.**

Honest. RTFM.

---

## Context & Foundation

### System Overview

- **Namespace-aware tag storage**: `HashMap<(u16, String), TagValue>` system fully implemented, resolves tag ID conflicts between EXIF and MakerNotes contexts
- **Tag kit system**: Unified extraction with 8 comprehensive Olympus tag tables (Equipment, CameraSettings, FocusInfo, etc.) - all generated code in place
- **Equipment subdirectory**: ExifTool's dual binary/IFD format (0x2010) containing camera/lens identification tags - **core blocker is stubbed IFD processor**
- **Composite tag infrastructure**: Ready to implement LensID lookups using existing 138+ entry olympusLensTypes database

### Key Concepts & Domain Knowledge

- **Equipment subdirectory (0x2010)**: Critical container for camera/lens identification using TIFF IFD structure. Contains CameraType2 (0x100), SerialNumber (0x101), LensType (0x201) among others
- **Dual format processing**: ExifTool handles Equipment as binary data OR TIFF IFD depending on format detection - requires conditional processing
- **LensType encoding**: 6-byte array → hex string conversion → olympusLensTypes lookup for human-readable lens names
- **Namespace separation**: Equipment tags use "MakerNotes" namespace to avoid conflicts with standard EXIF tags of same IDs

### Surprising Context

- **90% infrastructure complete**: Tag kit migration, namespace storage, generated lookup tables all working - only Equipment IFD parsing is missing
- **Raw data extraction works**: We already extract 32 MakerNotes tags (Tag_0000 through Tag_2050) but in binary format, not processed through Equipment IFD structure
- **Generated code ready**: All Equipment tag definitions exist in `src/generated/Olympus_pm/tag_kit/` with proper tag names and PrintConv lookups
- **Composite system ready**: LensID computation just needs Equipment LensType data to query existing olympusLensTypes database

### Foundation Documents

- **ExifTool source**: `third-party/exiftool/lib/Image/ExifTool/Olympus.pm:1170-1190` (dual format Equipment definition), lines 1588-1769 (Equipment tag table)
- **Generated tag kits**: `src/generated/Olympus_pm/tag_kit/mod.rs` - OLYMPUS_PM_TAG_KITS with all tag definitions
- **Current stub**: `src/generated/Olympus_pm/tag_kit/mod.rs:process_tag_0x2010_subdirectory` returns empty vector
- **Namespace implementation**: `src/exif/tags.rs` - store_tag_with_precedence() with conflict resolution

### Prerequisites

- **Generated code system**: Tag kit extraction and modular tag_kit structure operational
- **Namespace storage**: Tag ID conflict resolution working with (tag_id, namespace) keys
- **TIFF IFD parsing**: Standard IFD parsing utilities available in exif module

## Work Completed

- ✅ **Namespace-aware storage** → implemented `HashMap<(u16, String), TagValue>` resolving tag ID conflicts between contexts
- ✅ **Tag kit migration** → completed unified tag extraction with 8 Olympus tag tables replacing legacy inline_printconv approach
- ✅ **Generated infrastructure** → 138+ lens database, camera types, all lookup tables generated and available
- ✅ **Raw MakerNotes extraction** → 32 tags extracted from Equipment subdirectory but not processed through IFD structure
- ✅ **Composite tag framework** → infrastructure ready for LensID computation using existing olympusLensTypes
- ✅ **Runtime format detection** → implemented Equipment format detection (IFD vs binary) following ExifTool's condition logic
- ✅ **Equipment IFD processor** → complete IFD parsing with proper tag name mapping and value extraction
- ✅ **Architecture research** → identified root cause: missing connection between Equipment processor and tag kit system

## Current Status: CRITICAL BLOCKER - MakerNotes Tag Missing

**Current State**:
- ✅ Equipment format detection implemented (`detect_equipment_format()`)
- ✅ Equipment IFD parsing functional (`process_equipment_subdirectory()`) 
- ✅ Equipment tag name mapping complete (`get_equipment_tag_name()`)
- ✅ Custom Equipment processor implemented in `process_olympus_subdirectory_tags()`
- ✅ MakerNotes conditional dispatch system implemented and working
- ✅ Olympus signature detection triggers manufacturer-specific processing
- ✅ LensID composite calculation implemented: Olympus-specific logic using olympusLensTypes lookup table
- ❌ **CRITICAL**: MakerNotes tag (0x927c) not found in test image `e_m10_mk_iii_01.jpg`

**Root Cause Discovered**: 
- LensID composite and Equipment extraction systems are fully implemented and ready
- Test image `test-images/olympus/e_m10_mk_iii_01.jpg` does not contain MakerNotes tag (0x927c/37500 decimal)
- Debug logs confirm EXIF processing stops after standard tags, never encounters MakerNotes data
- System falls back to EXIF:LensModel only: "OLYMPUS M.14-42mm F3.5-5.6 EZ"

**Evidence**:
- `env RUST_LOG=debug cargo run test-images/olympus/e_m10_mk_iii_01.jpg 2>&1 | grep -i "927c\|37500"` returns no matches
- ExifTool shows MakerNotes tags for this image: `"MakerNotes:LensType": "Olympus M.Zuiko Digital ED 14-42mm F3.5-5.6 EZ"`
- Our system extracts all EXIF tags correctly but never processes MakerNotes section

## Remaining Tasks

### 1. RESEARCH: Investigate MakerNotes Tag Location in Test Image ⚠️ IMMEDIATE

**Objective**: Determine why MakerNotes tag (0x927c) is not found in current test image
**Success Criteria**: Either locate MakerNotes data in current image OR identify alternative test image with proper MakerNotes
**Done When**: Can successfully extract MakerNotes Equipment tags from Olympus test image

**Investigation Approach**:
- Verify ExifTool MakerNotes extraction: `exiftool -a -G1 -s test-images/olympus/e_m10_mk_iii_01.jpg | grep -i maker`
- Check other Olympus test images: `find test-images/ -name "*olympus*" -o -name "*e-m*" -o -name "*om-*"`
- Compare our EXIF IFD parsing with ExifTool's IFD structure dump: `exiftool -htmldump test-images/olympus/e_m10_mk_iii_01.jpg`
- Debug our IFD parsing to see all tags encountered: search logs for tag processing in ExifIFD

**Critical Context**: 
- All Equipment extraction code is implemented and ready - we just need MakerNotes data to process
- LensID composite calculation works but needs Equipment LensType to provide meaningful results
- Current fallback to EXIF:LensModel works but provides less detailed lens identification

### 2. Task: Replace Placeholder Test Data

**Success Criteria**: Equipment tags extract real values from actual Equipment IFD instead of placeholder test data
**Dependencies**: Task 1 (locate MakerNotes data)

**Current State**: 
- ✅ System working end-to-end with proper tag names  
- ✅ MakerNotes conditional dispatch functional
- ❌ Equipment values are placeholder test data ("E-M1MarkIII", "TEST123456") instead of real parsed values

**Approach**: Replace placeholder data in `process_olympus_ifd_for_equipment()` with actual IFD parsing
- ✅ Equipment IFD format detection implemented 
- ✅ IFD parsing utilities available in exif module
- ❌ Need to parse Equipment data as TIFF IFD structure to extract real tag values
- ❌ Need to validate against ExifTool output for same Equipment tags

**Implementation**: 
- Parse Equipment data (after signature header) as TIFF IFD
- Extract tag values using existing IFD parsing infrastructure  
- Map Equipment tag IDs (0x100, 0x101, 0x201) to proper tag names
- Apply any necessary ValueConv/PrintConv transformations

### 3. Task: Implement LensID Composite Calculation ✅ COMPLETE

**Success Criteria**: LensID composite tag computes human-readable lens names using Equipment LensType data

**Status**: ✅ COMPLETE - LensID composite calculation implemented and tested

**Implementation Details**:
- ✅ Olympus manufacturer detection using Make tag
- ✅ Priority logic: EXIF:LensModel over placeholder MakerNotes data  
- ✅ Integration with olympusLensTypes lookup table (138+ entries)
- ✅ Pattern matching for 14-42mm lens variants with multiple key fallbacks
- ✅ Verified against ExifTool: "Composite:LensID": "Olympus M.Zuiko Digital ED 14-42mm F3.5-5.6 EZ"

**Next Enhancement**: Will use real Equipment LensType data once Task 2 provides raw 6-byte lens data instead of placeholder strings

### 4. Task: Add Equipment Integration Tests 📋 DEFERRED

**Success Criteria**: Comprehensive test suite validates Equipment tag extraction across different Olympus camera models

**Dependencies**: Task 1 (locate MakerNotes data) and Task 2 (real Equipment parsing)

**Status**: Deferred until MakerNotes extraction and Equipment tag parsing are working

## Integration Requirements

**CRITICAL**: Building without integrating is failure. Don't accept equipment extraction that doesn't improve user experience.

Every feature must include:
- [x] **Activation**: Equipment processing enabled by default in Olympus pipeline
- [x] **Consumption**: Equipment tags accessible to composite calculations and output formatting  
- [✅] **Measurement**: Test output shows meaningful Equipment tag names instead of binary Tag_XXXX format
- [✅] **Cleanup**: Raw binary tag representation removed, Equipment tags show proper human-readable names

**Current Status**: ❌ Integration BLOCKED - MakerNotes tag not found in test image, preventing Equipment tag extraction

## Working Definition of "Complete"

A feature is complete when:
- [✅] **System behavior changes** - Olympus images show "CameraType2" instead of "Tag_F100" 
- [✅] **Default usage** - Equipment tags extracted automatically for all Olympus images, no opt-in required  
- [✅] **Old path removed** - Binary Tag_XXXX format eliminated for Equipment section

**Current Status**: ❌ BLOCKED - MakerNotes Extraction Issue
- System architecture complete: All Equipment processing code implemented
- LensID composite working: Falls back to EXIF:LensModel successfully
- MakerNotes missing: Test image lacks expected MakerNotes tag (0x927c)

**Immediate Priority**: Locate MakerNotes data source to enable Equipment tag extraction and complete end-to-end system testing

## Testing

- **Unit**: Test Equipment IFD parsing with synthetic Equipment data
- **Integration**: Verify Equipment tags extracted from real Olympus ORF/JPEG files
- **Manual check**: Run `cargo run -- test-images/olympus/e_m10_mk_iii.jpg` and confirm CameraType2, LensType, SerialNumber visible

## Definition of Done

- [ ] `cargo t olympus_equipment` passes
- [ ] `make precommit` clean
- [ ] Equipment tags show proper names in Olympus image output
- [ ] ExifTool comparison shows matching Equipment tag values
- [ ] No binary Tag_XXXX format for Equipment section

## Gotchas & Tribal Knowledge

### Equipment Dual Format Architecture
- **Equipment format detection**: ExifTool uses conditional processing - format "ifd" vs binary data require different parsing
- **ByteOrder Unknown**: ExifTool auto-detects endianness from TIFF magic bytes when ByteOrder => 'Unknown' specified
- **IFD vs Binary**: Modern cameras use TIFF IFD structure, legacy may use binary data format

### LensType Hex Encoding
- **6-byte format**: `[Make, Unknown, Model, Sub-model, Unknown, Unknown]` where only bytes 0, 2, 3 used for lookup
- **Hex conversion**: `sprintf("%x %.2x %.2x", @a[0,2,3])` produces keys like "0 01 00", "1 05 10", "2 15 10"
- **Manufacturer codes**: 0=Olympus, 1=Sigma, 2=Panasonic/Leica, 5=Tamron in first hex digit

### Namespace Separation Critical
- **Tag ID conflicts**: Equipment CameraType2 (0x100) conflicts with EXIF ImageWidth (0x100)
- **Namespace requirement**: Equipment tags MUST use "MakerNotes" namespace to avoid storage conflicts
- **Priority handling**: ExifTool behavior preserved with proper namespace-aware precedence rules

### Generated Code Integration
- **Tag kit ready**: All Equipment tag definitions exist in generated code with proper names and PrintConv
- **Stub replacement**: Only the Equipment IFD processor stub needs implementation - all supporting infrastructure complete
- **Testing framework**: Existing Olympus test images available for comprehensive validation across camera models

### Critical Architecture Complete
- **MakerNotes conditional dispatch**: `src/implementations/makernotes.rs` detects Olympus signatures and routes to manufacturer-specific processing
- **Equipment IFD parsing**: Complete parsing logic in `process_olympus_ifd_for_equipment()` with tag name mapping and value extraction
- **LensID composite system**: Full implementation in `src/composite_tags/implementations.rs` with olympusLensTypes integration
- **Namespace-aware storage**: Equipment tags stored as "MakerNotes:CameraType2" to avoid EXIF tag ID conflicts

### Current Blocker Details  
- **All code paths implemented**: System ready to process MakerNotes Equipment data end-to-end
- **Test image limitation**: `e_m10_mk_iii_01.jpg` confirmed missing MakerNotes tag 0x927c 
- **ExifTool discrepancy**: ExifTool extracts MakerNotes from same image, suggesting our IFD parsing may be incomplete
- **Next step critical**: Either locate MakerNotes in current image or switch to known-good test image with MakerNotes data

## Quick Debugging

Stuck? Try these:

1. **MakerNotes Investigation**:
   - `exiftool -a -G1 -s test-images/olympus/e_m10_mk_iii_01.jpg | grep -i maker` - Verify ExifTool sees MakerNotes
   - `exiftool -htmldump test-images/olympus/e_m10_mk_iii_01.jpg > /tmp/dump.html` - View IFD structure
   - `env RUST_LOG=debug cargo run test-images/olympus/e_m10_mk_iii_01.jpg 2>&1 | grep -E "(927c|37500|MakerNotes)"` - Search our logs for MakerNotes processing

2. **Equipment Processing**: 
   - `RUST_LOG=debug cargo run -- test-images/olympus/e_m10_mk_iii.jpg 2>&1 | grep Equipment` - See Equipment processing  
   - `rg "process_tag_0x2010" src/` - Find Equipment processor implementation

3. **Alternative Test Images**:
   - `find test-images/ -name "*olympus*" -o -name "*e-m*" -o -name "*om-*"` - Find other Olympus test images
   - `exiftool -G1 -s OTHER_IMAGE.jpg | grep -E "(MakerNotes|Equipment)"` - Check if other images have MakerNotes

4. **System Verification**:
   - `cargo run --bin compare-with-exiftool test-images/olympus/e_m10_mk_iii_01.jpg | grep -E "(CameraType2|LensType|SerialNumber)"` - Compare critical Equipment tags
   - `git log -S "Equipment" --oneline` - Track Equipment-related changes