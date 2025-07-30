# P20d: Migrate Expression System to RPN Architecture

Migrate the current AST-based predicate expression system to RPN (Reverse Polish Notation) for architectural consistency with the existing math expression compiler.

---

## ⚠️ CRITICAL REMINDERS

If you read this document, you **MUST** read and follow [CLAUDE.md](../CLAUDE.md) as well as [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md):

- **Trust ExifTool** (Translate and cite references, but using codegen is preferred)
- **Ask clarifying questions** (Maximize "velocity made good")
- **Assume Concurrent Edits** (STOP if you find a compilation error that isn't related to your work)
- **Don't edit generated code** (read [CODEGEN.md](CODEGEN.md) if you find yourself wanting to edit `src/generated/**.*rs`)
- **Keep documentation current** (so update this TPP with status updates, and any novel context that will be helpful to future engineers that are tasked with completing this TPP. Do not use hyperbolic "DRAMATIC IMPROVEMENT"/"GROUNDBREAKING PROGRESS" styled updates -- that causes confusion and partially-completed low-quality work)

**NOTE**: These rules prevent build breaks, data corruption, and wasted effort across the team. 

If you are found violating any topics in these sections, **your work will be immediately halted, reverted, and you will be dismissed from the team.**

Honest. RTFM.

---

## Project Overview

- **Goal**: Achieve architectural consistency by migrating predicate expressions from AST to RPN, matching the existing math expression compiler design
- **Problem**: Mixed paradigms (RPN for math expressions, AST for predicates) create cognitive overhead and inconsistent patterns across the codebase
- **Constraints**: Zero behavior changes, maintain ExifTool compatibility, preserve all existing functionality and performance

## Context & Foundation

**REQUIRED**: Assume reader is unfamiliar with this domain. Provide comprehensive context.

### System Overview

- **Expression evaluation system**: Handles conditional logic for ExifTool compatibility - determines when to apply specific parsers, tag processors, and value conversions based on camera metadata context
- **Math expression compiler**: `codegen/src/expression_compiler.rs` compiles simple arithmetic (`$val / 8`) to inline Rust code using RPN and Shunting Yard algorithm  
- **Predicate expression evaluator**: `src/expressions/` currently uses AST parsing to evaluate boolean logic (`$manufacturer eq 'Canon' and $model =~ /EOS/`) for processor selection and conditional tags

### Key Concepts & Domain Knowledge

- **RPN (Reverse Polish Notation)**: Stack-based expression representation where operators follow operands (`3 4 +` instead of `3 + 4`), eliminates need for parentheses and precedence during evaluation
- **Shunting Yard Algorithm**: Converts infix expressions to RPN using operator precedence tables, already proven in math compiler
- **AST (Abstract Syntax Tree)**: Current hierarchical representation where each expression type is a tree node, intuitive but requires recursive evaluation
- **ExifTool expressions**: Perl-style conditional logic used throughout ExifTool for context-dependent behavior (`$$self{Model} =~ /EOS R5/`, `$$valPt =~ /^0204/`)

### Surprising Context

**CRITICAL**: Document non-intuitive aspects that aren't obvious from casual code inspection:

- **Mixed paradigms create cognitive overhead**: Engineers must learn RPN for math expressions but AST for predicates, making maintenance and extension confusing
- **Performance is NOT the issue**: Both systems are fast enough for typical usage (expressions evaluated once per tag/file), architectural consistency is the real benefit
- **ExifTool uses eval()**: ExifTool itself has no custom parser - it relies on Perl's built-in eval, so our approach doesn't need to match ExifTool's internal architecture, just its behavior
- **Debugging complexity trades off**: AST provides clear tree structure for debugging, RPN requires stack trace understanding, but RPN enables better tooling consistency
- **34 consuming files means high migration risk**: Any breaking change affects processor registry, all generated tag kits, conditional logic, and binary data parsing

### Foundation Documents

- **Design docs**: [CODEGEN.md](CODEGEN.md) for expression compilation patterns
- **ExifTool source**: `lib/Image/ExifTool.pm` for expression evaluation patterns, SubDirectory definitions in various `.pm` modules
- **Start here**: 
  - Study `codegen/src/expression_compiler.rs` architecture and patterns
  - Examine `src/expressions/tests/mod.rs` for comprehensive test coverage (590 lines)
  - Review usage in `src/processor_registry/mod.rs` and generated tag kits
  - Note: New predicate compiler will live in `codegen/src/` for architectural consistency

### Prerequisites

- **Knowledge assumed**: Understanding of Rust ownership, pattern matching, and basic compiler concepts (tokenization, parsing, evaluation)
- **Setup required**: Working exif-oxide build environment, ability to run `cargo t expressions` and `make precommit`

**Context Quality Check**: Can a new engineer understand WHY this approach is needed after reading this section?

## Work Completed

- ✅ **Architecture analysis** → identified RPN benefits (consistency, extensibility) outweigh AST advantages (readability)
- ✅ **Usage research** → documented 34 consuming files and key expression patterns
- ✅ **Design decision** → chose incremental migration over big-bang rewrite to minimize risk

## Remaining Tasks

**REQUIRED**: Each task must be numbered, actionable, and include success criteria.

### 1. Task: Design RPN Predicate System

**Success Criteria**: Complete architectural design document with concrete types, evaluation model, and migration strategy that passes architecture review
**Approach**: Model after `expression_compiler.rs` but adapted for predicate evaluation
**Dependencies**: None

**Success Patterns**:
- ✅ Types mirror math compiler structure with predicate-specific tokens
- ✅ Clear precedence rules documented for all operators
- ✅ Migration strategy preserves existing behavior exactly
- ✅ Design includes debugging/inspection capabilities from start

```rust
// Core token types for predicate expressions
#[derive(Debug, Clone, PartialEq)]
pub enum PredicateToken {
    // Context variables ($manufacturer, $model, $tagID)
    ContextVar(String),
    
    // Special variables ($$valPt, $val{N})  
    DataVar,
    IndexedVar(u32),
    
    // Literals (strings, numbers, hex values)
    String(String),
    Integer(i64),
    Float(f64), 
    Hex(u32),
    
    // Regex patterns (/EOS/, /^0204/)
    RegexPattern(String),
    
    // Operators with precedence
    Op(PredicateOp),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PredicateOp {
    // Comparison (precedence 4)
    Eq, Ne, Gt, Lt, Gte, Lte,
    
    // Pattern matching (precedence 4)  
    RegexMatch, RegexNotMatch,
    
    // Logical NOT (precedence 3)
    Not,
    
    // Logical AND (precedence 2)
    And,
    
    // Logical OR (precedence 1)
    Or,
    
    // Functions (precedence 5)
    Exists,
}

#[derive(Debug, Clone)]
pub struct CompiledPredicate {
    original_expr: String,
    rpn_tokens: Vec<PredicateToken>,
}
```

### 2. Task: Implement Predicate Compiler

**Success Criteria**: `cargo t predicate_compiler` passes, handles all existing expression patterns from test suite
**Approach**: Create `codegen/src/predicate_compiler.rs` adapting Shunting Yard algorithm from math compiler
**Dependencies**: Task 1 (design must be complete)

**Success Patterns**:
- ✅ All 34 expression patterns from current test suite compile successfully
- ✅ Precedence matches standard logical operator rules: `!` > `and` > `or`
- ✅ Complex regex patterns parse correctly: `/^(inf|undef)$/`, `/^0204/`
- ✅ Generated RPN tokens match expected sequence for test cases

### 3. Task: Implement Stack-Based Evaluation

**Success Criteria**: RPN evaluation produces identical results to current AST system for all test cases, `cargo t predicate_evaluation` passes
**Approach**: Implement stack-based evaluation engine in predicate compiler, reusing existing field resolution logic
**Dependencies**: Task 2 (compiler must be working)

**Success Patterns**:
- ✅ All existing expression tests pass with RPN evaluation
- ✅ Stack overflow/underflow handled gracefully with clear error messages
- ✅ TagValue comparison logic exactly matches current AST behavior
- ✅ Regex cache integration maintains performance characteristics

### 4. Task: Migrate ExpressionEvaluator

**Success Criteria**: All 34 consuming files work unchanged, `cargo t expressions` passes, public API identical
**Approach**: Replace internal AST evaluation with RPN compilation/evaluation while keeping public API unchanged
**Dependencies**: Task 3 (evaluation engine must be working)

**Success Patterns**:
- ✅ `evaluate_context_condition()` and `evaluate_data_condition()` signatures unchanged
- ✅ Internal compilation cache implemented for frequently used expressions
- ✅ Memory management handles compiled expression cleanup properly
- ✅ Performance benchmarks show no regression from current system

### 5. Task: Update All Consumers

**Success Criteria**: All 34 consuming files compile and run unchanged, integration tests pass
**Approach**: Test each usage pattern individually, update any code depending on AST internals
**Dependencies**: Task 4 (ExpressionEvaluator must be migrated)

**Success Patterns**:
- ✅ Processor registry, generated tag kits, conditions.rs all work unchanged
- ✅ No breaking changes to Expression enum or public types
- ✅ All imports of `crate::expressions::` continue to work
- ✅ Integration tests demonstrate end-to-end functionality preserved

### 6. RESEARCH: Performance Validation

**Objective**: Verify RPN evaluation matches or exceeds current AST performance for real-world usage
**Success Criteria**: Benchmarks show <5% regression for common expression patterns, document results
**Done When**: Performance report completed showing before/after measurements

## Prerequisites

- **Current expression system stable** → All tests passing
- **Expression compiler mature** → `codegen/src/expression_compiler.rs` well-tested
- **Comprehensive test coverage** → 590 lines of tests must all pass with new implementation

## Testing

**Unit**: 
- Test PredicateToken parsing for all supported patterns
- Test RPN compilation with various precedence combinations
- Test stack evaluation for each operator type
- Test error handling for malformed expressions

**Integration**:
- Verify all existing expression tests pass unchanged
- Test with real processor contexts from tag kit usage
- Validate binary data pattern matching still works
- Check regex caching integration

**Manual check**: 
- Run `cargo t expressions` and confirm all 590 lines of tests pass
- Run real-world tag extraction to verify behavior unchanged
- Performance benchmarks show no regression

## Definition of Done

- [ ] `cargo t expressions` passes with new RPN implementation
- [ ] All 34 consuming files work unchanged
- [ ] Performance benchmarks show no regression
- [ ] `make precommit` clean
- [ ] Documentation updated with new architecture
- [ ] Migration guide for future expression work

## Additional Gotchas & Tribal Knowledge

**Format**: Surprise → Why → Solution (Focus on positive guidance)

- **AST debugging tools won't work with RPN** → RPN uses stack execution model → Build debugging tools that reconstruct logical tree structure from RPN tokens
- **Regex caching tied to AST evaluation** → RPN compilation separates from evaluation → Pass regex cache to evaluation phase, not compilation phase  
- **Precedence bugs are subtle** → Shunting Yard must implement exact precedence rules → Extensively test precedence combinations, especially `!` vs `and` vs `or`
- **Memory management changes completely** → RPN tokens allocated once, reused → Implement proper cleanup for compiled expressions, avoid memory leaks
- **Error reporting becomes less intuitive** → RPN limited to token positions → Maintain mapping from tokens back to original expression source positions
- **ExifTool doesn't use custom parser** → We still must handle all expression patterns → Validate against ExifTool behavior, not parsing approach
- **34 consuming files means migration risk** → Any API change breaks consumers → Keep public API identical, change only internal implementation

## Quick Debugging

Stuck? Try these:

1. **Expression parsing issues**: Compare with math compiler's tokenization
2. **Precedence problems**: Check Shunting Yard precedence table
3. **Evaluation errors**: Add stack tracing to see execution flow
4. **Performance regression**: Profile with `cargo bench` before/after
5. **Test failures**: Use `cargo t expressions -- --nocapture` to see debug output
6. **Integration issues**: Check for AST-specific dependencies in consumers

**Common patterns**:
- `grep -r "Expression::" src/` - Find AST-specific code that needs updating
- `rg "evaluate_.*condition" src/` - Find evaluation call sites
- `cargo t --package exif-oxide --lib expressions` - Run just expression tests


## Second Opinion

  Strategic Assessment: Skip P20d Migration

  After seeing your todo backlog, this is not worth doing for these reasons:

  You Have Bigger Fish to Fry

  Looking at your priorities:
  - P10a: Only 59% EXIF success rate - this blocks PhotoStructure deployment
  - P16: Binary data extraction partially complete - users can't extract thumbnails
  - P20: Arithmetic expressions - actual maintenance pain with 15+ manual functions

  P20d is architectural nice-to-have while P10a/P16/P20 are user-blocking functionality gaps.

  Architecture vs. Functionality Trade-off

  You said it perfectly: "forward progress has been retarded by insufficient architectural planning" BUT "we've suffered from
  over-engineered solutions too."

  This is the classic trade-off:
  - Over-architecture risk: P20d migration (590 tests, 34 consumers at risk, debatable benefit)
  - Under-architecture pain: You already solved this by creating the math expression compiler

  Current Expression System Isn't Blocking Progress

  The expression system works fine. It's not creating maintenance burden or slowing feature development. The real maintenance
  burdens are:
  - P20: "15+ individual arithmetic functions" - this IS causing whack-a-mole
  - P10a: Manual tag implementation preventing core functionality
  - P16: Half-finished binary extraction

  Recommendation: Abandon P20d, Focus on P10a/P16/P20

  1. Mark P20d as "Won't Fix - Opportunity Cost Too High"
  2. P10a: Get to 95% EXIF success - this unblocks your primary user
  3. P16: Finish binary data extraction - basic user expectation
  4. P20: Replace arithmetic whack-a-mole with expression compiler

  These provide immediate user value vs architectural consistency that may never pay back the investment.

  Bottom line: Your instinct about "crapton of additional work" is right. Save architectural improvements for when they solve
  actual pain points, not theoretical consistency issues.
