# Technical Project Plan: SubDirectory Processing System

## Project Overview

- **Goal**: Implement ExifTool-compatible subdirectory processing that correctly selects tag variants based on camera model and processes nested metadata structures
- **Problem**: Current implementation doesn't match ExifTool's architecture - no Condition evaluation, wrong PrintConv timing, missing tag table switching
- **Constraints**: Must exactly match ExifTool's behavior, zero new bugs, maintain backwards compatibility

## Context & Foundation

### System Overview

- **SubDirectory Processing**: ExifTool's recursive metadata architecture for handling nested structures like maker notes, where tags can have multiple variants selected via Conditions (e.g., Canon tag 0xf has 50+ CustomFunctions variants based on camera model)
- **Tag Tables**: Generated Rust modules from ExifTool's perl tables (e.g., `Canon_pm::camera_settings_tags`) that define tag structures, formats, and conversions
- **Expression Evaluator**: Our existing system (`src/expressions/`) that parses and evaluates ExifTool-style conditions like `$$self{Model} =~ /EOS-1D/`

### Key Concepts & Domain Knowledge

- **SubDirectory**: A tag containing nested metadata that references another tag table for processing
- **Condition**: Perl expression that selects which tag variant to use based on context (model, make, data pattern)
- **TagTable**: Collection of tag definitions for a specific metadata structure (e.g., CanonCustom::Functions1D)
- **ProcessBinaryData vs ProcessDirectory**: Two different processing modes - binary extracts from fixed offsets, directory follows IFD structure

### Surprising Context

- **PrintConv happens AFTER extraction**: We incorrectly apply PrintConv during subdirectory processing - ExifTool applies it later during value display
- **Tag variants aren't alternatives**: Canon tag 0xf isn't one tag with options - it's 50+ completely different tags selected by camera model
- **Generated modules already exist**: We have `CanonCustom_pm::functions1d_tags.rs` etc. but no way to load them dynamically
- **Conditions use Perl eval**: ExifTool evaluates `$$self{Model}` expressions with full Perl context - we need to map this to our Rust context

### Foundation Documents

- **ExifTool source**: `third-party/exiftool/lib/Image/ExifTool.pm:10701` (GetTagInfo with Conditions)
- **Design docs**: `docs/analysis/subdirectory-mechanics-analysis.md` (DRY pattern analysis)
- **Start here**: `src/exif/subdirectory_processing.rs` (current broken implementation)

### Prerequisites

- **Knowledge assumed**: Understanding of ExifTool tag tables, Rust codegen system
- **Setup required**: Access to test images with various Canon camera models

## Work Completed

- ✅ Research → analyzed ExifTool's subdirectory architecture in `SUBDIRECTORY_SYSTEM.md`
- ✅ Analysis → documented gap between our implementation and ExifTool's in `subdirectory-processing-gap-analysis.md`
- ✅ Design → created DRY solution using table registry pattern in `subdirectory-mechanics-analysis.md`

## TDD Foundation Requirement

### Task 0: Not applicable - architecture/design phase

**Success Criteria**: Architecture implementation enables existing failing tests to pass
- Multiple Canon CustomFunctions tests currently skip subdirectory processing
- Tests will validate correct variant selection when implementation complete

---

## Remaining Tasks

### Task A: Extract SubDirectory Definitions with Conditions

**Success Criteria**:
- [ ] **Implementation**: Perl extraction script → `codegen/scripts/extract_subdirectory_defs.pl` extracts all variants
- [ ] **Integration**: Codegen uses definitions → `codegen/src/main.rs:145` calls extraction script
- [ ] **Generated data**: JSON files created → `extracted_data/Image_ExifTool_Canon_subdirectory.json` contains tag 0xf variants
- [ ] **Validation**: Canon 0xf has 50+ variants → `jq '.["0x000f"] | length' extracted_data/Image_ExifTool_Canon_subdirectory.json` returns >50

**Implementation Details**: 
- Extract tag variants from ExifTool modules using perl script
- Handle both single definitions and arrays of variants
- Capture Condition, SubDirectory parameters (TagTable, Validate, ProcessProc, etc.)

**Success Patterns**:
- ✅ Canon tag 0xf extracts all CustomFunctions variants with model-specific Conditions
- ✅ Nikon ColorBalance variants captured with different ByteOrder settings
- ✅ JSON structure matches our SubDirectoryDef schema

### Task B: Build TagTable Registry System

**Success Criteria**:
- [ ] **Implementation**: Registry module → `codegen/src/table_registry/mod.rs` implements TagTableRegistry
- [ ] **Integration**: Registry builder → `codegen/src/table_registry/builder.rs` scans generated modules
- [ ] **Generated loader**: Table loader created → `src/generated/table_loader.rs` with load_tag_table() function
- [ ] **Mapping complete**: All tables mapped → `grep -c "=>" src/generated/table_loader.rs` shows 200+ mappings
- [ ] **Unit tests**: Registry lookups work → `cargo t test_table_registry` passes

**Implementation Details**:
- Map ExifTool names like "Image::ExifTool::Canon::CameraSettings" to Rust paths
- Generate massive match statement for runtime table loading
- Include metadata (is_binary_data, format, first_entry)

**Dependencies**: Task A complete (need extracted definitions)

**Success Patterns**:
- ✅ Can load Canon::CameraSettings by string name at runtime
- ✅ Registry knows if table is binary data vs IFD format
- ✅ All existing generated modules are discoverable

### Task C: Enhance Expression Evaluator for SubDirectory Conditions

**Success Criteria**:
- [ ] **Implementation**: Condition evaluation → `src/expressions/mod.rs:evaluate_subdirectory_condition()` added
- [ ] **Model context**: Context structure → `src/expressions/types.rs:SubDirectoryContext` with model/make fields
- [ ] **Perl variable mapping**: $$self support → Parser handles `$$self{Model}` expressions
- [ ] **Unit tests**: Conditions evaluate → `cargo t test_subdirectory_conditions` passes
- [ ] **Integration test**: Canon model matching → Test evaluates `$$self{Model} =~ /EOS-1D/` correctly

**Implementation Details**:
- Add SubDirectoryContext with model, make, firmware, value_data fields
- Map Perl's $$self variables to our context
- Support regex matching for model patterns

**Success Patterns**:
- ✅ `$$self{Model} =~ /EOS 5D/` returns true for "Canon EOS 5D Mark III"
- ✅ Data pattern conditions like `$$valPt =~ /^0204/` work with binary data
- ✅ Complex conditions with AND/OR logic evaluate correctly

### Task D: Create Generic SubDirectory Processor

**Success Criteria**:
- [ ] **Implementation**: Processor module → `src/exif/subdirectory/processor.rs` with SubDirectoryProcessor
- [ ] **Variant selection**: Logic implemented → `select_variant()` picks correct variant based on conditions
- [ ] **Table loading**: Dynamic loading → Uses table_loader to load tag tables by name
- [ ] **Binary vs IFD**: Dual processing → Handles both ProcessBinaryData and ProcessDirectory modes
- [ ] **NO PrintConv**: Raw values only → PrintConv removed from subdirectory processing
- [ ] **Unit tests**: Processor logic → `cargo t test_subdirectory_processor` passes

**Implementation Details**:
- ONE processor for ALL manufacturers (DRY pattern)
- Select variant by evaluating conditions in order
- Load table using registry, process based on table type
- Return raw TagValues without PrintConv

**Dependencies**: Tasks B and C complete

**Success Patterns**:
- ✅ Canon EOS 5D selects CustomFunctions5D variant
- ✅ Canon EOS-1D selects CustomFunctions1D variant
- ✅ Extracted values match ExifTool's raw output (before PrintConv)

### Task E: Generate SubDirectory Definition Files

**Success Criteria**:
- [ ] **Implementation**: Generator strategy → `codegen/src/strategies/subdirectory_gen.rs` generates Rust defs
- [ ] **Generated files**: Definition modules → `src/generated/Canon_pm/subdirectory_defs.rs` exists
- [ ] **All manufacturers**: Coverage complete → Files exist for Canon, Nikon, Sony, Olympus, etc.
- [ ] **Integration**: Modules compile → `cargo build` succeeds with new generated files
- [ ] **Validation**: Definitions match → Spot-check Canon 0xf has all variants from Task A

**Implementation Details**:
- Convert JSON from Task A to Rust static definitions
- Generate LazyLock<HashMap<u16, Vec<SubDirectoryDef>>>
- Include in manufacturer module structure

**Dependencies**: Task A complete

**Success Patterns**:
- ✅ CANON_SUBDIRECTORY_DEFS contains all extracted variants
- ✅ Each variant has condition, tag_table, validate fields populated
- ✅ Generated code compiles without warnings

### Task F: Wire SubDirectory Processing into ExifReader

**Success Criteria**:
- [ ] **Implementation**: Integration point → `src/exif/mod.rs:process_subdirectories()` method added
- [ ] **Context extraction**: Model/make obtained → Gets from extracted tags for condition evaluation
- [ ] **Manufacturer routing**: Per-manufacturer calls → Processes Canon, Nikon, Sony subdirectories
- [ ] **Tag replacement**: Arrays removed → Original binary arrays replaced with extracted tags
- [ ] **Integration test**: End-to-end works → `cargo t test_canon_customfunctions` passes
- [ ] **Manual validation**: Output changes → `cargo run -- canon_eos_5d.cr2` shows CustomFunctions tags

**Implementation Details**:
- Call after main tag extraction, before PrintConv
- Build ProcessingContext from extracted Model/Make tags
- Process each manufacturer's subdirectory tags
- Store extracted tags with synthetic IDs

**Dependencies**: Tasks D and E complete

**Success Patterns**:
- ✅ Canon CustomFunctions binary array disappears, individual settings appear
- ✅ Correct variant selected based on camera model
- ✅ All subdirectory tags have proper namespace prefixes

### Task G: Remove PrintConv from Current SubDirectory Implementation

**Success Criteria**:
- [ ] **Cleanup**: PrintConv removed → `src/exif/subdirectory_processing.rs` no longer calls apply_print_conv
- [ ] **Simplified logic**: Just extraction → Function only extracts and stores raw values
- [ ] **Tests still pass**: No regression → `cargo t` succeeds after removal
- [ ] **Documentation**: Updated comments → File header explains PrintConv happens later

**Implementation Details**:
- Remove lines 163-201 (PrintConv application)
- Simplify to just store raw extracted values
- Update function documentation

**Dependencies**: Task F complete (new system must be working first)

**Success Patterns**:
- ✅ Code is simpler and more focused
- ✅ Values stored are raw, not formatted
- ✅ PrintConv happens later in display pipeline

## Implementation Guidance

- **Registry pattern**: Follow our successful conv_registry design
- **Generated code**: Never hand-edit generated files, always fix extraction
- **Trust ExifTool**: Copy behavior exactly, including seemingly odd variant selection
- **Test with real cameras**: Use actual camera files from test-images/

## Integration Requirements

### Mandatory Integration Proof

- [ ] **Activation**: Subdirectory processing automatic → `src/exif/mod.rs:234` calls process_subdirectories()
- [ ] **Consumption**: Tags properly extracted → `cargo run -- canon_40d.cr2 | grep CustomFunction` shows values
- [ ] **Measurement**: Output matches ExifTool → Comparison script shows identical CustomFunctions
- [ ] **Cleanup**: Old broken code removed → `git log` shows removal of incorrect PrintConv application

## Prerequisites

- P06 (Unified Codegen) should be complete for optimal integration

## Testing

- **Unit**: Test variant selection, condition evaluation, table loading
- **Integration**: Verify Canon CustomFunctions, Nikon ColorBalance work correctly
- **Manual check**: Run `cargo run -- test-images/canon/eos_5d.cr2` and verify CustomFunctions5D used

## Definition of Done

- [ ] `cargo t test_canon_customfunctions` passes
- [ ] `cargo t test_nikon_colorbalance` passes
- [ ] `make precommit` clean
- [ ] Canon EOS models select correct CustomFunction variant
- [ ] PrintConv removed from subdirectory processing
- [ ] Output matches ExifTool for subdirectory tags