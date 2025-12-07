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
| 4 (current) | 16 | ~10 | || operator, sqrt(), context lookup booleans |

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

## Remaining Work

### Current Error State (after Phase 4 changes)

```bash
cargo check 2>&1 | grep "^error\[" | sort | uniq -c | sort -rn
```

Approximate breakdown:
- ~6 `E0308` mismatched types (sprintf tuple, unpack wrapping, power reference)
- ~3 `E0425` canon_ev not found
- ~1 `E0061` Ok() extra argument

### Task 1: sprintf Tuple Arguments

**Problem**: Comma inside sprintf args creates tuple instead of separate args.

**Example**: `sprintf("%.1f%%", $val * 100)` generates:
```rust
sprintf_perl((Into::<TagValue>::into("%.1f%%"), val * 100i32))  // WRONG: tuple
```

**Should be**:
```rust
sprintf_perl("%.1f%%", &[val * 100i32])  // Correct: separate args
```

**Root cause**: The comma inside ternary branch processing is being treated as a binary comma operator.

**Fix approach**: In the ternary normalization, detect when true/false branches contain function calls with comma-separated arguments and don't process those commas as operators.

**Files**: `expression_precedence.rs` - `parse_ternary_with_precedence()` or `parse_expression_sequence()`

### Task 2: power() Argument Reference

**Problem**: `power(base, val)` passes `&TagValue` where `TagValue` expected.

**File**: `hash_cf.rs`
```rust
(1i32 / (power(Into::<TagValue>::into(2i32), val)))  // val is &TagValue
```

**Fix options**:
1. Change `power()` in codegen-runtime to accept `&TagValue`
2. Generate `val.clone()` when used as power exponent

### Task 3: Ok() Extra Argument (Comma Operator)

**Problem**: Perl comma operator `($expr1, $expr2)` results in extra arg.

**File**: `hash_c9.rs`
```rust
Ok(regex_substitute_perl(...), val)  // Ok takes 1 arg, not 2!
```

**Fix**: Detect comma operators at statement level and wrap in block:
```rust
{
    let _ = regex_substitute_perl(...);
    Ok(val.clone())
}
```

### Task 4: unpack_binary Wrapping

**Problem**: Some `unpack_binary()` calls not wrapped in `TagValue::Array()`.

**File**: `hash_4c.rs`

**Fix**: Check all code paths generating unpack calls in `visitor.rs`.

### Task 5: canon_ev Function

**Problem**: `crate::implementations::canon::canon_ev` not found.

**Options**:
1. Implement the function in `src/implementations/canon.rs`
2. Remove from impl_registry so it falls back to placeholder

---

## Verification

```bash
rm -rf src/generated/functions
make codegen
cargo check           # Should have 0 errors
cargo t               # Tests should pass
make precommit        # Full validation
```

---

## Key Files Modified in Phase 4

```bash
# Boolean detection improvements
codegen/src/ppi/rust_generator/expressions/binary_ops.rs
  - wrap_condition_for_bool() - added ctx.and_then detection
  - is_boolean_expression() - added .contains(), .is_match(), ! patterns
  - Made is_boolean_expression() public

# Perl || operator fix
codegen/src/ppi/rust_generator/visitor.rs
  - Lines 1680-1702: || generates ternary-like code
  - Lines 613-630: Unknown functions trigger fallback

# Ternary with function calls
codegen/src/ppi/normalizer/passes/expression_precedence.rs
  - Lines 223-231: should_process() allows ternary with Structure::List
  - Lines 759-765: Any Word+List is function call

# Manual fix
src/implementations/raw_conv.rs
  - convert_exif_text() takes ctx parameter
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

### Remaining
- [ ] sprintf tuple arguments fixed
- [ ] `power()` val argument cloned
- [ ] Comma operator returns final value only
- [ ] All `unpack_binary()` calls wrapped
- [ ] canon_ev function implemented or removed from registry
- [ ] Error count reduced to 0
- [ ] `make precommit` passes

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

1. **The sprintf tuple issue is the most impactful** - it affects multiple files. Fix this first.

2. **Be careful with ternary processing** - the Phase 4 changes made ternary processing more aggressive, which fixed sqrt() but may cause edge cases with function arguments.

3. **Test incrementally**: After each fix, run `make codegen && cargo check` before moving to the next.

4. **When in doubt, fallback**: If a pattern is too complex, return `UnsupportedStructure` to generate a placeholder.
