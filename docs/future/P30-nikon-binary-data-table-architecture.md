# Technical Project Plan: Nikon BinaryDataTable Architecture System

## Project Overview

- **Goal**: Replace ad-hoc binary data processing with systematic BinaryDataTable structures that mirror ExifTool's proven architecture, enabling easy addition of new camera models
- **Problem**: Current Nikon binary data extraction is functional but scattered across custom processing functions, making it difficult to add new models and maintain consistency
- **Constraints**: Must maintain existing functionality while transitioning to structured approach, support encrypted Nikon sections

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

**REQUIRED**: Assume reader is unfamiliar with this domain. Provide comprehensive context.

### System Overview

- **Nikon Binary Processing**: Current system with complete decryption and model-specific processing for D850, Z8, Z9, Z7 cameras, extracting metadata from encrypted ShotInfo, LensData, and ColorBalance sections
- **ExifTool BinaryData Architecture**: Structured table-driven approach where each camera model has declarative offset/format specifications (e.g., `%Image::ExifTool::Nikon::ShotInfoD850`)
- **Current Implementation**: Hardcoded offset calculations scattered across `encrypted_processing.rs`, `binary_data_extraction.rs` with custom processing logic per model

### Key Concepts & Domain Knowledge

- **ShotInfo Variants**: Camera-specific binary data structures containing exposure settings, each model has different layouts and offset positions
- **BinaryDataTable Structure**: Declarative specification of field offsets, formats, and tag mappings that drive generic processing logic
- **Encrypted Section Processing**: Nikon binary data requires decryption before table-based extraction can occur
- **Model Detection**: Camera model determines which BinaryDataTable variant to use for processing

### Surprising Context

**CRITICAL**: Document non-intuitive aspects that aren't obvious from casual code inspection:

- **ExifTool Uses Tables**: ExifTool's Nikon.pm defines structured tables like `%Image::ExifTool::Nikon::ShotInfoD850` rather than custom parsing code
- **Current System Works**: Existing implementation successfully extracts all required tags from encrypted sections - this is optimization/maintainability work, not bug fixing
- **Model-Specific Offsets**: Z-series cameras use different offset calculation schemes than DSLR models (0x24 vs 0x0c base offsets)
- **Table-Driven Benefits**: Adding new camera model becomes "define table + register model" instead of "write custom parsing logic"
- **Generic Processing Power**: Single table processor can handle any camera model with appropriate table definition

### Foundation Documents

- **Design docs**: [CORE-ARCHITECTURE.md](../guides/CORE-ARCHITECTURE.md) binary data processing patterns, existing Nikon implementation architecture
- **ExifTool source**: `third-party/exiftool/lib/Image/ExifTool/Nikon.pm` BinaryData table definitions (lines ~14500-15500 for various ShotInfo tables)
- **Start here**: `src/implementations/nikon/encrypted_processing.rs` current binary processing, `src/implementations/nikon/binary_data_extraction.rs` extraction patterns

### Prerequisites

- **Knowledge assumed**: Understanding of binary data parsing, Rust struct design, ExifTool BinaryData table format
- **Setup required**: Working Nikon decryption system, test files from supported camera models

**Context Quality Check**: Can a new engineer understand WHY structured tables are better than hardcoded parsing logic?

## Work Completed

- ✅ **Complete Nikon Decryption** → Core decryption algorithms working for all encrypted sections (ShotInfo, LensData, ColorBalance)
- ✅ **Model-Specific Processing** → D850, Z8, Z9, Z7 support with model detection and offset handling
- ✅ **End-to-End Integration** → Encrypted binary data processing integrated into main Nikon pipeline with 79+ passing tests

## Remaining Tasks

**REQUIRED**: Each task must be numbered, actionable, and include success criteria.

### 1. Task: Design BinaryDataTable Structure

**Success Criteria**: Rust struct definitions that can represent ExifTool's BinaryData table format with support for encrypted sections
**Approach**: Analyze ExifTool's ShotInfo table patterns, design Rust equivalent with offset/format/tag specifications
**Dependencies**: None

**Success Patterns**:

- ✅ BinaryDataTable struct represents ExifTool table format faithfully
- ✅ Support for various data formats (u8, u16, u32, i16, string, etc.)
- ✅ Encrypted section handling integrated into table processing
- ✅ Clear separation between table definition and processing logic

### 2. Task: Implement Generic Table Processor

**Success Criteria**: Single processor function that can extract tags from any binary data using BinaryDataTable definitions
**Approach**: Build table-driven processor that replaces model-specific hardcoded parsing logic
**Dependencies**: Task 1 (table structure design)

**Success Patterns**:

- ✅ Generic processor handles all existing camera models without custom code
- ✅ Encrypted data decryption integrated seamlessly with table processing
- ✅ Error handling for malformed binary data and missing fields
- ✅ Debug output shows table-driven field extraction process

### 3. Task: Convert Existing Models to Table Definitions

**Success Criteria**: D850, Z8, Z9, Z7 processing converted from hardcoded logic to BinaryDataTable definitions with identical output
**Approach**: Extract existing offset/format logic into declarative table configurations
**Dependencies**: Task 2 (generic processor)

**Success Patterns**:

- ✅ All existing model processing produces identical tag output
- ✅ Table definitions match ExifTool's ShotInfo table structures
- ✅ No regressions in existing test coverage (79+ tests continue passing)
- ✅ Code volume reduction from eliminating custom parsing functions

### 4. Task: Enable Easy Model Addition

**Success Criteria**: Adding new camera model requires only table definition + model registration, demonstrated with one additional model
**Approach**: Create configuration system for new model support with minimal code changes
**Dependencies**: Task 3 (existing models converted)

**Success Patterns**:

- ✅ New model support added with <20 lines of code (table definition only)
- ✅ Model detection automatically routes to appropriate table
- ✅ Generic processor handles new model without modifications
- ✅ Documentation shows clear process for adding future models

**Task Quality Check**: Can another engineer pick up any task and complete it without asking clarifying questions?

## Integration Requirements

**CRITICAL**: Building without integrating is failure. Don't accept tasks that build "shelf-ware."

Every feature must include:
- [ ] **Activation**: BinaryDataTable system replaces existing hardcoded binary processing by default
- [ ] **Consumption**: Main Nikon processing pipeline uses table-driven extraction automatically
- [ ] **Measurement**: Can prove table system working via identical tag extraction output and simplified codebase
- [ ] **Cleanup**: Custom parsing functions removed, hardcoded offset calculations eliminated

**Red Flag Check**: If this seems like "build table system but keep using hardcoded parsing," ask for clarity. We're replacing scattered processing logic with systematic architecture to enable easy model support for PhotoStructure.

## Working Definition of "Complete"

*Use these criteria to evaluate your own work - adapt to your specific context:*

A feature is complete when:
- ✅ **System behavior changes** - New camera models can be added via table definition instead of custom code
- ✅ **Default usage** - Table-driven processing is used automatically for all supported models
- ✅ **Old path removed** - Hardcoded offset calculations and custom parsing functions eliminated
- ❌ Code exists but isn't used *(example: "table system implemented but hardcoded parsing remains active")*
- ❌ Feature works "if you call it directly" *(example: "table processor exists but isn't integrated into main pipeline")*

*Note: These are evaluation guidelines, not literal requirements for every task.*

## Prerequisites

None - existing Nikon implementation provides complete foundation

## Testing

- **Unit**: Test table processor with known ExifTool BinaryData patterns
- **Integration**: Verify end-to-end tag extraction produces identical output for all existing camera models
- **Manual check**: Run `cargo t nikon` and confirm all 79+ tests pass with table-driven processing

## Definition of Done

- [ ] `cargo t nikon` passes for all existing functionality with table-driven processing
- [ ] `make precommit` clean
- [ ] Table-driven system produces identical tag output to hardcoded approach
- [ ] Adding new model demonstrated with table definition only

## Implementation Guidance

### Recommended Patterns

- **Table Structure**: Mirror ExifTool's BinaryData format with Rust type safety
- **Generic Processing**: Single processor function with table parameter, not per-model functions
- **Configuration-Driven**: Model registration via const definitions, not hardcoded dispatch logic

### Tools to Leverage

- **Existing encrypted processing**: Build on proven decryption and model detection systems
- **Binary parsing utilities**: Use established byte order and format conversion patterns
- **Test infrastructure**: Leverage comprehensive existing test coverage for validation

### ExifTool Translation Notes

- **BinaryData Tables**: ExifTool's `ProcessProc => \&ProcessBinaryData` patterns with offset/format specifications
- **ShotInfo Examples**: Study `%Image::ExifTool::Nikon::ShotInfoD850` structure for table definition patterns
- **Format Codes**: ExifTool's format codes (int8u, int16u, etc.) need Rust type equivalents

## Clear Application for PhotoStructure

**Primary Motivation**: PhotoStructure users with newer Nikon cameras (like future Z-series models) should get the same rich metadata extraction as currently supported cameras. The BinaryDataTable system makes it trivial to add support for new models without custom development effort.

**Specific Impact**:
- **Rapid Model Support**: New camera models can be supported in hours instead of days by adding table definitions
- **Consistency**: All Nikon cameras use the same extraction architecture, reducing bugs and maintenance
- **Future-Proofing**: Architecture scales to handle Nikon's continued camera releases without code rewrites
- **Reliability**: Table-driven approach reduces chances of parsing errors from hardcoded offset calculations
- **Maintenance**: Single codebase handles all models instead of scattered custom processing functions

**Business Context**: Camera manufacturers release new models frequently. A systematic approach to binary data processing ensures PhotoStructure can support new cameras quickly, keeping users happy and reducing support burden.

## Additional Gotchas & Tribal Knowledge

**Format**: Surprise → Why → Solution (Focus on positive guidance)

- **ExifTool tables look complex** → They handle 20+ years of camera variations → Start with simple cases, build complexity gradually
- **Binary data seems fragile** → Camera firmware bugs require defensive parsing → Always validate offsets and data lengths before extraction
- **Table-driven feels slower** → Performance difference is negligible → Focus on maintainability benefits over micro-optimizations
- **Model variants multiply** → Each camera has subtly different data layouts → Use model detection to route to appropriate table definitions

## Quick Debugging

Stuck? Try these:

1. `rg "ShotInfo" third-party/exiftool/lib/Image/ExifTool/Nikon.pm` - Find ExifTool table patterns
2. `cargo t nikon_encrypted -- --nocapture` - See binary data extraction debug output
3. `xxd binary_data.bin` - Hex dump binary data to understand structure
4. Compare table definition with ExifTool's BinaryData format for consistency