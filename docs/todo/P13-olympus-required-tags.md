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

## Remaining Tasks

### 1. Task: Implement Equipment IFD Parser

**Success Criteria**: `process_tag_0x2010_subdirectory` parses TIFF IFD structure and extracts Equipment tags with proper tag IDs and names

**Approach**: 
- Replace stub implementation with TIFF IFD parsing logic
- Handle ExifTool's dual binary/IFD format detection
- Extract individual Equipment tags (CameraType2, SerialNumber, LensType) with proper tag IDs

**Dependencies**: None - all infrastructure in place

**Success Patterns**:
- ✅ Equipment tags extracted with proper names ("CameraType2", "LensType", "SerialNumber")
- ✅ Raw binary format (Tag_0100, Tag_0101, Tag_0201) replaced with meaningful names
- ✅ Test shows 6+ Equipment tags instead of 0

### 2. Task: Connect Equipment Tag Name Resolution

**Success Criteria**: Equipment tag IDs correctly resolve to proper names using generated tag kit definitions

**Approach**:
- Link Equipment IFD tag extraction to OLYMPUS_PM_TAG_KITS lookup
- Ensure Equipment tags use "MakerNotes" namespace for proper conflict resolution
- Apply PrintConv processing for camera type and lens type lookups

**Dependencies**: Task 1 (Equipment IFD Parser)

**Success Patterns**:
- ✅ CameraType2 shows "E-M10MarkIII" instead of binary data
- ✅ SerialNumber shows "BHXA00022" instead of Tag_0101
- ✅ LensType shows hex format "0 21 10" ready for olympusLensTypes lookup

### 3. Task: Implement LensID Composite Calculation

**Success Criteria**: LensID composite tag computes human-readable lens names using Equipment LensType data

**Approach**: 
- Add LensID to composite tag dispatcher using Equipment:LensType dependency
- Implement lens lookup logic matching ExifTool's olympusLensTypes conversion
- Handle LensType hex format (6 bytes → "0 21 10" → "Olympus M.Zuiko Digital ED 14-42mm F3.5-5.6 EZ")

**Dependencies**: Task 2 (Equipment Tag Name Resolution)

**Success Patterns**:
- ✅ LensID composite shows full lens name from olympusLensTypes database
- ✅ Handles both Olympus and third-party lenses (Sigma, Panasonic/Leica, Tamron)
- ✅ Fallback to raw LensType value for unknown lenses

### 4. Task: Add Equipment Integration Tests

**Success Criteria**: Comprehensive test suite validates Equipment tag extraction across different Olympus camera models

**Approach**:
- Create tests using existing Olympus test images (E-M10 Mark III, OM-5, TG-7)
- Validate critical Equipment tags are extracted with proper names and values
- Compare output with ExifTool for accuracy verification

**Dependencies**: Task 2 (Equipment Tag Name Resolution)

**Success Patterns**:
- ✅ Tests pass with 6+ Equipment tags extracted for each camera model
- ✅ Values match ExifTool output for camera identification
- ✅ No regressions in existing EXIF tag extraction

## Integration Requirements

**CRITICAL**: Building without integrating is failure. Don't accept equipment extraction that doesn't improve user experience.

Every feature must include:
- [x] **Activation**: Equipment processing enabled by default in Olympus pipeline
- [x] **Consumption**: Equipment tags accessible to composite calculations and output formatting  
- [x] **Measurement**: Test output shows meaningful Equipment tag names instead of binary Tag_XXXX format
- [x] **Cleanup**: Raw binary tag representation removed, Equipment tags show proper human-readable names

**Red Flag Check**: If Equipment tags still show as "Tag_0100" or binary data, the integration is incomplete.

## Working Definition of "Complete"

A feature is complete when:
- ✅ **System behavior changes** - Olympus images show "LensType" instead of "Tag_0201"
- ✅ **Default usage** - Equipment tags extracted automatically for all Olympus images, no opt-in required  
- ✅ **Old path removed** - Binary Tag_XXXX format eliminated for Equipment section
- ❌ Code exists but shows raw binary (example: "Equipment extraction implemented but still shows Tag_0100")
- ❌ Feature works "if you look carefully" (example: "Equipment data available but not in main output")

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

## Quick Debugging

Stuck? Try these:

1. `RUST_LOG=debug cargo run -- test-images/olympus/e_m10_mk_iii.jpg 2>&1 | grep Equipment` - See Equipment processing
2. `cargo run --bin compare-with-exiftool test-images/olympus/e_m10_mk_iii.jpg | grep -E "(CameraType2|LensType|SerialNumber)"` - Compare critical Equipment tags
3. `rg "process_tag_0x2010" src/` - Find Equipment processor implementation
4. `git log -S "Equipment" --oneline` - Track Equipment-related changes