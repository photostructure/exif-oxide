# P20: Arithmetic ValueConv Expression System

Replace individual arithmetic ValueConv functions with codegen-time expression compilation system.

## Project Overview

- **Goal**: Replace 15+ individual arithmetic functions (`divide_8_value_conv`, `subtract_5_value_conv`, etc.) with a codegen-time expression compiler that generates inline arithmetic code
- **Problem**: Current whack-a-mole approach creates maintenance burden - each new arithmetic expression like `$val / 16` requires a new manual function
- **Constraints**: Zero runtime dependencies, must trust ExifTool expressions exactly, generated code must be readable and performant

---

## ⚠️ CRITICAL REMINDERS

- **MANDATORY: read THESE TWO DOCUMENTS**: [CLAUDE.md](../CLAUDE.md) | [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md)
- **Concurrent edits**: If build errors aren't near your code → STOP, tell user
- **Ask questions**: Confused about approach? Debugging >1hr? ASK before continuing
- **Keep this document updated with progress!**: Use 🟢Done/🟡WIP/🔴Blocked status as you work.
- **Add discoveries and research**: Add context that will be helpful to future engineers completing this task, or for future relevant tasks.
- **Don't oversell your progress**: Do not use hyperbolic "DRAMATIC IMPROVEMENT!"/"GROUNDBREAKING PROGRESS" styled updates.

Key sections to always apply from CLAUDE.md:
- "Assume Concurrent Edits" - Critical safety rule
- "Trust ExifTool" - Core principle #1  
- "Only perl can parse perl" - Codegen constraints
- "Look for easy codegen wins" - Maintenance strategy

---

## Context & Foundation

- **Why**: Manual arithmetic functions don't scale. We have `divide_8_value_conv`, `divide_256_value_conv`, `subtract_5_value_conv`, `add_3_value_conv`, etc. Each new ExifTool expression requires a new function.
- **Docs**: Current registry system in `codegen/src/conv_registry.rs` lines 144-147, existing functions in `src/implementations/value_conv.rs`
- **Start here**: Examine existing arithmetic functions and registry patterns

**Current State Analysis**:
```rust
// Registry entries (conv_registry.rs:144-147)
m.insert("$val / 8", ("crate::implementations::value_conv", "divide_8_value_conv"));
m.insert("$val / 256", ("crate::implementations::value_conv", "divide_256_value_conv"));
m.insert("$val - 5", ("crate::implementations::value_conv", "subtract_5_value_conv"));
m.insert("$val + 3", ("crate::implementations::value_conv", "add_3_value_conv"));

// Plus many more: divide_6_value_conv, multiply_100_value_conv, subtract_104_divide_8_value_conv, etc.
```

**Technical Approach**: Use Shunting Yard algorithm to parse arithmetic expressions at codegen time, convert to RPN, then generate inline arithmetic code instead of function calls.

## Work Completed

- ✅ **Research Phase** → identified Shunting Yard algorithm as simple solution (~80 lines) with complete Rust reference implementation
- ✅ **Architecture Decision** → chose codegen-time compilation over runtime parsing to avoid adding dependencies to deployed code  
- ✅ **Expression Classification** → simple arithmetic (`$val op constant`) gets compiled, complex expressions keep custom functions
- ✅ **Function Inventory** → documented all 23 arithmetic functions that can be replaced by expression compilation
- ✅ **Expression Compiler Implementation** → complete Shunting Yard compiler with full test coverage
- ✅ **Registry Classification System** → automatic detection of compilable vs custom function expressions

### 🟢 Current Arithmetic Functions Analysis

**Simple Arithmetic (Can Be Compiled)**:
- `multiply_100_value_conv` → `$val * 100`
- `divide_8_value_conv` → `$val / 8`
- `divide_256_value_conv` → `$val / 256` 
- `divide_6_value_conv` → `$val/6`
- `subtract_5_value_conv` → `$val - 5`
- `add_3_value_conv` → `$val + 3`
- `subtract_104_divide_8_value_conv` → `($val-104)/8`
- `canon_div_32_plus_5_value_conv` → `$val / 32 + 5`
- `canon_div_10_value_conv` → `$val / 10`
- `canon_div_100_value_conv` → `$val / 100`
- `canon_plus_1_value_conv` → `$val + 1`
- `canon_millimeter_value_conv` → `$val * 25.4 / 1000`

**Complex Expressions (Keep Custom Functions)**:
- `power_neg_div_3_value_conv` → `2 ** (-$val/3)` (exponentials)
- `reciprocal_10_value_conv` → `$val ? 10 / $val : 0` (conditionals)
- `sony_exposure_time_value_conv` → `$val ? 2 ** (6 - $val/8) : 0` (conditional + exponential)
- `sony_iso_value_conv` → `$val ? exp(($val/8-6)*log(2))*100 : $val` (conditional + logarithmic)
- `sony_fnumber_value_conv` → `2 ** (($val/8 - 1) / 2)` (exponentials)
- `apex_shutter_speed_value_conv` → `IsFloat($val) && abs($val) < 100 ? 2**(-$val) : 0` (function calls + conditionals)
- `apex_aperture_value_conv` → `2**($val / 2)` (exponentials)
- `canon_auto_iso_value_conv` → `exp($val / 32 * log(2)) * 100` (logarithmic + exponential)
- `canon_base_iso_value_conv` → `exp($val / 32 * log(2)) * 100 / 32` (logarithmic + exponential)
- String processing functions (regex-based)
- Bitwise operations like `canon_directory_number_value_conv`

**Impact**: 12 simple arithmetic functions can be eliminated (~50% reduction), while 11 complex functions remain as custom implementations.

## Remaining Tasks

### 🟢 Task: Document Current Arithmetic Functions

**Success**: Complete inventory of all individual arithmetic functions that can be replaced

**Completed**: ✅ Identified 12 simple arithmetic functions that can be compiled and 11 complex functions that should remain as custom implementations

### 🟢 Task: Implement Expression Compiler

**Success**: Working Shunting Yard implementation that converts `$val / 8` to inline Rust code

**Completed**: ✅ Complete implementation in `codegen/src/expression_compiler.rs` with:
- Full Shunting Yard algorithm (~620 lines including tests)
- Support for `+`, `-`, `*`, `/`, parentheses, `$val` variable, decimal numbers
- RPN compilation and Rust code generation
- **Comprehensive test coverage (15 test cases)**:
  - Operator precedence and associativity
  - Parentheses handling and nesting
  - Real-world expression patterns
  - Code generation (simple vs complex cases)
  - Whitespace handling
  - Edge cases and error conditions
  - Complete `is_compilable()` validation
- Smart code generation (simple expressions → direct arithmetic, complex → stack-based)

### 🟢 Task: Update Registry Classification

**Success**: Registry automatically detects simple arithmetic vs complex expressions

**Completed**: ✅ Added to `codegen/src/conv_registry.rs`:
- `ValueConvType` enum for classification
- `classify_valueconv_expression()` function with automatic detection
- `get_compilable_expressions()` utility for debugging
- Test coverage confirming 12 compilable expressions found in registry

### Task: Generate Inline Arithmetic Code

**Success**: Tag kit generator produces direct arithmetic instead of function calls

Example transformation:
```rust
// Instead of:
0x1234 => divide_8_value_conv(value)

// Generate:
0x1234 => {
    match value.as_f64() {
        Some(val) => Ok(TagValue::F64(val / 8.0)),
        None => Ok(value.clone()),
    }
}
```

### Task: Remove Obsolete Functions

**Success**: All individual arithmetic functions removed, no functionality lost

**Approach**: Delete after confirming generated code works correctly

## Prerequisites

None - this is a self-contained improvement to the codegen system.

## Testing

- **Unit**: Test expression compiler with various arithmetic patterns
- **Integration**: Verify generated tag kit code produces identical results to current functions  
- **Manual check**: Run `cargo run -- test-image.jpg` and confirm arithmetic ValueConv results unchanged

## Definition of Done

- [ ] Shunting Yard expression compiler implemented in codegen
- [ ] Registry classifies expressions correctly (simple vs complex)
- [ ] Tag kit generator produces inline arithmetic code for simple expressions
- [ ] All existing individual arithmetic functions removed from `value_conv.rs`
- [ ] `make precommit` passes
- [ ] Generated code is readable and maintainable
- [ ] Zero new runtime dependencies
- [ ] Documentation updated in `CODEGEN.md`

## Gotchas & Tribal Knowledge

**Format**: Surprise → Why → Solution

- **Generated code looks complex** → Avoid this trap → Keep generated arithmetic simple and readable, prefer `val / 8.0` over complex stack operations
- **Expression parsing fails** → Some ExifTool expressions use Perl idioms → Only compile basic arithmetic, fallback to custom functions for complex cases
- **Performance concerns about inline code** → Generated code is actually faster → Direct arithmetic beats function call overhead
- **Existing tests break** → Generated code must produce identical results → Test against current function outputs before removing them

## Quick Debugging

Stuck? Try these:

1. `grep -r "divide_8_value_conv" src/` - Find all current usages
2. `rg "\\\$val [+\-*/] \d+" codegen/src/conv_registry.rs` - Find arithmetic patterns  
3. `cargo t value_conv -- --nocapture` - Test arithmetic conversions
4. Check generated tag kit files in `src/generated/*/tag_kit/` for compilation results

## Reference Implementation

Complete Shunting Yard algorithm in Rust (~80 lines):

```rust
type Number = f64;

#[derive(Debug, Copy, Clone, PartialEq)]
struct Operator {
    token: char,
    operation: fn(Number, Number) -> Number,
    precedence: u8,
    is_left_associative: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Digit(Number),
    Operator(Operator),
    LeftParen,
    RightParen,
}

// ... [rest of reference implementation] ...
```

This provides the foundation for converting `$val / 8` expressions to RPN at codegen time.