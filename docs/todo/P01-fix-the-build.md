# P01: Fix the Build

**Prerequisites**: Read [GETTING-STARTED.md](../GETTING-STARTED.md) first, especially the "Current Project State" section.

## Part 1: Define Success

**Problem**: Build fails with compilation errors, blocking all development.

**Why it matters**: Nothing else can proceed - no testing, no new features, no integration.

**Solution**: Unify function signatures in codegen to match type system expectations.

**Success test**:
```bash
make precommit  # MUST pass with zero errors
```

**Key constraint**: Don't break existing working code. Generated functions that can't be fixed should emit placeholders, not broken syntax.

---

## Part 2: Share Your Expertise

### A. Root Cause Analysis

The errors typically break down as:
- **Type mismatches** (E0308) - cascading from signature issues
- **Dereference errors** (E0614)
- **Method not found** (E0599)
- **Argument count errors** (E0061)

**The core issue**: `PrintConv::Function` and `ValueConv::Function` expect:
```rust
// PrintConv::Function signature
fn(&TagValue, Option<&ExifContext>) -> TagValue

// ValueConv::Function signature
fn(&TagValue, Option<&ExifContext>) -> Result<TagValue, ExifError>
```

But generated functions may have inconsistent signatures:
1. Some missing context parameter: `fn(&TagValue) -> TagValue`
2. Some returning Result: `fn(&TagValue, Option<&ExifContext>) -> Result<TagValue, ExifError>`
3. Some are `fn item` not `fn pointer` (needs explicit cast)

### B. Key Files

```bash
# The type definitions (what signatures SHOULD be)
src/types/tag_info.rs          # PrintConv::Function definition

# Where signatures are generated (what to FIX)
codegen/src/ppi/rust_generator/   # Function generation
codegen/src/strategies/           # Strategy output templates
codegen/src/impl_registry/fallback_helper.rs  # Registry fallback generation

# Sample generated code locations
src/generated/Exif_pm/           # EXIF module tags
src/generated/Canon_pm/          # Canon manufacturer tags
src/generated/functions/         # PPI-generated functions
```

### C. The Fix Strategy

**Option A**: Make all generated functions match expected signature ✅ CHOSEN
- Add `_ctx: Option<&ExifContext>` parameter to all functions
- Unwrap Result to TagValue (return original value on error)
- Cast fn items to fn pointers where needed

**Option B**: Change type definitions to accept what's generated
- Make PrintConv/ValueConv accept Result-returning functions
- Add wrapper that handles the unwrapping
- More invasive but might be cleaner long-term

**Recommendation**: Start with Option A - it's more surgical and less risky.

### D. Landmines

1. **NEVER edit `src/generated/` directly** - changes will be overwritten by `make codegen`
2. **Many errors are cascading** - fixing the root cause (signature mismatch) should resolve most of them
3. **Some errors are in generated lookup tables** - these are separate codegen bugs

### E. Learned the Hard Way

From previous build fixes:
- The `cast_\` backslash disaster took 3 hours - `\$val` pattern not handled
- sprintf argument explosion took 2 hours - binary ops inside function args
- unpack/pack/split chain took 4 hours - multi-function chains without parens

**Golden rule**: If codegen can't generate valid Rust, throw `CodeGenError::UnsupportedStructure` to trigger fallback placeholder. A placeholder that compiles is infinitely better than malformed code.

---

## Part 3: Progress (December 2025)

### Completed ✅

#### 1. ExifError Type Unification
**Problem**: Two ExifError types - `types::errors::ExifError` and `codegen_runtime::ExifError` - caused ~154 type mismatch errors.

**Fix**:
- Updated `codegen-runtime/src/types.rs` to be the single source of truth for `ExifError`
- Added `Io(#[from] std::io::Error)` and `FileDetection(String)` variants
- Updated `src/types/errors.rs` to re-export from codegen_runtime
- Updated `src/types/mod.rs` to avoid duplicate exports

#### 2. PrintConv/ValueConv Slot Mismatches
**Problem**: Same Perl expression used as both PrintConv and ValueConv got ONE function (first-wins), but they need DIFFERENT return types.

**Fix**: Updated `codegen/src/ppi/fn_registry/registry.rs`:
- `hash_ast_structure()` now includes `expression_type` in the hash
- Added `Hash` derive to `ExpressionType` enum in `codegen/src/ppi/types.rs`

#### 3. Lookup Table Type Issues
**Problem**: Mixed-type arrays (strings + integers) declared as `[&'static str; N]` but contained bare integers.

**Fix**: Updated `codegen/src/strategies/scalar_array.rs`:
- `format_array_elements()` now checks if `element_type == "&'static str"`
- If so, stringifies integer and float values

#### 4. Manual Function Signatures
**Problem**: ~35 print_conv and ~25 value_conv functions had old signature without `ctx` parameter.

**Fix**:
- Updated all functions in `src/implementations/print_conv.rs`
- Updated all functions in `src/implementations/value_conv.rs`
- Added `ExifContext` import to both files

#### 5. Registry Function Signatures
**Problem**: Registry type aliases and apply_* methods didn't include context parameter.

**Fix**: Updated `src/registry.rs`:
- `PrintConvFn` now includes `Option<&ExifContext>` parameter
- `ValueConvFn` now includes `Option<&ExifContext>` parameter
- `RawConvFn` now includes `Option<&ExifContext>` parameter
- `apply_print_conv()`, `apply_value_conv()`, `apply_raw_conv()` now accept and pass ctx

#### 6. Fallback Helper Context Passing
**Problem**: Generated wrapper functions didn't pass ctx to implementation functions.

**Fix**: Updated `codegen/src/impl_registry/fallback_helper.rs`:
- `generate_printconv_function()` now generates `{}(val, ctx)` instead of `{}(val)`
- `generate_valueconv_function()` now generates `{}(val, ctx)` instead of `{}(val)`
- `generate_condition_function()` now generates `{}(val, ctx)` instead of `{}(val)`

#### 7. Visitor ExifTool Function Calls
**Problem**: When calling registered ExifTool implementation functions, ctx wasn't passed.

**Fix**: Updated `codegen/src/ppi/rust_generator/visitor.rs` lines 580-591:
- Added ctx to function call arguments for `FunctionImplementation::ExifToolModule` cases

### Current Error State

**Before**: 280+ compilation errors
**After Phase 1**: 117 compilation errors (58% reduction)
**After Phase 2**: 41 compilation errors (65% total reduction from 117)
**After Phase 3**: 16 compilation errors (61% reduction from Phase 2)

```bash
# Run to see current state
./scripts/capture.sh cargo check
grep "^error\[" /tmp/stderr_*.txt | sort | uniq -c | sort -rn
```

Current breakdown (as of Dec 2025, Phase 3):
- 14 `error[E0308]: mismatched types`
- 2 `error[E0061]: argument count mismatches`

#### 8. TagValue Truthiness Support (Phase 2)
**Problem**: Perl ternary expressions like `$val ? ... : ...` need bool conversion.

**Fix**: Added `is_truthy()` method to TagValue in `codegen-runtime/src/tag_value/conversion.rs:127-153`:
- Follows Perl semantics: 0, "", "0", empty arrays are false
- All other values are truthy

#### 9. Ternary Expression Handling (Phase 2)
**Problem**: Generated ternary expressions had multiple issues:
- Conditions using `val` directly (not bool)
- Branches returning bare `val` (not owned TagValue)
- Integer/string literals not wrapped

**Fix**: Created shared helper functions in `codegen/src/ppi/rust_generator/expressions/binary_ops.rs`:
- `wrap_condition_for_bool()` - Adds `.is_truthy()` for TagValue conditions
- `wrap_branch_for_owned()` - Adds `.clone()` for `val`, `.into()` for literals
- `is_boolean_expression()` - Detects existing comparisons to avoid double-wrapping

Updated both `generator.rs` and `visitor.rs` to use these helpers (DRY).

#### 10. Power Operator Handling (Phase 2)
**Problem**: Perl `**` operator was either:
- Left as `**` in generated code (invalid Rust)
- Parsed incorrectly as `* *` (multiply + dereference) in expressions like `100 * 2**(16 - $val/256)`

**Fix**:
- `wrap_literal_for_tagvalue()` - Wraps bare integers with `.into()` for `power()` function
- Added `get_operator_precedence()` for proper operator splitting
- `try_binary_operation_pattern()` now splits on LOWEST precedence first
- `generate_binary_operation_from_parts()` recursively processes operands

Example: `100 * 2**(16 - $val/256)` now generates:
```rust
100i32 * power(2i32.into(), ((16i32.into()) - (val / 256i32)))
```

#### 11. Type Annotation Ambiguity Fix (Phase 3)
**Problem**: `.into()` calls on literals inside expressions create type inference ambiguity.

**Fix**: Updated `codegen/src/ppi/rust_generator/expressions/binary_ops.rs`:
- `wrap_literal_for_tagvalue()` now uses `Into::<TagValue>::into()` turbofish syntax
- `wrap_branch_for_owned()` also updated for consistency

#### 12. chr() and uc() Return Types (Phase 3)
**Problem**: `chr()` and `uc()` returned `String` but generated code expected `TagValue`.

**Fix**: Updated `codegen-runtime/src/string/transform.rs`:
- `chr()` now returns `TagValue::String(...)` directly
- `uc()` now returns `TagValue::String(...)` directly

#### 13. Join/Unpack Pattern Handling (Phase 3)
**Problem**: `join " ", unpack "FORMAT", $val` generated incorrect nested calls with wrong arg counts.

**Fix**: Updated `codegen/src/ppi/rust_generator/visitor.rs`:
- `join` handler now detects when second child is `unpack` FunctionCall
- Generates `join_unpack_binary(separator, format, &data)` directly for this pattern
- Added `join_vec()` fallback function for other join cases
- `unpack` standalone now wraps result in `TagValue::Array(...)`

Added `codegen-runtime/src/data/mod.rs`:
- New `join_vec(separator, &[TagValue])` function for non-unpack joins

#### 14. Missing ExpressionEvaluator Methods (Phase 3)
**Problem**: `evaluate_context_condition` and `evaluate_expression` methods didn't exist.

**Fix**: Added stub implementations to `src/types/binary_data.rs`:
- Both methods return `Err(...)` to trigger fallback behavior
- Marked as TODO: P07 for full implementation later

#### 15. Registry Global Wrapper Functions (Phase 3)
**Problem**: `apply_value_conv()`, `apply_raw_conv()`, `apply_print_conv_with_tag_id()` were called with 2 args but methods needed 3.

**Fix**: Updated `src/registry.rs`:
- All global wrapper functions now pass `None` as ctx parameter

#### 16. print_fraction Signature (Phase 3)
**Problem**: Generated code called `print_fraction(val, ctx)` but function only took 1 arg.

**Fix**: Updated `src/implementations/print_conv.rs`:
- `print_fraction()` now takes `(val: &TagValue, _ctx: Option<&ExifContext>)`

#### 17. Temporary HashMap Borrow Issues (Phase 3)
**Problem**: `HashMap::new()` passed as reference created temporaries dropped while borrowed.

**Fix**: Updated `src/registry.rs` and `src/types/binary_data.rs`:
- Store `HashMap::new()` in named variable before passing reference to `ExpressionEvaluator::new()`

#### 18. TagValue Comparison with f64 (Phase 3)
**Problem**: Generated code `val > 655.345f64` had no `PartialOrd<f64>` implementation.

**Fix**: Added to `codegen-runtime/src/tag_value/ops.rs`:
- `impl PartialEq<f64> for TagValue` and `impl PartialEq<f64> for &TagValue`
- `impl PartialOrd<f64> for TagValue` and `impl PartialOrd<f64> for &TagValue`

---

## Part 4: Remaining Work (16 errors)

### Task 1: TagValue as Bool Condition (6 errors)

**Problem**: Context-based conditions return `TagValue` but are used in `if` statements.

**Affected files**: `hash_39.rs`, `hash_8f.rs`, `hash_be.rs`, `hash_d7.rs`, `hash_d9.rs`

**Example** (hash_39.rs):
```rust
if ctx
    .and_then(|c| c.get_data_member("TimeScale").cloned())
    .unwrap_or(TagValue::U32(1))  // <-- This is TagValue, not bool!
{
```

**Root cause**: The visitor generates context lookups but doesn't add `.is_truthy()` to convert to bool.

**Fix approach**: In `codegen/src/ppi/rust_generator/visitor.rs`, when generating context-based conditions, wrap the result with `.is_truthy()`:
```rust
ctx.and_then(|c| c.get_data_member("TimeScale").cloned())
   .unwrap_or(TagValue::U32(1))
   .is_truthy()  // <-- Add this
```

**Files to modify**: `codegen/src/ppi/rust_generator/visitor.rs` - look for `get_data_member` generation.

### Task 2: Perl || Operator (3 errors)

**Problem**: Perl's `$val || "inf"` doesn't translate to Rust's `||`.

**Affected file**: `hash_cb.rs`

**Current broken output**:
```rust
(val || Into::<TagValue>::into("inf"))  // Rust || expects bools!
```

**Correct output should be**:
```rust
if val.is_truthy() { val.clone() } else { Into::<TagValue>::into("inf") }
```

**Root cause**: The binary operator handler treats `||` as Rust's boolean OR, not Perl's "return first truthy value" operator.

**Fix approach**: In `codegen/src/ppi/rust_generator/expressions/binary_ops.rs` or the visitor, detect `||` operator and generate ternary-like code instead.

### Task 3: sqrt() Missing Argument (1 error)

**Problem**: Expression `$val ? sqrt(2)**($val/256) : 0` generates `sqrt()` with no arguments.

**Affected file**: `hash_4d.rs`

**Current broken output**:
```rust
if val.is_truthy() {
    sqrt()  // <-- Missing argument!
} else {
    Into::<TagValue>::into(0i32)
}
```

**Root cause**: The power operator `**` parsing is consuming `sqrt(2)` incorrectly, treating `sqrt` as a standalone call instead of `sqrt(2)` as the base.

**Fix approach**: Debug with `cargo run --bin debug-ppi -- --verbose '$val ? sqrt(2)**($val/256) : 0'` to see how the AST is structured. The issue is likely in how `BinaryOperation` nodes with `**` handle function call children.

### Task 4: power() Argument Reference (1 error)

**Problem**: `power(base, val)` passes `&TagValue` where `TagValue` expected.

**Affected file**: `hash_cf.rs`

**Current broken output**:
```rust
(1i32 / (power(Into::<TagValue>::into(2i32), val)))
//                                           ^^^ &TagValue
```

**Fix approach**: Either:
1. Change `power()` signature to accept `&TagValue`
2. Add `.clone()` to `val` in the generated code when it's used as power exponent

### Task 5: Ok() Extra Argument (1 error)

**Problem**: Comma operator in Perl generates extra argument to `Ok()`.

**Affected file**: `hash_c9.rs`

**Current broken output**:
```rust
Ok(regex_substitute_perl(...), val)  // Ok takes 1 arg, not 2!
```

**Root cause**: Perl comma operator `(expr1, expr2)` is being interpreted as tuple construction. The last value in a comma sequence is the result in Perl.

**Fix approach**: Detect comma operators and only use the final value:
```rust
{
    let _ = regex_substitute_perl(...);
    Ok(val.clone())
}
```

### Task 6: Mixed-Type Array Issues (2 errors after regen)

**Problem**: `available_options.rs` and `rggb_lookup.rs` have integers in `[&'static str; N]` arrays.

**Note**: These files were deleted and will be regenerated by `make codegen`. If they still have the issue after regeneration, check `codegen/src/strategies/scalar_array.rs`:
- The `format_array_elements()` function should stringify integers when `element_type == "&'static str"`

### Task 7: unpack_binary Not Wrapped (1 error)

**Problem**: Some `unpack_binary()` calls are still not wrapped in `TagValue::Array()`.

**Affected file**: `hash_4c.rs`

**Root cause**: The visitor fix for `unpack` may not cover all code paths. Check if there's a different code path generating unpack calls.

### Verification After Fixes

```bash
rm -rf src/generated/functions
make codegen
cargo check           # Should have 0 errors
cargo t               # Tests should pass
make precommit        # Full validation
```

### Golden Rule Reminder

If a codegen pattern can't be fixed correctly, emit a **placeholder** instead:

```rust
// In the visitor, when encountering unsupported patterns:
return Err(CodeGenError::UnsupportedStructure(
    format!("Unsupported pattern: {}", description)
));
```

This triggers the fallback placeholder generator which produces compiling (though non-functional) code. A placeholder that compiles is infinitely better than broken syntax.

---

## Key Files Modified in Phase 2

```bash
# New/modified codegen helpers
codegen/src/ppi/rust_generator/expressions/binary_ops.rs
  - wrap_literal_for_tagvalue()
  - wrap_condition_for_bool()
  - wrap_branch_for_owned()
  - get_operator_precedence()
  - process_expression_recursively()

# Updated to use shared helpers
codegen/src/ppi/rust_generator/generator.rs
codegen/src/ppi/rust_generator/visitor.rs

# New TagValue method
codegen-runtime/src/tag_value/conversion.rs
  - is_truthy()
```

## Key Files Modified in Phase 3

```bash
# Type annotation fix (turbofish syntax)
codegen/src/ppi/rust_generator/expressions/binary_ops.rs
  - wrap_literal_for_tagvalue() → Into::<TagValue>::into()
  - wrap_branch_for_owned() → Into::<TagValue>::into()

# Return type fixes
codegen-runtime/src/string/transform.rs
  - chr() → returns TagValue
  - uc() → returns TagValue

# Join/unpack pattern handling
codegen/src/ppi/rust_generator/visitor.rs
  - join handler detects unpack child, generates join_unpack_binary()
  - unpack handler wraps in TagValue::Array()

codegen-runtime/src/data/mod.rs
  - New join_vec() function

codegen-runtime/src/lib.rs
  - Export join_vec

# Stub methods for expression evaluation
src/types/binary_data.rs
  - evaluate_context_condition() stub
  - evaluate_expression() stub

# Registry and signature fixes
src/registry.rs
  - Global wrappers pass None for ctx
  - Fixed temporary HashMap borrow

src/implementations/print_conv.rs
  - print_fraction() takes ctx parameter

# TagValue trait implementations
codegen-runtime/src/tag_value/ops.rs
  - PartialEq<f64> for TagValue and &TagValue
  - PartialOrd<f64> for TagValue and &TagValue
```

---

## Emergency Recovery

```bash
# If changes break more than they fix
git checkout HEAD -- codegen/src/
make codegen
cargo check 2>&1 | grep "^error" | wc -l  # Back to baseline

# Incremental approach
git stash
# Apply one change at a time
git stash pop
make codegen && cargo check
```

---

## Quality Checklist

### Phase 1-2 (Complete)
- [x] ExifError type unified across crates
- [x] PrintConv/ValueConv slot mismatches fixed (expression type in hash)
- [x] Lookup table mixed-type arrays fixed
- [x] Manual print_conv functions have ctx parameter
- [x] Manual value_conv functions have ctx parameter
- [x] Registry type aliases updated with ctx parameter
- [x] Fallback helper passes ctx to implementations
- [x] Visitor passes ctx to ExifTool function calls
- [x] TagValue has `is_truthy()` for Perl truthiness semantics
- [x] Ternary conditions wrapped with `.is_truthy()`
- [x] Ternary branches handle ownership (`.clone()`, `.into()`)
- [x] Power operator generates `power()` function call
- [x] Operator precedence handled for complex expressions
- [x] DRYed up helper functions between generator and visitor

### Phase 3 (Complete)
- [x] Type annotation ambiguity fixed with `Into::<TagValue>::into()` turbofish
- [x] `chr()` returns TagValue instead of String
- [x] `uc()` returns TagValue instead of String
- [x] `join unpack` pattern generates `join_unpack_binary()` directly
- [x] Standalone `unpack` wraps in `TagValue::Array()`
- [x] `join_vec()` function added for non-unpack joins
- [x] ExpressionEvaluator stub methods added (evaluate_context_condition, evaluate_expression)
- [x] Registry global wrappers pass `None` for ctx
- [x] `print_fraction()` accepts ctx parameter
- [x] Temporary HashMap borrow issues fixed
- [x] `PartialEq<f64>` and `PartialOrd<f64>` added for TagValue

### Phase 4 (Remaining - 16 errors)
- [ ] TagValue as bool conditions need `.is_truthy()` (6 errors)
- [ ] Perl `||` operator generates ternary-like code (3 errors)
- [ ] `sqrt()` argument preserved through power parsing (1 error)
- [ ] `power()` val argument cloned (1 error)
- [ ] Comma operator returns final value only (1 error)
- [ ] Mixed-type arrays regenerate correctly (2 errors after regen)
- [ ] All `unpack_binary()` calls wrapped (1 error)
- [ ] Error count reduced to 0
- [ ] `make precommit` passes
- [ ] No manual edits to `src/generated/`
