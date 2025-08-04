# Naming Conventions: Design Decisions and Rationale

## Overview

This document explains the naming convention decisions in exif-oxide, particularly focusing on cases where the "obvious" choice might seem wrong but is actually correct for data integrity reasons.

## snake_case Implementation: Information Preservation vs Aesthetics

### The Problem

When converting ExifTool symbol names to Rust identifiers, we need to convert from various Perl naming styles to snake_case. The "obvious" implementation would be to make acronyms look nice:

```rust
// "Smart" implementation (WRONG)
"canonModelID" → "canon_model_id"    // Looks nice
"CTMD"         → "ctmd"              // Looks nice  
"LensID"       → "lens_id"           // Looks nice
```

However, this approach is **information-losing** and can cause **silent data corruption**.

### The Correctness Issue

ExifTool is a **25-year-old Perl codebase** with potential historical inconsistencies in naming patterns. Consider this scenario:

**Hypothetical ExifTool symbols:**
```perl
%canonModelID   # Historical symbol with caps "ID"
%canonModelId   # Hypothetical newer symbol with mixed case
```

**"Smart" implementation results (DANGEROUS):**
```rust
canonModelID → canon_model_id   
canonModelId → canon_model_id   // COLLISION! Same identifier
```

**Current implementation results (SAFE):**
```rust
canonModelID → canon_model_i_d
canonModelId → canon_model_id   // No collision - distinguishable
```

### Real-World Evidence

From ExifTool's `Canon.pm` source code:
```perl
use vars qw($VERSION %canonModelID %canonLensTypes);
```

The actual symbols use patterns like `%canonModelID` with uppercase acronyms, validating our concern about preserving the exact character-level information.

### The Tradeoff

**Current Implementation (Information-Preserving):**
- ✅ **Correctness**: No data loss, no identifier collisions
- ✅ **Trust ExifTool**: Preserves every bit of naming information
- ❌ **Aesthetics**: Generates `lens_i_d` instead of `lens_id`

**"Smart" Implementation (Information-Losing):**
- ✅ **Aesthetics**: Generates nice-looking `lens_id`  
- ❌ **Correctness**: Potential silent data corruption
- ❌ **Trust ExifTool**: Loses original naming distinctions

### Design Decision: Correctness Over Aesthetics

**We choose information preservation over aesthetics.**

In a battle-tested codebase like ExifTool that handles real-world camera quirks, seemingly "obviously better" improvements can introduce subtle correctness bugs. The aesthetic issue (`lens_i_d` vs `lens_id`) is a small price to pay for **data integrity**.

### Implementation Location

The information-preserving snake_case implementation is currently in:
- `codegen/src/strategies/simple_table.rs` - Strategy system
- `codegen/src/generators/lookup_tables/path_utils.rs` - Path utilities  
- Multiple other locations throughout the codebase

All implementations follow the same principle: **every uppercase letter triggers a word boundary** to preserve maximum information.

### Code Example

```rust
/// Convert CamelCase/camelCase to snake_case for Rust naming
/// IMPORTANT: Information-preserving - each uppercase = word boundary
/// This prevents identifier collisions in ExifTool symbol translations
fn snake_case(&self, name: &str) -> String {
    let mut result = String::new();
    let mut chars = name.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch.is_uppercase() && !result.is_empty() {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap_or(ch));
    }
    
    result
}
```

## Trust ExifTool Principle Applied

This naming convention decision exemplifies the **Trust ExifTool** principle:

1. **Don't "improve" ExifTool** - Even naming patterns might have historical significance
2. **Preserve all information** - Don't make assumptions about what's "obviously" redundant
3. **Data integrity first** - Correctness trumps aesthetics
4. **Battle-tested wisdom** - 25 years of real-world usage trumps theoretical improvements

## Future Considerations

If we ever encounter actual ExifTool symbols that would collide under the current system, we would need to investigate ExifTool's own collision handling mechanisms and replicate them exactly.

**Do not "fix" the snake_case implementation** without first:
1. Comprehensive analysis of all ExifTool symbols across all modules
2. Verification that no naming collisions exist in the source
3. Understanding of ExifTool's own symbol resolution mechanisms
4. Extensive testing with the complete ExifTool symbol table

## Related Design Decisions

This principle applies to other naming and data preservation decisions throughout the codebase:

- Tag ID preservation (don't "clean up" hex formatting)
- Value preservation (don't "improve" numeric representations)  
- Order preservation (don't "optimize" data structure ordering)
- Comment preservation (don't "clean up" ExifTool documentation text)

**When in doubt, preserve more information rather than less.**