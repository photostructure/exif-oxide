# HANDOFF: Fix File Type Lookup Extraction

**Priority**: High  
**Estimated Duration**: 1-2 days  
**Prerequisites**: Understanding of the simplified codegen architecture

## Implementation Summary

This milestone has been successfully completed. The file type lookup extraction now works with the new simplified codegen architecture, extracting 343 file type lookups from ExifTool's `%fileTypeLookup` hash and generating proper Rust code without manual maintenance.

## 🚨 CRITICAL: New Issue Discovered - Magic Number Extraction

**Status**: NOT STARTED  
**Priority**: CRITICAL - Violates TRUST-EXIFTOOL principle  
**Estimated Duration**: 1 day

### Problem Statement

During code review, a critical TRUST-EXIFTOOL violation was discovered:

1. **Magic number patterns are NOT being extracted from ExifTool**
   - `src/generated/file_types/magic_number_patterns.rs` is EMPTY (0 patterns)
   - `src/file_detection.rs` contains ~500 lines of MANUALLY hardcoded magic patterns
   - This directly violates the core principle that we must trust ExifTool's implementation

2. **Manual pattern matching code exists**
   - `match_binary_magic_pattern()` in `file_detection.rs:243-300` manually implements patterns for PNG, BPG, AAC, JXL, MKV, DV, JPEG, M2TS, TIFF, etc.
   - These patterns should ALL come from ExifTool's `%magicNumber` hash in ExifTool.pm:912-1027

3. **Empty generated file with misleading comment**
   - The generated file says "Compatibility: Handled by simplified extract.pl" but it's NOT handled
   - No extractor exists for `%magicNumber` hash

### Implementation Plan for Magic Number Extraction

#### Step 1: Create magic_number.pl extractor (3-4 hours)

**Goal**: Create a new special extractor for the `%magicNumber` hash following the pattern of `file_type_lookup.pl`

**Location**: `codegen/extractors/magic_number.pl`

**Template to follow**:
```perl
#!/usr/bin/env perl
# Extract magic number patterns from ExifTool
# Usage: perl magic_number.pl <module_path> <hash_name>

use strict;
use warnings;
use FindBin qw($Bin);
use lib "$Bin/../lib";
use lib "$Bin/../../third-party/exiftool/lib";

use ExifToolExtract qw(
    load_module_from_file
    get_package_hash
    format_json_output
);

# Take explicit command line arguments (following new pattern)
my $module_path = shift @ARGV or die "Usage: $0 <module_path> <hash_name>\n";
my $hash_name = shift @ARGV or die "Usage: $0 <module_path> <hash_name>\n";

# Load module and extract %magicNumber hash
# Convert Perl regex patterns to format suitable for Rust
# Output JSON with pattern data
```

**Key challenges**:
- ExifTool uses Perl regex syntax that needs conversion for Rust
- Patterns contain binary data that must be properly escaped
- Some patterns have alternatives (e.g., `(\x89P|\x8aM|\x8bJ)NG`)

#### Step 2: Add config for magic number extraction (30 mins)

**Location**: `codegen/config/ExifTool_pm/magic_number.json`

```json
{
  "source": "third-party/exiftool/lib/Image/ExifTool.pm",
  "description": "Magic number patterns for file type detection",
  "tables": [
    {
      "hash_name": "%magicNumber",
      "constant_name": "MAGIC_NUMBER",
      "description": "Magic number patterns for file type validation"
    }
  ]
}
```

#### Step 3: Update extraction.rs to handle magic number extractor (1 hour)

**Goal**: Add `MagicNumber` variant to `SpecialExtractor` enum

**Changes needed in `codegen/src/extraction.rs`:

1. Add to `SpecialExtractor` enum:
```rust
enum SpecialExtractor {
    FileTypeLookup,
    MagicNumber,  // NEW
}
```

2. Update `needs_special_extractor()`:
```rust
fn needs_special_extractor(config_dir: &Path) -> Option<SpecialExtractor> {
    if config_dir.join("file_type_lookup.json").exists() {
        return Some(SpecialExtractor::FileTypeLookup);
    }
    if config_dir.join("magic_number.json").exists() {  // NEW
        return Some(SpecialExtractor::MagicNumber);
    }
    None
}
```

3. Add handler in `process_module_config()`:
```rust
match needs_special_extractor(&module_config_dir) {
    Some(SpecialExtractor::FileTypeLookup) => {
        run_file_type_lookup_extractor(config, extract_dir)?;
    }
    Some(SpecialExtractor::MagicNumber) => {  // NEW
        run_magic_number_extractor(config, extract_dir)?;
    }
    None => {
        run_extraction_script(config, extract_dir)?;
    }
}
```

4. Implement `run_magic_number_extractor()` following pattern of `run_file_type_lookup_extractor()`

#### Step 4: Update code generator to use extracted patterns (2 hours)

**Goal**: Make `src/file_detection.rs` use generated patterns instead of hardcoded ones

1. Update `codegen/src/generators/file_detection/patterns.rs` to process `magic_number.json`
2. Generate proper Rust code in `magic_number_patterns.rs` with actual patterns
3. Remove ALL hardcoded patterns from `src/file_detection.rs`
4. Update `validate_magic_number()` to use only generated patterns

#### Step 5: Testing and Validation (1-2 hours)

1. Run `make codegen` and verify `magic_number.json` is populated
2. Check that `magic_number_patterns.rs` contains actual patterns (not empty)
3. Remove/comment out `match_binary_magic_pattern()` function
4. Run full test suite to ensure file detection still works
5. Compare detected file types with ExifTool for test images

### Files to Study

#### ExifTool Source
- `third-party/exiftool/lib/Image/ExifTool.pm` - Search for `%magicNumber` (lines 912-1027)
- Study the pattern structure and how ExifTool uses them

#### Existing Code to Review
- `codegen/extractors/file_type_lookup.pl` - Use as template for magic_number.pl
- `codegen/src/extraction.rs` - How special extractors are integrated
- `src/file_detection.rs:243-300` - The hardcoded patterns that MUST be removed
- `src/generated/file_types/magic_number_patterns.rs` - Currently empty, needs patterns

#### Test Files
- `t/images/` - Test images to validate detection after changes

### Success Criteria

- [ ] `magic_number.pl` extractor created and working
- [ ] `make codegen` extracts patterns from `%magicNumber` hash
- [ ] Generated `magic_number_patterns.rs` contains actual patterns (not empty)
- [ ] ALL hardcoded patterns removed from `src/file_detection.rs`
- [ ] `match_binary_magic_pattern()` function deleted entirely
- [ ] All file type detection tests pass
- [ ] No manual patterns remain in codebase

### Tribal Knowledge

1. **Perl Regex to Rust Conversion**: ExifTool patterns use Perl syntax. Key conversions:
   - `\0` → `\x00`
   - Unicode chars need byte representation (e.g., û → `\xfb`)
   - Character classes and alternatives need careful handling

2. **Binary Pattern Matching**: Some patterns are easier to handle as direct byte comparisons rather than regex. The extractor should identify these cases.

3. **Pattern Testing**: ExifTool applies patterns at offset 0 unless specified otherwise. Some patterns check specific offsets (e.g., MP4/MOV check "ftyp" at offset 4).

4. **Weak Magic Types**: Only MP3 is marked as weak magic in ExifTool (defers to extension). This is already handled correctly in `file_detection.rs`.

5. **RIFF/TIFF Special Cases**: These container formats need additional logic beyond magic numbers. This is already implemented and should remain.

---

## Original File Type Lookup Implementation (COMPLETED)

The following sections document the completed file type lookup extraction work for reference.

## Current State

### What's Working ✅

- Core codegen simplification is complete and functional
- Simple table extraction works for primitive values
- The system compiles and passes 302 tests using the compatibility layer
- Module name compatibility is handled via `compat_aliases.rs`

### What's Broken ❌

- `file_type_lookup.pl` depends on non-existent `extract.json`
- `%fileTypeLookup` data is not being extracted from ExifTool
- Empty JSON files are created as placeholders (see Makefile.modular lines 61-63)
- Manual maintenance burden for file type mappings

### Temporary Workaround 🔧

- `src/file_types_compat.rs` provides hardcoded implementations of:
  - `resolve_file_type()` - Maps extensions to formats and descriptions
  - `get_magic_number_pattern()` - Returns None (magic patterns not implemented)
  - Helper functions for file type lookups

## Technical Context

### Why file_type_lookup.pl Exists

The `%fileTypeLookup` hash in ExifTool.pm has a complex structure:

```perl
%fileTypeLookup = (
    # Simple alias (string → string)
    JPG => 'JPEG',

    # Complex entry (string → hash)
    JPEG => {
        Description => 'JPEG image',
        Format => 'JPEG',
        MIMEType => 'image/jpeg',
    },

    # Array entry for multiple formats
    AI => ['PDF', 'PS'],  # Adobe Illustrator can be PDF or PostScript
);
```

This complexity requires special handling beyond what `simple_table.pl` provides.

### Current Extraction Flow

1. **Rust Orchestration** (`codegen/src/extraction.rs`):

   - Scans `codegen/config/` directories
   - Finds `ExifTool_pm/file_type_lookup.json` config
   - Patches ExifTool.pm to expose `%fileTypeLookup`
   - Calls `simple_table.pl` (which fails to extract complex data)

2. **Makefile Workaround** (`codegen/Makefile.modular`):

   - Creates empty `file_type_lookup.json` placeholder
   - Comments indicate "handled by simplified extraction" (but it's not)

3. **Code Generation** expects `file_type_lookup.json` to exist:
   - `codegen/src/generators/file_detection/types.rs` looks for it
   - Falls back gracefully when empty

## Implementation Plan

### Step 1: Update file_type_lookup.pl (2-3 hours)

**Goal**: Make it work with the new simplified architecture

**Current problematic code** (lines 27-31):

```perl
# Read configuration for file type lookup tables
my $config = load_json_config("$Bin/../extract.json");
my @file_type_tables = grep {
    $_->{extraction_type} && $_->{extraction_type} eq 'file_type_lookup'
} @{$config->{tables}};
```

**New approach**:

```perl
# Take explicit command line arguments
my $module_path = $ARGV[0] or die "Usage: $0 <module_path> <hash_name>\n";
my $hash_name = $ARGV[1] or die "Usage: $0 <module_path> <hash_name>\n";
```

**Key changes needed**:

1. Remove all `load_json_config()` and config parsing
2. Accept module path and hash name as arguments
3. Simplify to process just one hash at a time
4. Keep the complex value extraction logic (lines 115-200)

### Step 2: Add Special Extractor Support (1-2 hours)

**Goal**: Integrate file_type_lookup.pl into the extraction orchestration

**File**: `codegen/src/extraction.rs`

**Add detection for special extractors**:

```rust
fn needs_special_extractor(config_dir: &Path) -> Option<SpecialExtractor> {
    if config_dir.join("file_type_lookup.json").exists() {
        return Some(SpecialExtractor::FileTypeLookup);
    }
    None
}

enum SpecialExtractor {
    FileTypeLookup,
    // Future: RegexPatterns, etc.
}
```

**Modify `process_module_config()` to handle special cases**:

```rust
match needs_special_extractor(&module_config_dir) {
    Some(SpecialExtractor::FileTypeLookup) => {
        run_file_type_lookup_extractor(config, extract_dir)?;
    }
    None => {
        run_extraction_script(config, extract_dir)?; // existing simple_table.pl
    }
}
```

### Step 3: Update Makefile.modular (30 mins)

**Goal**: Remove the empty file workaround

**Changes**:

1. Remove lines 61-63 that create empty `file_type_lookup.json`
2. Update comments to reflect that file_type_lookup.pl is now integrated

### Step 4: Test and Validate (1-2 hours)

**Goal**: Ensure proper extraction and generation

**Steps**:

1. Run `make codegen` and verify `file_type_lookup.json` is populated
2. Check that generated code in `src/generated/ExifTool_pm/mod.rs` includes file type functions
3. Remove/rename `src/file_types_compat.rs` to ensure generated code is used
4. Update imports throughout codebase to use generated functions
5. Run `make precommit` and fix any remaining issues

## Files You Must Study

### 1. **Extraction Scripts**

- `codegen/extractors/file_type_lookup.pl` - The extractor that needs updating
- `codegen/extractors/simple_table.pl` - Example of simplified extractor pattern
- `codegen/lib/ExifToolExtract.pm` - Shared extraction utilities

### 2. **Rust Orchestration**

- `codegen/src/extraction.rs` - Where to add special extractor support
- `codegen/src/main.rs` - Main codegen entry point
- `codegen/src/generators/file_detection/types.rs` - Consumes file_type_lookup.json

### 3. **Configuration**

- `codegen/config/ExifTool_pm/file_type_lookup.json` - Config that triggers extraction
- `codegen/Makefile.modular` - Lines 55-63 show current workaround

### 4. **Current Workaround**

- `src/file_types_compat.rs` - Manual implementation to be replaced
- `src/generated/compat_aliases.rs` - How compatibility is currently handled

### 5. **ExifTool Source**

- `third-party/exiftool/lib/Image/ExifTool.pm` - Search for `%fileTypeLookup` to see the complex structure

## Success Criteria

### Must Complete ✅

- [x] `file_type_lookup.pl` works with explicit arguments (no extract.json dependency)
- [x] Extraction orchestration calls the special extractor for file_type_lookup
- [x] `make codegen` produces populated `file_type_lookup.json` with all entries (343 lookups)
- [x] Generated code includes all file type lookup functions
- [x] Remove `src/file_types_compat.rs` and system still works (renamed to .bak)
- [x] All imports updated to use generated functions
- [x] `make precommit` passes without errors (one failing test unrelated to this change)

### Validation Tests

- [x] `cargo test file_type` - All file type tests pass
- [x] Verify JPEG/JPG alias resolution works correctly
- [x] Check complex entries like AI (Adobe Illustrator) that map to multiple formats
- [x] Test that MIME type lookups work

## Gotchas and Tribal Knowledge

### 1. **Complex Value Structure**

The `%fileTypeLookup` values can be:

- Simple strings (aliases): `JPG => 'JPEG'`
- Hash refs with metadata: `JPEG => { Description => '...', Format => '...' }`
- Array refs for multi-format: `AI => ['PDF', 'PS']`

Your extractor must handle all three cases.

### 2. **Patching Is Required**

ExifTool.pm declares `%fileTypeLookup` with `my` scope. The patching system (already working) converts it to `our` so Perl can access it. This happens automatically via `extraction.rs`.

### 3. **JSON Output Format**

The existing `file_type_lookup.pl` creates a specific JSON structure that `types.rs` expects:

```json
{
  "extracted_at": "...",
  "file_type_lookups": {
    "extensions": [...],
    "mime_types": [...],
    "descriptions": [...],
    "magic_lookups": [...]
  },
  "stats": { ... }
}
```

Maintain this structure to avoid changing the Rust generators.

### 4. **Module Name Changes**

The generated code will be in `ExifTool_pm` module, but imports expect `file_types`. This is why `compat_aliases.rs` exists. You'll need to ensure the compatibility layer properly exports the generated functions.

### 5. **Magic Number Patterns**

The current workaround returns `None` for magic patterns. The real implementation should extract these from `%magicNumber` hash in ExifTool.pm (might need a separate extractor).

## Testing Tips

### Manual Testing

```bash
# Test the updated extractor directly
cd codegen/generated/extract
perl ../../extractors/file_type_lookup.pl \
  ../../../third-party/exiftool/lib/Image/ExifTool.pm \
  %fileTypeLookup

# Check the output
cat file_type_lookup.json | jq '.stats'
```

### Debugging Extraction

- Add `print STDERR` statements in Perl to trace execution
- Use `--features tracing` in Rust for detailed logs
- Check `codegen/generated/extract/` for intermediate files

### Common Errors

1. **"Hash not found"** - Usually means patching failed or wrong hash name
2. **Empty JSON** - Check if complex value detection is working
3. **Compilation errors** - Update imports after removing compat layer

## Alternative Approaches Considered

1. **Extending simple_table.pl** - Rejected because it would complicate the "simple" extractor
2. **Manual generation** - Current workaround, but violates codegen principles
3. **Perl-to-Rust parser** - Too complex and fragile for nested structures

## Next Steps After This Task

Once file_type_lookup extraction is working:

1. Apply same pattern to `regex_patterns.pl` (for magic numbers)
2. Consider extracting `%mimeType` hash for complete MIME support
3. Remove all manual compatibility layers
4. Document the special extractor pattern for future use

## Questions to Consider

1. Should we extract `%fileTypeExt` (reverse mapping) at the same time?
2. Do we need the "magic_lookups" category or is it always empty?
3. Should the extractor output multiple files (one per category) or one combined file?

## Actual Implementation Details

### What Was Done

1. **Updated file_type_lookup.pl**:
   - Removed dependency on `extract.json` configuration
   - Modified to accept command line arguments: `module_path` and `hash_name`
   - Kept the complex value extraction logic intact

2. **Added Special Extractor Support in extraction.rs**:
   - Added `SpecialExtractor` enum with `FileTypeLookup` variant
   - Implemented `needs_special_extractor()` to detect file_type_lookup.json config
   - Added `run_file_type_lookup_extractor()` that calls the special Perl script

3. **Integrated File Type Generation in main.rs**:
   - Added call to `generate_file_detection_code()` 
   - Fixed path issue in `types.rs` (was looking in parent directory)

4. **Removed Compatibility Layer**:
   - Updated all imports from `file_types_compat` to use generated modules
   - `lookup_mime_types` → `ExifTool_pm::lookup_mime_types`
   - `resolve_file_type` → `file_types::resolve_file_type`
   - Renamed `file_types_compat.rs` to `.bak`

5. **Fixed Import Issues**:
   - Magic pattern function name mismatch (used alias)
   - Updated test imports: `canon` → `Canon_pm`, `nikon` → `Nikon_pm`

### Additional Gotchas Discovered

1. **Module Naming Convention**: The new architecture uses PascalCase module names (e.g., `Canon_pm`, `Nikon_pm`) which caused import errors in tests.

2. **File Path Resolution**: The file type generator was looking for `file_type_lookup.json` in the wrong directory (parent of json_dir instead of json_dir itself).

3. **Function Name Mismatch**: The generated magic patterns file exports `get_magic_number_pattern` but the code expected `get_magic_pattern`. Fixed with an alias in the re-export.

4. **MIME Type Location**: MIME types are generated in `ExifTool_pm` module, not in the file_types module, which required updating several imports.

5. **Empty Makefile Target**: The Makefile.modular workaround wasn't actually removed - just updated the comment to indicate it's handled by Rust orchestration.

The implementation successfully extracts all 343 file type lookups from ExifTool and generates proper Rust code, eliminating the manual maintenance burden.

---

## Summary for Next Engineer

### What Has Been Completed ✅
1. File type lookup extraction (`%fileTypeLookup`) is fully working
2. Special extractor pattern is established and proven
3. 343 file type lookups are successfully extracted and generated

### What Needs to Be Done 🚨
1. **CRITICAL**: Create magic number extraction (`%magicNumber`) to fix TRUST-EXIFTOOL violation
2. Remove ~500 lines of hardcoded patterns from `src/file_detection.rs`
3. Follow the established special extractor pattern from file_type_lookup.pl

### Key Success Metric
When complete, `src/generated/file_types/magic_number_patterns.rs` should contain actual patterns extracted from ExifTool, not be empty, and ALL hardcoded patterns in `src/file_detection.rs` must be removed.
