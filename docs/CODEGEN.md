# Code Generation System

## READ FIRST

Before making ANY edits to codegen or src/core:

- [TRUST-EXIFTOOL.md](./TRUST-EXIFTOOL.md) - fundamental project law
- [ANTI-PATTERNS.md](./ANTI-PATTERNS.md) - mistakes causing PR rejections
- [SIMPLE-DESIGN.md](./SIMPLE-DESIGN.md) - design principles
- [ARCHITECTURE.md](./ARCHITECTURE.md) - system overview

**INSTANT REJECTION TRIGGERS**:

- `split_whitespace()` on AST nodes - destroys type safety
- Editing `src/generated/**/*.rs` - fix `codegen/src/` instead
- Manual ExifTool data transcription - 100+ bugs from this mistake
- Bypassing `ExpressionPrecedenceNormalizer` for operators

## Architecture

The code generation system has two components:

| Component           | Purpose                                         | When Used         |
| ------------------- | ----------------------------------------------- | ----------------- |
| **codegen**         | CLI tool that generates Rust from ExifTool Perl | Build time only   |
| **src/core**        | Runtime module that generated code imports      | Compile + runtime |

```
ExifTool Perl → [codegen] → src/generated/*.rs → [uses crate::core] → exif-oxide binary
```

Note: `src/core` was previously a separate crate (`codegen_runtime`) but is now integrated into the main crate as a module to simplify publishing.

## Data Flow Overview

```
third-party/exiftool/lib/Image/ExifTool/*.pm
                    ↓
        exiftool-patcher.sh (converts `my` → `our` for introspection)
                    ↓
        field_extractor.pl (Perl symbol table introspection + PPI parsing)
                    ↓
        JSON Lines (one symbol per line, includes inline AST)
                    ↓
        Rust: field_extractor.rs (spawns Perl, captures JSON)
                    ↓
        Vec<FieldSymbol>
                    ↓
        StrategyDispatcher (pattern matching → strategy selection)
                    ↓
        Strategy.extract() → PPI normalization → Rust code generation
                    ↓
        src/generated/**/*.rs (1,300+ files across 50 modules)
```

## codegen Module Structure

```
codegen/
├── src/
│   ├── main.rs                    # Entry point: extraction → strategies → output
│   ├── field_extractor.rs         # Spawns Perl, parses JSON Lines output
│   ├── strategies/                # Pattern-based symbol handlers
│   │   ├── mod.rs                 # StrategyDispatcher + ExtractionStrategy trait
│   │   ├── tag_kit.rs             # Tag tables (PrintConv/ValueConv)
│   │   ├── composite_tag.rs       # Composite tag definitions
│   │   ├── simple_table.rs        # Key-value lookup tables
│   │   ├── scalar_array.rs        # Primitive arrays
│   │   ├── binary_data.rs         # ProcessBinaryData tables
│   │   ├── file_type_lookup.rs    # File type detection
│   │   ├── magic_numbers.rs       # Binary magic patterns
│   │   ├── mime_type.rs           # MIME type mappings
│   │   └── boolean_set.rs         # Membership sets
│   ├── ppi/                       # Perl AST → Rust code pipeline
│   │   ├── types.rs               # PpiNode structure
│   │   ├── parser.rs              # JSON → PpiNode
│   │   ├── normalizer/            # Multi-pass AST transformation
│   │   │   └── passes/            # Individual rewrite passes
│   │   ├── rust_generator/        # PpiNode → Rust code
│   │   └── fn_registry/           # Function deduplication by hash
│   └── common/utils.rs            # String escaping, formatting
├── scripts/
│   ├── field_extractor.pl         # Core extraction via Perl introspection
│   ├── ppi_ast.pl                 # Standalone expression → AST tool
│   ├── exiftool-patcher.pl        # Converts `my` → `our` declarations
│   └── PPI/Simple.pm              # Custom JSON serializer for PPI
└── config/
    └── exiftool_modules.json      # Modules to process (~50 total)
```

## Strategy System (Duck-Typing Pattern Matching)

Each symbol from `field_extractor.pl` is routed to the first matching strategy:

| Priority | Strategy       | Detects                | Example Symbol                  |
| -------- | -------------- | ---------------------- | ------------------------------- |
| 1        | CompositeTag   | `is_composite_table=1` | `%Composite`                    |
| 2        | FileTypeLookup | File type patterns     | `%fileTypeLookup`               |
| 3        | MagicNumber    | Magic byte patterns    | `%magicNumber`                  |
| 4        | MimeType       | MIME mappings          | `%mimeType`                     |
| 5        | SimpleTable    | String key-value pairs | `%canonWhiteBalance`            |
| 6        | ScalarArray    | Primitive arrays       | `@fileTypes`                    |
| 7        | TagKit         | Tag definitions        | `%Image::ExifTool::Canon::Main` |
| 8        | BinaryData     | Binary processing      | `%processBinaryData`            |
| 9        | BooleanSet     | Membership tests       | `%isDat`                        |

**No configuration files needed** - strategies use pattern recognition via `can_handle()`.

## PPI Pipeline (Perl Expression → Rust Code)

For expressions like `PrintConv => '$val + 4'`, the pipeline:

1. **Perl → JSON AST** (via `ppi_ast.pl`):

```json
{
  "class": "PPI::Statement",
  "children": [
    { "class": "PPI::Token::Symbol", "content": "$val" },
    { "class": "PPI::Token::Operator", "content": "+" },
    { "class": "PPI::Token::Number", "content": "4", "numeric_value": 4 }
  ]
}
```

2. **Normalization** (multi-pass AST rewriting):

   - `ExpressionPrecedenceNormalizer` - operator precedence (ALL binary ops go here)
   - Ternary, conditional, join/unpack pattern handlers

3. **Code Generation** (visitor pattern → Rust):

```rust
pub fn ast_value_10ca38d8(val: &TagValue, ctx: Option<&ExifContext>) -> Result<TagValue, ExifError> {
    Ok((val + 4i32))
}
```

4. **Deduplication** (hash-based registry):
   - Functions grouped by 2-char hash prefix: `functions/hash_XX.rs`
   - Identical expressions share one function

## Generated Code Structure

```
src/generated/
├── mod.rs                         # Re-exports all modules
├── functions/                     # PPI-generated functions (194 files)
│   ├── hash_10.rs ... hash_ff.rs  # Grouped by AST hash prefix
│   └── mod.rs
├── Canon_pm/                      # Manufacturer module (100+ files)
│   ├── mod.rs
│   ├── main_tags.rs               # Tag definitions
│   ├── lens_types.rs              # Lookup table
│   └── ...
├── Nikon_pm/, Sony_pm/, ...       # 48 manufacturer modules
└── ExifTool_pm/                   # Cross-cutting utilities
    ├── mime_type.rs
    ├── magic_numbers.rs
    └── file_type_lookup.rs
```

### Generated Code Examples

**Tag Definition** (from TagKitStrategy):

```rust
pub static CANON_MAIN_TAGS: LazyLock<HashMap<u16, TagInfo>> = LazyLock::new(|| {
    HashMap::from([
        (8, TagInfo {
            name: "FileNumber",
            format: "unknown",
            print_conv: Some(PrintConv::Function(ast_print_c033b0a1)),
            value_conv: None,
        }),
    ])
});
```

**Lookup Table** (from SimpleTableStrategy):

```rust
static OFF_ON_DATA: &[(u8, &str)] = &[(0, "Off"), (1, "On")];
pub static OFF_ON: LazyLock<HashMap<u8, &str>> = LazyLock::new(|| OFF_ON_DATA.iter().copied().collect());
pub fn lookup_off_on(key: u8) -> Option<&'static str> { OFF_ON.get(&key).copied() }
```

**Arithmetic Function** (from PPI):

```rust
/// Original perl: $val / 100
pub fn ast_value_fd4074b6(val: &TagValue, ctx: Option<&ExifContext>) -> Result<TagValue, ExifError> {
    Ok(val / 100i32)
}
```

**Complex Function** (from PPI):

```rust
/// Original perl: 100 * 2**(16 - $val/256)
pub fn ast_value_161c6918(val: &TagValue, ctx: Option<&ExifContext>) -> Result<TagValue, ExifError> {
    Ok(100i32 * power(2i32.into(), (16i32 - (val / 256i32)).into()))
}
```

## Core Module Structure

The runtime module (`src/core/`) provides all helpers that generated code imports:

```
src/core/
├── mod.rs                    # Re-exports for `use crate::core::*;`
├── types.rs                  # ExifContext, ExifError
├── missing.rs                # Missing implementation tracking
├── tag_value/
│   ├── mod.rs                # TagValue enum (U8, U16, String, Rational, etc.)
│   ├── ops.rs                # Arithmetic operators (+, -, *, /, >>, &)
│   ├── conversion.rs         # as_u8(), as_f64(), is_truthy()
│   └── display.rs            # Display formatting
├── math/
│   ├── basic.rs              # exp, log, sin, cos, sqrt, abs, int
│   ├── safe.rs               # safe_division (returns 0 for /0)
│   └── camera.rs             # Photographic: aperture, ISO, shutter
├── string/
│   ├── extraction.rs         # substr_2arg, substr_3arg, index_*
│   ├── transform.rs          # chr, uc, regex_replace
│   └── format.rs             # concat, repeat_string
├── data/
│   └── unpack.rs             # Binary unpacking (Perl pack formats)
└── fmt/
    └── sprintf.rs            # Perl-compatible sprintf
```

### Key Runtime Functions

| Category   | Function                            | Perl Equivalent                  |
| ---------- | ----------------------------------- | -------------------------------- |
| **Math**   | `safe_division(a, b)`               | Safe `$a / $b`                   |
| **Math**   | `power(base, exp)`                  | `$base ** $exp`                  |
| **Math**   | `int(val)`                          | `int($val)`                      |
| **String** | `substr_3arg(s, off, len)`          | `substr($s, $off, $len)`         |
| **String** | `uc(val)`                           | `uc($val)`                       |
| **String** | `chr(code)`                         | `chr($code)`                     |
| **Data**   | `unpack_binary(spec, val)`          | `unpack($spec, $val)`            |
| **Data**   | `join_unpack_binary(sep, fmt, val)` | `join($sep, unpack($fmt, $val))` |
| **Format** | `sprintf_perl(fmt, args)`           | `sprintf($fmt, @args)`           |

### TagValue Arithmetic

Generated code uses TagValue operators directly:

```rust
// Generated code uses these patterns:
Ok((val + 5i32))           // Addition
Ok(val / 100i32)           // Division
Ok(100i32 * power(...))    // Complex expressions
```

The `ops.rs` module implements `Add`, `Sub`, `Mul`, `Div` for TagValue with smart type coercion.

## Build Commands

```bash
make codegen                    # Full pipeline (patches ExifTool, runs codegen, formats)
make clean && make codegen      # Clean rebuild
make precommit                  # Full validation before commit
cargo t                         # Run tests (includes test-helpers feature)
```

### Debugging

```bash
# Debug single expression through full pipeline
cargo run -p codegen --bin debug-ppi -- --verbose '$val + 4'

# Debug strategy selection
cd /home/mrm/src/exif-oxide/codegen && RUST_LOG=debug cargo run

# Raw PPI AST (Perl only)
codegen/scripts/ppi_ast.pl 'sprintf("%.2f", $val)'
```

### Expression Tests

```bash
make generate-expression-tests                    # Generate test files
cargo test -p codegen --test generated_expressions  # Run expression tests
make debug-expression EXPR='$val * 25'            # Debug single expression
```

## Missing Implementation Tracking

When expressions can't be translated, placeholder functions are generated:

```rust
pub fn ast_print_c033b0a1(val: &TagValue, ctx: Option<&ExifContext>) -> TagValue {
    tracing::warn!("Missing implementation for expression in {}", file!());
    crate::core::missing::missing_print_conv(
        0, "UnknownTag", "UnknownGroup",
        "$_=$val,s/(\\d+)(\\d{4})/$1-$2/,$_",  // Original Perl
        val,
    )
}
```

Usage:

```bash
./target/debug/exif-oxide --show-missing image.jpg
```

## Extension Points

### Adding New Expression Patterns

1. **Binary operators**: Add to `ExpressionPrecedenceNormalizer::get_precedence()`
2. **String operations**: Add to `StringOperationsHandler`
3. **Function patterns**: Add to `ComplexPatternHandler::try_*_pattern()`

### Adding New Runtime Functions

1. Add function to appropriate `src/core/` module
2. Re-export from `mod.rs` for `use crate::core::*;` access
3. Update generator to emit calls to new function

### Adding New Strategies

Implement `ExtractionStrategy` trait:

```rust
pub trait ExtractionStrategy: Send + Sync {
    fn name(&self) -> &'static str;
    fn can_handle(&self, symbol: &FieldSymbol) -> bool;  // Duck-typing
    fn extract(&mut self, symbol: &FieldSymbol, ctx: &mut ExtractionContext) -> Result<()>;
    fn finish_module(&mut self, module_name: &str) -> Result<()>;
    fn finish_extraction(&mut self, ctx: &mut ExtractionContext) -> Result<Vec<GeneratedFile>>;
}
```

Register in `all_strategies()` with appropriate priority.

## Troubleshooting

### Expression Not Generating Correct Code

```bash
cargo run -p codegen --bin debug-ppi -- --verbose 'YOUR_EXPRESSION'
```

Check that normalized AST shows `BinaryOperation` nodes for operators.

### Strategy Not Claiming Symbol

```bash
cd /home/mrm/src/exif-oxide/codegen && RUST_LOG=trace cargo run
```

Look for "Strategy X can_handle returned false for symbol Y" messages.

### Build Failures

```bash
make check-perl              # Validate Perl scripts
make clean && make codegen   # Clean rebuild
```

## Code Style Consistency

**TagValue Construction**:

```rust
TagValue::String(String::new())   // Empty string
TagValue::String(s.into())        // Convert to string
TagValue::Empty                   // Default fallback
```

**Runtime Function Calls**:

```rust
crate::core::sprintf_perl(fmt, args)    // Correct
crate::core::math::abs(val)             // Correct
format!("{}", val)                      // WRONG - breaks Perl compat
```

## Performance

- **LazyLock**: All static tables initialized once on first access
- **HashMap O(1)**: Function and table lookups
- **Hash-based deduplication**: Identical expressions generate one function
- **Zero runtime parsing**: All Perl→Rust translation at build time

## Related Documentation

- [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md) - Core principle
- [ANTI-PATTERNS.md](ANTI-PATTERNS.md) - What not to do
- [ARCHITECTURE.md](ARCHITECTURE.md) - System overview
- [GETTING-STARTED.md](GETTING-STARTED.md) - Quick start
