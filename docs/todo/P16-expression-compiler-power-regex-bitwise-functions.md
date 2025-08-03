# P16: Expression Compiler Enhancement - Power, Regex, Bitwise & Function Operations

## Project Overview

- **Goal**: Extend the expression compiler to support power operations (`**`, `exp()`), regex operations (`s///`, `tr///`), bitwise operations (`&`, `|`, `>>`, `<<`), and complex multi-argument ExifTool functions, enabling proper evaluation of critical ValueConv/PrintConv expressions for supported tags
- **Problem**: Current expression compiler cannot execute essential ExifTool expressions that are fundamental to photography metadata processing, causing fallback to string representations and preventing proper value conversions for APEX calculations, GPS coordinates, string cleanup, and manufacturer-specific processing
- **Constraints**: Must maintain ExifTool compatibility exactly, support all operator precedence rules, handle Perl-style semantics correctly

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

## Context & Foundation

### System Overview

- **Expression Compiler**: Located in `codegen/src/expression_compiler/`, converts ExifTool Perl expressions to Rust code using AST-based parsing and code generation
- **Value Conversion Pipeline**: Raw bytes → Format → ValueConv (via expression compiler) → PrintConv (via expression compiler) → Display value
- **Tag Kit System**: Extracts ValueConv/PrintConv expressions from ExifTool source and stores them as strings in generated Rust code, expecting expression compiler to evaluate them at runtime
- **Supported Tags**: 275 tags in `config/supported_tags.json` that represent the core metadata needed for PhotoStructure's photo management

### Key Concepts & Domain Knowledge

- **APEX System**: Additive System of Photographic Exposure - uses logarithmic values for exposure calculations, requires power operations like `2**(-val)` to convert to linear measurements
- **ValueConv vs PrintConv**: ValueConv preserves data types for programmatic use, PrintConv formats for human display, both use same expression evaluation system
- **ExifTool Functions**: Specialized utilities like `Image::ExifTool::GPS::ToDMS()` that handle complex conversions, often with multiple arguments and context-sensitive behavior
- **Perl Expression Context**: Expressions have access to variables `$val`, `$self`, `$tag`, `@val`, plus extensive Perl standard library

### Surprising Context

- **Power operations are NOT optional**: APEX exposure calculations are fundamental to EXIF - without `2**(-val)` support, aperture, exposure time, and ISO values cannot be computed correctly
- **Regex operations handle camera quirks**: Each manufacturer has unique string formatting bugs that require specific regex cleanup patterns discovered over 25 years of ExifTool development
- **GPS functions are multi-argument**: Simple `ToDMS($val)` won't work - actual signature is `ToDMS($self, $val, $doPrintConv, $ref)` with complex parameter interactions
- **Generated code already contains these expressions**: Tag kit extraction has already captured thousands of complex expressions as strings - they just need evaluation infrastructure
- **Bitwise operations are manufacturer-specific**: Mostly used for Canon/Sony/Olympus internal formats, but essential for those tags to work
- **Expression evaluation happens at codegen time**: Unlike runtime evaluation, this compiles Perl expressions to static Rust code

### Foundation Documents

- **Design docs**: [CODEGEN.md](../CODEGEN.md) - Expression compiler architecture and AST design
- **ExifTool source**: 
  - `third-party/exiftool/lib/Image/ExifTool/Canon.pm:2492` - Canon EV power operations
  - `third-party/exiftool/lib/Image/ExifTool/Sony.pm:3229` - Sony ISO calculations using `**`
  - `third-party/exiftool/lib/Image/ExifTool/GPS.pm:17-21` - GPS coordinate function signatures
  - `third-party/exiftool/lib/Image/ExifTool/Exif.pm:2312` - APEX exposure time conversion
- **Start here**: `codegen/src/expression_compiler/parser.rs` - Current AST implementation, `codegen/src/expression_compiler/codegen.rs` - Code generation logic

### Prerequisites

- **Knowledge assumed**: Rust AST construction, Perl expression semantics, photography metadata basics (APEX, GPS coordinates)
- **Setup required**: Rust toolchain, ExifTool submodule (`git submodule update --init`), test images in `test-images/`

## Work Completed

- ✅ **Comprehensive ExifTool analysis** → identified 200+ power operations, 50+ regex operations, 30+ complex functions in supported tags
- ✅ **AST-based expression compiler** → switched from RPN to AST architecture supporting ternary, function calls, and string operations
- ✅ **Tag kit extraction** → all required expressions already captured as strings in generated code, ready for compilation
- ✅ **Power operations implementation** → added `**` operator with right-associative precedence, uses Rust `.powf()` for APEX calculations
- ✅ **Unary minus support** → context-aware tokenizer distinguishes `-` as unary vs binary, enables expressions like `2**(-$val)`
- ✅ **Registry pattern obsolescence** → power expressions now auto-compile instead of requiring manual conv_registry entries
- ✅ **Legacy RPN code removal** → simplified expression parser to always use AST, removed ~400 lines of unused RPN fallback code

## Status Summary

**Progress**: 3 of 5 tasks complete (60%) - Foundation excellent, but integration and research remain
- ✅ **Task 1**: Power operations and unary minus - COMPLETE with 13+ tests passing
- ✅ **Task 2**: Regex operations - COMPLETE with 5+ tests passing (s///, tr///, flags support)  
- ✅ **Task 3**: Bitwise operations - COMPLETE with 6+ tests passing (&, |, <<, >>, hex parsing)
- ❌ **Task 4**: Multi-argument function research - INCOMPLETE (no research document exists)
- ❌ **Task 5**: Tag kit integration - INCOMPLETE (expression compiler not wired into codegen pipeline)

**Critical Gap**: Expression compiler works excellently but is not integrated into production codegen - expressions still stored as strings instead of compiled code

**Next Priority**: Task 5 (Tag Kit Integration) should precede Task 4 - get current functionality integrated before expanding scope

## Remaining Tasks

### ✅ 1. Task: Implement Power Operations (`**`, `exp()`, `pow()`, `log()`) - COMPLETED

**Status**: All success criteria met. Power operations and unary minus implemented with comprehensive test coverage.

**Results**: 
- ✅ All Canon EV calculations work: APEX power expressions compile correctly with `2**(-$val)` pattern
- ✅ Sony ISO calculations work: `100 * 2**(16 - $val/256)` generates correct Rust `.powf()` calls
- ✅ APEX conversions work: `2**(-$val)` for exposure time calculations with proper unary minus support
- ✅ Test coverage: 13 power/unary tests passing (8 power + 5 unary minus scenarios)

**Implementation**: 
- ✅ Added `Power` to `OpType` enum, tokenizer parses `**` with precedence 4 (right-associative)
- ✅ Added `UnaryMinus` AST node with context-aware tokenizer (distinguishes `-` usage by preceding token)
- ✅ Rust code generation uses `f64::powf()`, generates `(-operand)` for unary minus
- ✅ Right-associative parsing: `2**3**2` correctly evaluates as `2**(3**2)` = 512

**Registry Obsolescence**: Power expressions like `2**(-$val/3)` now auto-compile, eliminating need for manual conv_registry entries

### ✅ 2. Task: Implement Regex Operations (`s///`, `tr///`) - COMPLETED

**Status**: All success criteria met. Regex substitution and transliteration implemented with comprehensive test coverage.

**Results**: 
- ✅ Olympus firmware parsing: `s/(.{3})$/\.$1/` compiles and generates Rust regex code
- ✅ String cleanup: `s/\0+$//` pattern works for null terminator removal
- ✅ Canon firmware patterns: Case-insensitive substitution `s/Alpha/a/i` supported
- ✅ Test coverage: 5 comprehensive tests passing (regex + transliteration scenarios)

**Implementation**: 
- ✅ Added `RegexSubstitution` and `Transliteration` AST nodes with pattern/replacement/flags
- ✅ Extended tokenizer to parse `s/pattern/replacement/flags` and `tr/search/replace/flags`
- ✅ Code generation using Rust `regex` crate with case-insensitive and global flags
- ✅ Transliteration with delete (`d`) and complement (`c`) flag support
- ✅ Handles character class expansion and filtering operations

### ✅ 3. Task: Implement Bitwise Operations (`&`, `|`, `>>`, `<<`) - COMPLETED

**Status**: All success criteria met. All bitwise operations implemented with proper precedence and hex number support.

**Results**: 
- ✅ Version extraction: `$val >> 16` and `$val & 0xffff` patterns compile correctly with integer conversion
- ✅ Canon file naming: Shift and AND operations work with proper precedence
- ✅ Multi-flag extraction: `(($val >> 13) & 0x7)` pattern compiles and extracts 3-bit fields
- ✅ Test coverage: 6 comprehensive tests passing (all operators + precedence + hex numbers)

**Implementation**: 
- ✅ Added `BitwiseAnd`, `BitwiseOr`, `LeftShift`, `RightShift` to `OpType` enum
- ✅ Extended tokenizer to parse `&`, `|`, `<<`, `>>` with proper precedence (shifts higher than AND/OR)
- ✅ Hex number parsing: `0xffff` now tokenizes correctly as 65535.0
- ✅ Code generation converts f64 → i64 → bitwise operation → f64 for proper integer behavior
- ✅ Updated `is_compilable` to allow bitwise operators (removed blocking checks)

### 4. RESEARCH: Complex Multi-argument ExifTool Functions - INCOMPLETE

**Current Status**: No comprehensive research document exists. Critical gap in understanding multi-argument function requirements.

**Objective**: Create technical specification for GPS coordinate functions and other multi-argument ExifTool utilities used in supported tags

**Success Criteria**: 
- [ ] **Research document**: Create `docs/research/multi-argument-exiftool-functions.md` analyzing function signatures
- [ ] **Function categorization**: Document which functions can be statically compiled vs need runtime context  
- [ ] **Implementation strategy**: Concrete plan for `Image::ExifTool::GPS::ToDMS($self, $val, $doPrintConv, $ref)`
- [ ] **Usage analysis**: Top 10 functions by frequency with complexity assessment
- [ ] **ExifTool validation**: Verify function behavior with 20+ examples

**Implementation Details**: 
- Research GPS functions: `ToDMS()`, `ToDegrees()` - critical for 12+ GPS tags
- Analyze Canon functions: `CanonEv()` - used throughout Canon tag processing  
- Document string utilities: `Decode()`, `ConvertDateTime()` - widespread usage
- Study binary functions: `pack()`, `unpack()` - manufacturer-specific processing

**Integration Strategy**: Results inform Task 5 integration complexity

**Dependencies**: None - can proceed independently

### 5. Task: Integration with Tag Kit Code Generation - INCOMPLETE  

**Current Status**: Expression compiler implemented but NOT integrated into production codegen pipeline. Generated modules still store expressions as strings.

**Success Criteria**:
- [ ] **Codegen integration**: Tag kit generator invokes expression compiler on ValueConv/PrintConv strings → `codegen/src/generators/tag_kit/mod.rs` modified
- [ ] **Code replacement**: Generated Rust code contains actual computations instead of string expressions → `src/generated/*/tag_kit/*.rs` files updated
- [ ] **Compilation success**: `make codegen && cargo t` passes without expression-related failures
- [ ] **Manual validation**: `cargo run -- test-images/canon/Canon_T3i.jpg` shows computed values instead of string expressions
- [ ] **ExifTool comparison**: 50+ tag examples match ExifTool output exactly
- [ ] **Error handling**: Unsupported expressions gracefully fall back to string representation with clear diagnostics

**Implementation Details**: 
- Modify `codegen/src/generators/tag_kit/mod.rs` to invoke expression compiler during generation
- Update generated code templates to include necessary imports and error handling
- Replace string storage with generated Rust function calls for compilable expressions
- Add comprehensive test coverage for expression integration

**Integration Strategy**: Start with power/bitwise/regex expressions (Tasks 1-3 complete), expand to multi-argument functions after Task 4

**Dependencies**: Tasks 1-3 complete (sufficient for basic integration), Task 4 recommended for full functionality

**Success Patterns**:
- ✅ Generated Rust code is readable and efficient, includes comments with original ExifTool expressions
- ✅ Build errors include helpful diagnostics with ExifTool source references  
- ✅ Runtime performance matches or exceeds string-based fallbacks
- ✅ Memory usage improves due to elimination of string storage for compiled expressions

## Implementation Guidance

### Recommended Patterns

- **AST Node Design**: Follow existing pattern in `types.rs` with enum variants for each operation type, implement `Display` for debugging
- **Error Handling**: Use `anyhow::Result<T>` consistently, include ExifTool source location in error messages
- **Code Generation**: Generate readable Rust code with comments indicating original ExifTool expression
- **Testing Strategy**: Create comprehensive test suite with ExifTool comparison - each test should verify identical output

### Tools to Leverage

- **Existing AST infrastructure**: `Expression` enum and `generate_expression()` function
- **Regex crate**: Use `regex::Regex` for pattern matching, ensure Perl compatibility
- **Test framework**: Use integration tests with actual camera files from `test-images/`
- **ExifTool comparison**: Use `scripts/compare-with-exiftool.sh` for validation

### Architecture Considerations

- **Compilation vs Runtime**: All expression evaluation happens at codegen time, produces static Rust code
- **Context Variables**: Support for `$val`, `$self`, `$tag` requires careful Rust code generation
- **Error Propagation**: Unsupported expressions should gracefully fall back to string representation
- **Performance**: Generated code should be zero-cost abstractions where possible

### ExifTool Translation Notes

- **Perl Context**: `$val` becomes function parameter, `$self` becomes context object reference
- **Return Types**: Match ExifTool behavior exactly - some functions return strings, others return numbers
- **Edge Cases**: Perl's handling of undefined values, division by zero, string-to-number coercion
- **Operator Precedence**: Must match Perl exactly, especially for complex expressions with mixed operators

## Integration Requirements

- [ ] **Activation**: Expression compiler is used by default for all tag kit generation
- [ ] **Consumption**: Generated tag kit code contains actual computations, not string expressions
- [ ] **Measurement**: Can verify expressions are compiled by checking generated Rust code and runtime behavior
- [ ] **Cleanup**: String-based expression storage is replaced with compiled functions

## Testing

### Critical Test Cases

**Power Operations** (from ExifTool analysis):
- Canon EV: `exp(Image::ExifTool::Canon::CanonEv($val)*log(2)/2)` with values [20, 32, 95, 96]
- Sony ISO: `100 * 2**(16 - $val/256)` with values [3072, 3328, 3840, 4096] 
- APEX exposure: `2**(-$val)` with values [5, 8, 13] (1/32, 1/256, 1/8192 seconds)

**Regex Operations** (from ExifTool analysis):
- Olympus firmware: `$val=sprintf("%x",$val);$val=~s/(.{3})$/\.$1/;$val` with input 0x12345
- String cleanup: `s/\0+$//` with null-terminated strings
- Canon parsing: Multi-step substitution chain from Canon.pm:1626-1628

**Bitwise Operations** (from ExifTool analysis):
- Version parsing: `sprintf("%d.%.4d",$val >> 16, $val & 0xffff)` with value 0x00020003
- Flag extraction: `(($val >> 13) & 0x7)` with various bit patterns

### Integration Testing

- **Unit**: Test each operation type in isolation with edge cases
- **Integration**: Run complete tag extraction pipeline on real camera files
- **Manual check**: `cargo run --bin exif-oxide test-images/canon/Canon_T3i.CR2` shows correct aperture/exposure calculations

## Definition of Done

- [ ] `cargo t expression_compiler` passes all power, regex, and bitwise tests
- [ ] `make codegen` generates compilable Rust code for all tag kits
- [ ] `make precommit` clean
- [ ] Manual ExifTool comparison shows <1% difference in supported tag values
- [ ] Generated code performance is equivalent or better than string-based fallbacks

## Additional Research Areas (Future TPPs)

Based on analysis, these areas need separate investigation:

- **P17: Advanced ExifTool Function Library** - Multi-argument functions like GPS coordinate processing
- **P18: Binary Data Expression Support** - `pack()`, `unpack()`, byte manipulation functions  
- **P19: Composite Tag Expression Context** - Access to multiple source values via `@val`, `@prt`, `@raw`
- **P20: Perl Standard Library Compatibility** - `sprintf()`, string functions, date/time utilities

## Quick Debugging

When expressions fail to compile:

1. `grep -r "failing_expression" src/generated/` - Find where it's used
2. `rg "failing_expression" third-party/exiftool/` - Find ExifTool source
3. `RUST_LOG=debug cargo run --bin generate_rust` - See compilation details
4. Check `codegen/src/expression_compiler/tests.rs` for similar patterns

**Common Issues**:
- **Perl operator precedence** - `2**3*4` is `(2**3)*4` = 32, not `2**(3*4)` = 4096
- **String vs numeric context** - Perl auto-converts, Rust requires explicit conversion
- **Variable scoping** - `$val` is parameter, not global variable in generated Rust

## Examples from ExifTool Source

**Power Operation Examples** (verified working in ExifTool):
```perl
# Canon.pm:2492 - MaxAperture calculation
ValueConv => 'exp(Image::ExifTool::Canon::CanonEv($val)*log(2)/2)'

# Sony.pm:3229 - ISO calculation  
ValueConv => '100 * 2**(16 - $val/256)'

# Exif.pm:2312 - APEX exposure time
ValueConv => 'IsFloat($val) && abs($val)<100 ? 2**(-$val) : 0'
```

**Regex Operation Examples**:
```perl
# Olympus.pm:811 - Firmware version formatting
PrintConv => '$val=sprintf("%x",$val);$val=~s/(.{3})$/\.$1/;$val'

# Canon.pm:1626-1628 - Complex firmware parsing
PrintConvInv => '$_=$val; s/Alpha ?/a/i; s/Beta ?/b/i; s/Unknown ?\((.)\)/$1/i; s/ ?rev ?(.)\./0$1/; s/ ?rev ?//; tr/a-fA-F0-9//dc; return hex $_'
```

**Bitwise Operation Examples**:
```perl
# FlashPix.pm:504 - Version extraction
ValueConv => 'sprintf("%d.%.4d",$val >> 16, $val & 0xffff)'

# Olympus.pm:3448 - Multi-field extraction
ValueConv => '(($val >> 13) & 0x7) . " " . (($val >> 12) & 0x1) . " " . (($val >> 11) & 0x1)'
```

These examples represent real-world usage patterns that must work identically in our expression compiler.

## Future Work & Refactoring Ideas

### High-Priority Integration (Next Engineer Should Focus Here)

- **Tag Kit Integration**: Task 5 is critical - expression compiler is excellent but not connected to production codegen
- **Integration Tests**: Create comprehensive end-to-end tests proving expression compiler works in real metadata extraction
- **Performance Validation**: Measure compiled expression performance vs string-based fallbacks
- **Error Handling**: Improve diagnostic messages for unsupported expressions during codegen

### Architecture Improvements

- **Modular Function Library**: Extract multi-argument functions into separate crate for reuse across modules
- **Expression Caching**: Cache compiled expressions during codegen to avoid repeated compilation
- **Type System Enhancement**: Add proper type checking for expression variables ($val, $self, etc.)
- **Precedence Validation**: Add comprehensive precedence tests comparing with Perl behavior

### Future TPP Candidates

- **P17: Multi-argument Function System** - GPS coordinate processing, Canon EV functions, string utilities
- **P18: Binary Data Expression Support** - pack(), unpack(), byte manipulation for raw format support
- **P19: Composite Tag Expression Enhancement** - Access to multiple source values, cross-tag calculations
- **P20: Perl Standard Library Compatibility** - sprintf(), date/time functions, advanced string operations

### Code Quality Improvements

- **Test Coverage Expansion**: Add property-based tests for operator precedence and edge cases
- **Documentation Enhancement**: Add inline documentation with ExifTool source references
- **Refactor Large Files**: Split expression compiler into focused modules (parser, codegen, validation)
- **Benchmark Suite**: Create performance benchmarks comparing expression approaches

### Related Work

- **P16a**: GPS ValueConv regression fix - demonstrated importance of proper registry vs compilation prioritization
- **P14b**: GPS location processing - will benefit from multi-argument function research (Task 4)
- **P12**: Composite tag calculations - needs expression compiler integration for complex calculations

**Status**: Expression compiler foundation is excellent and comprehensive. The critical missing piece is integration into production codegen (Task 5), which would immediately provide value for the ~500 power/regex/bitwise expressions already extracted from ExifTool.