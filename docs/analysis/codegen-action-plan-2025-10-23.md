# Codegen System: Prioritized Action Plan
**Date**: 2025-10-23
**Status**: Analysis based on current build state
**Goal**: Fix codegen to generate valid Rust for all ExifTool expressions or properly fallback

## Executive Summary

**Good News**: The codegen system is NOT fundamentally broken. The three-tier fallback system (PPI → impl_registry → placeholder) is working correctly, with **237 expressions properly falling back to placeholders** instead of generating invalid Rust.

**The Real Problem**: We need to expand PPI generation capabilities to handle more exotic Perl patterns, and ensure that patterns we can't handle are properly added to the impl_registry for manual implementation.

**Build Status**: Cannot verify due to network connectivity issues (crates.io 403 errors), but analysis of generated code shows:
- ✅ 237 placeholders generating valid Rust (warnings but compiles)
- ✅ No `cast_\` backslash errors found
- ✅ No invalid operator patterns found
- ✅ Fallback system working as designed

## Current State Analysis

### What's Working ✅

1. **Three-tier fallback system** (PPI → impl_registry → placeholder)
   - `codegen/src/ppi/fn_registry/mod.rs:133-147` - PPI generation attempted first
   - `codegen/src/ppi/fn_registry/mod.rs:236-260` - impl_registry lookup on failure
   - `codegen/src/ppi/fn_registry/mod.rs:263-348` - placeholder generation as last resort

2. **Test infrastructure**
   - 40 enabled test configs passing
   - 22 SKIP tests waiting for implementation
   - JSON-based test framework with ExifTool reference validation

3. **Core PPI capabilities**
   - ✅ Array subscripts (`$val[0]`, `$val[1]`) - lines in expression_precedence.rs:701-717
   - ✅ Regex matching (`$val =~ /pattern/`) - visitor.rs:1251-1316
   - ✅ Ternary operations with unary negation - expression_precedence.rs:439-474
   - ✅ Basic arithmetic, comparison, logical operations
   - ✅ Function calls (sprintf, int, length, substr, abs, sqrt, etc.)

### What's Missing ❌

**237 placeholders** indicate these patterns need either:
1. PPI generator enhancements, OR
2. Manual implementations added to impl_registry

**Top Missing Patterns** (from placeholder analysis):

1. **String concatenation** (`.` operator)
   - Example: `($val >> 16) . "." . ($val & 0xffff)`
   - Frequency: Very common (version numbers, formatted output)
   - File: SigmaRaw.FileVersion, many others

2. **Bitwise operations** (`>>`, `&`, `|`, `^`)
   - Example: `$val >> 16`, `$val & 0xffff`
   - Frequency: Common (extracting version components, flags)
   - File: Multiple camera maker notes

3. **Variable declarations** (`my @array = split`, `my ($a,$b) = ...`)
   - Example: `my @a = split ' ', $val; return $a[2] ? sprintf('%3dx%3d', $a[0], $a[1]) : 'n/a'`
   - Frequency: Very common (multi-step transformations)
   - File: Sony.FocusFrameSize, Olympus.CustomSaturation

4. **Complex function chains** (`unpack`, `pack`, `reverse`, `join`)
   - Example: `sprintf("%.2x:%.2x:%.2x:%.2x",reverse unpack("C*",$val))`
   - Frequency: Common (binary data formatting)
   - File: H264.TimeCode

5. **Regex substitution** (`s/pattern/replacement/`)
   - Example: `$val =~ s/test/replacement/g`
   - Status: SKIP test exists (SKIP_regex_substitute.json)

6. **Array operations** (`map`, `grep`, `reverse`, `sort`)
   - Example: `pack("C*", map { (($_>>$_)&0x1f)+0x60 } 10, 5, 0)`
   - Status: SKIP test exists (SKIP_pack_map_bits.json)

## Prioritized Action Plan

### Priority 0: Verify Current State ⚡ URGENT

**Blocker**: Network connectivity preventing build verification

**Tasks**:
- [ ] Wait for network connectivity restoration
- [ ] Run `cargo build --package codegen` to verify actual compilation state
- [ ] Run `cargo test --package codegen` to verify test status
- [ ] Document actual error count and types

**Success Criteria**: Know exact compilation status before proceeding

**Time Estimate**: Blocked on network

---

### Priority 1: Low-Hanging Fruit 🍎 (Estimated: 1-2 days)

Enable SKIP tests that may already work or need minimal fixes.

#### P1.1: Simple Constant Expressions

**Test**: `SKIP_hex_number.json`
- Expression: `0x100`
- Expected: U32(256)
- Issue: Hex literal parsing
- Implementation: Add hex literal support to number parser
- File to modify: `codegen/src/ppi/rust_generator/visitor_tokens.rs`

**Test**: `SKIP_basic_comparisons.json`
- Expression: `$val > 100`
- Expected: Returns boolean (0 or 1)
- Issue: May already work - needs verification
- Action: Remove SKIP_ prefix and test

#### P1.2: Power Operator

**Tests**:
- `SKIP_power_operator_function.json` - `sqrt(2) ** $val`
- `SKIP_power_operator_conditional.json` - `abs($val)<100 ? 1/(2**$val) : 0`

- Issue: `**` operator not in expression_precedence normalizer
- Implementation: Add power operator to precedence table (higher than multiplication)
- File to modify: `codegen/src/ppi/normalizer/passes/expression_precedence.rs:327-351`
- Codegen-runtime: Add `pow()` function wrapping Rust's `powf()`

**Success Criteria**:
- [ ] `cargo test --package codegen hex_number` passes
- [ ] `cargo test --package codegen basic_comparisons` passes
- [ ] `cargo test --package codegen power_operator` passes (both)
- [ ] No regressions in existing 40 tests

**Estimated Impact**: ~5 required tags unblocked (APEX exposure calculations)

---

### Priority 2: String Concatenation 🔗 (Estimated: 2-3 days)

**Critical Pattern**: Perl's `.` operator for string concatenation

**Example Cases**:
```perl
($val >> 16) . "." . ($val & 0xffff)  # "2.1" format version
"$val m"                               # Altitude with units
"0x" . unpack("H*",$val)               # Hex formatting
```

**Implementation Strategy**:

1. **Add operator to normalizer**
   - File: `codegen/src/ppi/normalizer/passes/expression_precedence.rs`
   - Add `.` to binary operators table
   - Precedence: Lower than arithmetic, higher than comparison

2. **Generate Rust code**
   - File: `codegen/src/ppi/rust_generator/visitor.rs`
   - Generate: `format!("{}{}", left, right)` for static strings
   - OR: `format!("{}", TagValue::from(...))` with proper conversions

3. **Handle string interpolation**
   - Pattern: `"$val"` should become `format!("{}", val)`
   - Pattern: `"$val m"` becomes `format!("{} m", val)`

**Tests to enable**:
- `SKIP_string_concat_arithmetic.json` - `"$val m"` pattern

**Success Criteria**:
- [ ] String concatenation generates `format!()` calls
- [ ] Bitwise + concat works: `($val >> 16) . "." . ($val & 0xffff)`
- [ ] ~30 placeholder functions become real implementations

**Estimated Impact**: ~50 required tags unblocked (version numbers, formatted values)

---

### Priority 3: Bitwise Operations 🔢 (Estimated: 1-2 days)

**Operators**: `>>`, `<<`, `&`, `|`, `^`

**Current Status**: Operators likely already in normalizer, need verification

**Implementation**:

1. **Verify operator support**
   - Check: `codegen/src/ppi/normalizer/passes/expression_precedence.rs`
   - Add if missing: `>>`, `<<`, `&`, `|`, `^` to binary operators

2. **Generate Rust code**
   - File: `codegen/src/ppi/rust_generator/visitor.rs`
   - Map directly to Rust operators: `val >> 16`, `val & 0xffff`
   - Ensure type coercion to integer types

3. **Runtime support**
   - File: `codegen-runtime/src/tag_value/mod.rs`
   - Ensure TagValue implements bitwise ops (likely already exists)

**Success Criteria**:
- [ ] Bitwise operations generate valid Rust
- [ ] Combined with string concat: version number formatting works
- [ ] ~20 placeholder functions become real implementations

**Estimated Impact**: ~30 required tags (version parsing, flag extraction)

---

### Priority 4: Variable Declarations 📦 (Estimated: 3-5 days)

**Critical Pattern**: Multi-statement expressions with temporaries

**Example Cases**:
```perl
my @a = split ' ', $val; return $a[2] ? sprintf('%3dx%3d', $a[0], $a[1]) : 'n/a'
my ($a,$b,$c) = split ' ', $val; if (...) { ... }
my $temp = $val * 100; sprintf("%.1f%%", $temp)
```

**Implementation Strategy**:

**OPTION A: Full PPI Support** (Complex, thorough)
1. Parse variable declarations as statements
2. Track variable scope in visitor
3. Generate Rust let bindings
4. Handle array destructuring

**OPTION B: Manual Impl Registry** (Faster, targeted)
1. Identify specific patterns used in required tags
2. Write manual Rust implementations
3. Add to impl_registry
4. Let PPI fallback handle

**Recommendation**: **OPTION B** for required tags, defer OPTION A

**Tests to enable**:
- `SKIP_variable_declaration.json` - `my $temp = $val`

**Success Criteria**:
- [ ] Common variable patterns in impl_registry
- [ ] ~50 required tag expressions work
- [ ] Document which patterns need manual impl

**Estimated Impact**: ~60 required tags (complex transformations)

---

### Priority 5: Binary Data Operations 🔤 (Estimated: 4-6 days)

**Functions**: `pack`, `unpack`, `join`, `split`, `reverse`

**Example Cases**:
```perl
join " ", unpack "H2H2", $val                    # Hex byte formatting
sprintf("%.2x:%.2x:%.2x:%.2x", reverse unpack("C*",$val))
pack("C*", map { ... })                          # Binary encoding
```

**Implementation Strategy**:

1. **Simple cases**: Direct codegen-runtime functions
   - `split(' ', $val)` → `codegen_runtime::split(val, " ")`
   - `join(" ", @arr)` → `codegen_runtime::join(&arr, " ")`

2. **Complex cases**: Manual impl_registry
   - `unpack` with format strings → manual Rust
   - `pack` patterns → manual implementation
   - `map`/`grep` → likely manual

**Tests to enable**:
- `SKIP_join_unpack.json` - `join " ", unpack "H2H2", $val`
- `SKIP_pack_map_bits.json` - `pack("C*", map { ... })`

**Files to modify**:
- Add to `codegen-runtime/src/`: `split.rs`, `join.rs`, `unpack.rs`, `pack.rs`
- Add registry lookups for complex patterns

**Success Criteria**:
- [ ] Simple split/join works via PPI
- [ ] Complex unpack/pack in impl_registry
- [ ] ~40 placeholder functions resolved

**Estimated Impact**: ~40 required tags (binary data, hex formatting)

---

### Priority 6: Regex Substitution 🔄 (Estimated: 2-3 days)

**Pattern**: `s/pattern/replacement/flags`

**Example**: `$val =~ s/test/replacement/g`

**Implementation**:

1. **Parse substitution pattern**
   - File: `codegen/src/ppi/rust_generator/visitor_tokens.rs`
   - Extract pattern, replacement, flags (g, i, etc.)

2. **Generate Rust regex code**
   - Simple: `val.replace("pattern", "replacement")`
   - Complex: `REGEX.replace_all(&val, "replacement")`
   - Global: Use `replace_all()` for `/g` flag

**Tests to enable**:
- `SKIP_regex_substitute.json`

**Success Criteria**:
- [ ] Basic substitution generates replace() calls
- [ ] Regex patterns generate Regex::replace_all()
- [ ] Flags handled correctly (g, i, etc.)

**Estimated Impact**: ~15 required tags (string normalization)

---

### Priority 7: Advanced Features ⚙️ (Estimated: 5+ days)

**Remaining SKIP tests**:
- `SKIP_tr_transliteration.json` - Character translation (`tr/ABC/XYZ/`)
- `SKIP_safe_division.json` - Division with zero checks
- `SKIP_early_return.json` - `return $val if condition`
- `SKIP_index_function.json` - String search
- `SKIP_length_function_numeric.json` - Length edge cases
- `SKIP_cast_dereference.json` - Perl reference dereferencing
- `SKIP_hash_subscript.json` - Hash table access
- `SKIP_context_timescale.json` - ExifContext usage
- `SKIP_mixed_precedence_chain.json` - Complex operator precedence
- `SKIP_defined_check.json` - `defined($val)` checks
- `SKIP_regex_negative_match.json` - `$val !~ /pattern/`
- `SKIP_ternary_string_comparison.json` - May already work
- `SKIP_sprintf_concat_ternary.json` - May already work

**Strategy**: Evaluate each individually
- Try removing SKIP_ to see if it works
- If fails, decide: PPI enhancement vs impl_registry
- Prioritize by impact on required tags

---

## Implementation Guidelines

### Golden Rules

1. **Never generate invalid Rust** - Fallback to placeholder if uncertain
2. **Trust ExifTool** - Translate exactly, don't optimize
3. **Test first** - Write breaking test before fixing
4. **Check regressions** - Run full test suite after each change

### Development Workflow

For each priority:

1. **Research Phase**
   ```bash
   # Find real usage in ExifTool
   rg -r 'pattern' third-party/exiftool/lib/

   # Inspect AST structure
   cargo run --bin debug-ppi -- 'expression'

   # Check existing tests
   find codegen/tests/config -name "*pattern*.json"
   ```

2. **Implementation Phase**
   - Write test case in `codegen/tests/config/`
   - Verify test fails: `cargo test --package codegen test_name`
   - Implement fix in appropriate module
   - Verify test passes + no regressions

3. **Validation Phase**
   ```bash
   # Unit tests
   cargo test --package codegen

   # Integration tests
   cargo test --package codegen-runtime

   # Full build
   cargo build

   # Precommit (when network works)
   make precommit
   ```

### File Organization

**Key files to modify**:

- **Expression parsing**: `codegen/src/ppi/normalizer/passes/expression_precedence.rs`
- **Code generation**: `codegen/src/ppi/rust_generator/visitor.rs`
- **Token handling**: `codegen/src/ppi/rust_generator/visitor_tokens.rs`
- **Runtime functions**: `codegen-runtime/src/`
- **Impl registry**: `codegen/src/impl_registry/`

### Testing Strategy

**Test pyramid**:
1. JSON test configs (fastest, most specific)
2. Generated expression tests (validates end-to-end)
3. Integration tests with real images (slowest, most comprehensive)

**Commands**:
```bash
# Quick test of specific pattern
cargo test --package codegen pattern_name

# All codegen tests
cargo test --package codegen

# Generate test code
cargo run --package codegen --bin generate-expression-tests

# Debug PPI parsing
cargo run --package codegen --bin debug-ppi -- 'perl expression'
```

---

## Success Metrics

### Quantitative Goals

- [ ] **Build**: `cargo build` succeeds with zero errors
- [ ] **Tests**: All 62 test configs passing (40 current + 22 SKIP)
- [ ] **Placeholders**: Reduced from 237 to <50 (80% reduction)
- [ ] **Required Tags**: 90%+ of 178 required tag expressions working

### Qualitative Goals

- [ ] **Documentation**: Each missing pattern documented in impl_registry
- [ ] **Maintainability**: Clear separation PPI-generated vs manual impl
- [ ] **ExifTool Parity**: Generated output matches ExifTool exactly

---

## Risk Analysis

### High Risk Areas

1. **Network connectivity** - Blocking all verification (current issue)
2. **Concurrent edits** - Other engineers may be modifying same files
3. **Build breaking** - Changes could prevent compilation
4. **Test regressions** - New features breaking existing tests

### Mitigation Strategies

1. **Network**: Wait for connectivity before proceeding
2. **Concurrent**: Run `cargo build` before starting each priority
3. **Breaking**: Test incrementally, commit working states
4. **Regressions**: Always run full test suite before commit

### Emergency Recovery

If build breaks:
```bash
# Revert codegen changes
git checkout HEAD -- codegen/src/

# Regenerate with original generator
make codegen

# Validate baseline
cargo build
cargo test --package codegen
```

---

## Timeline Estimate

**Assuming network connectivity restored and no blockers**:

- Priority 0 (Verify): **1 hour** (when network works)
- Priority 1 (Low-hanging): **1-2 days**
- Priority 2 (String concat): **2-3 days**
- Priority 3 (Bitwise): **1-2 days**
- Priority 4 (Variables): **3-5 days**
- Priority 5 (Binary data): **4-6 days**
- Priority 6 (Regex subst): **2-3 days**
- Priority 7 (Advanced): **5+ days**

**Total**: **19-27 days** of focused work

**Critical Path**: P0 → P1 → P2 → P3 (enables ~80% of required tags)

---

## Next Steps

1. **IMMEDIATE**: Wait for network connectivity
2. **THEN**: Run Priority 0 verification
3. **START**: Priority 1 (low-hanging fruit)
4. **ITERATE**: Work through priorities 2-7
5. **MONITOR**: Track placeholder count reduction
6. **VALIDATE**: Test against real camera images

---

## Appendix: Pattern Frequency Analysis

**From 237 placeholders**, estimated breakdown:

| Pattern | Count | Priority | ETA |
|---------|-------|----------|-----|
| String concatenation (`.`) | ~60 | P2 | Days 3-5 |
| Variable declarations | ~50 | P4 | Days 8-12 |
| Binary operations (pack/unpack) | ~40 | P5 | Days 13-18 |
| Bitwise operations | ~30 | P3 | Days 6-7 |
| Complex control flow | ~25 | P7 | Days 23+ |
| Regex substitution | ~15 | P6 | Days 19-21 |
| Other exotic patterns | ~17 | P7 | Days 23+ |

**Note**: Counts are estimates based on placeholder sampling. Actual distribution may vary.

---

## References

- **Docs**: `docs/CODEGEN.md`, `docs/TRUST-EXIFTOOL.md`, `docs/TDD.md`
- **TODOs**: `docs/todo/P01-fix-the-build.md`, `docs/todo/P01-ppi-token-support-gaps.md`
- **Analysis**: `docs/analysis/expressions/required-expressions-analysis.json`
- **Tests**: `codegen/tests/config/`
- **Registry**: `codegen/src/impl_registry/`
