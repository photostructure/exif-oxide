# Technical Project Plan: IFD Parsing Completeness and Context Resolution

## Project Overview

- **Goal**: Fix remaining Sony namespace assignment issue and verify/address any remaining IFD context edge cases to maximize metadata extraction completeness
- **Problem**: Sony MakerNotes tags show as "Tag_xxxx" instead of proper names due to namespace assignment defaulting to "MakerNotes" during subdirectory processing; need verification of other claimed context bugs
- **Constraints**: Must preserve existing ExifIFD, GPS IFD, and namespace-aware architecture while fixing edge cases, maintain ExifTool compatibility

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

- **Mature IFD Infrastructure**: Complete ExifIFD (20250706), GPS IFD (20250727), and namespace-aware tag storage architecture supporting context-aware processing with `HashMap<(u16, String), TagValue>` storage
- **Sophisticated Context System**: IFD context tracking with group assignment, recursion prevention, and manufacturer-specific MakerNotes processing across 14+ implementations
- **Subdirectory Processing Pipeline**: Generic subdirectory processing with condition evaluation, binary data extraction, and cross-module table references supporting Canon (51% coverage), Nikon, Sony, and others

### Key Concepts & Domain Knowledge

- **IFD Context Bugs**: Existing sophisticated system has edge cases where context assignment fails (e.g., ColorSpace→Canon instead of ExifIFD group1)
- **Namespace Edge Cases**: Despite namespace-aware storage, some tags still experience context resolution failures in complex subdirectory scenarios
- **Coverage Gaps**: High-impact modules like Exif (122 subdirs), DNG (94 subdirs) remain at 0% configuration coverage despite working infrastructure
- **Context Resolution**: ExifTool's dynamic tag table switching requires precise context tracking during nested subdirectory processing

### Surprising Context

**CRITICAL**: Document non-intuitive aspects that aren't obvious from casual code inspection:

- **Infrastructure is Complete**: ExifIFD work (20250706-ExifIFD.md) shows comprehensive context tracking, GPS IFD bug (20250727-P10c) resolved namespace collisions with sophisticated architecture
- **Most Claimed Issues Are Fixed**: ExifIFD tests are PASSING (not failing as previously documented), Exif_pm config EXISTS at `codegen/config/Exif_pm/tag_kit.json`
- **Primary Remaining Issue**: Sony namespace assignment bug - creates "MakerNotes:Tag_xxxx" instead of proper Sony tag names during subdirectory processing
- **Coverage vs Functionality**: Subdirectory coverage of 13.89% (260/1872) represents configuration gaps, not architectural limitations - Canon shows 51% coverage with working processors
- **Context Assignment Works Generally**: Most namespace switching works correctly; Sony issue is specific to two-phase processing (main IFD + subdirectories)
- **Historical Success**: GPS coordinate extraction (P10c) required major architecture overhaul that successfully implemented namespace-aware storage system

### Foundation Documents

- **Completed Architecture**: [20250706-ExifIFD.md](../done/20250706-ExifIFD.md) - Complete ExifIFD implementation with context tracking
- **Namespace Resolution**: [20250727-P10c-gps-ifd-parsing-bug.md](../done/20250727-P10c-gps-ifd-parsing-bug.md) - Namespace-aware storage architecture 
- **Subdirectory Coverage**: [P10b-subdirectory-coverage-expansion.md](P10b-subdirectory-coverage-expansion.md) - Configuration generation for missing modules
- **ExifTool source**: `third-party/exiftool/lib/Image/ExifTool/Exif.pm` lines 6174-7128 (ProcessExif with dynamic table switching)

### Prerequisites

- **Knowledge assumed**: Understanding of completed ExifIFD/GPS IFD architectures, ExifTool's dynamic tag table system, Rust namespace-aware HashMap patterns
- **Setup required**: Working IFD test suite, `make compat` environment, test images demonstrating context bugs

**Context Quality Check**: Can a new engineer understand WHY this is edge case debugging on mature architecture rather than greenfield development?

## Work Completed

- ✅ **ExifIFD Architecture (June 2025)** → Complete implementation with group assignment, context tracking, API compatibility, and comprehensive test coverage - **TESTS ARE PASSING**
- ✅ **GPS IFD Namespace Resolution (July 2025)** → Major architectural breakthrough implementing `HashMap<(u16, String), TagValue>` storage to prevent tag collisions  
- ✅ **Sophisticated Context System** → IFD context stack management, recursion prevention, manufacturer signature detection across 14+ implementations
- ✅ **Subdirectory Processing Pipeline** → Generic processing with condition evaluation, binary data extraction, and 51% Canon coverage demonstrating effectiveness
- ✅ **Exif_pm Configuration (Pre-August 2025)** → Complete configuration exists at `codegen/config/Exif_pm/tag_kit.json` with composite tags, simple tables, and tag kit definitions
- ⚠️ **Sony Context Parameter Bug (July 31, 2025)** → **PARTIALLY FIXED** - apply_conversions() function updated but core namespace assignment issue remains

## Remaining Tasks

**REQUIRED**: Each task must be numbered, actionable, and include success criteria.

### 1. Task: Fix Sony Namespace Assignment During Subdirectory Processing

**Success Criteria**: Sony tags show proper names like "SonyExposureTime" instead of "MakerNotes:Tag_2000", `cargo run --bin exif-oxide third-party/exiftool/t/images/Sony.jpg | grep -c Tag_` shows 0 instead of 8
**Approach**: Fix namespace assignment in `create_tag_source_info()` or Sony subdirectory processing to preserve "Sony" namespace instead of defaulting to "MakerNotes"
**Dependencies**: None - builds on existing Sony signature detection

**Success Patterns**:

- ✅ Sony MakerNotes tags show proper names: "SonyExposureTime", "SonyFNumber", etc.
- ✅ Two-phase Sony processing (main IFD + subdirectories) maintains Sony namespace throughout
- ✅ `apply_conversions()` Sony condition triggers correctly with preserved namespace
- ✅ No regressions in existing Canon/Nikon/Olympus processing functionality

**Root Cause Identified**: Namespace assignment in subdirectory processing phase defaults to "MakerNotes" instead of preserving manufacturer-specific namespace from main IFD detection.

**VALIDATION FINDINGS (August 1, 2025)**:

**✅ Sony Context Bug Confirmed**: Root cause verified - namespace assignment defaults to "MakerNotes" during Sony subdirectory processing instead of preserving "Sony" namespace from initial detection.

**❌ Major TPP Claims Corrected**:
- **ExifIFD Tests**: `test_exif_ifd_group_assignment` is **PASSING**, not failing as documented
- **Exif_pm Config**: **EXISTS** at `codegen/config/Exif_pm/tag_kit.json`, contrary to "Missing: No Exif_pm config" claim
- **Context Assignment**: Most namespace switching works correctly; Sony is specific edge case, not systemic failure

**✅ Infrastructure Status Confirmed**:
- GPS IFD namespace resolution working correctly
- ExifIFD architecture functional with passing tests
- 51% Canon coverage demonstrates subdirectory processing effectiveness
- Sophisticated context system handles complex manufacturer scenarios successfully

**🔍 Remaining Work Identified**:
- **Primary Issue**: Sony namespace assignment in `create_tag_source_info()` during subdirectory processing 
- **Secondary**: DNG config generation for RAW metadata support (94 subdirs = 5% coverage potential)
- **Validation**: Need evidence-based testing of any other claimed context issues

**Status**: Issues are narrower and more specific than originally documented. Infrastructure is solid; remaining work is targeted fixes rather than architectural overhaul.

### 2. Task: Generate DNG Module Configuration for RAW Metadata Support

**Success Criteria**: DNG module configuration created and operational, basic RAW metadata extraction enabled, coverage increased from 13.89% baseline
**Approach**: Use existing `auto_config_gen.pl` to generate DNG module config (94 subdirs = 5.0% potential coverage increase)
**Dependencies**: Task 1 (Sony issue resolved to validate configuration generation doesn't introduce similar issues)

**Success Patterns**:

- ✅ `codegen/config/DNG_pm/tag_kit.json` created and generates working processors
- ✅ DNG module enables Adobe DNG and camera RAW metadata extraction
- ✅ Coverage report shows measurable increase from 13.89% baseline
- ✅ No regressions in existing manufacturer processing

**Status Update**: Exif_pm configuration already exists - focus on DNG which represents significant coverage opportunity for RAW file support.

### 3. Task: Verify and Address Any Remaining Context Edge Cases

**Success Criteria**: Verification of claimed context issues with targeted testing, resolution of any confirmed edge cases found
**Approach**: Systematic testing of complex manufacturer + ExifIFD scenarios to identify actual (not claimed) context assignment failures  
**Dependencies**: Tasks 1-2 (Sony issue resolved, DNG config added to increase test coverage)

**Success Patterns**:

- ✅ Targeted testing of ColorSpace context assignment with Canon files confirms current behavior
- ✅ Complex scenarios (Canon CR2 + GPS + MakerNotes + ExifIFD) maintain correct context throughout processing
- ✅ Any confirmed edge cases are resolved with minimal architectural impact
- ✅ Composite tag dependencies work correctly across namespace boundaries

**Validation Focus**: Since ExifIFD tests are passing, need evidence-based identification of any remaining context issues rather than assumption-based fixes.

### 4. Task: Measure and Document Current IFD Parsing Completeness

**Success Criteria**: Current baseline established through working `make compat` tests, impact of Sony fix and DNG config measured
**Approach**: Fix compatibility test execution issues, establish current success rate baseline, measure improvements from Tasks 1-2
**Dependencies**: Tasks 1-2 (Sony issue resolved, DNG config added)

**Success Patterns**:

- ✅ `make compat` executes successfully without truncation/timeout issues
- ✅ Current baseline success rate documented (replacing unverified 39% claim)
- ✅ Sony namespace fix shows measurable improvement in tag extraction
- ✅ DNG configuration enables new RAW metadata extraction capabilities

**Measurement Strategy**: Focus on establishing accurate current baseline rather than assuming 39% (66/167) metric, then measure concrete improvements from completed fixes.

**Task Quality Check**: Can another engineer pick up any task and complete it without asking clarifying questions?

## Integration Requirements

**CRITICAL**: Building without integrating is failure. Don't accept tasks that build "shelf-ware."

Every feature must include:
- [ ] **Activation**: Fixed context assignment and new configurations used automatically in default processing
- [ ] **Consumption**: Existing metadata extraction pipeline benefits from improved IFD parsing immediately
- [ ] **Measurement**: Can prove improvements via `make compat` success rate increases and specific test case resolutions
- [ ] **Cleanup**: Context assignment edge cases eliminated, configuration gaps filled, debugging aids removed from production code

**Red Flag Check**: If this seems like "fix edge cases but don't validate end-to-end impact," ask for clarity. We're improving metadata extraction completeness for PhotoStructure's production workflows.

## Working Definition of "Complete"

*Use these criteria to evaluate your own work - adapt to your specific context:*

A feature is complete when:
- ✅ **System behavior changes** - More metadata tags extracted successfully from real image files, fewer context assignment failures
- ✅ **Default usage** - IFD parsing improvements benefit all metadata extraction automatically, not opt-in fixes
- ✅ **Old path removed** - Context assignment bugs eliminated, missing configuration gaps filled
- ❌ Code exists but isn't used *(example: "context fixes implemented but tests still failing")*
- ❌ Feature works "if you call it directly" *(example: "new configurations exist but aren't integrated into main pipeline")*

*Note: These are evaluation guidelines, not literal requirements for every task.*

## Prerequisites

- **Dependency 1**: Existing ExifIFD architecture (completed 20250706) → validate with ExifIFD test suite
- **Dependency 2**: GPS IFD namespace resolution (completed 20250727) → validate with GPS coordinate extraction
- **Dependency 3**: Subdirectory processing pipeline (operational) → validate with Canon implementation showing 51% coverage

## Testing

- **Unit**: Test IFD context assignment with isolated ExifIFD, GPS IFD, and manufacturer subdirectory scenarios
- **Integration**: Verify end-to-end metadata extraction improvements with representative files from each manufacturer
- **Manual check**: Run `make compat` and confirm measurable success rate improvements from baseline 39% (66/167 tags)

## Definition of Done

- [ ] Sony namespace issue resolved: `cargo run --bin exif-oxide third-party/exiftool/t/images/Sony.jpg | grep -c Tag_` shows 0 instead of 8
- [ ] DNG configuration generated and operational: `codegen/config/DNG_pm/tag_kit.json` exists and enables RAW metadata extraction
- [ ] `make precommit` clean with no regressions
- [ ] Current success rate baseline established through working `make compat` tests
- [ ] Subdirectory coverage measurably increased from 13.89% baseline with DNG support

## Implementation Guidance

### Recommended Patterns

- **Context Debugging**: Use existing IFD context tracking infrastructure, focus on timing of context assignment vs manufacturer processing
- **Configuration Generation**: Leverage proven `auto_config_gen.pl` used successfully for Pentax, Matroska, MIE, and Jpeg2000 modules
- **Namespace Validation**: Build on GPS IFD success patterns that resolved tag ID collision architecture

### Tools to Leverage

- **Existing test infrastructure**: Comprehensive ExifIFD test suite, GPS IFD validation, manufacturer-specific test coverage
- **Configuration tools**: Working `auto_config_gen.pl` and `subdirectory_discovery.pl` for coverage analysis
- **Comparison validation**: `make compat` system for measuring tag extraction improvements

### ExifTool Translation Notes

- **Dynamic Tag Table Switching**: ExifTool's `TagTable => 'Image::ExifTool::GPS::Main'` patterns require precise timing in context assignment
- **ProcessExif Context**: Study lines 6174-7128 in Exif.pm for dynamic table pointer management during subdirectory processing  
- **Subdirectory Dispatch**: Understand how ExifTool maintains context across complex manufacturer subdirectory chains

## Clear Application for PhotoStructure

**Primary Motivation**: PhotoStructure users need complete metadata extraction from their image libraries. Current 39% success rate means 61% of critical metadata (GPS coordinates, lens information, camera settings) is missing from PhotoStructure's database, reducing search effectiveness and user satisfaction.

**Specific Impact**:
- **GPS Location Data**: Fix ExifIFD context bugs to ensure GPS metadata appears correctly for photo mapping features
- **Camera/Lens Information**: Improved IFD parsing extracts complete camera and lens metadata for equipment-based searches
- **RAW File Support**: DNG module configuration enables metadata extraction from Adobe DNG and camera RAW files
- **Search Reliability**: Higher tag extraction success rate means PhotoStructure's search and filtering features find more relevant photos

**Business Context**: Photographers rely on metadata for organizing tens of thousands of photos. Missing metadata reduces PhotoStructure's value proposition and creates support burden from users asking why their GPS coordinates or lens information doesn't appear.

## Integration with Existing Work

### Builds On Completed Architecture
- **ExifIFD Foundation (20250706-ExifIFD.md)**: Uses sophisticated context tracking and group assignment system
- **GPS IFD Resolution (20250727-P10c)**: Leverages namespace-aware storage architecture that solved tag collision issues
- **Subdirectory Infrastructure**: Extends proven subdirectory processing pipeline demonstrated by Canon's 51% coverage

### Coordinates with Planned Work  
- **P10a-exif-required-tags.md**: Provides IFD parsing improvements needed to achieve 90%+ tag extraction goals
- **P10b-subdirectory-coverage-expansion.md**: Implements specific configurations identified as high-impact by coverage analysis
- **P16-MILESTONE-19-Binary-Data-Extraction.md**: Ensures proper IFD context for binary data extraction (previews, thumbnails)

### Avoids Duplication
This TPP focuses on **completeness and edge case resolution** rather than architectural rebuilding. The sophisticated ExifIFD, GPS IFD, and namespace systems are preserved and extended, not replaced.

## Gotchas & Tribal Knowledge

**Format**: Surprise → Why → Solution (Focus on positive guidance)

- **ExifIFD tests failing doesn't mean architecture is broken** → Context assignment timing edge case → Debug context switching in manufacturer subdirectory processing
- **High subdirectory coverage percentage looks intimidating** → Many are configuration gaps, not implementation complexity → Use working `auto_config_gen.pl` patterns from successful modules
- **Namespace conflicts seem like fundamental design flaw** → GPS IFD resolution proved architecture is sound → Focus on edge cases in complex subdirectory chains
- **Context assignment seems fragile** → ExifTool's dynamic table switching requires precise timing → Study ProcessExif context management patterns
- **Sony tags show as "Tag_xxxx" instead of names** → Namespace assignment defaults to "MakerNotes" during subdirectory processing instead of preserving "Sony" → Fix namespace assignment in create_tag_source_info() or Sony subdirectory processing to maintain Sony context
- **Coverage numbers seem inconsistent** → Based on text mentions in configs, not functional correctness → Coverage report at `docs/generated/SUBDIRECTORY-COVERAGE.md` shows 12.23% (229/1872), updated from 13.88% estimate
- **DNG support looks missing** → No `codegen/config/DNG_pm/` exists yet → DNG is Adobe's multi-manufacturer RAW format, needs dedicated configuration generation
- **Large files hard to analyze** → ReadTool truncates >2000 lines → Split large modules like `src/exif/tags.rs` (387+ lines) into focused submodules

## Next Engineer Handoff - Critical Files & Context

### **Key Source Files Needing Attention**

**Primary Fix Location**:
- ✅ `src/exif/tags.rs:257` - `apply_conversions()` function - **FIXED** context parameter bug (added source_info parameter and checks both ifd_name and namespace)
- ✅ `src/exif/mod.rs:593` - **UPDATED** call site to pass source_info parameter  
- `src/exif/ifd.rs:75-100` - Sony MakerNotes signature detection with context assignment
- `tests/exif_ifd_context_tests.rs:25-50` - Failing tests that validate fixes
- `tests/exif_ifd_tests.rs:22-50` - Group assignment tests showing ColorSpace→Canon issue

**ExifTool Reference Points**:
- `third-party/exiftool/lib/Image/ExifTool/Exif.pm:6174-7128` - ProcessExif dynamic tag table switching
- `third-party/exiftool/lib/Image/ExifTool/Exif.pm:8825` - GPS SubDirectory TagTable definition
- `third-party/exiftool/lib/Image/ExifTool/GPS.pm:51-82` - GPS-specific tag table context

**Configuration Status**:
- **Missing**: No `codegen/config/DNG_pm/`, `Exif_pm/`, `JPEG_pm/` configs exist
- **Working Pattern**: `codegen/config/Canon_pm/tag_kit.json` shows successful implementation
- **Generation Tool**: `codegen/extractors/auto_config_gen.pl` proven with Pentax, Matroska, MIE, Jpeg2000

### **Immediate Tasks with Specific Success Criteria**

**Task 1 - Fix Sony Namespace Assignment Issue**:
- ✅ **CONDITION IMPLEMENTED**: Fixed condition in `apply_conversions()` to check both `ifd_name == "Sony"` AND `source_info.namespace == "Sony"`  
- ❌ **CORE ISSUE CONFIRMED**: Namespace assignment during Sony MakerNotes processing defaults to "MakerNotes" instead of "Sony"
- **Root Cause**: Two-phase Sony processing (main IFD + subdirectories) - namespace gets reset during subdirectory phase
- **Current Status**: 8 `Tag_xxxx` entries still showing instead of proper Sony names
- **Next Steps**: Fix namespace assignment in `create_tag_source_info()` at lines 90-110 in `src/exif/tags.rs`
- **Validation Commands**: 
  - `cargo run --bin exif-oxide third-party/exiftool/t/images/Sony.jpg | grep -c Tag_` should show 0 instead of 8
- **Target Files**: 
  - `src/exif/tags.rs:90-110` - `create_tag_source_info()` namespace logic (manufacturer-specific namespace preservation)
  - Alternative: `src/implementations/sony/mod.rs` - Sony subdirectory processing preservation

**Task 2 - Generate DNG Configuration**:
- **Target**: Create `codegen/config/DNG_pm/tag_kit.json` (94 subdirs = 5% coverage increase)
- **Status**: Exif_pm config already exists, focus on DNG for RAW metadata support
- **Method**: `cd codegen && ./extractors/auto_config_gen.pl third-party/exiftool/lib/Image/ExifTool/DNG.pm > config/DNG_pm/tag_kit.json`
- **Validation**: `make codegen` succeeds, enables Adobe DNG and camera RAW metadata extraction

### **Status Update from August 1, 2025 Validation**

**✅ Major TPP Claims Validated**:
- Comprehensive validation of all major infrastructure claims against actual codebase state
- Discovered significant disconnect between documented status and actual working functionality
- Confirmed sophisticated IFD infrastructure is working correctly (GPS namespace resolution, ExifIFD context tracking)
- Verified subdirectory processing pipeline effectiveness with Canon's 51% coverage

**❌ Critical Documentation Errors Identified**: 
- ExifIFD tests are PASSING, not failing as claimed in Task 1
- Exif_pm configuration EXISTS, contrary to "Missing: No Exif_pm config" in Task 2 
- Most context assignment works correctly; Sony is specific edge case, not systemic issue

**✅ Confirmed Remaining Issues**:
- Sony namespace assignment bug: 8 `Tag_xxxx` entries instead of proper names
- DNG configuration missing for RAW metadata support
- Baseline success rate claims (39% / 66/167) require validation due to test execution issues

**🔍 Key Discovery**:
- Infrastructure is more complete than documented; remaining work is targeted fixes rather than major architectural changes

### **Refactoring Opportunities Identified**

**Priority 1 - Sony Namespace Fix** (Primary remaining task):
- Fix `create_tag_source_info()` in `src/exif/tags.rs:90-110` to preserve Sony namespace during subdirectory processing
- Root cause: Two-phase processing resets namespace from "Sony" to "MakerNotes" during subdirectory phase
- Pattern: Manufacturer-specific namespace preservation throughout processing pipeline

**Priority 2 - DNG Configuration Generation** (High-impact addition):
- Generate `codegen/config/DNG_pm/tag_kit.json` using proven `auto_config_gen.pl` methodology
- 94 subdirectories = 5% coverage increase for RAW metadata support
- Enables Adobe DNG and camera RAW file metadata extraction

**Priority 3 - Evidence-Based Edge Case Validation**:
- Systematic testing of claimed context issues (ColorSpace assignment, etc.) with actual test files
- Focus on verification rather than assumption-based fixes since ExifIFD tests are passing
- Document any confirmed edge cases for targeted resolution

**Priority 4 - Success Rate Baseline Establishment**:
- Fix `make compat` execution issues (truncation, timeout) to establish accurate current baseline
- Replace unverified 39% (66/167) claims with measured current state
- Document impact of Sony fix and DNG configuration on metadata extraction completeness

## Quick Debugging

Stuck? Try these:

1. `cargo run --bin exif-oxide third-party/exiftool/t/images/Sony.jpg | grep Tag_` - See current Sony namespace issue (8 entries)
2. `cargo run --bin exif-oxide third-party/exiftool/t/images/Sony.jpg | grep Sony` - Should show proper Sony tag names after fix  
3. `ls codegen/config/DNG_pm/` - Should exist after Task 2 completion
4. `cargo run --bin exif-oxide test-images/dng/sample.dng` - Should extract metadata after DNG config generation
5. `rg "create_tag_source_info" src/ -A 10 -B 5` - Find namespace assignment logic for Sony fix
6. `./codegen/extractors/auto_config_gen.pl third-party/exiftool/lib/Image/ExifTool/DNG.pm` - Generate DNG configuration