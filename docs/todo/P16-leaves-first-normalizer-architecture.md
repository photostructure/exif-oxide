# P16: Leaves-First Normalizer Architecture

## Project Overview

- **Goal**: Replace 8 recursive normalizers with leaves-first central orchestrator + simple single-node transformers, eliminating interface mismatches and reducing codebase complexity by ~60%
- **Problem**: Current normalizers each implement their own tree traversal, creating precedence conflicts, interface mismatches (NestedFunctionNormalizer → Generator), and 8x code duplication. Generator can't handle nested FunctionCall structures from NestedFunctionNormalizer.
- **Constraints**: Must preserve exact ExifTool semantics, maintain all existing test compatibility, zero performance regression

## Context & Foundation

### System Overview

- **AST Normalizer Pipeline**: Transforms PPI AST nodes from Perl-like structures into canonical forms before Rust code generation. Currently 8 normalizers each traverse tree recursively in precedence order.
- **Rust Generator**: Consumes normalized AST nodes and generates Rust functions. Expects simple FunctionCall nodes and flat pattern matching, struggles with nested structures.
- **Precedence System**: 3-tier (High/Medium/Low) ensures string ops run before ternary ops run before function calls, matching Perl operator precedence.

### Key Concepts & Domain Knowledge

- **Post-order traversal**: Children processed before parents, matching Perl's "innermost first" evaluation semantics
- **Normalization**: Converting complex PPI patterns into simple canonical AST nodes (FunctionCall, StringConcat, TernaryOp, etc.)
- **Interface mismatch**: NestedFunctionNormalizer creates `FunctionCall(join, [sep, FunctionCall(unpack, [...])])` but generator only handles flat structures

### Surprising Context

- **Each normalizer duplicates tree traversal**: 8 normalizers × recursive visitor pattern = massive code duplication and maintenance burden
- **NestedFunctionNormalizer breaks generator**: Creates nested FunctionCall structures that `handle_normalized_function_call` can't process, causing "join requires exactly 2 arguments" errors
- **Pattern matching works better**: Generator's `try_join_unpack_pattern` successfully handles complex cases that normalized AST approach fails on
- **Precedence conflicts**: Current system relies on pass ordering rather than proper AST precedence, causing subtle bugs when normalizers interfere
- **Single traversal is faster**: Post-order visitor can apply all normalizers in one pass instead of 8 separate tree walks

### Foundation Documents

- **Design docs**: [docs/CODEGEN.md](docs/CODEGEN.md) - normalizer integration, [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - module structure
- **ExifTool source**: N/A - this is pure architecture improvement
- **Start here**: 
  - `codegen/src/ppi/normalizer/mod.rs:49-101` - current precedence system
  - `codegen/src/ppi/normalizer/passes/nested_functions.rs` - complex pattern that needs simplification
  - `codegen/src/ppi/rust_generator/expressions/normalized.rs:91-142` - how generator handles FunctionCall nodes

### Prerequisites

- **Knowledge assumed**: Rust traits, AST traversal patterns, compiler precedence systems
- **Setup required**: Working codegen test environment (`cargo t -p codegen`)

## Work Completed

- ✅ **Root cause analysis** → identified interface mismatch between NestedFunctionNormalizer and generator
- ✅ **Architecture research** → verified leaves-first approach solves precedence and complexity issues
- ✅ **Pattern validation** → confirmed generator's `try_join_unpack_pattern` works correctly for complex cases
- ✅ **Complexity analysis** → documented 8x code duplication in current recursive normalizers
- ✅ **Performance analysis** → single-pass traversal vs 8-pass current approach

## TDD Foundation Requirement

### Task A: Implement central leaves-first AST traverser

**Success Criteria**:

- [ ] **Implementation**: Post-order traverser created → `codegen/src/ppi/normalizer/leaves_first.rs:1-150` implements `LeavesFirstNormalizer`
- [ ] **Integration**: Exports single normalize function → `codegen/src/ppi/normalizer/mod.rs:138` calls `leaves_first::normalize()`
- [ ] **Task 0 passes**: `cargo t test_leaves_first_equivalence` now succeeds
- [ ] **Unit tests**: `cargo t test_leaves_first_traversal` validates post-order execution
- [ ] **Manual validation**: `RUST_LOG=debug cargo t test_join_unpack_end_to_end` shows single-pass normalization
- [ ] **Cleanup**: N/A - new implementation alongside old
- [ ] **Documentation**: Architecture doc updated → `docs/ARCHITECTURE.md:normalizer_section` documents new approach

**Implementation Details**: 
- Create `LeavesFirstNormalizer` struct with `normalize(ast: PpiNode) -> PpiNode` method
- Implement post-order traversal visiting children before applying normalizers to parent
- Apply normalizers in precedence order at each node using trait objects
- Preserve exact same normalization logic, just change orchestration

**Integration Strategy**: Export as alternative to current `normalize()` function for side-by-side testing
**Validation Plan**: Compare output with current normalizer on all existing test cases
**Dependencies**: None

**Success Patterns**:
- ✅ Single tree traversal instead of 8 separate passes
- ✅ Children always normalized before parents (post-order semantics)
- ✅ Precedence preserved by applying normalizers in order at each node

### Task B: Create single-node normalizer trait

**Success Criteria**:

- [ ] **Implementation**: New trait defined → `codegen/src/ppi/normalizer/single_node.rs:1-30` defines `SingleNodeNormalizer` trait
- [ ] **Integration**: Used by leaves-first traverser → `leaves_first.rs:45` calls `transform_single_node()`
- [ ] **Task 0 passes**: Still passing with new trait structure
- [ ] **Unit tests**: `cargo t test_single_node_interface` validates trait contract
- [ ] **Manual validation**: `cargo t -p codegen -- --nocapture` shows normalizers applied to individual nodes
- [ ] **Cleanup**: N/A - new trait alongside existing
- [ ] **Documentation**: Trait documented → `single_node.rs:1-15` includes usage examples

**Implementation Details**:
```rust
pub trait SingleNodeNormalizer {
    fn name(&self) -> &str;
    fn precedence_level(&self) -> PrecedenceLevel;
    fn transform_single_node(&self, node: PpiNode) -> PpiNode; // NO recursion
}
```

**Integration Strategy**: LeavesFirstNormalizer uses trait objects to apply transforms
**Validation Plan**: Ensure each normalizer only sees one node at a time
**Dependencies**: Task A complete

**Success Patterns**:
- ✅ No recursion logic in individual normalizers
- ✅ Clean separation between traversal and transformation
- ✅ Easy to test individual transformations in isolation

### Task C: Convert FunctionCallNormalizer to single-node

**Success Criteria**:

- [ ] **Implementation**: Simplified normalizer → `codegen/src/ppi/normalizer/passes/function_calls.rs:21-35` removes all recursion
- [ ] **Integration**: Implements SingleNodeNormalizer → same file implements new trait
- [ ] **Task 0 passes**: Function call normalization still works correctly
- [ ] **Unit tests**: `cargo t test_function_call_single_node` validates simplified logic
- [ ] **Manual validation**: `cargo t test_function_call_normalization` passes with simplified implementation
- [ ] **Cleanup**: Recursive code removed → old `transform()` method replaced
- [ ] **Documentation**: N/A

**Implementation Details**: 
- Remove `utils::transform_children()` calls
- Keep only pattern matching for `length $val` → `FunctionCall(length, [val])`
- Transform becomes ~10 lines instead of ~50 lines
- No tree traversal logic needed

**Integration Strategy**: SingleNodeNormalizer trait automatically handles orchestration
**Validation Plan**: All function call tests continue passing
**Dependencies**: Task B complete

**Success Patterns**:
- ✅ 80% code reduction in FunctionCallNormalizer
- ✅ No recursive calls in transform method
- ✅ Identical output for simple function patterns

### Task D: Convert remaining normalizers to single-node

**Success Criteria**:

- [ ] **Implementation**: All 7 normalizers converted → `find codegen/src/ppi/normalizer/passes/ -name "*.rs" -exec grep -L "transform_children" {} \;` returns all files
- [ ] **Integration**: All implement SingleNodeNormalizer → `cargo t test_all_normalizers_single_node` passes
- [ ] **Task 0 passes**: Complex patterns still normalize correctly
- [ ] **Unit tests**: Individual tests for each converted normalizer pass
- [ ] **Manual validation**: `cargo t test_join_unpack_pattern test_safe_division_pattern test_sprintf_concat` all pass
- [ ] **Cleanup**: All recursive logic removed → `rg "transform_children|utils::transform" codegen/src/ppi/normalizer/passes/` returns empty
- [ ] **Documentation**: N/A

**Implementation Details**: 
Convert in order: StringOpNormalizer, SafeDivisionNormalizer, TernaryNormalizer, PostfixConditionalNormalizer, SneakyConditionalAssignmentNormalizer, SprintfNormalizer, NestedFunctionNormalizer

**Integration Strategy**: Each normalizer focuses only on single-node pattern recognition
**Validation Plan**: Comprehensive test suite validates all patterns still work
**Dependencies**: Task C complete

**Success Patterns**:
- ✅ No normalizer contains tree traversal logic
- ✅ All existing test patterns continue working
- ✅ Dramatic code simplification across all normalizers

### Task E: Remove NestedFunctionNormalizer completely

**Success Criteria**:

- [ ] **Implementation**: File deleted → `codegen/src/ppi/normalizer/passes/nested_functions.rs` no longer exists
- [ ] **Integration**: Pattern matching handles join+unpack → `test_join_unpack_pattern` uses generator pattern matching instead
- [ ] **Task 0 passes**: Complex nested patterns work via pattern matching
- [ ] **Unit tests**: `cargo t test_pattern_matching_handles_nested` validates generator approach
- [ ] **Manual validation**: `cargo t test_join_unpack_end_to_end` passes using pattern matching path
- [ ] **Cleanup**: All references removed → `rg "NestedFunctionNormalizer" codegen/src/` returns empty
- [ ] **Documentation**: Architecture doc updated → `docs/ARCHITECTURE.md:normalizer_section` removes nested function references

**Implementation Details**: 
- Delete nested_functions.rs entirely
- Remove from mod.rs exports
- Verify generator pattern matching handles all use cases
- May need to enhance `try_join_unpack_pattern` for edge cases

**Integration Strategy**: Generator pattern matching becomes primary path for complex function patterns
**Validation Plan**: All join+unpack tests pass through pattern matching
**Dependencies**: Task D complete

**Success Patterns**:
- ✅ Interface mismatch eliminated
- ✅ Complex patterns handled by proven pattern matching approach
- ✅ No nested FunctionCall structures generated

### Task F: Remove old recursive normalizer infrastructure

**Success Criteria**:

- [ ] **Implementation**: Old code deleted → `codegen/src/ppi/normalizer/mod.rs` contains only leaves-first approach
- [ ] **Integration**: All production paths use new normalizer → `grep -r "normalize(" codegen/src/` shows only leaves-first calls
- [ ] **Task 0 passes**: Full integration test passes with new architecture
- [ ] **Unit tests**: `cargo t -p codegen` passes with 100% new architecture
- [ ] **Manual validation**: `cargo run --bin codegen` generates identical output to baseline
- [ ] **Cleanup**: Old infrastructure removed → `git diff --stat` shows old normalizer code deleted
- [ ] **Documentation**: Architecture guide updated → `docs/ARCHITECTURE.md:75-120` documents final architecture

**Implementation Details**: 
- Remove old `AstNormalizer` struct and precedence pass infrastructure
- Keep only `LeavesFirstNormalizer` as single normalizer implementation
- Update all imports and references
- Ensure no performance regression

**Integration Strategy**: Complete replacement of old system
**Validation Plan**: Full test suite passes, performance benchmarks show improvement
**Dependencies**: Task E complete

**Success Patterns**:
- ✅ Single normalizer implementation instead of dual approaches
- ✅ 60%+ code reduction in normalizer module
- ✅ No interface mismatches between normalizer and generator

## Implementation Guidance

### Recommended Patterns

- **Post-order visitor**: Use standard recursive pattern `visit_children_first(node)` then `apply_normalizers(node)`
- **Trait objects**: `Vec<Box<dyn SingleNodeNormalizer>>` for applying normalizers in precedence order
- **Single responsibility**: Each normalizer only recognizes patterns on current node, no tree walking
- **Precedence preservation**: Apply normalizers in current precedence order (High → Medium → Low)

### Tools to Leverage

- **Existing precedence types**: Reuse `PrecedenceLevel` enum from current system
- **Pattern recognition**: Keep successful patterns from current normalizers, just remove recursion
- **Test infrastructure**: All existing normalizer tests should continue passing

### Architecture Considerations

- **Generator compatibility**: Ensure normalized AST nodes remain compatible with existing generator code
- **Performance**: Single tree traversal should be faster than 8 separate passes
- **Maintainability**: New normalizers become trivial to add (just implement SingleNodeNormalizer)

### ExifTool Translation Notes

- **Perl precedence preserved**: Post-order traversal maintains Perl's "innermost first" evaluation
- **Pattern fidelity**: Keep exact same pattern recognition logic, just change orchestration
- **No semantic changes**: This is pure architecture improvement, no behavior changes

## Integration Requirements

### Mandatory Integration Proof

- [ ] **Activation**: New normalizer used by default → `codegen/src/ppi/rust_generator/generator.rs:82` calls `leaves_first::normalize()`
- [ ] **Consumption**: All existing code paths work → `cargo t -p codegen` passes with new architecture
- [ ] **Measurement**: Performance improvement → `cargo bench normalizer_performance` shows faster execution
- [ ] **Cleanup**: Old approach removed → `git log --oneline -5` shows deletion commits

### Integration Verification Commands

**Production Usage Proof**:
- `grep -r "leaves_first::normalize" codegen/src/` → Should show usage in generator.rs
- `rg "AstNormalizer" codegen/src/` → Should return empty after Task F
- `cargo run --bin codegen -- --help` → Should work with new architecture

**Performance Verification**:
- `cargo t test_complex_normalization_patterns -- --nocapture` → Should show single-pass debug output
- Compare before/after: `time cargo t -p codegen` → Should be faster

## Definition of Done

- [ ] `cargo t test_leaves_first_equivalence` passes
- [ ] `cargo t -p codegen` passes (all existing tests work)
- [ ] `make precommit` clean
- [ ] All 8 normalizers simplified to single-node transformers
- [ ] NestedFunctionNormalizer completely removed
- [ ] Generator pattern matching handles complex function cases
- [ ] Performance improvement measurable
- [ ] Architecture documentation updated

## Additional Gotchas & Tribal Knowledge

- **Generated code still untouchable** → Fix normalizers, not generated output → Always test against real ExifTool patterns
- **Precedence matters critically** → Wrong order breaks Perl semantics → Test with complex nested expressions
- **Pattern matching is proven** → Generator patterns work → Don't overcomplicate with more AST transforms
- **Single-pass is faster** → 8 tree walks → 1 tree walk → Measure performance improvement
- **Debugging becomes easier** → Single node transformation → Easy to isolate which normalizer fails

## Quick Debugging

Stuck? Try these:

1. `RUST_LOG=debug cargo t test_normalizer_specific_pattern -- --nocapture` - See which normalizer applies
2. `rg "pattern_name" codegen/src/ppi/normalizer/passes/` - Find normalizer for specific pattern  
3. `cargo t test_single_node_normalizer_name` - Test individual normalizer in isolation
4. `git show HEAD~1:codegen/src/ppi/normalizer/mod.rs` - Compare before/after architecture changes