# P07: Expand PPI Token Support for Rust Code Generation

## Project Overview

- **Goal**: Expand PPI token support from 20% to 90%+ expression coverage, enabling automatic Rust code generation for thousands of ExifTool expressions
- **Problem**: Our current `rust_generator.rs` only supports 9 out of 35 PPI token types, leaving 4,000+ expressions unconvertible and requiring manual implementation
- **Constraints**: Must preserve exact ExifTool semantics, maintain codegen performance, ensure generated Rust is idiomatic and safe

## Context & Foundation

### System Overview

- **PPI Pipeline**: Perl expressions → PPI AST JSON → Rust code generation → Runtime execution
- **Field Extractor**: `field_extractor_with_ast.pl` parses ExifTool modules and extracts `PrintConv`, `ValueConv`, and `Condition` expressions as PPI AST structures
- **Rust Generator**: `rust_generator.rs` converts PPI AST nodes to Rust functions that call runtime support libraries 
- **Function Registry**: `fn_registry.rs` deduplicates identical AST structures across modules to prevent code bloat
- **Tag Kit Integration**: Generated functions are wired into tag processing pipeline for automatic execution

### Key Concepts & Domain Knowledge

- **PPI (Perl Parsing Interface)**: Creates Abstract Syntax Tree from Perl code without executing it, enabling semantic analysis and transformation
- **ExifTool Expression Types**: `Condition` (boolean logic for tag variants), `ValueConv` (raw value transformation), `PrintConv` (human-readable formatting)
- **Trust ExifTool Principle**: Generated Rust must produce identical results to original Perl - no "improvements" or optimizations that change behavior
- **AST Deduplication**: Identical expression structures share single Rust function to minimize binary size and compilation time

### Surprising Context

- **PPI Structure Complexity**: Many simple-looking Perl expressions parse into complex nested AST structures with 10+ token types
- **Self-Reference Pattern**: `$$self{Model}` is the most common pattern (2,420 occurrences) requiring `PPI::Token::Cast` + `PPI::Structure::Subscript` support
- **Expression Containers**: Most actual logic lives in `PPI::Statement::Expression` nodes (4,172 occurrences) that we don't currently handle
- **Regex Prevalence**: Pattern matching (`=~`, `!~`) appears in 731 expressions but we have zero regex support
- **Hidden Performance Impact**: Current 80% parse failure rate means manual fallbacks dominate execution paths
- **Function Chain Complexity**: Expressions like `unpack "H*", pack "C*", split " ", $val` generate invalid Rust (still Perl syntax) because we don't handle comma-separated function composition
- **Type System Gaps**: Generated arithmetic like `val / 256` fails to compile because TagValue doesn't implement arithmetic operations
- **Partial Conversion Failure**: Current implementation can parse complex expressions but generates non-compiling Rust, making the feature unusable in practice

### Foundation Documents

- **Analysis Results**: `docs/analysis/ppi-token-analysis.md` - Complete frequency analysis of 5,056 expressions from 49 modules
- **PPI Guide**: `docs/guides/PPI-GUIDE.md` - Comprehensive guide to PPI token types and patterns
- **Trust ExifTool**: `docs/TRUST-EXIFTOOL.md` - Core principle for implementation decisions
- **Current Implementation**: `codegen/src/ppi/rust_generator.rs:94-105` - Existing token support in `visit_node()`

### Prerequisites

- **Knowledge assumed**: Understanding of AST traversal, Rust code generation, regex compilation
- **Setup required**: ExifTool modules must be patched (`codegen/scripts/exiftool-patcher.sh` completed)
- **Performance baseline**: Current PPI parsing overhead acceptable for codegen phase

## Work Completed

- ✅ **Comprehensive Analysis** → analyzed 5,056 expressions from 49 modules → chose frequency-based prioritization over complexity-based because impact is measurable
- ✅ **Token Frequency Data** → identified 27 unsupported token types → rejected alphabetical implementation order due to low-impact tokens
- ✅ **Impact Assessment** → ranked tokens by occurrence count → chose 4-phase approach over single-phase due to engineering resource constraints

## TDD Foundation Requirement

### Task 0: Integration Test

**Purpose**: Ensure PPI enhancement delivers measurable improvement in expression conversion rates with verifiable end-to-end functionality.

**Success Criteria**:

- [ ] **Test exists**: `tests/integration_p07_ppi_enhancement.rs:test_ppi_coverage_improvement`
- [ ] **Test fails**: `cargo t test_ppi_coverage_improvement` fails showing current ~20% conversion rate
- [ ] **Integration focus**: Test validates actual tag processing uses generated PPI functions, not just unit functionality  
- [ ] **TPP reference**: Test includes comment `// P07: PPI Enhancement - see docs/todo/P07-ppi-enhancement.md`
- [ ] **Measurable outcome**: Test demonstrates conversion rate improvement after each phase (P1: 60%, P2: 75%, P3: 85%, P4: 90%+)

**Requirements**:

- Must test real ExifTool expressions from our configured modules
- Should fail because critical tokens like `PPI::Statement::Expression` aren't supported
- Must test that generated code actually compiles (not just generates)
- Must demonstrate generated functions execute correctly in tag processing pipeline
- Should test specific problematic patterns: function chains, arithmetic operations, regex matching
- Include error message: `"// Fails until P07 complete - requires PPI::Statement::Expression, PPI::Token::Cast, PPI::Structure::Subscript support and TagValue arithmetic ops"`

**Test Cases Must Include**:
- `unpack "H*", pack "C*", split " ", $val` → Should generate compiling Rust (currently generates invalid Perl syntax)
- `$val / 256` → Should compile with TagValue arithmetic (currently fails to compile)
- `$$self{Model} =~ /Canon/` → Should handle cast + subscript + regex (currently unsupported tokens)

## Remaining Tasks

### Task A: Implement Critical Foundation Tokens (Phase 1)

**Success Criteria**:

- [ ] **Implementation**: PPI::Statement::Expression handler → `codegen/src/ppi/rust_generator.rs:110-140` implements `visit_expression()`
- [ ] **Implementation**: PPI::Token::Cast handler → `codegen/src/ppi/rust_generator.rs:155-180` implements `visit_cast()`  
- [ ] **Implementation**: PPI::Structure::Subscript handler → `codegen/src/ppi/rust_generator.rs:185-220` implements `visit_subscript()`
- [ ] **Implementation**: PPI::Token::Regexp::Match handler → `codegen/src/ppi/rust_generator.rs:225-260` implements `visit_regexp_match()`
- [ ] **Integration**: visit_node() dispatches new tokens → `codegen/src/ppi/rust_generator.rs:94-108` includes new cases
- [ ] **Task 0 passes**: `cargo t test_ppi_coverage_improvement` shows 60% conversion rate for Phase 1 expressions
- [ ] **Unit tests**: `cargo t test_critical_ppi_tokens` passes for all 4 token types
- [ ] **Manual validation**: `RUST_LOG=debug cargo run --bin codegen` shows successful conversion of `$$self{Model} =~ /Canon/` pattern
- [ ] **Cleanup**: No old debug code remains → `grep -r "TODO.*PPI" codegen/src/` returns empty
- [ ] **Documentation**: Types.rs helper methods → `codegen/src/ppi/types.rs:200-250` implements is_cast(), is_subscript(), etc.

**Implementation Details**: 
- **Expression**: Process children recursively, handle complex nesting and function composition patterns like `unpack "H*", pack "C*", split " ", $val`
- **Cast**: Map `$$self{Field}` → `ctx.get("Field").unwrap_or_default()`, handle array/hash derefs
- **Subscript**: Generate bounds-checked indexing with proper error handling
- **Regexp**: Compile patterns at runtime, cache compiled regexes for performance

**Critical Pattern Handling**:
- **Complex Function Chains**: `unpack "H*", pack "C*", split " ", $val` → Parse comma-separated functions, compose operations correctly
- **Type Safety**: Ensure all generated Rust code compiles with proper TagValue conversions
- **Expression Nesting**: Handle parenthetical grouping and operator precedence correctly

**Integration Strategy**: Wire into existing `visit_node()` dispatcher, ensure fn_registry deduplication works

**Validation Plan**: Test with top 20 most frequent expression patterns from analysis data, including complex chains

**Dependencies**: Task A1 (TagValue arithmetic ops) complete

**Success Patterns**:
- ✅ Canon.pm model detection expressions convert successfully to working Rust functions
- ✅ Generated code handles both `$$self{Model}` and `$val[0]` access patterns correctly  
- ✅ Regex compilation happens once per function, not per execution
- ✅ Complex function chains like `join " ", unpack "H2H2", $val` generate valid, compiling Rust code

### Task A1: Implement TagValue Arithmetic Operations (Foundation Prerequisite)

**Success Criteria**:

- [ ] **Implementation**: TagValue arithmetic traits → `src/types/mod.rs:200-350` implements Div, Mul, Add, Sub for TagValue
- [ ] **Implementation**: Type coercion logic → `src/types/mod.rs:355-400` handles string→number conversion automatically
- [ ] **Integration**: Generated code compiles → `$val / 256` expressions produce valid Rust that compiles
- [ ] **Unit tests**: `cargo t test_tagvalue_arithmetic` passes for all arithmetic operations
- [ ] **Manual validation**: `cargo run --bin codegen` generates compiling code for `$val * 25.4 / 1000` expressions
- [ ] **Cleanup**: No type errors in generated code → `cargo check --bin codegen` passes cleanly
- [ ] **Documentation**: Arithmetic semantics documented → `docs/design/TAGVALUE-ARITHMETIC.md` explains conversion rules

**Implementation Details**:
```rust
// Handle arithmetic with automatic type coercion
impl std::ops::Div<i32> for &TagValue {
    type Output = TagValue;
    fn div(self, rhs: i32) -> TagValue {
        match self {
            TagValue::I32(v) => TagValue::I32(v / rhs),
            TagValue::F64(v) => TagValue::F64(v / rhs as f64),
            TagValue::String(s) => {
                if let Ok(num) = s.parse::<f64>() {
                    TagValue::F64(num / rhs as f64)
                } else {
                    // Preserve ExifTool's behavior for non-numeric strings
                    TagValue::String(format!("({} / {})", s, rhs))
                }
            }
        }
    }
}
```

**Integration Strategy**: Required before any arithmetic expressions can compile in generated code

**Validation Plan**: Test with real Canon.pm expressions that use division, multiplication, etc.

**Dependencies**: None - this enables everything else

**Success Patterns**:
- ✅ `$val / 100` compiles and executes correctly
- ✅ String-to-number coercion matches ExifTool behavior exactly
- ✅ Non-numeric strings handle gracefully without panics

### Task B: Implement Numeric & String Operations (Phase 2)

**Success Criteria**:

- [ ] **Implementation**: PPI::Token::Number::Hex handler → `codegen/src/ppi/rust_generator.rs:265-285` implements `visit_number_hex()`
- [ ] **Implementation**: PPI::Statement::Variable handler → `codegen/src/ppi/rust_generator.rs:290-320` implements `visit_variable()`
- [ ] **Implementation**: PPI::Token::Regexp::Substitute handler → `codegen/src/ppi/rust_generator.rs:325-360` implements `visit_regexp_substitute()`
- [ ] **Implementation**: Enhanced float handling → `codegen/src/ppi/rust_generator.rs:170-185` improves `visit_number()` for floats
- [ ] **Integration**: All numeric/string patterns work → `grep -r "0x\|\..*f\|my.*=" test-images/` expressions convert successfully
- [ ] **Task 0 passes**: `cargo t test_ppi_coverage_improvement` shows 75% conversion rate for Phase 1+2 expressions
- [ ] **Unit tests**: `cargo t test_numeric_string_ppi_tokens` passes for hex, variables, regex substitution
- [ ] **Manual validation**: `cargo run --bin codegen` converts `$val * 25.4 / 1000` and `my @a = split " ", $val` patterns
- [ ] **Cleanup**: Enhanced number handling replaces basic version → commit shows refactoring of existing `visit_number()`
- [ ] **Documentation**: Variable binding patterns documented → `docs/guides/PPI-GUIDE.md:350-380` shows Rust variable generation

**Implementation Details**:
- Hex numbers: Direct Rust hex literal generation with type inference
- Variables: Map `my @array = split()` to appropriate Rust bindings with proper scoping
- Regex substitute: Generate string replacement operations with capture group support
- Enhanced floats: Improve precision handling and scientific notation support

**Integration Strategy**: Build on Phase 1 foundation, ensure type system consistency

**Validation Plan**: Test with Canon arithmetic expressions and string manipulation patterns

**Dependencies**: Task A complete

**Success Patterns**:
- ✅ Canon.pm color temperature calculations (hex arithmetic) work correctly
- ✅ Multi-step variable assignments generate readable, safe Rust code
- ✅ String cleaning patterns (s/\xff+$//) execute with identical results to ExifTool

### Task C: RESEARCH - Analyze Composite Tag Dependencies

**Objective**: Identify which tags referenced by Composite expressions are missing from `config/supported_tags.json`

**Success Criteria**: 
- [ ] **Research complete**: Composite dependency analysis → `docs/analysis/composite-dependencies.md` lists unsupported required tags
- [ ] **Implementation plan**: Mitigation strategy → Same document outlines approach for handling missing dependencies
- [ ] **ExifTool verification**: Dependency patterns confirmed → `./exiftool -Composite test-images/*.jpg` validates analysis

**Done When**: Clear list of missing tags and strategy for handling them exists

### Task D: Implement Control Flow & Advanced Features (Phase 3)

**Success Criteria**:

- [ ] **Implementation**: PPI::Token::Magic handler → `codegen/src/ppi/rust_generator.rs:365-390` implements `visit_magic()` for `$_`, `$@`
- [ ] **Implementation**: PPI::Statement::Break handler → `codegen/src/ppi/rust_generator.rs:395-415` implements `visit_break()` for return/last/next
- [ ] **Implementation**: PPI::Token::Regexp::Transliterate handler → `codegen/src/ppi/rust_generator.rs:420-450` implements `visit_transliterate()`
- [ ] **Implementation**: PPI::Structure::Block handler → `codegen/src/ppi/rust_generator.rs:455-480` implements `visit_block()` for closures
- [ ] **Integration**: Control flow patterns work → Complex conditional expressions with returns convert successfully
- [ ] **Task 0 passes**: `cargo t test_ppi_coverage_improvement` shows 85% conversion rate for Phase 1+2+3 expressions
- [ ] **Unit tests**: `cargo t test_control_flow_ppi_tokens` passes for magic vars, breaks, transliteration, blocks
- [ ] **Manual validation**: `cargo run --bin codegen` converts expressions with `return $val` and `tr/a-z/A-Z/` patterns
- [ ] **Cleanup**: No TODO comments for medium-priority tokens → `grep -r "TODO.*magic\|TODO.*break" codegen/src/` returns empty
- [ ] **Documentation**: Control flow mapping documented → `docs/guides/PPI-GUIDE.md:400-450` shows Perl→Rust control flow

**Implementation Details**:
- Magic variables: Map `$_` to context-appropriate values, handle special cases
- Break statements: Generate appropriate Rust control flow (return, break, continue)
- Transliterate: Implement character-by-character replacement operations
- Blocks: Handle closure-like constructs for map/grep operations

**Integration Strategy**: Build on Phase 1+2, ensure error handling consistency

**Validation Plan**: Test with complex conditional expressions from Nikon.pm and Sony.pm

**Dependencies**: Task A complete, Task B complete

**Success Patterns**:
- ✅ Complex multi-branch conditionals generate correct Rust match/if expressions
- ✅ Character translation operations produce identical output to Perl tr/// operator
- ✅ Early returns and loop control work correctly in generated functions

### Task E: Achieve 90%+ Coverage (Phase 4)

**Success Criteria**:

- [ ] **Implementation**: Remaining low-frequency tokens → `codegen/src/ppi/rust_generator.rs:485-550` implements handlers for remaining 9 token types
- [ ] **Integration**: Edge cases handled → Unusual expression patterns from analysis data convert successfully
- [ ] **Task 0 passes**: `cargo t test_ppi_coverage_improvement` shows 90%+ conversion rate for all expression types
- [ ] **Unit tests**: `cargo t test_comprehensive_ppi_coverage` passes for all 27 previously unsupported token types
- [ ] **Manual validation**: `cargo run --bin codegen` successfully processes all modules with <5% expression fallback rate
- [ ] **Cleanup**: All placeholder TODOs removed → `grep -r "UnsupportedToken\|TODO.*PPI" codegen/src/` returns empty
- [ ] **Documentation**: Complete token reference → `docs/guides/PPI-GUIDE.md:500-600` documents all supported tokens with examples

**Implementation Details**: Handle remaining tokens with appropriate fallback strategies and error handling

**Integration Strategy**: Comprehensive testing with all 49 modules, performance validation

**Validation Plan**: Full regression test suite, comparison with ExifTool output across all supported tags

**Dependencies**: Task A complete, Task B complete, Task D complete

**Success Patterns**:
- ✅ Conversion rate exceeds 90% across all expression types
- ✅ Generated code performance meets or exceeds manual implementations
- ✅ Zero regressions in existing functionality

### Task F: Performance Optimization and Documentation

**Success Criteria**:

- [ ] **Implementation**: Performance optimizations → `codegen/src/ppi/fn_registry.rs:200-250` implements caching and batch processing
- [ ] **Integration**: Build time impact measured → Codegen phase adds <20% overhead vs current implementation
- [ ] **Manual validation**: `time cargo run --bin codegen` completes all modules in reasonable time
- [ ] **Cleanup**: Debug logging optimized → Only essential traces remain in production paths
- [ ] **Documentation**: Integration guide complete → `docs/guides/PPI-INTEGRATION.md` exists with usage examples

**Implementation Details**: Optimize AST parsing, implement regex compilation caching, batch function generation

**Integration Strategy**: Profile codegen performance, optimize bottlenecks

**Validation Plan**: Performance regression tests, memory usage monitoring

**Dependencies**: Task E complete

**Success Patterns**:
- ✅ Codegen time scales linearly with module count, not expression complexity
- ✅ Generated binary size growth is manageable (<50% increase for 4x expression coverage)
- ✅ Developer workflow remains smooth with enhanced PPI support

## Task Completion Standards

**RULE**: No checkbox can be marked complete without specific proof.

### Required Evidence Types

- **Code references**: `rust_generator.rs:line_range` where new handlers exist
- **Passing commands**: `cargo t test_name` or `cargo run --bin codegen` success 
- **Integration proof**: `grep -r "visit_expression\|visit_cast" codegen/src/` shows usage in dispatcher
- **Conversion rate**: Specific percentage improvement in Task 0 test results
- **Output validation**: Before/after examples showing generated Rust code quality

### ❌ Common Incomplete Patterns

**Implementation without Integration**:
- "Token handler implemented but visit_node() unchanged" → Missing dispatcher integration
- "Unit tests pass but Task 0 still fails" → Handler works in isolation but not in pipeline
- "Generated functions exist but tag processing doesn't use them" → No production consumption

**Testing without Validation**:
- "Tests pass but conversion rate unchanged" → Tests don't exercise real functionality  
- "Manual testing shows it works" → No automated regression protection
- "Code compiles but ExifTool comparison fails" → Output doesn't match reference implementation

## Implementation Guidance

### Recommended Patterns

- **Token Handler Structure**: Follow existing `visit_symbol()` pattern - validate inputs, generate Rust AST, handle errors gracefully
- **Error Handling**: Prefer graceful degradation to hard failures - unknown constructs should generate commented fallbacks
- **Rust Code Generation**: Use `format!()` for simple cases, proper AST building for complex expressions
- **Testing Strategy**: Each token type needs unit tests AND integration tests with real ExifTool expressions

### Tools to Leverage

- **PPI::Dumper**: Use for debugging complex expression parsing during development
- **ExifTool comparison**: `./compare-with-exiftool.sh` validates generated output matches reference
- **AST analysis**: Extend `types.rs` helper methods for pattern recognition
- **Regex compilation**: Consider lazy_static or once_cell for expensive regex compilation

### Architecture Considerations

- **Function Deduplication**: Ensure `fn_registry.rs` handles new token patterns correctly
- **Type Safety**: Generated Rust should be type-safe and follow Rust idioms
- **Performance**: PPI parsing happens at codegen time, so optimization focus is on generated code
- **Maintainability**: Code should be readable and debuggable by future engineers

### ExifTool Translation Notes

- **Preserve Semantics**: Generated Rust must produce identical results - measure with comparison tests
- **Handle Edge Cases**: ExifTool has 25 years of bug fixes - don't "improve" the logic
- **Context Variables**: `$$self`, `$val`, `$valPt` map to specific Rust contexts
- **Error Behavior**: Match ExifTool's error handling patterns exactly

## Integration Requirements

### Mandatory Integration Proof

Every implementation must include specific evidence of integration:

- [ ] **Activation**: New tokens used automatically → `rust_generator.rs:visit_node()` dispatches to new handlers by default
- [ ] **Consumption**: Generated functions called in production → `grep -r "ast_.*_[0-9a-f]" src/generated/` shows function usage
- [ ] **Measurement**: Conversion rate improvement measurable → Task 0 test shows percentage increase
- [ ] **Cleanup**: Old fallback paths removed → Expressions that previously failed now generate Rust code

### Integration Verification Commands

**Production Usage Proof**:
- `cargo run --bin codegen` → Should show dramatically improved conversion rates
- `grep -r "visit_expression\|visit_cast" codegen/src/` → Should show integration in dispatcher
- `cargo t test_ppi_coverage_improvement` → Should demonstrate measurable improvement

**Integration vs Implementation Test**:
- ❌ **Implementation only**: "Handler works when called directly with test AST"
- ✅ **Integrated**: "Real ExifTool expressions automatically generate working Rust functions"

## Working Definition of "Complete"

A PPI enhancement is complete when:

- ✅ **Conversion rate improves** - measurably more expressions convert to Rust successfully
- ✅ **Generated code works** - Rust functions produce identical output to original Perl
- ✅ **Integration automatic** - No manual intervention needed for new expressions using supported tokens
- ❌ Handler exists but expressions still fall back to manual implementation
- ❌ Unit tests pass but real-world conversion rate unchanged

## Prerequisites

- [ExifTool Patching] → [codegen/scripts/exiftool-patcher.sh completed] → verify with `grep -c "# EXIF-OXIDE PATCHED" third-party/exiftool/lib/Image/ExifTool/*.pm`

## Testing

- **Unit**: Test each token type handler with synthetic AST structures
- **Integration**: Verify real ExifTool expressions convert and execute correctly  
- **Performance**: Measure codegen overhead and generated code performance
- **Regression**: Ensure existing functionality continues working
- **Comparison**: Validate generated output matches ExifTool exactly

## Definition of Done

- [ ] `cargo t test_ppi_coverage_improvement` shows 90%+ conversion rate
- [ ] `make precommit` clean with all new code
- [ ] Generated Rust functions produce output identical to ExifTool for test cases
- [ ] Codegen performance acceptable (<50% overhead increase)
- [ ] Documentation complete and accurate for all new token types