# Technical Project Plan: Tag Kit PrintConv Automation System

## Project Overview

- **Goal**: Establish systematic extraction of legitimate ExifTool PrintConv patterns to eliminate manual transcription errors and reduce maintenance burden
- **Problem**: Risk of regression to manual HashMap transcription errors after cleanup, plus legitimate PrintConv logic that should be automated but isn't
- **Constraints**: Must distinguish legitimate PrintConv patterns from string-passthrough tags, zero-maintenance solution for monthly ExifTool updates

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

- **Tag Kit System**: Unified code generation pipeline that extracts tag definitions and conversion logic from ExifTool modules, generating Rust code with embedded PrintConv implementations
- **PrintConv Pipeline**: ExifTool's human-readable value conversion system (raw → ValueConv → PrintConv → display), where PrintConv handles final formatting like "ISO 1600" vs raw value "17"
- **Manual HashMap Problem**: Historical pattern of manually transcribing ExifTool lookup tables, leading to transcription errors that waste engineering days debugging

### Key Concepts & Domain Knowledge

- **Legitimate PrintConv**: ExifTool tag definitions with actual numeric→string conversion logic (e.g., `PrintConv => {0=>'Off',1=>'On'}`)
- **String Passthrough Tags**: ExifTool tags marked `Writable => 'string'` with no PrintConv logic - should return raw values
- **Inline PrintConv**: Conversion logic embedded directly in tag definitions vs standalone hash references
- **Complex PrintConv**: Conditional expressions, formulas, or multi-step logic requiring perl evaluation

### Surprising Context

**CRITICAL**: Document non-intuitive aspects that aren't obvious from casual code inspection:

- **Most "PrintConv" isn't**: Many ExifTool tags that look like they need conversion actually use string passthrough - the recent Nikon cleanup revealed most manual HashMaps were invalid
- **Tag Kit Can Handle Inline**: The tag kit system already supports inline PrintConv extraction, but configuration patterns aren't established for complex cases
- **Monthly ExifTool Updates**: Manual PrintConv transcription becomes maintenance burden with ExifTool's monthly release cycle
- **Transcription Error Pattern**: Historical "4 engineering days chasing ghosts" from single incorrect values in manually transcribed arrays
- **Codegen vs Manual Trade-off**: 100% automated extraction prevents transcription errors but requires sophisticated perl parsing for complex cases

### Foundation Documents

- **Design docs**: [CODEGEN.md](../CODEGEN.md) tag kit system, [PRINTCONV-VALUECONV-GUIDE.md](../guides/PRINTCONV-VALUECONV-GUIDE.md) conversion implementation patterns
- **ExifTool source**: Various modules' tag definitions with PrintConv patterns, e.g., `third-party/exiftool/lib/Image/ExifTool/Canon.pm` conditional logic
- **Start here**: `codegen/src/generators/tag_kit.rs` (tag kit generator), recent Nikon cleanup in `src/implementations/nikon/tags/print_conv/basic.rs`

### Prerequisites

- **Knowledge assumed**: Understanding of ExifTool tag table structure, Rust codegen principles, perl expression evaluation challenges
- **Setup required**: Working codegen pipeline, sample ExifTool modules with legitimate PrintConv patterns

**Context Quality Check**: Can a new engineer understand WHY automated PrintConv extraction prevents critical engineering time waste?

## Work Completed

- ✅ **Manual HashMap Cleanup** → Removed invalid manual lookup tables in Nikon implementation, established "Trust ExifTool" compliance patterns
- ✅ **Tag Kit System** → Unified extraction pipeline operational for basic tag definitions and simple PrintConv patterns
- ✅ **Transcription Error Research** → Documented historical 4-day debugging incidents caused by manual array transcription errors

## Remaining Tasks

**REQUIRED**: Each task must be numbered, actionable, and include success criteria.

### 1. Task: Research and Catalog Legitimate PrintConv Patterns

**Success Criteria**: Comprehensive analysis document identifying legitimate vs invalid PrintConv patterns across 3-5 manufacturer modules, with specific ExifTool source line references
**Approach**: Systematic audit of ExifTool modules to classify PrintConv types: standalone hashes, inline expressions, conditional logic, string passthrough
**Dependencies**: None

**Success Patterns**:

- ✅ Clear categorization of legitimate PrintConv patterns that should be automated
- ✅ Documentation of complex cases requiring specialized extraction logic
- ✅ Identification of string passthrough tags that should NOT have PrintConv
- ✅ Priority ranking based on tag frequency and transcription error risk

### 2. Task: Extend Tag Kit for Complex PrintConv Extraction

**Success Criteria**: Tag kit system successfully extracts and generates Rust code for complex PrintConv patterns identified in Task 1
**Approach**: Enhance perl extraction logic to handle conditional expressions, inline formulas, and multi-step conversion logic
**Dependencies**: Task 1 (pattern identification)

**Success Patterns**:

- ✅ Complex PrintConv expressions correctly translated to Rust match statements
- ✅ Generated code produces identical output to ExifTool for all test cases
- ✅ Fallback handling for unmapped values matches ExifTool's "Unknown (N)" format
- ✅ Zero manual transcription required for legitimate PrintConv patterns

### 3. Task: Configure PrintConv Automation for High-Risk Patterns

**Success Criteria**: All manually-identified high transcription error risk patterns (like 30-entry ISO mapping) converted to automated extraction
**Approach**: Apply enhanced tag kit system to real-world cases, validate against ExifTool output
**Dependencies**: Task 2 (extraction capability)

**Success Patterns**:

- ✅ ISO mapping and other high-risk manual arrays replaced with generated code
- ✅ All manufacturer modules checked for remaining manual HashMap violations
- ✅ Codegen configs documented for future maintenance
- ✅ Test coverage validates generated vs ExifTool output

### 4. Task: Establish PrintConv Automation Guidelines

**Success Criteria**: Clear documentation and patterns for future engineers to identify and automate legitimate PrintConv logic
**Approach**: Create decision tree and examples for when to use tag kit vs manual implementation
**Dependencies**: Tasks 1-3 (practical experience with automation)

**Success Patterns**:

- ✅ Decision flowchart for PrintConv automation vs manual implementation
- ✅ Examples of properly configured tag kit extraction for complex cases
- ✅ Guidelines prevent future manual HashMap transcription errors
- ✅ Integration with "Trust ExifTool" principles and documentation

**Task Quality Check**: Can another engineer pick up any task and complete it without asking clarifying questions?

## Integration Requirements

**CRITICAL**: Building without integrating is failure. Don't accept tasks that build "shelf-ware."

Every feature must include:
- [ ] **Activation**: Enhanced tag kit system is used for legitimate PrintConv patterns by default
- [ ] **Consumption**: Existing print conversion functions migrate to generated code where appropriate
- [ ] **Measurement**: Can prove PrintConv automation working via ExifTool comparison and reduced manual code
- [ ] **Cleanup**: Manual HashMap lookup tables replaced with generated equivalents, obsolete transcription eliminated

**Red Flag Check**: If this seems like "build better codegen but don't use it," ask for clarity. We're automating PrintConv to eliminate transcription errors and maintenance burden for PhotoStructure's metadata accuracy.

## Working Definition of "Complete"

*Use these criteria to evaluate your own work - adapt to your specific context:*

A feature is complete when:
- ✅ **System behavior changes** - Legitimate PrintConv patterns are automatically extracted instead of manually transcribed
- ✅ **Default usage** - Tag kit system handles complex PrintConv cases without manual intervention
- ✅ **Old path removed** - Manual HashMap transcription is eliminated for patterns that can be automated
- ❌ Code exists but isn't used *(example: "complex extraction implemented but manual HashMaps remain")*
- ❌ Feature works "if you call it directly" *(example: "enhanced tag kit exists but configs don't use it")*

*Note: These are evaluation guidelines, not literal requirements for every task.*

## Prerequisites

None - tag kit system already operational for basic cases

## Testing

- **Unit**: Test complex PrintConv extraction with known ExifTool patterns
- **Integration**: Verify end-to-end tag processing produces identical output to ExifTool for converted patterns
- **Manual check**: Run `cargo run --bin compare-with-exiftool test_image.jpg` and confirm no PrintConv regressions

## Definition of Done

- [ ] `cargo t tag_kit_printconv` passes for complex extraction patterns
- [ ] `make precommit` clean
- [ ] ExifTool comparison shows no regressions in converted PrintConv patterns
- [ ] Manual HashMap audit confirms no remaining high-risk transcription cases

## Implementation Guidance

### Recommended Patterns

- **Perl Expression Parsing**: Use ExifTool's own evaluation for complex expressions, then generate equivalent Rust code
- **Fallback Strategy**: Always include "Unknown (N)" fallback matching ExifTool's behavior
- **Test-Driven Extraction**: For each PrintConv pattern, create test case with known ExifTool output first

### Tools to Leverage

- **Existing tag kit infrastructure**: Build on proven extraction pipeline
- **ExifTool comparison tools**: Use `compare-with-exiftool` binary for validation
- **Simple table patterns**: Reference working simple_table extraction for complex pattern inspiration

### ExifTool Translation Notes

- **Conditional PrintConv**: ExifTool's `PrintConv => '$val == 0 ? "Off" : "On"'` patterns need rust match translation
- **Hash References**: `PrintConv => \%hashName` patterns already handled by tag kit
- **Formula Expressions**: Mathematical expressions require careful perl evaluation and rust equivalent generation

## Clear Application for PhotoStructure

**Primary Motivation**: PhotoStructure users need human-readable metadata values like "ISO 1600" and "AF-S" instead of raw numeric codes. The tag kit automation ensures these conversions happen reliably without manual transcription errors that could cause missing or incorrect metadata display.

**Specific Impact**:
- **Accuracy**: Eliminates transcription errors that show wrong camera settings to users
- **Coverage**: Automates complex conversion patterns that might otherwise be skipped due to implementation difficulty  
- **Maintenance**: Monthly ExifTool updates automatically pick up new conversion patterns without engineering effort
- **Reliability**: Prevents "4 engineering days chasing ghosts" debugging sessions from manual array mistakes

## Quick Debugging

Stuck? Try these:

1. `rg "PrintConv.*=>" third-party/exiftool/lib/Image/ExifTool/` - Find PrintConv patterns
2. `cargo t tag_kit -- --nocapture` - See tag kit extraction debug output
3. `./scripts/compare-with-exiftool.sh test.jpg` - Compare conversion output with ExifTool
4. Check `src/generated/*/tag_kit/` for existing successful extractions