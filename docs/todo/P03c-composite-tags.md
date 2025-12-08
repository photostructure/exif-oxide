# P03c: Composite Tags via PPI Code Generation

**Prerequisites**: [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md), [ARCHITECTURE.md](../ARCHITECTURE.md), [CODEGEN.md](../CODEGEN.md), [PRINTCONV-VALUECONV-GUIDE.md](../guides/PRINTCONV-VALUECONV-GUIDE.md), [COMPOSITE_TAGS.md](../../third-party/exiftool/doc/concepts/COMPOSITE_TAGS.md)

---

## Part 1: Define Success

**Problem**: Composite tags like Aperture, ShutterSpeed, Megapixels, GPSLatitude are "supported" in the codegen schema but never calculated - they return empty values.

**Why it matters**: Composite tags are the most commonly used metadata tags (ImageSize, GPS coordinates, Aperture). Without them, exif-oxide is incomplete for real-world use.

**Solution**: Extend the PPI pipeline to translate composite tag ValueConv/PrintConv expressions to Rust functions at codegen time, just like regular tags.

**Success test**:

```bash
cargo run --bin compare-with-exiftool -- third-party/exiftool/t/images/Canon.jpg 2>/dev/null | grep -c "Composite:"
# Should show 10+ matching composite tags, 0 missing
```

**Key constraint**: Composite expressions use `$val[n]` array access (dependencies), not single `$val` (tag value). The PPI **generator** (not parser) must emit different code for this context.

---

## Part 2: Share Your Expertise

### A. How ExifTool Composite Tags Work

Composite tags are calculated from other tags. ExifTool defines them in `%Composite` hashes with:

| Field       | Purpose                                        | Example                                               |
| ----------- | ---------------------------------------------- | ----------------------------------------------------- |
| `Require`   | Tags that MUST exist (indexed 0, 1, 2...)      | `{0 => 'GPS:GPSLatitude', 1 => 'GPS:GPSLatitudeRef'}` |
| `Desire`    | Tags that MAY exist (optional)                 | `{0 => 'FNumber', 1 => 'ApertureValue'}`              |
| `Inhibit`   | Tags that PREVENT this composite               | `{0 => 'Composite:LensID'}`                           |
| `ValueConv` | Expression using `@val`, `@prt`, `@raw` arrays | `'$val[1] =~ /^S/i ? -$val[0] : $val[0]'`             |
| `PrintConv` | Human-readable formatting                      | `'sprintf("%.1f", $val)'`                             |

**ExifTool source**: `lib/Image/ExifTool.pm:3929-4115` (BuildCompositeTags)

### B. The Three Arrays: `@val`, `@prt`, `@raw`

This is the **critical difference** from regular tags:

```perl
# Regular tag ValueConv (single value):
ValueConv => '$val + 4'              # $val is the tag's raw value

# Composite tag ValueConv (array of dependencies):
ValueConv => '$val[1] =~ /^S/i ? -$val[0] : $val[0]'
#            $val[1] is Require[1] (GPSLatitudeRef)
#            $val[0] is Require[0] (GPSLatitude)
```

ExifTool populates three parallel arrays (`lib/Image/ExifTool.pm:3553-3560`):

- `@raw` - Unconverted raw values from storage
- `@val` - Values after ValueConv applied
- `@prt` - Values after PrintConv applied (human-readable)

**Critical insight**: Some expressions reference `$prt[n]` to get the human-readable form of a dependency. Example:

```perl
# From Exif.pm:4695 - LightValue uses printed ISO string
ValueConv => 'Image::ExifTool::Exif::CalculateLV($val[0],$val[1],$prt[2])'
```

### C. Current Infrastructure Status

```bash
# Find existing composite infrastructure:
rg -l "composite|Composite" src/ --type rust

# Key files:
# src/composite_tags/mod.rs           - Entry point (stubs, waiting for codegen)
# src/composite_tags/orchestration.rs - Multi-pass resolution (commented out)
# src/composite_tags/dispatch.rs      - Router to implementations (commented out)
# src/composite_tags/implementations.rs - 35 manual implementations (dead code)
# codegen/src/strategies/composite_tag.rs - Extracts defs, stores Perl as strings
```

**What exists but doesn't work**:

- `CompositeTagStrategy` extracts definitions but stores ValueConv as raw Perl strings
- Orchestration code is ready but commented out waiting for `CompositeTagDef`
- 35 manual implementations exist but aren't wired up
- Generated `composite_tags.rs` files are empty (`HashMap::new()`)
- **`is_composite_table` flag never gets set** - patcher bug blocks detection

### D. Expression Analysis: 75% Simple, 25% Complex

Based on analysis of required composite tags:

| Category          | Tags                                                                                                           | Approach                                   |
| ----------------- | -------------------------------------------------------------------------------------------------------------- | ------------------------------------------ |
| **SIMPLE (75%)**  | Aperture, ShutterSpeed, ISO, Lens, GPSLatitude, GPSLongitude, GPSPosition, GPSDateTime, SubSecDateTimeOriginal | PPI auto-generation with composite context |
| **COMPLEX (25%)** | ImageSize (`$$self{TIFF_TYPE}`), Megapixels (`my @d` local array), LensID (`@raw` + complex algorithm)         | Keep existing manual implementations       |

### E. Why PPI Already Works (Almost)

**Key insight from investigation**: PPI **parses** `$val[n]` and `$prt[n]` correctly! The issue is only in the **generator**:

```bash
# PPI parsing works:
$ ./codegen/scripts/ppi_ast.pl '$prt[0]'
# Shows: PPI::Token::Symbol($prt) + PPI::Structure::Subscript([0])

# Generator emits wrong code for composites:
$ cargo run -p codegen --bin debug-ppi -- '$prt[0]'
# Current:  get_array_element(prt, 0)  # assumes prt is a TagValue with array
# Needed:   prts.get(0).cloned()       # access prts parameter slice
```

The fix is a **generator context switch**, not a parser change.

### F. The Patcher Bug

**Root cause**: `field_extractor.pl:177` checks `is_composite_table = ($symbol_name eq 'Composite' && $has_composite_tags)`, but `has_composite_tags` requires a package variable `__hasCompositeTags` that the patcher never creates.

**Fix**: When patcher sees `AddCompositeTags('Image::ExifTool::GPS')`, it must add:

```perl
$__hasCompositeTags = 1;
```

### G. Learned the Hard Way

1. **`$val` vs `$val[0]` ambiguity**: In composite context, `$val` alone refers to the current computed value (inside PrintConv), while `$val[0]` refers to the first dependency.

2. **Inhibit evaluation order**: If checking `Inhibit` against an unbuilt composite, must defer (ExifTool.pm:4034-4036).

3. **`$prt[]` requires two-phase**: Some ValueConv expressions use `$prt[n]`, meaning we need PrintConv'd values of dependencies, not just ValueConv'd.

4. **Module-specific composites**: Canon, Nikon, etc. define their own `%Composite` tables. The `AddCompositeTags()` function merges them into a global registry.

5. **Circular dependency handling**: ExifTool does one final pass ignoring Inhibit tags if stuck (ExifTool.pm:4103-4110).

---

## Part 3: Tasks

### Task 0: Fix Patcher Composite Detection

**Success**: `make codegen` logs "Processing composite symbol: GPS::Composite" (not skipped)

**Implementation**:

The patcher must detect `AddCompositeTags` calls and set a marker variable.

In `codegen/scripts/exiftool-patcher.pl`, add detection for:

```perl
Image::ExifTool::AddCompositeTags('Image::ExifTool::GPS');
```

When found, insert before the call:

```perl
our $__hasCompositeTags = 1;
```

**Verification**:

```bash
# After patching GPS.pm:
grep '__hasCompositeTags' third-party/exiftool/lib/Image/ExifTool/GPS.pm
# Should show: our $__hasCompositeTags = 1;

# Run codegen and check logs:
RUST_LOG=debug make codegen 2>&1 | grep -i composite
# Should show: "Processing composite symbol" not "Skipping"
```

**If architecture changed**: The goal is to set a flag that `field_extractor.pl:74` can detect.

---

### Task 1: Add Composite Context to PPI Generator

**Success**: `cargo run -p codegen --bin debug-ppi -- --composite '$val[0] + $val[1]'` outputs `vals.get(0)...` not `get_array_element(val, 0)`

**Implementation**:

The PPI **parser** already handles `$val[n]`, `$prt[n]`, `$raw[n]`. We need a generator context switch:

1. Add `ExpressionContext` enum to `codegen/src/ppi/rust_generator/`:

   ```rust
   pub enum ExpressionContext {
       Regular,    // $val is single TagValue
       Composite,  // $val[n], $prt[n], $raw[n] are slice accesses
   }
   ```

2. When context is `Composite`, generate:

   ```rust
   // $val[0] →
   vals.get(0).cloned().unwrap_or_default()

   // $prt[2] →
   prts.get(2).cloned().unwrap_or_default()

   // $raw[1] →
   raws.get(1).cloned().unwrap_or_default()
   ```

3. Add `--composite` flag to `debug-ppi` binary for testing

**Key files to modify**:

```bash
codegen/src/ppi/rust_generator/mod.rs  # Add context enum and conditional generation
codegen/src/debug_ppi.rs               # Add --composite flag
```

**If architecture changed**: Find where `get_array_element` is generated and add context check.

---

### Task 2: Create Composite Function Signature

**Success**: Generated composite functions have the correct 3-array signature

**Implementation**:

1. Add new function type to `codegen/src/ppi/fn_registry/`:

   ```rust
   pub enum GeneratedFnKind {
       ValueConv,      // fn(val, ctx) -> Result<TagValue>
       PrintConv,      // fn(val, ctx) -> TagValue
       CompositeValue, // fn(vals, prts, raws) -> Result<TagValue>  // NEW
       CompositePrint, // fn(val, vals, prts) -> TagValue           // NEW
   }
   ```

2. Modify function generation in `rust_generator/` to emit correct signature based on kind

3. Update `CompositeTagStrategy` to:
   - Call PPI pipeline with `CompositeValue` kind
   - Store function pointer in `CompositeTagDef` instead of Perl string

**Key files to modify**:

```bash
codegen/src/ppi/fn_registry/mod.rs      # Function kind enum
codegen/src/ppi/rust_generator/mod.rs   # Signature generation
codegen/src/strategies/composite_tag.rs # Call PPI, store fn pointer
```

**If architecture changed**: The goal is function pointers, not strings. Find how regular PrintConv stores its function references.

---

### Task 3: Update CompositeTagDef Structure

**Success**: `CompositeTagDef` holds function pointers that can be called at runtime

**Implementation**:

Change from:

```rust
pub struct CompositeTagDef {
    pub value_conv: Option<&'static str>,   // Perl string - USELESS
    pub print_conv: Option<&'static str>,   // Perl string - USELESS
}
```

To:

```rust
pub struct CompositeTagDef {
    pub name: &'static str,
    pub module: &'static str,
    pub require: &'static [&'static str],
    pub desire: &'static [&'static str],
    pub inhibit: &'static [&'static str],
    pub value_conv_fn: Option<CompositeValueConvFn>,
    pub print_conv_fn: Option<CompositePrintConvFn>,
    pub groups: &'static [(u8, &'static str)],
}

pub type CompositeValueConvFn = fn(&[TagValue], &[TagValue], &[TagValue]) -> Result<TagValue, ExifError>;
pub type CompositePrintConvFn = fn(&TagValue, &[TagValue], &[TagValue]) -> TagValue;
```

**Location**: `codegen/src/strategies/composite_tag.rs` generates this struct. Also need to add the type aliases to `codegen-runtime/src/lib.rs` for runtime use.

**If architecture changed**: Find where `PrintConvFn` is defined for regular tags and follow that pattern.

---

### Task 4: Generate Composite Function Bodies

**Success**: `make codegen` produces `src/generated/composite_tags.rs` with working functions for SIMPLE expressions

**Implementation**:

Modify `CompositeTagStrategy::finish_extraction()` to:

1. For each composite definition with ValueConv:

   - Parse the Perl expression through PPI pipeline with `Composite` context
   - Generate a Rust function with composite signature
   - Store function reference in generated struct

2. For expressions that fail PPI translation:
   - Check if manual fallback exists in `COMPOSITE_FALLBACKS`
   - If yes: reference the fallback function
   - If no: generate placeholder with `missing_composite_value_conv()`

Example generated output:

```rust
/// GPSLatitude composite - sign from reference
/// ExifTool: lib/Image/ExifTool/GPS.pm:381
/// Original: $val[1] =~ /^S/i ? -$val[0] : $val[0]
fn composite_value_gps_latitude(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue]
) -> Result<TagValue, ExifError> {
    let ref_val = vals.get(1).and_then(|v| v.as_string()).unwrap_or_default();
    let lat_val = vals.get(0).cloned().unwrap_or_default();
    if ref_val.to_uppercase().starts_with('S') {
        Ok(-lat_val.abs())
    } else {
        Ok(lat_val)
    }
}
```

**Verification**:

```bash
make codegen
rg "composite_value_" src/generated/ --type rust | head -20
# Should show generated composite functions
```

**If architecture changed**: The goal is generated Rust functions. Find where `ast_value_*` functions are generated and follow that pattern.

---

### Task 5: Enable Runtime Orchestration

**Success**: `resolve_and_compute_composites()` returns actual computed values

**Implementation**:

1. Uncomment the orchestration code in `src/composite_tags/orchestration.rs`
2. Update it to use the new `CompositeTagDef` with function pointers
3. Wire up the dependency resolution to call the generated functions

Key change in orchestration:

```rust
// OLD (commented out): Called manual dispatch function
let computed = compute_composite_tag(composite_def, &available_tags, &built_composites);

// NEW: Call the generated function pointer directly
let computed = if let Some(value_fn) = composite_def.value_conv_fn {
    // Build vals/prts/raws arrays from resolved dependencies
    let (vals, prts, raws) = resolve_dependency_arrays(composite_def, &available_tags);
    value_fn(&vals, &prts, &raws).ok()
} else {
    None
};
```

**Files to modify**:

```bash
src/composite_tags/mod.rs          # Uncomment exports
src/composite_tags/orchestration.rs # Enable multi-pass logic
src/composite_tags/resolution.rs   # May need updates for array building
```

**Delete after wiring complete**:

```bash
src/composite_tags/dispatch.rs      # No longer needed (direct fn call)
```

**If architecture changed**: The orchestration logic itself is sound. Just update it to call function pointers instead of the dispatch match statement.

---

### Task 6: Migrate Complex Implementations to Fallbacks

**Success**: ImageSize, Megapixels, LensID work via manual fallbacks with clear PLACEHOLDER comments

**Implementation**:

These 3 composite expressions are too complex for automatic translation:

- **ImageSize**: Uses `$$self{TIFF_TYPE}` context
- **Megapixels**: Uses `my @d` local array
- **LensID**: Complex 200+ line algorithm with `@raw` access

**Migrate** (don't delete!) the existing implementations from `implementations.rs` to `codegen-runtime/src/composite_fallbacks.rs`:

```rust
// codegen-runtime/src/composite_fallbacks.rs

/// Registry of manually-implemented composite calculations
/// Each entry replaces a PPI-untranslatable expression
///
/// IMPORTANT: These are PLACEHOLDERS. As PPI capabilities expand,
/// move implementations to generated code.

use crate::{TagValue, ExifError};
use std::collections::HashMap;

/// ImageSize calculation - uses $$self{TIFF_TYPE} context
/// PLACEHOLDER: lib/Image/ExifTool/Exif.pm:4641-4660
pub fn composite_image_size(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    // Note: May need ExifContext for TIFF_TYPE access
) -> Result<TagValue, ExifError> {
    // Migrated from src/composite_tags/implementations.rs::compute_image_size
    // ... existing implementation ...
}

/// Megapixels calculation - uses local array extraction
/// PLACEHOLDER: lib/Image/ExifTool/Exif.pm:4662-4665
pub fn composite_megapixels(vals: &[TagValue], _prts: &[TagValue], _raws: &[TagValue]) -> Result<TagValue, ExifError> {
    // Migrated from src/composite_tags/implementations.rs::compute_megapixels
    // ... existing implementation ...
}

/// LensID calculation - complex algorithm
/// PLACEHOLDER: lib/Image/ExifTool/Exif.pm:5197-5255
pub fn composite_lens_id(vals: &[TagValue], _prts: &[TagValue], raws: &[TagValue]) -> Result<TagValue, ExifError> {
    // Migrated from src/composite_tags/implementations.rs::compute_lens_id
    // ... existing implementation ...
}

/// Registry for codegen to look up fallbacks
pub static COMPOSITE_FALLBACKS: LazyLock<HashMap<&'static str, CompositeValueConvFn>> = LazyLock::new(|| {
    HashMap::from([
        ("ImageSize", composite_image_size as CompositeValueConvFn),
        ("Megapixels", composite_megapixels as CompositeValueConvFn),
        ("LensID", composite_lens_id as CompositeValueConvFn),
    ])
});
```

In codegen, when PPI fails:

```rust
// Check if we have a manual fallback registered
if let Some(fallback_fn) = COMPOSITE_FALLBACKS.get(composite_name) {
    // Generate: value_conv_fn: Some(codegen_runtime::composite_fallbacks::composite_image_size)
} else {
    // Generate: value_conv_fn: Some(missing_composite_value_conv) with warning
}
```

**After migration, delete**:

```bash
src/composite_tags/implementations.rs  # All migrated to codegen-runtime
```

**If architecture changed**: Find where `missing_print_conv` is used and follow that pattern.

---

## Proof of Completion

- [ ] Patcher sets `__hasCompositeTags` for modules with `AddCompositeTags` calls
- [ ] `cargo run -p codegen --bin debug-ppi -- --composite '$val[0] + $val[1]'` generates `vals.get(0)...`
- [ ] `make codegen` generates composite functions (not empty HashMap)
- [ ] `rg "composite_value_" src/generated/` shows 50+ generated functions
- [ ] `cargo run --bin compare-with-exiftool -- third-party/exiftool/t/images/Canon.jpg | grep Composite` shows matches
- [ ] ImageSize, Megapixels, LensID work via fallbacks in `codegen-runtime/`
- [ ] `cargo t test_composite` passes (create integration tests)
- [ ] `make precommit` passes
- [ ] `src/composite_tags/implementations.rs` deleted (migrated to codegen-runtime)
- [ ] `src/composite_tags/dispatch.rs` deleted (replaced by direct fn calls)

---

## Expression Pattern Reference

Common patterns the PPI pipeline must handle (with composite context):

| Pattern         | Example                                 | Rust Translation              |
| --------------- | --------------------------------------- | ----------------------------- |
| First available | `$val[0] \|\| $val[1]`                  | `vals.get(0).or(vals.get(1))` |
| Sign from ref   | `$val[1] =~ /^S/i ? -$val[0] : $val[0]` | Ternary with regex            |
| Concatenation   | `"$val[0] $val[1]"`                     | `format!("{} {}", ...)`       |
| Simple math     | `$val[0] * $val[1] / 1000000`           | Direct arithmetic             |
| Sprintf         | `sprintf("%.1f", $val[0])`              | `sprintf_perl(...)`           |
| Printed value   | `$prt[2]`                               | `prts.get(2).cloned()`        |
| Math difference | `$val[1] - $val[0]`                     | `vals[1] - vals[0]`           |

---
