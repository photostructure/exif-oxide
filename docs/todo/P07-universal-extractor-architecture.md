# Technical Project Plan: P07 Universal Extractor Architecture

## Project Overview

- **Goal**: Replace config-driven extraction with universal symbol table introspection and strategy pattern system to eliminate maintenance burden and achieve systematic ExifTool compatibility
- **Problem**: 67 JSON configs create "whack-a-mole" missing tags, require manual maintenance for monthly ExifTool releases, and prevent complete ExifTool compatibility (91 missing tags, 54% failure rate)
- **Constraints**: Must maintain API compatibility with existing generated code during transition, preserve exact output structure and function signatures

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

- **Current codegen system**: 67 JSON configs across 40+ modules with 11 actively used Perl extractors, generating focused Rust code for specific patterns
- **Universal patching foundation**: `codegen/scripts/patch_all_modules.sh` converts all ExifTool `my` variables to `our` variables for symbol table access
- **Strategy pattern precedent**: Existing `conv_registry` and `expression_compiler` systems use strategy patterns for pattern recognition and code generation
- **ExifTool compatibility pipeline**: `make compat` tests 167 required tags across 303 files - currently 76 perfect correlation, **91 missing tags (54% failure rate)**

### Key Concepts & Domain Knowledge

- **Universal symbol table introspection**: Perl's ability to examine all variables/hashes/arrays exposed by a loaded module via `%package::` symbol table
- **JSON Lines format**: Streaming JSON (one JSON object per line) enabling processing of large datasets without memory constraints
- **Duck-typing pattern recognition**: Strategies examine JSON blob structure and return boolean "can handle" decision
- **Strategy pattern**: Multiple extractors examine data and determine handling capability, first-match-wins processing
- **Non-serializable values**: ExifTool symbols may contain function references, blessed objects, code refs that break JSON serialization

### Surprising Context

**CRITICAL**: Document non-intuitive aspects that aren't obvious from casual code inspection:

- **67 config files prevent complete extraction**: Missing configs = missing tags forever, no matter how comprehensive ExifTool's actual coverage is
- **Perl symbol table is comprehensive**: After universal patching, every ExifTool data structure is accessible - configs become redundant
- **Non-serializable values exist**: ExifTool symbols may contain function references, blessed objects, code refs that break JSON serialization
- **JSON Lines prevents memory issues**: Canon.pm alone exposes 500+ symbols - streaming format essential for processing large modules
- **Pattern recognition eliminates categorization**: Instead of pre-categorizing data, let strategies examine actual structure and decide handling capability
- **API compatibility is critical**: Generated code consumers expect exact function signatures and module organization
- **Compilation is the ultimate compatibility test**: If `src/` compiles with new `src/generated/`, API compatibility is maintained

### Foundation Documents

- **Design docs**: [CODEGEN.md](../CODEGEN.md) - Current extraction system, [ARCHITECTURE.md](../ARCHITECTURE.md) - High-level system overview
- **ExifTool source**: Universal patching in `codegen/scripts/patch_all_modules.sh`, symbol table introspection in `codegen/scripts/auto_config_gen.pl`
- **Start here**: `codegen/src/main.rs` (current orchestration), `codegen/src/extractors/` (existing extractors), `src/generated/` (current output structure)

### Prerequisites

- **Knowledge assumed**: Understanding of Perl symbol tables, Rust trait patterns, JSON Lines streaming format, current codegen pipeline architecture
- **Setup required**: Working codegen environment with universal patching applied, `make codegen` functional, test images available for validation

**Context Quality Check**: Can a new engineer understand WHY universal symbol table introspection eliminates config maintenance and directly solves the 91 missing tags issue?

## Work Completed

- ✅ **Universal patching system** → `codegen/scripts/patch_all_modules.sh` successfully converts all ExifTool `my` variables to `our` for symbol table access
- ✅ **Auto-config generation proof of concept** → `codegen/scripts/auto_config_gen.pl` demonstrates complete symbol table introspection
- ✅ **Strategy pattern precedent** → `codegen/src/conv_registry/` proves strategy-based pattern recognition works in this codebase
- ✅ **ExifTool pattern research** → Analysis of 100+ modules confirms most single-use extractors are appropriately specialized
- ✅ **Task A: Universal Perl Symbol Extractor** → Complete implementation with working integration

## TDD Foundation Requirement

### Task 0: Compilation-Based Validation (Simplified)

**Not applicable** - API compatibility maintained by design, validated through compilation

**Success Criteria**: `make clean-all codegen check` succeeds - if `src/` compiles with new `src/generated/`, API compatibility is maintained

**Justification for skipping Task 0**:
- **Pure architecture change**: Universal extraction maintains identical API by design - generates same function signatures and module organization
- **Compilation is sufficient validation**: If existing consumer code compiles with new generated code, API compatibility is preserved
- **No behavior changes**: Universal extraction captures more data but maintains same interfaces - superset behavior, not different behavior
- **Final validation available**: `make clean-all codegen check` provides definitive compatibility proof

**Final Validation Command**: `make clean-all codegen check` as ultimate integration test

## Remaining Tasks

### Task A: Universal Perl Symbol Extractor with Non-Serializable Detection

**Success Criteria**:
- [ ] **Implementation**: Universal extractor script → `codegen/extractors/universal_extractor.pl` implements complete symbol table introspection with graceful error handling
- [ ] **Non-serializable detection**: Problem values logged → `codegen/generated/extract/non_serializable.log` contains function refs, blessed objects, code refs that can't be JSON serialized
- [ ] **Integration**: Main pipeline calls universal extractor → `codegen/src/main.rs` replaces config-based extraction with universal approach
- [ ] **JSON Lines output**: Streaming format → `perl universal_extractor.pl Canon.pm` outputs `.jsonl` with one JSON object per symbol
- [ ] **Unit tests**: `cargo t test_universal_extractor_parsing` validates JSON Lines parsing in Rust
- [ ] **Manual validation**: `perl universal_extractor.pl third-party/exiftool/lib/Image/ExifTool/Canon.pm | wc -l` shows >500 extracted symbols
- [ ] **Cleanup**: N/A - keeping existing extractors during transition
- [ ] **Documentation**: Updated extractor guide → `codegen/extractors/README.md` explains universal extraction approach

**Implementation Details**: 
- Build on existing `auto_config_gen.pl` symbol table introspection
- Extract ALL hashes, arrays, scalars from symbol table with metadata
- Detect non-serializable values: `ref($value) && !$is_serializable` → log to `non_serializable.log`
- Output JSON Lines format: `{"type": "hash", "name": "canonWhiteBalance", "data": {...}, "module": "Canon", "metadata": {...}}`
- Include symbol type information for strategy pattern matching

**Integration Strategy**: Replace current individual extractor calls with single universal extraction phase

**Validation Plan**: Compare symbol count with sum of current extractions - should capture significantly more data

**Dependencies**: None

### Task B: Strategy Pattern System with Boolean Can-Handle

**Success Criteria**:
- [ ] **Implementation**: Strategy trait and dispatcher → `codegen/src/strategies/mod.rs` implements `ExtractionStrategy` trait and `StrategyDispatcher`
- [ ] **Integration**: Main pipeline uses strategy system → `codegen/src/main.rs` processes universal extractor output through strategies
- [ ] **All 11 strategies implemented**: Complete coverage → All current extraction types converted to strategies in `codegen/src/strategies/`
- [ ] **Conflict resolution**: Strategy selection logged → `codegen/generated/extract/strategy_selection.log` shows which strategy handled each symbol
- [ ] **Unit tests**: `cargo t test_strategy_recognition` validates pattern matching for each strategy type
- [ ] **Manual validation**: `RUST_LOG=debug cargo run` shows strategy selection decisions for each symbol
- [ ] **Cleanup**: Remove config-driven extraction calls → Main pipeline no longer reads individual JSON configs
- [ ] **Documentation**: Strategy system guide → `codegen/src/strategies/README.md` explains pattern recognition approach

**Strategy Interface**:
```rust
trait ExtractionStrategy {
    fn name(&self) -> &'static str;
    fn can_handle(&self, symbol_data: &JsonValue) -> bool;
    fn extract(&mut self, symbol_data: &JsonValue, context: &mut ExtractionContext) -> Result<()>;
    fn finish_module(&mut self, module_name: &str) -> Result<()>;
    fn finish_extraction(&mut self) -> Result<Vec<GeneratedFile>>;
}
```

**Conflict Resolution**: First-match wins with predictable ordering (TagKitStrategy → SimpleTableStrategy → others)

**Required Strategy Implementations** (maintain existing functionality):
- `TagKitStrategy` → Recognizes tag tables with PrintConv: complex tag definition patterns
- `SimpleTableStrategy` → Recognizes HashMap patterns: `{"key": "value"}` structures  
- `CompositeTagStrategy` → Recognizes composite definitions: dependency and calculation patterns
- `BooleanSetStrategy` → Recognizes membership sets: `{"key": 1}` patterns for existence checking
- `BinaryDataStrategy` → Recognizes ProcessBinaryData tables: offset/format/tag structures
- Plus 6 specialized strategies for manufacturer-specific patterns

**Integration Strategy**: Process JSON Lines stream through strategy dispatcher, let compatible strategies handle each symbol

**Validation Plan**: Each strategy produces byte-identical output to current extractor for same input

**Dependencies**: Task A complete (universal extractor provides JSON Lines input)

### Task C: Standardized Output Location System

**Success Criteria**:
- [ ] **Implementation**: Output location logic → `codegen/src/strategies/output_locations.rs` implements standardized path generation
- [ ] **Integration**: All strategies use standard locations → Generated files follow consistent `src/generated/[Module]_pm/[symbol_name].rs` pattern
- [ ] **Generic pattern support**: Cross-module patterns → Static arrays generate to `src/generated/[Module]_pm/arrays/[array_name].rs`
- [ ] **Unit tests**: `cargo t test_output_locations` validates path generation logic
- [ ] **Manual validation**: `find src/generated -name "*.rs" | head -20` shows consistent naming patterns
- [ ] **Cleanup**: Remove inconsistent output paths → All generated files follow standard location patterns
- [ ] **Documentation**: Output location guide → `docs/OUTPUT-LOCATIONS.md` documents standard path patterns

**Implementation Details**:
- Module-specific data: `src/generated/[Module]_pm/[symbol_name].rs`
- Generic patterns: `src/generated/[Module]_pm/[pattern_type]/[symbol_name].rs`
- Maintain existing module organization for compatibility
- Symbol names converted to snake_case for file names

**Integration Strategy**: All strategies call standardized path generation functions

**Validation Plan**: Verify all generated files follow consistent naming and organization patterns

**Dependencies**: Task B complete (strategies need output location logic)

### Task D: Consumer Code Update and Final Integration

**Success Criteria**:
- [ ] **Implementation**: Import path updates → Non-generated code updated to use new standardized paths (if any changes needed)
- [ ] **Integration**: All consumers work with new structure → `cargo build` succeeds with updated imports
- [ ] **Final validation**: `make clean-all codegen check` succeeds with API compatibility verification
- [ ] **Unit tests**: `cargo t` - all existing tests pass with new import paths
- [ ] **Manual validation**: `make compat` maintains 76/167 baseline compatibility (no regression)
- [ ] **Cleanup**: Config files removed → `rm -rf codegen/config/` since universal extraction eliminates config dependency
- [ ] **Documentation**: Migration completed → Update relevant docs with new architecture

**Implementation Details**:
- Update import statements in `src/implementations/` to use new generated paths (if needed)
- Ensure all existing function calls work with new module organization
- Maintain backward compatibility during transition

**Integration Strategy**: Systematic update of all non-generated code that imports generated modules (may be none needed)

**Validation Plan**: Build succeeds, all tests pass, ExifTool compatibility maintained or improved

**Dependencies**: Task C complete (standardized output locations available)

## Integration Requirements

**CRITICAL**: Building without integrating is failure. Don't accept tasks that build "shelf-ware."

### Mandatory Integration Proof

Every feature must include specific evidence of integration:
- [ ] **Activation**: Universal extraction used by default → `codegen/src/main.rs` calls universal extractor instead of config-driven extractors  
- [ ] **Consumption**: Generated code maintains API compatibility → All existing consumers continue working without changes
- [ ] **Measurement**: Can prove compatibility maintained → `make compat` shows no regression from 76/167 baseline
- [ ] **Cleanup**: Config dependency eliminated → `make codegen` works without individual JSON configs

### Integration Verification Commands

**Production Usage Proof**:
- `grep -r "universal_extractor" codegen/src/` → Should show usage in main pipeline
- `make compat` → Should maintain current compatibility baseline (no regression)
- `cargo build` → Should succeed with new generated code structure

**Red Flag Check**: If this becomes "build universal extraction but keep using configs," that's failure. We're eliminating config dependency entirely.

## Working Definition of "Complete"

A feature is complete when:
- ✅ **System behavior changes** - Universal extraction captures all ExifTool-exposed data instead of conservative config subsets
- ✅ **Default usage** - New extraction runs automatically, config dependency eliminated  
- ✅ **Old path removed** - Manual config maintenance eliminated, conservative extraction replaced
- ❌ Code exists but configs still required *(example: "universal extractor implemented but still using config files")*
- ❌ Feature works "if you call it directly" *(example: "universal extraction works but main pipeline still uses old extractors")*

## Prerequisites

None - this is fundamental architecture work that other improvements depend on

## Testing

- **Unit**: Test universal extractor symbol recognition, strategy pattern matching, output location generation
- **Integration**: Verify end-to-end extraction produces API-compatible output to current system
- **Manual check**: Run `make compat` and confirm no regression from 76/167 baseline compatibility

## Definition of Done

- [ ] `make clean-all codegen check` succeeds (compilation-based API compatibility)
- [ ] `make precommit` clean
- [ ] `make compat` maintains current tag compatibility (76/167 baseline, no regression)
- [ ] Universal extraction handles all ExifTool modules without config dependency
- [ ] Generated code maintains exact API compatibility with existing consumers

## Implementation Guidance

### Generated Code API Compatibility (must preserve exactly)

**SimpleTableStrategy** → `canonwhitebalance.rs`:
```rust
static CANON_WHITE_BALANCE_DATA: &[(u8, &'static str)] = &[...];
pub static CANON_WHITE_BALANCE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(...);
pub fn lookup_canon_white_balance(key: u8) -> Option<&'static str> { ... }
```

**StaticArrayStrategy** → `arrays/xlat_0.rs`:
```rust
pub static XLAT_0: [u8; 256] = [193, 191, 109, ...]; // Direct indexing for crypto
```

**TagKitStrategy** → `tag_kit/core.rs`:
```rust
static PRINT_CONV_3: LazyLock<HashMap<String, &'static str>> = LazyLock::new(...);
// Complex PrintConv mappings with inline expressions vs function references
```

### Recommended Patterns

- **JSON Lines streaming**: Use `serde_json::Deserializer::from_reader()` with `BufReader` for memory-efficient processing
- **Duck-typing recognition**: Simple boolean matching rather than complex confidence scoring
- **Standard output locations**: Follow existing `src/generated/[Module]_pm/` patterns
- **API preservation**: Every generated function must maintain exact signature and location

### Tools to Leverage

- **Existing symbol table code**: Build on `auto_config_gen.pl` symbol introspection logic
- **Current strategy patterns**: Follow conv_registry architecture for consistency
- **Universal patching**: Leverage existing `patch_all_modules.sh` for symbol table access

## Additional Gotchas & Tribal Knowledge

**Format**: Surprise → Why → Solution

- **Non-serializable values break extraction** → ExifTool has function refs, blessed objects → Log to `non_serializable.log`, continue with other symbols
- **Multiple strategies claim same symbol** → Pattern overlap possible → First-match wins with predictable ordering (TagKit → SimpleTable → others)
- **Universal extraction generates massive code** → All symbols vs selective configs → Monitor build times, consider lazy loading for large tables
- **Strategy conflicts hard to debug** → Complex symbol structures → Log all decisions to `strategy_selection.log` with reasoning
- **Config removal breaks existing workflows** → Engineers expect config files → Document new universal approach, provide migration guide

## Quick Debugging

Stuck? Try these:

1. `perl codegen/extractors/universal_extractor.pl Canon.pm | head -20` - See what symbols are extracted
2. `RUST_LOG=debug cargo run` - Watch strategy selection decisions  
3. `make compat` - Measure tag compatibility baseline
4. `git log --oneline -10 codegen/` - Check recent codegen changes for conflicts