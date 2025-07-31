# Technical Project Plan: Casio Required Tags Implementation

## Project Overview

- **Goal**: Implement comprehensive Casio MakerNotes tag extraction to support all required tags from supported_tags.json
- **Problem**: No Casio-specific infrastructure exists - missing codegen extraction, tag kit system, and MakerNotes processor implementation
- **Constraints**: Must support Casio's consumer camera lines and specialized models, handle legacy QV series through modern EX series

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

- **No existing infrastructure**: Zero Casio-specific codegen or processing exists in current codebase
- **Consumer camera manufacturer**: Focus on point-and-shoot, compact cameras with some professional features
- **Legacy support importance**: Many older Casio QV series cameras still in use, requiring backward compatibility
- **Limited current market**: Smaller market share but historical significance in digital camera development

### Key Concepts & Domain Knowledge

- **QV series legacy**: Historical significance as early digital camera pioneer, many QV files still in circulation
- **EX series modern**: Current Casio camera line with advanced features requiring comprehensive tag extraction
- **Consumer focus**: MakerNotes structure optimized for consumer features vs professional camera complexity
- **Archive importance**: Many legacy Casio images in photo archives requiring proper metadata extraction

### Surprising Context

- **Zero infrastructure**: Like other smaller manufacturers, no codegen extraction exists
- **Historical significance**: Casio was important early digital camera manufacturer, many legacy files exist
- **Consumer optimization**: MakerNotes structure may be simpler than professional camera manufacturers
- **Limited documentation**: Less community research compared to major manufacturers like Canon/Nikon

### Foundation Documents

- **ExifTool source**: `third-party/exiftool/lib/Image/ExifTool/Casio.pm` - Main tag table and processing logic
- **No existing codegen**: `src/generated/Casio_pm/` directory doesn't exist - needs complete extraction
- **Market context**: Smaller current market but significant historical presence in digital photography
- **Start here**: Need complete extraction pipeline: codegen → tag kit → processor implementation

### Prerequisites

- **Knowledge assumed**: Understanding of consumer camera vs professional camera MakerNotes differences
- **Setup required**: Casio test images (QV series legacy, EX series modern) for comprehensive validation

**Context Quality Check**: Can a new engineer understand WHY Casio support is needed despite smaller current market share?

## Work Completed

- ✅ **File type detection** → Basic camera detection may exist in file_type_lookup
- (No other infrastructure exists)

## Remaining Tasks

### 1. Task: Research supported_tags.json requirements for Casio MakerNotes

**Success Criteria**: Document all MakerNotes tags in supported_tags.json that should come from Casio cameras
**Approach**: Analyze supported_tags.json for generic MakerNotes tags, cross-reference with Casio.pm capabilities
**Dependencies**: None - research task

**Success Patterns**:
- ✅ Complete list of required MakerNotes tags that Casio should provide
- ✅ Understanding of consumer camera metadata vs professional camera requirements
- ✅ Priority ranking considering legacy file support importance

### 2. Task: Implement complete Casio codegen extraction

**Success Criteria**: `src/generated/Casio_pm/` directory created with comprehensive tag kit system
**Approach**: Add Casio.pm to codegen extraction system, generate tag tables for QV and EX series
**Dependencies**: Task 1 (requirements analysis)

**Success Patterns**:
- ✅ src/generated/Casio_pm/tag_kit/ directory with complete tag definitions
- ✅ Consumer camera tag processing optimized for Casio's simpler MakerNotes structure
- ✅ Legacy QV series compatibility maintained

### 3. Task: Create Casio MakerNotes processor implementation

**Success Criteria**: `src/implementations/casio/mod.rs` processes Casio MakerNotes using generated tag kit system
**Approach**: Follow Canon/Sony implementation patterns, optimize for consumer camera processing
**Dependencies**: Task 2 (codegen extraction)

**Success Patterns**:
- ✅ process_casio_makernotes() function extracts standard TIFF IFD MakerNotes
- ✅ Integration with main EXIF processing pipeline
- ✅ Proper namespace handling for MakerNotes group

### 4. Task: Implement consumer camera feature extraction

**Success Criteria**: Consumer-focused features (scene modes, digital effects, etc.) properly extracted with human-readable names
**Approach**: Focus on consumer camera features that Casio emphasizes in their MakerNotes
**Dependencies**: Task 3 (MakerNotes processor)

**Success Patterns**:
- ✅ Scene mode tags extracted with proper human-readable names
- ✅ Digital effect settings properly documented
- ✅ Consumer-focused metadata (easy sharing, auto-enhancement) extracted

### 5. Task: Validate against Casio camera series

**Success Criteria**: Tag extraction working across legacy QV series and modern EX series cameras
**Approach**: Test with sample files from different Casio generations, ensure broad compatibility
**Dependencies**: Task 4 (consumer features)

**Success Patterns**:
- ✅ Legacy QV series cameras extract proper MakerNotes for archive compatibility
- ✅ Modern EX series cameras extract comprehensive MakerNotes
- ✅ Consumer-focused metadata properly formatted for user comprehension

### 6. RESEARCH: Investigate legacy Casio format compatibility

**Objective**: Ensure proper handling of older Casio camera formats and MakerNotes structures
**Success Criteria**: Legacy compatibility documented and tested
**Done When**: Clear understanding of Casio's evolution in MakerNotes structure over time

## Implementation Guidance

### Recommended Patterns

- **Ground-up implementation**: Complete extraction pipeline needed from scratch
- **Consumer optimization**: Focus on consumer camera features vs professional complexity
- **Legacy compatibility**: Ensure older Casio cameras continue to work properly

### Tools to Leverage

- **Codegen framework**: Use existing tag kit extraction system for Casio.pm processing
- **Generic subdirectory processing**: Standard TIFF IFD processing utilities available
- **Consumer camera patterns**: Optimize for simpler MakerNotes structure vs professional cameras

### Architecture Considerations

- **Consumer focus**: Processing optimized for consumer camera metadata vs professional complexity
- **Namespace isolation**: Casio MakerNotes must use proper "MakerNotes" group
- **Legacy support**: Ensure processing works across Casio's camera evolution

## Integration Requirements

**CRITICAL**: Building without integrating is failure. Don't accept incomplete Casio support.

Every feature must include:
- [ ] **Activation**: Casio MakerNotes processing enabled by default for Casio cameras
- [ ] **Consumption**: Tags appear in standard extraction output without special flags
- [ ] **Measurement**: Can verify Casio tag extraction with grep on output
- [ ] **Cleanup**: Complete infrastructure built, no placeholder stubs remaining

**Red Flag Check**: If Casio images show zero MakerNotes tags, implementation is incomplete.

## Working Definition of "Complete"

A feature is complete when:
- ✅ **System behavior changes** - Casio images now extract MakerNotes tags automatically
- ✅ **Default usage** - No special configuration needed for Casio tag extraction
- ✅ **Old path removed** - No placeholder infrastructure, complete extraction system working
- ❌ Code exists but extracts zero tags *(example: "processor implemented but no tags appear")*
- ❌ Partial implementation *(example: "modern cameras work but legacy QV series fails")*

## Prerequisites

- EXIF foundation → P10a-exif-required-tags → verify with `cargo t exif_basic`
- Codegen system → verify with existing Canon/Sony tag kit generation

## Testing

- **Unit**: Test Casio MakerNotes parsing with synthetic IFD data
- **Integration**: Verify tag extraction from QV and EX series sample files
- **Manual check**: Run `cargo run test-images/casio/qv-sample.jpg` and confirm MakerNotes tags appear

## Definition of Done

- [ ] `cargo t casio` passes (if Casio-specific tests exist)
- [ ] `make precommit` clean
- [ ] Complete Casio codegen extraction in src/generated/Casio_pm/
- [ ] Casio MakerNotes tags appear in extraction output
- [ ] ExifTool comparison shows matching values for critical tags
- [ ] Legacy QV series compatibility maintained

## Gotchas & Tribal Knowledge

### Casio-Specific Considerations

- **Complete ground-up build**: Casio needs everything built from scratch
- **Consumer camera focus**: Simpler MakerNotes structure vs professional camera complexity
- **Legacy compatibility**: QV series may have different MakerNotes structure than modern EX series
- **Limited market presence**: Smaller current market but important for archive compatibility

### Implementation Challenges

- **Legacy format variations**: Older Casio cameras may use different MakerNotes structures
- **Consumer feature focus**: Different metadata priorities than professional cameras
- **Limited documentation**: Less community research available vs major manufacturers
- **Archive importance**: Many historical Casio files require proper metadata extraction

### Codegen Requirements

- **Main tag table**: Extract comprehensive Main table from Casio.pm
- **Consumer features**: Extract scene modes, digital effects, and consumer-focused settings
- **Legacy compatibility**: Ensure extraction works across Casio's camera evolution

## Quick Debugging

Stuck? Try these:

1. `ls src/generated/Casio_pm/` - Verify codegen extraction completed
2. `cargo run test-images/casio/sample.jpg | grep MakerNotes | wc -l` - Count MakerNotes tags
3. `exiftool -j -G test-images/casio/sample.jpg | jq '.[] | keys[]' | grep MakerNotes` - Compare with ExifTool
4. `rg "process_casio" src/` - Check if Casio processor is integrated into main pipeline
5. `rg "Casio" third-party/exiftool/` - Reference ExifTool implementation for consumer camera features