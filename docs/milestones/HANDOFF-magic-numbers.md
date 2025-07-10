# HANDOFF: Magic Number Pattern Extraction Implementation

**Priority**: CRITICAL - TRUST-EXIFTOOL Violation  
**Estimated Duration**: 1-2 days  
**Status**: Ready for implementation  
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

## ✅ Success Criteria

1. **Extraction Works**
   - [ ] `make codegen` successfully extracts patterns from %magicNumber
   - [ ] `generated/extract/regex_patterns.json` contains all ~90 patterns
   - [ ] No errors during extraction

2. **Generation Works**
   - [ ] `magic_number_patterns.rs` contains actual patterns (not empty)
   - [ ] Generated patterns compile without regex errors
   - [ ] Binary patterns (like BPG) are properly escaped

3. **TRUST-EXIFTOOL Compliance**
   - [ ] ALL hardcoded patterns removed from file_detection.rs
   - [ ] `match_binary_magic_pattern()` function deleted entirely
   - [ ] Only generated patterns used for detection

4. **Tests Pass**
   - [ ] `cargo test file_type` passes
   - [ ] `make precommit` passes
   - [ ] File detection matches ExifTool for test images

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