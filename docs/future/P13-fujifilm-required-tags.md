# Technical Project Plan: Fujifilm Required Tags Implementation

## Project Overview

- **Goal**: Implement comprehensive Fujifilm MakerNotes tag extraction to support all required tags from supported_tags.json
- **Problem**: Basic infrastructure exists (codegen, model detection) but no MakerNotes processing implementation, missing all Fujifilm-specific tags
- **Constraints**: Must follow ExifTool's FujiFilm.pm processing exactly, support X-series mirrorless and medium format GFX cameras

---

## ⚠️ CRITICAL REMINDERS

If you read this document, you **MUST** read and follow [CLAUDE.md](../CLAUDE.md) as well as [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md):

- **Trust ExifTool** (Translate and cite references, but using codegen is preferred)
- **Ask clarifying questions** (Maximize "velocity made good")
- **Assume Concurrent Edits** (STOP if you find a compilation error that isn't related to your work)
- **Don't edit generated code** (read [CODEGEN.md](CODEGEN.md) if you find yourself wanting to edit `src/generated/**.*rs`)
- **Keep documentation current** (so update this TPP with status updates, and any novel context that will be helpful to future engineers)

**NOTE**: These rules prevent build breaks, data corruption, and wasted effort across the team. 

If you are found violating any topics in these sections, **your work will be immediately halted, reverted, and you will be dismissed from the team.**

Honest. RTFM.

---

## Context & Foundation

### System Overview

- **Codegen infrastructure**: Basic FujiFilm.pm extraction exists with main_model_detection and ffmv_binary_data modules, but missing comprehensive tag kit system
- **Market significance**: Fujifilm holds 5.8% camera market share, popular for X-series mirrorless (X-T5, X-H2) and medium format GFX cameras
- **Current state**: File detection works, RAF format support exists, but zero MakerNotes tag extraction implemented

### Key Concepts & Domain Knowledge

- **Fujifilm MakerNotes structure**: Uses standard TIFF IFD format with manufacturer-specific tags, simpler than Canon/Nikon encryption schemes
- **RAF format support**: Fujifilm's proprietary RAW format, already detected by file_type_lookup but needs MakerNotes processing
- **InternalSerialNumber encoding**: Complex format containing camera model ID, manufacturing date, and unique identifiers (line 94-100 in FujiFilm.pm)
- **Film simulation modes**: Fujifilm's signature color science requires specific tag extraction for film emulation settings

### Surprising Context

- **Minimal complexity**: Unlike Canon/Sony encryption, Fujifilm uses straightforward IFD processing - should be simpler implementation
- **Comprehensive codegen potential**: FujiFilm.pm has clean table structures ideal for tag kit extraction
- **Missing implementation gap**: Infrastructure exists but no actual MakerNotes processor implemented in src/implementations/
- **Market positioning**: Third-largest mirrorless manufacturer after Sony/Canon, significant user base requiring support

### Foundation Documents

- **ExifTool source**: `third-party/exiftool/lib/Image/ExifTool/FujiFilm.pm` - Main tag table starts line 84, key tags through line 500+
- **Generated infrastructure**: `src/generated/FujiFilm_pm/` contains partial extraction but missing tag kit system
- **Market research**: 5.8% global market share, strong in mirrorless segment with X-series and GFX medium format
- **Start here**: Need to create `src/implementations/fujifilm/` following Canon/Sony/Nikon patterns

### Prerequisites

- **Knowledge assumed**: Understanding of TIFF IFD processing, MakerNotes extraction patterns from other manufacturers
- **Setup required**: Fujifilm test images (X-T series, GFX series) for validation

**Context Quality Check**: Can a new engineer understand WHY Fujifilm support is needed and how it fits into the broader manufacturer ecosystem?

## Work Completed

- ✅ **File type detection** → Fujifilm RAF format properly detected in file_type_lookup
- ✅ **Basic codegen infrastructure** → FujiFilm.pm partially extracted with model detection and binary data modules
- ✅ **Format support foundation** → RAF format listed in SUPPORTED-FORMATS.md
- ✅ **MakerNotes conditional dispatch** → Added FujiFilm signature detection ("FUJIFILM"/"GENERALE") to src/implementations/makernotes.rs following ExifTool MakerNotes.pm specification with 8-byte header skip and LittleEndian processing
- ✅ **Tag kit system implementation** → Generated comprehensive FujiFilm tag kit with 100+ tag definitions including PrintConv lookup tables from FujiFilm.pm Main table
- ✅ **IFD parsing integration** → Implemented parse_fujifilm_ifd() with proper tag name resolution using generated tag kit, handles TIFF IFD structure with format-specific value extraction
- ✅ **EXIF pipeline integration** → FujiFilm processor integrated into MakerNotes 0x927C processing chain with proper namespace handling

## Remaining Tasks

### 1. Task: Research current supported_tags.json requirements for Fujifilm

**Success Criteria**: Document all MakerNotes tags in supported_tags.json that should come from Fujifilm cameras
**Approach**: Analyze supported_tags.json for generic MakerNotes tags that Fujifilm should provide (LensModel, InternalSerialNumber, etc.)
**Dependencies**: None - research task

**Success Patterns**:
- ✅ Complete list of required MakerNotes tags documented
- ✅ Priority ranking based on tag frequency and importance
- ✅ Cross-reference with ExifTool FujiFilm.pm tag table

### 2. Task: Implement comprehensive FujiFilm tag kit extraction

**Success Criteria**: Complete tag kit system generated from FujiFilm.pm with all major tag tables
**Approach**: Extend codegen system to extract FujiFilm Main table (line 84+) and subdirectory tables using tag_kit framework
**Dependencies**: Task 1 (requirements analysis)

**Success Patterns**:
- ✅ src/generated/FujiFilm_pm/tag_kit/ directory created with comprehensive tag definitions
- ✅ All major FujiFilm tag tables extracted (Main, FFMV, face recognition)
- ✅ PrintConv expressions properly generated for film simulation modes, scene recognition

### 3. Task: Create Fujifilm MakerNotes processor implementation

**Success Criteria**: `src/implementations/fujifilm/mod.rs` processes Fujifilm MakerNotes using generated tag kit
**Approach**: Follow Canon/Sony/Nikon implementation patterns, integrate with generic subdirectory processing system
**Dependencies**: Task 2 (tag kit extraction)

**Success Patterns**:
- ✅ process_fujifilm_makernotes() function extracts standard TIFF IFD MakerNotes
- ✅ Integration with main EXIF processing pipeline
- ✅ Proper namespace handling for MakerNotes group

### 4. Task: Implement key required tags extraction

**Success Criteria**: Critical tags like InternalSerialNumber, LensModel, FilmMode extracted with ExifTool-compatible values
**Approach**: Focus on high-priority tags from supported_tags.json, implement complex value conversions where needed
**Dependencies**: Task 3 (MakerNotes processor)

**Success Patterns**:
- ✅ InternalSerialNumber properly decoded from complex format (model ID + date + serial)
- ✅ Film simulation modes extracted with human-readable names
- ✅ Lens information tags populated for both native and adapted lenses

### 5. RESEARCH: Validate against popular Fujifilm camera models

**Objective**: Test implementation against X-T5, X-H2, GFX 100S sample files to ensure broad compatibility
**Success Criteria**: Tag extraction works across Fujifilm's current camera lineup
**Done When**: ExifTool comparison shows >90% tag match for supported tags

## Implementation Guidance

### Recommended Patterns

- **Standard IFD processing**: Fujifilm uses clean TIFF IFD structure, no encryption complexity like other manufacturers
- **Tag kit integration**: Leverage generated tag definitions for consistent processing
- **Value conversion focus**: Many Fujifilm tags need human-readable PrintConv (film modes, scene recognition, etc.)

### Tools to Leverage

- **Existing codegen system**: Tag kit extraction framework should handle FujiFilm.pm cleanly
- **Generic subdirectory processing**: Standard TIFF IFD processing utilities already available
- **Generated lookup tables**: Model detection and binary data structures already extracted

### Architecture Considerations

- **Namespace isolation**: Fujifilm MakerNotes must use proper "MakerNotes" group to avoid EXIF conflicts
- **RAF format integration**: Ensure MakerNotes processing works for both JPEG and RAF files
- **Performance**: Fujifilm processing should be lightweight compared to encrypted manufacturers

## Integration Requirements

**CRITICAL**: Building without integrating is failure. Don't accept incomplete Fujifilm support.

Every feature must include:
- [ ] **Activation**: Fujifilm MakerNotes processing enabled by default for Fujifilm cameras
- [ ] **Consumption**: Tags appear in standard extraction output without special flags
- [ ] **Measurement**: Can verify Fujifilm tag extraction with grep on output
- [ ] **Cleanup**: Remove placeholder infrastructure that doesn't extract actual tags

**Red Flag Check**: If Fujifilm images still show zero MakerNotes tags, integration is incomplete.

## Working Definition of "Complete"

A feature is complete when:
- ✅ **System behavior changes** - Fujifilm images now extract MakerNotes tags automatically
- ✅ **Default usage** - No special configuration needed for Fujifilm tag extraction
- ✅ **Old path removed** - Placeholder codegen modules replaced with functional implementation
- ❌ Code exists but extracts zero tags *(example: "processor implemented but no tags appear")*
- ❌ Feature works "if you call it directly" *(example: "tag kit exists but processor doesn't use it")*

## Prerequisites

- EXIF foundation → P10a-exif-required-tags → verify with `cargo t exif_basic`
- Generic subdirectory processing → verify with existing Canon/Sony integration

## Testing

- **Unit**: Test Fujifilm MakerNotes parsing with synthetic IFD data
- **Integration**: Verify tag extraction from X-T5, X-H2, GFX sample files  
- **Manual check**: Run `cargo run test-images/fujifilm/x-t5.raf` and confirm MakerNotes tags appear

## Definition of Done

- [ ] `cargo t fujifilm` passes (if Fujifilm-specific tests exist)
- [ ] `make precommit` clean
- [ ] Fujifilm MakerNotes tags appear in extraction output
- [ ] ExifTool comparison shows matching values for critical tags
- [ ] No regression in existing EXIF tag extraction

## Gotchas & Tribal Knowledge

### Fujifilm-Specific Considerations

- **RAF vs JPEG processing**: Same MakerNotes structure in both formats - processor should handle both transparently
- **Film simulation complexity**: Film modes require specific PrintConv lookups for human-readable names
- **InternalSerialNumber decoding**: Complex format combining model ID, manufacturing date, and serial number
- **Medium format vs APS-C**: GFX cameras may have different tag layouts than X-series

### Implementation Shortcuts

- **No encryption**: Unlike Canon/Sony, Fujifilm uses standard TIFF IFD - no decryption needed
- **Clean tag tables**: FujiFilm.pm has well-structured tables ideal for codegen extraction
- **Existing infrastructure**: Model detection and basic binary data processing already implemented

## Quick Debugging

Stuck? Try these:

1. `cargo run test-images/fujifilm/sample.raf | grep MakerNotes | wc -l` - Count extracted MakerNotes tags
2. `exiftool -j -G test-images/fujifilm/sample.raf | jq '.[] | keys[]' | grep MakerNotes` - Compare with ExifTool
3. `rg "process_fujifilm" src/` - Check if Fujifilm processor is called
4. `ls src/generated/FujiFilm_pm/tag_kit/` - Verify tag kit extraction completed