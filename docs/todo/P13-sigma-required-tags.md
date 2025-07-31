# Technical Project Plan: Sigma Required Tags Implementation

## Project Overview

- **Goal**: Implement comprehensive Sigma/Foveon MakerNotes tag extraction to support all required tags from supported_tags.json
- **Problem**: No Sigma-specific infrastructure exists - missing codegen extraction, tag kit system, and MakerNotes processor implementation
- **Constraints**: Must support Sigma's unique Foveon sensor technology metadata and extensive lens database, handle both legacy and modern Art/Contemporary/Sports series

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

- **No existing infrastructure**: Like Pentax, zero Sigma-specific codegen or processing exists in current codebase
- **Foveon sensor specialist**: Sigma's unique layered sensor technology requires specific metadata extraction different from Bayer sensors
- **Dual camera/lens manufacturer**: Sigma makes both cameras (SD, dp series) and lenses for other manufacturers' cameras
- **Art/Contemporary/Sports taxonomy**: Modern Sigma lens naming with specific performance characteristics

### Key Concepts & Domain Knowledge

- **Foveon sensor metadata**: Unique layered sensor technology captures RGB at each pixel, requiring specific processing metadata
- **Sigma LensType system**: Hexadecimal lens identification (0x10, 0x103, etc.) with complex sub-variants (16.1, 16.2)
- **X3F format**: Sigma's proprietary RAW format for Foveon sensors, different from standard Bayer RAW formats
- **Mount versatility**: Sigma lenses available for Canon, Nikon, Sony, Pentax, L-mount systems with same lens IDs

### Surprising Context

- **Zero infrastructure**: No codegen extraction exists for Sigma.pm unlike other manufacturers
- **Foveon uniqueness**: Processing requirements different from standard Bayer sensor cameras (all other manufacturers)
- **Limited camera production**: Small camera market share but significant lens market presence
- **X3F format detection**: Already exists in file_type_lookup but no MakerNotes processing

### Foundation Documents

- **ExifTool source**: `third-party/exiftool/lib/Image/ExifTool/Sigma.pm` - Main tag table, sigmaLensTypes database lines 25-200+
- **No existing codegen**: `src/generated/Sigma_pm/` directory doesn't exist - needs complete extraction
- **Market position**: Niche camera manufacturer, major lens manufacturer with Art series popularity
- **Start here**: Need complete extraction pipeline: codegen → tag kit → processor implementation

### Prerequisites

- **Knowledge assumed**: Understanding of Foveon vs Bayer sensor differences, X3F format, Sigma lens mount systems
- **Setup required**: Sigma test images (SD series cameras, dp series compacts) for comprehensive validation

**Context Quality Check**: Can a new engineer understand WHY Sigma requires unique processing for Foveon sensor metadata?

## Work Completed

- ✅ **File type detection** → X3F format detection exists in file_type_lookup
- (No other infrastructure exists)

## Remaining Tasks

### 1. Task: Research supported_tags.json requirements for Sigma MakerNotes

**Success Criteria**: Document all MakerNotes tags in supported_tags.json that should come from Sigma cameras
**Approach**: Analyze supported_tags.json for generic MakerNotes tags, cross-reference with Sigma.pm capabilities
**Dependencies**: None - research task

**Success Patterns**:
- ✅ Complete list of required MakerNotes tags that Sigma should provide
- ✅ Understanding of Foveon-specific metadata vs standard camera metadata
- ✅ Priority ranking focusing on Sigma's unique sensor technology tags

### 2. Task: Implement complete Sigma codegen extraction

**Success Criteria**: `src/generated/Sigma_pm/` directory created with comprehensive tag kit system
**Approach**: Add Sigma.pm to codegen extraction system, generate tag tables and sigmaLensTypes database
**Dependencies**: Task 1 (requirements analysis)

**Success Patterns**:
- ✅ src/generated/Sigma_pm/tag_kit/ directory with complete tag definitions
- ✅ sigmaLensTypes database extracted with 100+ lens entries including hex ID system
- ✅ Foveon-specific tag processing requirements documented

### 3. Task: Create Sigma MakerNotes processor implementation

**Success Criteria**: `src/implementations/sigma/mod.rs` processes Sigma MakerNotes using generated tag kit system
**Approach**: Follow Canon/Sony implementation patterns, handle Foveon-specific processing requirements
**Dependencies**: Task 2 (codegen extraction)

**Success Patterns**:
- ✅ process_sigma_makernotes() function extracts standard TIFF IFD MakerNotes
- ✅ Integration with main EXIF processing pipeline
- ✅ Proper namespace handling for MakerNotes group

### 4. Task: Implement Sigma lens identification system

**Success Criteria**: Sigma lens identification using sigmaLensTypes database with hexadecimal ID system
**Approach**: Handle complex hex-based lens identification (0x10, 16.1, 16.2 variants) following ExifTool logic
**Dependencies**: Task 3 (MakerNotes processor)

**Success Patterns**:
- ✅ LensType properly decoded from hexadecimal values to human-readable lens names
- ✅ Sub-variant handling (16.1, 16.2) working correctly for lens model differentiation
- ✅ Art/Contemporary/Sports series lenses properly identified

### 5. Task: Validate against Sigma camera models

**Success Criteria**: Tag extraction working across SD series, dp series cameras with Foveon sensor metadata
**Approach**: Test with sample files from different Sigma camera lines, ensure Foveon-specific processing
**Dependencies**: Task 4 (lens identification)

**Success Patterns**:
- ✅ SD series (DSLR-style with Foveon) extract proper MakerNotes and sensor metadata
- ✅ dp series (compact with Foveon) extract proper MakerNotes and sensor metadata
- ✅ X3F files process correctly with MakerNotes extraction

### 6. RESEARCH: Investigate Foveon sensor metadata requirements

**Objective**: Understand unique Foveon sensor processing metadata vs standard Bayer sensor processing
**Success Criteria**: Document Foveon-specific tags and processing requirements
**Done When**: Clear understanding of Sigma's unique sensor technology metadata needs

## Implementation Guidance

### Recommended Patterns

- **Ground-up implementation**: Complete extraction pipeline needed from scratch like Pentax
- **Foveon-aware processing**: Handle unique sensor technology metadata requirements
- **Hex lens ID system**: Implement complex hexadecimal lens identification with sub-variants

### Tools to Leverage

- **Codegen framework**: Use existing tag kit extraction system for Sigma.pm processing
- **Generic subdirectory processing**: Standard TIFF IFD processing utilities available
- **Lens database patterns**: Follow patterns from other manufacturers' lens identification systems

### Architecture Considerations

- **Foveon metadata isolation**: Ensure Foveon-specific processing doesn't interfere with Bayer sensor processing
- **Namespace isolation**: Sigma MakerNotes must use proper "MakerNotes" group
- **X3F format integration**: Ensure MakerNotes processing works for Sigma's proprietary X3F format

## Integration Requirements

**CRITICAL**: Building without integrating is failure. Don't accept incomplete Sigma support.

Every feature must include:
- [ ] **Activation**: Sigma MakerNotes processing enabled by default for Sigma cameras
- [ ] **Consumption**: Tags appear in standard extraction output without special flags
- [ ] **Measurement**: Can verify Sigma tag extraction with grep on output
- [ ] **Cleanup**: Complete infrastructure built, no placeholder stubs remaining

**Red Flag Check**: If Sigma images show zero MakerNotes tags, implementation is incomplete.

## Working Definition of "Complete"

A feature is complete when:
- ✅ **System behavior changes** - Sigma images now extract MakerNotes tags automatically
- ✅ **Default usage** - No special configuration needed for Sigma tag extraction
- ✅ **Old path removed** - No placeholder infrastructure, complete extraction system working
- ❌ Code exists but extracts zero tags *(example: "processor implemented but no tags appear")*
- ❌ Partial implementation *(example: "basic tags work but lens identification missing")*

## Prerequisites

- EXIF foundation → P10a-exif-required-tags → verify with `cargo t exif_basic`
- Codegen system → verify with existing Canon/Sony tag kit generation

## Testing

- **Unit**: Test Sigma MakerNotes parsing with synthetic IFD data
- **Integration**: Verify tag extraction from SD, dp series sample files
- **Manual check**: Run `cargo run test-images/sigma/sd1.x3f` and confirm MakerNotes tags appear

## Definition of Done

- [ ] `cargo t sigma` passes (if Sigma-specific tests exist)
- [ ] `make precommit` clean
- [ ] Complete Sigma codegen extraction in src/generated/Sigma_pm/
- [ ] Sigma MakerNotes tags appear in extraction output
- [ ] ExifTool comparison shows matching values for critical tags
- [ ] Foveon sensor metadata properly extracted

## Gotchas & Tribal Knowledge

### Sigma-Specific Considerations

- **Complete ground-up build**: Like Pentax, Sigma needs everything built from scratch
- **Foveon sensor uniqueness**: Layered sensor technology requires different metadata processing than Bayer sensors
- **Hex lens ID complexity**: LensType uses hexadecimal values with decimal sub-variants (0x10, 16.1, 16.2)
- **Limited camera market**: Small camera production but significant lens manufacturing presence

### Implementation Challenges

- **X3F format specifics**: Proprietary RAW format may have different MakerNotes structure than JPEG
- **Foveon processing metadata**: Unique sensor technology may require specific tag extraction
- **Lens mount variations**: Same lens available for multiple camera brands with same Sigma ID
- **Art series complexity**: Modern lens naming system with performance classifications

### Codegen Requirements

- **Main tag table**: Extract comprehensive Main table from Sigma.pm
- **Lens database**: Extract sigmaLensTypes with hexadecimal ID system and sub-variants
- **Foveon tags**: Extract sensor-specific metadata tags unique to layered sensor technology

## Quick Debugging

Stuck? Try these:

1. `ls src/generated/Sigma_pm/` - Verify codegen extraction completed
2. `cargo run test-images/sigma/sample.x3f | grep MakerNotes | wc -l` - Count MakerNotes tags
3. `exiftool -j -G test-images/sigma/sample.x3f | jq '.[] | keys[]' | grep MakerNotes` - Compare with ExifTool
4. `rg "process_sigma" src/` - Check if Sigma processor is integrated into main pipeline
5. `rg "sigmaLensType" third-party/exiftool/` - Reference ExifTool lens identification system