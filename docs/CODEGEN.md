# ExifTool Integration: Code Generation & Implementation

**🚨 CRITICAL: All integration follows [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md) - we translate ExifTool exactly, never innovate.**

**⚠️ MANUAL PORTING BANNED**: We've had 100+ bugs from manual transcription of ExifTool data. All ExifTool data MUST be extracted automatically.

**🚨 READ FIRST**: [ANTI-PATTERNS.md](ANTI-PATTERNS.md) - Critical mistakes that cause PR rejections

## 🚨 STOP: READ THIS BEFORE ANY PPI/EXPRESSION WORK 🚨

**WE'VE HAD 5+ EMERGENCY RECOVERIES** from engineers who ignored these warnings and committed "architectural vandalism" that broke the entire PPI system. **Your PR WILL BE REJECTED** if you violate these patterns.

**SPECIFIC VIOLATIONS THAT CAUSE IMMEDIATE REJECTION**:
- `split_whitespace()` on AST nodes → **BANNED** - destroys type safety 
- Deleting pattern recognition code → **BANNED** - breaks real camera files
- `args[N].starts_with()` on stringified AST → **BANNED** - violates visitor pattern
- Manual transcription of ExifTool data → **BANNED** - causes silent bugs
- Inconsistent binary operations handling → **BANNED** - breaks expression architecture

**READ**: [P07-emergency-ppi-recovery.md](todo/P07-emergency-ppi-recovery.md) shows what happens when these rules are ignored.

## 5 Critical Principles

Before touching any code, understand these principles that prevent architectural vandalism:

1. **Trust ExifTool** - Translate exactly, never manually port or "improve" ExifTool logic
2. **Use Strategy Pattern** - Current system auto-discovers symbols, no config files needed
3. **Use AST Patterns** - Work with structured data, never stringify and re-parse
4. **Use Generated Tables** - Import from `src/generated/`, never manually transcribe
5. **Generated Code is Read-Only** - Fix generators in `codegen/src/`, never edit `src/generated/**/*.rs`

## Current Architecture (P02 Completed)

**🎯 UNIFIED PRECEDENCE CLIMBING**: All binary operations use precedence climbing normalization following proven LLVM/Pratt parsing techniques.

**Benefits over previous approach:**
- **Perl-correct precedence**: Uses actual Perl operator precedence table 
- **Separation of concerns**: Normalization handles parsing, visitor handles code generation
- **Architectural consistency**: All binary operations become `BinaryOperation` AST nodes
- **Massive consolidation**: 8 normalizers → 3 normalizers (75% reduction)

**Current Pipeline**:
```
1. ExpressionPrecedenceNormalizer  (handles all binary ops, strings, ternary, functions)
2. ConditionalStatementsNormalizer (statement restructuring)  
3. SneakyConditionalAssignmentNormalizer (multi-statement control flow)
```

**How it works**:
```rust
// 1. PPI parses: $val + 4
//    Raw tokens: [$val, +, 4] (flat sequence)

// 2. ExpressionPrecedenceNormalizer applies precedence climbing
//    Result: BinaryOperation("+", $val, 4) (structured AST)

// 3. Visitor generates clean Rust code
//    Output: (val + 4i32) (correct syntax)
```

## Build Commands

### Primary Commands
```bash
make codegen                    # Full pipeline - processes all modules automatically
cd codegen && cargo run --release  # Direct execution with debug output
make clean                      # Clean generated files
```

### Development & Debugging
```bash
cd codegen && RUST_LOG=debug cargo run     # Verbose strategy selection logs
cd codegen && cargo run -- --module Canon  # Process single module
make check-perl                             # Validate Perl scripts
```

### Testing
```bash
cargo test                      # Full test suite
make compat-test               # ExifTool compatibility tests
make precommit                 # Full validation before commit
```

## PPI Expression Debugging

**Essential Tools** for debugging complex expression generation:

**1. Complete Pipeline Debugging**
```bash
# Shows: Original Expression → Raw PPI AST → Normalized AST → Generated Rust
cargo run --package codegen --bin debug-ppi -- --verbose 'expression_here'

# Example:
cargo run --package codegen --bin debug-ppi -- --verbose '$val + 4'
cargo run --package codegen --bin debug-ppi -- --verbose 'sprintf("%s:%s", unpack "H4H2", $val)'
```

**2. Expression Test Framework** (for rapid iteration)
```bash
# Generate tests from JSON configs and run
make generate-expression-tests
cargo test -p codegen --test generated_expressions

# Test specific expression file
make test-expression-file FILE=tests/config/value_conv/basic_addition.json

# Debug single expression
make debug-expression EXPR='$$val * 25'
```

**3. Raw AST Structure Analysis**
```bash
# Parse Perl expression and show raw PPI AST structure
cd codegen && ./scripts/ppi_ast.pl 'sprintf("%.2f", $val / 100)'
```

## Extension Points

### ✅ CORRECT: Add New Expression Patterns

When adding support for new operators or patterns:

1. **For binary operations** (`+`, `-`, `*`, `=~`, etc.): Add to precedence table in `ExpressionPrecedenceNormalizer::get_precedence()`
2. **For string operations** (regex, concatenation): Add to `StringOperationsHandler::handle_regex_operation()` 
3. **For function patterns** (`sprintf`, `unpack`): Add to `ComplexPatternHandler::try_*_pattern()`

### ❌ BANNED: Bypassing Precedence Climbing Architecture

**NEVER handle binary operations outside the normalization pipeline**:

```rust
// ❌ BANNED - Bypasses precedence climbing normalization
if child.class == "PPI::Token::Operator" && child.content == "*" {
    // Manual operator handling in visitor - creates precedence bugs
}

// ❌ BANNED - String parsing binary operations
if parts.join(" ").contains(" * ") {
    // Destroys AST structure and precedence information  
}
```

**THE RULE**: ALL binary operations MUST be processed by `ExpressionPrecedenceNormalizer` during AST normalization.

## Generated Code Structure

The system organizes generated code into focused modules:

```
src/generated/
├── Canon_pm/                  # Canon-specific tables
│   ├── white_balance.rs       # White balance lookup
│   └── lens_types.rs          # Lens type lookup
├── functions/                 # PPI-generated functions
│   ├── hash_c7.rs             # Arithmetic: $val + 4 → (val + 4i32)
│   └── hash_ef.rs             # Complex: ($val + 100) / 2
└── ExifTool_pm/              # Core ExifTool tables
    └── magic_numbers.rs       # File detection
```

**Generated vs Manual Code**:
- **Generated**: Lookup tables, tag definitions, arithmetic functions from PPI expressions
- **Manual**: Complex PrintConv/ValueConv logic, manufacturer-specific processors

## Daily Development Workflow

### Adding PrintConv/ValueConv Functions

**Step 1: Check for Generated Table**
```bash
find src/generated -name "*orientation*" -o -name "*white_balance*"
```

**Step 2: Implement Using Generated Table**
```rust
/// EXIF Orientation PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:281-290 (%orientation hash)
pub fn orientation_print_conv(val: &TagValue) -> TagValue {
    use crate::generated::exif::lookup_orientation;
    
    if let Some(orientation_val) = val.as_u8() {
        if let Some(description) = lookup_orientation(orientation_val) {
            return TagValue::string(description);
        }
    }
    TagValue::string(format!("Unknown ({val})"))
}
```

**Step 3: Test Against ExifTool**
```bash
cargo run --bin compare-with-exiftool test.jpg EXIF:
```

## 🚨 Pre-Commit Validation: Avoid PR Rejection 🚨

**RUN THESE COMMANDS BEFORE SUBMITTING ANY PR**:

```bash
# Check for banned AST string parsing patterns (MUST return empty)
rg "split_whitespace|\.join.*split|args\[.*\]\.starts_with" codegen/src/ppi/
# If this finds matches: YOUR PR WILL BE REJECTED

# Check for bypassed binary operations (MUST return empty)
rg "children\[.*\]\.class.*Operator.*=~" codegen/src/ppi/rust_generator/
# Binary operations must go through ExpressionPrecedenceNormalizer

# Check for disabled infrastructure (MUST return empty) 
rg "DISABLED|TODO.*normalize|//.*normalize" codegen/src/ppi/rust_generator/
# If this finds disabled code: ENABLE PROPERLY, don't disable working systems

# Verify precedence climbing is working
cargo run --package codegen --bin debug-ppi -- --verbose '$val * 100'
# Should show: BinaryOperation("*", $val, 100) in normalized AST
```

**If any of these checks fail, fix the issues BEFORE submitting your PR.**

## Troubleshooting

### Expression Generation Issues

**Problem**: Binary operation not generating expected Rust code
**Solution**: Check debug-ppi output - ensure operation creates `BinaryOperation` node in normalized AST

**Problem**: Arithmetic expressions return original value unchanged (e.g., `$val + 4` returns `val`)
**Solution**: Bug in precedence climbing - verify `ExpressionPrecedenceNormalizer` is creating proper binary operations

**Problem**: Complex expressions like `($val + 100) / 2` only partially work
**Solution**: Precedence climbing bug - check operator precedence tables and token consumption logic

### Build Failures
```bash
# Check Perl syntax
make check-perl

# Debug strategy selection
cd codegen && RUST_LOG=trace cargo run

# Clean and rebuild
make clean && make codegen
```

### Expression Test Development

**Create new test configs** in `codegen/tests/config/` following the JSON schema:
```json
{
    "expression": "$val + 4",
    "type": "ValueConv", 
    "description": "Simple addition by 4",
    "exiftool_reference": "common arithmetic pattern",
    "test_cases": [
        {"input": {"U32": 10}, "expected": {"U32": 14}},
        {"input": {"U32": 0}, "expected": {"U32": 4}}
    ]
}
```

**Run tests**:
```bash
make generate-expression-tests    # Generate from JSON configs
cargo test -p codegen --test generated_expressions  # Run all expression tests
```

## Performance Characteristics

- **Zero Runtime Cost**: LazyLock static tables with HashMap lookups
- **Type Safety**: Compile-time validation of all keys and values
- **O(1) Function Dispatch**: HashMap-based registry lookup
- **Graceful Degradation**: Never panics on missing implementations

## Related Documentation

### Essential Reading
- [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md) - Core principle for all integration work
- [ANTI-PATTERNS.md](ANTI-PATTERNS.md) - Critical mistakes that cause PR rejections
- [GETTING-STARTED.md](GETTING-STARTED.md) - Practical implementation guide

### Detailed References
- [STRATEGY-DEVELOPMENT.md](STRATEGY-DEVELOPMENT.md) - Adding new symbol pattern support
- [ARCHITECTURE.md](ARCHITECTURE.md) - High-level system overview
- [API-DESIGN.md](design/API-DESIGN.md) - Public API structure and TagValue design