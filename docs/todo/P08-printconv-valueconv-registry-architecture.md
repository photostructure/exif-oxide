# Technical Project Plan: PrintConv/ValueConv Registry Architecture

## Project Overview

**Goal**: Fix the broken PrintConv pipeline by implementing a codegen-time registry that maps Perl expressions to Rust function names, eliminating runtime overhead and circular dependencies.

**Problem**: The tag kit system correctly extracts PrintConv/ValueConv definitions from ExifTool, but generated code contains TODO placeholders that prevent proper formatting. This causes core camera settings to display as raw values like `[39, 10]` instead of human-readable formats like `3.9`.

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

## Remaining Tasks

### Phase 1: Create Codegen Registry (High Confidence)

1. **Create `codegen/src/conv_registry.rs`**:
```rust
use std::collections::HashMap;
use once_cell::sync::Lazy;

// Registry maps Perl expressions to (module_path, function_name)
static PRINTCONV_REGISTRY: Lazy<HashMap<&'static str, (&'static str, &'static str)>> = Lazy::new(|| {
    let mut m = HashMap::new();
    
    // Common sprintf patterns
    m.insert("sprintf(\"%.1f mm\",$val)", ("crate::implementations::print_conv", "focallength_print_conv"));
    m.insert("sprintf(\"%.1f\",$val)", ("crate::implementations::print_conv", "decimal_1_print_conv"));
    m.insert("sprintf(\"%.2f\",$val)", ("crate::implementations::print_conv", "decimal_2_print_conv"));
    m.insert("sprintf(\"%+d\",$val)", ("crate::implementations::print_conv", "signed_int_print_conv"));
    m.insert("sprintf(\"%.3f mm\",$val)", ("crate::implementations::print_conv", "focal_length_3_decimals_print_conv"));
    
    // Conditional expressions
    m.insert("$val =~ /^(inf|undef)$/ ? $val : \"$val m\"", ("crate::implementations::print_conv", "gpsaltitude_print_conv"));
    
    // Module-scoped functions
    m.insert("GPS.pm::ConvertTimeStamp($val)", ("crate::implementations::value_conv", "gpstimestamp_value_conv"));
    m.insert("ID3.pm::ConvertTimeStamp($val)", ("crate::implementations::value_conv", "id3_timestamp_value_conv"));
    
    // Complex expressions (placeholder names from tag_kit.pl)
    m.insert("complex_expression_printconv", ("crate::implementations::print_conv", "complex_expression_print_conv"));
    
    m
});

static VALUECONV_REGISTRY: Lazy<HashMap<&'static str, (&'static str, &'static str)>> = Lazy::new(|| {
    let mut m = HashMap::new();
    
    // GPS conversions
    m.insert("Image::ExifTool::GPS::ToDegrees($val)", ("crate::implementations::value_conv", "gps_coordinate_value_conv"));
    m.insert("Image::ExifTool::GPS::ConvertTimeStamp($val)", ("crate::implementations::value_conv", "gpstimestamp_value_conv"));
    
    // APEX conversions
    m.insert("IsFloat($val) && abs($val)<100 ? 2**(-$val) : 0", ("crate::implementations::value_conv", "apex_shutter_speed_value_conv"));
    m.insert("2 ** ($val / 2)", ("crate::implementations::value_conv", "apex_aperture_value_conv"));
    
    m
});

/// Look up PrintConv implementation by Perl expression
/// Tries module-scoped lookup first, then unscoped
pub fn lookup_printconv(expr: &str, module: &str) -> Option<(&'static str, &'static str)> {
    // Normalize module name (GPS_pm -> GPS.pm)
    let normalized_module = module.replace("_pm", ".pm");
    
    // Try module-scoped first
    let scoped_key = format!("{}::{}", normalized_module, expr);
    for (key, value) in PRINTCONV_REGISTRY.iter() {
        if key == &scoped_key {
            return Some(*value);
        }
    }
    
    // Fall back to exact match
    PRINTCONV_REGISTRY.get(expr).copied()
}

/// Look up ValueConv implementation by Perl expression
pub fn lookup_valueconv(expr: &str, module: &str) -> Option<(&'static str, &'static str)> {
    // Same module-scoped logic as PrintConv
    let normalized_module = module.replace("_pm", ".pm");
    let scoped_key = format!("{}::{}", normalized_module, expr);
    
    for (key, value) in VALUECONV_REGISTRY.iter() {
        if key == &scoped_key {
            return Some(*value);
        }
    }
    
    VALUECONV_REGISTRY.get(expr).copied()
}

/// Normalize expression for consistent lookup
/// Handles whitespace normalization and other variations
pub fn normalize_expression(expr: &str) -> String {
    // Collapse whitespace
    expr.split_whitespace().collect::<Vec<_>>().join(" ")
}
```

2. **Add to `codegen/src/lib.rs`**:
```rust
pub mod conv_registry;
```

3. **Modify `tag_kit_modular.rs` to use registry**:
```rust
// Add import
use crate::conv_registry::{lookup_printconv, lookup_valueconv};

// In generate_print_conv_match function (around line 400)
match print_conv_type {
    PrintConvType::Expression(expr) => {
        if let Some((module_path, func_name)) = lookup_printconv(expr, module_name) {
            // Generate direct function call
            writeln!(output, "            {}::{}(value)", module_path, func_name)?;
        } else {
            // Track missing implementation
            writeln!(output, "            crate::implementations::missing::missing_print_conv({:?}, {:?}, value)", tag_id, expr)?;
        }
    }
    PrintConvType::Manual(func_name) => {
        if let Some((module_path, func_name)) = lookup_printconv(func_name, module_name) {
            writeln!(output, "            {}::{}(value)", module_path, func_name)?;
        } else {
            writeln!(output, "            crate::implementations::missing::missing_print_conv({:?}, {:?}, value)", tag_id, func_name)?;
        }
    }
    // ... handle other types
}
```

### Phase 2: Implement Missing Tracking (High Confidence)

1. **Create `src/implementations/missing.rs`**:
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
1. ✅ Core EXIF tags display correctly (FNumber, ExposureTime, FocalLength)
2. ✅ Generated code compiles without manual intervention
3. ✅ `make precommit` passes
4. ✅ Compatibility test failures reduced by >50%

### Quality Gates
- [ ] No circular dependencies between generated and manual code
- [ ] Missing implementations tracked and shown with --show-missing
- [ ] All existing PrintConv/ValueConv implementations still work
- [ ] Generated code is readable with clear function calls
- [ ] Performance: No measurable slowdown vs runtime registry

### Completion Checklist
- [ ] Phase 1: Codegen registry implemented
- [ ] Phase 2: Missing tracking implemented
- [ ] Phase 3: Existing code retrofitted
- [ ] Phase 4: BITMASK research TPP created
- [ ] Phase 5: Documentation complete
- [ ] All tests passing
- [ ] Code review completed

## Gotchas & Tribal Knowledge

### Critical Understanding

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

**⚠️ Remember**: Update this document as you implement! Don't wait until the end.