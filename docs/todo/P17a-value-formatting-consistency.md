# Technical Project Plan: Value Formatting Consistency

## Project Overview

- **Goal**: Ensure all tag values match ExifTool's exact formatting conventions by identifying and fixing remaining formatting edge cases through systematic compatibility testing
- **Problem**: Despite comprehensive PrintConv infrastructure, specific formatting edge cases still cause compatibility test failures and inconsistent user experience
- **Constraints**: Zero runtime overhead, maintain existing API compatibility, preserve numeric types in JSON output

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

- **Value Conversion Pipeline**: Raw bytes → ValueConv (mathematical) → PrintConv (formatting) → JSON output. ValueConv maintains precision for calculations; PrintConv handles display formatting and type detection.
- **PrintConv Infrastructure**: Comprehensive `src/implementations/print_conv.rs` with 40+ formatting functions for EXIF standard, GPS, Canon-specific, and composite tags. Most core EXIF tags already implemented.
- **Codegen System**: Generated lookup tables in `src/generated/*/` including Flash modes, Orientation values, and manufacturer-specific tables extracted from ExifTool Perl source.
- **Comparison Tools**: `compare-with-exiftool.rs` binary uses same normalization logic as compatibility tests, filtering formatting variations to show only actual differences.

### Key Concepts & Domain Knowledge

- **JSON Type Preservation**: ExifTool outputs numeric JSON for values that look like numbers, string JSON otherwise. Our `TagValue::string_with_numeric_detection()` mimics this behavior.
- **ExifTool's Formatting Hierarchy**: PrintConv functions can return different types - numeric values become JSON numbers, strings become JSON strings. Type preservation affects downstream tool compatibility.
- **Rational Formatting Rules**: Based on ExifTool's PrintFNumber/PrintExposureTime logic - different precision rules for different value ranges and usage contexts.

### Surprising Context

- **PrintConv Infrastructure Already Exists**: Unlike the original TPP assumptions, comprehensive PrintConv infrastructure is implemented. FNumber, ExposureTime, FocalLength, Flash, and most core EXIF tags have working implementations.
- **Generated Tables Working**: Flash lookup, Orientation, and other manufacturer tables are generated and working. The original TPP's "Phase 2" tasks are largely complete.
- **Value vs Display Distinction**: ExifTool sometimes returns numeric values (for calculations) vs formatted strings (for display). Our system must match both behaviors depending on context.
- **Edge Case Focus**: Since major formatting is working, remaining issues are likely edge cases around precision, units, or specific value ranges.

### Foundation Documents

- **Current Implementation**: `src/implementations/print_conv.rs` - all existing PrintConv functions with ExifTool citations
- **Value Pipeline**: `src/implementations/value_conv.rs` - mathematical conversions feeding into PrintConv
- **Comparison Infrastructure**: `src/bin/compare-with-exiftool.rs` - normalized comparison tool
- **ExifTool References**: All PrintConv functions cite specific ExifTool source lines

### Prerequisites

- **Knowledge assumed**: Understanding of ExifTool's JSON output behavior and type detection
- **Setup required**: `compare-with-exiftool` binary built, test images available

## Work Completed

- ✅ **PrintConv Infrastructure** → Comprehensive system with 40+ functions implemented
- ✅ **Core EXIF Formatting** → FNumber, ExposureTime, FocalLength, Flash, Orientation working
- ✅ **Generated Lookup Tables** → Flash, Orientation, and manufacturer tables from codegen
- ✅ **Comparison Tools** → `compare-with-exiftool.rs` with normalization logic
- ✅ **Value Conversion Pipeline** → Mathematical conversions for GPS, APEX, Canon, Sony values

## Remaining Tasks

### 1. Task: Identify Current Formatting Differences Through Systematic Testing

**Success Criteria**: Complete analysis of formatting differences across representative test files with specific examples of remaining edge cases
**Approach**: Use comparison tools and compatibility tests to identify specific formatting mismatches
**Dependencies**: None

**Success Patterns**:
- ✅ Run `compare-with-exiftool` on 10+ representative files covering major manufacturers
- ✅ Categorize differences into: precision issues, unit formatting, type detection, missing PrintConvs
- ✅ Document specific ExifTool vs exif-oxide examples for each category
- ✅ Prioritize by frequency and impact on required tags from `docs/tag-metadata.json`

### 2. Task: Fix High-Priority Formatting Edge Cases

**Success Criteria**: Address the most common formatting differences identified in Task 1 with ExifTool-matching implementations
**Approach**: Update existing PrintConv functions or add missing ones based on systematic analysis
**Dependencies**: Task 1 must identify specific issues

**Success Patterns**:
- ✅ Each fix cites specific ExifTool source line numbers
- ✅ Fixes maintain JSON type compatibility with ExifTool
- ✅ Edge cases in precision, units, or value ranges addressed
- ✅ Regression tests prevent breaking existing working tags

### 3. RESEARCH: Analyze Composite Tag Formatting Consistency

**Objective**: Determine if composite tags (Aperture, ShutterSpeed, ImageSize) format consistently with their EXIF counterparts
**Success Criteria**: Document any composite-specific formatting requirements and implementation gaps
**Done When**: Clear report on composite tag formatting status with specific examples

### 4. Task: Implement Missing PrintConv Functions Identified by Analysis

**Success Criteria**: Any missing PrintConv functions identified in analysis are implemented with ExifTool compatibility
**Approach**: Add new PrintConv functions to `print_conv.rs` following existing patterns
**Dependencies**: Tasks 1-3 must identify specific missing functions

**Success Patterns**:
- ✅ New functions follow ExifTool source exactly with proper citations
- ✅ Functions use `TagValue::string_with_numeric_detection()` for type preservation
- ✅ Unit tests verify formatting matches ExifTool for edge cases

## Implementation Guidance

### Recommended Patterns

- **Use Comparison Tools First**: Always run `compare-with-exiftool file.jpg` to see actual differences before making changes
- **Type Detection Pattern**: Use `TagValue::string_with_numeric_detection(result)` to match ExifTool's JSON numeric detection
- **ExifTool Citation Pattern**: Every function should have `/// ExifTool: lib/Image/ExifTool/Module.pm:line_numbers` comments
- **Edge Case Testing**: Test boundary conditions (zero values, very large/small numbers, undefined rationals)

### Tools to Leverage

- **`compare-with-exiftool.rs`**: Primary comparison tool with normalization logic
- **Compatibility test framework**: `tests/exiftool_compatibility_tests.rs` for systematic verification
- **Existing PrintConv functions**: Pattern library for consistent implementation style

### ExifTool Translation Notes

- **Numeric Detection**: ExifTool uses `Image::ExifTool::IsFloat()` - we use `string_with_numeric_detection()`
- **Rational Handling**: Different precision rules for different contexts (FNumber vs FocalLength)
- **Unit Formatting**: Always include units where ExifTool does ("mm", "s", etc.)

## Testing

- **Systematic Comparison**: Run `compare-with-exiftool` on test files from each major manufacturer
- **Regression Testing**: Verify fixes don't break existing working tags with `cargo t`
- **Edge Case Validation**: Test with zero denominators, infinite values, very large numbers
- **Type Preservation**: Verify JSON output types match ExifTool exactly

## Definition of Done

- [ ] `compare-with-exiftool` shows <5 formatting differences on representative test files
- [ ] No regressions in existing compatibility test pass rates
- [ ] All required tags from `docs/tag-metadata.json` format consistently with ExifTool
- [ ] `make precommit` passes cleanly

## Gotchas & Tribal Knowledge

**Format Quality Check**: Can you identify the specific formatting difference by running the comparison tool, and does your fix cite the exact ExifTool source?

### Error Patterns to Avoid

- **Over-Engineering**: Don't rewrite working PrintConv functions - focus on edge cases and missing functions
- **Type Confusion**: Preserve ExifTool's JSON type behavior (numeric vs string) exactly
- **Citation Gaps**: Every formatting decision must trace back to specific ExifTool source lines

### Dependencies

- Comparison tools working properly
- Representative test files covering major camera manufacturers
- Understanding of ExifTool's type detection behavior