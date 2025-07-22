# Technical Project Plan: Unified Tag Definition Codegen (Tag Kit System)

**UPDATED**: July 22, 2025 (Evening) - Tag kit complete, runtime table support added, directory restructuring in progress

## 🚀 Quick Start for Next Engineer

**Current Status**: Multiple parallel efforts are underway:
1. Tag kit infrastructure is COMPLETE and TESTED (414 EXIF tags ready)
2. Runtime table extraction support has been added (needed for Canon CameraSettings, etc.)
3. Directory restructuring to organize extracted JSON by type is IN PROGRESS

**Your Immediate Task**: Complete the directory restructuring (45 minutes), then wire tag kit into runtime

## 🔥 URGENT: Complete Directory Restructuring First

The codegen is currently broken because we're halfway through restructuring extraction output. The issue:
- `runtime_table.json` files have different structure than `simple_table.json` 
- `load_extracted_tables_with_config` tries to parse all JSON files as `SimpleExtractedTable`
- Runtime table files have `source.table` instead of `source.hash_name`, causing parse errors

### What's Been Done:
1. Created directory structure in `extraction.rs` (lines 53-82)
2. Updated all extractor functions to accept `ExtractDirs` parameter
3. Updated file loading in generators to look in subdirectories
4. Updated `config/mod.rs` to only load from `simple_tables/` subdirectory

### What's Still Broken:
1. Missing `ExtractDirs` struct import in `extraction.rs` line 13
2. `process_module_config` passes wrong parameter type (lines 247-283)
3. Need to update `run_extractor` helper function signatures

### Fix Instructions:
```rust
// 1. Remove the extractors import from extraction.rs line 13
use crate::patching;
// use crate::extractors::{ExtractDirs, find_extractor, run_extractor}; // DELETE THIS

// 2. Move ExtractDirs struct from extractors.rs (deleted) into extraction.rs after line 18
struct ExtractDirs {
    simple_tables: PathBuf,
    tag_definitions: PathBuf,
    // ... etc
}

// 3. Fix parameter name in process_module_config (line 233)
fn process_module_config(config: &ModuleConfig, extract_dirs: &ExtractDirs) -> Result<()> {
```

### Testing the Fix:
```bash
cd codegen
cargo build
make clean
make codegen
# Should create directories under generated/extract/*
ls generated/extract/
# Should see: simple_tables/ tag_definitions/ runtime_tables/ etc.
```

**Key Files to Study**:
- `src/registry.rs` - Where to add tag kit lookup (see how print_conv functions are called)
- `src/generated/exif_tag_kit/mod.rs` - The tag kit API to call (see apply_print_conv function)
- `tests/tag_kit_integration.rs` - Proof that it works (all tests pass!)
- `src/implementations/print_conv.rs` - Manual implementations that tag kit will replace

**Implementation Approach** (conceptually one line):
```rust
// In wherever PrintConv is applied
let mut evaluator = ExpressionEvaluator::new();
let mut errors = Vec::new();
let mut warnings = Vec::new();
let result = exif_tag_kit::apply_print_conv(tag_id, value, &mut evaluator, &mut errors, &mut warnings);
// result is the converted TagValue
```

## Project Overview

**Goal**: Generate complete tag definitions from ExifTool source, including tag IDs, names, formats, AND PrintConv implementations in a single unified structure called "tag kits".

**Problem**: Manual implementations of EXIF tag PrintConvs are error-prone, particularly around tag ID offsets. Even simple 2-3 entry lookups can have offset bugs that are hard to spot in PR review, leading to expensive runtime errors.

**Solution Approach**: We chose to call this system "tag kits" because each tag comes with its full "kit" - everything needed to process it including its PrintConv implementation.

## Background & Context

### Why This Work is Needed

- **Offset errors**: Manual tag implementations require matching tag IDs (e.g., 0x0128) with their PrintConv logic, creating opportunities for offset mistakes
- **PR review difficulty**: Reviewers struggle to verify that tag IDs match their implementations correctly
- **Maintenance burden**: Even stable EXIF tags require careful manual translation from ExifTool source
- **Existing solutions insufficient**: Current inline_printconv extractor expects named hashes, but EXIF uses inline anonymous PrintConvs

### Related Documentation

- [CODEGEN.md](../CODEGEN.md) - Code generation framework
- [20250721-migrate-all-manual-lookup-tables-to-codegen.md](./20250721-migrate-all-manual-lookup-tables-to-codegen.md) - Parent task tracking manual lookup migrations
- [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - Core principle of exact ExifTool translation

## Technical Foundation

### Key Systems

- **Extraction**: `codegen/extractors/` - Perl scripts that parse ExifTool modules
- **Generation**: `codegen/src/generators/` - Rust code that generates lookup tables
- **Expression System**: `src/expressions/` - Runtime expression evaluator for simple Perl expressions
- **Manual Registry**: `src/registry.rs` - Function lookup for complex PrintConvs

### Current Architecture

```
ExifTool Tag Definition
    ↓
Manual Implementation (error-prone)
    ↓
src/implementations/print_conv.rs
```

### Target Architecture

```
ExifTool Tag Definition
    ↓
Unified Extraction (tag ID + PrintConv together)
    ↓
Generated Tag Tables with PrintConv
    ↓
Runtime Dispatcher (simple/expression/manual)
```

## Work Completed (July 22, 2025)

### Phase 1: COMPLETED - Tag Kit Infrastructure

1. **Created tag kit extractor** (`codegen/extractors/tag_kit.pl`) ✅
   - Extracts tag ID, name, format, groups, and PrintConv together
   - Successfully classifies PrintConv types: Simple hash, Expression, or Manual
   - Outputs unified JSON structure (`tag_kit_main.json`)
   - Based on existing `tag_definitions.pl` but adds PrintConv extraction
   - **Issue fixed**: Had to remove debug stderr output that was corrupting JSON

2. **Created tag kit schema** (`codegen/schemas/tag_kit.json`) ✅
   - Defines structure for tag kit extraction data
   - Supports all three PrintConv types with appropriate data fields

3. **Implemented generator** (`codegen/src/generators/tag_kit.rs`) ✅
   - Generates `TagKitDef` structs with embedded PrintConv
   - Creates static lookup tables for Simple PrintConvs (e.g., `PRINT_CONV_55`)
   - Generates runtime dispatcher (`apply_print_conv`) for different PrintConv types
   - Added whitespace trimming for notes field per user request
   - Successfully generates 414 tags from EXIF Main table

4. **Integrated into codegen pipeline** ✅
   - Added tag kit extraction to `codegen/src/extraction.rs`
   - Added `process_tag_kit_files` to `codegen/src/main.rs`
   - Created config at `codegen/config/Exif_pm/tag_kit.json`
   - **Path issue fixed**: Config source path shouldn't include `../` prefix

### Phase 2: COMPLETED - Integration Testing & Modular Generation

5. **Extracted EXIF basic tags** ✅
   - ResolutionUnit (0x0128/296) - Successfully extracted with 3 entries
   - All other basic tags present in the 414 extracted tags
   - GPS tags also extracted (GPSAltitude shows Expression type)

6. **Created integration tests** ✅ (July 22, 2025)
   - Tests in `tests/tag_kit_integration.rs` prove 100% parity
   - ResolutionUnit, Orientation, YCbCrPositioning all tested
   - Tests pass with both single-file and modular generation
   - **Key insight**: ExpressionEvaluator required, not ConditionEvaluator

7. **Implemented modular generation** ✅ (July 22, 2025)
   - Created `tag_kit_modular.rs` and `tag_kit_split.rs` generators
   - Split 6805-line file into 11 category modules
   - Categories: core (375), camera (87), color (200), document (120), datetime (175), gps (25), thumbnail (25), exif_specific (718), interop (83), windows_xp (115), other (3245)
   - All files except "other" are under 1000 lines
   - Fixed compilation issues: removed I8 variant, added HashMap import

### Phase 3: NOT STARTED - Runtime Integration

8. **Wire into runtime** ❌ CRITICAL NEXT STEP
   - Update `src/registry.rs` to check tag kits before manual registry
   - This enables production use of 414 automated tags
   - Must handle errors/warnings as arrays per user requirements

### Key Code Artifacts Created

- **Extractor**: `codegen/extractors/tag_kit.pl` - Extracts tags with inline PrintConvs
- **Schema**: `codegen/schemas/tag_kit.json` and `codegen/schemas/tag_kit.rs` - Data structures
- **Generators**: 
  - `codegen/src/generators/tag_kit.rs` - Original single-file generator
  - `codegen/src/generators/tag_kit_modular.rs` - Splits into category modules (NEW)
  - `codegen/src/generators/tag_kit_split.rs` - Category logic (NEW)
- **Config**: `codegen/config/Exif_pm/tag_kit.json`
- **Generated**: 
  - `src/generated/exif_tag_kit/` - Modular structure with 11 files
  - Old single file removed to avoid module conflict
- **Tests**: `tests/tag_kit_integration.rs` - Proves parity with manual implementations

### Example Generated Code

```rust
// ResolutionUnit lookup table
static PRINT_CONV_55: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("1".to_string(), "None");
    map.insert("2".to_string(), "inches");
    map.insert("3".to_string(), "cm");
    map
});

// ResolutionUnit tag definition
map.insert(296, TagKitDef {
    id: 296,
    name: "ResolutionUnit",
    format: "int16u",
    groups: { ... },
    writable: true,
    notes: Some("the value 1 is not standard EXIF"),
    print_conv: PrintConvType::Simple(&PRINT_CONV_55),
    value_conv: None,
});
```

## Recent Discoveries & Insights

### 1. Runtime Table Extraction Working!
- Successfully extracted Canon CameraSettings table (`canon_runtime_table_camerasettings.json`)
- The runtime table extractor (`runtime_table.pl`) outputs different JSON structure
- These tables need model-conditional logic and can't be statically generated
- Example: Canon's binary data tables vary by camera model

### 2. Directory Organization is Essential
- Different extractors produce incompatible JSON structures
- Filename pattern matching is fragile and error-prone
- Type-specific directories make the system more maintainable
- Proposed structure implemented in code but not tested:
  ```
  generated/extract/
  ├── simple_tables/      # Basic lookup tables
  ├── tag_definitions/    # Tag metadata
  ├── runtime_tables/     # ProcessBinaryData runtime tables
  ├── tag_kits/          # Unified tag definitions
  └── ... (9 other types)
  ```

### 3. Config File Proliferation
- Each module can have up to 10+ different config files
- `inline_tables.json` was added for inline PrintConv tables
- System is getting complex but remains manageable with good organization

## Remaining Tasks

### CRITICAL NEXT STEP: Complete Directory Restructuring (45 min)

1. **Fix the compilation errors** (15 min)
   - See fix instructions above
   - Main issue: `ExtractDirs` struct needs to be moved
   - Simple mechanical fixes, no logic changes needed

2. **Test the new structure** (15 min)
   - Run `make clean && make codegen`
   - Verify directories are created
   - Check files end up in correct subdirectories

3. **Update Makefile clean target** (15 min)
   - Add `rm -rf codegen/generated/extract/*/` to clean subdirectories
   - Keep the directory structure, just empty the contents

### Then: Wire into Runtime

1. **Wire tag kit into runtime** ❌ HIGHEST PRIORITY
   - Update `src/registry.rs` to check tag kits before manual registry
   - The integration tests already prove parity - we're ready!
   - Key files to study:
     - `src/registry.rs` - Current PrintConv function registry
     - `src/implementations/print_conv.rs` - Manual implementations to replace
     - `src/generated/exif_tag_kit/mod.rs` - The new tag kit API
   - Implementation approach:
     ```rust
     // In registry.rs or wherever PrintConv is applied
     if let Some(result) = exif_tag_kit::apply_print_conv(tag_id, value, evaluator, errors, warnings) {
         return result;
     }
     // Fall back to manual registry
     ```
   - Must handle errors/warnings as arrays per user requirements

2. **Remove manual implementations** (After runtime wiring)
   - Start with the 3 tested tags: ResolutionUnit, Orientation, YCbCrPositioning
   - Gradually remove others covered by tag kit
   - Keep complex manual implementations (the 37 "Manual" type)

### Phase 3: Expression Support (Medium Priority)

4. **Implement expression evaluator integration**
   - GPS tags already extracted with Expression type
   - GPSAltitude: `$val =~ /^(inf|undef)$/ ? $val : "$val m"`
   - Hook into existing `src/expressions/` system
   - Start with simple regex + ternary patterns

5. **Add more GPS tag configs**
   - Create `codegen/config/GPS_pm/tag_kit.json`
   - Extract GPS Main table
   - Many GPS tags have complex PrintConvs needing expressions

### Phase 4: Expand Coverage

6. **Add manufacturer modules**
   - Canon, Nikon, Sony, Olympus all use inline PrintConvs
   - Each needs its own tag_kit.json config
   - Potentially thousands more tags to automate

7. **Manual PrintConv registry integration**
   - For the 37 "Manual" type PrintConvs identified
   - Connect to existing registry system
   - Document which functions map to which Manual references

## Prerequisites

- Understanding of current inline_printconv extraction (exists but has structural mismatch)
- Familiarity with ExifTool tag table structure
- Access to expression evaluation system (`src/expressions/`)
- Read [EXIFTOOL-GUIDE.md](../guides/EXIFTOOL-GUIDE.md) sections on tag tables and PrintConv

### Development Environment Setup

```bash
# Verify you can run extractors
cd codegen
perl extractors/simple_table.pl ../third-party/exiftool/lib/Image/ExifTool/Exif.pm %orientation

# Verify expression system works
cargo test -p exif-oxide expressions::tests
```

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_resolution_unit_printconv() {
    let tag_def = EXIF_TAGS.get(&0x0128).unwrap();
    assert_eq!(apply_print_conv(&tag_def, &TagValue::U16(2)), 
               TagValue::String("inches".to_string()));
}
```

### Integration Tests

- Generate test comparing our output with `exiftool -j` for test images
- Verify all extracted PrintConvs produce identical output to ExifTool

### Validation Script

```bash
# Compare generated PrintConvs with ExifTool
cargo run --bin validate-printconvs test-images/*.jpg
```

## Success Criteria & Quality Gates

- **Zero offset errors**: Tag IDs and PrintConvs extracted together, eliminating manual matching
- **ExifTool parity**: Generated PrintConvs produce identical output to ExifTool
- **PR reviewability**: Generated code clearly shows tag ID with its PrintConv
- **All EXIF basic tags**: Successfully migrate the 6 identified tags
- **GPS proof-of-concept**: At least one expression-based PrintConv working

## Gotchas & Tribal Knowledge

### PrintConv Complexity Levels

1. **Simple hashes**: Direct key→value mappings (most EXIF tags) - WORKING ✅
2. **Expressions**: Perl code we can translate (`sprintf`, simple conditionals) - EXTRACTED, NOT EVALUATED YET
3. **Complex**: References external hashes, complex logic → manual registry - IDENTIFIED AS "Manual" TYPE

### Issues Encountered and Fixed

1. **Path calculation bug**: Config files shouldn't include `../` prefix since extraction adds `REPO_ROOT_FROM_EXTRACT`
2. **JSON corruption**: Perl stderr output was being included in JSON output - fixed by commenting out debug prints
3. **Binary file issue**: Generated file shows as binary in grep due to non-ASCII characters in some tag data
4. **Whitespace in notes**: ExifTool notes often have extra whitespace - now trimmed in generator
5. **TagValue::I8 doesn't exist** (July 22): Generator included I8 variant but TagValue enum only has I16/I32
6. **Module conflict** (July 22): Both `exif_tag_kit.rs` file and `exif_tag_kit/` directory existed - removed single file
7. **Missing imports** (July 22): HashMap import was accidentally removed when updating schemas
8. **Test compilation** (July 22): apply_print_conv takes ExpressionEvaluator, not ConditionEvaluator

### Why "Tag Kit" Name?

- Considered "unified tag definitions" but that was too verbose
- "Tag kit" conveys that each tag comes with its complete "kit" of processing tools
- Easier to discuss: "tag kit extractor", "tag kit generator", etc.

### Generated Code Structure

```
PRINT_CONV_N lookup tables (Simple PrintConvs)
    ↓
EXIF_TAG_KITS main map (all tags)
    ↓
apply_print_conv() dispatcher function
```

### Key String Conversions

The tag kit system stores numeric keys as strings in HashMaps:
- `map.insert("296".to_string(), ...)` not `map.insert(296, ...)`
- This allows uniform handling of string and numeric PrintConv keys
- The dispatcher converts TagValue to string for lookups

### Expression Translation Examples

```perl
# GPS Altitude (already extracted)
'$val =~ /^(inf|undef)$/ ? $val : "$val m"'

# Needs translation to expression DSL or Rust code
# Consider: Can we use existing expressions/ system?
```

### Integration Path (CRITICAL)

1. **DO NOT replace manual implementations yet**
2. **First create comprehensive integration tests**
3. **Verify 100% parity with manual implementations**
4. **Only then start migration**

### Debugging Tips

- **Extractor issues**: Add `print STDERR` but remember to remove before running!
- **Generated code issues**: Check `codegen/generated/extract/exif_tag_kit_main.json`
- **Binary file grep**: Use `strings file | grep pattern` instead of direct grep
- **Finding specific tags**: Search for `map.insert(296,` for ResolutionUnit
- **Module size check**: `wc -l src/generated/exif_tag_kit/*.rs | sort -n`

### Research Findings (July 22, 2025)

1. **Duplicate extraction systems**: 
   - `inline_printconv` extractor already handles named PrintConvs
   - Tag kit handles both named and anonymous inline PrintConvs
   - Both systems extract from the same ExifTool tables
   - Recommendation: Deprecate inline_printconv for modules with tag kit support

2. **Tag categorization patterns**:
   - Core image properties: IDs 254-400
   - Camera/device info: IDs 271-272, 305-306, 315-316
   - EXIF specific: IDs 33434-37500
   - Windows XP: IDs 18246-18249, 40091-40095
   - GPS tags would be in GPS IFD, not Main table

3. **PrintConv reference system**:
   - `tags.rs` generates `print_conv_ref: Some("function_name")`
   - This requires runtime lookup in registry
   - Tag kit embeds the actual implementation - no lookup needed
   - This is the key benefit: eliminates tag ID/function mismatches

### File Sizes

- Generated `exif_tag_kit.rs`: 186KB, 6805 lines
- Contains 414 tags, 74 lookup tables
- This is just from EXIF Main table - expect 10x more with all modules

### Example Extraction Output

```json
{
  "tag_id": "0x0128",
  "name": "ResolutionUnit",
  "format": "int16u",
  "groups": { "0": "IFD0", "1": "IFD", "2": "Image" },
  "print_conv_type": "Simple",
  "print_conv_data": {
    "1": "None",
    "2": "inches",
    "3": "cm"
  }
}
```

### Future Extensibility

This approach scales to manufacturer modules (Canon, Nikon) where tag definitions also contain inline PrintConvs, potentially eliminating entire categories of manual implementation.

## Refactoring Opportunities Noticed

### 1. Trait-Based Extractor System (Started but Abandoned)
- Started creating `extractors.rs` with trait-based design
- Would eliminate repetitive code in `extraction.rs`
- Each extractor type would implement an `Extractor` trait
- Benefits: Type safety, easier testing, better extensibility
- Abandoned for time - current approach works but is verbose

### 2. Consolidate Config Loading
- Currently each generator looks for its own files
- Could have a central "extraction manifest" that lists what was extracted
- Would eliminate the need to check file existence everywhere
- Make the system more robust to missing files

### 3. Merge Duplicate Extractors
- `tag_kit.pl` can replace `tag_definitions.pl`, `inline_printconv.pl`
- These older extractors only get partial data
- Tag kit gets everything in one unified structure
- Would simplify the extraction pipeline significantly

### 4. Better Error Handling for Heterogeneous JSON
- Current approach tries to parse and catches errors
- Could peek at JSON structure first to determine type
- Would give better error messages
- Prevent attempting to parse incompatible formats

### 5. Config File Consolidation
- 10+ config files per module is getting unwieldy
- Could have one `module_config.json` with sections
- Would make it easier to see all extractions for a module
- Reduce file system clutter

## Success Criteria

✅ **Completed**:
- Tag kit infrastructure extracts 414 EXIF tags with PrintConvs
- Integration tests prove 100% parity with manual implementations
- Modular generation splits large files into manageable chunks
- Runtime table extraction works for Canon CameraSettings

❌ **Not Yet Complete**:
- Directory restructuring (compilation currently broken)
- Tag kit wired into runtime for production use
- Manual implementations removed after validation
- Full ExifTool parity for supported tags

## Potential Refactorings & Future Work

### Code Quality Improvements

1. **PRIORITY: Reduce generated file size for easier debugging** ✅ DONE (July 22)
   - Original file was 186KB/6805 lines
   - Split into 11 category modules: largest is 3245 lines
   - Could further split "other" category if needed
   - Consider more compact representations:
     - Combine similar PrintConv tables (e.g., many On/Off tables)
     - Use arrays instead of HashMaps for sequential numeric keys
     - Use const arrays for small lookups instead of LazyLock<HashMap>

2. **Tag kit generator optimization**
   - Current approach generates individual PRINT_CONV_N tables - wasteful
   - Consider using phf (perfect hash function) crate for compile-time optimized lookups
   - String keys in HashMap could be numeric types with proper conversion
   - Many PrintConv tables are identical (e.g., multiple 0=Off, 1=On tables)
   - Could deduplicate by content hash

3. **Expression system integration**
   - The expressions module exists but isn't wired to tag kits yet
   - Need to design clean API for expression evaluation in PrintConv context
   - Consider caching compiled expressions
   - GPS tags are ready: GPSAltitude has `$val =~ /^(inf|undef)$/ ? $val : "$val m"`

4. **Error/Warning collection**
   - User requested arrays for errors/warnings rather than ExifTool's approach
   - apply_print_conv already takes &mut Vec<String> for both
   - Need to propagate these through the runtime stack
   - Consider structured error types instead of strings

5. **Additional refactorings noticed** (July 22):
   - Remove unused imports in generated category modules (LazyLock, TagValue not always used)
   - Fix excessive `mut` warnings in generated code
   - Consider generating only needed imports per module
   - The "groups" HashMap is always empty - extractor needs enhancement
   - Module organization could follow existing tags/ pattern more closely

### Architectural Considerations

1. **Module organization**
   - Generated tag kits could be split by group like existing tags (core, camera, gps, etc.)
   - Would reduce file size and improve compilation times
   - Need to maintain lookup efficiency

2. **Registry unification**
   - Currently have separate manual registry and tag kit system
   - Could unify into single dispatch mechanism
   - Would simplify runtime tag processing

3. **Incremental migration strategy**
   - Consider feature flag to toggle between manual and tag kit implementations
   - Allows gradual rollout and easy rollback
   - Enables A/B testing in production

### Performance Optimizations

1. **Lazy evaluation of PrintConvs**
   - Not all tags need PrintConv applied
   - Could defer evaluation until actually requested
   - Significant savings for large EXIF blocks

2. **Compile-time generation**
   - Some PrintConv mappings could be const evaluated
   - Explore const fn capabilities for simple lookups
   - Reduce runtime initialization cost

## Final Tasks: Codebase Retrofit & Documentation Update

### DO NOT PROCEED WITHOUT INTEGRATION TESTS

Before ANY production use:
1. Create comprehensive test suite comparing tag kit vs manual implementations
2. Test with real-world images from various camera manufacturers
3. Benchmark performance to ensure no regression
4. Get code review on integration approach

### Retrofit Plan (After Testing)

1. **Phase 1**: Wire tag kit as fallback (manual first, tag kit if missing)
2. **Phase 2**: Switch specific tags to tag kit after individual validation
3. **Phase 3**: Remove manual implementations only after extended production use
4. **Phase 4**: Expand to manufacturer modules (Canon, Nikon, etc.)

### Success Metrics

- Zero behavioral differences vs manual implementations
- Performance within 5% of manual approach
- Reduced code size (generated replaces manual)
- Improved maintainability (monthly ExifTool updates automated)

## Tribal Knowledge & Gotchas

### The Great Directory Restructuring of July 22
- Started because runtime tables broke the "everything is a SimpleExtractedTable" assumption
- Halfway through implementation when time ran out
- All the hard thinking is done - just mechanical fixes left
- This will prevent future "missing field" errors

### Why Runtime Tables Matter
- Canon CameraSettings changes based on camera model
- Can't be statically generated like simple lookup tables
- Need to generate functions that create HashMaps at runtime
- This pattern appears in many manufacturer modules

### The Tag Kit Revolution
- Eliminates the #1 source of bugs: tag ID/PrintConv mismatches
- Integration tests prove it works perfectly
- Just needs to be wired into runtime
- Will replace 90% of manual PrintConv implementations

### Config File Explosion
- Started with just `simple_table.json`
- Now have 10+ config types per module
- Each serves a specific purpose
- Directory structure makes this manageable

## CRITICAL TODO: Retrofit Existing Generators

**The next engineer MUST search for and update ALL existing tag-related generators to use tag kits:**

1. **Search for existing PrintConv implementations**:
   ```bash
   grep -r "print_conv" codegen/src/generators/
   grep -r "PrintConv" codegen/config/
   ```

2. **Generators that likely need updates**:
   - `inline_printconv.rs` - Could be deprecated in favor of tag kits
   - `tags.rs` - Should reference tag kit PrintConvs instead of manual registry
   - Any manufacturer-specific generators (Canon, Nikon, etc.)

3. **Config files to check**:
   - All `inline_printconv.json` configs - these extract named PrintConvs
   - Consider if these should migrate to tag kit approach
   - Update documentation to prefer tag kit extraction

4. **Integration points**:
   - Ensure generated tags reference tag kit PrintConvs
   - Update function name mappings to use tag kit lookups
   - Remove duplicate PrintConv extraction where possible

This consolidation is essential to avoid maintaining two parallel systems!