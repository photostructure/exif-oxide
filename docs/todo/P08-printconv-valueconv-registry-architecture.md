# Technical Project Plan: PrintConv/ValueConv Registry Architecture

## Project Overview

**Goal**: Fix the broken PrintConv pipeline by implementing a codegen-time registry that maps Perl expressions to Rust function names, eliminating runtime overhead and circular dependencies.

**Problem**: The tag kit system correctly extracts PrintConv/ValueConv definitions from ExifTool, but generated code contains TODO placeholders that prevent proper formatting. This causes core camera settings to display as raw values like `[39, 10]` instead of human-readable formats like `3.9`.

## MANDATORY READING

These are relevant, mandatory, prerequisite reading for every task:

- [@CLAUDE.md](CLAUDE.md)
- [@docs/TRUST-EXIFTOOL.md](docs/TRUST-EXIFTOOL.md).

## DO NOT BLINDLY FOLLOW THIS PLAN

Building the wrong thing (because you made an assumption or misunderstood something) is **much** more expensive than asking for guidance or clarity.

The authors tried their best, but also assume there will be aspects of this plan that may be odd, confusing, or unintuitive to you. Communication is hard!

**FIRSTLY**, follow and study **all** referenced source and documentation. Ultrathink, analyze, and critique the given overall TPP and the current task breakdown.

If anything doesn't make sense, or if there are alternatives that may be more optimal, ask clarifying questions. We all want to drive to the best solution and are delighted to help clarify issues and discuss alternatives. DON'T BE SHY!

## KEEP THIS UPDATED

This TPP is a living document. **MAKE UPDATES AS YOU WORK**. Be concise -- avoid lengthy prose.

**What to Update:**

- 🔍 **Discoveries**: Add findings with links to source code/docs (in relevant sections)
- 🤔 **Decisions**: Document WHY you chose approach A over B (in "Work Completed")
- ⚠️ **Surprises**: Note unexpected behavior or assumptions that were wrong (in "Gotchas")
- ✅ **Progress**: Move completed items from "Remaining Tasks" to "Work Completed"
- 🚧 **Blockers**: Add new prerequisites or dependencies you discover

**When to Update:**

- After each research session (even if you found nothing - document that!)
- When you realize the original approach won't work
- When you discover critical context not in the original TPP
- Before context switching to another task

The Engineers of Tomorrow are interested in your discoveries, not just your final code!


## Background & Context

The current system has two disconnected parts:
- **Tag Kit** (generated): Knows PrintConv types but has TODO placeholders
- **Registry** (runtime): Has implementations but requires runtime lookup

This TPP implements "Plan J" - a codegen-time registry that generates direct function calls, eliminating runtime overhead and the missing `apply_manual_print_conv_fallback` function.

**Related docs**:
- [P09-printconv-pipeline-fix.md](P09-printconv-pipeline-fix.md) - Original problem analysis
- [../design/PRINTCONV-DESIGN-DECISIONS.md](../design/PRINTCONV-DESIGN-DECISIONS.md) - Why PrintConv returns TagValue
- [../../third-party/exiftool/doc/concepts/PRINT_CONV.md](../../third-party/exiftool/doc/concepts/PRINT_CONV.md) - ExifTool's PrintConv system

## Technical Foundation

### Key Codebases
- `codegen/extractors/tag_kit.pl` - Extracts PrintConv/ValueConv from ExifTool
- `codegen/src/generators/tag_kit_modular.rs` - Generates tag kit modules
- `src/generated/*/tag_kit/mod.rs` - Generated code with TODO placeholders
- `src/implementations/print_conv.rs` - Manual PrintConv implementations
- `src/implementations/value_conv.rs` - Manual ValueConv implementations
- `src/exif/tags.rs` - Runtime conversion application

### Current Architecture Issues
1. Generated code cannot import from `src/` (circular dependency)
2. Runtime pattern matching for every conversion
3. Hardcoded mappings in `tags.rs` for ValueConv
4. Missing fallback function that's already being called

## Work Completed

### Analysis Phase
- Identified ~1000+ sprintf patterns in ExifTool
- Found module-scoped function conflicts (e.g., `ConvertTimeStamp` in ID3.pm and GPS.pm)
- Discovered BITMASK and multi-line code block patterns
- Confirmed same architecture issues affect both PrintConv and ValueConv

### Design Decisions
1. **No new crate needed** - Codegen only needs a registry returning function names
2. **Module-scoped lookups** - Try `Canon.pm::funcname()` before `funcname()`
3. **Perl expression as key** - Map exact Perl strings to Rust functions
4. **Direct function calls** - Generate imports and calls at codegen time

### Implementation Completed (2025-07-26)

#### Phase 1: Create Codegen Registry ✅
- Created `codegen/src/conv_registry.rs` with PRINTCONV_REGISTRY and VALUECONV_REGISTRY
- Implemented lookup functions with module-scoped support
- Added initial mappings for common sprintf patterns and ExifTool functions
- Added `PrintFraction` function to registry and implemented in Rust

#### Phase 2: Codegen Integration ✅
- Modified `tag_kit_modular.rs` to use registry for Expression and Manual types
- Generated direct function calls in `apply_print_conv` functions
- Fixed duplicate match arms by using HashMap deduplication
- All generated code now compiles successfully

#### Phase 3: Implementation Status ✅
- Registry generates direct function calls like `crate::implementations::print_conv::print_fraction(value)`
- No runtime lookup overhead - all expressions resolved at compile time
- Missing implementations fallback to generic handling
- Code successfully compiles and runs

### Key Achievements
1. **Zero runtime overhead** - All PrintConv expressions resolved to direct function calls at codegen time
2. **Type safety** - Compilation fails if referenced functions don't exist
3. **No circular dependencies** - Generated code calls into implementations, not vice versa
4. **Maintainable** - New conversions added to registry without touching generated code

### Implementation Notes
- The final implementation generates direct function calls in match statements rather than function pointers
- Tag definitions retain expression metadata for documentation/debugging purposes
- The `apply_print_conv` functions contain the actual dispatch logic with direct calls
- HashMap deduplication prevents duplicate match arms when tags have multiple definitions

### Key Discoveries During Implementation (2025-07-26)

1. **Engineer Misunderstanding**: Initial implementation generated runtime match statements on expressions - exactly what we were trying to avoid. Root cause: engineer didn't grasp compile-time vs runtime resolution.

2. **Duplicate Match Arms**: Generated code had multiple entries for same tag_id causing compiler warnings. Solution: Use HashMap for deduplication during codegen.

3. **Function Organization**: Created `src/implementations/generic.rs` for shared PrintConv logic to reduce code duplication across 40+ generated modules.

4. **Actual Generated Code**: Successfully generates direct calls like:
   ```rust
   6 => crate::implementations::print_conv::print_fraction(value),
   ```

5. **PrintFraction Implementation**: Added as proof of concept - converts rationals to fractional strings with sign (e.g., "+1/2", "-2/3")

### Files Modified During Implementation

1. **Created**:
   - `codegen/src/conv_registry.rs` - The compile-time registry mapping expressions to functions
   - `src/implementations/generic.rs` - Shared PrintConv handling to reduce duplication
   - `src/implementations/print_conv.rs::print_fraction()` - Example implementation

2. **Modified**:
   - `codegen/src/generators/tag_kit_modular.rs` - Added registry lookup and deduplication
   - `codegen/src/lib.rs` - Added conv_registry module
   - All `src/generated/*/tag_kit/mod.rs` - Now contain direct function calls

3. **Key Code Snippets**:
   ```rust
   // codegen/src/generators/tag_kit_modular.rs - Deduplication fix
   let mut tag_convs_map: HashMap<u32, (String, String)> = HashMap::new();
   // ... collect without duplicates
   
   // Generated output in src/generated/Canon_pm/tag_kit/mod.rs
   match tag_id {
       6 => crate::implementations::print_conv::print_fraction(value),
       // ... direct calls, no strings!
   }
   ```

## Remaining Tasks

**🎯 Next Engineer Focus**: The core registry architecture is complete and working. Focus on:
1. Implementing missing conversion tracking for better debugging
2. Adding more PrintConv/ValueConv functions to the registry
3. Removing hardcoded conversions from src/exif/tags.rs

### Phase 1: Implement Missing Tracking

**Acceptance Criteria**: --show-missing flag reports unimplemented PrintConv/ValueConv expressions

**✅ Correct Output:**
```bash
$ cargo run -- --show-missing test.jpg
# After processing:
Missing PrintConv implementations:
  - sprintf("ISO %d", $val) [used by tags: 0x8827, 0x8832]
  - Image::ExifTool::Canon::CanonEv($val) [used by tag: 0x1034]
```

**❌ Common Mistake:**
- Recording every call (causes duplicates)
- Not grouping by expression
- Missing the tag context

**Implementation Notes**:
1. Create `src/implementations/missing.rs`:
```rust
//! Track missing PrintConv/ValueConv implementations for --show-missing

use crate::types::TagValue;
use std::sync::Mutex;
use once_cell::sync::Lazy;

#[derive(Debug, Clone)]
pub struct MissingConversion {
    pub tag_id: u32,
    pub expression: String,
    pub conv_type: ConversionType,
}

#[derive(Debug, Clone)]
pub enum ConversionType {
    PrintConv,
    ValueConv,
}

static MISSING_CONVERSIONS: Lazy<Mutex<Vec<MissingConversion>>> = Lazy::new(|| {
    Mutex::new(Vec::new())
});

/// Record a missing PrintConv implementation
pub fn missing_print_conv(tag_id: u32, expr: &str, value: &TagValue) -> TagValue {
    let mut missing = MISSING_CONVERSIONS.lock().unwrap();
    
    // Only record each unique expression once
    let already_recorded = missing.iter().any(|m| {
        m.expression == expr && matches!(m.conv_type, ConversionType::PrintConv)
    });
    
    if !already_recorded {
        missing.push(MissingConversion {
            tag_id,
            expression: expr.to_string(),
            conv_type: ConversionType::PrintConv,
        });
    }
    
    value.clone()
}

/// Record a missing ValueConv implementation
pub fn missing_value_conv(tag_id: u32, expr: &str, value: &TagValue) -> TagValue {
    let mut missing = MISSING_CONVERSIONS.lock().unwrap();
    
    let already_recorded = missing.iter().any(|m| {
        m.expression == expr && matches!(m.conv_type, ConversionType::ValueConv)
    });
    
    if !already_recorded {
        missing.push(MissingConversion {
            tag_id,
            expression: expr.to_string(),
            conv_type: ConversionType::ValueConv,
        });
    }
    
    value.clone()
}

/// Get all missing conversions for --show-missing
pub fn get_missing_conversions() -> Vec<MissingConversion> {
    MISSING_CONVERSIONS.lock().unwrap().clone()
}

/// Clear missing conversions (useful for testing)
pub fn clear_missing_conversions() {
    MISSING_CONVERSIONS.lock().unwrap().clear();
}
```

2. **Add to `src/implementations/mod.rs`**:
```rust
pub mod missing;
```

3. **Update `src/formats/mod.rs` for --show-missing**:
```rust
// In extract_metadata function, after tag extraction
if show_missing {
    let missing_convs = crate::implementations::missing::get_missing_conversions();
    
    let mut missing_strs = Vec::new();
    for miss in missing_convs {
        missing_strs.push(format!(
            "{:?} for tag 0x{:04x}: {}",
            miss.conv_type,
            miss.tag_id,
            miss.expression
        ));
    }
    
    if !missing_strs.is_empty() {
        missing_implementations = Some(missing_strs);
    }
}
```

### Phase 3: Retrofit Existing Code (High Confidence)

1. **Remove hardcoded ValueConv mappings from `tags.rs`** (lines 116-157):
```rust
// DELETE the manual ValueConv matching - it will be in generated code now
```

2. **Remove the fallback call from `tags.rs`** (lines 298-302):
```rust
// DELETE this entire block - no longer needed
// if print == value && !warnings.is_empty() {
//     if let Some(tag_def) = tag_kit::EXIF_PM_TAG_KITS.get(&(tag_id as u32)) {
//         print = apply_manual_print_conv_fallback(tag_def, &value);
//     }
// }
```

3. **Update tag_kit.pl to preserve complex expressions**:
```perl
# Around line 225, modify complex expression handling
if (!ref $print_conv) {
    # String expression
    if (is_simple_expression($print_conv)) {
        return ('Expression', $print_conv);
    }
    # Keep the original expression for complex cases
    return ('Expression', $print_conv);  # Was: ('Manual', 'complex_expression_printconv')
}
```

4. **Run `make codegen`** to regenerate all tag kits with direct function calls

### Phase 4: BITMASK Research (Requires Analysis)

**Create separate TPP**: `P15c-bitmask-printconv-implementation.md`

Research tasks:
1. Count BITMASK occurrences: `rg "BITMASK" third-party/exiftool/lib/Image/ExifTool`
2. Analyze patterns - are they always simple bit mappings?
3. Design generic bitmask handler or generate specific implementations?

### Phase 5: Documentation (Post-Implementation)

1. **Create `docs/guides/PRINTCONV-VALUECONV-GUIDE.md`**:
```markdown
# PrintConv/ValueConv Implementation Guide

## Overview
PrintConv and ValueConv functions are resolved at codegen time using a registry...

## Adding New Conversions
1. Implement function in `src/implementations/print_conv.rs` or `value_conv.rs`
2. Add registry entry in `codegen/src/conv_registry.rs`
3. Run `make codegen` to update generated code

## Module-Scoped Functions
When ExifTool modules have same-named functions...

## Testing
Use --show-missing to find unimplemented conversions...
```

2. **Update `CODEGEN.md`** section on PrintConv/ValueConv

## Prerequisites

None - can begin immediately. The missing fallback function is already being called, so fixing it will immediately improve output.

## Testing Strategy

### Unit Tests
```rust
// In codegen/src/conv_registry.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_module_scoped_lookup() {
        let result = lookup_printconv("ConvertTimeStamp", "GPS_pm");
        assert_eq!(result, Some(("crate::implementations::value_conv", "gpstimestamp_value_conv")));
    }
    
    #[test]
    fn test_expression_normalization() {
        assert_eq!(
            normalize_expression("sprintf( \"%.1f mm\" , $val )"),
            "sprintf(\"%.1f mm\",$val)"
        );
    }
}
```

### Integration Tests
```bash
# Before fix - shows raw rationals
cargo run -- test-images/canon/Canon_40D.jpg | grep FNumber
# "EXIF:FNumber": [4, 1]

# After fix - shows formatted value
cargo run -- test-images/canon/Canon_40D.jpg | grep FNumber  
# "EXIF:FNumber": 4.0

# Test --show-missing
cargo run -- --show-missing test-images/canon/Canon_40D.jpg 2>&1 | grep Missing
```

### Compatibility Testing
```bash
make compat-test
# Should see significant reduction in failures
```

## Success Criteria & Quality Gates

### Primary Success Criteria
1. ✅ Core EXIF tags display correctly (FNumber, ExposureTime, FocalLength) - **ACHIEVED**
2. ✅ Generated code compiles without manual intervention - **ACHIEVED**
3. ✅ `make precommit` passes - **ACHIEVED**
4. ⏳ Compatibility test failures reduced by >50% - **IN PROGRESS** (need more conversions)

### Quality Gates
- [x] No circular dependencies between generated and manual code
- [ ] Missing implementations tracked and shown with --show-missing
- [x] All existing PrintConv/ValueConv implementations still work
- [x] Generated code is readable with clear function calls
- [x] Performance: No measurable slowdown vs runtime registry

### Completion Checklist
- [x] Phase 1: Codegen registry implemented ✅ (2025-07-26)
- [ ] Phase 2: Missing tracking implemented
- [ ] Phase 3: Existing code retrofitted
- [ ] Phase 4: BITMASK research TPP created
- [ ] Phase 5: Documentation complete
- [x] All tests passing
- [ ] Code review completed

## Gotchas & Tribal Knowledge

### Critical Understanding

**⚠️ MOST IMPORTANT**: This is about COMPILE-TIME resolution, not RUNTIME!
- The registry is used during `make codegen` to generate direct function calls
- Generated code should NOT contain expression strings in match arms
- If you see `PrintConvType::Expression("...")` in a match arm, that's WRONG

1. **Expression Variations**: ExifTool may use different quote styles or whitespace:
   - `sprintf("%.1f",$val)` vs `sprintf('%.1f',$val)`
   - Consider normalizing during both extraction and lookup

2. **Module Name Format**: 
   - ExifTool: `GPS.pm`, `Canon.pm`
   - Our code: `GPS_pm`, `Canon_pm`
   - Registry must handle both formats

3. **Tag Kit Regeneration**: After changing registry, must run `make codegen` for all modules

4. **Complex Expression Keys**: For multi-line Perl blocks, consider:
   ```rust
   // If expression > 100 chars, use truncated + hash
   let key = if expr.len() > 100 {
       format!("{}...{:x}", &expr[..80], md5::compute(expr))
   } else {
       expr.to_string()
   };
   ```

### Why This Architecture?

1. **Compile-time resolution**: No runtime overhead for lookups
2. **Type safety**: Codegen fails if functions don't exist
3. **Maintainability**: Central registry for all conversions
4. **Incremental**: Can add conversions without touching generated code

### Common Pitfalls

1. **Don't edit generated files** - Changes lost on next `make codegen`
2. **Test with real images** - Synthetic test data may not trigger all paths
3. **Check both value and print fields** - Some tags only use ValueConv
4. **Module scoping matters** - Wrong function = wrong output

### Future Improvements

1. **Auto-generate registry** from compatibility test results
2. **Expression evaluator** for simple sprintf patterns
3. **Lint rule** to catch manual registry usage in new code
4. **Performance profiling** to verify no regression

---
