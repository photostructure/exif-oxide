# Universal RAW Format Codegen Extractors Implementation and Migration Plan

## 📊 Executive Summary

This milestone implements **universal codegen extractors** that **eliminate 1000+ lines of manual maintenance** across all RAW format implementations. **Phase 1 & 2 are complete and proven** - the Tag Table Structure Extractor successfully replaced manual code across Canon, Olympus, and Nikon with more accurate, comprehensive ExifTool-derived implementations.

**Current Status**: ✅ **ProcessBinaryData Extractor COMPLETE** - FujiFilm implementation working and validated

## 🎯 For the Next Engineer

### What You're Building
You're implementing the **2 remaining universal extractors** to complete the milestone:

1. **ProcessBinaryData Table Extractor** - ✅ **COMPLETE** - Working implementation proven with FujiFilm FFMV
2. **Model Detection Pattern Extractor** - Automates camera model detection logic  
3. **Conditional Tag Definition Extractor** - Automates complex conditional tag mappings

### Why This Matters
- **Monthly ExifTool releases** add new cameras, lenses, and bug fixes
- **Manual porting** of these changes is unsustainable (1000+ lines to maintain)
- **Generated code** updates automatically with zero maintenance burden
- **ExifTool accuracy** prevents the mapping errors found in manual implementations

## 📊 Current Status (Last Updated: 2025-07-19)

### ✅ COMPLETED: Phases 1 & 2 - Universal Pattern Proven
- **Tag Table Structure Extractor**: ✅ Complete and universal
- **Canon**: 84 generated variants (vs 24 manual), 215+ lines eliminated
- **Olympus**: 119 generated variants, ~15 lines eliminated, tests updated
- **Nikon**: 111 generated variants available for future use
- **Pattern Validation**: ✅ Works identically across all manufacturers
- **Build Status**: ✅ All tests passing, compilation clean

### ✅ COMPLETED: ProcessBinaryData Table Extractor (2025-07-19)

**🎉 IMPLEMENTATION SUCCESS**: ProcessBinaryData extractor is fully working and validated with FujiFilm FFMV table.

**Final Implementation Status**:
- **Perl Extractor**: ✅ Complete (`codegen/extractors/process_binary_data.pl`)
- **Rust Generator**: ✅ Complete (`codegen/src/generators/process_binary_data.rs`)
- **Build Integration**: ✅ Complete (added to extraction.rs, lookup_tables/mod.rs, generators/mod.rs)
- **Config Discovery**: ✅ Complete (added "process_binary_data.json" to supported config files)
- **Test Configuration**: ✅ Complete (`codegen/config/FujiFilm_pm/process_binary_data.json`)
- **Build System Testing**: ✅ Complete (`cargo run --release` successful)
- **Generated Code**: ✅ Complete (`src/generated/FujiFilm_pm/ffmv_binary_data.rs`)
- **Validation**: ✅ Complete (compiles without errors, generates clean API)

**Generated Output Example**:
- **Extracted JSON**: `fujifilm_binary_data.json` with complete table metadata
- **Generated Rust**: Type-safe `FujiFilmFFMVTable` with HashMap lookups
- **API Methods**: `get_tag_name()`, `get_format()`, `get_offsets()`

### 📋 REMAINING: Next Extractors (High Impact)
1. **Model Detection Pattern Extractor** - Medium complexity, high value  
2. **Conditional Tag Definition Extractor** - High complexity, highest value

## 🛠️ Essential Background for Next Engineer

### 🎯 Your Mission: Complete the Final 2 Extractors

You need to implement **Model Detection Pattern Extractor** and **Conditional Tag Definition Extractor** to complete MILESTONE-17. The ProcessBinaryData foundation is complete and working - use it as your template.

### 🏗️ ProcessBinaryData Implementation (Your Template)

**Files Created by Previous Engineer** (study these as your implementation template):
- **Perl Extractor**: `codegen/extractors/process_binary_data.pl` - Clean, well-documented pattern
- **Rust Generator**: `codegen/src/generators/process_binary_data.rs` - Type-safe code generation
- **Build Integration**: Changes in `extraction.rs:35,113,152,210,237,338`, `lookup_tables/mod.rs:125-153,363-383`, `generators/mod.rs:9,23`
- **Test Config**: `codegen/config/FujiFilm_pm/process_binary_data.json` - Working configuration example
- **Generated Output**: `src/generated/FujiFilm_pm/ffmv_binary_data.rs` - Clean Rust API

**Validation Proof**:
```bash
cd codegen && cargo run --release  # ✅ Works 
cd .. && cargo check --quiet       # ✅ Compiles
```

**Generated API Example**:
```rust
let table = FujiFilmFFMVTable::new();
table.get_tag_name(0);     // → "MovieStreamName"
table.get_format(0);       // → "string[34]"
table.get_offsets();       // → [0]
```

### Critical Documents to Study
1. **[TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md)** - ⚠️ CRITICAL: We translate ExifTool exactly, never "improve"
2. **[EXIFTOOL-INTEGRATION.md](../design/EXIFTOOL-INTEGRATION.md)** - Complete codegen architecture and patterns
3. **[ENGINEER-GUIDE.md](../ENGINEER-GUIDE.md)** - Development workflow and best practices
4. **[20250719-enhanced-codegen-configuration-analysis.md](20250719-enhanced-codegen-configuration-analysis.md)** - Manufacturer complexity analysis and implementation strategy

### 🚀 Next Extractor 1: Model Detection Pattern Extractor

**What This Extracts**: Camera model-specific conditions from ExifTool Main tables
**Target Patterns**:
```perl
# Canon.pm example
0xc => [
    { Condition => '$$self{Model} =~ /EOS D30\\b/', Name => 'ImageType' },
    { Condition => '$$self{Model} =~ /EOS-1D/', Name => 'FirmwareVersion' },
    # Multiple model-specific interpretations for same tag ID
]
```

**Implementation Strategy**:
1. **Start Simple**: FujiFilm (8 conditional entries) → test basic model regex extraction
2. **Add Complexity**: Canon (complex conditional arrays) → handle multiple conditions per tag ID
3. **Pattern**: Follow ProcessBinaryData implementation exactly - same files, same integration points

**Files to Create**:
- `codegen/extractors/model_detection.pl` - Extract model conditions from Main tables
- `codegen/src/generators/model_detection.rs` - Generate model matching logic
- `codegen/config/FujiFilm_pm/model_detection.json` - Start with simplest manufacturer
- Add to `extraction.rs`, `lookup_tables/mod.rs`, `generators/mod.rs` (follow ProcessBinaryData pattern)

### 🎯 Next Extractor 2: Conditional Tag Definition Extractor  

**What This Extracts**: Complex conditional tag arrays with count-based and binary pattern conditions
**Target Patterns**:
```perl
# Canon.pm examples
{ Condition => '$count == 582', Name => 'ColorData1' },
{ Condition => '$count == 653', Name => 'ColorData2' },
{ Condition => '$$valPt =~ /^\\0/ and $$valPt !~ /^(\\0\\0\\0\\0|\\x00\\x40\\xdc\\x05)/', Name => 'VignettingCorrUnknown1' },
```

**Implementation Strategy**:
1. **Start with Count Conditions**: Simplest type - just numeric comparisons
2. **Add Binary Pattern Matching**: Regex patterns on raw binary data  
3. **Handle Cross-Tag Dependencies**: DataMember → RawConv → ValueConv chains

**Files to Create**:
- `codegen/extractors/conditional_tags.pl` - Extract conditional tag definitions
- `codegen/src/generators/conditional_tags.rs` - Generate conditional processing logic
- `codegen/config/Canon_pm/conditional_tags.json` - Canon has most examples
- Add to same integration points as ProcessBinaryData

### 🔧 Implementation Template (Copy ProcessBinaryData Pattern)

**Step 1: Create Perl Extractor** (copy `process_binary_data.pl`):
- Same argument structure: `<module_path> <table_name>`
- Same JSON output format
- Same error handling and validation

**Step 2: Create Rust Generator** (copy `process_binary_data.rs`):
- Same serde data structures pattern
- Same HashMap + LazyLock generation
- Same clippy-compliant output

**Step 3: Integrate Build System** (copy ProcessBinaryData changes):
- Add to `SpecialExtractor` enum in `extraction.rs:35`
- Add to config_files array in `extraction.rs:113` 
- Add dispatch in `extraction.rs:237`
- Add extractor function (copy `run_process_binary_data_extractor`)
- Add to `needs_special_extractor_by_name` in `extraction.rs:338`
- Add to `lookup_tables/mod.rs` (copy ProcessBinaryData handling)
- Add to `generators/mod.rs` exports

**Step 4: Test & Validate**:
```bash
cd codegen && cargo run --release  # Must work
cd .. && cargo check --quiet       # Must compile
```

### ProcessBinaryData Implementation (95% Complete)

**Files Created/Modified by Last Engineer**:
- **Perl Extractor**: `codegen/extractors/process_binary_data.pl` - Extracts ProcessBinaryData tables to JSON
- **Rust Generator**: `codegen/src/generators/process_binary_data.rs` - Generates Rust table structures
- **Integration Points**: Modified `codegen/src/extraction.rs` (lines 113, 151, 209, 236-238, 337, 540-559)
- **Generation Logic**: Modified `codegen/src/generators/lookup_tables/mod.rs` (lines 125-153, 363-383)
- **Config Example**: `codegen/config/Canon_pm/process_binary_data.json` - Canon SensorInfo test case

**Critical Bug Fixed**: Config discovery was failing because "process_binary_data.json" wasn't in the supported config files list in `extraction.rs:104-113`.

### Key ExifTool Documentation
- **[PROCESS_PROC.md](../third-party/exiftool/doc/concepts/PROCESS_PROC.md)** - How ProcessBinaryData works
- **[PATTERNS.md](../third-party/exiftool/doc/concepts/PATTERNS.md)** - Common patterns across modules
- **[MODULE_OVERVIEW.md](../third-party/exiftool/doc/concepts/MODULE_OVERVIEW.md)** - ExifTool module structure

### Working Code to Study

#### ProcessBinaryData Extractor (95% Complete - Your Starting Point)
- **Extractor**: `codegen/extractors/process_binary_data.pl` - **WORKING** - Test with: `perl process_binary_data.pl ../third-party/exiftool/lib/Image/ExifTool/Canon.pm SensorInfo`
- **Generator**: `codegen/src/generators/process_binary_data.rs` - **COMPLETE** - Generates table structs with HashMap lookups
- **Config Example**: `codegen/config/Canon_pm/process_binary_data.json` - **READY FOR TESTING**
- **Integration**: **COMPLETE** - Added to extraction.rs, lookup_tables/mod.rs, generators/mod.rs

#### Tag Table Structure Extractor (Your Foundation Pattern)
- **Extractor**: `codegen/extractors/tag_table_structure.pl` - Proven working pattern
- **Generator**: `codegen/src/generators/tag_structure.rs` - Generates Rust enums with clippy compliance  
- **Config Examples**: 
  - `codegen/config/Canon_pm/tag_table_structure.json` - Working configuration
  - `codegen/config/Olympus_pm/tag_table_structure.json` - Universal validation
  - `codegen/config/Nikon_pm/tag_table_structure.json` - Universal validation
- **Integration**: `codegen/src/extraction.rs:34` (SpecialExtractor enum) & `codegen/src/extraction.rs:329` (dispatch)

#### Generated Code Examples
- **Canon**: `src/generated/Canon_pm/tag_structure.rs` - 84 variants with full metadata
- **Olympus**: `src/generated/Olympus_pm/tag_structure.rs` - 119 variants with full metadata
- **Nikon**: `src/generated/Nikon_pm/tag_structure.rs` - 111 variants with full metadata

#### Manual Code Replaced (Study These Changes)
- **Canon**: `src/raw/formats/canon.rs` - Import replaced 215+ lines of manual enum
- **Olympus**: `src/raw/formats/olympus.rs:27,42-55,69-94` - HashMap→array conversion with generated enum

## 🔧 What Was Accomplished (Phase 1 & 2 Success)

### Phase 1: Canon Validation
- **Generated vs Manual**: 84 generated variants vs 24 manual variants (3.5x improvement)
- **Accuracy Improvement**: Fixed 0x0003 mapping error (FlashInfo vs ShotInfo)
- **Code Elimination**: 215+ lines removed from `src/raw/formats/canon.rs:368-583`
- **Test Updates**: Updated test cases to match ExifTool's correct mappings

### Phase 2: Universal Pattern Validation
- **Olympus Integration**: Created config, generated 119 tags, replaced HashMap with array
- **Test Modernization**: Updated `test_section_mapping` to use generated enum methods
- **Nikon Preparation**: Created config, generated 111 tags (ready for future use)
- **Compilation Clean**: Fixed test compilation errors, all tests passing

### Key Technical Discoveries
1. **Manual implementations often have errors** - Found incorrect tag ID mappings in Canon code
2. **ExifTool is the source of truth** - Generated code found 60+ additional tags missed by manual code
3. **Testing is critical** - Had to update test cases to match ExifTool's correct mappings
4. **Array vs HashMap**: Array-based approach is more type-safe and performant than HashMap

## 🔍 Issues You'll Need to Address

### Critical Requirements
1. **Trust ExifTool completely** - Never "improve" or "optimize" ExifTool logic
2. **Handle Perl complexity** - ProcessBinaryData has complex format strings and conditionals
3. **Maintain type safety** - Generated Rust code must be compile-time safe
4. **Test thoroughly** - Generated code often exposes errors in manual implementations

### Common Pitfalls to Avoid
1. **Don't trust manual implementations** - They often have errors vs ExifTool
2. **Always validate against ExifTool source** - Use `third-party/exiftool/lib/Image/ExifTool/*.pm`
3. **Test case updates required** - Generated code may expose manual test errors
4. **Clippy compliance matters** - Use modern Rust patterns like `matches!` macro

## 📋 Immediate Next Steps (ProcessBinaryData 95% Complete)

### ⚠️ PRIORITY 1: Complete ProcessBinaryData Testing

**Last Engineer Status**: Fixed critical config discovery bug. Now ready for final validation.

**What to Do Now**:
1. **Run full build test**: `cd codegen && cargo run --release`
2. **Look for**: "📷 Processing process_binary_data tables..." in output
3. **Verify**: `canon_binary_data.json` file created in `generated/extract/`
4. **Check**: Generated Rust code in `src/generated/Canon_pm/sensorinfo_binary_data.rs`
5. **Validate**: No compilation errors, imports working

**If Build Fails**: Check these integration points the last engineer added:
- `extraction.rs:113` - "process_binary_data.json" in supported config files
- `extraction.rs:151` - "process_binary_data.json" in table field parsing logic  
- `extraction.rs:209` - "process_binary_data" in patching skip list
- `extraction.rs:337` - "process_binary_data" -> ProcessBinaryData mapping
- `lookup_tables/mod.rs:125-153` - ProcessBinaryData generation logic

### Pattern Already Extracted (Reference)
```perl
# Canon.pm SensorInfo table (test case)
%Image::ExifTool::Canon::SensorInfo = (
    PROCESS_PROC => \&Image::ExifTool::ProcessBinaryData,
    FORMAT => 'int16s',
    FIRST_ENTRY => 1,
    GROUPS => { 0 => 'MakerNotes', 2 => 'Image' },
    1 => 'SensorWidth',
    2 => 'SensorHeight',
    # ... 10 total tags
);
```

### After ProcessBinaryData is Complete: Next Extractors

#### Option 1: Model Detection Pattern Extractor (Next Priority)
**Why This**: Medium complexity, clear boundaries, good learning experience

**Study These Files**:
- Search for model-specific conditionals in ExifTool Main tables
- Look for patterns like `$format{MODEL_NAME}` in tag definitions

#### Option 2: Conditional Tag Definition Extractor
**Why Last**: Most complex, requires understanding other extractors first

### Implementation Steps Template (For Future Extractors)

1. **Create Extractor Script**: `codegen/extractors/your_extractor.pl`
   - Study `tag_table_structure.pl` as template
   - Use explicit argument passing (file path + specific targets)
   - Output structured JSON

2. **Create Generator**: `codegen/src/generators/your_generator.rs`
   - Study `tag_structure.rs` as template
   - Generate direct Rust code (no macros)
   - Ensure clippy compliance

3. **Add Configuration Support**: `codegen/config/ModuleName_pm/your_extractor.json`
   - Use same pattern as existing configs
   - Explicit source paths, no guessing

4. **Integrate with Build System**: 
   - Add to `codegen/src/extraction.rs` SpecialExtractor enum
   - Add dispatch logic
   - Test with `make codegen`

5. **Replace Manual Code**:
   - Find manual implementations to replace
   - Update imports to use generated code
   - Update tests to match ExifTool accuracy

6. **Validate and Test**:
   - Run `make precommit`
   - Compare output with ExifTool reference
   - Update any failing tests

## 🧪 Testing Strategy

### Development Cycle
```bash
make codegen              # Regenerate all code  
cargo check              # Verify compilation
cargo test               # Run unit tests
make precommit           # Full validation pipeline
```

### Validation Against ExifTool
```bash
# Test with real images (if available)
cargo run -- test-images/canon/*.CR2 --debug
exiftool -j test-images/canon/*.CR2 > expected.json
# Compare outputs for accuracy
```

### Success Criteria Checklist
- [ ] Extractor generates valid JSON from ExifTool source
- [ ] Generator produces valid, clippy-compliant Rust code
- [ ] Generated code compiles without warnings
- [ ] Manual code successfully replaced
- [ ] All tests pass
- [ ] Output matches ExifTool reference behavior

## 🔍 Debugging the ProcessBinaryData Implementation

### Known Issues from Last Engineer
1. **Unused Import Warning**: `process_binary_data::generate_process_binary_data` - This should disappear once generation works
2. **Config Discovery Bug**: Fixed - "process_binary_data.json" not in supported files list
3. **Manual Testing Works**: Direct perl script execution produces correct JSON
4. **Build Integration**: All components added but not yet tested end-to-end

### Troubleshooting Guide
**If config not discovered**: Check `extraction.rs:104-113` for "process_binary_data.json" in config_files array
**If extraction not called**: Check `extraction.rs:214` dispatch logic and `needs_special_extractor_by_name()`
**If generation fails**: Check `lookup_tables/mod.rs:125-153` ProcessBinaryData handling
**If compilation fails**: Check imports in `generators/mod.rs:23` and usage in `lookup_tables/mod.rs:141-142`

## 🔮 Future Refactoring Considerations (Post-ProcessBinaryData)

### High-Value Improvements Identified
1. **Generator Base Classes**: Extract common patterns from tag_structure.rs and process_binary_data.rs for reuse
2. **Error Standardization**: Unified error types across all extractors with context
3. **Config Discovery**: Replace hardcoded file list with pattern matching (*.json)
4. **Testing Infrastructure**: Automated comparison against ExifTool reference output

### Code Organization Improvements Needed
1. **Module Splitting**: Break large generators into focused, testable components
2. **Utility Libraries**: Common Perl extraction utilities in ExifToolExtract.pm
3. **Type Safety**: Stronger typing in JSON intermediate format with serde validation
4. **Integration Simplification**: Reduce boilerplate when adding new extractor types

### Performance Optimizations
1. **Parallel Extraction**: Run multiple extractors concurrently during build
2. **Incremental Generation**: Only regenerate changed configurations  
3. **Caching**: Cache ExifTool analysis results between builds

## 📊 Impact Metrics

### ✅ Proven Results (Phases 1 & 2 Complete)
- **Canon**: 215+ lines eliminated, 3.5x more comprehensive, mapping errors fixed
- **Olympus**: ~15 lines eliminated, type-safe array implementation
- **Nikon**: 111 tag structure ready for future use
- **Total Achieved**: 230+ lines eliminated with universal pattern proven
- **Maintenance**: Zero ongoing maintenance for tag definitions

### 🎯 Projected Impact (ProcessBinaryData + Remaining Extractors)
- **ProcessBinaryData**: ~400+ lines across manufacturers (95% complete)
- **Model Detection**: ~200+ lines across manufacturers  
- **Conditional Tags**: ~300+ lines across manufacturers
- **Total Remaining**: ~900+ lines elimination potential

### ⏱️ Development Time Impact
- **Manual Implementation**: 2-3 months per manufacturer
- **With Universal Extractors**: 1-2 weeks per manufacturer  
- **Monthly ExifTool Updates**: Hours → Minutes (fully automated)

## 🚀 Immediate Action Plan

### Your First Hour (ProcessBinaryData Completion)
- [ ] `cd codegen && cargo run --release` - Test the 95% complete implementation
- [ ] Look for "📷 Processing process_binary_data tables..." in output
- [ ] Check if `generated/extract/canon_binary_data.json` is created
- [ ] Verify generated Rust code appears in `src/generated/Canon_pm/`
- [ ] If issues: Check integration points in `extraction.rs` and `lookup_tables/mod.rs`

### Your First Day (If ProcessBinaryData Works)
- [ ] Add more ProcessBinaryData tables (Canon ColorData, Nikon LensData, etc.)
- [ ] Test with real images: `cargo run -- test-images/canon/*.CR2`
- [ ] Update manual implementations to use generated tables
- [ ] Run `make precommit` to ensure all tests pass

### Your First Week (Next Extractor)
- [ ] Study Model Detection patterns in ExifTool Main tables
- [ ] Create model_detection.pl extractor following ProcessBinaryData pattern
- [ ] Implement model_detection.rs generator
- [ ] Add to build system using established integration points

## 🔑 Key Success Factors

- **Trust ExifTool completely** - Don't "improve" anything
- **Test against real ExifTool output** - Generated code exposes manual errors
- **Clippy compliance matters** - Use modern Rust patterns
- **Atomic operations** - Build system handles ExifTool patching safely
- **Incremental progress** - Start with one manufacturer, expand gradually

**The foundation is solid. The pattern is proven. You have a clear roadmap to complete the remaining extractors and achieve full automation.**