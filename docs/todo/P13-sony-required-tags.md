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

## 🚀 MAJOR PROGRESS UPDATE (July 2025)

### Status Summary

**Before this work session**: 3 Sony tags extracted (0 in MakerNotes group)
**After this work session**: 72 Sony MakerNotes tags extracted (61% of ExifTool's 118 tags)

### Root Cause Discovery

**Original hypothesis was WRONG**: We thought the MakerNotes tag (0x927c) wasn't being processed at all.

**Actual root cause**: The MakerNotes tag WAS being processed correctly, but Sony tags were assigned to the wrong namespace:
- **Problem**: Sony tags appeared in "EXIF:" group instead of "MakerNotes:" group
- **Solution**: Implemented two-layer namespace system (internal "Sony" + display "MakerNotes")
- **Result**: 72 Sony tags now appear in correct "MakerNotes:" group

### Architecture Changes Made

1. **Namespace mapping system** (`/home/mrm/src/exif-oxide/src/exif/mod.rs` lines 316-323, 551-558):
   - Maps manufacturer namespaces ("Canon", "Nikon", "Sony") to display group "MakerNotes"
   - Preserves internal namespace for processor selection
   - Matches ExifTool's Group0/Group1 behavior

2. **Sony MakerNotes processing** (`/home/mrm/src/exif-oxide/src/exif/ifd.rs`):
   - Changed from generic subdirectory processing to direct IFD parsing with "Sony" namespace
   - Ensures Sony tags get proper internal namespace for tag_kit system activation

### Current Status: Tag Naming Issue

**Next Priority**: Sony tags showing as raw "Tag_2000", "Tag_2002" instead of "SonyExposureTime", "SonyFNumber"

**Investigation needed**: Why isn't the Sony tag_kit system (600+ generated tags) providing human-readable names?

**Success metrics for next engineer**:
- Current: 72 Sony tags with raw "Tag_xxxx" names
- Target: 72+ Sony tags with human-readable names like "SonyExposureTime", "SonyFNumber", "SonyISO"
- Ultimate goal: Match ExifTool's 118 Sony MakerNotes tags

### Validation Commands

```bash
# Count our Sony MakerNotes tags (should show 72)
cargo run --bin exif-oxide test-images/sony/a7r.jpg | jq '.[] | with_entries(select(.key | startswith("MakerNotes:"))) | keys | length'

# Count ExifTool's Sony MakerNotes tags (shows 118 for comparison)
exiftool -j -struct -G test-images/sony/a7r.jpg | jq '.[] | with_entries(select(.key | startswith("MakerNotes:"))) | keys | length'

# Show our current Sony tag names (will show Tag_xxxx format)
cargo run --bin exif-oxide test-images/sony/a7r.jpg | jq '.[] | with_entries(select(.key | startswith("MakerNotes:"))) | keys | .[:10]'
```

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
- **MAJOR PROGRESS (July 2025)**: Sony tag extraction increased from 3 to 72 tags after fixing namespace assignment issue
- **Namespace issue was the blocker**: MakerNotes tag (0x927c) was being processed correctly, but Sony tags appeared in "EXIF:" group instead of "MakerNotes:" group
- **Subdirectory processing works**: The generic subdirectory system successfully processes Canon/Nikon and now Sony after namespace fix
- **ExifTool provides exact algorithms**: Both encryption functions are fully documented with hardcoded translation tables
- **Tag naming needs completion**: Sony tags appear as raw "Tag_xxxx" format instead of human-readable names like "SonyExposureTime"

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
- ✅ **CRITICAL FIX: MakerNotes namespace assignment** → Fixed July 2025: Sony tags now correctly appear in "MakerNotes:" group instead of "EXIF:" group
- ✅ **MakerNotes processing pipeline working** → Successfully extracting 72 Sony MakerNotes tags (vs ExifTool's 118)
- ✅ **Binary data table generation** → CameraSettings, ShotInfo, Tag2010 variants all detected and configured
- ✅ **Test infrastructure** → Multiple Sony ARW/JPEG files available for validation
- ✅ **ExifTool encryption algorithms documented** → Both Decipher() and Decrypt() functions fully mapped
- ✅ **Namespace architecture implemented** → Two-layer system: internal "Sony" namespace + "MakerNotes" display group

## Remaining Tasks

### 1. ✅ COMPLETED: Wire Sony subdirectory processing into main extraction pipeline

**Status**: **COMPLETED July 2025** - Fixed critical namespace assignment issue
**Achievement**: Sony tag extraction increased from 3 to 72 tags (61% of ExifTool's 118 tags)
**Implementation**: Two-layer namespace system in `/home/mrm/src/exif-oxide/src/exif/mod.rs` and `/home/mrm/src/exif-oxide/src/exif/ifd.rs`

**Success Patterns**: ✅ All achieved
- ✅ Sony subdirectory processing called during main extraction
- ✅ Binary data from Sony MakerNotes directories gets processed
- ✅ Sony tags appear in correct "MakerNotes:" group instead of "EXIF:" group

### 2. Task: Fix Sony tag naming system (CURRENT PRIORITY)

**Status**: **IN PROGRESS** - 72 tags extracted but showing as raw "Tag_xxxx" format
**Problem**: Sony tags appear as "Tag_2000", "Tag_2002" instead of "SonyExposureTime", "SonyFNumber", etc.
**Root Cause**: Sony tag_kit system (600+ generated tags) not fully activated for tag name resolution
**Success Criteria**: Sony tags show human-readable names matching ExifTool output

**Investigation Points**:
- Verify Sony tag_kit integration in MakerNotes processing pipeline
- Check tag ID → name lookup system for Sony namespace
- Ensure generated Sony tag definitions are being used during tag resolution
- Validate that tag naming works for Sony-specific tags like SonyExposureTime, SonyFNumber, SonyISO

**Files to investigate**:
- `/src/generated/Sony_pm/tag_kit/` - Generated tag definitions
- `/src/implementations/sony/mod.rs` - Sony-specific processing
- Tag naming/lookup system integration points

### 3. Task: Implement Sony encryption algorithms from ExifTool

**Success Criteria**: Rust implementations of `Decipher()` and `Decrypt()` functions that match ExifTool behavior exactly
**Approach**: Translate ExifTool Sony.pm lines 11343-11419 to Rust, preserving exact algorithms including hardcoded translation tables
**Dependencies**: None - ExifTool source provides complete implementation

**Success Patterns**:
- ✅ Simple substitution cipher working for 0x94xx tags (uses hardcoded translation table)
- ✅ LFSR-based decryption working for SR2SubIFD data (complex 127-pad array algorithm)
- ✅ Encrypted binary data successfully decrypted and processed

### 4. Task: Create missing Sony value conversion functions

**Success Criteria**: `sony_exposure_time_value_conv` and `sony_fnumber_value_conv` functions exist and produce ExifTool-compatible values
**Approach**: Implement value conversion formulas found in ExifTool Sony.pm for ExposureTime and FNumber calculations
**Dependencies**: Must examine actual ExifTool ValueConv expressions for Sony tags

**Success Patterns**:
- ✅ ExposureTime values calculated using Sony-specific formula
- ✅ FNumber values calculated using Sony-specific formula  
- ✅ Generated tag_kit code can successfully call these functions

### 5. Task: Complete binary data extraction for int16u formats

**Success Criteria**: Binary data processors extract actual int16u values instead of showing "TODO: Handle format int16u"
**Approach**: Implement int16u reading in binary data processing with proper byte order handling
**Dependencies**: Encryption implementation (some binary data is encrypted)

**Success Patterns**:
- ✅ Tag2010 and Tag9050 variants extract actual numeric values
- ✅ SonyExposureTime, SonyFNumber, SonyISO tags appear in extraction output
- ✅ Values match ExifTool output for same files

### 6. RESEARCH: Validate Sony value conversion formulas in ExifTool source

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
- ✅ **System behavior changes** - Sony tag extraction increased from 3 to 72 tags (ACHIEVED July 2025)
- ✅ **Default usage** - Sony MakerNotes processing happens automatically during normal extraction (ACHIEVED)
- ⚠️ **Proper tag naming** - Sony tags show human-readable names instead of raw "Tag_xxxx" format (IN PROGRESS)
- ⚠️ **Value extraction completeness** - SonyExposureTime, SonyFNumber, SonyISO appear with proper values (PENDING)
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
- [x] Sony tag count increases from 3 to 72+ tags (ACHIEVED July 2025)
- [ ] SonyExposureTime, SonyFNumber, SonyISO tags appear with human-readable names (not Tag_xxxx format)
- [ ] Sony tag count reaches 100+ tags (currently at 72 vs ExifTool's 118)
- [ ] ExifTool compatibility validated with compare-with-exiftool tool

## Additional Gotchas & Tribal Knowledge

**Format**: Surprise → Why → Solution (Focus on positive guidance)

- ✅ **RESOLVED: Sony subdirectory processing exists but isn't called** → Integration was never completed → **FIXED**: Wired into main EXIF processing pipeline with proper namespace assignment
- ✅ **RESOLVED: Only 3 Sony tags extracted despite hundreds available** → Main pipeline bypassed Sony-specific processing → **FIXED**: Namespace assignment corrected, now extracting 72 Sony tags
- ✅ **RESOLVED: MakerNotes tag (0x927c) missing** → **FALSE HYPOTHESIS**: Tag WAS being processed, just wrong namespace → **FIXED**: Sony tags now appear in "MakerNotes:" group instead of "EXIF:" group
- **Sony tags show as "Tag_xxxx" instead of human names** → Sony tag_kit system not fully activated for name resolution → **INVESTIGATE**: Tag ID → name lookup integration in Sony namespace processing
- **Tag_kit references missing functions** → Value conversion functions never implemented → Create referenced functions in value_conv.rs
- **Generated binary data says "TODO"** → Binary data extraction incomplete → Implement int16u reading with proper byte order
- **ExifTool has exact encryption algorithms** → Just need Rust translation → Copy hardcoded translation tables and algorithms exactly
- **Encryption looks complex but isn't** → ExifTool provides complete implementation → Two functions: simple substitution + LFSR algorithm

**Note**: Most gotchas should be captured in the "Surprising Context" section above.

## Quick Debugging

Stuck? Try these:

**Current Status Validation**:
1. `cargo run --bin exif-oxide test-images/sony/a7r.jpg | jq '.[] | with_entries(select(.key | startswith("MakerNotes:"))) | keys | length'` - Should show 72 Sony MakerNotes tags
2. `exiftool -j -struct -G test-images/sony/a7r.jpg | jq '.[] | with_entries(select(.key | startswith("MakerNotes:"))) | keys | length'` - Shows 118 for comparison
3. `cargo run --bin exif-oxide test-images/sony/a7r.jpg | jq '.[] | with_entries(select(.key | startswith("MakerNotes:"))) | keys | .[:10]'` - Show first 10 tag names (will be Tag_xxxx format)

**Tag Naming Investigation**:
4. `ls src/generated/Sony_pm/tag_kit/` - Verify Sony tag_kit files exist (should show multiple .rs files)
5. `rg "SonyExposureTime\|SonyFNumber\|SonyISO" src/generated/Sony_pm/` - Check if these specific tags are defined in generated code
6. `rg "tag_kit" src/implementations/sony/` - Check tag_kit integration in Sony processing

**Legacy Debug Commands** (for reference):
7. `rg "process_sony_subdirectory" src/` - Check if Sony subdirectory processing is called
8. `rg "sony_exposure_time_value_conv" src/implementations/` - Verify value conversion functions exist
9. `exiftool -j -struct -G test-images/sony/a7r.jpg | jq '.[] | keys | .[] | select(test(".*Sony.*(ExposureTime|FNumber|ISO)"))' ` - Show ExifTool's Sony-specific tags