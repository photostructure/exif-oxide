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

## Work Completed

- ✅ **Full XMP Processor** → Complete RDF/XML parser with namespace awareness, UTF-16/BOM handling, Extended XMP reassembly
- ✅ **Format Integration** → JPEG APP1, TIFF IFD0, standalone .xmp file processing integrated into main extraction pipeline  
- ✅ **Generated Namespace Tables** → XMP_pm namespace mappings, character conversions extracted from ExifTool via codegen
- ✅ **Required Tags Identified** → 63 required XMP tags documented in tag-metadata.json with frequency data
- ✅ **Testing Framework** → Compat system ready for validation, ExifTool reference files contain XMP tags

## Remaining Tasks

### 1. RESEARCH: Current XMP Tag Output Analysis

**Objective**: Determine which XMP tags (if any) are currently working and understand output format gaps

**Success Criteria**: 
- Document current XMP processor output structure vs ExifTool individual tag format
- Identify which of the 63 required XMP tags appear in our test files  
- List specific ExifTool XMP namespace tables that need tag_kit extraction

**Approach**: 
- Run `cargo run --bin compare-with-exiftool test-images/canon_eos_r8.jpg XMP:` to see current vs expected output
- Check `generated/exiftool-json/` files for XMP tag examples
- Map required tags to ExifTool namespace tables (dc, xmp, xmpRights, etc.)

**Dependencies**: None

### 2. Task: Create XMP Tag Kit Codegen Configuration

**Success Criteria**: XMP tag definitions extracted from ExifTool and available as generated Rust code

**Approach**: Create `codegen/config/XMP_pm/tag_kit.json` to extract tag tables from ExifTool XMP.pm namespace tables

**Dependencies**: Task 1 (need namespace analysis)

**Success Patterns**:
- ✅ Generated tag definitions in `src/generated/XMP_pm/tag_kit/` modules
- ✅ `make codegen` runs without errors and produces XMP tag structures
- ✅ Tag definitions include PrintConv/ValueConv references where applicable

### 3. Task: Implement Individual XMP Tag Extraction

**Success Criteria**: XMP processor produces individual TagEntry objects for each XMP property, not single structured object

**Approach**: 
- Extend `XmpProcessor::process_xmp_data()` to flatten structured XMP into individual TagEntry objects
- Map namespace prefixes to "XMP" group for ExifTool compatibility 
- Handle RDF containers (Bag/Seq → arrays, Alt → language alternatives) appropriately

**Dependencies**: Task 2 (need generated tag definitions)

**Success Patterns**:
- ✅ Individual `XMP:Rating`, `XMP:Title`, `XMP:Subject` etc. TagEntry objects produced
- ✅ Structured XMP data correctly flattened to ExifTool-compatible format
- ✅ Namespace prefixes properly resolved using generated tables

### 4. Task: Add Required XMP Tags to Supported Tags

**Success Criteria**: All 63 required XMP tags added to `config/supported_tags.json` and pass compat testing

**Approach**: 
- Add each required XMP tag from tag-metadata.json to supported_tags.json
- Run `make compat-force` to regenerate reference data including XMP 
- Fix any conversion implementations needed for complex XMP structures

**Dependencies**: Task 3 (need individual tag extraction working)

**Success Patterns**:
- ✅ All 63 XMP tags present in supported_tags.json
- ✅ `make compat-test` passes for XMP tags
- ✅ ExifTool comparison shows matching output for XMP-rich test files

### 5. Task: Implement XMP/EXIF Precedence Rules

**Success Criteria**: When tags exist in both EXIF and XMP, EXIF values take precedence per ExifTool behavior

**Approach**: 
- Identify overlapping tags (Make, Model, ExposureTime, FNumber, etc.)
- Modify extraction pipeline to prefer EXIF over XMP for duplicate tags
- Document precedence rules following ExifTool's approach

**Dependencies**: Task 4 (need XMP tags working)

**Success Patterns**:
- ✅ EXIF:Make preferred over XMP:Make when both present
- ✅ Precedence behavior matches ExifTool exactly
- ✅ No duplicate tags in final output

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

## Definition of Done

- [ ] All 63 required XMP tags from tag-metadata.json extracting as individual TagEntry objects
- [ ] `make compat-test` passes for XMP tags with <5 failures
- [ ] ExifTool comparison shows matching output: `cargo run --bin compare-with-exiftool image.jpg XMP:`
- [ ] XMP/EXIF precedence rules implemented correctly
- [ ] Generated XMP tag definitions via codegen tag_kit system
- [ ] `make precommit` clean

## Gotchas & Tribal Knowledge

### XMP Processing Complexity

- **Namespace Prefix Arbitrariness** → Same URI can use different prefixes (dc:title vs dublin:title), must use URI-based resolution
- **RDF Syntax Variations** → Same data can be expressed as attributes, elements, or containers - parser must handle all forms
- **Language Alternatives** → Alt containers with xml:lang require special handling for internationalization
- **Extended XMP Reassembly** → Large XMP packets split across multiple JPEG APP1 segments need correct ordering and reassembly

### Current Implementation Gaps

- **Single Structured Output** → Current processor creates one "XMP" TagEntry instead of individual tags applications expect
- **Missing Tag Definitions** → No codegen extraction of ExifTool XMP namespace tables means no tag implementations
- **No Flattening Logic** → Structured RDF data not converted to individual TagEntry objects matching ExifTool format

### Integration Gotchas

- **Precedence Rules Critical** → EXIF must take precedence over XMP for overlapping tags to match ExifTool behavior
- **Group Name Consistency** → All XMP tags must use "XMP" group regardless of namespace for ExifTool compatibility
- **Empty vs Missing Values** → Distinguish between empty XMP properties and missing properties for correct null handling