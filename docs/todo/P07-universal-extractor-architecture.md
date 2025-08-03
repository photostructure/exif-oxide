# Technical Project Plan: P07 Universal Extractor Architecture

## Project Overview

- **Goal**: Replace conservative config-driven extraction with universal symbol table introspection to eliminate config maintenance burden and create foundation for systematic tag coverage improvements
- **Problem**: Current conservative codegen approach with manual JSON configs creates "whack-a-mole" missing tags, requires ongoing maintenance for monthly ExifTool releases, and prevents achieving complete ExifTool compatibility
- **Constraints**: Must maintain compatibility with existing generated code during transition, preserve conv registry and expression compiler systems exactly as-is

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

- **Current codegen system**: **67 JSON configs** across 40+ modules with **11 actively used Perl extractors** (`tag_kit.pl`, `simple_table.pl`, etc.) specifying what to extract from ExifTool modules, generating focused Rust code for specific patterns
- **Config distribution**: TagKit (31 configs), CompositeTag (13), SimpleTable (8), others (15) - creates massive maintenance burden
- **Universal patching**: Recent improvement using `codegen/scripts/patch_all_modules.sh` to convert all `my` variables to `our` variables in ExifTool modules for symbol table access
- **ExifTool compatibility pipeline**: `make compat` tests 167 required tags across 303 files - currently 76 perfect correlation, **91 missing tags (54% failure rate)**
- **Strategy dispatch pattern**: Existing conv_registry and expression_compiler systems use strategy patterns to recognize and handle different conversion types

### Complete Inventory of Production Extraction Types (11 active types)

**High-Volume Extractors:**
1. **`TagKitExtractor`** (31 configs) → Complex tag definitions with embedded PrintConv expressions
2. **`CompositeTagsExtractor`** (13 configs) → Composite tags with require/desire dependencies  
3. **`SimpleTableExtractor`** (8 configs) → Basic HashMap lookups with LazyLock initialization

**Medium-Volume Extractors:**
4. **`TagTableStructureExtractor`** (3 configs) → Manufacturer table structure enums
5. **`ProcessBinaryDataExtractor`** (3 configs) → Binary data parsing table definitions
6. **`BooleanSetExtractor`** (2 configs) → Membership testing sets (PNG_pm, ExifTool_pm)

**Specialized Single-Use Extractors:**
7. **`SimpleArrayExtractor`** (1 config) → Static byte arrays for cryptographic operations (Nikon XLAT)
8. **`RuntimeTableExtractor`** (1 config) → Runtime HashMap creation with conditional logic
9. **`RegexPatternsExtractor`** (1 config) → Magic number patterns for file detection
10. **`ModelDetectionExtractor`** (1 config) → Camera model pattern recognition
11. **`FileTypeLookupExtractor`** (1 config) → File type detection with discriminated unions

**Unused Extractors** (implemented but no configs): `InlinePrintConvExtractor`, `TagDefinitionsExtractor`

### Key Concepts & Domain Knowledge

- **Symbol table introspection**: Perl's ability to examine all variables/hashes/arrays exposed by a loaded module via the `%package::` symbol table
- **Conservative extraction**: Current approach where configs specify exactly what to extract, leading to missed ExifTool data when configs are incomplete
- **JSON Lines format**: Streaming JSON where each line is a complete JSON object, enabling processing of large datasets without loading entire files into memory
- **Pattern recognition confidence**: Strategy pattern where multiple extractors examine data and return confidence levels (Perfect/Good/Maybe/No) for handling capability
- **Stateful accumulation**: Processing pattern where extractors maintain state across multiple files/modules and emit final generated code at completion

### Surprising Context

**CRITICAL**: Document non-intuitive aspects that aren't obvious from casual code inspection:

- **67 config files prevent complete extraction**: The biggest impediment to ExifTool compatibility is that our configs only specify what we *think* we need, not what ExifTool actually exposes - missing configs = missing tags forever
- **Perl symbol table is the source of truth**: ExifTool modules expose everything via symbol tables after universal patching, making configs redundant - we can discover everything ExifTool knows automatically
- **11 extractors create artificial boundaries**: Having separate `tag_kit.pl`, `simple_table.pl`, etc. creates false distinctions - the same ExifTool data could be handled by multiple extraction strategies depending on recognition patterns
- **JSON streaming prevents memory issues**: ExifTool modules can expose thousands of hashes/arrays (Canon.pm has 500+ symbols) - JSON Lines format allows processing arbitrarily large symbol table dumps without memory constraints  
- **Cross-module dependencies exist**: Some ExifTool patterns reference data from other modules (e.g., shared lookup tables) - stateful processing enables dependency resolution across the entire extraction pipeline
- **Pattern recognition is more reliable than categorization**: Instead of trying to categorize ExifTool data upfront, let strategies examine actual data structure and decide if they can handle it
- **Post-processing is critical**: Each extractor type produces completely different output structures - universal system must replicate all 11 output patterns exactly

### Foundation Documents

- **Design docs**: [CODEGEN.md](../CODEGEN.md) - Current extraction system, [ARCHITECTURE.md](../ARCHITECTURE.md) - High-level system overview
- **ExifTool source**: Universal patching in `codegen/scripts/patch_all_modules.sh`, symbol table introspection in `codegen/scripts/auto_config_gen.pl`
- **Start here**: `codegen/src/main.rs` (current orchestration), `codegen/extractors/` (existing pattern-specific extractors), `src/generated/` (current output structure)

### Prerequisites

- **Knowledge assumed**: Understanding of Perl symbol tables, Rust strategy/trait patterns, JSON Lines streaming format, current codegen pipeline architecture
- **Setup required**: Working codegen environment with universal patching applied, `make codegen` functional, test images available for validation

**Context Quality Check**: Can a new engineer understand WHY symbol table introspection eliminates the config maintenance problem and directly solves the 91 missing tags issue?

## Work Completed

- ✅ **Universal patching system** → `codegen/scripts/patch_all_modules.sh` successfully converts all ExifTool `my` variables to `our` for symbol table access
- ✅ **Auto-config generation proof of concept** → `codegen/scripts/auto_config_gen.pl` demonstrates complete symbol table introspection and automatic config generation
- ✅ **Strategy pattern precedent** → `codegen/src/conv_registry/` and `codegen/src/expression_compiler/` prove strategy-based pattern recognition works effectively in this codebase
- ✅ **Streaming JSON research** → JSON Lines format identified as solution for large symbol table dumps without memory constraints

## TDD Foundation Requirement

### Task 0: Golden Dataset Validation Test

**Purpose**: Ensure universal extraction produces equivalent or superset output compared to current config-driven system.

**Success Criteria**:
- [ ] **Test exists**: `tests/integration_p18_golden_dataset.rs:test_generated_code_equivalence`
- [ ] **Test fails**: `cargo t test_generated_code_equivalence` fails with "Universal extraction not implemented - using golden dataset for comparison"
- [ ] **Integration focus**: Test validates universal extraction generates superset of current `src/generated/**/*.rs` structure
- [ ] **TPP reference**: Test includes comment `// P07: Universal Extractor Architecture - see docs/todo/P18-universal-extractor-architecture.md`
- [ ] **Measurable outcome**: Test compares directory structures and verifies universal extraction includes all current functionality plus optional additions

**Implementation Strategy**:
1. **Archive current output**: `cp -r src/generated/ tests/fixtures/p18_golden_dataset/` before starting work
2. **Generate comparison**: Universal system writes to temporary directory, test compares structures
3. **Validation logic**: Every file/function in golden dataset must exist in universal output (superset validation)
4. **Cleanup after success**: Remove golden dataset and this test once universal extraction is proven equivalent

**Requirements**:
- Must validate that universal extraction produces at minimum all current generated code
- Should fail because universal extraction system replaces current config-driven pipeline
- Must demonstrate that no existing functionality is lost in architectural transition
- Include error message: `"// Fails until P07 complete - requires universal extraction to match or exceed golden dataset"`

**Quality Check**: Can you run the test, see it compare actual file structures, and understand exactly what functionality must be preserved during the transition?

## Remaining Tasks

### Task A: Universal Perl Extractor Implementation

**Success Criteria**:
- [ ] **Implementation**: Universal extractor script → `codegen/extractors/universal_extractor.pl` implements complete symbol table introspection
- [ ] **Integration**: Main pipeline uses universal extractor → `codegen/src/main.rs:89` calls universal extractor instead of specific extractors
- [ ] **JSON Lines output**: Streaming format implemented → `perl universal_extractor.pl Canon.pm` outputs `.jsonl` with one JSON object per symbol
- [ ] **Unit tests**: `cargo t test_universal_extractor_parsing` validates JSON Lines parsing in Rust
- [ ] **Manual validation**: `perl universal_extractor.pl third-party/exiftool/lib/Image/ExifTool/Canon.pm | wc -l` shows >500 extracted symbols
- [ ] **Cleanup**: N/A - keeping existing extractors during transition
- [ ] **Documentation**: Extractor documented → `codegen/extractors/README.md` explains universal extraction approach

**Implementation Details**: 
- Use existing `auto_config_gen.pl` symbol table introspection as foundation
- Extract ALL hashes, arrays, scalars, functions from symbol table with metadata
- Output JSON Lines format: `{"type": "hash", "name": "canonWhiteBalance", "data": {...}, "metadata": {...}}`
- Include symbol dependencies and cross-references for later processing

**Integration Strategy**: Replace current `extract_all_simple_tables()` call with universal extraction, modify JSON processing to handle JSON Lines format

**Validation Plan**: Compare symbol count between universal extractor and sum of all current extractors - should capture significantly more data

**Dependencies**: None

### Task B: Rust Strategy Recognition System with Complete Type Coverage

**Success Criteria**:
- [ ] **Implementation**: Strategy trait and dispatcher → `codegen/src/strategies/mod.rs` implements `ExtractionStrategy` trait and `StrategyDispatcher`
- [ ] **Integration**: Main pipeline uses strategy system → `codegen/src/main.rs:145` processes universal extractor output through strategy dispatcher
- [ ] **All 11 strategies implemented**: Complete coverage → All production extraction types in `codegen/src/strategies/`
- [ ] **Unit tests**: `cargo t test_strategy_recognition` validates confidence levels and pattern matching
- [ ] **Manual validation**: `RUST_LOG=debug cargo run` shows strategy recognition decisions for each symbol
- [ ] **Cleanup**: N/A - additive system
- [ ] **Documentation**: Strategy system documented → `codegen/src/strategies/README.md` explains pattern recognition approach

**Required Strategy Implementations** (must handle all 11 production patterns):
- `SimpleTableStrategy` → HashMap lookups (8 configs)
- `TagKitStrategy` → Complex tag definitions (31 configs) 
- `CompositeTagStrategy` → Dependencies and calculations (13 configs)
- `BooleanSetStrategy` → Membership testing (2 configs)
- `ArrayStrategy` → Static arrays (1 config)
- `BinaryDataStrategy` → Binary parsing tables (3 configs)
- `FileTypeStrategy` → File detection (1 config)
- `RuntimeTableStrategy` → Conditional HashMap creation (1 config)
- `RegexPatternStrategy` → Magic numbers (1 config)
- `ModelDetectionStrategy` → Camera patterns (1 config)
- `TagTableStructureStrategy` → Enums (3 configs)

**Implementation Details**:
```rust
trait ExtractionStrategy {
    fn can_handle(&self, data: &JsonValue, context: &ExtractionContext) -> ConfidenceLevel; 
    fn extract(&mut self, data: &JsonValue, context: &mut ExtractionContext) -> Result<()>;
    fn finish_file(&mut self, file_path: &str) -> Result<()>;
    fn finish_codegen(&mut self) -> Result<Vec<GeneratedFile>>;
}
```

**Post-Processing Requirement**: Each strategy's `finish_codegen()` must:
- Generate **identical file structures** to current system (exact API compatibility)
- Handle **cross-module dependencies** (shared lookups generated once)
- Maintain **module organization** (Canon_pm/, tag_kit/ subdirectories)
- Preserve **function signatures** for existing consumers
- Include proper **Rust documentation** and ExifTool source references

**Integration Strategy**: Process JSON Lines stream through strategy dispatcher, let highest-confidence strategy handle each symbol

**Validation Plan**: Log strategy decisions and confidence levels, verify no symbols are unhandled due to strategy gaps

**Dependencies**: Task A complete (universal extractor provides JSON Lines input)

### Task C: Stateful Cross-Module Processing

**Success Criteria**:
- [ ] **Implementation**: Extraction context system → `codegen/src/strategies/context.rs` implements stateful accumulation across files/modules
- [ ] **Integration**: Strategies use shared context → All strategy implementations use `ExtractionContext` for cross-module state
- [ ] **Dependency resolution**: Context tracks dependencies → `ExtractionContext::resolve_dependencies()` generates code in correct order
- [ ] **Unit tests**: `cargo t test_stateful_processing` validates state accumulation and dependency resolution
- [ ] **Manual validation**: Process multiple modules and verify shared lookups are generated once, referenced correctly
- [ ] **Cleanup**: N/A - new capability
- [ ] **Documentation**: Context system documented → `codegen/src/strategies/CONTEXT.md` explains stateful processing

**Implementation Details**:
- `ExtractionContext` maintains state across multiple ExifTool modules
- Strategies accumulate related data (e.g., all tag definitions) across modules
- Final code generation happens after all modules processed, enabling cross-module optimization

**Integration Strategy**: Pass shared context through all strategy operations, generate final code in `finish_codegen()` phase

**Validation Plan**: Verify strategies can share data across modules, dependencies resolved correctly, no duplicate code generation

**Dependencies**: Task B complete (strategy system provides context framework)

### Task D: Golden Dataset Validation and Migration

**Success Criteria**:
- [ ] **Implementation**: Golden dataset comparison → `tests/integration_p18_golden_dataset.rs` implements superset validation against archived `src/generated/`
- [ ] **Integration**: Universal extraction produces validated output → Test compares universal output against golden dataset for completeness
- [ ] **Task 0 passes**: `cargo t test_generated_code_equivalence` now succeeds with superset validation
- [ ] **Unit tests**: `cargo t test_file_structure_equivalence` validates directory structure and API compatibility
- [ ] **Manual validation**: `make compat` shows maintained compatibility at 76/167 baseline (no regression)
- [ ] **Cleanup**: Remove golden dataset and validation test → `rm -rf tests/fixtures/p18_golden_dataset/ tests/integration_p18_golden_dataset.rs` after successful migration
- [ ] **Documentation**: Migration guide → `docs/UNIVERSAL-EXTRACTOR-MIGRATION.md` documents transition process and golden dataset approach

**Implementation Details**:
- Archive current `src/generated/**/*.rs` as golden dataset before starting work
- Universal extraction writes to temporary directory for comparison
- Validation ensures every function/struct/module in golden dataset exists in universal output
- Allow superset (additional generated code) but prevent subset (missing functionality)

**Integration Strategy**: Use golden dataset as authoritative reference for backwards compatibility during architecture transition

**Validation Plan**: File-by-file comparison ensuring no existing generated code is lost, with optional additions allowed

**Dependencies**: Task C complete (stateful processing generates final code)

## Integration Requirements

**CRITICAL**: Building without integrating is failure. Don't accept tasks that build "shelf-ware."

### Mandatory Integration Proof

Every feature must include specific evidence of integration:
- [ ] **Activation**: Universal extraction used by default → `codegen/src/main.rs` calls universal extractor instead of config-driven extractors
- [ ] **Consumption**: Generated code uses universal extraction output → `src/generated/` contains equivalent structure to current system
- [ ] **Measurement**: Can prove compatibility maintained → `make compat` shows no regression from 76/167 baseline
- [ ] **Cleanup**: Config files eliminated → `find codegen/config -name "*.json" | wc -l` returns 0 after migration

### Integration Verification Commands

**Production Usage Proof**:
- `grep -r "universal_extractor" codegen/src/` → Should show usage in main pipeline
- `make compat` → Should show significant improvement in tag compatibility percentage
- `ls codegen/config/*/` → Should be empty or significantly reduced after universal extraction

**Red Flag Check**: If this seems like "build better extraction but keep using configs," ask for clarity. We're eliminating configs entirely and using pure symbol table introspection.

## Working Definition of "Complete"

A feature is complete when:
- ✅ **System behavior changes** - Universal extraction captures all ExifTool-exposed data instead of conservative config subsets
- ✅ **Default usage** - New extraction runs automatically, configs eliminated
- ✅ **Old path removed** - Manual config maintenance eliminated, conservative extraction replaced
- ❌ Code exists but configs still required *(example: "universal extractor implemented but still using config files")*
- ❌ Feature works "if you call it directly" *(example: "universal extraction works but main pipeline still uses old extractors")*

## Prerequisites

None - this is fundamental architecture work that other improvements depend on

## Testing

- **Unit**: Test universal extractor symbol recognition, strategy pattern matching, context state management
- **Integration**: Verify end-to-end extraction produces equivalent or superior output to current system
- **Manual check**: Run `make compat` and confirm no regression from 76/167 baseline compatibility

## Definition of Done

- [ ] `cargo t test_universal_extraction` passes (Task 0)
- [ ] `make precommit` clean
- [ ] `make compat` maintains current tag compatibility (76/167 baseline, no regression)
- [ ] Config file count reduced to near zero: `find codegen/config -name "*.json" | wc -l` < 5
- [ ] Universal extraction handles all ExifTool modules without manual intervention

## Implementation Guidance

### Generated Code Output Patterns (must preserve exactly)

**`SimpleTableStrategy`** → `canonwhitebalance.rs`:
```rust
static CANON_WHITE_BALANCE_DATA: &[(u8, &'static str)] = &[...];
pub static CANON_WHITE_BALANCE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(...);
pub fn lookup_canon_white_balance(key: u8) -> Option<&'static str> { ... }
```

**`BooleanSetStrategy`** → `isdatchunk.rs`:
```rust
pub static PNG_DATA_CHUNKS: LazyLock<HashMap<String, bool>> = LazyLock::new(...);
pub fn lookup_isdatchunk(key: &String) -> bool { PNG_DATA_CHUNKS.contains_key(key) }
```

**`TagKitStrategy`** → `tag_kit/core.rs`:
```rust
static PRINT_CONV_3: LazyLock<HashMap<String, &'static str>> = LazyLock::new(...);
// Complex PrintConv mappings with inline expressions vs function references
```

**`ArrayStrategy`** → `xlat_0.rs`:
```rust
pub static XLAT_0: [u8; 256] = [193, 191, 109, ...]; // Direct indexing for crypto
```

### Recommended Patterns

- **JSON Lines streaming**: Use `serde_json::Deserializer::from_reader()` with `BufReader` for memory-efficient processing
- **Strategy confidence levels**: Use numeric confidence scores (0-100) instead of enums for tie-breaking
- **Graceful degradation**: Unknown patterns should generate placeholder code with warnings, not fail extraction
- **Complete post-processing**: Every strategy must fully implement `finish_codegen()` to produce final Rust files

### Tools to Leverage

- **Existing symbol table code**: Build on `auto_config_gen.pl` symbol introspection logic
- **Current strategy patterns**: Follow conv_registry and expression_compiler architecture for consistency
- **Universal patching**: Leverage existing `patch_all_modules.sh` for symbol table access

### ExifTool Translation Notes

- **Symbol table completeness**: Extract everything exposed, let strategies filter - don't pre-filter in Perl
- **Dependency preservation**: Maintain ExifTool symbol relationships and cross-references in extraction
- **Metadata inclusion**: Extract not just data but ExifTool's own metadata about each symbol for informed processing

## Additional Gotchas & Tribal Knowledge

- **Symbol table size varies dramatically** → Canon.pm has 500+ symbols, others have <50 → JSON Lines streaming essential for memory management
- **Not all symbols are extractable** → Some ExifTool symbols are pure Perl code or complex references → Strategy confidence levels handle this gracefully
- **ExifTool modules have circular dependencies** → Some modules reference each other → Stateful processing resolves dependencies after all modules loaded
- **Generated code backward compatibility is critical** → Existing code expects specific generated APIs → Golden dataset validation prevents breaking changes
- **Universal extraction will find previously unknown patterns** → May discover ExifTool data types we haven't seen before → Strategy system designed to be extensible for new patterns
- **Golden dataset is temporary validation tool** → Archive current `src/generated/` before starting, delete after successful migration → Don't let it become permanent fixture
- **Superset validation allows exploration** → Universal extraction may generate additional useful code beyond current configs → Test allows additions but prevents losses
- **67 configs create maintenance nightmare** → Every ExifTool release potentially requires updating dozens of configs → Universal extraction eliminates this entirely
- **Post-processing complexity is the real challenge** → Universal extraction is straightforward Perl, but each of the 11 strategies must replicate exact output patterns → Most implementation effort will be in `finish_codegen()` methods
- **Strategy recognition patterns** → HashMap = SimpleTable, membership test = BooleanSet, tag definitions with PrintConv = TagKit, static arrays = Array, etc.
- **Module organization must be preserved** → Canon_pm/ directories, tag_kit/ subdirectories, exact function names → Any API changes break existing consumers

## Quick Debugging

Stuck? Try these:

1. `perl codegen/extractors/universal_extractor.pl Canon.pm | head -20` - See what symbols are extracted
2. `RUST_LOG=debug cargo run` - Watch strategy recognition decisions
3. `make compat` - Measure tag compatibility improvement  
4. `git log --oneline -10 codegen/` - Check recent codegen changes for conflicts