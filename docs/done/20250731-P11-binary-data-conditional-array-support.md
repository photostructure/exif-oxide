# Binary Data Conditional Array Support Implementation

## Project Overview

- **Goal**: Implement conditional array support for ExifTool's ProcessBinaryData tables, enabling proper extraction of manufacturer-specific tags with model-dependent processing logic
- **Problem**: Canon ShotInfo ExposureTime tag (offset 22) has different value conversion formulas for 20D/350D vs other models, requiring conditional variant selection at runtime
- **Constraints**: Must maintain backward compatibility with existing binary data processing, integrate seamlessly with codegen pipeline

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

- **ProcessBinaryData Pipeline**: ExifTool uses ProcessBinaryData tables to extract metadata from manufacturer-specific binary data blocks, with format specifications like `int16s`, `FIRST_ENTRY => 1`, and tag offset mappings
- **Codegen Infrastructure**: Perl extraction scripts (`process_binary_data.pl`) parse ExifTool modules to generate Rust binary data parsers, creating HashMap lookups and processing functions
- **Conditional Arrays**: ExifTool supports arrays of tag definitions at the same offset, with conditions like `$self{Model} =~ /\b(20D|350D)\b/` to select appropriate variant based on camera model

### Key Concepts & Domain Knowledge

- **Binary Data Variants**: Single tag offset can have multiple definitions with different ValueConv/PrintConv logic depending on camera model, firmware, or other context
- **Condition Evaluation**: ExifTool conditions use Perl regex and variable references (`$self{Model}`, `$val{0}`) that must be translated to our expression system
- **Two-Phase Processing**: Binary extraction produces raw values, then PrintConv/ValueConv converts to human-readable format using model-specific logic

### Surprising Context

**CRITICAL**: Document non-intuitive aspects that aren't obvious from casual code inspection:

- **Array Structure Complexity**: ExifTool uses array references `22 => [{ Condition => '...', Name => 'Tag' }, { Name => 'Tag' }]` where first matching condition wins, fallback to unconditioned variant
- **Condition Syntax Translation**: ExifTool `$self{Model}` becomes our `$model` in expression evaluator, regex patterns need escaping adjustments
- **Backward Compatibility Requirements**: Existing `BinaryDataTag` struct must support both legacy single-definition tags and new conditional variants without breaking existing code
- **Expression System Integration**: Our `ExpressionEvaluator` already supports context conditions but needed integration point for binary data variant selection

### Foundation Documents

- **ExifTool Reference**: `third-party/exiftool/lib/Image/ExifTool/Canon.pm` lines 2721+ (ShotInfo table), specifically tag 22 with conditional ExposureTime variants
- **Codegen Infrastructure**: `codegen/extractors/process_binary_data.pl` - Perl extraction script that parses ProcessBinaryData tables
- **Binary Data Types**: `src/types/binary_data.rs` - Core types for ProcessBinaryData handling
- **Expression System**: `src/expressions/mod.rs` - Condition evaluation framework

### Prerequisites

- **Knowledge assumed**: Understanding of ExifTool ProcessBinaryData format, Rust pattern matching, conditional compilation
- **Setup required**: Working codegen environment, Canon test images for validation

**Context Quality Check**: Can a new engineer understand WHY conditional arrays are needed (different camera models need different processing) and HOW they integrate with existing systems?

## Work Completed

- ✅ **Conditional Array Extraction** → Enhanced `process_binary_data.pl` to detect array structures and extract variants with conditions, format specifications, and PrintConv data
- ✅ **Type System Extension** → Added `BinaryDataTagVariant` struct and `variants` field to `BinaryDataTag`, maintaining backward compatibility with `from_legacy()` helper
- ✅ **Condition Evaluation Integration** → Implemented `get_active_variant()` method using existing `ExpressionEvaluator::evaluate_context_condition()` for runtime variant selection
- ✅ **Codegen Schema Updates** → Updated `TagVariant` struct in `process_binary_data.rs` generator to handle conditional arrays in JSON extraction
- ✅ **Canon ShotInfo Integration** → Added ShotInfo to Canon binary data config, successfully generating conditional ExposureTime variants
- ✅ **Test Validation** → Created comprehensive test validating Canon 20D vs 5D ExposureTime processing shows correct variant selection and different ValueConv formulas
- ✅ **Condition Translation Fix** → Added `translate_condition()` function to convert ExifTool's `$$self{Model}` format to expression system's `$model` format, enabling proper runtime condition evaluation

## Remaining Tasks

**STATUS**: All tasks completed successfully. Implementation is fully functional.

### Integration Status

✅ **Activation**: Conditional array support is automatically used when ProcessBinaryData tables contain array structures  
✅ **Consumption**: Canon ShotInfo processing now uses generated conditional variants instead of stub implementations  
✅ **Measurement**: Tests validate correct variant selection for different camera models  
✅ **Cleanup**: Legacy single-variant path maintained for backward compatibility, no obsolete code remains

## Working Definition of "Complete"

A feature is complete when:
- ✅ **System behavior changes** - Canon ExposureTime now uses model-specific conversion formulas instead of generic processing
- ✅ **Default usage** - Conditional arrays are processed automatically during binary data extraction, no opt-in required
- ✅ **Old path removed** - No obsolete code, legacy support maintained through backward-compatible helpers
- ✅ Code is actively used - Canon ShotInfo processing leverages generated conditional variants for ExposureTime tag

## Definition of Done

- ✅ `cargo test types::binary_data::tests::test_conditional_array_exposure_time` passes
- ✅ `cargo check` clean compilation
- ✅ Canon ShotInfo binary data extraction generates proper conditional variants
- ✅ ExifTool condition format (`$$self{Model}`) correctly translated to expression system format (`$model`)
- ✅ Runtime condition evaluation works correctly for different Canon camera models

## Implementation Guidance

### Key Technical Achievements

- **Seamless Integration**: Conditional arrays work transparently with existing binary data pipeline, no API changes required
- **Expression System Reuse**: Leveraged existing `evaluate_context_condition()` without modifications, only needed proper condition format translation
- **Backward Compatibility**: All existing binary data processing continues unchanged, new variant system is additive
- **Codegen Automation**: Conditional arrays are automatically extracted and generated, no manual maintenance required

### Architecture Patterns Used

- **Variant Selection**: Runtime condition evaluation with fallback to default variant ensures robustness
- **Helper Methods**: `from_legacy()` and `simple()` constructors maintain clean migration path for existing code
- **JSON Schema Evolution**: Extended extraction schema to support variants while maintaining simple tag compatibility

### ExifTool Translation Patterns

- **Condition Translation**: ExifTool `$$self{Model} =~ /\b(20D|350D)\b/` → our `$model =~ /(20D|350D)/`
- **Variable Mapping**: `$$self{Model}` → `$model`, `$$self{Make}` → `$manufacturer`, `$$self{Manufacturer}` → `$manufacturer`
- **Regex Cleanup**: Remove `\b` word boundaries (handled differently by our regex engine)
- **Array Structure**: ExifTool array refs `[{Condition => ..., Name => ...}, {...}]` → our `Vec<BinaryDataTagVariant>`
- **Fallback Logic**: ExifTool's "first match wins, unconditioned as default" → our variant iteration with condition evaluation

## Quick Debugging

For future conditional array issues:

1. `rg "conditional.*true" codegen/generated/extract/binary_data/` - Find extracted conditional tags
2. `cargo test --lib types::binary_data::tests::test_conditional_array_exposure_time` - Verify variant selection logic
3. Check `$model` vs `$self{Model}` in condition strings - common translation issue
4. Verify ProcessorContext has correct `manufacturer` and `model` fields populated

---

**COMPLETION STATUS**: ✅ All tasks completed successfully. Conditional array support is fully implemented and tested.