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
**After**: 117 compilation errors (58% reduction)

```bash
# Run to see current state
./scripts/capture.sh cargo check
grep "^error\[" /tmp/stderr_*.txt | sort | uniq -c | sort -rn
```

Current breakdown:
- 100 `error[E0308]: mismatched types`
- 4 `error[E0614]: type TagValue cannot be dereferenced`
- 3 `error[E0061]: this method takes 3 arguments but 2 arguments were supplied`
- Various other minor errors

---

## Part 4: Remaining Work

### Task 1: Fix Generated Expression Type Issues

**Problem**: The PPI code generator produces expressions with wrong types.

**Error patterns** (run `grep -E "expected.*found" /tmp/stderr_*.txt | sort | uniq -c | sort -rn`):

| Count | Error Pattern | Root Cause |
|-------|--------------|------------|
| ~28 | `expected TagValue, found &TagValue` | Return statements missing `.clone()` |
| ~25 | `expected TagValue, found i32` | Integer literals not wrapped in `TagValue::I32()` |
| ~18 | `expected bool, found &TagValue/TagValue` | Conditions using `val` where bool expected |

**Where to fix**: `codegen/src/ppi/rust_generator/`

**Specific files to investigate**:
```bash
# Find where return statements are generated
rg "Ok\(val\)|val\.clone\(\)" codegen/src/ppi/rust_generator/

# Find where integer literals are generated
rg "i32\)" codegen/src/ppi/rust_generator/

# Find where conditions are generated
rg "if.*val" codegen/src/ppi/rust_generator/
```

**Fix approach**:
1. In `generator.rs` around line 100-130, check the return wrapping logic
2. Integer literals need `TagValue::I32(N)` wrapper, not bare `Ni32`
3. Boolean conditions need proper conversion: `val.as_bool()` or similar

### Task 2: Fix Dereference Errors

**Problem**: 4 errors of `type TagValue cannot be dereferenced`

**To investigate**:
```bash
grep -B 5 "cannot be dereferenced" /tmp/stderr_*.txt
```

Likely cause: Generated code tries to dereference TagValue with `*val` somewhere.

### Task 3: Fix Method Call Errors

**Problem**: `evaluate_context_condition` and `evaluate_expression` methods not found

**To investigate**:
```bash
grep -B 5 "evaluate_context_condition\|evaluate_expression" /tmp/stderr_*.txt
```

These methods may need to be added to `ExpressionEvaluator` or the calls need updating.

### Task 4: Verify Full Build

After fixing remaining issues:
```bash
rm -rf src/generated/functions
make codegen
cargo check           # Should have 0 errors
cargo t               # Tests should pass
make precommit        # Full validation
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
- [ ] Generated expressions return correct types (TagValue vs i32)
- [ ] Generated expressions handle references correctly (&TagValue vs TagValue)
- [ ] Generated conditions produce bool (not TagValue)
- [ ] Error count reduced to 0
- [ ] `make precommit` passes
- [ ] No manual edits to `src/generated/`
