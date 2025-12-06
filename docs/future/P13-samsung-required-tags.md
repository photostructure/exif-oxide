# Technical Project Plan: Samsung Required Tags Implementation

## Project Overview

- **Goal**: Implement comprehensive Samsung MakerNotes tag extraction to support all required tags from supported_tags.json
- **Problem**: No Samsung-specific infrastructure exists - missing codegen extraction, tag kit system, and MakerNotes processor implementation
- **Constraints**: Must support legacy Samsung NX series cameras and older digital cameras, handle discontinued but still-used camera systems

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

- **No existing infrastructure**: Zero Samsung-specific codegen or processing exists in current codebase
- **Legacy NX system**: Samsung's discontinued mirrorless camera system still in use by photographers
- **Archive importance**: Many Samsung camera files in photo archives requiring proper metadata extraction
- **Discontinued manufacturer**: No new cameras but existing files need continued support

### Key Concepts & Domain Knowledge

- **NX series importance**: Samsung's mirrorless camera system with dedicated user base despite discontinuation
- **Legacy digital cameras**: Earlier Samsung point-and-shoot and compact cameras still in circulation
- **Archive compatibility**: Critical for photo management systems to handle existing Samsung camera files
- **Unique lens system**: NX mount lenses with specific identification requirements

### Surprising Context

- **Zero infrastructure**: Like other smaller manufacturers, no codegen extraction exists
- **Discontinued but active**: No new Samsung cameras but significant existing file base
- **NX system quality**: Professional-level mirrorless system with sophisticated MakerNotes requirements
- **Archive priority**: Focus on supporting existing files rather than new camera development

### Foundation Documents

- **ExifTool source**: `third-party/exiftool/lib/Image/ExifTool/Samsung.pm` - Main tag table and NX series processing
- **No existing codegen**: `src/generated/Samsung_pm/` directory doesn't exist - needs complete extraction
- **Market context**: Discontinued manufacturer but significant archive of existing files
- **Start here**: Need complete extraction pipeline: codegen → tag kit → processor implementation

### Prerequisites

- **Knowledge assumed**: Understanding of discontinued camera system support, NX mount lens system
- **Setup required**: Samsung test images (NX series mirrorless, legacy compact cameras) for comprehensive validation

**Context Quality Check**: Can a new engineer understand WHY Samsung support is needed despite discontinued camera production?

## Work Completed

- ✅ **File type detection** → Basic camera detection may exist in file_type_lookup
- (No other infrastructure exists)

## Remaining Tasks

### 1. Task: Research supported_tags.json requirements for Samsung MakerNotes

**Success Criteria**: Document all MakerNotes tags in supported_tags.json that should come from Samsung cameras
**Approach**: Analyze supported_tags.json for generic MakerNotes tags, cross-reference with Samsung.pm capabilities
**Dependencies**: None - research task

**Success Patterns**:
- ✅ Complete list of required MakerNotes tags that Samsung should provide
- ✅ Understanding of NX series professional features vs legacy compact camera metadata
- ✅ Priority ranking focusing on archive compatibility importance

### 2. Task: Implement complete Samsung codegen extraction

**Success Criteria**: `src/generated/Samsung_pm/` directory created with comprehensive tag kit system
**Approach**: Add Samsung.pm to codegen extraction system, generate tag tables for NX series and legacy cameras
**Dependencies**: Task 1 (requirements analysis)

**Success Patterns**:
- ✅ src/generated/Samsung_pm/tag_kit/ directory with complete tag definitions
- ✅ NX series professional camera tag processing implemented
- ✅ Legacy Samsung camera compatibility maintained

### 3. Task: Create Samsung MakerNotes processor implementation

**Success Criteria**: `src/implementations/samsung/mod.rs` processes Samsung MakerNotes using generated tag kit system
**Approach**: Follow Canon/Sony implementation patterns, handle both professional NX series and consumer legacy cameras
**Dependencies**: Task 2 (codegen extraction)

**Success Patterns**:
- ✅ process_samsung_makernotes() function extracts standard TIFF IFD MakerNotes
- ✅ Integration with main EXIF processing pipeline
- ✅ Proper namespace handling for MakerNotes group

### 4. Task: Implement Samsung lens identification system

**Success Criteria**: Samsung NX lens identification properly extracted with human-readable names
**Approach**: Handle NX mount lens identification system from Samsung MakerNotes
**Dependencies**: Task 3 (MakerNotes processor)

**Success Patterns**:
- ✅ NX lens identification working with proper lens names
- ✅ Legacy Samsung camera lens information extracted where available
- ✅ Professional camera metadata properly formatted

### 5. Task: Validate against Samsung camera series

**Success Criteria**: Tag extraction working across NX series mirrorless and legacy Samsung cameras
**Approach**: Test with sample files from different Samsung camera generations, ensure broad compatibility
**Dependencies**: Task 4 (lens identification)

**Success Patterns**:
- ✅ NX series cameras (NX1, NX500, etc.) extract comprehensive professional MakerNotes
- ✅ Legacy Samsung cameras extract appropriate MakerNotes for archive compatibility
- ✅ Professional and consumer metadata properly differentiated

### 6. RESEARCH: Investigate Samsung archive format compatibility

**Objective**: Ensure proper handling of Samsung's camera format evolution and archive requirements
**Success Criteria**: Archive compatibility documented and tested across Samsung camera generations
**Done When**: Clear understanding of Samsung's MakerNotes evolution and archive support needs

## Implementation Guidance

### Recommended Patterns

- **Ground-up implementation**: Complete extraction pipeline needed from scratch
- **Archive focus**: Optimize for existing file support rather than new camera features
- **NX system priority**: Focus on professional NX series features while maintaining legacy support

### Tools to Leverage

- **Codegen framework**: Use existing tag kit extraction system for Samsung.pm processing
- **Generic subdirectory processing**: Standard TIFF IFD processing utilities available
- **Archive compatibility patterns**: Focus on supporting existing files vs new development

### Architecture Considerations

- **Archive optimization**: Processing optimized for existing file support vs new camera development
- **Namespace isolation**: Samsung MakerNotes must use proper "MakerNotes" group
- **Legacy support**: Ensure processing works across Samsung's discontinued camera lines

## Integration Requirements

**CRITICAL**: Building without integrating is failure. Don't accept incomplete Samsung support.

Every feature must include:
- [ ] **Activation**: Samsung MakerNotes processing enabled by default for Samsung cameras
- [ ] **Consumption**: Tags appear in standard extraction output without special flags
- [ ] **Measurement**: Can verify Samsung tag extraction with grep on output
- [ ] **Cleanup**: Complete infrastructure built, no placeholder stubs remaining

**Red Flag Check**: If Samsung images show zero MakerNotes tags, implementation is incomplete.

## Working Definition of "Complete"

A feature is complete when:
- ✅ **System behavior changes** - Samsung images now extract MakerNotes tags automatically
- ✅ **Default usage** - No special configuration needed for Samsung tag extraction
- ✅ **Old path removed** - No placeholder infrastructure, complete extraction system working
- ❌ Code exists but extracts zero tags *(example: "processor implemented but no tags appear")*
- ❌ Partial implementation *(example: "NX series works but legacy cameras fail")*

## Prerequisites

- EXIF foundation → P10a-exif-required-tags → verify with `cargo t exif_basic`
- Codegen system → verify with existing Canon/Sony tag kit generation

## Testing

- **Unit**: Test Samsung MakerNotes parsing with synthetic IFD data
- **Integration**: Verify tag extraction from NX series and legacy Samsung sample files
- **Manual check**: Run `cargo run test-images/samsung/nx1.jpg` and confirm MakerNotes tags appear

## Definition of Done

- [ ] `cargo t samsung` passes (if Samsung-specific tests exist)
- [ ] `make precommit` clean
- [ ] Complete Samsung codegen extraction in src/generated/Samsung_pm/
- [ ] Samsung MakerNotes tags appear in extraction output
- [ ] ExifTool comparison shows matching values for critical tags
- [ ] Archive compatibility maintained across Samsung camera generations

## Gotchas & Tribal Knowledge

### Samsung-Specific Considerations

- **Complete ground-up build**: Samsung needs everything built from scratch
- **Discontinued manufacturer**: Focus on archive support rather than new development
- **NX system complexity**: Professional mirrorless system with sophisticated MakerNotes requirements
- **Legacy compatibility**: Various Samsung camera generations may have different MakerNotes structures

### Implementation Challenges

- **Archive priority**: Supporting existing files more important than new camera features
- **NX lens system**: Unique mount system with specific identification requirements
- **Format evolution**: Samsung camera MakerNotes may have evolved significantly over time
- **Limited community**: Smaller user base compared to active manufacturers

### Codegen Requirements

- **Main tag table**: Extract comprehensive Main table from Samsung.pm
- **NX series features**: Extract professional camera features from mirrorless system
- **Legacy compatibility**: Ensure extraction works across Samsung's camera evolution

## Quick Debugging

Stuck? Try these:

1. `ls src/generated/Samsung_pm/` - Verify codegen extraction completed
2. `cargo run test-images/samsung/sample.jpg | grep MakerNotes | wc -l` - Count MakerNotes tags
3. `exiftool -j -G test-images/samsung/sample.jpg | jq '.[] | keys[]' | grep MakerNotes` - Compare with ExifTool
4. `rg "process_samsung" src/` - Check if Samsung processor is integrated into main pipeline
5. `rg "Samsung" third-party/exiftool/` - Reference ExifTool implementation for NX series and legacy cameras