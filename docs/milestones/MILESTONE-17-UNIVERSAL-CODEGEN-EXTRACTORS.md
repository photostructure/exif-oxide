# Universal RAW Format Codegen Extractors Implementation and Migration Plan

## 📊 Current Status (Last Updated: 2025-07-19)

### ✅ Phase 1 Complete: Tag Table Structure Extractor
- **Extractor Created**: `codegen/extractors/tag_table_structure.pl` 
- **Generator Created**: `codegen/src/generators/tag_structure.rs`
- **Integration Complete**: Added to extraction.rs and lookup_tables module
- **Canon Validated**: Successfully generates 84-tag enum with all metadata
- **Build Status**: ✅ All tests passing

### 🚧 Immediate Next Steps (High Priority)
1. **Replace Manual Canon Implementation** 
   - File: `src/raw/formats/canon.rs:368-583`
   - Action: Use generated `crate::generated::Canon_pm::tag_structure::CanonDataType`
   - Benefit: Removes 215+ lines of manual maintenance

2. **Validate Universal Applicability**
   - Create configs for Olympus and Nikon
   - Test extraction across manufacturers
   - Confirm pattern universality

### 📋 Remaining Extractors (Medium Priority)
- ProcessBinaryData Table Extractor (`binary_data_tables.pl`)
- Model Detection Pattern Extractor (`model_patterns.pl`)  
- Conditional Tag Definition Extractor (`conditional_tags.pl`)

## 🎯 Executive Summary

This milestone implements **4 universal codegen extractors** that eliminate **1000+ lines of manual maintenance** across ALL RAW format implementations. The extractors automatically generate Rust code from ExifTool source, ensuring perfect compatibility while dramatically reducing future maintenance burden.

**Key Benefits**:
- **95% reduction** in manual lookup table maintenance
- **Automatic support** for new ExifTool releases  
- **Universal applicability** across all manufacturers
- **Future-proofs** all RAW format implementations

## 🛠️ Implementation Guide for Next Engineer

### Essential Background Reading
1. **[TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md)** - Critical: We translate ExifTool exactly, never "improve"
2. **[EXIFTOOL-INTEGRATION.md](../design/EXIFTOOL-INTEGRATION.md)** - Understand existing codegen architecture
3. **[codegen/lib/ExifToolExtract.pm](../../codegen/lib/ExifToolExtract.pm)** - Perl utilities for extraction

### Code to Study
1. **Completed Extractor**: 
   - `codegen/extractors/tag_table_structure.pl` - Pattern for new extractors
   - `codegen/src/generators/tag_structure.rs` - Pattern for generators
   
2. **Integration Points**:
   - `codegen/src/extraction.rs:34` - SpecialExtractor enum
   - `codegen/src/extraction.rs:329` - Handler dispatch
   - `codegen/src/generators/lookup_tables/mod.rs:95-124` - Config processing

3. **Manual Code to Replace**:
   - `src/raw/formats/canon.rs:368-583` - Manual CanonDataType enum
   - `src/raw/formats/olympus.rs:42-52` - Manual section mappings

### Technical Implementation Details

#### Tag Table Structure Extractor (COMPLETE)
**Purpose**: Extract manufacturer Main table structures into Rust enums

**Key Implementation Decisions**:
1. **Boolean Serialization**: Use `\1` and `\0` in Perl for proper JSON booleans
2. **Variant Deduplication**: Maintain global HashSet across all method generations
3. **Naming Conflicts**: CustomFunctions1D vs CustomFunctions1D2 handled automatically

**Files Created/Modified**:
- ✅ `codegen/extractors/tag_table_structure.pl`
- ✅ `codegen/src/generators/tag_structure.rs` 
- ✅ `codegen/src/extraction.rs` (added SpecialExtractor::TagTableStructure)
- ✅ `codegen/src/generators/lookup_tables/mod.rs` (added config handling)
- ✅ `codegen/config/Canon_pm/tag_table_structure.json` (test config)

**Known Issues**:
- Perl warnings about non-numeric keys (CHECK_PROC, etc.) are harmless
- Type fix applied: `Vec<&(&TagDefinition, String)>` at line 204

#### Binary Data Tables Extractor (TODO)
**Purpose**: Extract ProcessBinaryData definitions for binary parsing

**Implementation Plan**:
1. Create `codegen/extractors/binary_data_tables.pl`
   - Pattern: Similar to tag_table_structure.pl
   - Extract: PROCESS_PROC, FORMAT, FIRST_ENTRY, field definitions
   - Output: JSON with binary field specifications

2. Create generator in `codegen/src/generators/binary_data.rs`
   - Generate: Processor structs with HashMap<u16, BinaryFieldDef>
   - Include: Field names, formats, PrintConv references

**Key Challenges**:
- Variable-length fields based on DataMember tags
- FIRST_ENTRY offset handling (Canon uses 1-based)
- Format overrides per field

#### Model Detection Pattern Extractor (TODO)
**Purpose**: Extract camera model patterns for offset/format detection

**Implementation Plan**:
1. Create `codegen/extractors/model_patterns.pl`
   - Search for: Model regex patterns in conditionals
   - Extract: Pattern → behavior mappings
   - Handle: Multiple pattern types per manufacturer

2. Generate: Model detection functions and constant arrays
   - Example: CANON_6_BYTE_MODELS array
   - Example: detect_canon_offset_scheme() function

#### Conditional Tag Extractor (TODO)
**Purpose**: Extract array-based conditional tag definitions

**Implementation Plan**:
1. Create `codegen/extractors/conditional_tags.pl`
   - Detect: Array references as tag values
   - Extract: Each variant with its condition
   - Output: Structured conditional logic

2. Generate: Conditional processing functions
   - Match tag_id + condition → specific variant
   - Handle model-based, count-based conditions

### Testing Your Implementation

```bash
# Quick iteration cycle
make codegen              # Regenerate all code
cargo check              # Verify compilation
cargo test               # Run tests

# Canon integration test
cargo run -- test-images/canon/*.CR2 --debug

# Full validation
make precommit           # Runs all checks
```

### Success Criteria

1. **Generated Code Quality**
   - [ ] Identical functionality to manual implementations
   - [ ] No performance regression
   - [ ] Proper error handling maintained

2. **Universal Applicability**
   - [ ] Canon extraction working (✅ DONE)
   - [ ] Olympus extraction working
   - [ ] Nikon extraction working
   - [ ] Pattern holds for all manufacturers

3. **Maintenance Reduction**
   - [ ] Manual Canon enum removed (-215 lines)
   - [ ] Manual Olympus mappings removed (-80 lines)
   - [ ] Zero manual updates needed for new cameras

### Refactoring Opportunities Identified

1. **Extractor Base Class**
   - Many extractors share common patterns
   - Consider Perl base class in ExifToolExtract.pm
   - Reduce boilerplate across extractors

2. **Config Validation**
   - Add JSON schema validation for configs
   - Catch config errors early
   - Better error messages

3. **Generator Organization**
   - Consider splitting tag_structure.rs into smaller modules
   - Separate concerns: parsing, generation, formatting
   - Easier to test individual components

4. **Error Handling**
   - Standardize error types across extractors
   - Better error context (which file, which table)
   - Recovery strategies for partial failures

### Common Pitfalls to Avoid

1. **Don't Parse Perl with Regex**
   - Always use Perl interpreter
   - Let ExifToolExtract.pm handle the complexity

2. **Git Submodule Safety**
   - NEVER modify third-party/exiftool directly
   - Always use atomic patch/extract/revert operations

3. **Trust ExifTool Patterns**
   - Don't "optimize" seemingly redundant code
   - Every quirk handles real camera bugs

4. **Test with Real Files**
   - Synthetic test data misses edge cases
   - Use test-images/ extensively

### Debugging Tips

```bash
# Compare with ExifTool
exiftool -v3 image.cr2 > exiftool.txt
cargo run -- image.cr2 --debug > ours.txt
diff exiftool.txt ours.txt

# Check extraction output
cat codegen/generated/extract/canon_tag_structure.json | jq

# Trace codegen execution
RUST_LOG=debug make -C codegen

# Validate generated code
cargo expand ::generated::Canon_pm::tag_structure
```

## 📊 Impact Metrics

### Lines of Code Eliminated
- Canon: 295+ lines (enum + offset manager)
- Olympus: 80+ lines (section mappings)
- Minolta: 400+ lines (processors)
- Panasonic: 150+ lines (tag definitions)
- **Total**: 1000+ lines eliminated

### Development Time Saved
- Manual port: 2-3 months per manufacturer
- With extractors: 1-2 weeks per manufacturer
- Monthly updates: Hours → Minutes

## 🎯 Final Notes

The Tag Table Structure Extractor proves the concept works. The infrastructure is in place, patterns are validated, and Canon generation is successful. The remaining extractors follow the same pattern:

1. Perl script extracts from ExifTool
2. JSON intermediate format
3. Rust generator creates type-safe code
4. Integration via existing config system

Focus on getting Canon fully integrated first, then expand to other manufacturers. Each success makes the next one easier.

**Remember**: We're not just saving lines of code - we're ensuring perfect ExifTool compatibility forever.