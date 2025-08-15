# P15: Refactor PPI Module Structure for Maintainability

## Project Overview

- **Goal**: Refactor PPI module from 5 giant files (4576 total lines) into focused, maintainable modules that follow Simple Design principles and improve developer productivity
- **Problem**: Current structure has 1600-line test files and 900+ line implementation files that mix responsibilities, making maintenance, extension, and navigation extremely difficult for engineers
- **Constraints**: Zero API changes, no behavior changes, all existing tests must continue passing, backward compatibility with strategies/ consumers required

## Context & Foundation

### System Overview

- **PPI Module**: Converts Perl Parsing Interface JSON structures from ExifTool field extraction into Rust source code during codegen. Core responsibility is translating Perl expressions (ValueConv, PrintConv, Condition) into equivalent Rust functions that preserve exact ExifTool semantics
- **Codegen Pipeline**: Raw ExifTool perl → field_extractor.pl → PPI JSON → PPI rust_generator → Generated Rust functions → Runtime execution via ast:: support library
- **Consumer Integration**: Only 2 files in strategies/ import PPI functionality: `tag_kit.rs` (expression processing) and `mod.rs` (function registry), giving us flexibility for internal reorganization

### Key Concepts & Domain Knowledge

- **PPI (Perl Parsing Interface)**: JSON representation of parsed Perl AST structures output by `codegen/scripts/field_extractor.pl`, contains tokenized perl expressions with metadata
- **Expression Types**: Three categories with different return signatures - ValueConv (data transformation), PrintConv (human formatting), Condition (boolean logic for tag variants)
- **AST Normalization**: Recent addition that transforms complex patterns into canonical forms, reducing expression generator complexity from 730+ lines to <250 lines
- **Function Registry**: Deduplication system that generates shared functions for identical Perl expressions across multiple ExifTool modules, organized by AST hash

### Surprising Context

**CRITICAL**: Document non-intuitive aspects that aren't obvious from casual code inspection:

- **Test file dominance**: `rust_generator/tests.rs` is 1600 lines (35% of entire PPI module) and tests completely different functionalities that should be in separate files
- **Normalizer impact**: The normalizer was added specifically to simplify expression generation, meaning `expressions.rs` can be dramatically simplified from its current 701-line state
- **Limited API surface**: Despite 4576 lines of code, only 2 files in strategies/ actually import PPI functionality, indicating most complexity is internal implementation detail
- **File size tool limitations**: Read tool truncates files >2000 lines, making current structure difficult to analyze and debug during development
- **Mixed responsibilities**: `rust_generator/mod.rs` handles function generation, visitor patterns, AND complex parsing logic in a single 655-line file
- **Generated vs. hand-written**: All PPI code is hand-written (unlike src/generated/), but follows patterns that could benefit from clearer separation of concerns

### Foundation Documents

- **Design docs**: [docs/SIMPLE-DESIGN.md](../SIMPLE-DESIGN.md) - Kent Beck's Four Rules that guide this refactoring
- **ExifTool source**: `third-party/exiftool/lib/Image/ExifTool.pm` - Core perl expression patterns we translate
- **Start here**: `codegen/src/ppi/mod.rs` for module overview, then `rust_generator/tests.rs:1-100` to understand expression patterns

### Prerequisites

- **Knowledge assumed**: Understanding of AST visitors, Rust trait system, ExifTool expression format (PrintConv/ValueConv)
- **Setup required**: Working codegen environment with `make codegen` passing

## Work Completed

- ✅ **Structure Analysis** → identified 5 files over 580 lines with `tests.rs` at 1600 lines being primary problem
- ✅ **Consumer Analysis** → confirmed only 2 files in strategies/ import PPI, giving refactoring flexibility
- ✅ **Responsibility Mapping** → documented mixed concerns in each large file (generation + visitor + parsing logic)
- ✅ **Impact Assessment** → verified normalizer reduces expression complexity, enabling simpler post-refactor structure

## TDD Foundation Requirement

### Task 0: Not applicable - pure refactoring with identical behavior

**Success Criteria**: All existing tests continue passing, module structure improved, no functionality changes

**Verification Commands**:
- `cargo t` - All tests pass before and after refactoring
- `make codegen` - Codegen continues working identically  
- `grep -r "use crate::ppi::" codegen/src/strategies/` - Consumer imports remain functional

## Remaining Tasks

### Task A: Split Giant Test File into Focused Modules

**Success Criteria**:

- [ ] **Implementation**: Test organization by functionality → `codegen/src/ppi/rust_generator/tests/` directory with 7 focused modules
- [ ] **File Structure**: Tests split by concern → `arithmetic.rs` (50-80 lines), `string_ops.rs` (40-70 lines), `functions.rs` (80-120 lines), `conditionals.rs` (60-90 lines), `control_flow.rs` (40-60 lines), `complex_patterns.rs` (80-150 lines), `mod.rs` (20-40 lines utilities)
- [ ] **Integration**: All tests discoverable → `cargo t rust_generator` runs all test modules
- [ ] **Cleanup**: Giant file removed → `git rm codegen/src/ppi/rust_generator/tests.rs` and verify `wc -l tests/` shows no file over 150 lines
- [ ] **Verification**: No test regression → `cargo t` passes with identical output to baseline
- [ ] **Navigation**: Improved discoverability → `find codegen/src/ppi/rust_generator/tests/ -name "*.rs" | wc -l` returns 7

**Implementation Details**: Group by functionality - arithmetic tests (+,-,*,/,%) go to arithmetic.rs, sprintf/length/unpack to functions.rs, string concatenation to string_ops.rs, ternary operators to conditionals.rs, tr/// operations to control_flow.rs, multi-statement blocks to complex_patterns.rs

**Integration Strategy**: Update `rust_generator/mod.rs` to declare `tests` submodule, move shared utilities to `tests/mod.rs`

**Validation Plan**: `cargo t` identical before/after, `cargo t arithmetic` runs subset correctly

**Dependencies**: None

### Task B: Extract Core Visitor Logic into Focused Modules

**Success Criteria**:

- [ ] **Implementation**: Visitor split by responsibility → `codegen/src/ppi/rust_generator/visitor/` directory with 6 focused modules under 200 lines each
- [ ] **File Structure**: Clear boundaries → `core.rs` (PpiVisitor trait), `document.rs` (document/statement processing), `tokens.rs` (numbers/strings/symbols), `operators.rs` (binary/unary), `structures.rs` (lists/blocks), `mod.rs` (public API)
- [ ] **Integration**: Visitor functionality preserved → `cargo t visitor` runs all visitor tests, no API changes to PpiVisitor trait
- [ ] **Cleanup**: Giant file removed → `git rm codegen/src/ppi/rust_generator/visitor.rs` and verify `wc -l visitor/` shows no file over 200 lines
- [ ] **API Preservation**: Consumer compatibility → `grep -r "PpiVisitor" codegen/src/strategies/` imports still work
- [ ] **Navigation**: Clear entry points → `visitor/mod.rs` exports all traits, implementation split by token type

**Implementation Details**: Move trait definition to core.rs, document/statement handling to document.rs, all token visitors (visit_symbol, visit_number, etc.) to tokens.rs, operator logic to operators.rs, complex structures to structures.rs

**Integration Strategy**: Maintain `pub use visitor::*` in rust_generator/mod.rs for backward compatibility

**Validation Plan**: `cargo t` passes, `cargo check --package codegen` confirms no import errors

**Dependencies**: Task A complete (test structure established)

### Task C: Simplify Main Generator by Extracting Concerns

**Success Criteria**:

- [ ] **Implementation**: Generator concerns separated → `rust_generator/generator.rs` (120-180 lines), `signature.rs` (40-80 lines), `pattern_matching.rs` (80-150 lines), `mod.rs` (30-60 lines)
- [ ] **File Structure**: Single responsibility → `generator.rs` has core RustGenerator and function generation, `signature.rs` handles type-specific signatures, `pattern_matching.rs` has pack/map and complex pattern detection
- [ ] **Integration**: API unchanged → `RustGenerator::new()` and `generate_function()` work identically from consumer perspective
- [ ] **Cleanup**: Giant mod.rs reduced → `wc -l rust_generator/mod.rs` under 60 lines, complex logic moved to appropriate modules
- [ ] **API Preservation**: Public interface intact → `pub use generator::RustGenerator` maintains import compatibility
- [ ] **Behavior**: Identical output → `make codegen` produces identical generated files

**Implementation Details**: Extract RustGenerator struct to generator.rs, move signature generation logic to signature.rs, move extract_pack_map_pattern and complex parsing to pattern_matching.rs

**Integration Strategy**: Keep public API exports in mod.rs, use internal imports between new modules

**Validation Plan**: `make codegen && git diff src/generated/` shows no changes, `cargo t` passes

**Dependencies**: Task B complete (visitor structure established)

### Task D: Split Expression Combiner by Operation Type

**Success Criteria**:

- [ ] **Implementation**: Expression logic split → `expressions/binary_ops.rs` (150-200 lines), `string_ops.rs` (100-150 lines), `normalized.rs` (80-120 lines), `patterns.rs` (120-180 lines), `mod.rs` (40-80 lines)
- [ ] **File Structure**: Operation-focused → Binary operators and comparisons in binary_ops.rs, concatenation/regex in string_ops.rs, normalized AST handling in normalized.rs, complex patterns in patterns.rs
- [ ] **Integration**: ExpressionCombiner trait preserved → All trait methods work identically, no consumer changes needed
- [ ] **Cleanup**: Giant expressions.rs removed → `git rm codegen/src/ppi/rust_generator/expressions.rs` and verify new files under target sizes
- [ ] **API Preservation**: Trait compatibility → `ExpressionCombiner` imports in strategies/ continue working
- [ ] **Behavior**: Expression generation identical → Generated function output unchanged for all expression types

**Implementation Details**: Move binary operator logic (perl_to_rust_operator, handle_binary_operation) to binary_ops.rs, string concatenation and regex to string_ops.rs, normalized AST handlers to normalized.rs, complex patterns to patterns.rs

**Integration Strategy**: Re-export ExpressionCombiner from expressions/mod.rs, maintain trait implementation structure

**Validation Plan**: `make codegen && cargo t` passes, expression generation produces identical output

**Dependencies**: Task C complete (generator structure finalized)

### Task E: Optimize Function Registry by Separating Concerns

**Success Criteria**:

- [ ] **Implementation**: Registry split by concern → `fn_registry/registry.rs` (200-250 lines), `stats.rs` (80-120 lines), `generation.rs` (180-220 lines), `fallback.rs` (120-160 lines), `mod.rs` (40-60 lines)
- [ ] **File Structure**: Clear boundaries → Core registry logic in registry.rs, statistics tracking in stats.rs, code generation in generation.rs, fallback handling in fallback.rs
- [ ] **Integration**: PpiFunctionRegistry API unchanged → All public methods preserved, strategies/ consumers work identically
- [ ] **Cleanup**: Giant fn_registry.rs removed → `git rm codegen/src/ppi/fn_registry.rs` and verify new structure under line limits
- [ ] **API Preservation**: Public interface intact → `PpiFunctionRegistry::new()` and `register_ast()` signatures unchanged
- [ ] **Behavior**: Function generation identical → `make codegen` produces same function files

**Implementation Details**: Move core PpiFunctionRegistry struct and AST registration to registry.rs, ConversionStats and RegistryStats to stats.rs, function file generation to generation.rs, impl_registry fallback logic to fallback.rs

**Integration Strategy**: Maintain public exports through fn_registry/mod.rs, preserve all existing method signatures

**Validation Plan**: `make codegen && ls src/generated/functions/` shows identical file structure, `cargo t` passes

**Dependencies**: Task D complete (expression system stable)

### Task F: Split Normalizer Passes into Individual Files

**Success Criteria**:

- [ ] **Implementation**: One pass per file → `normalizer/passes/safe_division.rs` (40-60 lines), `function_calls.rs` (60-80 lines), `string_ops.rs` (100-140 lines), `mod.rs` (30-50 lines)
- [ ] **File Structure**: Single responsibility → Each normalization pass in its own file, clear pass ordering in mod.rs
- [ ] **Integration**: Normalizer API unchanged → `normalize()` function works identically, pass ordering preserved
- [ ] **Cleanup**: Giant passes.rs removed → `git rm codegen/src/ppi/normalizer/passes.rs` and verify new files under 140 lines
- [ ] **API Preservation**: Public interface intact → `normalize()` import still works from rust_generator
- [ ] **Behavior**: AST normalization identical → Generated functions unchanged after normalization

**Implementation Details**: Extract SafeDivisionNormalizer to safe_division.rs, FunctionCallNormalizer to function_calls.rs, StringOpNormalizer to string_ops.rs, maintain NormalizationPass trait in each file

**Integration Strategy**: Re-export all passes from normalizer/passes/mod.rs, preserve pass ordering in AstNormalizer::new()

**Validation Plan**: `cargo t normalizer` passes, `make codegen` produces identical output

**Dependencies**: Task E complete (all major modules refactored)

## Integration Requirements

### Mandatory Integration Proof

Every refactoring task must include specific evidence of API preservation:

- [ ] **API Compatibility**: Existing consumer imports work → `cargo check --package codegen` passes
- [ ] **Behavioral Preservation**: Generated code unchanged → `make codegen && git diff src/generated/` shows no differences  
- [ ] **Test Preservation**: All tests pass → `cargo t` identical output before/after each task
- [ ] **Import Verification**: Strategy consumers unchanged → `grep -r "use crate::ppi::" codegen/src/strategies/` continues working

### Integration Verification Commands

**API Preservation Proof**:
- `cargo check --package codegen` → Should pass after each task completion
- `grep -r "pub use" codegen/src/ppi/` → Should show all public exports maintained
- `make codegen` → Should complete successfully with identical generated output

**Behavior Preservation Test**:
- ❌ **Structure only**: "Files are smaller and better organized"
- ✅ **Integrated**: "Files are smaller, better organized, AND all existing functionality works identically"

## Implementation Guidance

### Recommended Patterns

- **Module boundaries**: Split by functionality (tests by operation type, visitor by token type, etc.), not by file size
- **API preservation**: Always maintain `pub use` exports in parent mod.rs files to avoid breaking consumer imports
- **Trait implementations**: Keep trait implementations with trait definitions when possible for cohesion
- **File size targets**: Aim for 50-200 lines per file for optimal navigation and maintenance

### Tools to Leverage

- **wc -l**: Verify file sizes stay under targets
- **cargo check**: Ensure no import errors after refactoring
- **git mv**: Preserve file history when moving code between files
- **rg/grep**: Verify all references updated when moving public items

### Architecture Considerations

- **Backward compatibility**: Consumers in strategies/ must continue working without changes
- **Module hierarchy**: Keep logical grouping (rust_generator/, normalizer/, etc.) while splitting large files
- **Trait coherence**: Keep related traits and implementations in same module when possible
- **Public API surface**: Minimize changes to pub use exports during refactoring

### ExifTool Translation Notes

- **No translation changes**: This refactoring doesn't affect Perl→Rust translation logic
- **Expression semantics**: All expression generation must produce identical Rust code
- **Normalization preservation**: AST normalization passes must maintain exact transformation behavior

## Working Definition of "Complete"

A refactoring task is complete when:

- ✅ **File structure improved** - Target file sizes achieved, clear module boundaries established
- ✅ **API preserved** - All consumer imports continue working without modification
- ✅ **Behavior identical** - Generated code output unchanged, all tests pass
- ✅ **Navigation enhanced** - Engineers can find specific functionality quickly in focused modules
- ❌ Code moved but imports broken _(example: "files split but strategies/ can't import PpiVisitor")_
- ❌ Structure improved but behavior changed _(example: "better organization but generated functions different")_

## Prerequisites

None - this is foundational refactoring that enables future PPI enhancements.

## Testing

- **Unit**: Test file organization and module boundaries
- **Integration**: Verify codegen pipeline continues working identically  
- **Manual check**: Run `make codegen && cargo t` and confirm identical behavior

## Definition of Done

- [ ] `cargo t` passes with identical output to baseline
- [ ] `make codegen` produces identical generated files
- [ ] `wc -l codegen/src/ppi/**/*.rs` shows no file over 250 lines
- [ ] Consumer imports in strategies/ work unchanged
- [ ] File structure follows Clear module boundaries by responsibility

## Additional Gotchas & Tribal Knowledge

- **src/generated/** immunity → PPI generates functions but doesn't live in generated/ → Always preserve hand-written nature of PPI code
- **Normalizer reduces complexity** → expressions.rs can be much simpler than current 701 lines → Leverage normalizer patterns when splitting expression logic
- **Only 2 consumers** → strategies/tag_kit.rs and strategies/mod.rs are only external users → Internal reorganization has minimal blast radius
- **Test discovery** → Rust test discovery works recursively → Moving tests to subdirectories automatically discovered by `cargo t`
- **File size matters for tools** → Read tool truncates at 2000 lines → Target 50-200 lines per file for optimal development experience
- **Public API preservation critical** → strategies/ imports must continue working → Always maintain backward-compatible exports

## Quick Debugging

Stuck during refactoring? Try these:

1. `cargo check --package codegen` - Find import errors immediately
2. `wc -l codegen/src/ppi/**/*.rs` - Verify file size progress
3. `make codegen && git diff src/generated/` - Confirm behavior preservation  
4. `grep -r "use crate::ppi::" codegen/src/strategies/` - Check consumer compatibility