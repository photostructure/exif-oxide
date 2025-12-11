# P03c: Composite Tags via PPI Code Generation

**Prerequisites**: [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md), [ARCHITECTURE.md](../ARCHITECTURE.md), [CODEGEN.md](../CODEGEN.md), [PRINTCONV-VALUECONV-GUIDE.md](../guides/PRINTCONV-VALUECONV-GUIDE.md), [COMPOSITE_TAGS.md](../../third-party/exiftool/doc/concepts/COMPOSITE_TAGS.md)

**Analysis Data** (run `make expression-analysis` to regenerate):
- [composite-dependencies.json](../analysis/expressions/composite-dependencies.json) - 54 composite tag definitions with dependencies
- [required-expressions-analysis.json](../analysis/expressions/required-expressions-analysis.json) - 154 required tags with expression patterns

---

## Current Status (2025-12-08)

### ✅ Completed Tasks

| Task | Description | Key Changes |
|------|-------------|-------------|
| **0** | Patcher composite detection | `__hasCompositeTags` marker inserted, 18+ modules detected |
| **1** | ExpressionContext enum | `ExpressionContext::Composite` in [codegen/src/ppi/types.rs](../../codegen/src/ppi/types.rs) |
| **2** | Composite function types | `CompositeValueConvFn`, `CompositePrintConvFn` in [src/core/types.rs](../../src/core/types.rs#L120-L153) |
| **3** | CompositeTagDef update | Struct now uses function pointers, preserves Perl in `*_expr` fields |
| **4** | Generate composite function bodies | **46 functions generated**, 29 ValueConv + 17 PrintConv pointers set |
| **4b** | Fix lint warnings | Per-function `#[allow(...)]` attributes, `.first()` for index 0 |

### Key Infrastructure Now Available

1. **Context-aware code generation** - `RustGenerator::with_context(ExpressionContext::Composite, ...)` generates:
   - `$val[0]` → `vals.first().cloned().unwrap_or(TagValue::Empty)` (uses `.first()` for clippy)
   - `$val[n]` → `vals.get(n).cloned().unwrap_or(TagValue::Empty)` (n > 0)
   - `$prt[n]` / `$raw[n]` → same pattern with `prts` / `raws`
   - Bare `$val` in composite context → `vals.first().cloned().unwrap_or(TagValue::Empty)`

2. **Unit tests** proving it works - see [codegen/src/ppi/rust_generator/tests/mod.rs](../../codegen/src/ppi/rust_generator/tests/mod.rs#L109-L232)

3. **Generated output** - [src/generated/composite_tags.rs](../../src/generated/composite_tags.rs) now has **46 working functions**:
   ```rust
   #[allow(unused_variables, clippy::get_first, clippy::collapsible_else_if, ...)]
   pub fn composite_valueconv_exif_aperture(
       vals: &[TagValue],
       prts: &[TagValue],
       raws: &[TagValue],
       ctx: Option<&ExifContext>,
   ) -> Result<TagValue, ExifError> {
       Ok(if vals.first().cloned().unwrap_or(TagValue::Empty).is_truthy() {
           vals.first().cloned().unwrap_or(TagValue::Empty).clone()
       } else {
           vals.get(1).cloned().unwrap_or(TagValue::Empty)
       })
   }

   pub static COMPOSITE_EXIF_APERTURE: CompositeTagDef = CompositeTagDef {
       name: "Aperture",
       value_conv: Some(composite_valueconv_exif_aperture),  // Function pointer!
       value_conv_expr: Some("$val[0] || $val[1]"),
       // ...
   };
   ```

4. **Lint compliance** - All generated code passes `make lint`:
   - Per-function `#[allow(...)]` attributes suppress generated-code lints
   - Uses `.first()` instead of `.get(0)` to satisfy `clippy::get_first`
   - `#[allow(unused_imports)]` for imports that may not always be used

### 🚧 Remaining Tasks

| Task | Description | Complexity |
|------|-------------|------------|
| **5** | Enable runtime orchestration | MEDIUM |
| **6** | Migrate complex implementations to exif-oxide-core fallbacks | MEDIUM |

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

**Edge case - bare `$val` in composite context**: Some composites use `$val` without subscript even though they have dependencies:

```perl
# CircleOfConfusion - has Require[ScaleFactor35efl] but uses $val not $val[0]
ValueConv => 'sqrt(24*24+36*36) / ($val * 1440)'
```

In this case, `$val` refers to the **first dependency's value** (same as `$val[0]`). The generator must recognize that in composite context, bare `$val` maps to `vals.get(0)` when the expression contains no subscripts but the tag has dependencies.

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

Based on analysis of required composite tags (see `docs/analysis/expressions/composite-dependencies.json`):

| Category          | Tags                                                                                                           | Approach                                   |
| ----------------- | -------------------------------------------------------------------------------------------------------------- | ------------------------------------------ |
| **SIMPLE (75%)**  | Aperture, ShutterSpeed, ISO, Lens, GPSLatitude, GPSLongitude, GPSPosition, GPSDateTime, SubSecDateTimeOriginal | PPI auto-generation with composite context |
| **COMPLEX (25%)** | ImageSize (`$$self{TIFF_TYPE}`), Megapixels (`my @d` local array), LensID (`@raw` + complex algorithm)         | Keep existing manual implementations       |

### D2. PrintConv Function Call Dominance

**Critical finding from `docs/analysis/expressions/required-expressions-analysis.json`**: **58 of 71 PrintConv usages are function calls**, not simple expressions:

| Function                                    | Count | Blocking Tags                                                    |
| ------------------------------------------- | ----- | ---------------------------------------------------------------- |
| `$self->ConvertDateTime($val)`              | 28    | SubSecDateTimeOriginal, GPSDateTime, all datetime composites     |
| `Image::ExifTool::GPS::ToDMS(...)`          | 10    | GPSLatitude, GPSLongitude (all manufacturers)                    |
| `Image::ExifTool::Exif::PrintExposureTime(...)` | 4     | ShutterSpeed                                                     |
| `Image::ExifTool::Exif::PrintFNumber(...)`  | 2     | Aperture                                                         |
| `Image::ExifTool::Canon::PrintFocalRange(...)` | 1     | Canon.Lens                                                       |

**Implication**: These PrintConv function calls need fallback implementations in `src/core/`. Priority order:
1. `ConvertDateTime` - blocks 28 tags
2. `ToDMS` - blocks GPS display formatting
3. `PrintExposureTime` / `PrintFNumber` - core camera tag display

### D3. Composites Without ValueConv (Pass-Through)

Several composites have **no ValueConv** - they pass through dependency values and only apply PrintConv:

- `SubSecCreateDate`, `SubSecDateTimeOriginal`, `SubSecModifyDate` - datetime assembly
- `ThumbnailImage`, `PreviewImage`, `JpgFromRaw` - binary extraction
- `DigitalZoom`, `OriginalDecisionData` - direct value pass-through

The orchestration must handle this case: resolve dependencies, build `vals` array, skip ValueConv, apply PrintConv.

### D4. Composite-on-Composite Dependencies

Some composites depend on OTHER composites. The multi-pass orchestration handles this, but test with these chains:

```
Megapixels → ImageSize
CircleOfConfusion → ScaleFactor35efl
DOF → FocalLength, Aperture, CircleOfConfusion
HyperfocalDistance → FocalLength, Aperture, CircleOfConfusion
LightValue → Aperture, ShutterSpeed, ISO (all composites!)
```

Test with `Canon.jpg` which exercises these chains.

### D5. The `@raw` Array Distinction

**Critical for LensID (Nikon)**: Uses `@raw` not `@val`:

```perl
ValueConv => 'sprintf("%.2X"." %.2X"x7, @raw)'
```

This needs the **unconverted** dependency values. The `resolve_dependency_arrays()` function must populate all three arrays (`vals`, `prts`, `raws`), even if most composites only use `@val`.

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

1. **`$val` vs `$val[0]` ambiguity**: In composite context, `$val` alone refers to the current computed value (inside PrintConv), while `$val[0]` refers to the first dependency. Exception: some ValueConv expressions use bare `$val` to mean `$val[0]` (see CircleOfConfusion).

2. **Inhibit evaluation order**: If checking `Inhibit` against an unbuilt composite, must defer (ExifTool.pm:4034-4036).

3. **`$prt[]` requires two-phase**: Some ValueConv expressions use `$prt[n]`, meaning we need PrintConv'd values of dependencies, not just ValueConv'd.

4. **Module-specific composites**: Canon, Nikon, etc. define their own `%Composite` tables. The `AddCompositeTags()` function merges them into a global registry.

5. **Circular dependency handling**: ExifTool does one final pass ignoring Inhibit tags if stuck (ExifTool.pm:4103-4110).

6. **Pass-through composites**: Several composites have no ValueConv (SubSec*, ThumbnailImage, PreviewImage). They assemble dependency values directly into the result - don't skip them just because ValueConv is missing.

7. **`@raw` is rare but critical**: Only Nikon.LensID uses `@raw` in required tags, but it's essential. The `resolve_dependency_arrays()` must always populate all three arrays.

8. **PrintConv function calls dominate**: 58/71 PrintConv expressions are function calls like `$self->ConvertDateTime()`. These need manual fallbacks - PPI can't translate them.

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

**Location**: `codegen/src/strategies/composite_tag.rs` generates this struct. Also need to add the type aliases to `src/core/lib.rs` for runtime use.

**If architecture changed**: Find where `PrintConvFn` is defined for regular tags and follow that pattern.

---

### Task 4: Generate Composite Function Bodies ⬅️ NEXT TASK

**Success**: `make codegen` produces working functions in `src/generated/composite_tags.rs`, not just `None` placeholders

**Prerequisite Understanding**:

Tasks 1-3 have already built the foundation:
- `RustGenerator::with_context(ExpressionContext::Composite, ...)` generates context-aware code
- `CompositeValueConvFn` / `CompositePrintConvFn` types are defined
- `CompositeTagDef` struct has function pointer fields (currently `None`)

The generator already produces correct code for composite expressions - it just isn't being called yet!

**Proof the generator works** (run this to verify):

```bash
# This already works - generates composite-context code
cargo run -p codegen --bin debug-ppi -- --composite '$val[0] || $val[1]'
# Output: vals.get(0).cloned().unwrap_or_default() || vals.get(1).cloned().unwrap_or_default()
```

---

#### Step-by-Step Implementation

**Step 1: Study how TagKitStrategy generates functions**

Look at how regular PrintConv/ValueConv functions are generated:

```bash
# Key file to study:
codegen/src/strategies/tag_kit/printconv_generation.rs

# Find where it calls the PPI pipeline:
rg "generate_function|RustGenerator" codegen/src/strategies/tag_kit/
```

The pattern is:
1. Get the Perl expression from JSON
2. Parse it through PPI (`ppi_ast.pl` or inline AST)
3. Call `RustGenerator::new().generate_function(&ast)`
4. Store the generated function code

**Step 2: Modify `CompositeTagStrategy::generate_composite_tags_module()`**

Location: [codegen/src/strategies/composite_tag.rs:207](../../codegen/src/strategies/composite_tag.rs#L207)

Currently it outputs:
```rust
value_conv: None, // TODO: Generate via PPI pipeline
```

Change it to:
1. For each definition with `value_conv_expr`:
   - Call `ppi_ast.pl` to parse the expression
   - Create `RustGenerator::with_context(ExpressionType::ValueConv, ExpressionContext::Composite, ...)`
   - Generate the function body
   - Output both the function definition AND the pointer

**Step 3: Generate functions BEFORE the static definitions**

The generated file should look like:

```rust
// Generated composite functions (NEW SECTION)
fn composite_valueconv_exif_aperture(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    // Generated from: $val[0] || $val[1]
    Ok(vals.get(0).cloned().unwrap_or_default().or_else(||
        vals.get(1).cloned().unwrap_or_default()))
}

// Static definitions reference the functions
pub static COMPOSITE_EXIF_APERTURE: CompositeTagDef = CompositeTagDef {
    name: "Aperture",
    value_conv: Some(composite_valueconv_exif_aperture),  // Function pointer!
    // ...
};
```

**Step 4: Handle PPI failures gracefully**

Some expressions will fail (ExifTool function calls like `Image::ExifTool::Exif::PrintFNumber($val)`).

For failures:
```rust
value_conv: None,  // PPI failed: "Image::ExifTool::Exif::PrintFNumber($val)"
```

Track statistics:
```rust
info!("Generated {}/{} composite ValueConv functions ({} failed)",
      success_count, total_count, fail_count);
```

---

#### Key Code References

| What | Where |
|------|-------|
| PPI pipeline entry point | `codegen/src/ppi/shared_pipeline.rs` |
| RustGenerator with context | `codegen/src/ppi/rust_generator/generator.rs:47` |
| Example of calling PPI | `codegen/src/strategies/tag_kit/printconv_generation.rs` |
| Current composite generation | `codegen/src/strategies/composite_tag.rs:361-384` |
| Composite function types | `src/core/types.rs:120-153` |

---

#### Test Expression Corpus

These MUST generate successfully (they use patterns the generator already handles):

```perl
# Simple - should definitely work
$val[0] || $val[1]                                   # Aperture
$val[1] - $val[0]                                    # ImageWidth
"$val[0] $val[1]"                                    # DateTimeCreated

# Ternary with regex - should work
$val[1] =~ /^S/i ? -$val[0] : $val[0]               # GPSLatitude

# String interpolation with $prt
"$prt[0], $prt[1]"                                   # GPSPosition
```

These will FAIL (and should set `value_conv: None`):

```perl
Image::ExifTool::Exif::PrintFNumber($val)           # ExifTool function call
$self->ConvertDateTime($val[0])                      # Method call
```

---

#### Verification

```bash
# Regenerate and check for generated functions
make codegen

# Should show function definitions, not just "None"
grep -A2 "value_conv:" src/generated/composite_tags.rs | head -20

# Count successes
grep -c "value_conv: Some" src/generated/composite_tags.rs
# Target: At least 30+ (simple expressions)

# Check for actual function definitions
grep -c "fn composite_valueconv_" src/generated/composite_tags.rs
```

---

#### Common Pitfalls

1. **Function signature mismatch**: The generated function MUST match `CompositeValueConvFn`:
   ```rust
   fn(vals: &[TagValue], prts: &[TagValue], raws: &[TagValue], ctx: Option<&ExifContext>) -> Result<TagValue>
   ```

2. **Missing imports**: The generated file needs `use crate::core::TagValue;`

3. **Inline AST**: Check if the JSON has `ValueConv_ast` (pre-parsed) or just `ValueConv` (needs `ppi_ast.pl`)

4. **The `$val` vs `$val[n]` ambiguity**: In composite context, bare `$val` in ValueConv might mean `$val[0]`. Check ExifTool behavior if unsure.

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

**Migrate** (don't delete!) the existing implementations from `implementations.rs` to `src/core/composite_fallbacks.rs`:

```rust
// src/core/composite_fallbacks.rs

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
    // Generate: value_conv_fn: Some(crate::core::composite_fallbacks::composite_image_size)
} else {
    // Generate: value_conv_fn: Some(missing_composite_value_conv) with warning
}
```

**After migration, delete**:

```bash
src/composite_tags/implementations.rs  # All migrated to exif-oxide-core
```

**If architecture changed**: Find where `missing_print_conv` is used and follow that pattern.

---

## Proof of Completion

### Checklist

- [x] Patcher sets `__hasCompositeTags` for modules with `AddCompositeTags` calls
- [x] `cargo run -p codegen --bin debug-ppi -- --composite '$val[0] + $val[1]'` generates `vals.get(0)...`
- [x] `make codegen` generates composite functions (not empty HashMap)
- [x] `rg "composite_value_" src/generated/` shows 46 generated functions
- [x] `cargo run --bin compare-with-exiftool -- Canon.jpg | grep Composite` shows matches (7+ tags)
- [ ] ImageSize, Megapixels, LensID work via fallbacks in `src/core/` (Task 6)
- [ ] `cargo t test_composite` passes (create integration tests) (Task 6)
- [x] `make lint` passes
- [ ] `src/composite_tags/implementations.rs` deleted (migrated to exif-oxide-core) (Task 6)
- [ ] `src/composite_tags/dispatch.rs` deleted (replaced by direct fn calls) (Task 6)

### Specific Tag Verification

Run these commands to verify core composite tags work:

```bash
# GPS composites (decimal values per TRUST-EXIFTOOL.md)
cargo run --bin compare-with-exiftool -- third-party/exiftool/t/images/GPS.jpg 2>/dev/null \
  | grep -E "Composite:(GPSLatitude|GPSLongitude|GPSPosition|GPSAltitude)"
# Expected: All should match with decimal coordinate values

# Camera composites
cargo run --bin compare-with-exiftool -- third-party/exiftool/t/images/Canon.jpg 2>/dev/null \
  | grep -E "Composite:(Aperture|ShutterSpeed|ImageSize|Megapixels|ISO)"
# Expected: All should match

# Composite-on-composite chain (LightValue depends on Aperture, ShutterSpeed, ISO)
cargo run --bin compare-with-exiftool -- third-party/exiftool/t/images/Canon.jpg 2>/dev/null \
  | grep "Composite:LightValue"
# Expected: Should match (tests multi-pass resolution)

# Nikon LensID with @raw array
cargo run --bin compare-with-exiftool -- third-party/exiftool/t/images/Nikon.jpg 2>/dev/null \
  | grep "Composite:LensID"
# Expected: Should match (tests @raw array handling)
```

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

## Implementation Progress

### ✅ Task 0: Fix Patcher Composite Detection - COMPLETE

**Changes made:**
- Modified `codegen/scripts/exiftool-patcher.pl` (lines 59-65) to detect `AddCompositeTags` calls
- Inserts `our $__hasCompositeTags = 1;` before each `AddCompositeTags` call

### ✅ Tasks 1-3: Infrastructure - COMPLETE

All foundation work completed:
- `ExpressionContext::Composite` enum added
- `CompositeValueConvFn` / `CompositePrintConvFn` types defined
- `CompositeTagDef` updated with function pointer fields

### ✅ Task 4: Generate Composite Function Bodies - COMPLETE

**Work completed (2025-12-08):**

1. **Signature generation** - [codegen/src/ppi/rust_generator/signature.rs](../../codegen/src/ppi/rust_generator/signature.rs):
   - Added `generate_signature_with_context()` function
   - Composite functions get correct signature: `fn(vals, prts, raws, ctx) -> Result<TagValue>`
   - Per-function `#[allow(...)]` attributes for lint suppression

2. **Generator context awareness** - [codegen/src/ppi/rust_generator/generator.rs](../../codegen/src/ppi/rust_generator/generator.rs):
   - Updated `generate_function()` to use context-aware signatures
   - Updated `generate_body()` to wrap composite PrintConv in `Ok()`
   - Fixed array access to use `TagValue::Empty` instead of `unwrap_or_default()`
   - Uses `.first()` for index 0 to satisfy `clippy::get_first`

3. **Visitor enhancements** - [codegen/src/ppi/rust_generator/visitor.rs](../../codegen/src/ppi/rust_generator/visitor.rs):
   - Added `visit_symbol()` override for composite context
   - Bare `$val` → `vals.first().cloned().unwrap_or(TagValue::Empty)` in composite context
   - `@val`/`@prt`/`@raw` patterns properly rejected (array splatting not supported)
   - Added `interpolate_composite_string()` for patterns like `"$prt[0], $prt[1]"`

4. **CompositeTagStrategy** - [codegen/src/strategies/composite_tag.rs](../../codegen/src/strategies/composite_tag.rs):
   - Added `try_generate_function()` helper
   - Phase 1: Generate functions for all expressions via PPI
   - Phase 2: Write generated functions to output file
   - Phase 3: Set function pointers in static definitions
   - `#[allow(unused_imports)]` for imports that may not always be used

**Results:**
```bash
# Generated functions
grep -c "^pub fn composite_" src/generated/composite_tags.rs
# 46

# ValueConv pointers set
grep -c "value_conv: Some" src/generated/composite_tags.rs
# 29

# PrintConv pointers set
grep -c "print_conv: Some" src/generated/composite_tags.rs
# 17

# All lints pass
make lint  # ✅ Passes
cargo test -p codegen  # ✅ 127 passed, 3 ignored
```

### ✅ Task 5: Enable Runtime Orchestration - COMPLETE

**Work completed (2025-12-08):**

1. **resolution.rs** - Updated dependency resolution:
   - `can_build_composite()` now takes `&CompositeTagDef` (not `&str`)
   - Added inhibit tag checking (ExifTool: lib/Image/ExifTool.pm:4034-4036)
   - Added `resolve_dependency_arrays()` to build vals/prts/raws arrays for function calls

2. **orchestration.rs** - Enabled multi-pass logic with function pointers:
   - `compute_composite_value()` calls generated function pointers directly
   - Falls back to manual implementations for composites without generated functions
   - `apply_composite_print_conv()` calls generated PrintConv functions
   - GPS coordinates bypass PrintConv per TRUST-EXIFTOOL.md

3. **mod.rs** - Updated exports:
   - Exports `resolve_dependency_arrays` for use by orchestration
   - Made `implementations` module public for fallback access

**Results:**
```bash
# Composite tags now being computed:
cargo run --bin compare-with-exiftool -- Canon.jpg | grep "Composite:"
# Composite:Lens, Composite:ISO, Composite:LensID, Composite:SubSecCreateDate, etc.

# GPS composites with decimal format:
cargo run --bin compare-with-exiftool -- GPS.jpg | grep "Composite:GPS"
# Composite:GPSLatitude: Some(Number(54.9896666666667))
# Composite:GPSLongitude: Some(Number(-1.91416666666667))
```

### ⏳ Remaining Tasks

- **Task 6**: Migrate complex implementations + PrintConv fallbacks (some PPI expressions fail)

---

## Handoff Notes for Next Engineer

### What Was Done (Tasks 0-5 Complete)

**Task 5 is now complete** - runtime orchestration is enabled and composite tags are being computed.

#### Codegen Changes (Tasks 0-4):

1. **signature.rs** - Added `generate_signature_with_context()` for composite function signatures
2. **generator.rs** - Context-aware code generation with `.first()` for index 0
3. **visitor.rs** - Composite-specific handling for `$val`, `$prt[n]`, `$raw[n]`
4. **composite_tag.rs** - PPI pipeline integration generating 46 functions

#### Runtime Changes (Task 5):

1. **resolution.rs** - `can_build_composite(&CompositeTagDef)` + `resolve_dependency_arrays()`
2. **orchestration.rs** - Multi-pass loop calling function pointers, with manual fallbacks
3. **mod.rs** - Updated exports

### Current State

```bash
# Verify current state (all should pass)
make lint        # ✅ Passes
cargo test -p codegen  # ✅ 127 passed, 3 ignored

# Generated functions
grep -c "^pub fn composite_" src/generated/composite_tags.rs
# 46 functions

# Function pointers set
grep -c "value_conv: Some" src/generated/composite_tags.rs  # 29
grep -c "print_conv: Some" src/generated/composite_tags.rs  # 17
```

### Remaining Tasks

#### Task 5: Enable Runtime Orchestration

**Goal**: Wire up the generated functions so composite tags are actually calculated at runtime.

**Key files to examine**:
- `src/composite/orchestration.rs` - likely needs to be uncommented/enabled
- The `COMPOSITE_TAGS` registry in `src/generated/composite_tags.rs`

**Implementation approach**:
1. Look for `orchestration.rs` or similar disabled code
2. Wire up dependency resolution (resolve `require`/`desire` tags first)
3. Call the generated `value_conv` function with resolved values
4. Apply `print_conv` if present

#### Task 6: Fallback Implementations

**Goal**: Handle expressions that PPI can't translate via manual fallbacks in exif-oxide-core.

**Expressions that fail** (expected - use ExifTool function calls):
```
Image::ExifTool::Exif::PrintFNumber($val)  - Aperture PrintConv
Image::ExifTool::GPS::ToDMS(...)           - GPS coordinate formatting
$self->ConvertDateTime($val)               - DateTime PrintConv
sprintf("%.2X"." %.2X"x7, @raw)            - Array splatting (@raw)
```

**Implementation approach**:
1. Add fallback functions to `src/core/lib.rs`
2. Update `composite_tag.rs` to emit fallback function pointers when PPI fails
3. Match by tag name or expression pattern

### Related Issue

The failing test `test_key_exif_ifd_tag_grouping` is tracked in [P04-colorspace-support.md](P04-colorspace-support.md) - it's a separate bug where EXIF ColorSpace (0xA001) is not being parsed from ExifIFD. This is **unrelated** to composite tags.

---
