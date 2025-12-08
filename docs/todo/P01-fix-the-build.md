# P01: Fix the Build

**Prerequisites**: Read [GETTING-STARTED.md](../GETTING-STARTED.md) first.

## Success Criteria

```bash
make precommit  # MUST pass with zero errors
```

**Key constraint**: Generated functions that can't be fixed should emit placeholders, not broken syntax.

---

## Key Files

```bash
# Where signatures are generated (what to FIX)
codegen/src/ppi/rust_generator/visitor.rs        # Main code generation
codegen/src/ppi/rust_generator/expressions/      # Expression handling
codegen/src/ppi/normalizer/passes/               # AST normalization

# Generated output (NEVER edit directly)
src/generated/functions/    # PPI-generated functions
```

## Golden Rule

If codegen can't generate valid Rust, throw `CodeGenError::UnsupportedStructure`:

```rust
return Err(CodeGenError::UnsupportedStructure(
    format!("Unsupported pattern: {}", description)
));
```

This triggers fallback placeholder generation. **A placeholder that compiles is infinitely better than broken syntax.**

---

## Progress Summary

| Phase | Errors Before | Errors After | Key Fixes |
|-------|---------------|--------------|-----------|
| 1 | 280+ | 117 | ExifError unification, ctx parameters |
| 2 | 117 | 41 | is_truthy(), ternary handling, power operator |
| 3 | 41 | 16 | turbofish syntax, chr/uc return types, join/unpack |
| 4 | 16 | ~10 | || operator, sqrt(), context lookup booleans |
| 5 | 10 | **0** | sprintf tuple fix, canon_ev, division types, fallbacks |
| 6 | **Test suite** | **0 compile errors** | ctx parameters in tests, i64 literals, RustGenerator export |
| 7 | 6 | **0** | Binary ops with function calls, string concat literal wrapping |

## ✅ BUILD FIXED - Phase 7 Complete

**Status**: `cargo check` and `cargo test --no-run` both pass with **0 errors**.

Note: Pre-existing issues remain unrelated to the build fix (clippy warnings ~400, one cli_tag_filtering test failure).

---

## Phase 7: Binary Operations with Function Calls (6 errors → 0)

### 1. Expression Normalizer Not Processing Binary Ops with Function Calls (6 errors → 0)
**Problem**: Expressions like `100 * exp(($val-32)*log(2)/8)` generated `100i32 * exp (...)` without the `codegen_runtime::` prefix, causing type mismatches (`i32` returned instead of `TagValue`).

**Root cause**: The `should_process()` function in `ExpressionPrecedenceNormalizer` returned early when detecting a `Structure::List` if the first child wasn't a known function. For expressions like `100 * exp(...)`, the first child is `Number(100)`, not a function name, so the normalizer skipped processing.

**Fix**: Updated `expression_precedence.rs:234-243` to also check for binary operators when `Structure::List` is present:
```rust
if has_structure_list {
    // ... ternary check ...

    // Always process binary operations that contain function calls
    // e.g., 100 * exp(...), $val / log(2), etc.
    if self.has_binary_operators(&node.children) {
        return true;
    }

    // ... rest of function call check
}
```

### 2. String Concatenation Missing TagValue Wrapper (1 error → 0)
**Problem**: Expression `"0x" . unpack("H*",$val)` generated `codegen_runtime::string::concat(&"0x", ...)` where `&"0x"` is `&&str`, not `&TagValue`.

**Fix**: Added `wrap_for_string_concat()` function in `binary_ops.rs:43-61`:
```rust
pub fn wrap_for_string_concat(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.starts_with('"') && trimmed.ends_with('"') {
        format!("TagValue::string({})", trimmed)
    } else if trimmed.ends_with("i32") || trimmed.ends_with("f64") || trimmed.ends_with("u32") {
        format!("Into::<TagValue>::into({trimmed})")
    } else {
        s.to_string()
    }
}
```

Updated concat operator handling in:
- `binary_ops.rs:199-205` - use `wrap_for_string_concat()` for both operands
- `visitor.rs:1634-1638` - same fix for BinaryOperation nodes

### 3. Missing log() Prefix Fix
**Problem**: The `try_log_pattern()` in `patterns.rs:63` was generating `log({var})` without `codegen_runtime::` prefix.

**Fix**: Updated to `format!("codegen_runtime::log({var})")`.

---

## Phase 6: Test Suite Compilation Fixes

### 1. ctx Parameter for Test Functions (~76 errors → 0)
**Problem**: print_conv/value_conv functions added `ctx: Option<&ExifContext>` parameter, but tests didn't pass it.

**Files Fixed**:
- `src/implementations/print_conv.rs` - test module: Added `None` as second arg
- `src/implementations/value_conv.rs` - test module: Added `None` as second arg
- `src/registry.rs` - test module: Updated function signatures and calls
- `tests/value_conv_tests.rs` - Added `None` to all function calls
- `tests/exposuretime_printconv_test.rs` - Added `None` and fixed `.into()` → `TagValue::string()`
- `tests/process_binary_data_tests.rs` - Fixed `.into()` → `TagValue::string()`
- `tests/integration_tests.rs` - Updated test_converter signature and imports

### 2. Large Integer Literals (2 errors → 0)
**Problem**: `4294967296i32` and `4294965247i32` overflow i32 range.

**Fix**: Updated `codegen/src/ppi/rust_generator/visitor.rs:226-233`:
```rust
// Check if number fits in i32 range before using i32 suffix
let num: i64 = raw_number.parse().unwrap_or(0);
if num >= i32::MIN as i64 && num <= i32::MAX as i64 {
    Ok(format!("{}i32", raw_number))
} else {
    Ok(format!("{}i64", raw_number))
}
```

**Runtime Support**: Added i64 operators to `codegen-runtime/src/tag_value/ops.rs`:
- Added `impl Add/Sub/Mul/Div<i64> for TagValue` and `&TagValue`
- Added `impl PartialEq<i64> for TagValue` and `&TagValue`

### 3. RustGenerator Export (3 errors → 0)
**Problem**: `RustGenerator` not exported from `codegen::ppi` module.

**Fix**: Added to `codegen/src/ppi/mod.rs`:
```rust
pub use rust_generator::RustGenerator;
```

### 4. finish_extraction() Context Parameter (1 error → 0)
**Problem**: `tests/integration_p07b_scalar_arrays.rs` called `finish_extraction()` without context.

**Fix**: Changed to `finish_extraction(&mut context)`.

---

## Phase 4: Current Session Progress

### Completed

#### 1. TagValue as Bool Conditions (6 errors → 0)
**Problem**: `ctx.and_then(...).unwrap_or(TagValue::U32(1))` used directly in `if` without `.is_truthy()`.

**Fix**: Updated `wrap_condition_for_bool()` in `binary_ops.rs:76-79`:
```rust
// Context lookups ($$self{Field}) return TagValue, need is_truthy()
if condition.contains("ctx.and_then") || condition.contains("get_data_member") {
    return format!("({}).is_truthy()", condition);
}
```

#### 2. Perl || Operator (3 errors → 0)
**Problem**: `$val || "inf"` generated `(val || Into::<TagValue>::into("inf"))` - Rust `||` expects bools.

**Fix**: Updated `visitor.rs:1695-1702` to generate ternary-like code:
```rust
"||" => {
    // Perl || returns first truthy value or last value
    Ok(format!(
        "if ({}).is_truthy() {{ {}.clone() }} else {{ {} }}",
        left, left, wrap_branch_for_owned(&right)
    ))
}
```

#### 3. sqrt() Missing Argument (1 error → 0)
**Problem**: `$val ? sqrt(2)**($val/256) : 0` generated `sqrt()` without args.

**Root cause**: `should_process()` in ExpressionPrecedenceNormalizer skipped ternary expressions with `PPI::Structure::List` when first child wasn't a known function.

**Fix**: Updated `expression_precedence.rs:223-231`:
```rust
if has_structure_list {
    // Always process ternary expressions, even if they contain function calls
    if self.has_ternary_pattern(&node.children) {
        return true;
    }
    // ... rest of function call check
}
```

#### 4. Extended Boolean Expression Detection
**Fix**: Added more patterns to `is_boolean_expression()` in `binary_ops.rs:52-55`:
```rust
|| s.contains(".contains(")
|| s.contains(".is_match(")
|| s.starts_with('!')      // Negation: !expr
|| s.starts_with("(!")     // Negation in parens: (!expr)
```

#### 5. Function Call Recognition
**Fix**: In `expression_precedence.rs:759-765`, any Word followed by List is now treated as a function call, not just known functions. This handles ExifTool-specific functions like `ConvertDuration($val)`.

#### 6. Unknown Function Fallback
**Fix**: Updated `visitor.rs` to throw `UnsupportedStructure` for unknown functions instead of generating invalid code:
- Lines 613-620: ExifTool namespace functions not in registry
- Lines 624-630: Generic unknown functions

---

## Verification

```bash
rm -rf src/generated/functions
make codegen
cargo check           # Should have 0 errors
cargo t               # Tests should pass (22 pre-existing failures)
make precommit        # Full validation
```

---

## Quality Checklist

### Completed (Phases 1-4)
- [x] ExifError type unified across crates
- [x] PrintConv/ValueConv slot mismatches fixed
- [x] Manual print_conv/value_conv functions have ctx parameter
- [x] Registry type aliases updated with ctx parameter
- [x] TagValue has `is_truthy()` for Perl truthiness semantics
- [x] Ternary conditions wrapped with `.is_truthy()`
- [x] Power operator generates `power()` function call
- [x] Type annotation ambiguity fixed with turbofish syntax
- [x] `chr()` and `uc()` return TagValue
- [x] `join unpack` pattern generates `join_unpack_binary()`
- [x] Perl `||` operator generates ternary-like code
- [x] Context lookups wrapped with `.is_truthy()`
- [x] Unknown functions trigger fallback placeholders

### Phase 6 Additions (Test Suite)
- [x] Test functions updated with ctx parameter
- [x] Large integer literals use i64 suffix when needed
- [x] i64 operators added to codegen-runtime
- [x] RustGenerator exported from codegen::ppi
- [x] Test compilation passes (0 errors)

### Phase 7 Additions (Binary Ops with Function Calls)
- [x] Binary operations with function calls processed by normalizer
- [x] `100 * exp(...)` pattern generates correct `FunctionCall` nodes
- [x] String concatenation wraps literals in `TagValue::string()`
- [x] `log()` pattern uses `codegen_runtime::` prefix

### Build Status
- [x] `cargo check` passes with 0 errors
- [x] `cargo test --no-run` passes with 0 errors
- [ ] `make precommit` full validation (~400 clippy warnings, P03 task)

---

## Remaining Work (Future Tasks)

### P02: Fix Runtime Test Failures (22 tests)

The following tests fail at runtime due to missing lookup table registrations (not compilation issues):

**IPTC Tests (9 failures)**:
- `formats::iptc::tests::*` - Tests expect IPTC parsing that may not be fully implemented

**MinoltaRaw Tests (6 failures)**:
- `implementations::minolta_raw::tests::test_prd_*` - StorageMethod, BayerPattern lookups not registered
- `implementations::minolta_raw::tests::test_rif_*` - ProgramMode, ZoneMatching lookups use wrong approach

**PanasonicRaw Tests (5 failures)**:
- `implementations::panasonic_raw::tests::*` - Similar lookup registration issues

**BinaryData Tests (2 failures)**:
- `types::binary_data::tests::test_conditional_array_exposure_time` - Conditional array processing

### P03: Clippy Warning Cleanup (~400 warnings)

Run `cargo clippy` to identify and fix:
- Static variable naming (REGEX identifiers should be UPPER_SNAKE_CASE)
- Unused variables and imports
- Redundant clones
- Other Clippy lints

**Approach**:
```bash
cargo clippy --all-targets 2>&1 | grep "warning:" | sort | uniq -c | sort -rn
```

Priority fixes:
1. Generated code warnings (fix in codegen templates)
2. Test code warnings (quick manual fixes)
3. Library code warnings (careful review needed)

---

## Emergency Recovery

```bash
# If changes break more than they fix
git checkout HEAD -- codegen/src/
make codegen
cargo check 2>&1 | grep "^error" | wc -l  # Back to baseline
```

---

## Notes for Next Engineer

### Build is FIXED ✅

The compilation issues from P01 are resolved. `cargo check` and `cargo test --no-run` both pass with 0 errors.

### Next Steps

1. **P02: Runtime Test Failures** - 22 tests fail at runtime due to missing/incorrect lookup table registrations. These are logic bugs, not compilation issues.

2. **P03: Clippy Warnings** - ~400 warnings from generated code and tests. Most are:
   - REGEX variable naming (should use UPPER_SNAKE_CASE)
   - Unused variables in generated functions
   - Type inference suggestions

3. **Test incrementally**: After any fix, run `cargo t` to ensure no regressions.

4. **When in doubt, fallback**: If a pattern is too complex in codegen, return `UnsupportedStructure` to generate a placeholder.
