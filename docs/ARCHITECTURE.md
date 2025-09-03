# exif-oxide Architecture

**🚨 CRITICAL: This architecture is built on [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md) - the fundamental law of this project.**

**🚨 READ FIRST**: [ANTI-PATTERNS.md](ANTI-PATTERNS.md) - Critical mistakes that cause PR rejections

## 🚨 EMERGENCY WARNING: ARCHITECTURAL VANDALISM PREVENTION 🚨

**THIS PROJECT HAS HAD MULTIPLE EMERGENCY RECOVERIES** due to engineers who ignored architectural guidelines and broke core systems. Most recently, 546 lines of critical ExifTool pattern recognition were deleted, breaking support for Canon/Nikon/Sony cameras and requiring weeks of emergency recovery work.

**YOUR PR WILL BE IMMEDIATELY REJECTED** if you commit any of these violations:

### ❌ BANNED CODE PATTERNS (Instant Rejection)
```rust
// ❌ AST STRING PARSING - DESTROYS TYPE SAFETY
args[1].split_whitespace()           // Taking AST and re-parsing as string
parts.contains(&"unpack")            // String matching on AST data

// ❌ DELETED EXIFTOOL PATTERNS - BREAKS REAL CAMERA FILES  
// Any deletion of pattern recognition code without understanding purpose

// ❌ DISABLED INFRASTRUCTURE - CREATES TECHNICAL DEBT
// let normalized_ast = normalize()  // DISABLED - commenting out working systems

// ❌ MANUAL EXIFTOOL DATA - SILENT BUGS
match wb_value { 0 => "Auto", 1 => "Daylight" }  // Hand-transcribed lookup tables
```

**RECOVERY EVIDENCE**: See [P07-emergency-ppi-recovery.md](todo/P07-emergency-ppi-recovery.md) for documentation of the massive cleanup required after these patterns were ignored.

**ENFORCEMENT**: These aren't suggestions - **violations will result in immediate PR rejection and you will be asked to restart your work.**

## Overview

exif-oxide is a Rust translation of [ExifTool](https://exiftool.org/), focusing on mainstream metadata extraction from image files. This document explains the **why** behind our architectural decisions to help engineers stay "on the rails" and avoid costly mistakes.

**Scope**: We target mainstream metadata tags only (frequency > 80% in TagMetadata.json), reducing the implementation burden from 15,000+ tags to approximately 500-1000. We explicitly do not support ExifTool's custom tag definitions or user-defined tags.

## Core Philosophy: Why These Decisions Matter

### 1. Trust ExifTool - No Novel Parsing

**The Rule**: We port ExifTool exactly, never reinvent or "improve" its logic.

**Why This Matters**: ExifTool represents 25+ years of handling real-world camera firmware bugs. Every seemingly odd piece of code exists because some camera manufacturer violates the EXIF spec in that specific way. When engineers try to "clean up" or "simplify" ExifTool's logic, they break support for real camera files.

**Example**: Canon EOS cameras store maker note offsets differently depending on firmware version. ExifTool's complex offset calculation handles this. Simplifying it breaks thousands of Canon photos.

**Enforcement**: All manual implementations must cite ExifTool source line numbers. Any PR that "improves" ExifTool logic will be rejected.

### 2. Generated vs Manual Code Split

**The Architecture**: Generated code provides data structures; manual code provides ExifTool-equivalent logic.

**Why This Split**: Perl's flexibility makes automatic translation unreliable for complex expressions, but data extraction can be automated perfectly. This gives us the best of both worlds - zero-error data extraction with carefully ported logic.

**Generated Code** (`src/generated/`):
- Lookup tables (Canon white balance, Nikon lens IDs)
- Tag definitions with metadata  
- File type detection patterns
- Static arrays (encryption tables)

**Manual Code** (`src/implementations/`):
- PrintConv/ValueConv functions with complex logic
- Manufacturer-specific processors
- Binary data parsing algorithms

**Why Not Generate Everything**: We tried. Complex Perl expressions like `$val =~ s/(\d+)\.(\d+)\.(\d+)/$1*10000+$2*100+$3/e` are impossible to parse reliably. Manual porting with ExifTool references is more reliable than buggy automatic translation.

### 3. Strategy Pattern for Code Generation

**The System**: Universal symbol discovery with competing pattern recognition strategies.

**Why Not Configuration Files**: The old config-based system required manual maintenance for every ExifTool module. Engineers would forget to add configs for new modules, leading to missing support. The strategy pattern automatically discovers ALL symbols and routes them to appropriate generators.

**How It Works**:
1. `field_extractor.pl` discovers ALL symbols automatically from any ExifTool module
2. 9 strategies compete using duck-typing to claim symbols (first-match-wins)
3. Winning strategies generate appropriate Rust code
4. New modules work immediately without configuration

**Strategy Priority Examples**:
- `CompositeTagStrategy` (highest) - Dynamic tags calculated from other tags
- `SimpleTableStrategy` - Basic lookup tables like `canonWhiteBalance`
- `TagKitStrategy` - Complex tag definitions with PrintConv
- `BooleanSetStrategy` (lowest) - Membership tests like `isDatChunk`

**Why This Complexity**: ExifTool has dozens of different data patterns. A single generator can't handle them all correctly. Competing strategies ensure each pattern gets appropriate handling.

### 4. Expression Evaluation Hybrid System

**The Challenge**: ExifTool has two fundamentally different types of expressions with incompatible requirements.

**Regular Tags**: Dependencies known at build time (e.g., `PrintConv => { 0 => 'Off', 1 => 'On' }`)
- Can generate optimized static Rust code
- Zero runtime overhead for tag processing
- Example: White balance lookup becomes a static HashMap

**Composite Tags**: Dependencies resolved at runtime (e.g., GPS coordinates combining latitude + hemisphere)
- Must evaluate expressions like `$val[1] =~ /^S/i ? -$val[0] : $val[0]` with actual tag values
- Requires dynamic evaluation with file context
- Only calculated when requested

**Why Hybrid Architecture**: A single system can't optimize both cases. Compile-time generation handles 90%+ of tags with zero overhead. Runtime evaluation handles the complex 10% that require dynamic context.

**Three-Tier Runtime Execution**:
1. **Dynamic Evaluation** - Simple expressions like `$val / 8`
2. **Registry Lookup** - Complex operations like regex substitution
3. **Manual Implementation** - Camera-specific quirks that can't be automated

### 5. PPI→AST→Rust Pipeline Integration Testing

**The Challenge**: Complex Perl expressions require reliable translation to equivalent Rust code, with full pipeline validation and actual execution testing.

**Complete Test Generation Pipeline**: The system processes JSON test configurations through the entire PPI pipeline to generate and execute real Rust functions:

```json
{
    "expression": "$val * 2",
    "type": "ValueConv",
    "description": "Simple value multiplication by 2",
    "exiftool_reference": "lib/Image/ExifTool/Example.pm:123",
    "test_cases": [
        {"input": {"U32": 50}, "expected": {"U32": 100}},
        {"input": {"F64": 1.5}, "expected": {"F64": 3.0}}
    ]
}
```

**Full Pipeline Implementation**:
1. **JSON Configuration** → Test definitions with expressions and expected outputs
2. **PPI AST Generation** → Call `ppi_ast.pl` for each expression  
3. **AST Normalization** → Apply multi-pass normalizer for canonical forms
4. **Function Registration** → Use `PpiFunctionRegistry` for deduplication
5. **Function Generation** → Create actual Rust functions in `tests/generated/functions/`
6. **Test Generation** → Create test files that import and execute functions
7. **Execution & Validation** → Run generated code with real assertions

**Key Architectural Components**:
- **PpiFunctionRegistry**: Deduplicates identical expressions across test files
- **Two-phase processing**: Registration phase followed by generation phase
- **AST-based hashing**: Functions deduplicated by normalized AST structure
- **Debug mode**: Single-file processing without regenerating mod.rs files

**Why This Architecture**: The system provides:
- **Real execution testing**: Generated functions are compiled and run with actual inputs
- **Fast iteration**: <10 seconds for full test cycle with `cargo test`
- **Function deduplication**: Identical expressions share generated functions
- **Clear failure diagnosis**: Tests show exactly which expressions generate invalid Rust
- **Incremental debugging**: Single-file mode for rapid expression development

**Test Organization**:
```
codegen/tests/generated/
├── functions/               # Deduplicated generated functions
│   ├── hash_XX.rs          # Functions grouped by AST hash prefix
│   └── mod.rs              # Module declarations
├── value_conv/             # Tests organized by expression type
├── print_conv/
├── conditions/
├── types.rs                # Type aliases for test environment
└── mod.rs
```

This architecture validates the complete expression processing pipeline while providing rapid feedback during development.

### 6. State Management & Offset Calculations

**The Challenge**: EXIF data has complex nested structures with manufacturer-specific offset schemes.

**Why Stateful Processing**: ExifTool uses stateful processing because metadata extraction is inherently stateful. Tags depend on previous tags, directories reference other directories, and manufacturers use different offset calculation schemes.

**Key State Components**:
- **PROCESSED Hash** - Prevents infinite loops in circular directory references
- **VALUE Hash** - Stores extracted tags with namespace conflict resolution
- **DataMember Dependencies** - Earlier tags determine format of later tags
- **Directory Context** - Maintains PATH stack and offset calculations

**ExifReader Structure**: Our `ExifReader` closely mirrors ExifTool's `$self` object:
```rust
pub struct ExifReader {
    extracted_tags: HashMap<(u16, String), TagValue>,    // VALUE hash
    processed: HashMap<u64, String>,                     // PROCESSED hash
    path: Vec<String>,                                   // PATH stack
    data_members: HashMap<String, DataMemberValue>,      // DataMember storage
    base: u64,                                          // Offset calculations
    // ... more state
}
```

**Why Not Functional**: We tried functional approaches. They break on real camera files because EXIF processing requires tracking complex interdependencies that functional approaches can't handle efficiently.

**Offset Calculation Complexity**: 
- **Standard EXIF**: Offsets relative to TIFF header
- **Entry-Based** (Panasonic): Offsets relative to each IFD entry
- **Maker Note Fixing**: Manufacturer-specific base offset corrections
  - Canon: 4, 6, 16, or 28 byte offsets depending on model
  - Nikon: TIFF header at offset 0x0a from maker note start
  - Sony: Offset 0 or 4 depending on model era

### 7. Streaming-First Design

**The Decision**: All binary data handled via streaming references, not in-memory copies.

**Why Streaming**: EXIF data can contain embedded full-resolution images or videos. Loading everything into memory would create massive memory usage for large files. Streaming allows processing multi-gigabyte files with minimal memory footprint.

**Implementation**: Binary data uses borrowed references to the original file buffer, allowing efficient partial extraction without copying large data blocks.

## Architectural Protection: Why These Rules Exist

### Real Consequences of Ignoring These Patterns

**These rules exist because engineers have repeatedly made these exact mistakes:**

**EXAMPLE 1: The 546-Line Deletion Disaster**
- Engineer deleted "complex-looking" pattern recognition code 
- Broke pack/map bit extraction, safe division, sprintf patterns
- Required 3-week emergency recovery to restore functionality
- Cost: Multiple engineer-weeks of work to fix

**EXAMPLE 2: AST String Parsing Vandalism** 
- Engineer bypassed visitor pattern with `split_whitespace()` 
- Broke on expressions containing spaces, quotes, nested structures
- Required complete rewrite of functions.rs with proper AST traversal
- Cost: Emergency architecture recovery documented in P07c

**EXAMPLE 3: Manual Transcription Silent Failures**
- Engineer hand-copied Canon lens database instead of using codegen
- Introduced typos in hex values that broke specific camera models
- Failures only discovered months later during real-world testing
- Cost: 100+ bug reports from manual transcription errors

**EXAMPLE 4: Arithmetic Operations Bug (2025)**
- Critical precedence climbing bugs completely ignored basic arithmetic
- `$val + 4` generated `val` instead of `(val + 4i32)` - silently wrong!
- Broke ExifTool compatibility for temperature conversions, offset calculations
- Required emergency fix with proper precedence climbing algorithm
- Cost: Potential data corruption for thousands of ExifTool expressions

**Why These Patterns Are Banned**: Each rule prevents specific disasters that have already happened. The arithmetic bug shows how AST processing bugs can silently corrupt data while appearing to work.

## Development Workflow

### How to Work Within This Architecture

**Daily Development**: See [GETTING-STARTED.md](GETTING-STARTED.md) for complete workflow guidance.

**Key Principles**:
1. **Use generated tables** - Never manually transcribe ExifTool data
2. **Follow Trust ExifTool** - Port logic exactly, cite source line numbers
3. **Run strategy system** - `make codegen` automatically discovers new patterns
4. **Test against ExifTool** - Verify output matches for real camera files

**Adding Support for New Tags**:
1. Check if lookup table exists in `src/generated/`
2. Implement PrintConv/ValueConv using generated data
3. Register function in appropriate registry
4. Test with real camera files containing those tags

**Strategy Development** (rare): Only needed when automatic pattern recognition misses new ExifTool symbol types. See [STRATEGY-DEVELOPMENT.md](STRATEGY-DEVELOPMENT.md) for detailed guidance.

## Why This Architecture Works

### Benefits of These Design Decisions

1. **Correctness**: Trust ExifTool principle ensures we handle manufacturer quirks correctly
2. **Maintainability**: Clear separation between generated and manual code
3. **Performance**: Hybrid expression system optimizes common cases while handling complex edge cases
4. **Robustness**: Stateful processing handles ExifTool's full complexity
5. **Incremental**: System works at every stage, clear visibility into what's missing
6. **Demand-Driven**: Only implement what's actually used in real files

### What We Avoid

1. **Perl Parsing Nightmares**: No attempts to automatically parse complex Perl expressions
2. **Stub Function Explosion**: No thousands of TODO functions cluttering the codebase  
3. **Premature Optimization**: Manual implementation focuses on correctness first
4. **Novel Algorithms**: No reinventing metadata parsing - ExifTool already solved it
5. **Architectural Regression**: Strict enforcement prevents vandalism

## Key Design Insights

### Why ExifTool's Approach is Necessary

**Camera Manufacturers Don't Follow Specs**: Every camera has firmware bugs and spec violations. ExifTool's seemingly complex logic exists because real-world files require it.

**State is Necessary**: EXIF metadata has complex interdependencies that require stateful processing. Functional approaches fail on real files.

**Pattern Recognition is Essential**: ExifTool has dozens of different data patterns. Each needs specialized handling for correct translation.

**Manual Excellence**: Complex logic requires human understanding of camera quirks. Automatic translation of complex expressions creates bugs.

### Common Misconceptions

**"Why not simplify ExifTool's logic?"** - ExifTool's complexity comes from handling real-world camera bugs. Simplification breaks compatibility.

**"Why not generate everything automatically?"** - Perl expressions are too complex and ambiguous for reliable automatic translation.

**"Why not use functional programming?"** - EXIF processing is inherently stateful due to complex tag interdependencies.

**"Why so many strategies?"** - ExifTool has many different data patterns that need specialized handling.

## Implementation Status

### Completed Systems
- Basic JPEG/EXIF parsing with state management
- Strategy-based code generation system
- PrintConv/ValueConv infrastructure with registry
- Canon MakerNote support with offset fixing
- Composite tag framework with runtime evaluation
- **Unified precedence climbing normalizer** (consolidated 6 expression normalizers into 1)
- Complete PPI→AST→Rust test generation pipeline with function deduplication
- Two-phase test generation with PpiFunctionRegistry integration
- TagValue arithmetic operators for expression evaluation
- **Fixed arithmetic operations** - all basic math (`+`, `-`, `*`, `/`) working correctly

### Current Focus
See [MILESTONES.md](MILESTONES.md) for active development priorities.

### Future Capabilities
- Additional manufacturers (Nikon, Panasonic) following the established patterns
- Video metadata (MP4, QuickTime) using the same architecture
- Write support maintaining full ExifTool compatibility

## Related Documentation

### Essential Reading
- [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md) - Fundamental principle for all work
- [ANTI-PATTERNS.md](ANTI-PATTERNS.md) - Critical mistakes that cause PR rejections
- [GETTING-STARTED.md](GETTING-STARTED.md) - Development workflow within this architecture

### Implementation Guides
- [CODEGEN.md](CODEGEN.md) - Strategy system and build pipeline
- [API-DESIGN.md](design/API-DESIGN.md) - Public API structure
- [PROCESSOR-DISPATCH.md](guides/PROCESSOR-DISPATCH.md) - Advanced processor selection

### Deep Dives
- [PRINTCONV-VALUECONV-GUIDE.md](guides/PRINTCONV-VALUECONV-GUIDE.md) - Expression processing details
- [EXIFTOOL-GUIDE.md](guides/EXIFTOOL-GUIDE.md) - Working with ExifTool source

## PPI Expression Processing: The Critical Path

### Unified Precedence Climbing (2025 Architecture)

**KEY SUCCESS**: Consolidated 6 separate normalizers into 1 using precedence climbing algorithm:

```
8 normalizers (3,383 lines) → 3 normalizers (683 lines)

✅ UNIFIED: ExpressionPrecedenceNormalizer handles:
  - Binary operations (+, -, *, /, ==, !=, etc.)
  - String operations (concatenation, regex)
  - Function calls with proper precedence
  - Ternary operations (?:)
  - Complex patterns (join+unpack, safe division)

✅ PRESERVED: Structural passes remain separate:
  - ConditionalStatementsNormalizer (if/return restructuring)
  - SneakyConditionalAssignmentNormalizer (complex control flow)
```

**Processing Flow**:
```
PPI Raw AST → ExpressionPrecedenceNormalizer → BinaryOperation nodes → Visitor → Rust code

Example: $val + 4
├─ Raw: [Symbol($val), Operator(+), Number(4)]
├─ Normalized: BinaryOperation("+", Symbol($val), Number(4))
└─ Generated: (val + 4i32)
```

### Critical Debugging Commands

```bash
# Test any expression through the full pipeline
cargo run --bin debug-ppi -- --verbose '$val + 4'

# Validate precedence climbing works
make generate-expression-tests && cargo test -p codegen --test generated_expressions

# Check if expression creates correct BinaryOperation node
RUST_LOG=debug cargo run --bin debug-ppi -- '$val * 100 + 50'
# Should show: "Created binary operation: *" and "Created binary operation: +"
```

### Expression Processing Gotchas

**🚨 CRITICAL**: The precedence climbing algorithm is **complex but essential**:
- **DO NOT** simplify the precedence logic - it handles Perl's exact operator precedence
- **DO NOT** bypass ExpressionPrecedenceNormalizer for "simple" cases
- **ALWAYS** test with `debug-ppi` tool before assuming expressions work

**Common Failure Modes**:
1. **Unary vs Binary Detection**: `+4` (unary) vs `$val + 4` (binary) requires left-operand context
2. **Precedence Bugs**: `$val + 4 * 2` must generate `(val + (4i32 * 2i32))`, not `((val + 4i32) * 2i32)`
3. **Token Position Tracking**: Parser position bugs cause operations to be silently dropped

## Conclusion

This architecture embraces ExifTool's complexity rather than fighting it. The design decisions exist to prevent the specific mistakes that have caused PR rejections and architectural damage in the past.

**Key Insight**: ExifTool's value isn't in its Perl syntax, but in 25+ years of accumulated knowledge about camera metadata quirks. We preserve this knowledge through careful architectural choices that keep engineers "on the rails" and prevent costly mistakes.

**For New Engineers**: Follow the established patterns, read the referenced docs, and ask questions. The complexity exists for real technical reasons, not architectural preferences. Understanding the **why** behind these decisions will make you a more effective contributor to the project.