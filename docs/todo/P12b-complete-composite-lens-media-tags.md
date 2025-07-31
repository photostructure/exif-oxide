# Technical Project Plan: Complete Composite Lens & Media Tags

## Project Overview

- **Goal**: Complete the remaining 2 blocked composite tags (LensType, Duration) to achieve 100% implementation of all 6 required composite tags from P12b scope
- **Problem**: P20c delivered 60% completion - 4/6 composite tags working, but LensType and Duration blocked by missing codegen configurations
- **Constraints**: Zero breaking changes to existing 4 working composite tags, maintain ExifTool fidelity

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

- **P20c Module-Specific Composite Extraction**: Successfully delivered extraction infrastructure for Canon, Nikon, and QuickTime modules. System can extract composite tags from module-specific `%ModuleName::Composite` tables.
- **Multi-pass Composite Building**: Sophisticated dependency resolution system supporting composite-on-composite dependencies with circular dependency detection and performance monitoring.
- **Complete Implementation Infrastructure**: All 6 target composite tags have correct implementations in `src/composite_tags/implementations.rs` and proper dispatch routing in `src/composite_tags/dispatch.rs`.

### Key Concepts & Domain Knowledge

- **Composite Tags**: Calculated tags derived from other tags (e.g., Lens combines MinFocalLength + MaxFocalLength)
- **Module-Specific Composites**: Unlike main `%Image::ExifTool::Composite` table, some composites are defined in manufacturer modules (`%Canon::Composite`, `%Olympus::Composite`, etc.)
- **Required vs Mainstream**: All 6 target composite tags are marked `required: true` in `docs/tag-metadata.json` - not optional features

### Surprising Context

**CRITICAL**: The main problem is NOT missing implementations - all code exists and works correctly:

- **All implementations exist**: Complete, tested functions in `src/composite_tags/implementations.rs` (lines 1078-1412)
- **All dispatch routes configured**: Proper routing in `src/composite_tags/dispatch.rs` (lines 87, 100-103, 106)
- **P20c delivered substantial work**: 4/6 composite tags working, module-specific extraction infrastructure complete
- **Root cause is codegen configs**: Missing 7 simple JSON configuration files prevents definition generation

**This is NOT a complex implementation task** - it's a straightforward codegen configuration completion.

### Foundation Documents

- **Design docs**: [CODEGEN.md](CODEGEN.md) - Module-specific composite extraction system
- **ExifTool source**: 
  - `third-party/exiftool/lib/Image/ExifTool/Olympus.pm:4290-4299` (LensType)
  - Media modules: FLAC.pm, APE.pm, AIFF.pm, RIFF.pm, MPEG.pm, Vorbis.pm (Duration)
- **Start here**: `codegen/config/` directory structure and existing module configs

### Prerequisites

- **P20c completion**: Module-specific composite extraction infrastructure working
- **Ultra-deep research validation**: All implementation functions verified correct (July 30, 2025)

## Work Completed

- ✅ **P20c Infrastructure** → Module-specific composite extraction working for Canon, Nikon, QuickTime
- ✅ **4/6 Composite Tags Working** → Lens, LensID, LensSpec, Rotation have generated definitions and work end-to-end
- ✅ **All Implementations Complete** → compute_lens_type() and compute_duration() functions exist and correctly implement ExifTool algorithms
- ✅ **All Dispatch Routes** → All 6 composite tags properly routed in dispatch.rs
- ✅ **Comprehensive Testing** → Multi-pass dependency resolution, integration tests, performance monitoring all working

## Remaining Tasks

### 1. Task: Create Missing Olympus Composite Config

**Success Criteria**: LensType composite tag appears in `src/generated/composite_tags.rs` after `make codegen`
**Approach**: Create `codegen/config/Olympus_pm/composite_tags.json` following existing Canon/Nikon pattern
**Dependencies**: None

**Success Patterns**:
- ✅ Config matches existing module composite config structure
- ✅ Extraction generates `olympus__composite_tags.json` in `codegen/generated/extract/composite_tags/`
- ✅ LensType definition appears in generated composite_tags.rs with correct Olympus dependencies

**Implementation**: Create config file pointing to Olympus.pm with standard composite extraction settings.

### 2. Task: Create Missing Media Module Composite Configs  

**Success Criteria**: Duration composite tag appears in `src/generated/composite_tags.rs` from media modules
**Approach**: Create composite_tags.json configs for 6 media format modules
**Dependencies**: Task 1 can run in parallel

**Success Patterns**:
- ✅ All 6 media module configs created: FLAC_pm, APE_pm, AIFF_pm, RIFF_pm, MPEG_pm, Vorbis_pm
- ✅ Extraction generates duration composite definitions from multiple modules
- ✅ Duration composite appears in generated composite_tags.rs with media-specific dependencies

**Implementation**: Create 6 config files following QuickTime composite config pattern, each pointing to respective media module.

### 3. Task: Regenerate Composite Definitions

**Success Criteria**: Both LensType and Duration composite tags present in generated definitions
**Approach**: Run `make codegen` to extract from new module configs
**Dependencies**: Tasks 1 and 2 must be complete

**Success Patterns**:
- ✅ `make codegen` runs without errors
- ✅ Both missing composite tags now present in `src/generated/composite_tags.rs`
- ✅ Generated definitions have correct dependencies matching ExifTool source

### 4. Task: End-to-End Integration Testing

**Success Criteria**: All 6 composite tags work with real test files using focused testing system
**Approach**: Use existing focused testing infrastructure with real image files
**Dependencies**: Task 3 must be complete

**Success Patterns**:
- ✅ `TAGS_FILTER="Composite:Lens,Composite:LensID,Composite:LensSpec,Composite:LensType,Composite:Duration,Composite:Rotation" make compat-tags` shows all 6 tags working
- ✅ Canon T3i JPEG shows Canon lens composites (Lens, LensID)
- ✅ Panasonic RW2 shows LensType composite
- ✅ Audio/video files show Duration composite

## Implementation Guidance

### Recommended Patterns

**Config File Pattern**: Use existing successful configs as templates:
- **Olympus config**: Copy `codegen/config/Canon_pm/composite_tags.json` structure, change source to Olympus.pm
- **Media configs**: Copy `codegen/config/QuickTime_pm/composite_tags.json` structure, change source paths

**Extraction Validation**: After `make codegen`, check for generated extraction files:
- `codegen/generated/extract/composite_tags/olympus__composite_tags.json` should contain LensType
- Media module extractions should contain Duration definitions

**Testing Strategy**: Use the focused testing system implemented in P12b:
```bash
# Test all 6 target composite tags together
TAGS_FILTER="Composite:Lens,Composite:LensID,Composite:LensSpec,Composite:LensType,Composite:Duration,Composite:Rotation" make compat-tags

# Test individual composites during development
TAGS_FILTER="Composite:LensType" make compat-tags
TAGS_FILTER="Composite:Duration" make compat-tags
```

## Integration Requirements

**CRITICAL**: Building without integrating is failure. Don't accept tasks that build "shelf-ware."

Every feature must include:
- [x] **Activation**: Composite tags are enabled by default in composite orchestration system
- [x] **Consumption**: Existing composite building system automatically uses new definitions
- [x] **Measurement**: Can prove composites work via focused testing with real files
- [ ] **Cleanup**: Missing composite tag definitions eliminated (LensType, Duration now present)

**Red Flag Check**: This task completes missing pieces of an existing system - all integration points already exist.

## Working Definition of "Complete"

A feature is complete when:
- ✅ **System behavior changes** - LensType and Duration composites now work where they previously failed  
- ✅ **Default usage** - Composite tags automatically computed when dependencies available
- ✅ **Old path removed** - No more missing required composite tags
- ❌ Code exists but isn't used *(prevented by requiring integration testing)*
- ❌ Feature works "if you call it directly" *(prevented by using existing composite orchestration)*

## Prerequisites

- **P20c completion** → Module-specific composite extraction → verify Canon/Nikon/QuickTime configs work
- **Ultra-deep research** → Implementation analysis complete → all functions verified correct

## Testing

- **Unit**: Verify each config generates expected extraction JSON files
- **Integration**: End-to-end test with real files using focused testing system  
- **Manual check**: Run `TAGS_FILTER` commands to confirm all 6 composite tags working

## Definition of Done

- [ ] `TAGS_FILTER="Composite:Lens,Composite:LensID,Composite:LensSpec,Composite:LensType,Composite:Duration,Composite:Rotation" make compat-tags` shows all 6 tags working
- [ ] `make precommit` clean
- [ ] LensType composite works with Panasonic RW2 files  
- [ ] Duration composite works with audio/video files
- [ ] All 6 required composite tags from tag-metadata.json are functional

## Current State Summary (July 30, 2025)

**Ultra-Deep Research Completed**: Comprehensive analysis of entire composite tag system revealed actual status:

| Composite Tag | Generated Definition | Implementation | Dispatch | Test Status | Blocker |
|---------------|---------------------|----------------|----------|-------------|---------|
| Lens | ✅ Canon-specific | ✅ Complete | ✅ Routed | ✅ Working | None |
| LensID | ✅ 2 variants | ✅ Complete | ✅ Routed | ✅ Working | None |
| LensSpec | ✅ Nikon-specific | ✅ Complete | ✅ Routed | ✅ Working | None |
| Rotation | ✅ QuickTime-specific | ✅ Complete | ✅ Routed | ✅ Working | None |
| LensType | ❌ Missing | ✅ Complete | ✅ Routed | ❌ Blocked | Missing Olympus config |
| Duration | ❌ Missing | ✅ Complete | ✅ Routed | ❌ Blocked | Missing media configs |

**Root Cause Identified**: Missing 7 codegen configuration files prevents definition generation for 2/6 composite tags.

**Next Steps**: Create missing configs → run codegen → test integration → completion.

## Gotchas & Tribal Knowledge

**Format**: Surprise → Why → Solution

- **P20c claimed completion but 2 tags missing** → Codegen only covered 3 modules, missed Olympus + media → Create missing module configs, don't reimplement existing code
- **All implementations exist but tags don't work** → Generated definitions missing, not implementation bugs → Fix codegen configs, don't debug implementation functions  
- **Duration needs 6 different modules** → Each audio/video format has separate Duration composite → Create configs for all relevant media modules
- **LensType is Panasonic-specific despite being in Olympus module** → ExifTool's historical organization → Trust the source, create Olympus config as-is

## Quick Debugging

Stuck? Try these:

1. `ls codegen/config/*/composite_tags.json` - See which module configs exist
2. `make codegen 2>&1 | grep composite` - Check composite extraction output
3. `ls codegen/generated/extract/composite_tags/` - Verify extraction files generated
4. `grep -n "LensType\|Duration" src/generated/composite_tags.rs` - Check if definitions generated
5. `TAGS_FILTER="Composite:LensType" make compat-tags` - Test specific composite