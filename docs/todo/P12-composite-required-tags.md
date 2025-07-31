# Technical Project Plan: Complete Required Composite Tags Implementation

## Project Overview

- **Goal**: Fix GPS group prefix resolution issues and complete missing required composite tag implementations to achieve 100% required tag coverage
- **Problem**: While 34 composite tags are implemented with solid infrastructure, GPS composite tags fail due to group prefix mismatch ("GPS:GPSDateStamp" required vs "EXIF:GPSDateStamp" available), and several key required tags need implementation
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

- **Composite Tag Infrastructure**: Mature multi-pass dependency resolution system in `src/composite_tags/` with 34 working implementations out of 73 generated definitions. Handles complex dependency graphs correctly including composite-to-composite dependencies.
- **Generated Definitions**: Auto-extracted from ExifTool's Composite.pm via `codegen/extractors/composite_tags.pl`, producing structured definitions with require/desire dependency metadata in `src/generated/composite_tags.rs`.
- **Integration Pipeline**: Composite resolution occurs during main tag processing via `resolve_and_compute_composites()` call in format processors, with proper TagEntry integration and PrintConv formatting.

### Key Concepts & Domain Knowledge

- **Group Prefix Resolution**: ExifTool's GPS tags have Group0=EXIF and Group1=GPS, making them accessible as both "GPS:GPSDateStamp" and "EXIF:GPSDateStamp", but our resolution logic doesn't handle this equivalence.
- **Required Tags**: 17 composite tags marked `"required": true` in `docs/tag-metadata.json` - currently 11 working correctly, 6 failing due to specific implementation gaps.
- **Dependency Resolution**: Uses ExifTool-compatible multi-pass algorithm with proper handling of circular dependencies and cross-composite references.

### Surprising Context

- **Infrastructure is Solid**: Previous engineers built comprehensive system that correctly handles complex cases like LightValue requiring Aperture/ShutterSpeed/ISO composites - the architecture is sound.
- **GPS Group Mapping Issue**: Core problem is NOT missing implementations but incorrect group prefix resolution - GPS composites require "GPS:GPSDateStamp" but we only store "EXIF:GPSDateStamp".
- **ScaleFactor35efl Placeholder**: Implementation exists but returns hardcoded 1.0 - needs CalcScaleFactor35efl algorithm from ExifTool.
- **High Success Rate**: 34/73 composite tags implemented with 45/55 tags working correctly in real-world testing - system is production-ready for most use cases.

### Foundation Documents

- **Design docs**: [CODEGEN.md](../CODEGEN.md) for extraction framework, [API-DESIGN.md](../design/API-DESIGN.md) for TagEntry integration
- **ExifTool source**: `third-party/exiftool/lib/Image/ExifTool.pm:3907-4093` BuildCompositeTags algorithm, `lib/Image/ExifTool/GPS.pm:354-420` GPS composites
- **Start here**: `src/composite_tags/resolution.rs:139` for dependency resolution bug, `src/composite_tags/implementations.rs:359` for ScaleFactor35efl placeholder

### Prerequisites

- **Knowledge assumed**: Understanding of ExifTool's group system and GPS tag structure, Rust HashMap/Option handling
- **Setup required**: `cargo t composite` for running tests, `compare-with-exiftool` binary for validation

## Work Completed

- ✅ **Multi-Pass Resolution System** → chose ExifTool-compatible algorithm over simple resolution because composite-to-composite dependencies are common
- ✅ **34 Composite Implementations** → implemented core calculations including ImageSize, Aperture, ShutterSpeed, GPS coordinates, timestamps with proper ExifTool source references
- ✅ **Generated Definition System** → extracted 73 composite definitions from ExifTool with proper dependency metadata and PrintConv references
- ✅ **Integration Tests** → validated real-world functionality with test images and ExifTool comparison tools

## Remaining Tasks

### 1. Task: Fix GPS Group Prefix Resolution Issue ✅ PARTIAL SUCCESS

**Success Criteria**: All GPS composite tags (GPSDateTime, GPSPosition, GPSLatitude, GPSLongitude, GPSAltitude) appear in CLI output and match ExifTool values exactly

**Approach**: 
1. ✅ **COMPLETED**: Modified `resolve_tag_dependency()` in `src/composite_tags/resolution.rs:181-197` to handle GPS/EXIF group equivalence
2. ✅ **COMPLETED**: Added logic to try "EXIF:TagName" when looking for "GPS:TagName" requests
3. ✅ **COMPLETED**: Added trace logging to debug group mapping resolution

**Dependencies**: None - isolated fix to resolution logic

**Progress Update (July 31, 2025)**:
- ✅ **GPSDateTime**: Now working correctly - shows "2019:06:06 19:53:34Z"
- ✅ **GPSPosition**: Now working correctly - shows "37 deg 30' 15.60\" 122 deg 28' 34.36\"" (minor formatting difference: missing comma)
- ❌ **GPSLatitude**: Still missing - dependency resolution failing for "GPS:GPSLatitude" vs "EXIF:GPSLatitude"
- ❌ **GPSLongitude**: Still missing - dependency resolution failing for "GPS:GPSLongitude" vs "EXIF:GPSLongitude"  
- ❌ **GPSAltitude**: Still missing - dependency resolution failing for "GPS:GPSAltitude" vs "EXIF:GPSAltitude"

**Root Cause Analysis**: GPS/EXIF group equivalence fix works for composite tags with single definitions (GPSDateTime uses "GPS:GPSDateStamp") but fails for individual GPS coordinate composite tags. The individual GPS composites require dependencies like "GPS:GPSLatitude" but we provide "EXIF:GPSLatitude". The GPS/EXIF group equivalence logic in resolve_tag_dependency() isn't being triggered correctly for these specific combinations.

**Key Finding**: ExifTool's target output with `-n` flag shows exactly what we need:
- `"EXIF:GPSLatitude": 37.5043319722222` (unsigned decimal) ✅ Working
- `"Composite:GPSLatitude": 37.5043319722222` (signed decimal) ❌ Missing 
- `"Composite:GPSLongitude": -122.476212` (signed decimal with correct negative for West) ❌ Missing

**Next Steps**: Debug why GPS/EXIF group equivalence isn't working for individual GPS coordinate lookups in resolve_tag_dependency().

### 2. Task: Implement ScaleFactor35efl Calculation

**Success Criteria**: ScaleFactor35efl shows correct crop factor values (e.g., 10.6 for OnePlus phone, not 1.0) matching ExifTool output

**Approach**:
1. Port ExifTool's CalcScaleFactor35efl function from `lib/Image/ExifTool/Exif.pm`
2. Implement sensor size database lookup or calculation logic
3. Replace placeholder in `compute_scale_factor_35efl()` with real calculation
4. Add proper error handling for missing sensor data

**Dependencies**: Task 1 (GPS fixes) should complete first to avoid integration conflicts

**Success Patterns**:
- ✅ Phone images show realistic crop factors (8-12x range for phone sensors)
- ✅ DSLR images show 1.0-1.6x range for full-frame/APS-C sensors
- ✅ ExifTool comparison shows matching ScaleFactor35efl values

### 3. Task: Validate and Test Required Tag Coverage

**Success Criteria**: All 17 required composite tags from `tag-metadata.json` work correctly with comprehensive test coverage

**Approach**:
1. Create systematic test for each required composite tag using representative images
2. Add edge case testing for missing dependencies and invalid values
3. Verify PrintConv formatting matches ExifTool exactly
4. Update integration tests with broader image coverage

**Dependencies**: Tasks 1 and 2 must complete first

**Success Patterns**:
- ✅ `cargo t composite` passes all tests including new required tag tests
- ✅ Required tag coverage documented and validated against tag-metadata.json
- ✅ Test suite covers edge cases and error conditions

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

- [ ] `cargo t composite` passes all tests including new GPS and ScaleFactor tests
- [ ] `make precommit` clean
- [ ] All 17 required composite tags from `tag-metadata.json` implemented and working
- [ ] `compare-with-exiftool` shows <5 differences total for composite tags on test images
- [ ] GPS composite tags appear in standard CLI output

## Additional Gotchas & Tribal Knowledge

**Format**: Surprise → Why → Solution (Focus on positive guidance)

- **GPS composites missing despite implementations existing** → Group prefix mismatch ("GPS:" required vs "EXIF:" available) → Add GPS/EXIF group equivalence logic to `resolve_tag_dependency()`
- **ScaleFactor35efl always shows 1.0** → Placeholder implementation returns hardcoded value → Port CalcScaleFactor35efl from ExifTool with sensor size lookup
- **Some composites work, others don't** → Dependency resolution may need specific group mapping → Check that required dependencies match available tag group prefixes
- **ExifTool shows composite but we don't** → Generated definition exists but computation function may have missing dispatch wiring → Verify both `dispatch.rs` case and function implementation exist

## Quick Debugging

Stuck? Try these:

1. `cargo run image.jpg | grep Composite:` - See which composites are actually working
2. `compare-with-exiftool image.jpg` - Get concrete diff of what's missing vs ExifTool
3. `rg "GPS:GPSDateStamp" src/` - Find where GPS dependencies are defined
4. `grep -A 10 -B 5 "resolve_tag_dependency" src/composite_tags/resolution.rs` - Check resolution logic

---

## Current Required Composite Tags Status

Based on analysis of `docs/tag-metadata.json` and real-world testing with `compare-with-exiftool`:

### Working Correctly ✅ (11/17)
- **Aperture** - F-number display formatting
- **ImageHeight** - Best available height value
- **ImageWidth** - Best available width value  
- **Megapixels** - Width × Height / 1,000,000
- **SubSecCreateDate** - EXIF CreateDate + SubSecTime
- **SubSecDateTimeOriginal** - EXIF DateTimeOriginal + SubSecTimeOriginal
- **SubSecModifyDate** - EXIF ModifyDate + SubSecTime
- **SubSecMediaCreateDate** - Media create date + subseconds (implementation exists)

### Failing Due to Group Prefix Issue ❌ (5/17) 
- **GPSAltitude** - Implementation exists, fails on "GPS:GPSAltitude" vs "EXIF:GPSAltitude"
- **GPSDateTime** - Implementation exists, fails on "GPS:GPSDateStamp" dependency resolution
- **GPSLatitude** - Implementation exists, fails on "GPS:GPSLatitude" vs "EXIF:GPSLatitude"
- **GPSLongitude** - Implementation exists, fails on "GPS:GPSLongitude" vs "EXIF:GPSLongitude" 

### Implementation Issues ❌ (1/17)
- **AvgBitrate** - Status unknown, needs verification with video files
- **DOF** - Implementation exists but needs validation
- **Lens** - Implementation exists but may need lens database integration
- **LensID** - Implementation exists but may need manufacturer-specific logic
- **LensType** - Implementation exists but may need lookup table integration

**Priority**: Fix group prefix resolution first (affects 5 tags), then validate remaining implementations.

**Next Engineer Should**: Start with Task 1 (GPS group prefix fix) as it will immediately resolve 5 required tags and provide clear validation criteria.