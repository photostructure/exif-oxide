# Technical Project Plan: Panasonic Required Tags Implementation

## Project Overview

- **Goal**: Complete Panasonic MakerNotes tag extraction to support all required tags from supported_tags.json, focusing on standard cameras and Leica co-branded models
- **Problem**: Comprehensive infrastructure exists (tag kit, PanasonicRaw implementation) but missing main Panasonic MakerNotes processor integration
- **Constraints**: Must support both Panasonic and Leica branded cameras sharing the same tag structure, handle complex lens identification system

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

- **Tag kit system**: Complete Panasonic tag kit extraction exists in `src/generated/Panasonic_pm/tag_kit/` with comprehensive tag definitions
- **PanasonicRaw support**: Separate PanasonicRaw implementation exists for RW2 format with PrintConv functions already implemented  
- **Leica integration**: Panasonic.pm handles both Panasonic and Leica cameras with shared MakerNotes structure and extensive lens database
- **Market position**: Major Micro Four Thirds manufacturer, full-frame L-mount alliance partner with Leica and Sigma

### Key Concepts & Domain Knowledge

- **Dual brand support**: Same MakerNotes structure used by Panasonic Lumix and Leica cameras, requiring unified processing approach
- **Extensive lens database**: Complex lens identification system with 150+ entries including Leica rangefinder lenses and modern Lumix lenses
- **PanasonicRaw vs standard**: RW2 files need different processing from standard JPEG/video MakerNotes (separate implementations)
- **supported_tags.json requirements**: 4 specific PanasonicRaw tags already defined: ApertureValue, JpgFromRaw2, Orientation, ShutterSpeedValue

### Surprising Context

- **Infrastructure almost complete**: Tag kit extraction fully implemented, only missing main MakerNotes processor integration
- **Separate PanasonicRaw module**: RW2 format already has working implementation with PrintConv functions - separate from main MakerNotes
- **Leica complexity**: Same module handles both consumer Panasonic and premium Leica cameras with different naming/branding conventions
- **Video metadata**: Panasonic has extensive video capabilities requiring specific tag extraction for hybrid photo/video cameras

### Foundation Documents

- **ExifTool source**: `third-party/exiftool/lib/Image/ExifTool/Panasonic.pm` - Main table starts line ~300, extensive lens database lines 46-200+
- **Generated infrastructure**: `src/generated/Panasonic_pm/tag_kit/` contains complete tag extraction system ready for integration
- **Existing implementation**: `src/implementations/panasonic_raw.rs` shows working PrintConv pattern for RW2 format
- **Start here**: Need to create main Panasonic MakerNotes processor following Canon/Sony patterns

### Prerequisites

- **Knowledge assumed**: Understanding of TIFF IFD processing, MakerNotes extraction, Micro Four Thirds and L-mount lens systems
- **Setup required**: Panasonic test images (GH series, G series, S series) for comprehensive validation

**Context Quality Check**: Can a new engineer understand WHY Panasonic support requires both standard MakerNotes and separate RW2 processing?

## Work Completed

- ✅ **Complete tag kit extraction** → Panasonic.pm fully processed with comprehensive tag definitions in generated/Panasonic_pm/tag_kit/
- ✅ **PanasonicRaw implementation** → Working PrintConv functions for RW2 format tags, integrated with supported_tags.json
- ✅ **Leica lens database** → Extensive lens identification system generated from ExifTool source
- ✅ **File format detection** → Panasonic cameras and formats properly detected in file_type_lookup

## Remaining Tasks

### 1. Task: Analyze current supported_tags.json coverage for Panasonic MakerNotes

**Success Criteria**: Document which MakerNotes tags in supported_tags.json should come from Panasonic standard (not PanasonicRaw) processing
**Approach**: Review supported_tags.json for generic MakerNotes tags, cross-reference with Panasonic.pm Main table
**Dependencies**: None - research task

**Success Patterns**:
- ✅ Clear distinction between PanasonicRaw tags (already implemented) and standard MakerNotes tags needed
- ✅ Priority list of missing MakerNotes tags that Panasonic cameras should provide
- ✅ Understanding of Leica vs Panasonic tag naming conventions

### 2. Task: Create Panasonic MakerNotes processor implementation

**Success Criteria**: `src/implementations/panasonic/mod.rs` processes standard Panasonic MakerNotes using existing tag kit system
**Approach**: Follow Canon/Sony implementation patterns, integrate with generated tag kit system for Main table processing
**Dependencies**: Task 1 (requirements analysis)

**Success Patterns**:
- ✅ process_panasonic_makernotes() function extracts standard TIFF IFD MakerNotes
- ✅ Integration with main EXIF processing pipeline for both Panasonic and Leica cameras
- ✅ Proper namespace handling using "MakerNotes" group

### 3. Task: Implement lens identification system integration

**Success Criteria**: Panasonic/Leica lens identification tags properly extracted with human-readable names from extensive lens database
**Approach**: Leverage generated leicaLensTypes and lens identification logic from tag kit system
**Dependencies**: Task 2 (MakerNotes processor)

**Success Patterns**:
- ✅ LensType and LensModel tags populated with proper lens names for both Panasonic and Leica lenses
- ✅ Complex lens identification logic (2-integer splitting, frame selector bits) working correctly
- ✅ Fallback handling for uncoded/manual lenses

### 4. Task: Validate against major Panasonic camera series

**Success Criteria**: Tag extraction working across GH (video), G (photo), S (full-frame) camera series
**Approach**: Test with sample files from different Panasonic camera lines, ensure broad compatibility
**Dependencies**: Task 3 (lens identification)

**Success Patterns**:
- ✅ GH series (GH5, GH6) video-focused cameras extract proper MakerNotes
- ✅ G series (G9, G100) photo-focused cameras extract proper MakerNotes  
- ✅ S series (S1, S5) full-frame L-mount cameras extract proper MakerNotes
- ✅ Leica cameras (if available) extract compatible MakerNotes

### 5. RESEARCH: Validate PanasonicRaw vs standard MakerNotes separation

**Objective**: Ensure clear separation between RW2-specific tags (already implemented) and standard JPEG/video MakerNotes
**Success Criteria**: No overlap or conflict between PanasonicRaw and standard Panasonic MakerNotes processing
**Done When**: Clear documentation of which processor handles which file types and tag sources

## Implementation Guidance

### Recommended Patterns

- **Dual processor approach**: Maintain separation between PanasonicRaw (RW2) and standard MakerNotes (JPEG/video) processing
- **Tag kit integration**: Leverage existing comprehensive tag kit system for consistent processing
- **Leica compatibility**: Ensure processing works for both Panasonic and Leica branded cameras with same MakerNotes structure

### Tools to Leverage

- **Existing tag kit system**: Complete Panasonic tag kit extraction ready for integration
- **PanasonicRaw pattern**: Working implementation in panasonic_raw.rs shows successful PrintConv integration
- **Generated lens database**: Extensive leicaLensTypes lookup already extracted from ExifTool
- **Generic subdirectory processing**: Standard TIFF IFD processing utilities available

### Architecture Considerations

- **Namespace separation**: Standard MakerNotes must use "MakerNotes" group, avoid conflict with PanasonicRaw group
- **Format-specific routing**: Ensure RW2 files use PanasonicRaw processor, JPEG/video use standard MakerNotes processor
- **Leica branding**: Handle lens names and camera identification for both Panasonic and Leica variants

## Integration Requirements

**CRITICAL**: Building without integrating is failure. Don't accept incomplete Panasonic support.

Every feature must include:
- [ ] **Activation**: Panasonic MakerNotes processing enabled by default for Panasonic/Leica cameras
- [ ] **Consumption**: Tags appear in standard extraction output without special flags
- [ ] **Measurement**: Can verify Panasonic MakerNotes extraction with grep on output
- [ ] **Cleanup**: No overlap or conflict with existing PanasonicRaw implementation

**Red Flag Check**: If Panasonic JPEG files show zero MakerNotes tags while RW2 files work, integration is incomplete.

## Working Definition of "Complete"

A feature is complete when:
- ✅ **System behavior changes** - Panasonic JPEG/video files now extract MakerNotes tags automatically
- ✅ **Default usage** - No special configuration needed for Panasonic tag extraction
- ✅ **Old path removed** - No placeholder infrastructure remaining that doesn't extract actual tags
- ❌ Code exists but extracts zero tags *(example: "processor implemented but no MakerNotes appear")*
- ❌ Feature conflicts with existing PanasonicRaw *(example: "tag extraction duplicated between processors")*

## Prerequisites

- EXIF foundation → P10a-exif-required-tags → verify with `cargo t exif_basic`
- PanasonicRaw implementation → already completed → verify with existing RW2 tag extraction

## Testing

- **Unit**: Test Panasonic MakerNotes parsing with synthetic IFD data
- **Integration**: Verify tag extraction from GH6, G9, S5 sample files
- **Manual check**: Run `cargo run test-images/panasonic/gh6.jpg` and confirm MakerNotes tags appear

## Definition of Done

- [ ] `cargo t panasonic` passes (if Panasonic-specific tests exist)
- [ ] `make precommit` clean
- [ ] Panasonic MakerNotes tags appear in JPEG/video extraction output
- [ ] No conflict or duplication with existing PanasonicRaw tag extraction
- [ ] ExifTool comparison shows matching values for critical MakerNotes tags

## Gotchas & Tribal Knowledge

### Panasonic-Specific Considerations

- **Dual processor architecture**: PanasonicRaw handles RW2 files, main processor handles JPEG/video - ensure no overlap
- **Leica integration complexity**: Same MakerNotes structure but different branding/naming conventions
- **Lens identification system**: Complex 2-integer splitting logic with frame selector bits for manual lens detection
- **Video-specific tags**: Panasonic hybrid cameras have extensive video metadata requiring specific tag extraction

### Implementation Shortcuts

- **Tag kit ready**: Complete tag extraction system already generated - just needs processor integration
- **PrintConv pattern**: PanasonicRaw implementation shows working pattern for PrintConv integration
- **No encryption**: Standard TIFF IFD processing, no complex decryption like other manufacturers

### Architecture Separation

- **PanasonicRaw vs Panasonic**: Maintain clear separation - RW2 files use PanasonicRaw, JPEG/video use standard MakerNotes
- **Namespace isolation**: Use proper "MakerNotes" group for standard processing, distinct from "PanasonicRaw" group
- **supported_tags.json compliance**: 4 PanasonicRaw tags already working, focus on missing MakerNotes tags

## Quick Debugging

Stuck? Try these:

1. `cargo run test-images/panasonic/sample.jpg | grep MakerNotes | wc -l` - Count MakerNotes tags (should be >0)
2. `cargo run test-images/panasonic/sample.rw2 | grep PanasonicRaw | wc -l` - Verify RW2 processing still works
3. `exiftool -j -G test-images/panasonic/sample.jpg | jq '.[] | keys[]' | grep MakerNotes` - Compare with ExifTool
4. `rg "process_panasonic" src/` - Check if Panasonic processor is called from main pipeline