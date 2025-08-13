# PPI Token Analysis for ExifTool Expressions

Generated: Wed Aug 13 14:46:27 2025

## Executive Summary

- **Modules analyzed**: 49 ExifTool modules from `config/exiftool_modules.json`
- **Total expressions**: 5,056 unique Condition, ValueConv, and PrintConv expressions
- **Parse failures**: 0 (PPI successfully parsed all expressions)
- **Unique PPI token types**: 35 different token classes found
- **Currently unsupported types**: 27 token types not handled in `rust_generator.rs`
- **Current coverage**: ~20% of expressions fully convertible to Rust
- **Target coverage**: 90%+ after implementing priority tokens

## Methodology

This analysis was conducted to determine which PPI tokens we should prioritize for "biggest bang for the buck" in expanding our Perl → Rust code generation:

1. **Expression Extraction**: Extracted all `Condition`, `ValueConv`, and `PrintConv` expressions from modules listed in `config/exiftool_modules.json`
2. **PPI Parsing**: Used PPI (Perl Parsing Interface) to parse each expression into Abstract Syntax Tree
3. **Token Frequency Analysis**: Counted occurrences of each PPI token class across all expressions
4. **Support Gap Identification**: Compared against tokens currently supported in `rust_generator.rs:visit_node()` (lines 94-105)
5. **Impact Assessment**: Ranked unsupported tokens by frequency and complexity to identify implementation priorities

## Current State: Supported vs Unsupported Tokens

Based on `rust_generator.rs` line 94-105, we currently support only these PPI token types:

### ✅ Currently Supported (8 token types)

- `PPI::Document` - Root document container
- `PPI::Statement` - Basic statement container
- `PPI::Token::Symbol` - Variables ($val, $$self{Field})
- `PPI::Token::Operator` - Operators (+, -, eq, =~, etc.)
- `PPI::Token::Number` - Basic numeric literals
- `PPI::Token::Quote::Double` - Double-quoted strings
- `PPI::Token::Quote::Single` - Single-quoted strings
- `PPI::Token::Word` - Function names and keywords
- `PPI::Structure::List` - Function argument lists

### ❌ Currently Unsupported (27 token types)

| PPI Token Class                       | Occurrences | Impact       | Priority |
| ------------------------------------- | ----------- | ------------ | -------- |
| `PPI::Statement::Expression`          | 4,172       | **Critical** | P1       |
| `PPI::Token::Cast`                    | 2,420       | **Critical** | P1       |
| `PPI::Structure::Subscript`           | 1,730       | **Critical** | P1       |
| `PPI::Token::Regexp::Match`           | 731         | **High**     | P1       |
| `PPI::Structure::Constructor`         | 459         | **High**     | P2       |
| `PPI::Token::Comment`                 | 406         | **High**     | P3       |
| `PPI::Statement::UnmatchedBrace`      | 220         | **High**     | P3       |
| `PPI::Token::Number::Hex`             | 207         | **High**     | P2       |
| `PPI::Statement::Variable`            | 204         | **High**     | P2       |
| `PPI::Token::Magic`                   | 174         | **High**     | P2       |
| `PPI::Token::Regexp::Substitute`      | 168         | **High**     | P2       |
| `PPI::Statement::Break`               | 145         | **High**     | P2       |
| `PPI::Token::Number::Float`           | 126         | **High**     | P2       |
| `PPI::Structure::Block`               | 103         | **High**     | P2       |
| `PPI::Token::Regexp::Transliterate`   | 76          | Medium       | P3       |
| `PPI::Token::Quote::Literal`          | 55          | Medium       | P3       |
| `PPI::Token::Number::Exp`             | 55          | Medium       | P3       |
| `PPI::Statement::Include`             | 15          | Medium       | P3       |
| `PPI::Statement::Null`                | 11          | Low          | P4       |
| `PPI::Statement::Compound`            | 10          | Low          | P4       |
| _(7 additional low-frequency tokens)_ | <10 each    | Low          | P4       |

## Critical Token Analysis

### Phase 1: Critical Tokens (Blocks >50% of expressions)

#### PPI::Statement::Expression (4,172 occurrences)

**Description**: Container for complex expressions, function calls, and conditional logic.

**Examples**:

```perl
# Conditional with array access
$val[1] ? $val[0] / $val[1] : undef

# Complex arithmetic with hex
($val & 0xffc0) >> 6) * 10000 + (($val >> 16) & 0xff)

# Multiple operations
my @a = split ' ', $val; sprintf('%.2f - %.2f m', $a[0] <= $a[1] ? @a : reverse @a)
```

**Implementation**: Process children recursively like `PPI::Statement`, but handle more complex nesting.

#### PPI::Token::Cast (2,420 occurrences)

**Description**: Dereference operators for accessing nested data structures.

**Examples**:

```perl
# Self-reference access (most common)
$$self{Model} =~ /EOS D30\b/
$$self{FileType} eq "CR3"

# Array/hash dereference
@$arrayref
%$hashref
$$ref
```

**Implementation**: Map `$$self{Field}` → `ctx.get("Field").unwrap_or_default()`, handle other derefs appropriately.

#### PPI::Structure::Subscript (1,730 occurrences)

**Description**: Hash and array element access.

**Examples**:

```perl
# Array subscript
$val[1] ? $val[0] / $val[1] : undef
$a[0], $a[1] >> 28, $a[1] & 0xfffffff

# Hash subscript (mainly $$self{Field})
$$self{Model} =~ /Canon/
$$self{OPTIONS}{ExtractEmbedded}
```

**Implementation**: Generate appropriate Rust indexing syntax with bounds checking.

#### PPI::Token::Regexp::Match (731 occurrences)

**Description**: Pattern matching operations.

**Examples**:

```perl
# Model detection (very common pattern)
$$self{Model} =~ /EOS D30\b/
$$self{Model} =~ /EOS-1D/

# Binary data pattern matching
$$valPt =~ /^LIGOGPSINFO\0/
$$valPt =~ /^\x4a\xb0\x3b\x0f\x61\x8d\x40\x75/
```

**Implementation**: Generate regex compilation and matching code with proper error handling.

### Phase 2: High Impact Tokens (100-1000 occurrences)

#### PPI::Token::Number::Hex (207 occurrences)

**Examples**: `0xfffffff`, `0xffc0`, `0xff`
**Implementation**: Direct Rust hex literal support.

#### PPI::Token::Number::Float (126 occurrences)

**Examples**: `25.4`, `655.345`, `0.5`
**Implementation**: Enhanced float literal handling (already partially supported).

#### PPI::Statement::Variable (204 occurrences)

**Examples**:

```perl
my @a = split ' ', $val;
my $d = ($val & 0xffc00) >> 10;
our $global = $val;
```

**Implementation**: Generate appropriate Rust variable bindings.

#### PPI::Token::Regexp::Substitute (168 occurrences)

**Examples**:

```perl
$val =~ s/^8 //; $val
$val =~ s/\xff+$//; $val
$val =~ s/(\d+)(\d{4})/$1-$2/; $val
```

**Implementation**: Generate Rust string replacement operations.

## Impact Analysis

### Coverage Improvement Projections

| Phase   | Tokens Added | Est. Coverage | Expressions Unlocked |
| ------- | ------------ | ------------- | -------------------- |
| Current | 9 types      | ~20%          | ~1,000               |
| Phase 1 | +4 types     | ~60%          | ~3,000               |
| Phase 2 | +6 types     | ~75%          | ~3,800               |
| Phase 3 | +8 types     | ~85%          | ~4,300               |
| Phase 4 | +9 types     | ~90%          | ~4,550               |

### Expression Pattern Frequency

**Most Common Patterns Requiring New Support**:

1. **Self-reference conditions**: `$$self{Model} =~ /Canon/` (Cast + Subscript + Regexp)
2. **Array operations**: `$val[0] / $val[1]` (Subscript + Expression)
3. **Hex arithmetic**: `$val & 0xff` (Number::Hex + Expression)
4. **String substitution**: `$val =~ s/foo/bar/; $val` (Regexp::Substitute + Expression)
5. **Variable declarations**: `my @a = split ' ', $val` (Statement::Variable + Expression)

## Implementation Recommendations

### Priority 1: Foundation (Unblocks 60% of expressions)

Focus on these four token types first as they form the foundation for most ExifTool expressions:

1. **PPI::Statement::Expression** - Essential for complex expressions
2. **PPI::Token::Cast** - Required for `$$self{Field}` pattern
3. **PPI::Structure::Subscript** - Needed for array/hash access
4. **PPI::Token::Regexp::Match** - Critical for model detection

### Priority 2: Numeric & String Operations

5. **PPI::Token::Number::Hex** - Common in bit manipulation
6. **PPI::Token::Number::Float** - Already partially supported
7. **PPI::Statement::Variable** - Enables complex multi-step expressions
8. **PPI::Token::Regexp::Substitute** - String manipulation patterns
9. **PPI::Token::Magic** - Special variables like `$_`
10. **PPI::Statement::Break** - Control flow (return, last, next)

### Priority 3: Advanced Features

Continue with remaining medium-frequency tokens to achieve 85%+ coverage.

## Files to Modify

### Primary Implementation

1. **`codegen/src/ppi/rust_generator.rs`** - Add visit methods in `visit_node()` around line 94
2. **`codegen/src/ppi/types.rs`** - Add helper methods for new token analysis

### Supporting Changes

3. **`codegen/src/ppi/fn_registry.rs`** - Ensure deduplication works with new patterns
4. **`ast/src/lib.rs`** - Add runtime support functions if needed (e.g., regex compilation)

### Testing

5. **`tests/ppi_*_test.rs`** - Add comprehensive tests for each new token type
6. **Integration tests** - Validate with real ExifTool expressions

## Validation Strategy

### Test Coverage Requirements

- **Unit tests**: Each new token type needs dedicated test cases
- **Integration tests**: Real ExifTool expressions from analysis data
- **Regression tests**: Ensure existing functionality continues working
- **Performance tests**: PPI parsing adds overhead - measure impact

### Success Metrics

- **Expression conversion rate**: Track percentage of expressions successfully converted
- **Generated code quality**: Ensure Rust output is idiomatic and correct
- **ExifTool compatibility**: Generated functions must produce identical results

## Appendix: Full Token Frequency Data

<details>
<summary>Complete token frequency breakdown (35 types total)</summary>

| Token Class                    | Count     | Supported | Notes                            |
| ------------------------------ | --------- | --------- | -------------------------------- |
| PPI::Token::Whitespace         | 12,847    | ❌        | Skip (not semantically relevant) |
| PPI::Token::Structure          | 10,236    | ❌        | Structural only                  |
| PPI::Statement::Expression     | 4,172     | ❌        | **Critical**                     |
| PPI::Token::Cast               | 2,420     | ❌        | **Critical**                     |
| PPI::Structure::Subscript      | 1,730     | ❌        | **Critical**                     |
| PPI::Token::Operator           | 1,653     | ✅        | Already supported                |
| PPI::Token::Symbol             | 1,456     | ✅        | Already supported                |
| PPI::Statement                 | 1,089     | ✅        | Already supported                |
| PPI::Token::Word               | 982       | ✅        | Already supported                |
| PPI::Token::Regexp::Match      | 731       | ❌        | **Critical**                     |
| PPI::Structure::Constructor    | 459       | ❌        | High priority                    |
| PPI::Token::Comment            | 406       | ❌        | Low priority                     |
| PPI::Token::Number             | 387       | ✅        | Already supported                |
| PPI::Structure::List           | 246       | ✅        | Already supported                |
| PPI::Statement::UnmatchedBrace | 220       | ❌        | Medium priority                  |
| PPI::Token::Number::Hex        | 207       | ❌        | **High priority**                |
| PPI::Statement::Variable       | 204       | ❌        | **High priority**                |
| PPI::Token::Magic              | 174       | ❌        | High priority                    |
| PPI::Token::Regexp::Substitute | 168       | ❌        | **High priority**                |
| PPI::Statement::Break          | 145       | ❌        | High priority                    |
| PPI::Token::Number::Float      | 126       | ❌        | **High priority**                |
| PPI::Structure::Block          | 103       | ❌        | High priority                    |
| _(Remaining 13 types)_         | <100 each | ❌        | Lower priority                   |

</details>

---

_This analysis provides the foundation for implementing comprehensive PPI support in exif-oxide, enabling automatic translation of thousands of ExifTool expressions to efficient Rust code._
