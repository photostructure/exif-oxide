# Technical Project Plan: Sony Required Tags Integration

## Project Overview

- **Goal**: Integrate existing Sony tag_kit system with runtime extraction to enable SonyExposureTime, SonyFNumber, and SonyISO tags
- **Problem**: Complete Sony infrastructure exists but is disconnected from main processing pipeline
- **Constraints**: Must implement ExifTool-compatible encryption without changing ExifTool's logic

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

- **Sony tag_kit system**: Comprehensive tag extraction from ExifTool Sony.pm with 600+ tags across 9 categories (camera, color, core, datetime, etc.)
- **Subdirectory processing**: Generic binary data extraction system with Sony-specific functions that exist but aren't wired to main pipeline
- **ExifTool integration**: Complete codegen infrastructure that has successfully extracted all Sony metadata structures

### Key Interactions

- **Tag_kit → Runtime**: Generated tag definitions reference value conversion functions that don't exist yet
- **Subdirectory processing → Main pipeline**: Sony function exists but is never called during EXIF extraction
- **Encryption → Binary data**: ExifTool has two encryption algorithms (simple substitution + LFSR) that need Rust implementation

### Key Concepts & Domain Knowledge

- **Sony MakerNotes encryption**: Two-tier system - simple substitution cipher for 0x94xx tags, complex LFSR for SR2SubIFD
- **Binary data extraction**: Sony uses ProcessBinaryData tables extensively - Tag2010 variants (a-j) and Tag9050 variants (a-d) contain the required tags
- **Value conversion formulas**: Sony uses specific mathematical formulas for exposure calculations that differ from standard EXIF

### Surprising Context

- **Infrastructure already exists**: 95% of Sony support is already implemented through codegen - just needs integration
- **Only 3 tags currently extracted**: Despite having hundreds of tags available, main pipeline only processes 3 basic Sony tags
- **Subdirectory processing works**: The generic subdirectory system successfully processes Canon/Nikon but Sony integration was never completed
- **ExifTool provides exact algorithms**: Both encryption functions are fully documented with hardcoded translation tables
- **Tag_kit references missing functions**: Generated code calls `sony_exposure_time_value_conv` but function doesn't exist

### Foundation Documents

- **ExifTool source**: `/third-party/exiftool/lib/Image/ExifTool/Sony.pm` lines 11343-11419 contain complete encryption implementation
- **Generated metadata**: `/src/generated/Sony_pm/tag_kit/` contains all extracted Sony tag definitions
- **Sony module overview**: `/third-party/exiftool/doc/modules/Sony.md` explains encryption system and processing flow
- **Start here**: `/src/implementations/sony/mod.rs` - subdirectory processing function exists but isn't called

### Prerequisites

- **Knowledge assumed**: Understanding of TIFF/EXIF structure, binary data processing, and ExifTool processing flow
- **Setup required**: Test Sony ARW/JPEG files available in `test-images/sony/` directory

**Context Quality Check**: Can a new engineer understand WHY this approach is needed after reading this section?

## Work Completed

- ✅ **Complete tag_kit extraction** → Sony.pm fully processed with 600+ tags across 9 semantic categories
- ✅ **Subdirectory processing function** → `process_sony_subdirectory_tags()` implemented using generic system
- ✅ **Sony tag naming integration** → Tag ID to name lookup working ("Sony:AFType" vs "Tag_927C")
- ✅ **Binary data table generation** → CameraSettings, ShotInfo, Tag2010 variants all detected and configured
- ✅ **MakerNotes namespace** → Sony tags correctly assigned to "MakerNotes" group
- ✅ **Test infrastructure** → Multiple Sony ARW/JPEG files available for validation
- ✅ **ExifTool encryption algorithms documented** → Both Decipher() and Decrypt() functions fully mapped

## Remaining Tasks

### 1. Task: Wire Sony subdirectory processing into main extraction pipeline

**Success Criteria**: `process_sony_subdirectory_tags()` is called during Sony MakerNotes processing and extracts binary data
**Approach**: Integrate Sony subdirectory processing call into main EXIF processing flow where other manufacturers are handled
**Dependencies**: None - function exists and works

**Success Patterns**:
- ✅ Sony subdirectory processing called during main extraction
- ✅ Binary data from Sony MakerNotes directories gets processed
- ✅ Debug logging shows "Sony subdirectory processing completed"

### 2. Task: Implement Sony encryption algorithms from ExifTool

**Success Criteria**: Rust implementations of `Decipher()` and `Decrypt()` functions that match ExifTool behavior exactly
**Approach**: Translate ExifTool Sony.pm lines 11343-11419 to Rust, preserving exact algorithms including hardcoded translation tables
**Dependencies**: None - ExifTool source provides complete implementation

**Success Patterns**:
- ✅ Simple substitution cipher working for 0x94xx tags (uses hardcoded translation table)
- ✅ LFSR-based decryption working for SR2SubIFD data (complex 127-pad array algorithm)
- ✅ Encrypted binary data successfully decrypted and processed

### 3. Task: Create missing Sony value conversion functions

**Success Criteria**: `sony_exposure_time_value_conv` and `sony_fnumber_value_conv` functions exist and produce ExifTool-compatible values
**Approach**: Implement value conversion formulas found in ExifTool Sony.pm for ExposureTime and FNumber calculations
**Dependencies**: Must examine actual ExifTool ValueConv expressions for Sony tags

**Success Patterns**:
- ✅ ExposureTime values calculated using Sony-specific formula
- ✅ FNumber values calculated using Sony-specific formula  
- ✅ Generated tag_kit code can successfully call these functions

### 4. Task: Complete binary data extraction for int16u formats

**Success Criteria**: Binary data processors extract actual int16u values instead of showing "TODO: Handle format int16u"
**Approach**: Implement int16u reading in binary data processing with proper byte order handling
**Dependencies**: Encryption implementation (some binary data is encrypted)

**Success Patterns**:
- ✅ Tag2010 and Tag9050 variants extract actual numeric values
- ✅ SonyExposureTime, SonyFNumber, SonyISO tags appear in extraction output
- ✅ Values match ExifTool output for same files

### 5. RESEARCH: Validate Sony value conversion formulas in ExifTool source

**Objective**: Find exact ValueConv expressions for Sony ExposureTime/FNumber tags in Sony.pm
**Success Criteria**: Document actual formulas used by ExifTool for Sony-specific calculations
**Done When**: Value conversion formulas identified and documented with ExifTool source line references

**Task Quality Check**: Can another engineer pick up any task and complete it without asking clarifying questions?

## Implementation Guidance

### Recommended Patterns

- **Encryption implementation**: Use ExifTool's exact translation table approach - hardcoded byte arrays for performance
- **Value conversion functions**: Follow same pattern as existing Canon/Nikon value conversion functions in `src/implementations/value_conv.rs`
- **Binary data processing**: Leverage existing int16u reading patterns from Canon/Nikon processors
- **Integration approach**: Mirror Canon subdirectory processing integration in main EXIF pipeline

### Tools to Leverage

- **Compare-with-exiftool binary**: Use for validation - compares normalized values to avoid formatting differences
- **Generated tag_kit definitions**: All Sony metadata structures already extracted - just need runtime integration
- **Existing subdirectory processing system**: Generic system already handles Canon/Nikon - Sony just needs wiring
- **Test image collection**: Comprehensive Sony ARW/JPEG files for validation across different camera models

### Architecture Considerations

- **Don't modify generated code**: All changes go in `src/implementations/` - never edit `src/generated/`
- **Preserve ExifTool compatibility**: Value output must match ExifTool exactly (use same formulas)
- **Follow encryption patterns**: ExifTool has two distinct algorithms - implement both exactly as specified
- **Binary data safety**: Ensure proper bounds checking when reading encrypted binary data

### Performance Notes

- **Encryption overhead**: Simple substitution cipher is fast, LFSR decryption is more complex but still efficient
- **Tag_kit lookup**: Generated HashMap lookups are O(1) - no performance concerns
- **Binary data processing**: Most Sony cameras have <100KB of MakerNotes data - processing is fast

### ExifTool Translation Notes

- **Preserve exact algorithms**: Don't "optimize" ExifTool's encryption - it handles real-world camera quirks
- **Use hardcoded translation tables**: ExifTool provides 246-byte translation table - copy exactly
- **Maintain byte order awareness**: Sony uses little-endian but this varies by data structure
- **Handle encryption variants**: Different camera models use different encryption keys/methods

## Integration Requirements

**CRITICAL**: Building without integrating is failure. Don't accept tasks that build "shelf-ware."

Every feature must include:
- [ ] **Activation**: Sony subdirectory processing enabled by default in main extraction pipeline
- [ ] **Consumption**: Encrypted binary data is automatically decrypted and processed during normal EXIF extraction
- [ ] **Measurement**: Can verify Sony tag extraction by comparing tag count before/after integration
- [ ] **Cleanup**: Remove "TODO: Handle format int16u" comments, replace with actual value extraction

**Red Flag Check**: If a task seems like "build encryption functions but don't use them," ask for clarity. We're not writing tools to sit on a shelf - everything must get us closer to "ExifTool in Rust for PhotoStructure."

## Working Definition of "Complete"

*Use these criteria to evaluate your own work - adapt to your specific context:*

A feature is complete when:
- ✅ **System behavior changes** - Sony tag extraction increases from 3 to 100+ tags
- ✅ **Default usage** - Sony MakerNotes processing happens automatically during normal extraction
- ✅ **Old path removed** - "TODO" comments eliminated, actual value extraction implemented
- ❌ Code exists but isn't used *(example: "encryption functions implemented but subdirectory processing still disabled")*
- ❌ Feature works "if you call it directly" *(example: "Sony functions exist but main pipeline doesn't call them")*

*Note: These are evaluation guidelines, not literal requirements for every task.*

## Prerequisites

- Sony tag_kit system → P13-sony-required-tags → verify with `ls src/generated/Sony_pm/tag_kit/`
- Generic subdirectory processing → [CORE-ARCHITECTURE.md](../guides/CORE-ARCHITECTURE.md) → verify with existing Canon integration

## Testing

- **Unit**: Test encryption/decryption functions with known binary data samples
- **Integration**: Verify Sony tag extraction on ARW/JPEG files from different camera models
- **Manual check**: Run `cargo run --bin exif-oxide test-images/sony/a7_iii.arw` and confirm SonyExposureTime, SonyFNumber, SonyISO appear

## Definition of Done

- [ ] `cargo t sony` passes (if Sony-specific tests exist)
- [ ] `make precommit` clean
- [ ] SonyExposureTime, SonyFNumber, SonyISO tags appear in extraction output
- [ ] Sony tag count increases from 3 to 100+ tags
- [ ] ExifTool compatibility validated with compare-with-exiftool tool

## Additional Gotchas & Tribal Knowledge

**Format**: Surprise → Why → Solution (Focus on positive guidance)

- **Sony subdirectory processing exists but isn't called** → Integration was never completed → Wire into main EXIF processing pipeline
- **Tag_kit references missing functions** → Value conversion functions never implemented → Create referenced functions in value_conv.rs
- **Generated binary data says "TODO"** → Binary data extraction incomplete → Implement int16u reading with proper byte order
- **ExifTool has exact encryption algorithms** → Just need Rust translation → Copy hardcoded translation tables and algorithms exactly
- **Only 3 Sony tags extracted despite hundreds available** → Main pipeline bypasses Sony-specific processing → Enable subdirectory processing integration
- **Encryption looks complex but isn't** → ExifTool provides complete implementation → Two functions: simple substitution + LFSR algorithm

**Note**: Most gotchas should be captured in the "Surprising Context" section above.

## Quick Debugging

Stuck? Try these:

1. `cargo run --bin exif-oxide test-images/sony/a7_iii.arw | grep Sony | wc -l` - Count current Sony tags
2. `rg "process_sony_subdirectory" src/` - Check if Sony subdirectory processing is called
3. `rg "sony_exposure_time_value_conv" src/implementations/` - Verify value conversion functions exist
4. `exiftool -j -struct -G test-images/sony/a7_iii.arw | jq 'keys' | grep -i sony` - Compare with ExifTool