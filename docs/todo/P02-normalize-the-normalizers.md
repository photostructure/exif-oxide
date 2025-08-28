# Technical Project Plan: Unified Precedence Climbing Expression Normalizer

## Goal Definition

- **What Success Looks Like**: Replace 6 fragmented PPI normalizers (2,700+ lines) with 1 unified precedence climbing normalizer that handles all Perl expression patterns consistently.
- **Core Problem**: Expression handling is fragmented across 6 separate normalizers with duplicate precedence logic, inconsistent operator handling, and complex multi-token pattern recognition.
- **Key Constraints**: Must pass all existing expression tests in `codegen/tests/config/`, maintain compatibility with rust_generator visitor, follow Trust ExifTool exactly.
- **ExifTool Alignment**: Implement Perl operator precedence exactly as ExifTool relies on Perl's built-in precedence (perlop documentation), including function call precedence and comma operator handling.
- **Success Validation**: All expression tests pass (`make generate-expression-tests && cargo test -p codegen --test generated_expressions`), normalizer code reduced from ~3,383 to ~683 lines (6→2 normalizers).

## Mandatory Context Research Phase

### Step 1: Project Foundation Review

**MANDATORY READING** (you must understand these constraints before implementing):

- **CLAUDE.md:42-72**: **CRITICAL VIOLATIONS THAT CAUSE INSTANT REJECTION** - Never use `split_whitespace()` on AST nodes, never delete ExifTool pattern recognition, must fix generators in `codegen/src/` not `src/generated/**/*.rs`. We've had 5+ emergency recoveries from engineers who ignored these patterns.
- **TRUST-EXIFTOOL.md:24-28**: **THE FUNDAMENTAL LAW** - Copy ExifTool logic exactly, preserve operation order, keep all special cases. ExifTool uses Perl's built-in precedence with function calls binding tighter than operators. Every seemingly odd piece of code exists for real camera bugs.
- **SIMPLE-DESIGN.md:49-56**: **PRIMARY DESIGN PRINCIPLE** - Rule 4 (Fewest Elements) eliminates 6 separate normalizers; Rule 3 (No Duplication) consolidates repeated pattern recognition logic. This project directly applies these rules.
- **TDD.md:190-198**: **VALIDATION FRAMEWORK** - Use expression test framework (`codegen/tests/config/`) as validation. This provides comprehensive end-to-end testing of PPI→AST→Rust pipeline with real ExifTool patterns.
- **ARCHITECTURE.md:311-315**: **INTEGRATION REQUIREMENT** - Multi-pass AST normalization system with `RewritePass` trait. You must consolidate within existing framework, never replace it.

### Step 2: Precedent Analysis

**CRITICAL ARCHITECTURAL CONTEXT** (understand before changing anything):

- **Existing Patterns**: Multi-pass system at `codegen/src/ppi/normalizer/multi_pass.rs:125-143` uses explicit pass ordering with `RewritePass` trait implementation. Follow this exact pattern - don't invent new architectures.
- **Dependencies**: `codegen/src/ppi/rust_generator/visitor.rs` expects specific AST node types like `BinaryOperation`, `TernaryOperation`, `FunctionCall`. You can create new canonical nodes but visitor must handle them.
- **Integration Points**: Pass registration in `multi_pass.rs`, AST consumption in visitor, compatibility with `debug-ppi` tool output. Breaking any of these integration points will cause system-wide failures.
- **Generated Code**: This is pure AST transformation logic - no lookup tables involved, must be manually implemented following Trust ExifTool principles. Never attempt to auto-generate this code.

### Step 3: ExifTool Research

**PERL PRECEDENCE REQUIREMENTS** (ExifTool behavior you must replicate exactly):

- **Source Analysis**: ExifTool relies on Perl's built-in operator precedence from perlop documentation. Function calls without parentheses have very high precedence, comma has very low precedence. Study perlop sections 5-23 to understand this completely.
- **Critical Edge Cases** (these will break if you get precedence wrong):
  - Right-associative exponentiation (`2**3**4` = `2**(3**4)`) 
  - Function call precedence (`join " ", unpack "H2H2", $val` = `join(" ", unpack("H2H2", $val))`)
  - Ternary precedence between logical OR and assignment
  - String concatenation specific precedence level
- **Test Cases**: 29 expression test configs in `codegen/tests/config/` including join_unpack, safe_division, ternary, function_calls patterns. These represent real ExifTool expressions that must work.
- **Output Format**: Generated Rust code structure can change as long as final execution results remain identical to current system

### Step 4: Risk Assessment

**FAILURE MODES** (specific ways this consolidation can break):

- **What Could Go Wrong**: 
  - Breaking function call precedence causing wrong argument grouping (`join " ", unpack "H2H2", $val` becomes `join("", unpack(" ", "H2H2", $val))`)
  - Precedence bugs in complex expressions with mixed operators and function calls
  - Regression in multi-token pattern recognition (join+unpack combinations fail)
  - Performance impact from complex precedence climbing vs simple pattern matching
- **Validation Strategy**: Compare AST output before/after using `debug-ppi`, ensure all expression test configs pass with identical results. Any difference in generated Rust code indicates a precedence bug.
- **Integration Testing**: Full expression test suite (`make generate-expression-tests && cargo test -p codegen --test generated_expressions`) must pass. This is your only reliable validation that precedence is correct.

**Quality Gate**: Can another engineer understand this consolidates 6 expression normalizers using proven precedence climbing algorithm while preserving 2 structural normalizers.

---

## TDD Integration Test (Task 0)

### Alternative Success Criteria: Expression Test Framework

**Purpose**: Prove unified precedence climbing system handles all expression patterns correctly and maintains backward compatibility.

**Success = All expression tests pass**: `make generate-expression-tests && cargo test -p codegen --test generated_expressions` succeeds with unified normalizer

This replaces traditional TDD as this is a refactoring effort with existing comprehensive test coverage in the expression test framework.

---

## Task Definition

### Task A: RESEARCH - Complete Expression Pattern and Precedence Analysis

**At the end of this task**: Comprehensive precedence table with all ExifTool expression operators and detailed analysis documenting which 6 normalizers can be unified via precedence climbing. This enables precise architectural design.

- [ ] **Analysis Complete**: `docs/research/precedence-climbing-consolidation.md` documents all operators with exact precedence values → Shows perlop precedence table with function call, comma, and all current normalizer operators
- [ ] **ExifTool Study**: Perl precedence behavior validated with interpreter testing → Cite perlop sections 5-23, test `join " ", unpack "H2H2", $val` precedence in actual Perl
- [ ] **Test Cases**: All 29 expression configs categorized by normalizer type → Map each test to current normalizer and required precedence handling  
- [ ] **Integration Strategy**: 6-normalizer consolidation boundaries defined → Clear separation between expression precedence (6 normalizers) vs structural transformation (2 remaining)

**Dependencies**: None

### Task B: Design Unified Precedence Climbing Architecture  

**At the end of this task**: Complete architectural design for consolidating exactly 6 expression normalizers using pure precedence climbing algorithm. This enables implementation with clear scope and responsibilities.

- [ ] **Implementation**: Architecture document with precedence climbing design → `docs/design/unified-precedence-architecture.md` shows single `ExpressionPrecedenceNormalizer` replacing 6 normalizers
- [ ] **Integration**: AST node design for all expression types → Define canonical nodes for binary ops, ternary, function calls, string ops, safe division, join+unpack
- [ ] **ExifTool Alignment**: Complete Perl precedence table implementation → Function calls (precedence 100), operators (50-5), comma (1) exactly matching perlop
- [ ] **Cleanup**: Migration strategy from 6 individual normalizers → Feature flag approach with gradual replacement validation

**Dependencies**: Task A complete

### Task C: Implement Core Precedence Climbing Algorithm

**At the end of this task**: Working precedence climbing handles binary operations, string concatenation, and function call precedence. This proves the core algorithm works for ExifTool expression patterns.

- [ ] **Implementation**: Core precedence climbing with function calls → `cargo test -p codegen test_precedence_core` passes for binary ops and function precedence
- [ ] **Integration**: Unified normalizer integrated with multi-pass system → `grep -r "ExpressionPrecedenceNormalizer" codegen/src/ppi/normalizer/` shows registration
- [ ] **ExifTool Alignment**: Function and operator precedence correct → `debug-ppi 'join " ", unpack "H2H2", $val'` shows proper nesting: `join(" ", unpack("H2H2", $val))`
- [ ] **Cleanup**: Binary, string, and function normalizers disabled → Feature flags prevent conflicts during transition

**Dependencies**: Task B complete

### Task D: Extend to Complex Multi-Token Patterns (Join+Unpack, Ternary, Safe Division)

**At the end of this task**: Unified precedence system handles all complex expression patterns including multi-function composition and specialized ternary patterns. This completes the 6-normalizer consolidation.

- [ ] **Implementation**: All complex patterns supported → Tests for join+unpack, ternary, safe division pass with precedence climbing
- [ ] **Integration**: All 6 target normalizers replaced → Only `ExpressionPrecedenceNormalizer` handles binary, string, ternary, function calls, join+unpack, safe division
- [ ] **ExifTool Alignment**: Complex expressions maintain correct precedence → `debug-ppi '$val ? join ":", unpack "H2*", $data : 0'` shows proper operator and function grouping
- [ ] **Cleanup**: 6 old normalizers removed from active pipeline → Multi-pass system only loads unified normalizer for expression handling

**Dependencies**: Task C complete

### Task E: Comprehensive Expression Test Validation and Final Cleanup

**At the end of this task**: Unified expression system passes all expression test configs with massive code reduction while preserving 2 structural normalizers unchanged. This completes the consolidation.

- [ ] **Implementation**: All expression tests pass → `make generate-expression-tests && cargo test -p codegen --test generated_expressions` succeeds with no regressions
- [ ] **Integration**: Structural normalizers preserved → `ConditionalStatementsNormalizer` and `SneakyConditionalAssignmentNormalizer` remain as focused single-purpose passes  
- [ ] **ExifTool Alignment**: Generated Rust identical to baseline → Expression execution results unchanged, AST structure improved
- [ ] **Cleanup**: Massive code reduction achieved → Normalizer passes reduced from 8 files/3,383 lines to 3 files/683 lines (~2,700 line reduction)

**Dependencies**: Task D complete

## Validation Requirements

### Required Evidence

- **Commands that pass**: `make generate-expression-tests && cargo test -p codegen --test generated_expressions`, `debug-ppi` comparison tests, `make precommit`
- **Code locations**: `codegen/src/ppi/normalizer/passes/expression_precedence.rs` for unified normalizer implementation
- **Integration proof**: Multi-pass system registration showing only 3 total normalizers, rust_generator compatibility maintained
- **Behavior changes**: ~2,700 line code reduction (6→1 consolidation), identical expression test results, improved AST consistency

### Anti-Vandalism Validation

**Integration Requirements**: Unified system must connect to existing visitor patterns and expression test framework.

- ✅ **Production Usage**: `grep -r "ExpressionPrecedenceNormalizer" codegen/src/ppi/normalizer/` shows actual usage in multi-pass pipeline
- ✅ **Behavior Change**: Normalizer file count reduced from 8 to 3, line count reduced by ~80% in passes directory
- ✅ **Cleanup Complete**: `find codegen/src/ppi/normalizer/passes -name "*binary*" -o -name "*string*" -o -name "*ternary*" -o -name "*safe_division*" -o -name "*function_calls*" -o -name "*join_unpack*"` returns empty
- ❌ **Shelf-ware**: Unified normalizer exists but old normalizers still active in pipeline
- ❌ **Half-integrated**: Precedence climbing works for some patterns but not others

**Common Over-Selling Patterns** (DO NOT mark tasks complete if any apply):
- "Core algorithm works" but join+unpack patterns still fail
- "Tests pass" but only unit tests, not full expression test suite  
- "Consolidation complete" but old normalizer files still exist
- "Ready for review" but `make precommit` fails due to expression test regressions

## Precedence Climbing Scope (Final)

Based on comprehensive analysis, precedence climbing consolidates **75% of normalizers**:

### ✅ UNIFIED BY PRECEDENCE CLIMBING (6 normalizers → 1):
1. **BinaryOperatorNormalizer** - Standard operator precedence (`*`, `+`, `==`, etc.)
2. **StringOpNormalizer** - String concatenation operator precedence (`.`)  
3. **TernaryNormalizer** - Ternary operator precedence (`?:`)
4. **SafeDivisionNormalizer** - Specialized ternary pattern (`$val ? 1/$val : 0`)
5. **FunctionCallNormalizer** - Function call precedence (very high, binds before operators)
6. **JoinUnpackPass** - Multi-function precedence (`join " ", unpack "H2H2", $val`)

### ✅ PRESERVED AS FOCUSED PASSES (2 normalizers unchanged):
1. **ConditionalStatementsNormalizer** - Statement restructuring (`return $val if $condition`)
2. **SneakyConditionalAssignmentNormalizer** - Multi-statement control flow (`$val > 1800 and $val -= 3600; -$val / 10`)

**Result**: 8 normalizers → 3 normalizers (~2,700 line reduction, 75% consolidation)

This achieves massive complexity reduction while respecting architectural boundaries between expression precedence and structural transformation concerns.