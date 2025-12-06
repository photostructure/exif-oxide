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

```bash
# Run to see current state
./scripts/capture.sh cargo check
grep "^error\[" /tmp/stderr_*.txt | sort | uniq -c | sort -rn
```

Current breakdown (as of Dec 2025, Phase 2):
- 23 `error[E0308]: mismatched types`
- 7 `error[E0283/E0284]: type annotations needed`
- 7 `error[E0061]: argument count mismatches`
- 3 `error[E0599]: method not found`
- 1 `error[E0277]: comparison type mismatch`

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

---

## Part 4: Remaining Work (41 errors)

### Task 1: Type Annotation Ambiguity (7 errors)

**Problem**: `.into()` calls on literals inside expressions create type inference ambiguity.

**Example** (hash_16.rs line 31):
```rust
Ok(100i32 * power(2i32.into(), ((16i32.into()) - (val / 256i32))))
//                              ^^^^^^^^^^^^^^ - type annotations needed
```

**Fix approach**:
Either use explicit type: `Into::<TagValue>::into(16i32)` or redesign `power()` to accept generic types.

**Files to check**: `codegen/src/ppi/rust_generator/expressions/binary_ops.rs` - the `wrap_literal_for_tagvalue()` function.

### Task 2: Function Argument Mismatches (7 errors)

**Problem**: Several generated function calls have wrong argument counts.

| Function | Issue |
|----------|-------|
| `sqrt()` | Called with 0 args, needs 1 |
| `chr()` | Returns String, expected TagValue |
| `unpack_binary()` | Returns `Vec<TagValue>`, sometimes expected String |
| Various methods | Take 3 args, called with 2 |

**Fix approach**:
- Fix `sqrt()` generation to pass `val` argument
- Wrap `chr()` return in `TagValue::String(...)`
- Handle `unpack_binary()` return type correctly (join or index)

**Files to check**:
- `codegen/src/ppi/rust_generator/visitor.rs` - function call generation
- `codegen-runtime/src/math/` - function signatures

### Task 3: Missing ExpressionEvaluator Methods (3 errors)

**Problem**: Generated code calls methods that don't exist:
- `evaluate_context_condition`
- `evaluate_expression`

**To investigate**:
```bash
grep -rn "evaluate_context_condition\|evaluate_expression" src/
```

**Fix approach**: Either add these methods to `ExpressionEvaluator` or update codegen to not generate these calls.

### Task 4: Mixed-Type Array Issues (2 errors)

**Problem**: `available_options.rs` and `rggb_lookup.rs` still have integers in `[&'static str; N]` arrays.

**Files to check**: `codegen/src/strategies/scalar_array.rs` - the type detection and element formatting may need refinement.

### Task 5: Verify Full Build

After fixing remaining issues:
```bash
rm -rf src/generated/functions
make codegen
cargo check           # Should have 0 errors
cargo t               # Tests should pass
make precommit        # Full validation
```

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
- [ ] Type annotation ambiguity for `.into()` in expressions
- [ ] Function argument counts correct (sqrt, chr, etc.)
- [ ] ExpressionEvaluator methods exist or calls removed
- [ ] Error count reduced to 0
- [ ] `make precommit` passes
- [ ] No manual edits to `src/generated/`
