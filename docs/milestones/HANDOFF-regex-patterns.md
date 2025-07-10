# HANDOFF: Regex pattern extraction implementation

**Priority**: CRITICAL - TRUST-EXIFTOOL Violation  
**Estimated Duration**: 4-6 hours remaining  
**Status**: 🟡 PARTIALLY COMPLETE - Extraction working, generation needs fixing  
**Prerequisites**: Understanding of the codegen architecture and TRUST-EXIFTOOL principle

## 🚨 Critical Issue Being Addressed

### TRUST-EXIFTOOL Violation

The codebase contains ~500 lines of hardcoded magic number patterns in `src/file_detection.rs` (lines 243-500+), directly violating the fundamental TRUST-EXIFTOOL principle. These patterns were manually copied from ExifTool instead of being automatically extracted, which means:

1. **They will drift from ExifTool's implementation** as ExifTool releases monthly updates
2. **They may already contain errors** from manual transcription
3. **They violate the core principle** that we must trust ExifTool's implementation exactly

### Current State

- `src/generated/file_types/magic_number_patterns.rs` exists but is EMPTY (0 patterns)
- The infrastructure partially exists but isn't working:
  - `codegen/config/ExifTool_pm/regex_patterns.json` config file exists
  - `codegen/src/generators/file_detection/patterns.rs` expects to process patterns
  - `src/file_detection.rs` already imports `get_magic_pattern` but it returns nothing

## 📚 Critical Files to Study

### 1. **ExifTool Source** (MUST READ FIRST)

- `third-party/exiftool/lib/Image/ExifTool.pm` lines 912-1027
  - The `%magicNumber` hash containing all patterns
  - Note the Perl regex syntax and binary bytes

### 2. **Existing Extractors** (Study as Templates)

- `codegen/extractors/file_type_lookup.pl` - Special extractor pattern to follow
- `codegen/extractors/simple_table.pl` - Basic extractor (insufficient for regex)
- `codegen/lib/ExifToolExtract.pm` - Shared utilities

### 3. **Rust Integration**

- `codegen/src/extraction.rs` - Where to add special extractor support
- `codegen/src/generators/file_detection/patterns.rs` - Processes regex_patterns.json
- `codegen/src/main.rs` - Calls generate_file_detection_code()

### 4. **Current Implementation** (To Be Replaced)

- `src/file_detection.rs` lines 243-500 - Hardcoded patterns to DELETE
- `src/generated/file_types/magic_number_patterns.rs` - Currently empty

## 🔍 Issues Already Investigated

### 1. **Infrastructure Exists But Doesn't Work**

- The config file `regex_patterns.json` exists
- The generator expects data but the extractor is missing
- No regex_patterns.pl extractor exists to populate the data

### 2. **Naming Inconsistency**

- Config uses "regex_patterns" naming
- Handoff doc mentions "magic_number" naming
- Code uses "magic_patterns" in some places
- **Recommendation**: Stick with "regex_patterns" to match existing config

### 3. **Binary Data Challenge**

- ExifTool patterns contain literal bytes (e.g., BPG pattern has `\xfb`)
- patterns.rs already has code to handle non-UTF-8 bytes
- See lines 85-106 in patterns.rs for the cleaning logic

### 4. **Path Issue**

- patterns.rs looks for regex_patterns.json in parent directory
- Line 66: `json_dir.parent()` - this needs to be fixed

## 🛠️ Implementation Guide

### Step 1: Create regex_patterns.pl Extractor

Create `codegen/extractors/regex_patterns.pl` based on file_type_lookup.pl:

```perl
#!/usr/bin/env perl
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

# Command line args
my $module_path = shift @ARGV or die "Usage: $0 <module_path> <hash_name>\n";
my $hash_name = shift @ARGV or die "Usage: $0 <module_path> <hash_name>\n";

# Load module and get hash
my $module_name = load_module_from_file($module_path);
my $hash_ref = get_package_hash($module_name, $hash_name);

# Extract patterns with metadata
# Output JSON structure matching what patterns.rs expects
```

### Step 2: Update extraction.rs

Add to the `SpecialExtractor` enum:

```rust
enum SpecialExtractor {
    FileTypeLookup,
    RegexPatterns,  // NEW
}
```

Update `needs_special_extractor()`:

```rust
if config_dir.join("regex_patterns.json").exists() {
    return Some(SpecialExtractor::RegexPatterns);
}
```

### Step 3: Fix Pattern Path in patterns.rs

Line 66 needs to be:

```rust
let regex_patterns_path = json_dir.join("regex_patterns.json");
```

### Step 4: Remove Hardcoded Patterns

Delete the entire `match_binary_magic_pattern()` function and update `validate_magic_number()` to use only generated patterns.

## ✅ COMPLETED WORK

### ✅ Extraction Infrastructure (COMPLETE)

1. **Extractor Created**: Updated `codegen/extractors/regex_patterns.pl` to work with new simplified architecture

   - ✅ Takes command line arguments (module_path, hash_name)
   - ✅ Extracts %magicNumber hash from ExifTool.pm
   - ✅ Outputs proper JSON format with 110 patterns
   - ✅ Successfully tested: extracts all patterns correctly

2. **Integration with Extraction System**: Updated `codegen/src/extraction.rs`

   - ✅ Added `RegexPatterns` to `SpecialExtractor` enum
   - ✅ Added `needs_special_extractor_by_name()` function
   - ✅ Updated config discovery to handle multiple configs per module
   - ✅ Added `run_regex_patterns_extractor()` function
   - ✅ Multi-config discovery working: processes all JSON files in ExifTool_pm directory

3. **Configuration**: Set up `codegen/config/ExifTool_pm/regex_patterns.json`

   - ✅ Proper source path and hash_name configuration
   - ✅ Integrated with extraction orchestration

4. **Validation**: Extraction tested and confirmed working
   - ✅ `make codegen` successfully extracts 110 patterns from %magicNumber
   - ✅ `generated/extract/regex_patterns.json` contains all patterns (28KB file)
   - ✅ No extraction errors - uses `regex_patterns.pl` correctly
   - ✅ Patterns include complex ones: PNG, HTML with (?i), binary sequences

## 🚨 REMAINING WORK (4-6 hours)

### 🔧 Generation Issues (CRITICAL)

1. **Data Structure Mismatch**: `patterns.rs` expects old format, extractor produces new format

   - ❌ Generator looks for `regex_patterns.json` but in wrong location
   - ❌ Expects `{patterns: {magic_numbers: []}}` structure
   - ❌ Extractor outputs `{magic_patterns: []}` structure
   - **Fix needed**: Update `MagicNumberData` struct to match extractor output

2. **File Path Issue**: Generator looks in wrong directory

   - ❌ Line 66 in patterns.rs: `json_dir.parent()` should be `json_dir`
   - **Fix needed**: Change to `json_dir.join("regex_patterns.json")`

3. **Function Naming Incomplete**: Started rename but incomplete
   - ❌ Function `run_magic_number_extractor()` exists but should be `run_regex_patterns_extractor()`
   - ❌ Some references still use "magic_number" terminology
   - **Fix needed**: Complete rename to use "regex_patterns" consistently

### 🔧 Integration Issues

4. **Generated File Processing**: `magic_number_patterns.rs` still empty

   - ❌ Generator runs but doesn't process the extracted patterns
   - ❌ Still shows "Generated regex patterns with 0 magic number patterns"
   - **Fix needed**: Fix data structure parsing in `generate_magic_patterns()`

5. **Remove Hardcoded Patterns**: Critical for TRUST-EXIFTOOL compliance
   - ❌ ~500 lines of hardcoded patterns still in `src/file_detection.rs`
   - ❌ `match_binary_magic_pattern()` function still exists (lines 243-500)
   - **Fix needed**: Delete hardcoded patterns and use only generated ones

## 🛠️ SPECIFIC TASKS REMAINING

### Task 1: Fix Data Structure Mismatch (1-2 hours)

Update `codegen/src/generators/file_detection/patterns.rs`:

```rust
// Current extractor output format:
#[derive(Debug, Deserialize)]
pub struct RegexPatternsExtractorData {
    pub extracted_at: String,
    pub magic_patterns: Vec<MagicPatternEntry>,  // Note: "magic_patterns" not "patterns.magic_numbers"
    pub stats: MagicNumberStats,
}

#[derive(Debug, Deserialize)]
pub struct MagicPatternEntry {
    pub file_type: String,  // Note: "file_type" not "key"
    pub pattern: String,
    pub source: MagicPatternSource,
}
```

Update `generate_magic_patterns()` to parse the actual extractor output format.

### Task 2: Fix Path and Function Names (30 minutes)

1. Fix line 66 in patterns.rs: `json_dir.join("regex_patterns.json")`
2. Rename `run_magic_number_extractor()` to `run_regex_patterns_extractor()`
3. Update `needs_special_extractor_by_name()` to use "regex_patterns"

### Task 3: Complete Generation Pipeline (1-2 hours)

1. Ensure `generate_magic_patterns()` is called correctly during codegen
2. Verify generated `magic_number_patterns.rs` contains actual patterns
3. Test that generated patterns compile with `regex::bytes::Regex`

### Task 4: Remove Hardcoded Patterns (1-2 hours)

1. Delete `match_binary_magic_pattern()` function entirely (lines 243-500 in file_detection.rs)
2. Update `validate_magic_number()` to use only `get_magic_number_pattern()`
3. Remove all hardcoded pattern matching logic

### Task 5: Testing and Validation (1 hour)

1. Run `cargo test file_type` and fix any failures
2. Run `make precommit` and ensure it passes
3. Test file detection on sample images to match ExifTool output

## 🧠 KEY FINDINGS FROM INVESTIGATION

### What's Working ✅

- **Extraction system completely working**: 110 patterns extracted successfully
- **Multi-config support**: ExifTool_pm can now have multiple extraction configs
- **Perl extractor robust**: Handles all ExifTool patterns including binary data
- **JSON output clean**: 28KB file with proper structure

### What's Broken ❌

- **Data format mismatch**: Generator expects old complex format, extractor uses simple format
- **Path issues**: Looking in wrong directories
- **Naming inconsistency**: Mix of "magic_number" and "regex_patterns" terminology. We want a **regex_patterns** extractor. It will perl and rust called **regex_patterns.*** and read from **regex_patterns.json**. don't call it magic_patterns, magic_numbers, or any variation thereof. Like EXIF:Orientation, the _application_ and _generated code_ should be called some variation of magic... of course.

  1. The extractor should be called regex_patterns
  2. It should use regex_patterns.pl (which I already updated)
  3. It should create regex_patterns.json
  4. The config should be regex_patterns.json not magic_number.json

- **Generation not triggered**: Patterns extracted but not generated

### Architecture Notes

- **Multi-config discovery working**: The discovery system now properly handles multiple configs in a module directory (ExifTool_pm has 3 configs: simple_table.json, file_type_lookup.json, regex_patterns.json)
- **Special extractor pattern proven**: The same pattern used for file_type_lookup.pl works for regex patterns
- **Rust regex::bytes capable**: Can handle all ExifTool patterns including binary data

## 🎯 SUCCESS CRITERIA UPDATED

### Must Complete ✅

- [x] Extract patterns from %magicNumber hash (110 patterns extracted)
- [x] Regex extractor integrated with orchestration
- [x] Multi-config support working
- [ ] **Generator processes extracted patterns correctly**
- [ ] **magic_number_patterns.rs contains actual patterns (not empty)**
- [ ] **All hardcoded patterns removed from file_detection.rs**
- [ ] **Tests pass with generated patterns only**

The core extraction is complete and robust. The remaining work is fixing the data format mismatch and removing hardcoded patterns.

## 🧠 Tribal Knowledge

### Perl Pattern Gotchas

1. **Null bytes**: Perl uses `\0` but Rust needs `\x00`
2. **Unicode**: Characters like û (0xfb) need byte representation
3. **Alternatives**: Patterns like `(\x89P|\x8aM|\x8bJ)NG` need careful parsing
4. **Case-insensitive**: HTML pattern has `(?i)` flag

### Binary vs Regex Strategy

Some patterns are more efficient as direct byte comparisons. The extractor should identify these simple cases:

- Fixed byte sequences (e.g., "BM" for BMP)
- Short patterns without regex features
- Let the generator decide the implementation

### Testing Individual Patterns

```bash
# Test the extractor directly
cd codegen/generated/extract
perl ../../extractors/regex_patterns.pl \
  ../../../third-party/exiftool/lib/Image/ExifTool.pm \
  %magicNumber

# Check specific pattern
cat regex_patterns.json | jq '.patterns.magic_numbers[] | select(.key=="PNG")'
```

### Common Errors

1. **"Hash not found"** - Check if patching is needed for %magicNumber
2. **Empty JSON** - Extractor isn't outputting to stdout properly
3. **Regex compile errors** - Pattern needs escaping for Rust

### ExifTool Reference Comments

Always add comments referencing ExifTool source:

```rust
// ExifTool.pm:998 - PNG magic: (\x89P|\x8aM|\x8bJ)NG\r\n\x1a\n
```

## 🎯 Next Steps After Completion

1. **Document the Pattern** - Update EXIFTOOL-INTEGRATION.md with regex pattern extraction
2. **Consider Optimizations** - Some patterns could use byte matching for speed
3. **Monitor ExifTool Updates** - New patterns added monthly
4. **Apply to Other Hashes** - Same approach for %fileExtensions, %mimeType

## 📝 Final Notes

This is a critical fix that brings us back into compliance with TRUST-EXIFTOOL. The infrastructure is mostly in place - it just needs the extractor to connect the pieces. Focus on getting the basic extraction working first, then optimize later. Remember: we're translating ExifTool, not improving it.

When in doubt, check how file_type_lookup.pl works - it's the proven pattern for special extractors.

## 🔍 Additional Research Findings

### Pattern Structure in ExifTool

From examining ExifTool.pm lines 912-1027, the %magicNumber hash contains approximately 90 file type patterns. Key patterns include:

- **Simple ASCII**: `BMP => 'BM'`, `GIF => 'GIF8[79]a'`
- **Binary sequences**: `PNG => '(\x89P|\x8aM|\x8bJ)NG\r\n\x1a\n'`
- **Complex regex**: `MOV => '.{4}(free|skip|wide|ftyp|pnot|PICT|pict|moov|mdat|junk|uuid)'`
- **Case-insensitive**: `HTML => '(\xef\xbb\xbf)?\s*(?i)<(!DOCTYPE\s+HTML|HTML|\?xml)'`

### JSON Structure Expected by patterns.rs

The generator expects this structure (from patterns.rs analysis):

```json
{
  "extracted_at": "timestamp",
  "patterns": {
    "magic_numbers": [
      {
        "key": "PNG",
        "pattern": "(\\x89P|\\x8aM|\\x8bJ)NG\\r\\n\\x1a\\n",
        "rust_compatible": 1,
        "compatibility_notes": "",
        "source_table": {
          "module": "ExifTool.pm",
          "hash_name": "%magicNumber",
          "description": "Magic number patterns"
        }
      }
    ]
  },
  "compatibility_notes": "Overall notes"
}
```

### Clarifications on Implementation Tasks

1. **Extractor Output Format**: The extractor must output the exact JSON structure above, not the simpler format used by file_type_lookup.pl

2. **Pattern Validation**: The extractor should test each pattern with Rust's regex crate and set `rust_compatible` accordingly

3. **Special Handling Required**:

   - Escape sequences: Convert Perl `\0` to Rust `\x00`
   - Binary data: Properly escape bytes >127
   - Case flags: Extract `(?i)` and handle separately

4. **Integration with Existing Code**:
   - `validate_magic_number()` currently calls `match_binary_magic_pattern()`
   - After fix, it should use `get_magic_pattern()` and compile regex or use byte matching

### Additional Validation Steps

1. **Pattern Count Verification**:

   ```bash
   # Count patterns in ExifTool source
   perl -ne 'print if /^\s*\w+\s*=>/' third-party/exiftool/lib/Image/ExifTool.pm | wc -l
   # Should match count in generated JSON
   ```

2. **Binary Pattern Testing**:

   - Create test files with known magic numbers
   - Verify detection matches ExifTool exactly
   - Special attention to: PNG, JPEG, BPG, JXL

3. **Performance Comparison**:

   - Time file detection before/after
   - Some patterns may need optimization

4. **Regex Compilation Test**:
   ```rust
   // Add temporary test in patterns.rs
   #[test]
   fn test_all_patterns_compile() {
       for (key, pattern) in MAGIC_NUMBER_PATTERNS.iter() {
           regex::bytes::Regex::new(pattern)
               .unwrap_or_else(|e| panic!("Pattern {} failed: {}", key, e));
       }
   }
   ```

### Critical Implementation Details

1. **The Path Fix is Essential**: Without fixing line 66 in patterns.rs, the generator won't find the JSON file

2. **Extractor Must Run After Patching**: The %magicNumber hash might need patching to be accessible (check if it's `my` or `our`)

3. **Binary Safety**: Some patterns contain actual binary bytes that break JSON. The extractor must escape these properly

4. **Consider Two-Phase Approach**:
   - Phase 1: Get basic extraction working with string patterns
   - Phase 2: Optimize with byte matching for simple patterns

This additional context should help ensure successful implementation of the magic number extraction system.
