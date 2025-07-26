# Technical Project Plan: PrintConv Pipeline Fix

**⚠️ SUPERSEDED**: This TPP has been superseded by [P08-printconv-valueconv-registry-architecture.md](P08-printconv-valueconv-registry-architecture.md) which implements a superior codegen-time solution.

## Original Overview (Historical)

This document originally proposed implementing a runtime fallback function to fix the PrintConv pipeline. The approach would have added runtime pattern matching and hardcoded tag ID mappings.

## Why Superseded

The P15b approach is superior because:

1. **No runtime overhead** - All PrintConv lookups resolved at codegen time
2. **Type safety** - Codegen fails if functions don't exist
3. **No circular dependencies** - Clean separation between codegen and runtime
4. **Simpler architecture** - No fallback function or runtime registry needed
5. **Better maintainability** - Central registry in codegen, not scattered mappings

## Migration Path

All the research and analysis from this document has been incorporated into P15b, including:
- Problem identification
- File analysis
- Tag examples
- Testing strategies

Please refer to [P08-printconv-valueconv-registry-architecture.md](P08-printconv-valueconv-registry-architecture.md) for the current implementation plan.

---

*Original document content preserved below for historical reference*

---

# Technical Project Plan: PrintConv Pipeline Fix

## Project Overview

**Goal**: Fix the PrintConv pipeline to show human-readable values instead of raw rational arrays for core EXIF tags.

**Problem**: The tag kit system correctly extracts PrintConv definitions from ExifTool, and manual PrintConv implementations exist and work, but generated code contains TODO placeholders that prevent the connection between them. This causes core camera settings to display as raw values like `[39, 10]` instead of formatted values like `"3.9"`.

**Priority**: P15a - Critical for EXIF tag compatibility (blocks P10a completion)

## Background & Context

### The Dual Registry Architecture

The exif-oxide PrintConv system uses two complementary systems:

1. **Manual Registry** (`src/registry.rs`):
   - Runtime function lookup by name
   - ~20 working PrintConv implementations
   - Works perfectly when called directly

2. **Tag Kit System** (`src/generated/*/tag_kit/`):
   - Auto-generated from ExifTool definitions
   - Contains correct PrintConv metadata
   - Has TODO placeholders preventing execution

### Why This Architecture Exists

- **Codegen Constraint**: Generated code cannot import from `src/` (would create circular dependencies)
- **Runtime Flexibility**: Want to add/fix PrintConv without running `make codegen`
- **ExifTool Updates**: Monthly releases require automated extraction

### Related Prior Work

- **TagEntry API** (Milestone 8b): Separated `value` and `print` fields
- **PrintConv Design** (2025-07-02): Decided PrintConv returns `TagValue` not `String`
- **Tag Kit Migration** (2025-07-23): Moved from inline functions to unified tag kit

## Technical Foundation

### Key Files

1. **Generated Tag Kits** (`src/generated/*/tag_kit/mod.rs`):
   ```rust
   // Lines 977-992 contain TODO placeholders:
   PrintConvType::Expression(expr) => {
       // TODO: Implement expression evaluation
       warnings.push(...);
       value.clone()  // Returns raw value instead of formatted
   }
   PrintConvType::Manual(func_name) => {
       // TODO: Look up in manual registry
       warnings.push(...);
       value.clone()  // Returns raw value instead of formatted
   }
   ```

2. **Runtime Tag Processing** (`src/exif/tags.rs`):
   ```rust
   // Line 300 - Already tries to call fallback!
   if print == value && !warnings.is_empty() {
       if let Some(tag_def) = tag_kit::EXIF_PM_TAG_KITS.get(&(tag_id as u32)) {
           print = apply_manual_print_conv_fallback(tag_def, &value);
       }
   }
   ```

3. **Manual Registry** (`src/registry.rs`):
   - Working implementations for all core EXIF tags
   - Function lookup by name
   - Thread-safe with RwLock

### Existing Infrastructure That Works

- ✅ Tag kit correctly extracts PrintConv types from ExifTool
- ✅ Manual PrintConv functions are implemented and tested
- ✅ TagEntry API separates value and print fields
- ✅ Runtime code already tries to apply fallback
- ✅ Registry system works when called directly

## Current State Analysis

### What's Working ✅

1. **Tag Kit Extraction**:
   - `FNumber (33437)`: `PrintConvType::Manual("complex_expression_printconv")`
   - `FocalLength (37386)`: `PrintConvType::Expression("sprintf(\"%.1f mm\",$val)")`
   - `ExposureTime (33434)`: `PrintConvType::Manual("complex_expression_printconv")`

2. **Manual Implementations**:
   - `fnumber_print_conv()` - Formats F-stops
   - `focallength_print_conv()` - Adds "mm" units
   - `exposuretime_print_conv()` - Formats shutter speeds

3. **Fallback Mechanism**:
   - Runtime already detects tag kit failures
   - Calls `apply_manual_print_conv_fallback()` (but function missing!)

### What's Broken ❌

1. **Expression PrintConv**: TODO placeholder returns raw value
2. **Manual PrintConv**: TODO placeholder returns raw value
3. **Missing Fallback Function**: `apply_manual_print_conv_fallback()` doesn't exist

### Evidence From Real Output

```json
// Current (BROKEN):
{
  "EXIF:FNumber": [39, 10],        // Should be 3.9
  "EXIF:FocalLength": [175, 10],   // Should be "17.5 mm"
  "EXIF:ExposureTime": [1, 80]     // Should be "1/80"
}

// Expected (with working PrintConv):
{
  "EXIF:FNumber": 3.9,
  "EXIF:FocalLength": "17.5 mm",
  "EXIF:ExposureTime": "1/80"
}
```

## Implementation Plan

### Phase 1: Implement Missing Fallback Function

**File**: `src/exif/tags.rs`

```rust
fn apply_manual_print_conv_fallback(tag_def: &TagKitDef, value: &TagValue) -> TagValue {
    use crate::registry;
    
    match &tag_def.print_conv {
        PrintConvType::Expression(expr) => {
            // Handle common sprintf patterns
            handle_expression_print_conv(expr, value)
        }
        PrintConvType::Manual(func_name) => {
            // Map tag IDs to specific functions
            let resolved_name = resolve_manual_print_conv(tag_def.id, func_name);
            registry::apply_print_conv(resolved_name, value)
        }
        _ => value.clone(),
    }
}
```

### Phase 2: Expression Pattern Matching

```rust
fn handle_expression_print_conv(expr: &str, value: &TagValue) -> TagValue {
    // Common sprintf patterns
    if expr.contains("sprintf(\"%.1f mm\"") || expr.contains("sprintf('%.1f mm'") {
        registry::apply_print_conv("focallength_print_conv", value)
    } else if expr.contains("sprintf(\"%.1f\"") {
        // Simple decimal formatting
        if let Some(v) = value.as_f64() {
            TagValue::String(format!("{:.1}", v))
        } else {
            value.clone()
        }
    } else {
        // Unknown pattern - return raw value
        value.clone()
    }
}
```

### Phase 3: Manual PrintConv Mapping

```rust
fn resolve_manual_print_conv(tag_id: u32, func_name: &str) -> &'static str {
    match (tag_id, func_name) {
        // Core EXIF tags
        (33437, "complex_expression_printconv") => "fnumber_print_conv",
        (33434, "complex_expression_printconv") => "exposuretime_print_conv",
        (37377, "complex_expression_printconv") => "exposuretime_print_conv", // ShutterSpeedValue
        (37378, "complex_expression_printconv") => "fnumber_print_conv",     // ApertureValue
        (37381, "complex_expression_printconv") => "fnumber_print_conv",     // MaxApertureValue
        
        // Default: use the function name as-is
        _ => func_name,
    }
}
```

### Phase 4: Extend to Other Modules

Apply same pattern to GPS and other modules:

```rust
// In GPS section of tags.rs
if result == value && !warnings.is_empty() {
    if let Some(tag_def) = gps_tag_kit::GPS_PM_TAG_KITS.get(&(tag_id as u32)) {
        result = apply_manual_print_conv_fallback(tag_def, &value);
    }
}
```

## Success Criteria

### Primary Goals

1. **Core EXIF Tags Display Correctly**:
   ```bash
   # After implementation:
   cargo run -- test.jpg | grep -E "FNumber|FocalLength|ExposureTime"
   
   # Should show:
   "EXIF:FNumber": 3.9
   "EXIF:FocalLength": "17.5 mm"
   "EXIF:ExposureTime": "1/80"
   ```

2. **No Codegen Changes Required**:
   - Solution works with existing generated code
   - New PrintConv functions can be added without `make codegen`

3. **Compatibility Tests Pass**:
   ```bash
   make compat-test | grep "EXIF:"
   # Failures should drop significantly
   ```

### Quality Gates

- **Block P12, P13*, P17a** until PrintConv pipeline fixed
- **Compatibility threshold**: <10 EXIF-related failures
- **Performance**: No measurable slowdown from fallback mechanism

## Post-Completion Tasks (Research Required)

### 1. Audit Current PrintConv Usage

**Scope**: Unknown - requires systematic search

```bash
# Find all apply_print_conv calls
grep -r "apply_print_conv" src/

# Find direct tag kit usage
grep -r "tag_kit::apply_print_conv" src/

# Find modules with custom PrintConv handling
```

### 2. Identify Cleanup Opportunities

**Research Questions**:
- Which modules use tag kit directly vs manual registry?
- Are there modules with their own PrintConv implementations?
- Can we unify all PrintConv through the fallback mechanism?

### 3. Document Unified Approach

Create `docs/guides/PRINTCONV-GUIDE.md` covering:
- How to add new PrintConv functions
- When to use Expression vs Manual
- Tag ID mapping conventions
- Testing PrintConv implementations

### 4. Long-term Architecture Improvements

**Consider for future**:
- Should we generate the fallback mappings from ExifTool?
- Can we make Expression evaluation more sophisticated?
- Would a trait-based system be more idiomatic?

## Gotchas & Tribal Knowledge

### Critical Understanding

1. **DO NOT EDIT GENERATED FILES**:
   - Everything in `src/generated/` is regenerated by `make codegen`
   - Any manual edits will be lost
   - This is why we need the runtime fallback approach

2. **The Fallback Is Already Called**:
   - `src/exif/tags.rs` line 300 already calls the fallback
   - We just need to implement the missing function
   - This pattern exists in GPS and other modules too

3. **"complex_expression_printconv" Is A Placeholder**:
   - ExifTool uses complex Perl expressions we can't easily port
   - Tag kit extracts this as a Manual type with placeholder name
   - We map tag IDs to actual implementations

4. **Expression Patterns Are Simple**:
   - Most are just `sprintf` with format strings
   - We pattern match common cases
   - Unknown patterns fall back to raw values

### Development Tips

1. **Test With Real Images**:
   ```bash
   cargo run --bin compare-with-exiftool test.jpg EXIF:
   ```

2. **Check Tag Kit Definitions**:
   ```bash
   grep -A5 "FNumber" src/generated/Exif_pm/tag_kit/exif_specific.rs
   ```

3. **Verify Manual Functions**:
   ```bash
   grep "fnumber_print_conv" src/implementations/
   ```

## Future Considerations

### Potential Enhancements

1. **Expression Evaluator**:
   - Implement basic Perl expression evaluation
   - Handle more complex sprintf patterns
   - Support mathematical expressions

2. **Automatic Mapping Generation**:
   - Extract tag ID → function mappings from ExifTool
   - Generate fallback mappings during codegen
   - Reduce manual maintenance

3. **Performance Optimization**:
   - Cache tag kit lookups
   - Pre-compile expression patterns
   - Profile hot paths

### Architecture Questions

1. Should PrintConv be in tag kit or registry?
2. Is runtime dispatch the best approach?
3. How do we handle module-specific PrintConv?

These questions require further research and discussion with the team.  

## Core Implementation Files

  1. src/exif/tags.rs - Where the fallback function needs to be implemented (line 300)
  2. src/registry.rs - The manual PrintConv registry system
  3. src/implementations/print_conv.rs - Existing PrintConv implementations
  4. src/implementations/mod.rs - Where PrintConv functions are registered

  Generated Code (To Understand the Problem)

  5. src/generated/Exif_pm/tag_kit/mod.rs - Contains TODO placeholders (lines 977-992)
  6. src/generated/GPS_pm/tag_kit/mod.rs - GPS module with similar structure
  7. src/generated/Canon_pm/tag_kit/mod.rs - Manufacturer module example
  8. src/generated/Exif_pm/tag_kit/exif_specific.rs - Tag definitions with PrintConv types

  Code Generation System

  9. codegen/src/generators/tag_kit_modular.rs - Generates the TODO placeholders
  10. codegen/src/schemas/tag_kit.rs - Tag kit data structures
  11. codegen/config/Exif_pm/tag_kit.json - Configuration for EXIF tag extraction

  Architecture Documentation

  12. docs/CODEGEN.md - Explains codegen constraints and architecture
  13. docs/TRUST-EXIFTOOL.md - Core principle affecting implementation
  14. docs/ARCHITECTURE.md - Overall system design
  15. docs/design/PRINTCONV-DESIGN-DECISIONS.md - Why PrintConv returns TagValue
  16. docs/design/API-DESIGN.md - TagEntry structure and design

  Prior Work Documentation

  17. docs/done/20250702-PRINTCONV-DESIGN.md - Original PrintConv problem analysis
  18. docs/done/MILESTONE-8b-TagEnry-and-ValueConf.md - TagEntry API implementation
  19. docs/done/20250723-tag-kit-migration-and-retrofit.md - Tag kit system migration
  20. docs/done/20250722-tag-kit-codegen.md - Tag kit implementation details

  Related TPPs and Milestones

  21. docs/todo/P10a-exif-required-tags.md - Parent TPP being blocked
  22. docs/compatibility-failure-analysis.md - Shows the actual failures
  23. docs/MILESTONES.md - Overall project milestones and dependencies

  Test Files (To Validate Solution)

  24. tests/compatibility.rs - Compatibility test framework
  25. src/bin/compare-with-exiftool.rs - Tool to compare output
  26. test-images/ - Real images to test against

  Other Relevant Files

  27. src/types/mod.rs - TagValue and TagEntry types
  28. src/expressions/mod.rs - ExpressionEvaluator (for future Expression support)
  29. Makefile - make compat-test and make precommit targets
  30. scripts/compare-with-exiftool.sh - Shell script for quick comparisons

  Files to Search for Usage Patterns

  31. Any file containing apply_print_conv calls
  32. Any file containing tag_kit::apply_print_conv calls
  33. Any file containing PrintConvType::Expression or PrintConvType::Manual
  34. Any file in src/processor_registry/ that might have custom PrintConv handling

  These files would provide:
  - Context for understanding the current architecture
  - Constraints that affect the implementation
  - Validation that the proposed solution is correct
  - Examples of similar patterns already in use
  - Test cases to verify the fix works