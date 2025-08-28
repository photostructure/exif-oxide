# Technical Project Plan: Unified Precedence Climbing Expression Normalizer

## 🟦 TPP CREATION PHASE (Do This First)

**WHO**: TPP Author (you right now)  
**GOAL**: Gather sufficient context for implementer success  
**OUTPUT**: Research analysis written FOR the implementer  
**WORKFLOW**: Complete all research sections below, then move to Execution Phase

## 🟨 EXECUTION PHASE (Do This Later)

**WHO**: Implementer (could be you or someone else)  
**GOAL**: Complete tasks using research context  
**OUTPUT**: Working code with checkbox evidence  
**WORKFLOW**: Use research context to complete tasks with verifiable proof

## Goal Definition (MANDATORY FIRST SECTION)

- **What Success Looks Like**: Replace 6 fragmented PPI normalizers (2,700+ lines) with 1 unified precedence climbing normalizer that handles all Perl expression patterns consistently.
- **Core Problem**: Expression handling is fragmented across 6 separate normalizers with duplicate precedence logic, inconsistent operator handling, and complex multi-token pattern recognition.
- **Key Constraints**: Must pass all existing expression tests in `codegen/tests/config/`, maintain compatibility with rust_generator visitor, follow Trust ExifTool exactly.
- **ExifTool Alignment**: Implement Perl operator precedence exactly as ExifTool relies on Perl's built-in precedence (perlop documentation), including function call precedence and comma operator handling.
- **Success Validation**: All expression tests pass (`make generate-expression-tests && cargo test -p codegen --test generated_expressions`), normalizer code reduced from ~3,383 to ~683 lines (6→2 normalizers).

## Mandatory Context Research Phase

**CRITICAL**: Complete ALL sections before defining tasks. This prevents architectural vandalism and shallow solutions.

### Step 1: Project Foundation Review

**MANDATORY READING** (document your understanding - future implementers must read this):

**⚠️ CRITICAL**: Write this analysis for the IMPLEMENTER, not yourself. Use **IMPERATIVE LANGUAGE** ("You must...", "Never do...", "This will break if...") so implementers understand constraints.

**CLAUDE.md Analysis**: 
You must never use `split_whitespace()` on AST nodes as this breaks Perl expression parsing. Never delete ExifTool pattern recognition - we've had 5 emergency recoveries from this violation. You must fix generators in `codegen/src/` never `src/generated/**/*.rs` as generated code is automatically overwritten.

**TRUST-EXIFTOOL.md Analysis**:
You must copy ExifTool logic exactly including Perl's built-in precedence where function calls bind tighter than operators. Every seemingly odd piece of code exists to handle camera bugs from 25 years of real-world testing. Deviation breaks compatibility with thousands of camera models.

**SIMPLE-DESIGN.md Analysis**:
This project directly applies Rule 4 (Fewest Elements) by eliminating 6 separate normalizers and Rule 3 (No Duplication) by consolidating repeated pattern recognition logic. The unified precedence climbing algorithm achieves massive complexity reduction.

**TDD.md Analysis**:
You must validate using the expression test framework (`codegen/tests/config/`) which provides comprehensive end-to-end testing of PPI→AST→Rust pipeline. This framework tests real ExifTool expressions and is your only reliable validation method.

**ARCHITECTURE.md Analysis**:
You must integrate within the existing multi-pass AST normalization system using `RewritePass` trait at `codegen/src/ppi/normalizer/multi_pass.rs:125-143`. Never replace this framework - consolidate within it or you'll break the entire system.

### Step 2: Precedent Analysis

**CRITICAL ARCHITECTURAL CONTEXT** (document constraints for implementers):

**⚠️ CRITICAL**: Write this for the IMPLEMENTER. Explain what will BREAK if they don't follow existing patterns.

**Existing Patterns Analysis**:
The multi-pass system at `codegen/src/ppi/normalizer/multi_pass.rs:125-143` uses explicit pass ordering with `RewritePass` trait. You must follow this exact pattern - inventing new architectures will break integration with rust_generator visitor and debug-ppi tooling.

**Dependencies Analysis**:
The `codegen/src/ppi/rust_generator/visitor.rs` expects specific AST node types like `BinaryOperation`, `TernaryOperation`, `FunctionCall`. You can create new canonical nodes but the visitor must handle them or code generation will fail.

**Integration Points Analysis**: 
You must maintain pass registration in `multi_pass.rs`, AST consumption in visitor, and compatibility with `debug-ppi` tool output. Breaking these integration points causes system-wide failures as the entire codegen pipeline depends on them.

**Generated Code Analysis**:
This is pure AST transformation logic with no lookup tables. You must manually implement following Trust ExifTool principles. Never attempt auto-generation as expression parsing requires complex understanding of Perl precedence rules.

### Step 3: ExifTool Research

**EXIFTOOL BEHAVIOR REQUIREMENTS** (document what implementers must replicate exactly):

**⚠️ CRITICAL**: Write this for the IMPLEMENTER. Document the EXACT behavior they must replicate and WHY.

**Source Analysis**:
ExifTool relies on Perl's built-in operator precedence from perlop documentation sections 5-23. Function calls without parentheses have very high precedence, comma has very low precedence. You must implement this precedence table exactly.

**Critical Edge Cases**:
Right-associative exponentiation (`2**3**4` = `2**(3**4)`) will break if you implement left-associative. Function call precedence (`join " ", unpack "H2H2", $val` = `join(" ", unpack("H2H2", $val))`) will break if you get argument grouping wrong. These are real patterns in ExifTool modules.

**Test Cases**:
29 expression test configs in `codegen/tests/config/` including join_unpack, safe_division, ternary, function_calls patterns represent real ExifTool expressions. These must work identically or camera metadata parsing will fail.

**Output Format Requirements**:
Generated Rust code structure can change but final execution results must remain identical to current system. Any difference in computed values indicates a precedence implementation bug.

### Step 4: Risk Assessment

**FAILURE MODES** (document specific ways implementers can break things):

**⚠️ CRITICAL**: Write this for the IMPLEMENTER. Document SPECIFIC failure modes with EXAMPLES.

**What Could Go Wrong**:
Breaking function call precedence causes wrong argument grouping (`join " ", unpack "H2H2", $val` becomes `join("", unpack(" ", "H2H2", $val))`). Precedence bugs in complex expressions break camera metadata parsing. Performance impact from complex precedence climbing may be unacceptable vs simple pattern matching.

**Emergency Recovery Plan**:
Implement feature flag in `multi_pass.rs` to switch between unified/individual normalizers during transition. You can quickly revert to old normalizers if precedence bugs are discovered in production.

**Validation Strategy**:
You must compare AST output before/after using `debug-ppi` and ensure all expression test configs pass with identical results. Any difference in generated Rust code indicates precedence bugs that will break camera support.

**Integration Testing Requirements**:
Full expression test suite (`make generate-expression-tests && cargo test -p codegen --test generated_expressions`) must pass. This is your only reliable validation that precedence is correct for all ExifTool patterns.

## 🔍 TPP HANDOFF VALIDATION

**COMPLETE THIS BEFORE MARKING TPP READY FOR IMPLEMENTATION**

Before moving to Execution Phase, verify this TPP provides sufficient context for successful handoff:

- [x] **Context Sufficiency**: Another engineer can understand WHY this approach is needed (consolidation using precedence climbing)
- [x] **Implementation Clarity**: Tasks specify exact precedence climbing implementation with clear validation  
- [x] **Constraint Documentation**: All precedence gotchas documented with concrete ExifTool examples
- [x] **Success Measurement**: Clear commands (`debug-ppi`, expression tests) prove completion
- [x] **ExifTool Alignment**: Specific perlop sections and ExifTool patterns documented
- [x] **Integration Requirements**: Clear proof requirements showing multi-pass integration (not shelf-ware)

**HANDOFF TEST**: ✅ Yes - implementer has sufficient context to succeed without clarifying questions

---

## TDD Integration Test (Task 0)

### Alternative Success Criteria: Expression Test Framework

**Purpose**: Prove unified precedence climbing system handles all expression patterns correctly and maintains backward compatibility.

**Success = All expression tests pass**: `make generate-expression-tests && cargo test -p codegen --test generated_expressions` succeeds with unified normalizer

---

## Task Definition

### Task A: Implement Core Precedence Climbing Algorithm

**What works after this task**: Working precedence climbing handles binary operations, string concatenation, and function call precedence correctly.

**Implementation approach**: Build precedence climbing parser following perlop precedence table. Implement function call precedence (very high), operators (50-5), comma (1) exactly matching Perl behavior. Start with core binary operations and function calls.

**Validation commands**: 
- `cargo test -p codegen test_precedence_core` - proves binary ops and function precedence
- `debug-ppi 'join " ", unpack "H2H2", $val'` - shows proper nesting: `join(" ", unpack("H2H2", $val))`
- `grep -r "ExpressionPrecedenceNormalizer" codegen/src/ppi/normalizer/` - shows registration

**Dependencies**: None

**Completion checklist** (mark during execution):
- [ ] **Code implemented** → [codegen/src/ppi/normalizer/passes/expression_precedence.rs with core algorithm]
- [ ] **Tests passing** → [`cargo test -p codegen test_precedence_core` succeeds]  
- [ ] **Production integration** → [multi-pass system registers and uses unified normalizer]
- [ ] **Cleanup complete** → [feature flags prevent conflicts with old normalizers]

### Task B: Extend to Complex Multi-Token Patterns

**What is understood after this task**: All complex expression patterns (join+unpack, ternary, safe division) work with precedence climbing.

**Implementation approach**: Extend precedence climbing to handle multi-function composition and specialized ternary patterns. Focus on join+unpack combinations and ternary precedence between logical OR and assignment.

**Validation commands**: 
- `debug-ppi '$val ? join ":", unpack "H2*", $data : 0'` - shows proper operator and function grouping
- `cargo test -p codegen` - all complex pattern tests pass

**Dependencies**: Task A complete

**Completion checklist** (mark during execution):
- [ ] **Analysis documented** → [All 6 target normalizers replaced by unified system]
- [ ] **ExifTool behavior mapped** → [Complex expressions maintain correct precedence vs ExifTool]
- [ ] **Implementation strategy** → [All expression test configs pass with precedence climbing]
- [ ] **Test cases identified** → [6 old normalizers removed from active pipeline]