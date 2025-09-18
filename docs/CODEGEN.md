# ExifTool Integration: Code Generation & Implementation

## 🚨 READ FIRST 🚨

You MUST read these documents before making ANY EDITS.

- [TRUST-EXIFTOOL.md](./TRUST-EXIFTOOL.md) - the fundamental law of this project
- [ANTI-PATTERNS.md](./ANTI-PATTERNS.md) - critical mistakes that cause PR rejections
- [SIMPLE-DESIGN.md](./SIMPLE-DESIGN.md) - follow this, and your PR will get merged. Stray, and your work will be discarded.
- [ARCHITECTURE.md](./ARCHITECTURE.md) - follow this, and your PR will get merged. Stray, and your work will be discarded.

**⚠️ MANUAL PORTING BANNED**: We've had 100+ bugs from manual transcription of ExifTool data. All ExifTool data MUST be extracted automatically.

**WE'VE HAD 5+ EMERGENCY RECOVERIES** from engineers who ignored these warnings and committed "architectural vandalism" that broke the entire PPI system. **Your PR WILL BE REJECTED** if you violate these patterns.

**SPECIFIC VIOLATIONS THAT CAUSE IMMEDIATE REJECTION**:
- `split_whitespace()` on AST nodes → **BANNED** - destroys type safety 
- Deleting pattern recognition code → **BANNED** - breaks real camera files
- `args[N].starts_with()` on stringified AST → **BANNED** - violates visitor pattern
- Manual transcription of ExifTool data → **BANNED** - causes silent bugs
- Inconsistent binary operations handling → **BANNED** - breaks expression architecture

## 5 Critical Principles

Before touching any code, understand these principles that prevent architectural vandalism:

1. **Trust ExifTool** - Translate exactly, never manually port or "improve" ExifTool logic
2. **Use Strategy Pattern** - Current system auto-discovers symbols, no config files needed
3. **Use AST Patterns** - Work with structured data, never stringify and re-parse
4. **Use Generated Tables** - Import from `src/generated/`, never manually transcribe
5. **Generated Code is Read-Only** - Fix generators in `codegen/src/`, never edit `src/generated/**/*.rs`

## Current Architecture

The current perl codegen is driven by the [field_extraction.pl](../codegen/scripts/field_extractor.pl) -- for every field, the strategies in `codegen/src/strategies/*.rs` are requested to handle whatever the payload is. Many fields do not have registered strategies.

Many strategies receive simple structures like scalar arrays. 

For the tag_kit, composite (and soon, makernote conditions), the fields contain both scalar values as well as perl expressions, _and that perl expression parsed into an AST by PPI_:

mrm@speedy:~/src/exif-oxide$ codegen/scripts/ppi_ast.pl '$val + 4'

```json
{ "class" : "PPI::Document", "children" : [
    { "class" : "PPI::Statement", "children" : [
        { "class" : "PPI::Token::Symbol", "content" : "$val", "symbol_type" : "scalar" },
        { "class" : "PPI::Token::Operator", "content" : "+" },
        { "class" : "PPI::Token::Number", "content" : "4", "numeric_value" : 4 } ] } ] }
```

The Perl AST is normalized to make it easier for our rust generator by codegen/src/ppi/normalizer/multi_pass.rs

This is driven by process_symbols in codegen/src/strategies/mod.rs, and then the fn_registry DRYs up duplicate perl expressions, and they're written to src/generated/functions/hash_XX.rs based on the hash of the perl expression.

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

## Missing Conversion Tracking (`--show-missing`)

When the PPI generator encounters Perl expressions it cannot translate (e.g., `foreach` loops, complex function calls), it generates placeholder functions that:

1. **Return the value unchanged** - Ensures graceful degradation
2. **Track the missing implementation** - Records expression details for `--show-missing`
3. **Preserve the original Perl** - Escapes and embeds the expression for reference

### How It Works

```rust
// Generated placeholder for untranslatable PrintConv expression
pub fn ast_print_HASH(val: &TagValue, ctx: Option<&ExifContext>) -> TagValue {
    tracing::warn!("Missing implementation for expression in {}", file!());
    codegen_runtime::missing::missing_print_conv(
        0,              // tag_id (filled at runtime)
        "UnknownTag",   // tag_name (filled at runtime)
        "UnknownGroup", // group (filled at runtime)
        "Image::ExifTool::GPS::ToDMS($self, $val, 1, \"E\")", // original Perl
        val
    )
}
```

### Usage

```bash
# Show missing implementations for development
./target/debug/exif-oxide --show-missing image.jpg

# Output includes:
# "missing_implementations": [
#   "PrintConv: Image::ExifTool::GPS::ToDMS(...) [used by GPS:GPSLongitude]",
#   "ValueConv: my @a=split(' ',$val); $_/=500 foreach @a; join(' ',@a) [used by Pentax:SensorSize]"
# ]
```

### Testing

See `codegen-runtime/tests/missing_tracking.rs` for unit tests and `codegen/tests/missing_conversions_test.rs` for integration tests.

## Consistency Best Practices

**TL;DR**: Use the same approach for the same problem. Follow these patterns to avoid TIMTOWTDI (There Is More Than One Way To Do It).

### TagValue Construction

**✅ DO**:
```rust
TagValue::String(String::new())              // Empty strings
TagValue::String(s.into())                   // Convert to string
TagValue::Empty                              // Default fallback
```

**❌ DON'T**:
```rust
TagValue::String("".to_string())             // Unnecessary allocation
TagValue::String(String::from(""))           // Verbose
TagValue::U32(1)                             // Inconsistent defaults
```

### Function Call Generation

**✅ DO** - Use runtime helpers consistently:
```rust
codegen_runtime::math::abs(val)              // Mathematical functions
codegen_runtime::sprintf_perl(fmt, args)     // String formatting
codegen_runtime::string::length_string(val)  // String operations
```

**❌ DON'T** - Mix patterns arbitrarily:
```rust
log(val)           // Bare function (old pattern)
val.abs()          // Method call (inconsistent)
format!("{}", val) // Native Rust (breaks compatibility)
```

### String/Code Generation

**✅ DO** - Use formatdoc! for templates:
```rust
let code = formatdoc! {r#"
    pub fn {name}(val: &TagValue) -> TagValue {{
        // Generated function body
        {body}
    }}
"#, name = func_name, body = body_code};
```

**❌ DON'T** - Chain format! + push_str:
```rust
code.push_str(&format!("pub fn {}(", name));
code.push_str(&format!("val: &TagValue"));
code.push_str(") -> TagValue {\n");
```

### String Escaping

**✅ DO** - Use utility function:
```rust
use crate::common::utils::escape_for_rust_string;
format!("\"{}\"", escape_for_rust_string(value))
```

**❌ DON'T** - Manual replace chains:
```rust
value.replace('\\', "\\\\").replace('"', "\\\"")  // Incomplete, error-prone
```

### Error Handling

**✅ DO** - Consistent error types:
```rust
Result<String, CodeGenError>                 // Code generation errors
anyhow::Result<T>                           // General operations
```

**❌ DON'T** - Mix error patterns:
```rust
Result<T, Box<dyn std::error::Error>>       // Verbose
unwrap()                                    // In non-test code
```

### Architecture Guidelines

- **Single Visitor**: Use one visitor pattern, not multiple implementations
- **No Circular Traits**: Avoid traits that delegate back to the same struct
- **Focused Traits**: 1-2 traits max, each with clear responsibility
- **Consistent Imports**: Either fully qualified or imported, not mixed

See [P09-no-timtowdi.md](todo/P09-no-timtowdi.md) for detailed refactoring plan and alternatives analysis.

## Related Documentation

### Essential Reading
- [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md) - Core principle for all integration work
- [ANTI-PATTERNS.md](ANTI-PATTERNS.md) - Critical mistakes that cause PR rejections
- [GETTING-STARTED.md](GETTING-STARTED.md) - Practical implementation guide

### Detailed References
- [STRATEGY-DEVELOPMENT.md](STRATEGY-DEVELOPMENT.md) - Adding new symbol pattern support
- [ARCHITECTURE.md](ARCHITECTURE.md) - High-level system overview
- [API-DESIGN.md](design/API-DESIGN.md) - Public API structure and TagValue design