# Technical Project Plan: Subdirectory Coverage Expansion

## Project Overview

- **Goal**: Expand subdirectory coverage from 12.23% (229/1872) to 50%+ by implementing missing configurations for high-impact zero-coverage modules
- **Problem**: Critical modules like Exif (122 subdirs), DNG (94), JPEG (64), Pentax (51), Matroska (48) have 0% coverage, preventing meaningful metadata extraction from common file formats
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

- ✅ **Tag Kit Subdirectory Infrastructure** → Chose production-ready architecture over experimental approach because proven with Canon implementation
- ✅ **Runtime Evaluation System** → Completed July 25, 2025 with full condition pattern support at `src/runtime/`
- ✅ **Cross-Module Reference Handling** → Rejected direct perl parsing due to complexity, implemented stub generation approach
- ✅ **Canon Implementation Proof** → Achieved 51.0% coverage (75/147) demonstrating architecture effectiveness
- ✅ **Coverage Measurement Tools** → Built `subdirectory_discovery.pl` and dashboard integration for progress tracking

## Remaining Tasks

### 1. Task: Generate Exif Module Tag Kit Configuration

**Success Criteria**: `codegen/config/Exif_pm/tag_kit.json` exists and generates working subdirectory processors for EXIF metadata tags
**Approach**: Use `auto_config_gen.pl` to analyze `third-party/exiftool/lib/Image/ExifTool/Exif.pm` and extract subdirectory patterns
**Dependencies**: None - infrastructure complete

**Success Patterns**:
- ✅ Config generates compilation-ready Rust code
- ✅ Subdirectory processors extract meaningful tags from test images  
- ✅ Coverage report shows Exif module >5% implementation
- ✅ ExifTool comparison tests pass for extracted tags

### 2. Task: Generate DNG Module Tag Kit Configuration

**Success Criteria**: DNG format metadata extraction working with subdirectory processing
**Approach**: Extract from `DNG.pm` focusing on binary data tables and conditional processing
**Dependencies**: Exif module complete (DNG extends EXIF)

**Success Patterns**:
- ✅ Adobe DNG files produce proper metadata extraction
- ✅ RAW preview/thumbnail data processing functional
- ✅ Coverage increase of ~5% (94 subdirectories)

### 3. Task: Generate JPEG Module Tag Kit Configuration  

**Success Criteria**: JPEG metadata segments processed through subdirectory system
**Approach**: Focus on JPEG.pm APP segment processing and embedded metadata
**Dependencies**: None - mostly independent processing

**Success Patterns**:
- ✅ JPEG files show improved metadata extraction
- ✅ APP segment subdirectories correctly parsed
- ✅ Coverage increase of ~3.4% (64 subdirectories)

### 4. RESEARCH: Prioritize Remaining Zero-Coverage Modules

**Objective**: Identify next highest-impact modules after Exif/DNG/JPEG implementation
**Success Criteria**: Ranked list of modules by impact score (subdirectory_count × required_tag_weight)
**Done When**: Clear priority order for Pentax, Matroska, and other major modules established

## Implementation Guidance

**Recommended Patterns**:
- Use existing `auto_config_gen.pl` for initial config generation rather than manual creation
- Validate generated processors with real image files, not synthetic data
- Follow Canon implementation pattern in `src/implementations/*/mod.rs` for integration
- Generate cross-module reference stubs proactively to prevent compilation errors

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
- [ ] `make precommit` clean - no linting, compilation, or test errors
- [ ] Coverage reaches 35%+ (current 12.23% + target modules ~23%) measured by subdirectory implementation
- [ ] ExifTool compatibility maintained for all existing functionality
- [ ] At least 3 zero-coverage high-impact modules (Exif, DNG, JPEG) producing working subdirectory extraction

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

**Coverage Measurement Caveats**: The 12.23% coverage metric only checks text mentions, not functional correctness. Real coverage may be lower due to:
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

1. **Start with Exif Module**: Highest subdirectory count (122) and most universal impact
2. **DNG second**: Builds on EXIF foundation, high subdirectory count (94)
3. **JPEG for breadth**: Different processing patterns, good validation target (64 subdirs)
4. **Pentax for manufacturer diversity**: Prove architecture works across camera brands
5. **Iterate based on coverage metrics**: Focus on modules providing highest coverage gains

This plan builds on the mature subdirectory infrastructure to systematically expand coverage by targeting high-impact zero-coverage modules, focusing on configuration generation rather than architecture development.