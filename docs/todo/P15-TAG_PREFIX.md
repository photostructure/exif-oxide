# P15: Sony TAG_PREFIX Implementation for Unknown Tag Naming

## Project Overview

- **Goal**: Fix Sony TAG_PREFIX mechanism to show "MakerNotes:Sony_0x2000" instead of "MakerNotes:Tag_2000" for unknown tags, completing the TAG_PREFIX infrastructure for all manufacturers
- **Problem**: Sony unknown tags use generic "Tag_XXXX" naming instead of manufacturer-specific "Sony_0xXXXX" naming because multiple hardcoded `format!("Tag_{tag_id:04X}")` fallbacks in the codebase bypass the existing TAG_PREFIX mechanism
- **Constraints**: Must follow ExifTool's TAG_PREFIX behavior exactly, maintain existing namespace assignment patterns

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

**REQUIRED**: Assume reader is unfamiliar with this domain. Provide comprehensive context.

### System Overview

- **TAG_PREFIX mechanism**: ExifTool feature that gives unknown tags manufacturer-specific names (e.g., "Sony_0x2000") instead of generic names (e.g., "Tag_2000"). Implemented in `src/exif/mod.rs:205-229` via `generate_tag_prefix_name()` function
- **Sony binary data processing**: Uses `process_subdirectories_with_printconv()` to extract binary data from Sony MakerNotes subdirectories (Tag2010, Tag9050, AFInfo, etc.), bypassing standard IFD parsing where TAG_PREFIX is applied
- **Namespace assignment**: Tags get namespace identifiers ("EXIF", "GPS", "MakerNotes", "Sony") that determine group prefixes and influence TAG_PREFIX logic

### Key Concepts & Domain Knowledge

- **TAG_PREFIX**: ExifTool.pm:4468-4479 mechanism that adds manufacturer prefix to unknown tag names when no tag definition exists
- **Binary data extraction**: Sony stores metadata in binary chunks that require special ProcessBinaryData handling, not standard IFD tag parsing
- **Subdirectory processing**: Generic system in `src/exif/subdirectory_processing.rs` that extracts tags from binary data using manufacturer-specific functions
- **Unknown tag generation**: When no tag definition exists, system falls back to generic "Tag_XXXX" naming

### Surprising Context

**CRITICAL**: Document non-intuitive aspects that aren't obvious from casual code inspection:

- **TAG_PREFIX infrastructure exists but is bypassed**: The `generate_tag_prefix_name()` function works correctly but is bypassed by 15+ hardcoded `format!("Tag_{tag_id:04X}")` fallbacks throughout the codebase
- **Sony subdirectory processing not triggered**: Debug logs show Sony subdirectory processing isn't being called for the test image, so the subdirectory namespace assignment isn't the primary issue
- **Multiple fallback paths**: Unknown tag naming happens in several code paths (standard IFD parsing, manufacturer-specific lookups, fallback chains) that each have their own hardcoded fallbacks
- **ExifTool shows these tags by default**: The Sony test image unknown tags are visible in standard ExifTool output, not just with `-u` flag

### Foundation Documents

- **Design docs**: [PROCESSOR-DISPATCH.md](guides/PROCESSOR-DISPATCH.md) - processor selection strategy
- **ExifTool source**: 
  - ExifTool.pm:4468-4479 - TAG_PREFIX mechanism
  - Sony.pm ProcessBinaryData tables - binary data extraction logic
- **Start here**: 
  - `src/exif/mod.rs:205-229` - existing TAG_PREFIX implementation
  - `src/implementations/sony/mod.rs:32-50` - Sony subdirectory processing
  - `src/exif/subdirectory_processing.rs` - generic binary data extraction

### Prerequisites

- **Knowledge assumed**: Understanding of ExifTool ProcessBinaryData pattern, TAG_PREFIX mechanism, namespace assignment in IFD parsing
- **Setup required**: Working exif-oxide build, Sony test image at `third-party/exiftool/t/images/Sony.jpg`

## Work Completed

- ✅ **TAG_PREFIX infrastructure** → implemented `generate_tag_prefix_name()` function in `src/exif/mod.rs:205-233` with support for Canon, Sony, Nikon, Olympus, Panasonic, Fujifilm namespaces
- ✅ **DNG module configuration** → generated comprehensive `codegen/config/DNG_pm/tag_kit.json` with 98 extracted tag kits using `tag_kit.pl` extractor
- ✅ **Root cause analysis** → identified 23+ hardcoded `format!("Tag_{tag_id:04X}")` fallbacks across 8 files bypassing TAG_PREFIX mechanism
- ✅ **Behavior validation** → confirmed current output shows 8 Tag_ entries vs ExifTool's expected Sony_0xXXXX format
- ✅ **ExifTool comparison** → verified ExifTool uses "MakerNotes:Sony_0x2000", "MakerNotes:Sony_0x9001", etc. for unknown Sony tags

## TDD Foundation Requirement

### Task 0: Integration Test (TDD Foundation)

**Purpose**: Ensure the TPP solves a real, measurable problem with verifiable success criteria.

**Success Criteria**:
- [ ] **Test exists**: `tests/integration_p15_sony_tag_prefix.rs:test_sony_tag_prefix_behavior` validates Sony TAG_PREFIX behavior
- [ ] **Test fails initially**: `cargo t test_sony_tag_prefix_behavior` fails demonstrating current Tag_XXXX behavior vs expected Sony_0xXXXX
- [ ] **Integration focus**: Test validates end-to-end Sony TAG_PREFIX behavior using `third-party/exiftool/t/images/Sony.jpg`
- [ ] **TPP reference**: Test includes comment `// P15: Sony TAG_PREFIX Implementation - see docs/todo/P15-TAG_PREFIX.md`
- [ ] **Measurable outcome**: Test expects 7 "MakerNotes:Sony_0xXXXX" tags, currently gets 7 "MakerNotes:Tag_XXXX" tags
- [ ] **ExifTool reference**: Test uses ExifTool output (`./exiftool -u -j -G Sony.jpg`) as ground truth for expected tag names

## Remaining Tasks

### Task A: Create Integration Test

**Success Criteria**:
- [ ] **Implementation**: Integration test created → `tests/integration_p15_sony_tag_prefix.rs:test_sony_tag_prefix_behavior` validates Sony TAG_PREFIX behavior
- [ ] **Test fails**: `cargo t test_sony_tag_prefix_behavior` fails showing current Tag_XXXX behavior
- [ ] **ExifTool comparison**: Test uses ExifTool output as reference for expected behavior
- [ ] **Specific validation**: Test checks that Sony unknown tags get "Sony_0xXXXX" format instead of "Tag_XXXX"

**Implementation Details**: Create test that processes Sony test image and validates unknown tag naming follows TAG_PREFIX pattern
**Integration Strategy**: Use existing test infrastructure with Sony.jpg test image
**Validation Plan**: Compare exif-oxide output with expected TAG_PREFIX format
**Dependencies**: None

### Task B: Replace Hardcoded Tag_ Fallbacks

**Success Criteria**:
- [ ] **Implementation**: Replace 23+ hardcoded fallbacks → All `format!("Tag_{tag_id:04X}")` calls across 8 files replaced with `generate_tag_prefix_name()` calls
  - `src/exif/mod.rs:219,232,336,341,369,473,475,477,478,483,552,566,582,592,601,606,610,614` (14 locations)
  - `src/exif/processors.rs:647`, `src/exif/ifd.rs:135,865` (3 locations)
  - `src/implementations/makernotes.rs:364,542`, `src/implementations/olympus/mod.rs:364` (3 locations)
  - `src/composite_tags/resolution.rs:48,50` (2 locations)
- [ ] **Integration**: Proper TagSourceInfo passed → All call sites provide correct source context to enable manufacturer-specific naming
- [ ] **Task 0 passes**: `cargo t test_sony_tag_prefix_behavior` now succeeds
- [ ] **Manual validation**: `cargo run -- third-party/exiftool/t/images/Sony.jpg | grep -c "Tag_"` shows 1 instead of 8
- [ ] **Cleanup verification**: `grep -r "format!(\"Tag_" src/ | grep -v "generate_tag_prefix_name"` shows only legitimate remaining cases
- [ ] **Evidence**: `git show COMMIT_HASH` shows specific files modified to use TAG_PREFIX mechanism

**Implementation Details**: Replace hardcoded `format!("Tag_{tag_id:04X}")` with `generate_tag_prefix_name(tag_id, source_info)` calls across 8 files, ensuring proper TagSourceInfo context is available
**Integration Strategy**: Ensure all fallback paths call `generate_tag_prefix_name()` with proper source context
**Validation Plan**: Test with Sony image and verify TAG_PREFIX behavior across all code paths
**Dependencies**: Task A (integration test)

### Task C: Investigate Sony Namespace Assignment

**Success Criteria**:
- [ ] **Research**: Sony subdirectory processing analyzed → Understanding of why subdirectory processing isn't triggered for test image
- [ ] **Implementation**: Namespace fixes applied if needed → Sony tags get proper "Sony" namespace when subdirectory processing does occur
- [ ] **Integration**: Binary data processing uses TAG_PREFIX → When subdirectory processing is active, unknown tags get Sony_0xXXXX format
- [ ] **Validation**: Comment at `src/implementations/sony/mod.rs:40` resolved → Either fix implemented or comment updated to reflect current understanding

**Implementation Details**: Debug why Sony subdirectory processing isn't triggered and fix namespace assignment if needed
**Integration Strategy**: Ensure Sony binary data extraction integrates with TAG_PREFIX when active
**Validation Plan**: Test with images that trigger Sony subdirectory processing
**Dependencies**: Task B (hardcoded fallbacks fixed)

### Task D: Validate TAG_PREFIX Across All Manufacturers

**Success Criteria**:
- [ ] **Testing**: All manufacturers validated → Canon, Nikon, Olympus test images show proper TAG_PREFIX behavior
- [ ] **Integration**: Consistent behavior → All manufacturers follow same TAG_PREFIX pattern for unknown tags
- [ ] **Manual validation**: Multiple test images confirm TAG_PREFIX works across manufacturer boundaries
- [ ] **Documentation**: Any manufacturer-specific issues documented or resolved

**Implementation Details**: Test TAG_PREFIX mechanism with test images from different manufacturers
**Integration Strategy**: Ensure TAG_PREFIX works consistently regardless of manufacturer
**Validation Plan**: Use test images from multiple manufacturers to verify TAG_PREFIX behavior
**Dependencies**: Task C (Sony-specific issues resolved)

## Implementation Guidance

- **Binary data integration pattern**: Sony binary data extraction happens in `process_subdirectories_with_printconv()` which uses manufacturer-specific functions - TAG_PREFIX logic needs to be integrated at the point where unknown tag names are generated
- **Namespace debugging**: Use `RUST_LOG=debug` to trace namespace assignment during Sony processing - look for "Sony" vs "MakerNotes" in debug output
- **ExifTool translation**: Sony.pm ProcessBinaryData uses TAG_PREFIX automatically - our binary data extraction needs equivalent behavior
- **Testing approach**: Use `third-party/exiftool/t/images/Sony.jpg` as primary test case, verify with `-u` flag on ExifTool to see what unknown tags should exist

## Integration Requirements

- [x] **Activation**: TAG_PREFIX mechanism is enabled by default in standard IFD parsing
- [ ] **Consumption**: Sony binary data processing actively uses TAG_PREFIX for unknown tags
- [ ] **Measurement**: Can verify via debug logs and tag output that Sony unknown tags get manufacturer prefix
- [ ] **Cleanup**: No old approach to deprecate - this extends existing functionality

## Working Definition of "Complete"

A feature is complete when:
- ✅ **System behavior changes** - Sony unknown tags show manufacturer prefix instead of generic Tag_XXXX
- ✅ **Default usage** - TAG_PREFIX applies automatically during Sony binary data extraction
- ✅ **Consistent behavior** - All manufacturers follow same TAG_PREFIX pattern for unknown tags

## Testing

- **Unit**: Test `generate_tag_prefix_name()` function with Sony namespace ✅ (covered by integration test)
- **Integration**: `tests/integration_p15_sony_tag_prefix.rs:test_sony_tag_prefix_behavior` validates end-to-end TAG_PREFIX behavior ✅ PASSES
- **Manual check**: Run `cargo run -- third-party/exiftool/t/images/Sony.jpg | grep -c "Tag_"` and confirm result is 1 (not 8) ✅ VERIFIED
- **Regression prevention**: Integration test will catch any future regressions in TAG_PREFIX behavior
- **Cross-manufacturer validation**: Ready for expansion to Canon, Nikon, Olympus test images (Task D scope)


## Definition of Done

- [ ] **Integration test passes**: `cargo t test_sony_tag_prefix_behavior` succeeds
- [ ] **Sony TAG_PREFIX active**: `cargo run -- third-party/exiftool/t/images/Sony.jpg | grep -c "Tag_"` returns 1 (only EXIF:Tag_C4A5 remains)
- [ ] **Expected format**: Sony unknown tags show "MakerNotes:Sony_0xXXXX" format matching ExifTool behavior
- [ ] **Fallback cleanup**: `grep -r "format!(\"Tag_" src/ | grep -v generate_tag_prefix_name` shows minimal remaining cases
- [ ] **Multi-manufacturer validation**: Canon, Nikon, Olympus test images show consistent TAG_PREFIX behavior
- [ ] **Build success**: `make precommit` passes with all changes
- [x] **DNG codegen**: `codegen/config/DNG_pm/tag_kit.json` exists with 98 extracted tag kits

## Future Work / Refactoring Ideas

- **Unified unknown tag handling**: Create single point where all unknown tags (IFD and binary data) get TAG_PREFIX treatment
- **Codegen TAG_PREFIX extraction**: Extract TAG_PREFIX patterns directly from ExifTool source instead of hardcoding manufacturer list
- **Binary data processor consolidation**: Unify binary data extraction patterns across manufacturers to reduce code duplication
- **Namespace assignment audit**: Review all namespace assignments to ensure consistency with ExifTool group behavior
