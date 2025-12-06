# P09: Eliminate TIMTOWTDI in Codegen (There Is More Than One Way To Do It)

## Problem Statement

The codegen system suffers from excessive inconsistency - the same functionality implemented 3-5 different ways across different files. Engineers waste time choosing between approaches, debugging mixed patterns, and maintaining redundant code. Critical example: `generate_function_call_without_parens` in functions.rs:150 uses 4 different strategies for identical operations.

**Why it matters**: TIMTOWTDI multiplies maintenance burden by 3-5x and creates cognitive load choosing between arbitrary alternatives.

**Solution**: Standardize on one "best practice" approach per category, refactor all inconsistencies to use chosen patterns.

**Success test**: `rg "TagValue::String.*\.to_string|String::new" codegen/` returns only one pattern type

**Key constraint**: Must not break existing generated code - refactor generators, not their output.

## Research: Finding the Inconsistency Patterns

### A. TagValue Construction - 3 Different Approaches

```bash
# Find empty string creation patterns
rg "TagValue::String.*empty|String::new|\.to_string\(\)" codegen/src/
# Found: 94 instances of .to_string(), 2 of String::new(), 8 of .into()

# Find default value inconsistencies
rg "unwrap_or.*TagValue::|unwrap_or.*Empty|unwrap_or.*\d" codegen/src/
# Found: Mixed defaults (TagValue::Empty, TagValue::U32(1), raw 0)
```

**Inconsistency Count**:
- Empty strings: 3 different constructors
- Default values: 4 different fallback patterns
- String length operations: Duplicated across 3 files with subtle differences

### B. Function Call Generation - 4 Different Strategies

```bash
# Find mathematical function patterns
rg "log\(|\.abs\(\)|codegen_runtime::|power\(" codegen/src/
# Found: Bare functions, method calls, runtime helpers, special cases

# Find sprintf implementations
rg "sprintf_perl|format!\|codegen_runtime::sprintf" codegen/src/
# Found: 4 completely different sprintf generation approaches
```

**Critical Issue**: Math operations use inconsistent patterns:
- `log({})` - bare function
- `({}).abs()` - method call
- `codegen_runtime::math::abs({})` - runtime helper
- `power({}, {})` - special case function

### C. String Generation - 5 Different Escaping Patterns

```bash
# Find string escaping patterns
rg "replace.*\\\\\|escape_default|raw.*string" codegen/src/
# Found: Manual replace chains, utility functions, escape_default(), raw strings

# Find string building patterns
rg "formatdoc!|format!.*push_str|writeln!" codegen/src/
# Found: formatdoc! (26 instances), format!+push_str (352+ instances), writeln! (rare)
```

### D. Architecture - Circular Dependencies

```bash
# Find visitor pattern implementations
find codegen/src -name "*visitor*" -type f
# Found: visitor.rs (1700 lines), visitor_tokens.rs (functions), visitor_advanced.rs (empty)

# Find trait delegation patterns
rg "impl.*for RustGenerator" codegen/src/ppi/rust_generator/mod.rs
# Found: 4 traits that delegate back to the same struct
```

## Architecture Change Adaptation

If codegen architecture changes:

**If TagValue enum changes**:
1. Search: `rg "TagValue::" codegen/src/` to find all construction sites
2. Update: Chosen standardized patterns in centralized helpers
3. Goal unchanged: Consistent construction across all generators

**If expression system refactored**:
1. Search: `rg "ExpressionType::" codegen/src/` to find context-dependent logic
2. Maintain: Consistent return type strategy regardless of new architecture

## Tasks

### Task 1: Standardize TagValue Construction

**Success**: `rg "TagValue::String\(" codegen/src/ | wc -l` shows 90% reduction in patterns

**Implementation**:
1. Create `codegen_runtime/src/tag_value/construction.rs` with helpers:
   ```rust
   pub fn empty_string() -> TagValue { TagValue::String(String::new()) }
   pub fn string_from<T: Into<String>>(s: T) -> TagValue { TagValue::String(s.into()) }
   ```
2. Replace all `.to_string()` patterns in functions.rs:159-166
3. Consolidate string length operations from 3 files into expressions/normalized.rs

**If architecture changed**:
- No TagValue? Find new construction: `rg "String\|I32\|F64" src/`
- Goal unchanged: Single pattern for same operation

**Proof of completion**:
- [ ] Test passes: `cargo t expression_generation`
- [ ] Pattern unified: `rg "TagValue::String.*String::new" codegen/` finds all instances
- [ ] Old patterns removed: `rg "\.to_string\(\).*TagValue" codegen/` returns empty

### Task 2: Unify Function Call Generation

**Success**: `cargo run expression_test.json` shows consistent math function calls

**Implementation**:
1. Choose runtime helper pattern: `codegen_runtime::math::{function}`
2. Update function registry in fn_registry/mod.rs:29 to import all math functions
3. Replace bare function calls in visitor.rs:372-428
4. Standardize sprintf to single `codegen_runtime::sprintf_perl` approach

**If architecture changed**:
- No function registry? Find call sites: `rg "log\(|abs\(|exp\(" codegen/`
- Goal unchanged: One pattern for each mathematical operation

**Proof of completion**:
- [ ] Math unified: `rg "log\(.*[^runtime]" codegen/` returns empty (no bare function calls)
- [ ] sprintf unified: `rg "sprintf" codegen/ | grep -v sprintf_perl` returns empty
- [ ] Tests pass: `cargo t function_call_generation`

### Task 3: Standardize String Generation

**Success**: All multi-line templates use formatdoc!, escaping uses single utility

**Implementation**:
1. Extend `common/utils.rs` escape function to handle all cases
2. Replace manual replace chains in simple_table.rs:99, magic_numbers.rs:98
3. Convert format!+push_str patterns to formatdoc! in tag_kit.rs:142-147
4. Remove single `+` concatenation in simple_table.rs:134

**If architecture changed**:
- No formatdoc? Find templating: `rg "format!.*\n.*format!" codegen/`
- Goal unchanged: Single pattern for multi-line generation

**Proof of completion**:
- [ ] Escaping unified: `rg "replace.*\\\\\\\\" codegen/` shows only utility function usage
- [ ] Templates converted: `rg "push_str.*format!" codegen/` returns empty
- [ ] Style consistent: All generated code has uniform indentation

### Task 4: Eliminate Architectural Over-Engineering

**Success**: Single visitor pattern, no circular trait dependencies

**Implementation**:
1. Remove empty visitor_advanced.rs and visitor_tokens.rs helper functions
2. Eliminate circular trait delegation in rust_generator/mod.rs:36-114
3. Merge 5-trait ExpressionCombiner hierarchy into 2 focused traits
4. Consolidate error types to single CodeGenError across all generators

**If architecture changed**:
- No traits? Find delegation: `rg "impl.*for.*Generator" codegen/`
- Goal unchanged: Clear separation of concerns, no circular dependencies

**Proof of completion**:
- [ ] Visitor unified: `find codegen/ -name "*visitor*"` shows single file
- [ ] Traits simplified: `rg "impl.*for RustGenerator" codegen/` shows max 2 traits
- [ ] Errors consistent: `rg "anyhow::Result|CodeGenError" codegen/` shows single pattern
- [ ] Build clean: `cargo check codegen` with no warnings

## Integration with Core Principles

**From SIMPLE-DESIGN.md Rule 3 (No Duplication)**:
- String length operations duplicated across 3 files → Extract to single location
- sprintf implementations duplicated in 4 ways → Use single runtime helper

**From SIMPLE-DESIGN.md Rule 4 (Fewest Elements)**:
- 5-trait hierarchy → 2 focused traits
- Empty visitor modules → Remove entirely
- 3 empty string constructors → 1 helper function

## Emergency Recovery

```bash
# If refactoring breaks codegen
git diff HEAD~ > timtowtdi_changes.patch
git apply -R timtowtdi_changes.patch

# Validate specific generators still work
cargo run --bin codegen -- --module Canon.pm
diff old_output.rs new_output.rs  # Should be identical

# Full validation
make clean-all codegen && cargo t
```

**Break-glass procedure**: Each task is independent - revert individual sections if needed.