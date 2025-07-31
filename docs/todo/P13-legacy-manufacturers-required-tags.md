# Technical Project Plan: Legacy Manufacturers Required Tags Implementation

## Project Overview

- **Goal**: Implement comprehensive MakerNotes tag extraction for legacy camera manufacturers (Kodak, Minolta, Leaf, Lytro, Motorola) to support archive compatibility
- **Problem**: No infrastructure exists for these manufacturers - missing codegen extraction, tag kit systems, and MakerNotes processors
- **Constraints**: Focus on archive compatibility and existing file support rather than new development, handle discontinued camera systems

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

- **Archive-focused implementation**: All target manufacturers are discontinued or specialized, focus on existing file support
- **Legacy importance**: Significant archives of files from these manufacturers require proper metadata extraction
- **Specialized technologies**: Each manufacturer had unique technologies requiring specific metadata handling
- **Low maintenance priority**: Implement once for archive compatibility, minimal ongoing development expected

### Manufacturer-Specific Context

#### Kodak
- **Historical significance**: Major film and early digital camera manufacturer with extensive archives
- **Professional focus**: High-end digital cameras with sophisticated MakerNotes
- **DCS series importance**: Professional digital SLR cameras still used in professional archives

#### Minolta
- **Sony acquisition**: Merged into Sony but legacy files still in circulation
- **AF system innovation**: Advanced autofocus technology requiring specific metadata extraction
- **Archive compatibility**: Important for photo management systems handling legacy files

#### Leaf
- **Medium format specialty**: High-end medium format digital backs with professional metadata
- **Professional archives**: Critical for professional photography archives and studios
- **Unique file formats**: Specialized RAW formats requiring specific handling

#### Lytro
- **Light field technology**: Revolutionary but discontinued light field cameras
- **Unique metadata**: Light field capture requires specialized metadata extraction
- **Innovation archive**: Important for preserving early light field photography

#### Motorola
- **Mobile photography**: Early smartphone cameras with basic but important metadata
- **Historical mobile**: Early mobile photography metadata for digital archives
- **Limited complexity**: Simpler MakerNotes structure compared to dedicated cameras

### Key Concepts & Domain Knowledge

- **Archive priority**: Focus on supporting existing files rather than new camera development
- **Technology preservation**: Each manufacturer represents important technological developments in photography
- **Metadata completeness**: Professional archives require comprehensive metadata extraction
- **Legacy format support**: Ensure compatibility with discontinued but important file formats

### Surprising Context

- **Zero infrastructure**: None of these manufacturers have any codegen or processing infrastructure
- **Archive importance**: Despite discontinued status, significant file archives exist requiring support
- **Technology diversity**: Each manufacturer had unique innovations requiring specialized handling
- **Low complexity assumption**: Some manufacturers may have simpler MakerNotes than current assumptions

### Foundation Documents

- **ExifTool sources**: 
  - `third-party/exiftool/lib/Image/ExifTool/Kodak.pm`
  - `third-party/exiftool/lib/Image/ExifTool/Minolta.pm`  
  - `third-party/exiftool/lib/Image/ExifTool/Leaf.pm`
  - `third-party/exiftool/lib/Image/ExifTool/Lytro.pm`
  - `third-party/exiftool/lib/Image/ExifTool/Motorola.pm`
- **No existing codegen**: All `src/generated/*_pm/` directories missing for these manufacturers
- **Archive context**: Focus on file compatibility rather than new camera support

### Prerequisites

- **Knowledge assumed**: Understanding of discontinued camera system support, archive compatibility requirements
- **Setup required**: Sample files from each manufacturer for comprehensive validation testing

**Context Quality Check**: Can a new engineer understand WHY legacy manufacturer support is important for archive systems?

## Work Completed

- ✅ **File type detection** → Basic format detection may exist in file_type_lookup for some manufacturers
- (No other infrastructure exists for any manufacturer)

## Remaining Tasks

### 1. Task: Research supported_tags.json requirements for legacy manufacturers

**Success Criteria**: Document all MakerNotes tags in supported_tags.json that should come from legacy manufacturer cameras
**Approach**: Analyze supported_tags.json for generic MakerNotes tags, prioritize by manufacturer archive importance
**Dependencies**: None - research task

**Success Patterns**:
- ✅ Complete list of required MakerNotes tags by manufacturer priority
- ✅ Understanding of archive compatibility vs new development priorities
- ✅ Priority ranking: Kodak > Minolta > Leaf > Lytro > Motorola based on archive volume

### 2. Task: Implement codegen extraction for priority manufacturers

**Success Criteria**: Codegen extraction implemented for Kodak and Minolta first (highest archive priority)
**Approach**: Add manufacturer .pm files to codegen system in priority order
**Dependencies**: Task 1 (requirements analysis)

**Success Patterns**:
- ✅ src/generated/Kodak_pm/tag_kit/ directory with comprehensive tag definitions
- ✅ src/generated/Minolta_pm/tag_kit/ directory with comprehensive tag definitions
- ✅ Professional camera tag processing for DCS series and Minolta AF systems

### 3. Task: Create MakerNotes processors for priority manufacturers

**Success Criteria**: Working processors for Kodak and Minolta using generated tag kit systems
**Approach**: Follow established implementation patterns, optimize for archive compatibility
**Dependencies**: Task 2 (codegen extraction)

**Success Patterns**:
- ✅ process_kodak_makernotes() and process_minolta_makernotes() functions working
- ✅ Integration with main EXIF processing pipeline
- ✅ Proper namespace handling for MakerNotes group

### 4. Task: Implement specialized manufacturer support

**Success Criteria**: Codegen and processors for Leaf, Lytro, and Motorola with their unique requirements
**Approach**: Handle each manufacturer's specialized technology requirements
**Dependencies**: Task 3 (priority manufacturer processors)

**Success Patterns**:
- ✅ Leaf medium format metadata properly extracted
- ✅ Lytro light field metadata properly handled
- ✅ Motorola mobile camera metadata extracted for archive compatibility

### 5. Task: Validate archive compatibility across manufacturers

**Success Criteria**: Tag extraction working across all legacy manufacturers with proper archive support
**Approach**: Test with sample files from each manufacturer, ensure broad archive compatibility
**Dependencies**: Task 4 (specialized manufacturer support)

**Success Patterns**:
- ✅ Kodak DCS series and consumer cameras extract comprehensive MakerNotes
- ✅ Minolta AF system metadata properly extracted for Sony compatibility
- ✅ Leaf medium format files extract professional metadata
- ✅ Lytro light field files extract specialized metadata
- ✅ Motorola mobile files extract basic but complete metadata

### 6. RESEARCH: Archive integration testing

**Objective**: Validate integration with photo management systems and archive workflows
**Success Criteria**: Legacy manufacturer support properly integrated for archive use cases
**Done When**: Archive compatibility confirmed across all supported legacy manufacturers

## Implementation Guidance

### Recommended Patterns

- **Archive-first implementation**: Optimize for existing file support rather than new features
- **Priority-based rollout**: Implement Kodak/Minolta first, then Leaf/Lytro/Motorola
- **Technology-specific handling**: Each manufacturer's unique technology requires specialized approach

### Tools to Leverage

- **Codegen framework**: Use existing tag kit extraction system for all manufacturer .pm files
- **Generic subdirectory processing**: Standard TIFF IFD processing utilities available
- **Archive compatibility patterns**: Focus on comprehensive metadata extraction for archives

### Architecture Considerations

- **Archive optimization**: Processing optimized for comprehensive metadata extraction
- **Namespace isolation**: Each manufacturer's MakerNotes must use proper "MakerNotes" group
- **Technology preservation**: Maintain unique manufacturer technology metadata

## Integration Requirements

**CRITICAL**: Building without integrating is failure. Don't accept incomplete legacy support.

Every feature must include:
- [ ] **Activation**: Legacy manufacturer MakerNotes processing enabled by default
- [ ] **Consumption**: Tags appear in standard extraction output without special flags
- [ ] **Measurement**: Can verify manufacturer tag extraction with grep on output
- [ ] **Cleanup**: Complete infrastructure built for archive compatibility

**Red Flag Check**: If legacy manufacturer images show zero MakerNotes tags, implementation is incomplete.

## Working Definition of "Complete"

A feature is complete when:
- ✅ **System behavior changes** - Legacy manufacturer images now extract MakerNotes tags automatically
- ✅ **Default usage** - No special configuration needed for legacy manufacturer tag extraction
- ✅ **Old path removed** - Complete extraction system working for archive compatibility
- ❌ Code exists but extracts zero tags *(example: "processor implemented but no tags appear")*
- ❌ Partial implementation *(example: "Kodak works but Minolta fails")*

## Prerequisites

- EXIF foundation → P10a-exif-required-tags → verify with `cargo t exif_basic`
- Codegen system → verify with existing Canon/Sony tag kit generation

## Testing

- **Unit**: Test legacy manufacturer MakerNotes parsing with synthetic IFD data
- **Integration**: Verify tag extraction from sample files across all manufacturers
- **Manual check**: Run `cargo run test-images/kodak/dcs-sample.jpg` and confirm MakerNotes tags appear

## Definition of Done

- [ ] `cargo t legacy_manufacturers` passes (if tests exist)
- [ ] `make precommit` clean
- [ ] Complete codegen extraction in src/generated/{Kodak,Minolta,Leaf,Lytro,Motorola}_pm/
- [ ] All legacy manufacturer MakerNotes tags appear in extraction output
- [ ] ExifTool comparison shows matching values for critical tags
- [ ] Archive compatibility validated across all manufacturers

## Gotchas & Tribal Knowledge

### Legacy Manufacturer Considerations

- **Archive priority**: Focus on comprehensive metadata extraction for existing files
- **Technology diversity**: Each manufacturer had unique innovations requiring specialized handling
- **Documentation scarcity**: Less community research available for discontinued manufacturers
- **Priority implementation**: Kodak and Minolta higher priority due to larger archive volumes

### Implementation Challenges

- **Discontinued technology**: Understanding obsolete camera technologies for proper metadata extraction
- **Format variations**: Legacy formats may have unusual MakerNotes structures
- **Limited testing**: Fewer available sample files for validation testing
- **Archive requirements**: Professional archives need comprehensive metadata extraction

### Codegen Requirements

- **All manufacturer tables**: Extract Main tables from all five manufacturer .pm files
- **Technology-specific features**: Handle each manufacturer's unique technology metadata
- **Archive compatibility**: Ensure extraction supports comprehensive archive metadata needs

## Quick Debugging

Stuck? Try these:

1. `ls src/generated/{Kodak,Minolta,Leaf,Lytro,Motorola}_pm/` - Verify codegen extraction completed
2. `cargo run test-images/manufacturer/sample.jpg | grep MakerNotes | wc -l` - Count MakerNotes tags
3. `exiftool -j -G test-images/manufacturer/sample.jpg | jq '.[] | keys[]' | grep MakerNotes` - Compare with ExifTool
4. `rg "process_(kodak|minolta|leaf|lytro|motorola)" src/` - Check if processors integrated
5. `rg "(Kodak|Minolta|Leaf|Lytro|Motorola)" third-party/exiftool/` - Reference ExifTool implementations