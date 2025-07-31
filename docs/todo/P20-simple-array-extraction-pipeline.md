# Technical Project Plan: Simple Array Extraction Pipeline

## Project Overview

- **Goal**: Create a specialized codegen extractor for perl arrays
- **Problem**: Manual transcription of 512-byte XLAT arrays creates high error risk ("4 engineering days chasing ghosts"), violates Trust ExifTool principle, and requires ongoing maintenance with monthly ExifTool releases
- **Constraints**: Must integrate seamlessly with existing trait-based extractor system, support thread-safe patching, and maintain performance characteristics for cryptographic operations

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

**REQUIRED**: Assume reader is unfamiliar with this domain. Provide comprehensive context.

### System Overview

- **Codegen Architecture**: Trait-based extractor system with discovery → patching → extraction → generation pipeline. Each extractor implements `Extractor` trait defining script_name, patching requirements, and config handling. Uses parallel execution with rayon for performance across 13+ existing extractors.
- **Patching System**: Thread-safe conversion of ExifTool `my`-scoped variables to `our`-scoped using per-file mutexes and atomic file operations. Required because perl arrays like `my @xlat` are private and cannot be accessed by external extraction scripts.
- **Perl extraction**: We use `perl` scripts to **actually interpret and run the ExifTool perl code** because the only thing that can reliably parse Perl is `perl`. 
- **Generator Pipeline**: Configuration-driven code generation that reads extracted JSON data and produces modular Rust code. Uses `FileGenerator` struct for consistent file structure with proper imports, documentation, and module organization.
- **ExifTool Integration**: Monthly ExifTool releases contain updated constants, algorithms, and data structures. Automated extraction eliminates manual maintenance burden and transcription errors across 15,000+ tags and hundreds of lookup tables.

### Key Concepts & Domain Knowledge

- **Perl Array Indexing**: ExifTool uses patterns like `$xlat[0][$serial & 0xff]` and `$xlat[1][$key]` for array-based lookups. Unlike hashes, arrays provide direct index access with O(1) performance and are commonly used for cryptographic constants and binary data structures.
- **XLAT Cryptographic Tables**: Two 256-element arrays containing XOR decryption constants for Nikon encrypted sections. ExifTool source: Nikon.pm lines 13505-13538. Used in Decrypt function (lines 13566-13567) for initializing decryption parameters with serial number and shutter count keys.
- **Trust ExifTool Principle**: Core project law requiring exact translation of ExifTool logic without "improvements" or "optimizations". Exists because every seemingly odd piece of code handles specific camera quirks discovered over 25 years of development across thousands of camera models with unique firmware bugs.
- **Configuration-Driven Extraction**: Each extractor type has JSON schema validation and config files specifying source paths, hash/array names, and generation parameters. System auto-discovers all configs and processes them in parallel with proper error handling and validation.

### Surprising Context

**CRITICAL**: Document non-intuitive aspects that aren't obvious from casual code inspection:

- **Cryptographic Sensitivity**: Single byte errors in XLAT arrays completely break Nikon decryption for all encrypted sections (ShotInfo, LensData, ColorBalance). Historical issue: "4 engineering days chasing ghosts" from one incorrect byte in manual transcription. Arrays must be extracted with 100% accuracy.
- **Patching Thread Safety**: Build system uses per-file mutexes (`PATCH_MUTEXES: LazyLock<Mutex<HashMap<String, Arc<Mutex<()>>>>>`) to prevent concurrent patching of same ExifTool module. Required because extraction runs in parallel with rayon, but patching modifies shared ExifTool source files.
- **Array vs HashMap Performance**: Arrays provide direct index access `XLAT_0[index]` vs HashMap lookup `XLAT_MAP.get(&key)`. For cryptographic operations processing thousands of bytes, this difference matters. Arrays also use less memory (256 bytes vs HashMap overhead).
- **Build Pipeline Complexity**: Discovery phase scans config directories, extraction phase calls 13+ perl scripts in parallel, generation phase creates modular Rust code, integration phase updates module exports. Each phase has error handling, timing statistics, and atomic file operations.
- **ExifTool Module Scoping**: Many critical arrays are `my`-scoped (private) requiring patching to `our`-scoped (public) for extraction. Patching is idempotent and reversible via `git checkout` but must be coordinated across parallel extraction processes.

### Foundation Documents

- **Design docs**: [CODEGEN.md](../CODEGEN.md) - Complete codegen system overview, [ARCHITECTURE.md](../ARCHITECTURE.md) - System design principles
- **ExifTool source**: `third-party/exiftool/lib/Image/ExifTool/Nikon.pm` lines 13505-13538 (XLAT arrays), 13566-13567 (usage in Decrypt function), 13554-13588 (complete Decrypt function)
- **Start here**: `codegen/src/extractors/mod.rs` (trait system), `codegen/src/main.rs` (orchestration), `codegen/src/patching.rs` (patching system), `src/implementations/nikon/encryption.rs` lines 23-66 (manual arrays to replace)

### Prerequisites

- **Knowledge assumed**: Understanding of Rust trait systems, perl array vs hash differences, cryptographic XOR operations, parallel processing with mutexes, atomic file operations for build systems
- **Setup required**: Working ExifTool submodule, Rust toolchain with rayon for parallel processing, perl environment with ExifTool libraries, codegen build system functional

**Context Quality Check**: Can a new engineer understand WHY this approach is needed after reading this section?

## Work Completed

- ✅ **Codegen Architecture Research** → Analyzed trait-based extractor system, discovered 13 existing extractors follow consistent patterns with script_name, handles_config, requires_patching methods
- ✅ **ExifTool Source Analysis** → Located XLAT arrays in Nikon.pm:13505-13538, confirmed usage patterns in Decrypt function, verified 512 bytes of cryptographic constants requiring exact extraction
- ✅ **Patching System Study** → Understood thread-safe per-file mutex system, atomic file operations, regex-based `my` to `our` conversion, idempotent operation design
- ✅ **Generator Pipeline Investigation** → Examined lookup_tables module architecture, FileGenerator patterns, configuration processing, module integration system
- ✅ **Manual Array Risk Assessment** → Identified 512 hardcoded bytes in encryption.rs:23-66 as transcription error risk, confirmed single-byte errors break decryption completely

## Remaining Tasks

**REQUIRED**: Each task must be numbered, actionable, and include success criteria.

### 1. Task: Create simple_array.pl Perl Extractor

**Success Criteria**: Perl script successfully extracts XLAT arrays from patched Nikon.pm and outputs valid JSON with 256 elements per array index
**Approach**: Create perl script following simple_table.pl. The only difference is the output is an array instead of a hash. As the `simple_table` extraction pipeline, `simple_array` should be driven by configuration files as well, with something like this:

```json
    {
      "array_name": "%xlat[0]",
      "constant_name": "XLAT_0", 
      "key_type": "u8",
      "description": "First decoding array"
    },
    {
      "array_name": "%xlat[1]",
      "constant_name": "XLAT_1", 
      "key_type": "u8",
      "description": "Second decoding array"
    }
```

The perl patcher should patch 

**Dependencies**: None - can model after existing simple_table.pl

**Success Patterns**:

- ✅ Script accepts module path and array name as arguments: `perl simple_array.pl Nikon.pm xlat 0,1`
- ✅ Uses get_package_array() function to access `our @xlat` after patching
- ✅ Extracts specified indices (0,1) as separate JSON files: xlat_0.json, xlat_1.json
- ✅ Validates array length (256 elements expected) and data types (u8 bytes)
- ✅ Handles error cases gracefully: missing arrays, invalid indices, empty arrays
- ✅ Outputs progress to stderr and JSON data to stdout following existing patterns

### 2. Task: Implement SimpleArrayExtractor Rust Trait

**Success Criteria**: SimpleArrayExtractor integrates with extractor registry and processes simple_array.json configs correctly
**Approach**: Create Rust struct implementing Extractor trait with array-specific configuration handling
**Dependencies**: Task 1 (perl script exists)

**Success Patterns**:

- ✅ Implements all required Extractor trait methods: name(), script_name(), output_subdir(), handles_config()
- ✅ Returns "simple_array.pl" as script_name and "simple_arrays" as output_subdir
- ✅ Handles "simple_array" config type in handles_config() method
- ✅ Builds correct arguments for perl script: module_path, array_name, comma-separated indices
- ✅ Integrates into all_extractors() registry for auto-discovery
- ✅ Requires patching (returns true) for my-scoped array conversion

### 3. Task: Create Array Code Generator

**Success Criteria**: Generator produces static Rust arrays with lookup functions matching performance requirements
**Approach**: Add array generator to lookup_tables module following existing standard.rs patterns
**Dependencies**: Tasks 1 & 2 (extraction pipeline functional)

**Success Patterns**:

- ✅ Generates static arrays: `pub static XLAT_0: [u8; 256] = [0xc1, 0xbf, ...];`
- ✅ Creates bounds-checked lookup functions: `pub fn lookup_xlat_0(index: usize) -> Option<u8>`
- ✅ Creates direct access functions: `pub fn xlat_0(index: usize) -> u8` for guaranteed bounds
- ✅ Includes proper documentation with ExifTool source references
- ✅ Integrates with FileGenerator for consistent file structure and imports
- ✅ Supports multiple element types: u8, u16, i32, String as needed

### 4. Task: Add JSON Schema and Configuration Support

**Success Criteria**: simple_array.json configs validate correctly and integrate with discovery system  
**Approach**: Create schema file and extend config discovery to handle simple_array patterns
**Dependencies**: Tasks 1-3 (basic pipeline functional)

**Success Patterns**:

- ✅ Creates codegen/schemas/simple_array.json with array_name, indices, constant_names fields
- ✅ Validates configuration files during `make check-schemas` phase
- ✅ Extends discovery system in extraction.rs to recognize simple_array.json pattern
- ✅ Supports multiple arrays per config and multiple indices per array
- ✅ Integrates with parallel extraction pipeline and timing statistics

### 5. Task: Create Nikon XLAT Configuration and Integration

**Success Criteria**: XLAT arrays generate automatically and replace manual constants in encryption.rs
**Approach**: Create Nikon_pm/simple_array.json config and update encryption code to use generated arrays
**Dependencies**: Tasks 1-4 (complete pipeline functional)

**Success Patterns**:

- ✅ Creates config specifying xlat array with indices [0, 1] and constant names XLAT_0, XLAT_1
- ✅ Generates src/generated/Nikon_pm/xlat_0.rs and xlat_1.rs with 256-element arrays
- ✅ Updates encryption.rs to import and use generated arrays: `use crate::generated::Nikon_pm::{xlat_0, xlat_1};`
- ✅ Removes manual XLAT constants (lines 23-66) and updates decrypt function calls
- ✅ Validates identical decryption behavior with existing test suite (all 71+ Nikon tests pass)
- ✅ Confirms byte-for-byte array accuracy through direct comparison with ExifTool source

**Task Quality Check**: Can another engineer pick up any task and complete it without asking clarifying questions?

## Integration Requirements

**CRITICAL**: Building without integrating is failure. Don't accept tasks that build "shelf-ware."

Every feature must include:
- [x] **Activation**: simple_array extraction runs automatically during `make codegen` for all discovered configs
- [x] **Consumption**: Generated XLAT arrays actively used by Nikon decryption code instead of manual constants
- [x] **Measurement**: Can prove arrays are correct via test suite passage and byte-by-byte ExifTool comparison
- [x] **Cleanup**: Manual XLAT constants removed from encryption.rs, extraction pipeline handles future ExifTool updates automatically

**Red Flag Check**: If a task seems like "build array extractor but don't wire it anywhere," ask for clarity. We're not writing extractors to sit unused - everything must get us closer to "ExifTool in Rust for PhotoStructure."

## Working Definition of "Complete"

*Use these criteria to evaluate your own work - adapt to your specific context:*

A feature is complete when:
- ✅ **System behavior changes** - Nikon decryption uses generated arrays instead of manual constants
- ✅ **Default usage** - Array extraction happens automatically during codegen builds, not opt-in
- ✅ **Old path removed** - Manual XLAT arrays deleted, no fallback to hardcoded constants
- ❌ Code exists but isn't used *(example: "array extractor implemented but encryption.rs still uses manual arrays")*
- ❌ Feature works "if you call it directly" *(example: "simple_array.pl works but codegen doesn't call it")*

*Note: These are evaluation guidelines, not literal requirements for every task.*

## Prerequisites

None - all required infrastructure exists in codegen system.

## Testing

- **Unit**: Test perl array extraction with known XLAT data from ExifTool Nikon.pm source
- **Integration**: Verify generated arrays produce identical decryption results with existing Nikon test suite
- **Manual check**: Run `cargo t nikon` and confirm all 71+ tests pass with generated arrays

## Definition of Done

- [x] `cargo t nikon` passes with generated XLAT arrays replacing manual constants
- [x] `make precommit` clean including schema validation for simple_array configs  
- [x] XLAT arrays extracted automatically during `make codegen` with byte-perfect accuracy
- [x] Manual array constants removed from encryption.rs with no behavior changes

**✅ IMPLEMENTATION COMPLETED** - July 31, 2025

- **Simple array extraction pipeline**: Fully implemented with perl extractor, Rust trait integration, and code generator
- **XLAT arrays integrated**: 512 manual bytes in encryption.rs replaced with generated arrays from ExifTool source
- **Zero transcription errors**: Automated extraction eliminates manual maintenance burden and "4 engineering days chasing ghosts" risk
- **All tests passing**: 16/16 Nikon encryption tests pass including XLAT consistency validation
- **Codegen integration**: Arrays regenerate automatically during `make codegen` for future ExifTool updates

## Implementation Guidance

### Recommended Patterns

- **Perl Array Access**: Use `no strict 'refs'; my $array_ref = \@{$module_name . "::" . $array_name};` for symbolic reference access
- **Error Handling**: Validate array exists, has expected length, contains correct data types before extraction
- **Thread Safety**: Leverage existing per-file patching mutexes, no additional synchronization needed
- **Performance**: Generate direct array access functions for guaranteed bounds scenarios, Optional<T> functions for safety

### Tools to Leverage

- **Existing patching system**: Handles `my @xlat` to `our @xlat` conversion automatically  
- **FileGenerator struct**: Provides consistent file structure, imports, documentation generation
- **Trait system**: Follow SimpleTableExtractor patterns for consistency with existing extractors
- **JSON schema validation**: Use existing validation pipeline for configuration correctness

### ExifTool Translation Notes

- **Lines 13505-13538**: XLAT array definitions - translate exactly with no modifications or "improvements"
- **Lines 13566-13567**: Array usage patterns `$xlat[0][$index]` - maintain identical access semantics in Rust
- **Array scoping**: `my @xlat` requires patching but usage patterns remain identical after extraction

## Gotchas & Tribal Knowledge

### Known Edge Cases

1. **Array Bounds**: ExifTool uses `$xlat[0][$serial & 0xff]` ensuring index is always 0-255, but Rust needs explicit bounds checking
2. **Patching Dependencies**: Arrays must be patched before extraction, but patching affects shared ExifTool files requiring coordination
3. **Build Determinism**: Generated arrays must be identical across builds to avoid spurious git diffs - no timestamps or variable ordering
4. **Memory Layout**: Static arrays have different memory characteristics than HashMaps - benchmark if performance critical

### ExifTool Translation Challenges

- **Symbolic References**: Perl's `\@{$module_name . "::" . $array_name}` pattern requires careful implementation in extraction script
- **Index Syntax**: Perl's `@array[0]` vs Rust's `array[0]` - maintain semantic equivalence while respecting language differences
- **Error Handling**: ExifTool's liberal error handling vs Rust's strict approach - balance safety with functionality

### Performance Considerations

- **Array Access Speed**: Direct indexing `XLAT_0[index]` is faster than HashMap lookup for cryptographic operations
- **Memory Efficiency**: 256-byte static arrays vs HashMap overhead - significant for embedded/memory-constrained environments
- **Compilation Impact**: Large static arrays affect compile times - balance between performance and build speed

## Quick Debugging

Stuck? Try these:

1. `make check-schemas` - Validate simple_array configuration files
2. `cd codegen && RUST_LOG=debug cargo run -- config/Nikon_pm/simple_array.json` - Debug single config extraction
3. `exiftool -v3 test.nef` - Compare ExifTool's XLAT usage with our generated arrays
4. `xxd src/generated/Nikon_pm/xlat_0.rs` - Verify byte-by-byte array accuracy

---

## Summary

Simple array extraction pipeline addresses critical technical debt by eliminating manual transcription errors for cryptographic constants while establishing reusable infrastructure for future array-based extractions. Success metrics: automated XLAT generation, identical decryption behavior, zero manual maintenance with ExifTool updates.

**Estimated Effort**: 6-8 hours focused implementation following existing trait patterns vs ongoing maintenance burden and transcription error risk.