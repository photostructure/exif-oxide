# Technical Project Plan: PPI Expression Pipeline Restoration and Enhancement

## Goal Definition

- **What Success Looks Like**: All 15 SKIP tests pass, enabling 90%+ expression coverage for required tags
- **Core Problem**: Codegen broken (trait conflict), can't validate which tokens actually work; 15 tests skipped
- **Key Constraints**: Fix compilation first, use existing test infrastructure, validate with integration tests only
- **ExifTool Alignment**: Each expression must produce identical output to ExifTool's evaluation
- **Success Validation**: `make codegen-test` passes with all SKIP tests enabled

## Mandatory Context Research Phase

### Step 1: Project Foundation Review

All engineers working on any task in this TPP MUST READ:

- **CLAUDE.md**: Token support exists in visitor.rs but untested/broken
- **TRUST-EXIFTOOL.md**: Expressions must match ExifTool exactly
- **TDD.md**: Only count as "done" when integration tests pass
- **ARCHITECTURE.md**: Visitor → Normalizers → Generator pipeline

### Step 2: Current State Analysis

- **Build Status**: BROKEN - trait conflict between PpiVisitor and FunctionGenerator (both define expression_type)
- **Working Tests**: 8 configs pass (basic operations)
- **Skipped Tests**: 15 configs waiting (hex, float, ternary, regex, etc.)
- **Token Support**: visitor.rs has handlers for many tokens BUT they're untested/unvalidated

### Step 3: Test Infrastructure

- **Test Location**: `codegen/tests/config/*/`
- **Test Format**: JSON configs with expression, type, and test cases
- **Validation**: `make codegen-test` runs all non-SKIP tests
- **Success Metric**: Test marked working = remove SKIP_ prefix and passes

---

## Task Definition

### Task A: URGENT - Fix Compilation Error

**At the end of this task**: Codegen compiles and existing tests run. This unblocks all other work.

**The Problem**: Multiple trait methods with same name causing ambiguity:
- `FunctionGenerator::expression_type()` 
- `PpiVisitor::expression_type()`

- [ ] **Disambiguate Trait Methods**: Use explicit trait syntax or rename one method
- [ ] **Compilation Successful**: `cargo build -p codegen` succeeds
- [ ] **Existing Tests Pass**: `make codegen-test` runs without compilation errors
- [ ] **Debug Tool Works**: `cargo run -p codegen --bin debug-ppi` executable

**Dependencies**: None - BLOCKER for everything else

### Task B: Assess Current Token Support

**At the end of this task**: Know exactly which tokens work vs. claimed to work. This enables targeted fixes.

- [ ] **Test Each SKIP Config**: Try unskipping one at a time, document failures
- [ ] **Categorize Failures**: Group by root cause (missing token, bad generation, wrong normalizer)
- [ ] **Priority List**: Order by impact on required tags
- [ ] **Quick Wins Identified**: Tests that might just work if unskipped

**Dependencies**: Task A complete (need working build)

### Task C: Enable Hex Number Support

**At the end of this task**: `SKIP_hex_number.json` passes. This enables bitmask operations.

**TDD Workflow**:
1. [ ] **Unskip Test**: Remove SKIP_ prefix from hex_number.json
2. [ ] **Run Test**: `make codegen-test`, document exact failure
3. [ ] **Fix Issue**: Could be visitor, normalizer, or generator problem
4. [ ] **Validate Success**: Test passes, no regressions

**Dependencies**: Task B complete (know current state)

### Task D: Enable Float Literal Support

**At the end of this task**: `SKIP_float_literal.json` passes. This enables precise numeric operations.

**TDD Workflow**:
1. [ ] **Unskip Test**: Remove SKIP_ prefix
2. [ ] **Run Test**: Document failure mode
3. [ ] **Fix Issue**: Likely in visit_number_float()
4. [ ] **Validate Success**: Test passes

**Dependencies**: Task B complete

### Task E: Enable Arithmetic Operations

**At the end of this task**: `SKIP_arithmetic.json` passes. This enables complex calculations.

**TDD Workflow**:
1. [ ] **Unskip Test**: Remove SKIP_ prefix
2. [ ] **Run Test**: Document failure
3. [ ] **Fix Issue**: May need operator precedence handling
4. [ ] **Validate Success**: Test passes

**Dependencies**: Tasks C & D complete (arithmetic uses various number types)

### Task F: Enable Ternary Operations

**At the end of this task**: `SKIP_ternary_string_comparison.json` passes. This enables conditional logic.

- [ ] **Unskip Test**: Remove SKIP_ prefix
- [ ] **Investigate**: Ternary normalizer exists, why is test skipped?
- [ ] **Fix Issues**: Could be string comparison problem
- [ ] **Validate**: Test passes

**Dependencies**: Task B complete

### Task G: Enable Regex Operations

**At the end of this task**: `SKIP_regex_substitute.json` passes. This enables string transformations.

- [ ] **Unskip Test**: Remove SKIP_ prefix
- [ ] **Check Handler**: visit_regexp_substitute() exists
- [ ] **Fix Generation**: Ensure generates valid Rust regex code
- [ ] **Validate**: Test passes

**Dependencies**: Task B complete

### Task H: Enable Remaining Tests

**At the end of this task**: All 15 SKIP tests pass. This completes expression support.

Tests to enable:
- `SKIP_basic_comparisons.json`
- `SKIP_defined_check.json`
- `SKIP_join_unpack.json`
- `SKIP_pack_map_bits.json`
- `SKIP_safe_division.json`
- `SKIP_sprintf_concat_ternary.json`
- `SKIP_tr_transliteration.json`
- `SKIP_unary_minus.json`
- `SKIP_variable_declaration.json`
- `SKIP_voltage_display.json`

- [ ] **Systematic Approach**: One test at a time, fix, validate
- [ ] **Document Issues**: What was blocking each test
- [ ] **No Regressions**: All previously passing tests still pass
- [ ] **Final Validation**: `make codegen-test` with zero skips

**Dependencies**: Tasks C-G provide foundation

### Task I: Validate Against Required Tags

**At the end of this task**: All 178 required tag expressions generate valid Rust. This proves readiness.

- [ ] **Extract Required Expressions**: From required-expressions-analysis.json
- [ ] **Test Generation**: Run through pipeline
- [ ] **Success Rate**: Document percentage that work
- [ ] **Gap Analysis**: What's still missing for 100%

**Dependencies**: Task H complete

---

## Critical Discovery

The ppi-token-analysis.md from August is WRONG. The visitor already has handlers for supposedly "missing" tokens:
- PPI::Statement::Expression ✅ (line 21)
- PPI::Token::Cast ✅ (line 22)
- PPI::Structure::Subscript ✅ (line 23)
- PPI::Token::Regexp::Match ✅ (line 24)

But WITHOUT PASSING TESTS, we can't trust any of them work correctly.

## Revised Success Metrics

1. **Build Success**: Codegen compiles without errors
2. **Test Coverage**: 23/23 test configs pass (0 SKIP files)
3. **Required Tags**: 90%+ of 178 required expressions generate
4. **No Regressions**: Existing working tests stay working
5. **ExifTool Parity**: Generated code matches ExifTool output

## Quick Reference

### Current Status
- **Build**: BROKEN (trait conflict)
- **Working Tests**: 8 (basic operations only)
- **Skipped Tests**: 15 (waiting for fixes)
- **Token Handlers**: ~20 exist but untested

### Key Commands
- `cargo build -p codegen` - Check if compilation fixed
- `make codegen-test` - Run integration tests
- `ls codegen/tests/config/*/SKIP_*.json` - See what's blocked
- `cargo run -p codegen --bin debug-ppi -- 'expression'` - Test parsing (once fixed)

### Priority Order
1. Fix compilation (Task A) - BLOCKER
2. Assess what works (Task B)
3. Enable easy wins (hex, float)
4. Work through complex tests
5. Validate completeness