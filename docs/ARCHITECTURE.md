# exif-oxide Architecture

**🚨 CRITICAL: This architecture is built on [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md) - the fundamental law of this project.**

## Overview

exif-oxide is a Rust translation of [ExifTool](https://exiftool.org/), focusing on mainstream metadata extraction from image files. This document provides a high-level overview of the architecture and design decisions.

**Scope**: This project targets mainstream metadata tags only (frequency > 80% in TagMetadata.json), reducing the implementation burden from 15,000+ tags to approximately 500-1000. We explicitly do not support ExifTool's custom tag definitions or user-defined tags.

## Core Philosophy

1. **No Novel Parsing**: ExifTool has already solved every edge case - we port, not reinvent. Follow [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md)
2. **Manual Excellence**: Complex logic is manually ported with references to ExifTool source
3. **Simple Codegen**: Generator handles only straightforward, unambiguous translations
4. **Always Working**: System produces compilable code at every stage
5. **Transparent Progress**: Clear visibility into what's implemented vs TODO
6. **Mainstream Focus**: Only implement tags with >80% frequency or marked mainstream
7. **Streaming First**: All binary data handled via streaming to minimize memory usage

## 🚨 ARCHITECTURAL PROTECTION RULES

**CRITICAL**: Before modifying PPI system, expression generation, or codegen infrastructure:

### 🚨 AST Processing Rules

- **NEVER parse strings that were AST nodes** - Use visitor pattern and structured traversal
- **NEVER delete ExifTool pattern recognition** - Patterns handle specific camera quirks
- **NEVER disable working infrastructure** - Fix integration issues, don't disable systems
- **ALWAYS use PpiNode structure** - Fight string parsing, embrace AST traversal
- **ALWAYS reference ExifTool source** - Include exact line numbers for patterns

### 🚨 Emergency Indicators

If you see these patterns, the architecture has been damaged:

```rust
// 🚨 VANDALISM INDICATORS - RESTORE IMMEDIATELY
args[1].split_whitespace()           // String parsing of AST
parts.contains(&"unpack")            // String matching on AST data  
// let normalized_ast = normalize()  // DISABLED  // Disabled infrastructure
expressions.rs: <400 lines          // Deleted pattern recognition
```

**RECOVERY**: See `docs/todo/P07c-emergency-ppi-recovery.md` for systematic restoration procedure.

## Key Insights from ExifTool Analysis

### PROCESS_PROC Complexity

- 121+ uses of ProcessBinaryData across manufacturers
- Custom processors for encrypted data (Nikon), serial data (Canon), text formats (JVC)
- Sophisticated dispatch with table-level and SubDirectory overrides
- No code sharing between processors - each is self-contained

### ProcessBinaryData Sophistication

- Variable-length formats with offset tracking (`var_string`, `var_int16u`)
- Hook mechanism for dynamic format assignment
- Bit-level extraction with Mask/BitShift
- Complex format expressions like `string[$val{3}]`

### Error Handling Excellence

- MINOR_ERRORS classification system
- Graceful degradation with corruption
- Manufacturer-specific quirk handling
- Size limits and validation boundaries

### Non-UTF-8 Data Handling

- Binary magic numbers with raw bytes (e.g., BPG format: `BPG\xfb`)
- Pattern matching on binary data streams
- Proper escaping for Rust string literals
- JSON-safe representation of non-UTF-8 content

### State Management Requirements

- PROCESSED hash for recursion prevention
- DataMember dependencies between tags
- VALUE hash for extracted data
- Directory context (Base, DataPos, PATH)

## System Architecture

### Expression Evaluation Architecture

One of the most sophisticated aspects of exif-oxide is its hybrid expression evaluation system that mirrors ExifTool's complexity while optimizing for performance.

#### The Three-System Hybrid

**1. Codegen Strategies (Compile-time)**
- **TagKitStrategy**: Generates optimized Rust code for regular tags with static dependencies
- **CompositeTagStrategy**: Generates dependency metadata and raw expressions for runtime evaluation  
- **Clean Separation**: Uses `is_composite_table` metadata flag to route symbols to appropriate strategies
- **Output**: TagKit produces optimized lookup code; CompositeTag produces `CompositeTagDef` registry

**2. Runtime Expression Evaluation**
- **Regular Tags**: Use compile-time generated optimized code for maximum performance
- **Composite Tags**: Use dynamic evaluation due to `$val[n]` dependency patterns that require runtime resolution
- **Three-Tier Execution**: Dynamic → Registry → Manual handling covers the full complexity spectrum

**3. Shared Infrastructure**
- **Single Expression Compiler**: Used by both compile-time optimization and runtime evaluation
- **Unified Conv Registry**: Handles complex string operations and function dispatch across both systems
- **ProcessorContext**: Provides rich context comparable to ExifTool's `$$self` object state

#### Why This Complexity Exists

**Regular Tags vs Composite Tags Have Fundamentally Different Requirements:**

- **Regular Tags**: Dependencies are known at extraction time from ExifTool modules
  - Can generate optimized Rust code with direct lookups and calculations
  - Example: `PrintConv => { 0 => 'Off', 1 => 'On' }` becomes a static HashMap
  - Performance: Zero runtime overhead for tag processing

- **Composite Tags**: Dependencies are resolved at runtime based on what tags exist in each file
  - Must evaluate expressions like `$val[1] =~ /^S/i ? -$val[0] : $val[0]` with actual tag values
  - Example: GPS latitude combining coordinate value + hemisphere reference
  - Performance: Dynamic evaluation only when composite tags are requested

#### Expression Complexity Spectrum

The system handles the full range of ExifTool expressions through strategic tier assignment:

```
Simple arithmetic ($val / 8)              → Compile-time optimization
Array access ($val[0] + $val[1])          → Runtime Tier 1 (Dynamic)  
Context queries ($$self{Make} eq "Canon") → Runtime Tier 1.5 (Context-aware)
Regex operations ($val =~ s/pat/rep/)     → Runtime Tier 2 (Registry)
Camera-specific quirks                    → Runtime Tier 3 (Manual)
```

#### Data Flow Architecture

```
ExifTool Source → Field Extraction → Strategy Dispatch → Generated Code + Runtime Systems
      ↓                   ↓                ↓                        ↓
Regular Tags        TagKitStrategy    Optimized Rust Code    Direct Execution  
Composite Tags   CompositeStrategy   Metadata + Raw Perl    Dynamic Evaluation
                                           ↓
                                   Three-Tier Execution:
                                   Dynamic → Registry → Manual
```

#### Key Architectural Benefits

1. **Performance Optimization**: Compile-time generation where dependencies are static
2. **Full ExifTool Compatibility**: Runtime evaluation handles dynamic dependency resolution  
3. **Maintainability**: Shared expression compiler eliminates duplication between systems
4. **Incremental Complexity**: Three-tier system gracefully handles simple → complex expressions
5. **Zero Overhead**: Regular tags have no runtime evaluation cost

#### Common Architectural Misconceptions

- **"Why not use a single system?"** → Regular and composite tags have incompatible dependency models
- **"Isn't three tiers over-complex?"** → Mirrors ExifTool's natural expression complexity spectrum
- **"Why runtime evaluation at all?"** → Composite dependencies can't be resolved until file processing
- **"Is this premature optimization?"** → 90%+ of tags are regular with static dependencies

This hybrid architecture preserves ExifTool's full expressiveness while optimizing the common case of regular tag processing. The complexity exists because camera metadata itself is complex, not due to architectural choices.

## Core Runtime Architecture: State Management & Offset Calculations

### State Management Patterns

ExifTool's stateful approach is proven for complex nested metadata. We translate this exactly per [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md).

#### Key State Components in ExifTool

**PROCESSED Hash - Infinite Loop Prevention**
- **Purpose**: Prevents infinite loops in circular directory references
- **Implementation**: `$$self{PROCESSED}{$addr} = $dirName` where `$addr` combines DirStart + DataPos + Base offsets
- **Critical for**: Maker notes that reference other sections

**VALUE Hash - Extracted Tag Storage**  
- **Purpose**: Stores all extracted tag values indexed by tag key
- **Features**: Supports duplicate tag handling, deletion, and metadata association

**DataMember Dependencies - Complex Interdependencies**
- **Purpose**: Earlier tags determine format/count/behavior of later tags
- **Examples**: Canon AF data (`NumAFPoints` determines array sizes), Format expressions (`int16s[$val{0}]`)
- **Resolution Strategy**: Sequential processing with `%val` hash accumulating values

**Directory Processing Context - Nested State Management**
- **Components**: PATH stack (`$$self{PATH}`), Directory Info (Base, DataPos, DirStart, DirLen)
- **State transitions**: Push/pop on directory entry/exit

#### Current Implementation: Stateful Reader Object

Our `ExifReader` object closely mirrors ExifTool's `$self`:

```rust
pub struct ExifReader {
    // Core state - equivalent to ExifTool's member variables
    pub(crate) extracted_tags: HashMap<(u16, String), TagValue>, // VALUE hash with namespace
    pub(crate) tag_sources: HashMap<(u16, String), TagSourceInfo>, // Enhanced conflict resolution
    pub(crate) header: Option<TiffHeader>,                  // TIFF header with byte order
    pub(crate) data: Vec<u8>,                              // Raw EXIF data buffer
    pub(crate) warnings: Vec<String>,                       // Parse errors (non-fatal)

    // Stateful processing features  
    pub(crate) processed: HashMap<u64, String>,            // PROCESSED hash prevention
    pub(crate) path: Vec<String>,                          // PATH stack tracking
    pub(crate) data_members: HashMap<String, DataMemberValue>, // DataMember storage
    pub(crate) base: u64,                                  // Current base offset
    pub(crate) processor_dispatch: ProcessorDispatch,      // PROCESS_PROC system
    pub(crate) maker_notes_original_offset: Option<usize>, // MakerNotes offset tracking
    pub(crate) composite_tags: HashMap<String, TagValue>,  // Composite tag computation
}
```

**Key Features:**
- **Memory safety**: No risk of dangling pointers or use-after-free  
- **Namespace-aware storage**: Tags stored with (tag_id, namespace) keys for conflict resolution
- **Enhanced source tracking**: TagSourceInfo includes namespace, IFD name, and processor context

### Offset Calculation Systems

ExifTool's offset management represents 20+ years of handling real-world camera firmware quirks. We embrace this complexity rather than simplify it.

#### ExifTool's Offset Management Architecture

**Core Directory Info Structure:**
- **`Base`**: Base offset for all pointers in directory (usually TIFF header position)
- **`DataPos`**: File position of data block containing directory  
- **`DirStart`**: Offset to directory start within data block
- **Critical Formula**: `absolute_file_offset = Base + DataPos + relative_offset`

#### Multiple Offset Calculation Schemes

**Standard EXIF/TIFF (Most Common)**
- Offsets relative to TIFF header (Base = 0 for main EXIF)

**Entry-Based Offsets (Panasonic, some others)**  
- Offsets relative to each 12-byte IFD entry position
- Detection: `if ($$dirInfo{EntryBased} or $$tagTablePtr{$tagID}{EntryBased})`

**Maker Note Base Fixing**
- Automatic base fixing with `Image::ExifTool::MakerNotes::FixBase($et, $dirInfo)`
- Manufacturer-specific patterns:
  - **Canon**: 4, 6, 16, or 28 byte offsets depending on model
  - **Nikon**: TIFF header at offset 0x0a from maker note start  
  - **Sony**: Offset 0 or 4 depending on model era

#### Current Implementation: Offset Management

```rust  
pub struct DirectoryInfo {
    pub name: String,        // Directory name for debugging and PATH tracking
    pub dir_start: usize,    // Start offset of directory within data
    pub dir_len: usize,      // Length of directory data
    pub base: u64,          // Base offset for pointer calculations (ExifTool's Base)
    pub data_pos: u64,      // File position of data block (ExifTool's DataPos)
    pub allow_reprocess: bool, // Whether this directory allows reprocessing
}

pub struct TiffHeader {
    pub byte_order: ByteOrder, // Byte order tracking
    pub magic: u16,           // TIFF validation (42 for TIFF, 85 for RW2)
    pub ifd0_offset: u32,     // Offset to first IFD
}
```

**Implementation Status:**
- ✅ **Basic offset tracking** - DirectoryInfo and base offset management
- ✅ **TIFF header handling** - Byte order and validation  
- ✅ **Endianness support** - Throughout the parsing pipeline
- 🔄 **Advanced manufacturer-specific offset fixing** - Planned for manufacturer-specific milestones

#### Integration Between State & Offsets

State management and offset calculations work together to handle ExifTool's complex directory traversal:

```rust
impl ExifReader {
    // Integration of state management and offset tracking:
    pub(crate) processed: HashMap<u64, String>,     // Recursion prevention
    pub(crate) path: Vec<String>,                   // Directory hierarchy  
    pub(crate) base: u64,                          // Offset calculations

    // Processing pipeline combines these elements:
    // 1. PROCESSED tracking prevents infinite loops during recursion
    // 2. PATH management maintains current directory context  
    // 3. Base offset calculations ensure correct pointer resolution
    // 4. DirectoryInfo carries offset context between processing levels
}
```

### Build Pipeline: Unified Strategy Pattern

**🎯 CURRENT (2025)**: The unified strategy pattern has replaced the legacy config-driven approach with automatic pattern recognition:

```
1. field_extractor.pl discovers ALL symbols automatically from any ExifTool module
2. Strategies compete using duck-typing to claim symbols based on structure
3. Winning strategies generate appropriate Rust code for their claimed symbols
4. Files are organized into semantic modules with clean naming conventions

ExifTool Source → [Universal Discovery] → [Strategy Competition] → Generated Code
                         ↓                        ↓                        ↓
                 field_extractor.pl        Duck-typing patterns    Direct Rust output
                 (finds ALL symbols)     (zero configuration)    (modular structure)
```

**Key Evolution**: Single universal extractor with intelligent pattern recognition replaces dozens of specialized extractors with manual configuration.

### Unified Strategy Pattern Architecture

The strategy system implements sophisticated pattern recognition to automatically classify and process ExifTool symbols:

#### Strategy Competition Flow

1. **Universal Extraction**: `field_extractor.pl` uses Perl symbol table introspection to extract ALL symbols (hashes and arrays) from ExifTool modules
2. **Strategy Dispatch**: 9 strategies compete using `can_handle()` duck-typing methods  
3. **Pattern Recognition**: First-match-wins with carefully ordered strategy priority
4. **Code Generation**: Winning strategies generate appropriate Rust structures

#### Available Strategies (Priority Order)

| Priority | Strategy | Pattern Recognition | Generates |
|---|---|---|---|
| **1** | `CompositeTagStrategy` | `is_composite_table: 1` | Runtime metadata for dynamic evaluation |
| **2** | `FileTypeLookupStrategy` | Description + Format objects | File type discrimination code |
| **3** | `MagicNumberStrategy` | Binary escape sequences | Magic number matching patterns |
| **4** | `MimeTypeStrategy` | String-to-string mappings | MIME type lookup tables |
| **5** | `SimpleTableStrategy` | All string values, no tag markers | Static HashMap lookups |
| **6** | `TagKitStrategy` | Tag definition markers | Complete tag processing bundles |
| **7** | `BinaryDataStrategy` | Binary data attributes | ProcessBinaryData structures |
| **8** | `BooleanSetStrategy` | All values equal 1 | HashSet existence checks |
| **9** | `ScalarArrayStrategy` | Arrays of primitives | Static array constants |

#### Key Benefits

- **🔍 Complete Discovery**: Finds ALL symbols automatically, no configuration needed
- **🧩 Duck Typing**: Intelligent pattern recognition adapts to ExifTool changes
- **⚡ Zero Configuration**: New modules work immediately without setup
- **📈 Self-Extending**: Strategies handle new patterns without extraction layer changes

### Key Components

1. **Unified Strategy Pattern System** (documented above)

   - **Universal Discovery**: `field_extractor.pl` extracts ALL symbols automatically (`codegen/scripts/field_extractor.pl`)
   - **Strategy Competition**: 9 strategies compete using duck-typing pattern recognition (`codegen/src/strategies/mod.rs`)
   - **Pattern Recognition**: `can_handle()` methods classify symbols without configuration 
   - **Code Generation**: Strategies generate appropriate Rust structures for claimed symbols
   - **Zero Configuration**: Complete automatic discovery eliminates JSON config maintenance

2. **Expression Evaluation System** (integrated with strategy pattern)

   - **Hybrid Architecture**: Compile-time optimization + runtime evaluation
   - **TagKitStrategy**: Regular tag code generation with optimized lookups (`codegen/src/strategies/tag_kit.rs`)
   - **CompositeTagStrategy**: Dynamic metadata generation for runtime evaluation (`codegen/src/strategies/composite_tag.rs`)  
   - **ValueConvEvaluator**: Three-tier runtime execution system (`src/composite_tags/value_conv_evaluator.rs`)
   - **Shared Infrastructure**: Expression compiler, conv registry, processor context shared between strategies

3. **ExifTool Integration** ([CODEGEN.md](CODEGEN.md))

   - **Automatic Symbol Discovery**: Universal extraction from any ExifTool module
   - **Pattern-Based Processing**: Strategies handle tag tables, lookups, and metadata automatically
   - **PrintConv/ValueConv Classification**: Automatic routing between compile-time and runtime processing
   - **Manufacturer-Specific Support**: Strategies recognize and generate manufacturer processors

4. **Public API** ([design/API-DESIGN.md](design/API-DESIGN.md))

   - Streaming-first design
   - TagEntry with value/print fields
   - ExifTool compatibility
   - Composite tag integration

5. **Core Runtime Architecture** (documented above in this document)

   - Stateful ExifReader object with PROCESSED hash and PATH tracking
   - DataMember dependencies for complex interdependencies
   - Offset management and state tracking with manufacturer-specific schemes
   - ProcessorContext for `$$self` equivalent functionality

6. **Processor Dispatch** ([guides/PROCESSOR-DISPATCH.md](guides/PROCESSOR-DISPATCH.md))

   - Conditional processor selection
   - Runtime evaluation integration
   - Manufacturer-specific routing

## Development Workflow

### Zero-Configuration Strategy System (Current)

**🎯 Main Workflow**: The unified strategy pattern eliminates most manual configuration:

1. **Universal Extraction**: Run `make codegen` - automatically discovers ALL symbols from all ExifTool modules
2. **Automatic Routing**: Strategies compete using duck-typing to claim symbols without configuration
3. **Pattern Recognition**: `can_handle()` methods classify symbols as tables, lookups, tags, etc.
4. **Code Generation**: Winning strategies generate appropriate Rust code automatically
5. **Validation**: Test generated code against ExifTool output for correctness

### Strategy Development (For New Patterns)

**When automatic recognition doesn't handle a new ExifTool pattern:**

1. **Analyze**: Check `strategy_selection.log` to see which symbols aren't being claimed
2. **Create Strategy**: Implement `ExtractionStrategy` trait with pattern recognition logic
3. **Pattern Recognition**: Define precise `can_handle()` method using duck-typing
4. **Priority Placement**: Insert strategy at correct priority level for pattern specificity
5. **Test**: Validate pattern recognition and generated code against ExifTool

### Manual Implementation (For Complex Logic)

**For logic too complex for automatic generation:**

1. **Discover**: Use `--show-missing` to find needed implementations
2. **Classify**: Determine if expression needs compile-time or runtime handling
3. **Implement**: Port complex logic manually referencing ExifTool source
4. **Integrate**: Ensure generated and manual code work together seamlessly

See [TPP.md](TPP.md) for detailed task planning and [GETTING-STARTED.md](GETTING-STARTED.md) for development workflow.

## Implementation Status

### Completed

- Basic JPEG/EXIF parsing
- PrintConv infrastructure
- Canon MakerNote support
- Sony MakerNote basics
- Composite tag framework
- ProcessBinaryData for simple formats

### In Progress

- See [MILESTONES.md](MILESTONES.md) for current work

### Future

- Additional manufacturers (Nikon, Panasonic)
- Video metadata (MP4, QuickTime)
- Write support
- Advanced encryption

## Key Design Decisions

### Manual Implementation Over Code Generation

We manually implement complex logic because:

- Perl expressions are too complex to parse reliably
- Manual code can reference ExifTool source directly
- Easier to debug and understand
- Better performance characteristics

### Runtime Fallback Over Compile-Time Stubs

Missing implementations return raw values instead of panicking:

- System remains usable during development
- Clear visibility into what's missing
- No stub function explosion

### Streaming Over In-Memory

Binary data uses streaming references:

- Handles large embedded images/videos
- Minimal memory footprint
- Efficient for partial extraction

## Benefits of This Architecture

1. **Realistic**: No Perl parsing, just manual porting of what matters
2. **Incremental**: Ship working code immediately, improve coverage over time
3. **Maintainable**: Clear separation between generated and manual code
4. **Traceable**: Every manual implementation references ExifTool source
5. **Robust**: Handles ExifTool's full complexity through manual excellence
6. **No Panic**: System never crashes on missing implementations
7. **Demand-Driven**: Only implement what's actually used in real files
8. **Zero Stub Spam**: No thousands of TODO functions cluttering the codebase

## Documentation Guide

### Design Documents

- [API Design](design/API-DESIGN.md) - Public API structure
- [ExifTool Integration](CODEGEN.md) - Unified code generation and implementation guide

### Guides

- [Getting Started](GETTING-STARTED.md) - Getting started with the codebase and development workflow
- [ExifTool Guide](guides/EXIFTOOL-GUIDE.md) - Complete guide to working with ExifTool source

### Milestones

- [Current Milestones](MILESTONES.md) - Active development work
- [Completed Milestones](archive/DONE-MILESTONES.md) - Historical progress

### Technical Deep Dives

- [Processor Dispatch](guides/PROCESSOR-DISPATCH.md) - Processor dispatch strategy
- [PrintConv/ValueConv Guide](guides/PRINTCONV-VALUECONV-GUIDE.md) - Expression processing and conversion system

## Getting Help

1. **Read the ExifTool source** - The answer is usually there
2. **Check module docs** - See `third-party/exiftool/doc/`
3. **Use --show-missing** - Let it guide your implementation priority
4. **Ask clarifying questions** - The codebase is complex by necessity

## Architectural Integrity Enforcement

### Code Review Requirements

**ALL PPI/CODEGEN PRs MUST**:

1. **No AST String Parsing** - Reject any `split_whitespace()` on AST data
2. **Pattern Recognition Intact** - Verify `expressions.rs` maintains ExifTool patterns  
3. **Infrastructure Enabled** - Check normalizer and other systems are active
4. **ExifTool References** - Require source line numbers for all patterns
5. **Regression Testing** - Generated code must match previous output exactly

### Detection Commands

```bash
# Pre-PR validation  
rg "split_whitespace|args\[.*\]\.starts_with" codegen/src/ppi/  # Must be empty
wc -l codegen/src/ppi/rust_generator/expressions.rs            # Must be >400
grep "DISABLED\|TODO" codegen/src/ppi/rust_generator/mod.rs    # Must be empty
```

### Architectural Recovery

If architectural damage is detected:

1. **STOP** - Do not merge damaged code
2. **ASSESS** - Check `docs/todo/P07c-emergency-ppi-recovery.md`
3. **RESTORE** - Use systematic recovery procedures
4. **VALIDATE** - Ensure generated code matches ExifTool exactly

## Conclusion

This architecture embraces the reality of ExifTool's complexity. Rather than trying to automatically handle Perl's flexibility, we:

1. Use codegen only for unambiguous translations
2. Manually port complex logic with full traceability
3. Build an implementation palette that grows over time
4. Maintain ExifTool compatibility through careful porting
5. **Protect architectural integrity from vandalism**

This approach is more labor-intensive but results in:

- Correct handling of manufacturer quirks
- Predictable performance
- Maintainable code
- Clear progress tracking
- **Architectural consistency over time**

The key insight: ExifTool's value isn't in its Perl code, but in the accumulated knowledge of metadata formats. We preserve this knowledge through careful manual translation, not automatic parsing. **We protect this knowledge from architectural regression through clear enforcement guidelines.**
