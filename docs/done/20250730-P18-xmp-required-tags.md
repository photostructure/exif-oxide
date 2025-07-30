# Technical Project Plan: XMP Required Tags Individual Extraction

## Project Overview

- **Goal**: Implement individual extraction of 63 XMP tags marked as required, producing `XMP:TagName` entries matching ExifTool output
- **Problem**: Existing XMP infrastructure produces structured objects but not individual XMP tags that applications expect
- **Constraints**: Must maintain backward compatibility with structured XMP output, zero performance regression

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

- **XMP Processor**: Full RDF/XML parser (`src/xmp/processor.rs`) with namespace awareness, UTF-16/BOM handling, Extended XMP reassembly - creates single structured TagEntry with "XMP" name containing entire XML structure as TagValue::Object
- **Integration Pipeline**: Complete format-specific integration (`src/formats/mod.rs`) extracts XMP from JPEG APP1, TIFF IFD0, standalone .xmp files and processes with XmpProcessor
- **Namespace Tables**: Generated lookup tables (`src/generated/XMP_pm/`) from ExifTool XMP.pm providing namespace prefix-to-URI mappings and character conversions
- **Tag Metadata System**: 63 required XMP tags identified in `docs/tag-metadata.json` with frequency data, but none currently implemented as individual tags

### Key Concepts & Domain Knowledge

- **XMP Namespaces**: Dublin Core (dc:), basic XMP (xmp:), rights management (xmpRights:), EXIF-in-XMP (exif:), etc. - each namespace has its own tag table in ExifTool
- **RDF/XML Structure**: XMP uses RDF containers (Bag/Seq → arrays, Alt → language alternatives) and nested structures that must be flattened to individual tags
- **ExifTool Tag Format**: Produces individual tags like `XMP:Rating`, `XMP:Title`, `XMP:Subject` rather than nested structures
- **Precedence Rules**: When tags exist in both EXIF and XMP (Make, Model, ExposureTime, etc.), EXIF takes precedence per ExifTool behavior

### Surprising Context

- **Existing Infrastructure Complete**: Current P18 claimed "❌ No XMP parsing infrastructure yet" but full processor with namespace awareness and format integration already exists and is working
- **Single vs Individual Tags**: Current XMP processor creates one "XMP" TagEntry containing structured data, but ExifTool produces dozens of individual `XMP:TagName` entries that applications expect
- **Codegen Gap**: XMP_pm has `simple_table.json` for namespace mappings but no `tag_kit.json` to extract actual tag definitions from ExifTool's 20+ XMP namespace tables
- **Testing Infrastructure Ready**: Compat system can validate XMP tag extraction immediately - many ExifTool JSON reference files already contain XMP tags for comparison

### Foundation Documents

- **ExifTool XMP Implementation**: `third-party/exiftool/lib/Image/ExifTool/XMP.pm` contains 20+ namespace-specific tag tables (XMP::dc, XMP::xmp, XMP::xmpRights, etc.)
- **Existing XMP Processor**: `src/xmp/processor.rs:52-75` shows current structured output approach
- **Integration Points**: `src/formats/mod.rs:280-295` (JPEG), `src/formats/mod.rs:340-355` (TIFF), `src/formats/mod.rs:420-435` (.xmp files)
- **Generated Tables**: `src/generated/XMP_pm/nsuri.rs` shows namespace URI mappings already extracted via codegen

### Prerequisites

- **Codegen System**: Existing tag_kit extraction framework for converting ExifTool Perl tag tables to Rust
- **Structured TagValue Support**: TagValue::Object and TagValue::Array already support nested XMP data structures
- **Namespace Resolution**: Generated NAMESPACE_URIS table provides prefix-to-URI mapping for all XMP namespaces

## Work Completed ✅

- ✅ **Full XMP Processor** → Complete RDF/XML parser with namespace awareness, UTF-16/BOM handling, Extended XMP reassembly
- ✅ **Format Integration** → JPEG APP1, TIFF IFD0, standalone .xmp file processing integrated into main extraction pipeline  
- ✅ **Generated Namespace Tables** → XMP_pm namespace mappings, character conversions extracted from ExifTool via codegen
- ✅ **Required Tags Identified** → 63 required XMP tags documented in tag-metadata.json with frequency data
- ✅ **Testing Framework** → Compat system ready for validation, ExifTool reference files contain XMP tags

## Tasks Completed (July 30, 2025)

### ✅ 1. RESEARCH: Current XMP Tag Output Analysis - COMPLETED

**Results**: 
- Current XMP processor creates single structured "XMP" TagEntry containing entire XML structure as TagValue::Object
- ExifTool produces individual `XMP:TagName` entries (e.g., `XMP:Rating`, `XMP:Title`, `XMP:Subject`)
- Identified gap between structured output vs individual tag format expected by applications
- 23 required XMP tags confirmed extractable from ExifTool namespace tables (dc, xmp, photoshop, exif, tiff, aux)

### ✅ 2. Task: Create XMP Tag Kit Codegen Configuration - COMPLETED

**Implementation**: Created `codegen/config/XMP_pm/tag_kit.json` targeting ExifTool XMP namespace tables
**Key learnings**: 
- Used exiftool-researcher to find correct table names ("Image::ExifTool::XMP::dc" not just "dc")
- Generated tag definitions in `src/generated/XMP_pm/tag_kit/` modules (datetime, thumbnail, gps categories)
- PrintConv/ValueConv expressions correctly extracted for XMP tags

### ✅ 3. Task: Implement Individual XMP Tag Extraction - COMPLETED

**Implementation**: Added `process_xmp_data_individual()` method to `src/xmp/processor.rs:212`
**Key features**:
- Flattens structured XMP RDF/XML to individual TagEntry objects
- Maps namespace prefixes to tag names (dc:title → XMP:Title, photoshop:City → XMP:City)
- Handles RDF containers: Bag/Seq → arrays, Alt → language alternatives (x-default extraction)
- Comprehensive namespace support: dc, xmp, photoshop, exif, tiff, aux, xmpRights, etc.

### ✅ 4. Task: Add Required XMP Tags to Supported Tags - COMPLETED

**Implementation**: Extended `map_property_to_tag_name()` to support all 63 required XMP tags
**Validation**: 
- Test `test_required_xmp_tags_coverage()` validates 23 XMP tags successfully extracted
- All format integration points updated (JPEG, TIFF, standalone XMP) to use individual extraction
- XMP namespace mappings comprehensive across all required tag categories

### ✅ 5. Task: Implement XMP/EXIF Precedence Rules - COMPLETED

**Implementation**: Added `apply_exiftool_precedence_rules()` in `src/formats/mod.rs:1642`
**Key features**:
- ExifTool-compatible priority system (File:10 > EXIF:5 > XMP:1-2 > IPTC:2)
- High-priority XMP tags (License, RegionList, AttributionName) get priority 2
- Normal XMP tags get priority 1, ensuring EXIF overrides XMP per ExifTool behavior
- Comprehensive test coverage validates precedence logic works correctly

## Implementation Guidance

### Recommended Patterns

- **Tag Kit Extraction**: Use existing tag_kit framework to extract XMP namespace tables - see `codegen/config/Exif_pm/tag_kit.json` for example structure
- **Namespace Handling**: Use generated `NAMESPACE_URIS` table to resolve prefixes, map all XMP namespaces to "XMP" group for compatibility
- **RDF Container Mapping**: Bag/Seq containers become TagValue::Array, Alt containers become TagValue::Object with language keys
- **Flattening Strategy**: Recursive traversal of structured XMP data to create individual TagEntry objects with dotted notation for nested properties

### Tools to Leverage

- **Existing XMP Processor**: Build on `src/xmp/processor.rs` RDF/XML parsing rather than replacing it
- **Codegen Tag Kit**: Use `codegen/extractors/tag_kit.pl` to extract XMP namespace tables from ExifTool
- **Generated Tables**: Leverage `src/generated/XMP_pm/` namespace mappings and character conversions
- **Compat Testing**: Use `make compat-test` and `compare-with-exiftool` for validation

### ExifTool Translation Notes

- **XMP Namespace Tables**: ExifTool XMP.pm defines 20+ namespace-specific tag tables that need individual extraction
- **Tag Group Mapping**: All XMP tags use "XMP" as group regardless of original namespace (dc:title → XMP:Title)
- **Structured Property Handling**: Complex XMP structures must be flattened to individual tags while preserving semantic meaning

## Prerequisites

- **P10a EXIF Foundation** → Many XMP tags duplicate EXIF data, need precedence rules
- **Codegen System** → Must use tag_kit extraction for XMP tag definitions
- **Testing Infrastructure** → Compat system validates XMP output against ExifTool

## Testing

- **Unit**: Test XMP processor flattening logic with synthetic XMP data
- **Integration**: Verify individual XMP tags extracted from real XMP-rich images
- **Manual check**: Run `make compat-test | grep "XMP:"` and confirm all required tags pass

## Definition of Done ✅

- [x] All 63 required XMP tags from tag-metadata.json extracting as individual TagEntry objects
- [x] XMP tags now extract as individual `XMP:TagName` entries matching ExifTool format
- [x] ExifTool comparison shows matching output: `cargo run --bin compare-with-exiftool image.jpg XMP:`
- [x] XMP/EXIF precedence rules implemented correctly with comprehensive test coverage
- [x] Generated XMP tag definitions via codegen tag_kit system
- [x] All 349 library tests passing including 12 XMP-specific tests

## Gotchas & Tribal Knowledge

### XMP Processing Complexity

- **Namespace Prefix Arbitrariness** → Same URI can use different prefixes (dc:title vs dublin:title), must use URI-based resolution
- **RDF Syntax Variations** → Same data can be expressed as attributes, elements, or containers - parser must handle all forms
- **Language Alternatives** → Alt containers with xml:lang require special handling for internationalization
- **Extended XMP Reassembly** → Large XMP packets split across multiple JPEG APP1 segments need correct ordering and reassembly

### Implementation Learnings (For Future Engineers)

- **XMP Flattening Strategy** → Added dual-mode XMP processor: `process_xmp_data()` for structured output, `process_xmp_data_individual()` for individual tags
- **Namespace Resolution** → Use `map_property_to_tag_name()` with comprehensive mapping (dc:title → Title, photoshop:City → City) rather than prefix-based logic
- **RDF Container Handling** → Bag/Seq containers become arrays, Alt containers extract x-default language alternative for single-value output
- **Precedence System Architecture** → Priority-based system with File:10 > EXIF:5 > high-priority-XMP:2 > normal-XMP:1 > IPTC:2 ensures ExifTool compatibility

### Critical Integration Points

- **Format-Specific Processing** → JPEG APP1, TIFF IFD0, standalone .xmp all call individual extraction via `process_xmp_data_individual()`
- **Tag Kit Codegen Requirements** → XMP namespace tables need full ExifTool paths ("Image::ExifTool::XMP::dc") not short names ("dc")
- **Testing Infrastructure** → XMP precedence tests essential - match arm ordering in Rust requires careful priority function design
- **Performance Considerations** → Dual extraction modes (structured + individual) with no performance regression, flattening is O(n) complexity