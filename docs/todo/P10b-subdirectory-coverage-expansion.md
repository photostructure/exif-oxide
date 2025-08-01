# Technical Project Plan: Subdirectory Coverage Expansion

## Project Overview

- **Goal**: Expand subdirectory coverage from 12.23% (229/1872) to 50%+ by implementing missing configurations for high-impact zero-coverage modules
- **Status**: Phase 1 partially complete - coverage improved to 13.89% (260/1872), +31 subdirectories implemented. Critical gaps remain in functional integration.
- **Problem**: Critical modules like Exif (generated but non-functional), DNG (94 subdirs), JPEG (64) remain at 0% functional coverage, preventing meaningful metadata extraction from common file formats
- **Constraints**: Must maintain ExifTool compatibility, leverage existing mature tag kit architecture, no performance regression

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

- **Tag Kit Architecture**: Production-ready system generating binary data processors from ExifTool source. Produces working functions like `process_canon_camerasettings()` that extract meaningful tags (`MacroMode: "Macro"`, `SelfTimer: 10`) from raw binary arrays.
- **Runtime Evaluation System**: Fully implemented at `src/runtime/` with `SubdirectoryConditionEvaluator` handling complex patterns (`$$valPt =~ /pattern/`, `$$self{Model} =~ /EOS/`, `$count == N`) for dynamic subdirectory dispatch.
- **Subdirectory Processing Pipeline**: Generic `process_subdirectories_with_printconv()` architecture connects binary extraction with PrintConv formatting, used by Canon implementation to transform raw arrays into human-readable values.

### Key Concepts & Domain Knowledge

- **Subdirectory Coverage**: Measures implementation of ExifTool's SubDirectory references that parse binary data structures within maker notes and metadata
- **Binary Data Processing**: Converts raw byte arrays (e.g., `ColorData1: [10, 789, 1024, ...]`) into structured tags (e.g., `WB_RGGBLevelsAsShot: "2241 1024 1024 1689"`)
- **Cross-Module References**: 402 subdirectories reference tables from other modules, requiring stub generation and shared table extraction
- **Required Tags**: 130 tags marked `required: true` in `docs/tag-metadata.json` drive priority decisions

### Surprising Context

**CRITICAL**: The subdirectory infrastructure is mature and production-ready, not experimental:

- **Working Implementations Exist**: Canon module shows 51.0% coverage (75/147) with real binary data extraction producing meaningful tags
- **Runtime System Complete**: `src/runtime/condition_evaluator.rs` handles complex ExifTool condition patterns with regex caching and binary pattern matching
- **Coverage Measurement Limitations**: The 12.23% metric only checks for text mentions in configs/generated files, not functional correctness of processors
- **High-Impact Zero Coverage**: Modules like Exif (122 subdirs) represent 6.5% potential coverage gain but have no implementation

**Counter-Intuitive Reality**: Many "missing" features are actually complete - the gap is in configuration generation for high-impact modules, not core architecture.

### Foundation Documents

- **Working Architecture**: `src/generated/Canon_pm/tag_kit/mod.rs:11552` - Real subdirectory processors
- **Runtime Integration**: `src/exif/subdirectory_processing.rs` - Generic processing pipeline
- **Coverage Analysis**: `docs/generated/SUBDIRECTORY-COVERAGE.md` - Current state metrics
- **ExifTool Source**: `third-party/exiftool/lib/Image/ExifTool/*.pm` - Original subdirectory definitions

### Prerequisites

- **Knowledge Assumed**: Tag kit system architecture, ExifTool subdirectory patterns, binary data formats
- **Setup Required**: Working `make codegen` environment, test image collection for validation

**Context Quality Check**: Engineers should understand that this is config generation work on mature infrastructure, not greenfield development.

## Work Completed

### Infrastructure Foundation (Pre-July 2025)
- ✅ **Tag Kit Subdirectory Infrastructure** → Chose production-ready architecture over experimental approach because proven with Canon implementation
- ✅ **Runtime Evaluation System** → Completed July 25, 2025 with full condition pattern support at `src/runtime/`
- ✅ **Cross-Module Reference Handling** → Rejected direct perl parsing due to complexity, implemented stub generation approach
- ✅ **Canon Implementation Proof** → Achieved 51.0% coverage (75/147) demonstrating architecture effectiveness
- ✅ **Coverage Measurement Tools** → Built `subdirectory_discovery.pl` and dashboard integration for progress tracking

### Phase 1 Implementation (July 31, 2025) 
- ✅ **Nikon Enhancement** → Added 4 missing tables (OrientationInfo, SettingsInfoD810, BracketingInfoD810, ISOAutoInfoD810) to existing config, coverage 0.5%→2.8%
- ✅ **Canon Enhancement** → Added 3 advanced tables (PSInfo2, ColorCalib, ColorCalib2) for picture style effects and color calibration
- ✅ **Sony Enhancement** → Added SR2Private table for Sony RAW file processing
- ⚠️ **Pentax Module Created** → Config exists but non-functional: 0% coverage (0/51 subdirectories) despite generated configuration
- ⚠️ **Matroska Module Created** → Partial implementation: 4.2% coverage (2/48 subdirectories), not complete as claimed
- ✅ **MIE Module Created** → Media Information Exchange format with 19 tables in hierarchical structure  
- ⚠️ **Jpeg2000 Module Created** → Partial implementation: 9.4% coverage (3/32 subdirectories), not complete as claimed
- ⚠️ **Exif Module Stub Generated** → Config created with 33+ processors, but all return empty results - functional integration incomplete
- ❌ **Coverage Validation** → Generated code compiles but build fails on precommit due to single test failure (`test_exif_ifd_specific_tags` - ColorSpace group1 assignment), coverage improved 12.23%→13.89% (+31 subdirectories)

## Remaining Tasks

### 1. Task: Complete Exif Module Functional Integration

**Success Criteria**: Exif subdirectory processors extract meaningful metadata from real EXIF tags instead of returning empty results
**Approach**: Debug and fix the 33+ generated Exif processors to handle cross-module references and runtime conditions properly
**Dependencies**: Fix compilation errors preventing build validation

**Current Status**: Config exists, 31 processors generated, but all return `Ok(vec![])` due to cross-module reference stubs
**Root Cause**: 26 processors contain TODOs for cross-module table access to Kodak, IPTC, XMP, and JSON modules
**Success Patterns**:
- ✅ Cross-module reference system implemented (shared table access for Kodak, IPTC, XMP, JSON)
- ✅ Processors extract actual tag data from test images (not empty results)
- ✅ Coverage report shows functional Exif implementation >5%
- ✅ ExifTool comparison tests pass for extracted EXIF tags

### 2. Task: Create DNG Module Tag Kit Configuration

**Success Criteria**: DNG format metadata extraction working with subdirectory processing
**Approach**: Create `codegen/config/DNG_pm/tag_kit.json` from scratch using `auto_config_gen.pl` on `DNG.pm`
**Dependencies**: Exif module complete (DNG extends EXIF)
**Current Status**: No DNG module exists - needs creation from zero, not generation from existing

**Success Patterns**:
- ✅ Adobe DNG files produce proper metadata extraction
- ✅ RAW preview/thumbnail data processing functional
- ✅ Coverage increase of ~5% (94 subdirectories)

### 3. Task: Create JPEG Module Tag Kit Configuration  

**Success Criteria**: JPEG metadata segments processed through subdirectory system
**Approach**: Create `codegen/config/JPEG_pm/tag_kit.json` from scratch using `auto_config_gen.pl` on `JPEG.pm`
**Dependencies**: None - mostly independent processing
**Current Status**: No JPEG module exists - needs creation from zero, not generation from existing

**Success Patterns**:
- ✅ JPEG files show improved metadata extraction
- ✅ APP segment subdirectories correctly parsed
- ✅ Coverage increase of ~3.4% (64 subdirectories)

### 4. Phase 2: Target Remaining High-Impact Zero-Coverage Modules

**Completed Research Results**: Analysis shows remaining high-value targets:
- **Ricoh** (10 subdirs) - Zero coverage, compact camera support
- **ASF** (12 subdirs) - Windows Media format support  
- **FLAC** (4 subdirs) - Audio metadata extraction
- **PanasonicRaw** (9 subdirs) - Panasonic RAW format support

**Next Priority**: Fix Exif module functional integration first, then DNG/JPEG implementation

### 5. CRITICAL BLOCKER: Fix Build and Integration Issues

**Success Criteria**: `make precommit` passes and generated processors extract real metadata
**Approach**: Address compilation errors and functional integration gaps identified in current assessment
**Dependencies**: None - must be resolved before continuing with new modules

**Specific Issues Identified**:
- ❌ **Build Failure**: Single test failure `test_exif_ifd_specific_tags` due to ColorSpace group1 assignment (shows "Canon" instead of "ExifIFD")
- ❌ **Functional Integration Gap**: Exif processors return `Ok(vec![])` instead of extracting meaningful tags  
- ❌ **Cross-Module References**: 26 Exif processors contain TODOs for cross-module table access (Kodak, IPTC, XMP, JSON modules)
- ❌ **Coverage Measurement**: Tools may not accurately reflect functional vs. stub implementations

**Success Patterns**:
- ✅ `make precommit` completes without errors
- ✅ Exif processors extract real tag data from test images
- ✅ Coverage measurement reflects functional capability, not just code generation
- ✅ Cross-module reference handling produces working implementations

## Implementation Guidance

**Proven Patterns from Phase 1**:
- **Same-module focus**: Target tables with SubDirectory references within the same module for working processors
- **Tag kit approach**: Create comprehensive tag_kit.json configs rather than piecemeal extraction
- **Systematic validation**: Always run `make codegen && cargo check` to verify compilation success
- **Cross-module stubs**: Generate stubs for cross-module references to prevent compilation errors
- **Research first**: Use exiftool-researcher agent to understand module structure before implementation

**Tools to Leverage**:
- `codegen/extractors/auto_config_gen.pl` - Automated configuration generation
- `make subdirectory-coverage` - Progress tracking and validation
- `cargo run --bin compare-with-exiftool` - ExifTool compatibility validation
- `codegen/extractors/subdirectory_discovery.pl` - Gap analysis

**Architecture Considerations**:
- Subdirectory processors are pure functions: `fn(data: &[u8], ByteOrder) -> Result<Vec<(String, TagValue)>>`
- PrintConv application happens after binary extraction in generic processing pipeline
- Cross-module references require shared table extraction or stub generation
- Runtime condition evaluation should be last resort - prefer compile-time pattern matching

**Performance Notes**:
- Binary data processing is CPU-intensive - profile large files during implementation
- Regex compilation caching in runtime evaluator prevents performance degradation
- Subdirectory processing happens once per tag - not performance critical path

**ExifTool Translation Notes**:
- `%table = ( ... )` patterns in ExifTool become `process_*` functions in our system
- Binary data offsets use signed arithmetic for ExifTool compatibility (negative offsets reference from end)
- Condition patterns `$count == N` become Rust match arms in dispatcher functions

## Integration Requirements

**CRITICAL**: Building without integrating is failure. Don't accept tasks that build "shelf-ware."

Every feature must include:
- [x] **Activation**: Generated tag kit configs automatically used by manufacturer processors
- [x] **Consumption**: Generic subdirectory processing pipeline consumes generated functions  
- [x] **Measurement**: Coverage reports and ExifTool comparison show functional improvement
- [x] **Cleanup**: No manual binary data processing needed - tag kit handles all cases

**Red Flag Check**: If subdirectory processors generate but aren't called by manufacturer implementations, integration is incomplete.

## Working Definition of "Complete"

A module subdirectory implementation is complete when:
- ✅ **System behavior changes** - Real image files produce different/better metadata extraction
- ✅ **Default usage** - Manufacturer processor automatically uses generated subdirectory functions
- ✅ **Old path removed** - No raw binary arrays displayed for tags with working processors
- ❌ Config exists but processors not integrated *(example: "tag_kit.json created but Canon.pm still shows raw arrays")*
- ❌ Processors compile but aren't called *(example: "process_exif_gps generated but never invoked")*

## Prerequisites

- Understanding of tag kit architecture → Review `src/generated/Canon_pm/tag_kit/mod.rs` working examples
- ExifTool source familiarity → Read relevant `*.pm` files for target modules
- Test image access → Collect representative files for each target format

## Testing

- **Unit**: Test binary data processors with known input/output pairs from ExifTool
- **Integration**: Verify end-to-end extraction on real camera files produces correct metadata
- **Manual check**: Run `cargo run --bin compare-with-exiftool test.jpg` and confirm subdirectory tag alignment

## Definition of Done

- [ ] `cargo t subdirectory` passes - all subdirectory processing tests working
- [ ] `make precommit` clean - **CURRENTLY FAILING**: Single test failure `test_exif_ifd_specific_tags` (ColorSpace group1 assignment issue)
- [ ] Coverage reaches 35%+ (current 13.89% + target modules ~21%) measured by **functional** subdirectory implementation
- [ ] ExifTool compatibility maintained for all existing functionality
- [ ] At least 3 zero-coverage high-impact modules (Exif, DNG, JPEG) producing **working** subdirectory extraction (not empty stubs)
- [ ] **CRITICAL**: Generated processors must extract real metadata, not return empty results

## Success Criteria & Quality Gates

### Overall Success

- [ ] **Coverage Target**: Reach 50% subdirectory coverage (935+ implemented)
- [ ] **Quality Maintained**: No ExifTool compatibility regressions
- [ ] **Performance Preserved**: No >10% slowdown in metadata extraction
- [ ] **Architecture Validated**: Generic subdirectory processing proven with 5+ manufacturers

### Module-Specific Success

- [ ] **Exif Module**: 122 subdirectories → 30+ implemented (>25% coverage)
- [ ] **DNG Module**: 94 subdirectories → 20+ implemented (>20% coverage)  
- [ ] **JPEG Module**: 64 subdirectories → 15+ implemented (>23% coverage)
- [ ] **Integration Verified**: All generated processors called by manufacturer implementations

## Gotchas & Tribal Knowledge

**Coverage Measurement Caveats**: The 13.88% coverage metric only checks text mentions, not functional correctness. Real coverage may be lower due to:
- Generated stubs that return empty results
- Cross-module references generating TODO comments
- Processors that compile but aren't integrated into manufacturer modules

**ExifTool Source Complexity**: Module analysis reveals:
- Binary data tables use complex offset calculations with negative indices
- Condition patterns range from simple count checks to complex binary signature matching
- Cross-module references require careful dependency ordering during generation

**Configuration Generation Pitfalls**:
- Auto-generated configs may miss complex conditional logic
- Large tables (>100 entries) can cause compilation slowdowns
- Some ExifTool patterns don't translate directly to Rust match statements

**Integration Anti-Patterns**:
- Adding tag_kit config without updating manufacturer processor integration
- Generating processors that compile but never get called
- Assuming text-based coverage metrics represent functional capability

## Risk Mitigation

### Module Complexity Risk
- **Risk**: Some modules have patterns too complex for current generators
- **Mitigation**: Focus on high-impact binary data tables first, defer complex conditional logic
- **Monitoring**: Track percentage of generated vs stubbed processors per module

### Performance Risk  
- **Risk**: Large subdirectory counts could slow metadata extraction
- **Mitigation**: Profile with representative image collections, optimize hot paths
- **Measurement**: Benchmark extraction time before/after each major module addition

### Quality Risk
- **Risk**: Generated processors may not match ExifTool behavior exactly
- **Mitigation**: Comprehensive ExifTool comparison testing on diverse image collection
- **Validation**: Manual verification of key tags from each implemented module

## Implementation Order

**Phase 1 Completed (July 31, 2025)**: 
1. ✅ **Pentax Module**: Proved architecture works across camera brands (0%→60 processors)
2. ✅ **Video Format Support**: Matroska/MIE/Jpeg2000 for comprehensive multimedia support
3. ✅ **Manufacturer Enhancements**: Nikon/Canon/Sony improvements for better camera coverage

**Phase 2 Recommended Order**:
1. **Exif Module**: Highest subdirectory count (122) and most universal impact - many cross-module references require careful handling
2. **DNG Module**: Builds on EXIF foundation, high subdirectory count (94) - Adobe RAW format critical for photography workflow
3. **JPEG Module**: Different processing patterns, good validation target (64 subdirs) - fundamental image format support
4. **Ricoh/ASF modules**: Smaller scope for validation of continued approach effectiveness

**Key Insight from Phase 1**: Focus on same-module subdirectory references for working processors. Cross-module references (like Exif module) may require different approach or generate mostly stubs.

This plan builds on the mature subdirectory infrastructure and proven Phase 1 patterns to systematically expand coverage by targeting high-impact zero-coverage modules, focusing on configuration generation rather than architecture development.

## Current TPP Status Assessment (August 1, 2025)

**TPP COMPLETION STATUS: SIGNIFICANTLY INACCURATE DOCUMENTATION** 

### Critical Issues Preventing Completion

1. **Build Failure**: `make precommit` fails due to single test failure `test_exif_ifd_specific_tags` (ColorSpace group1 assignment issue) - IFD context bug, not infrastructure issue
2. **Functional Integration Gap**: Generated Exif processors exist but return empty results due to 26 cross-module reference TODOs  
3. **Coverage Target Missed**: Current 13.89% vs. target 50% - only reached ~28% of goal
4. **Module Implementation Quality**: DNG and JPEG modules don't exist at all, Exif module generates stubs due to cross-module references

### Validation Findings vs TPP Claims

**❌ Major Documentation Errors Identified**:
- **Phase 1 Claims**: Pentax (0% actual vs "complete"), Matroska (4.2% vs "3 core tables"), Jpeg2000 (9.4% vs "7 tables")
- **Build Issues**: Single IFD test failure vs claimed "missing GPS functions"
- **Module Status**: DNG/JPEG don't exist vs "generate from existing"

**✅ Confirmed Issues**:
- Exif module: 31 processors with 26 cross-module reference TODOs
- Coverage: 13.89% (260/1872) accurate
- Infrastructure: Mature and working (Canon 51% coverage proves effectiveness)

### Work Required for Completion

**Immediate Priority (Address Root Causes)**:
- Fix IFD context assignment test failure (not infrastructure issue)
- Implement cross-module reference system for Exif (26 TODOs to Kodak, IPTC, XMP, JSON)
- Create DNG and JPEG modules from scratch (don't exist currently)

**Remaining Implementation Work**:
- Generate and integrate DNG module tag kit configuration  
- Generate and integrate JPEG module tag kit configuration
- Achieve functional 35%+ coverage with working processors

**Recommendation**: This TPP requires significant additional work to reach completion. The infrastructure exists but functional integration and remaining module implementations need substantial effort. Consider breaking into smaller, focused TPPs for each major blocker (build fixes, Exif integration, DNG implementation, JPEG implementation).