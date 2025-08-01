# Technical Project Plan: Complete Required Composite Tags Implementation

## Project Overview

- **Goal**: Complete all 30 required composite tag implementations with proper ExifTool-compatible formatting to achieve 100% required tag coverage
- **Problem**: Currently 19/30 (63%) required composite tags work fully, 4/30 work with formatting issues (GPS missing directional indicators), and 7/30 are completely missing implementations
- **Constraints**: Must maintain ExifTool compatibility, use existing codegen infrastructure, and preserve current working functionality

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

- **Composite Tag Infrastructure**: Mature multi-pass dependency resolution system in `src/composite_tags/` with 33 total implementations, 73 generated definitions from codegen. Handles complex dependency graphs correctly including composite-to-composite dependencies.
- **Generated Definitions**: Auto-extracted from ExifTool's main Composite table via `codegen/extractors/composite_tags.pl`, producing structured definitions with require/desire dependency metadata in `src/generated/composite_tags.rs`.
- **Integration Pipeline**: Composite resolution occurs during main tag processing via `resolve_and_compute_composites()` call in format processors, with proper TagEntry integration and PrintConv formatting.
- **Dispatch System**: Central dispatcher in `src/composite_tags/dispatch.rs` routes computation requests to implementation functions based on composite name matching.

### Key Concepts & Domain Knowledge

- **Required vs Available Tags**: 30 composite tags marked `"required": true` in `docs/tag-metadata.json` - these represent essential metadata for PhotoStructure's use cases.
- **Group Prefix Resolution**: ExifTool's GPS tags have Group0=EXIF and Group1=GPS, making them accessible as both "GPS:GPSDateStamp" and "EXIF:GPSDateStamp" - our system handles this equivalence correctly.
- **Dependency Resolution**: Uses ExifTool-compatible multi-pass algorithm with proper handling of circular dependencies and cross-composite references.
- **PrintConv Formatting**: Composite tags must match ExifTool's exact output format, including directional indicators (N/S/E/W) for GPS coordinates.

### Surprising Context

- **Infrastructure is Actually Solid**: Multi-pass resolution, GPS group equivalence, ScaleFactor35efl calculation, and codegen extraction all work correctly - the foundation is production-ready.
- **GPS Tags Work but Format Differently**: GPS composite tags compute correctly but miss ExifTool formatting (e.g., missing "N" directional indicator in "37 deg 30' 15.60\" N").
- **Context-Dependent Implementations**: Some tags like LensID/Lens work for certain manufacturers but fail for others - indicates dependency or lookup table gaps.
- **Duplicate Definitions**: Generated code contains multiple definitions for same composite names (e.g., two GPSLatitude entries) - codegen extracts from different ExifTool modules.
- **Many More Required Than Expected**: Original assessment of 17 required tags was significantly wrong - actual count is 30 required composite tags.

### Foundation Documents

- **Design docs**: [CODEGEN.md](../CODEGEN.md) for extraction framework, [API-DESIGN.md](../design/API-DESIGN.md) for TagEntry integration
- **ExifTool source**: `third-party/exiftool/lib/Image/ExifTool.pm:3907-4093` BuildCompositeTags algorithm, `lib/Image/ExifTool/GPS.pm:354-420` GPS composites
- **Start here**: `src/composite_tags/resolution.rs:139` for dependency resolution bug, `src/composite_tags/implementations.rs:359` for ScaleFactor35efl placeholder

### Prerequisites

- **Knowledge assumed**: Understanding of ExifTool's group system and GPS tag structure, Rust HashMap/Option handling
- **Setup required**: `cargo t composite` for running tests, `compare-with-exiftool` binary for validation

## Work Completed

- ✅ **Multi-Pass Resolution System** → chose ExifTool-compatible algorithm over simple resolution because composite-to-composite dependencies are common (GPSPosition depends on GPSLatitude/GPSLongitude)
- ✅ **33 Composite Implementations** → implemented core calculations including ImageSize, Aperture, ShutterSpeed, GPS coordinates, timestamps with proper ExifTool source references
- ✅ **Generated Definition System** → extracted 73 composite definitions from ExifTool with proper dependency metadata and PrintConv references
- ✅ **GPS Group Resolution** → GPS/EXIF group equivalence working (GPS:GPSDateStamp resolves to EXIF:GPSDateStamp correctly)
- ✅ **ScaleFactor35efl Implementation** → Canon sensor diagonal algorithm implemented with high accuracy (OnePlus: 10.6, Canon 5D: 1.2, Canon T7i: 1.6 match ExifTool)
- ✅ **Integration Pipeline** → composite tags appear in main CLI output and work with comparison tools

## Remaining Tasks

### 1. Task: Fix GPS Composite Tag Formatting to Match ExifTool

**Success Criteria**: GPS composite tags show identical formatting to ExifTool including directional indicators
- GPSLatitude: "37 deg 30' 15.60\" N" (not "37 deg 30' 15.60\"")
- GPSLongitude: "122 deg 28' 34.36\" W" (not "122 deg 28' 34.36\"")
- GPSPosition: "37 deg 30' 15.60\" N, 122 deg 28' 34.36\" W" (with comma)
- GPSAltitude: "6.9 m Below Sea Level" (not "Unknown (7.0 m)")

**Approach**:
1. Examine GPS PrintConv implementations in `src/composite_tags/implementations.rs`
2. Add directional indicator logic using GPSLatitudeRef/GPSLongitudeRef/GPSAltitudeRef values
3. Fix altitude reference formatting to match ExifTool's "X.X m Above/Below Sea Level" format
4. Update GPSPosition to include comma separator and directional indicators

**Dependencies**: None - GPS tags are working, only formatting needs adjustment

**Success Patterns**:
- ✅ `compare-with-exiftool --group "Composite:" test-images/oneplus/gm1917_01.jpg` shows 0 differences for GPS tags
- ✅ Directional indicators (N/S/E/W) appear correctly based on ref values
- ✅ Altitude shows proper "Above/Below Sea Level" formatting with correct precision

### 2. Task: Implement Missing Required Composite Tags (7 remaining)

**Success Criteria**: All 7 missing required composite tags implemented and working
- DateTimeCreated
- DigitalCreationDateTime
- FileNumber
- GPSAltitudeRef
- GPSLatitudeRef
- GPSLongitudeRef

**Approach**:
1. Research ExifTool source for each missing tag's definition and dependencies
2. Add implementations to `src/composite_tags/implementations.rs` following existing patterns
3. Add dispatch cases to `src/composite_tags/dispatch.rs`
4. Verify generated definitions exist in `src/generated/composite_tags.rs` or add to codegen config

**Dependencies**: None - infrastructure is ready for new implementations

**Success Patterns**:
- ✅ All 30 required composite tags appear in CLI output for appropriate test images
- ✅ `compare-with-exiftool` shows working implementations matching ExifTool behavior
- ✅ Missing composite implementations reduce from 7 to 0

### 3. Task: Debug Context-Dependent Implementation Failures

**Success Criteria**: Investigate why some composite tags (ISO, Lens, LensID) work in some contexts but fail in others

**Approach**:
1. Test failing implementations with multiple manufacturer images (Canon, Nikon, Sony, etc.)
2. Add debug logging to identify missing dependencies for each failure case
3. Examine ExifTool source to understand manufacturer-specific requirements
4. Fix dependency resolution or add manufacturer-specific logic as needed

**Dependencies**: None - diagnostic work to understand existing implementation gaps

**Success Patterns**:
- ✅ ISO composite tag works for Canon test images (currently fails)
- ✅ Lens/LensID tags work across multiple manufacturers
- ✅ `compare-with-exiftool` shows consistent behavior across different camera brands

### 4. RESEARCH: Investigate Duplicate Composite Definitions

**Objective**: Understand why generated code contains multiple definitions with same names (GPSLatitude, GPSLongitude)

**Success Criteria**: Document whether duplicates are intentional (different modules) or codegen issue

**Approach**:
1. Examine `src/generated/composite_tags.rs` for duplicate entries
2. Trace back to ExifTool source to see if multiple modules define same composite names
3. Determine if codegen should merge or preserve separate definitions
4. Document findings and recommend solution if needed

**Dependencies**: None - research task

**Done When**: Clear understanding of duplicate definition handling and documentation of recommended approach

## Integration Requirements

**CRITICAL**: Building without integrating is failure. Don't accept tasks that build "shelf-ware."

Every feature must include:
- [x] **Activation**: Composite tag resolution enabled by default in tag processing pipeline
- [x] **Consumption**: Main CLI actively displays composite tags in standard output  
- [x] **Measurement**: `compare-with-exiftool` tool provides concrete validation metrics
- [ ] **Cleanup**: Remove placeholder implementations once real logic is complete

## Working Definition of "Complete"

A composite tag implementation is complete when:
- ✅ **CLI shows tag** - appears in `cargo run image.jpg` output without special flags
- ✅ **Values match ExifTool** - `compare-with-exiftool` shows identical or equivalent values
- ✅ **Edge cases handled** - graceful behavior with missing dependencies or invalid data
- ✅ **Test coverage** - integration test validates behavior across multiple image types
- ✅ **Source references** - implementation includes ExifTool file:line citations
- ❌ Function exists but not called by dispatch system
- ❌ Values computed but don't match ExifTool formatting
- ❌ Works for some images but fails on others

## Prerequisites

- **P10a EXIF Processing** → [P10a-exif-essential-tags.md](P10a-exif-essential-tags.md) → verify with `cargo t exif_tag_extraction`
- GPS source tag extraction must be working (currently is - we see EXIF:GPSLatitude etc. in output)

## Testing

- **Unit**: Test each modified resolution/computation function with mock dependency data
- **Integration**: Verify GPS composite tags with real GPS-enabled images in `tests/composite_tag_tests.rs`
- **Manual check**: Run `compare-with-exiftool test-images/oneplus/gm1917_01.jpg` and confirm <5 total differences

## Definition of Done

- [ ] `cargo t composite` passes all tests including formatting fixes
- [ ] `make precommit` clean  
- [ ] All 30 required composite tags from `tag-metadata.json` implemented and working
- [ ] `compare-with-exiftool --group "Composite:"` shows 0 differences for GPS tags on test images
- [ ] All required composite tags appear consistently across different manufacturer test images
- [ ] Context-dependent implementation failures resolved (ISO, Lens, LensID work for Canon images)

## Additional Gotchas & Tribal Knowledge

**Format**: Surprise → Why → Solution (Focus on positive guidance)

- **GPS composite tags missing directional indicators** → ExifTool includes N/S/E/W from reference tags → Use GPSLatitudeRef/GPSLongitudeRef values to append direction indicators
- **Composite tag implemented but doesn't appear** → Missing from dispatch system or dependency failure → Check both `dispatch.rs` case exists AND required dependencies are available with correct group prefixes
- **Same tag works for some manufacturers, not others** → Different dependency sources or manufacturer-specific requirements → Add debug logging to identify which dependencies are missing and check ExifTool source for manufacturer-specific logic
- **Multiple GPS definitions in generated code** → ExifTool defines same composite in different modules (main Composite table + QuickTime GPS coordinates) → Understand which definition applies to which context, may need priority ordering
- **Required tag count wrong in assessments** → tag-metadata.json is authoritative source → Always verify against `grep '"required": true' docs/tag-metadata.json` rather than assumptions

## Quick Debugging

Stuck? Try these:

1. `cargo run test-images/oneplus/gm1917_01.jpg | grep Composite:` - See which composites are working
2. `compare-with-exiftool --group "Composite:" test-images/canon/eos_5d_mark_iv.jpg` - Get manufacturer-specific differences
3. `rg "compute_gps_latitude" src/composite_tags/implementations.rs` - Check GPS implementation details
4. `grep -A 10 -B 5 "resolve_tag_dependency" src/composite_tags/resolution.rs` - Check resolution logic
5. `RUST_LOG=trace cargo run image.jpg 2>&1 | grep composite` - See dependency resolution debug output
6. `grep -C 5 "DateTimeCreated" src/generated/composite_tags.rs` - Check if missing tags have generated definitions

---

## Implementation Guidance

### Recommended Patterns

- **PrintConv Implementation**: Follow existing GPS implementations in `src/composite_tags/implementations.rs` - use helper functions for common formatting patterns
- **Dependency Resolution**: Use `available_tags.get()` with proper group prefixes, check both specific groups and fallback patterns
- **Error Handling**: Return `None` for missing dependencies, use `TagValue::String()` for formatted output
- **ExifTool Citations**: Include `// ExifTool: lib/Image/ExifTool/Module.pm:line_range` comments for traceability

### Tools to Leverage

- **compare-with-exiftool**: Use `--group "Composite:"` filter for focused testing
- **Debug Logging**: Add `trace!()` calls to track dependency resolution failures
- **Test Images**: Use manufacturer-specific images from `test-images/` for comprehensive testing
- **Generated Definitions**: Check `src/generated/composite_tags.rs` for existing dependency metadata

### ExifTool Translation Notes

- **GPS Formatting**: ExifTool's GPS PrintConv includes directional indicators - look for `$ref` variables in GPS.pm
- **Reference Values**: Many composite tags use `*Ref` suffix tags for formatting context (GPSLatitudeRef, GPSAltitudeRef)
- **Conditional Logic**: Some PrintConv expressions have inline conditionals - implement as match statements in Rust
- **Multiple Definitions**: ExifTool may define same composite name in multiple modules with different requirements

## Current Implementation Status (30 Required Tags)

### ✅ Working Correctly (19/30)
Aperture, DateTimeOriginal, Duration, GPSDateTime, ImageHeight, ImageSize, ImageWidth, Megapixels, PreviewImage, Rotation, ShutterSpeed, SubSecCreateDate, SubSecDateTimeOriginal, SubSecMediaCreateDate, SubSecModifyDate, ScaleFactor35efl (verified accurate)

### ⚠️ Working with Formatting Issues (4/30)  
GPSAltitude, GPSLatitude, GPSLongitude, GPSPosition (missing directional indicators and proper formatting)

### ❌ Context-Dependent Failures (4/30)
ISO, Lens, LensID, LensSpec, LensType (work in some contexts, fail in others - needs investigation)

### ❌ Missing Implementations (7/30)
DateTimeCreated, DigitalCreationDateTime, FileNumber, GPSAltitudeRef, GPSLatitudeRef, GPSLongitudeRef