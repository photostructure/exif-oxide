# Complete SubDirectory Binary Data Parsers

## Project Overview

- **High-level goal**: Complete the implementation of subdirectory binary data parsers to properly extract individual tag values instead of raw byte arrays
- **Problem statement**: While subdirectory dispatcher functions now correctly call processor functions (fixed 2025-07-25), the actual binary data parsing implementations remain as TODOs, causing tags like ProcessingInfo and CanonShotInfo to display as numeric arrays instead of meaningful values

## MANDATORY READING

These are relevant, mandatory, prerequisite reading for every task:

- [@CLAUDE.md](../CLAUDE.md)
- [@docs/TRUST-EXIFTOOL.md](../docs/TRUST-EXIFTOOL.md).

## DO NOT BLINDLY FOLLOW THIS PLAN

Building the wrong thing (because you made an assumption or misunderstood something) is **much** more expensive than asking for guidance or clarity.

The authors tried their best, but also assume there will be aspects of this plan that may be odd, confusing, or unintuitive to you. Communication is hard!

**FIRSTLY**, follow and study **all** referenced source and documentation. Ultrathink, analyze, and critique the given overall TPP and the current task breakdown.

If anything doesn't make sense, or if there are alternatives that may be more optimal, ask clarifying questions. We all want to drive to the best solution and are delighted to help clarify issues and discuss alternatives. DON'T BE SHY!

## KEEP THIS UPDATED

This TPP is a living document. **MAKE UPDATES AS YOU WORK**. Be concise. Avoid lengthy prose!

**What to Update:**

- 🔍 **Discoveries**: Add findings with links to source code/docs (in relevant sections)
- 🤔 **Decisions**: Document WHY you chose approach A over B (in "Work Completed")
- ⚠️ **Surprises**: Note unexpected behavior or assumptions that were wrong (in "Gotchas")
- ✅ **Progress**: Move completed items from "Remaining Tasks" to "Work Completed"
- 🚧 **Blockers**: Add new prerequisites or dependencies you discover

**When to Update:**

- After each research session (even if you found nothing - document that!)
- When you realize the original approach won't work
- When you discover critical context not in the original TPP
- Before context switching to another task

**Keep the content tight**

- If there were code examples that are now implemented, replace the code with a link to the final source.
- If there is a lengthy discussion that resulted in failure or is now better encoded in source, summarize and link to the final source.
- Remember: the `ReadTool` doesn't love reading files longer than 500 lines, and that can cause dangerous omissions of context.

The Engineers of Tomorrow are interested in your discoveries, not just your final code!


## Background & Context

- **Why this work is needed**: Users expect to see human-readable tag values (e.g., "WB_RGGBLevelsAsShot: 2241 1024 1024 1689") not raw arrays like "[28, 0, 2, 0, 0, 0...]"
- **Related docs**:
  - `/home/mrm/src/exif-oxide/docs/done/20250124-tag-kit-subdirectory-support.md` - Initial subdirectory support implementation
  - `/home/mrm/src/exif-oxide/docs/reference/SUBDIRECTORY-COVERAGE.md` - Current coverage status
  - `/home/mrm/src/exif-oxide/docs/CODEGEN.md` - Code generation system documentation
  - `third-party/exiftool/doc/concepts/PROCESS_PROC.md` - How ExifTool processes binary data

## Technical Foundation

### Key Codebases
- **ExifTool source**: `third-party/exiftool/lib/Image/ExifTool/*.pm` - The canonical implementation we're translating
- **Generated parsers**: `src/generated/*/tag_kit/mod.rs` - Where dispatcher functions live
- **Binary data framework**: `src/binary_data.rs` - Infrastructure for parsing fixed-format binary data
- **Code generator**: `codegen/src/generators/tag_kit_modular.rs` - Generates the dispatcher functions

### Key Concepts
- **SubDirectory tags**: Tags that reference other tables for parsing their binary data
- **Binary data tables**: Fixed-format structures with tags at specific byte offsets
- **Cross-module references**: Subdirectory tables that exist in different ExifTool modules

### ExifTool Binary Data Format
```perl
# Example from Canon.pm ShotInfo table:
1 => { # byte offset 1
    Name => 'AutoISO',
    Format => 'int16u', # 2-byte unsigned integer
    PrintConv => { 0 => 'Off', 1 => 'On' },
},
```

## Work Completed

1. **Subdirectory dispatcher fix** (2025-07-25):
   - Fixed code generator bug where unconditional subdirectories generated empty match statements
   - Dispatcher functions now correctly call processor functions
   - Example: `process_tag_0x4_subdirectory` now calls `process_canon_shotinfo`

2. **ColorData extraction working**:
   - ColorData6 and other ColorData variants successfully extract individual tags
   - Example: `WB_RGGBLevelsAsShot: "2241 1024 1024 1689"` instead of array

3. **Stub functions added**:
   - Added temporary stubs for cross-module references to allow compilation
   - Affected modules: Canon, Exif, Olympus, PanasonicRaw, Sony

## Remaining Tasks

### Phase 1: Implement Canon Binary Parsers (High Confidence)

1. **process_canon_shotinfo** (`src/generated/Canon_pm/tag_kit/mod.rs`):
   ```rust
   // Current stub:
   pub fn process_canon_shotinfo(data: &[u8], byte_order: ByteOrder) -> Result<Vec<(String, TagValue)>> {
       // TODO: Implement binary data parsing for Canon ShotInfo
       Ok(vec![])
   }
   ```
   - **Source table**: `third-party/exiftool/lib/Image/ExifTool/Canon.pm` starting at line 2851
   - **Table attributes**:
     ```perl
     %Image::ExifTool::Canon::ShotInfo = (
         %binaryDataAttrs,
         FORMAT => 'int16s',      # Default format for all entries
         FIRST_ENTRY => 1,        # First tag at offset 1
         DATAMEMBER => [ 19 ],    # Tag 19 affects parsing of other tags
         GROUPS => { 0 => 'MakerNotes', 2 => 'Image' },
     ```
   - **Example tags**:
     - AutoISO (offset 1): `ValueConv => 'exp($val/32*log(2))*100'`
     - BaseISO (offset 3): Direct value
     - MeasuredEV (offset 5): `ValueConv => '$val / 32'`
     - TargetAperture (offset 6): Complex ExifTool::Canon::CanonEv conversion
   - **Implementation approach**:
     1. Check if we already have a ShotInfo binary data table extracted
     2. If not, add to Canon_pm process_binary_data.json config
     3. Use the generated binary data parser
     4. Handle the DATAMEMBER dependency for tag 19

2. **process_canon_processing** (`src/generated/Canon_pm/tag_kit/mod.rs`):
   - **Source table**: `third-party/exiftool/lib/Image/ExifTool/Canon.pm` starting at line 5087
   - **Table attributes**:
     ```perl
     %Image::ExifTool::Canon::Processing = (
         %binaryDataAttrs,
         FORMAT => 'int16s',      # Default format for all entries
         FIRST_ENTRY => 1,        # First tag at offset 1
         GROUPS => { 0 => 'MakerNotes', 2 => 'Image' },
     ```
   - **Example tags**:
     - ToneCurve (offset 1): PrintConv with Standard/Manual/Custom
     - Sharpness (offset 2): Direct value (unsharp mask strength)
     - SharpnessFrequency (offset 3): PrintConv with n1-n5, Standard, Low, High
     - ColorTone (offset 9): Direct value (-4 to +4)
   - **Special handling**: Some tags have negative offsets (e.g., offset -1 for RawBrightnessAdj)
   - **Implementation**: Similar to ShotInfo, check for existing extraction first

3. **process_canon_crwparam**:
   - **Source**: Check if CRWParam has a defined table or needs special handling
   - May be related to CRW (Canon Raw) format specifics

### Phase 2: Cross-Module Extraction Strategy (Requires Research)

1. **Analyze cross-module dependencies**:
   - Run analysis to identify all cross-module subdirectory references
   - Create dependency graph showing which modules reference which tables
   - Example: Canon.pm → CanonCustom.pm, Exif.pm → manufacturer modules

2. **Design extraction approach**:
   - Option A: Extract all referenced tables into the referencing module's config
   - Option B: Create a separate "cross-module" extraction config
   - Option C: Implement runtime module loading (complex)

3. **Update tag_kit extractor**:
   - Modify to handle cross-module table references
   - May need to parse multiple .pm files in one extraction run

### Phase 3: Systematic Implementation

1. **Priority order** (based on tag frequency):
   - Canon custom functions (very common in Canon images)
   - Exif manufacturer subdirectories
   - Other manufacturer cross-references

2. **For each table**:
   - Extract table definition from source module
   - Generate parser function
   - Test with real images
   - Validate against ExifTool output

## Prerequisites

- Understanding of ExifTool's binary data format (see `PROCESS_PROC.md`)
- Familiarity with the tag kit extraction system
- Test images with the affected tags (Canon T3i.CR2 is a good example)

### Important: Check Existing Extractions First

Before implementing any binary parser manually:
1. Check if the table is already configured in `codegen/config/{Module}_pm/process_binary_data.json`
2. Look for existing generated code in `src/generated/{Module}_pm/binary_data_tables.rs`
3. ColorData tables are a good example of already-working binary data extraction

If the table is already extracted, the fix might be as simple as:
```rust
pub fn process_canon_shotinfo(data: &[u8], byte_order: ByteOrder) -> Result<Vec<(String, TagValue)>> {
    use crate::generated::Canon_pm::binary_data_tables::parse_shot_info;
    parse_shot_info(data, byte_order, None, None) // model and format params
}
```

## Testing Strategy

### Unit Tests
- Create test cases for each binary parser function
- Use known byte sequences with expected output values
- Example:
  ```rust
  #[test]
  fn test_process_canon_shotinfo() {
      let data = vec![0x00, 0x01, 0x00, 0x64, 0x00]; // AutoISO=1, BaseISO=100
      let result = process_canon_shotinfo(&data, ByteOrder::BigEndian).unwrap();
      assert_eq!(result[0], ("AutoISO".to_string(), TagValue::String("On".to_string())));
  }
  ```

### Integration Tests
- Test with real camera files (test-images/canon/*.CR2)
- Compare output with ExifTool using `cargo run --bin compare-with-exiftool`
- Ensure no regression in already-working ColorData extraction

### Manual Testing
```bash
# Test specific image
cargo run test-images/canon/Canon_T3i.CR2 | grep -E "(ProcessingInfo|CanonShotInfo|CRWParam)"

# Compare with ExifTool
./scripts/compare-with-exiftool.sh test-images/canon/Canon_T3i.CR2 MakerNotes:
```

## Success Criteria & Quality Gates

### Definition of Done
1. **Functionality**:
   - ProcessingInfo, CanonShotInfo, CRWParam show individual tag values, not arrays
   - Values match ExifTool output exactly (use compare-with-exiftool tool)
   - No regression in ColorData or other working subdirectories

2. **Code Quality**:
   - All parser functions follow ExifTool logic exactly (Trust ExifTool principle)
   - Comments reference ExifTool source locations
   - No manual lookup tables - use codegen for everything

3. **Testing**:
   - Unit tests pass for all new parser functions
   - Integration tests pass with real camera files
   - `make precommit` passes

## Gotchas & Tribal Knowledge

1. **Negative offsets**: Some tables (like Processing) use negative offsets counting from the end of the data. The binary data parser needs special handling for these.

2. **Format validation**: ExifTool often has `Validate` functions that check data integrity before parsing. We may need to implement these.

3. **Runtime conditions**: Some subdirectories have conditions like `$count == 582`. The dispatcher handles these, but be aware when testing.

4. **Cross-module complexity**: The CanonCustom module alone has ~30 different function tables. Don't try to implement all at once - focus on the most common ones first.

5. **Byte order**: Canon uses different byte orders for different data structures. Always respect the byte_order parameter passed to parser functions.

6. **Data size mismatches**: If the data size doesn't match expected table size, ExifTool often has fallback behavior. Check source for these cases.

7. **Hook information**: Some tables have Hook functions that modify behavior based on camera model. These need careful implementation.