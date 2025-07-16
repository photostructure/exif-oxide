# HANDOFF: Fix Boolean Set Code Generation

**Date:** 2025-07-16  
**Status:** In Progress - Build errors fixed, boolean sets partially working  
**Priority:** High - Blocking compilation  

## Prerequisite research

read CLAUDE.md TRUST-EXIFTOOL.md and EXIFTOOL-INTEGRATION.md -- as well as any other linked documentation that is relevant to these tasks.

## Issues Being Addressed

### 1. Build Failures (🚧 IN PROGRESS)
The codebase had compilation errors due to:
- Missing `file_types/mod.rs` file  
- Incorrect imports (`Lazy` vs `LazyLock`) in generated code  
- Simple table optimization changed to use `LazyLock` but imports were missing

### 2. Boolean Set Generation Issues (🚧 IN PROGRESS)
Boolean sets (PNG_pm and ExifTool_pm) are not being generated despite:
- Extraction working correctly (files exist in `codegen/generated/extract/`)
- Configuration files present and valid
- Patching system working

### 3. Remove some foot-guns: the static list of config directories

- codegen/src/main.rs and codegen/src/validation.rs both have a hard-coded list of configuration directories.
- the perl patching code used to have a hard-coded list of files and variables to patch

We want to make sure all new engineers need to do to add a new extraction is create a directory and JSON config, and run `make codegen` -- not waste 45 minutes wondering why their new config is being ignored.


## Root Cause Identified

The issue is a **module name format mismatch**:

1. **Simple tables** use module names like `"Canon.pm"` in extracted JSON
2. **Boolean sets** use module names like `"Image::ExifTool::PNG"` in extracted JSON
3. The code does `.replace(".pm", "_pm")` which works for simple tables but not boolean sets
4. This causes the matching logic to fail when looking for configs

## Code Already Fixed

### 1. File type module generation (main.rs:141-180)
- Changed to create `file_types/mod.rs` if it doesn't exist
- Properly imports and re-exports file type detection modules

### 2. LazyLock usage (standard.rs:116)
- Changed from `Lazy` to `LazyLock` to match std library
- Removed `once_cell` dependency requirement

### 3. Added debug code (main.rs:285-330)
- Added special handling for boolean_set files
- Added debug output to trace the matching issue

## Code to Study

### Key Files to Understand the Issue

1. **`/codegen/src/main.rs`** (lines 200-330)
   - The main matching logic that processes extracted JSON files
   - See the module name handling at line 220 and 290

2. **`/codegen/src/extraction.rs`** (lines 140-150)
   - Shows how config parsing strips `%` from hash names
   - Important for understanding the extraction pipeline

3. **`/codegen/src/generators/lookup_tables/mod.rs`**
   - Shows how simple tables and boolean sets are processed
   - Both use the same matching pattern

4. **Extraction output differences:**
   ```bash
   # Simple table extracted file
   cat codegen/generated/extract/canon_model_i_d.json | jq .source
   # Shows: module: "Canon.pm"
   
   # Boolean set extracted file  
   cat codegen/generated/extract/boolean_set_isDatChunk.json | jq .source
   # Shows: module: "Image::ExifTool::PNG"
   ```

## Success Criteria

1. ❌ Build compiles without errors
2. ❌ PNG_pm module is generated with all 3 boolean sets
3. ❌ ExifTool_pm module includes boolean sets at the end
4. ❌ All tests pass with `make precommit`
5. ❌ No warnings about "Could not find config for boolean_set_*"

## Remaining Tasks

### 1. Fix Module Name Matching
The core issue is in `main.rs` around line 220 and 290:
```rust
let module_name = simple_table.source.module.replace(".pm", "_pm");
```

This needs to handle both formats:
- `"Canon.pm"` → `"Canon_pm"`  
- `"Image::ExifTool::PNG"` → `"PNG_pm"`

**Suggested fix:**
```rust
let module_name = if simple_table.source.module.starts_with("Image::ExifTool::") {
    simple_table.source.module
        .strip_prefix("Image::ExifTool::")
        .unwrap()
        .to_string() + "_pm"
} else {
    simple_table.source.module.replace(".pm", "_pm")
};
```

### 2. Verify Boolean Set Generation
After fixing the module name:
1. Run `make codegen`
2. Check that `src/generated/PNG_pm/` directory is created
3. Verify `src/generated/ExifTool_pm/mod.rs` contains boolean sets

### 3. Clean Up Debug Code
Remove the debug code added in main.rs (lines 285-330) once the issue is fixed.

### 4. Run Full Test Suite
```bash
make precommit
```

## Important Notes

### Extraction vs Generation Pattern
The system follows this pattern:
1. **Extraction** creates JSON files in `codegen/generated/extract/`
2. **Generation** reads those files and matches them with configs
3. Configs use `%` prefix in hash names (per schema requirement)
4. Extracted files keep the `%` in hash names
5. Module names differ between simple tables and boolean sets

### Patching System
- The patching system works correctly (changes `my` to `our`)
- PNG.pm gets patched during extraction
- Patches are reverted after extraction completes

### Testing Individual Components
```bash
# Test extraction only
cd codegen && make -f Makefile.modular extract

# Test generation only  
cd codegen && cargo run --release

# Check specific module generation
ls -la src/generated/PNG_pm/

# Verify boolean set extraction worked
ls codegen/generated/extract/boolean_set_*.json
```

## Tribal Knowledge

1. **Module naming conventions** are inconsistent between ExifTool modules
2. **Simple tables** come from module-specific files (Canon.pm, Nikon.pm)
3. **Boolean sets** can come from the main ExifTool.pm or submodules
4. The **schema requires** `%` prefix for hash names in configs
5. **Don't remove** the `%` prefix - it's used for matching
6. The **fired engineer** tried to use `once_cell::Lazy` - stick with `std::sync::LazyLock`

Good luck! The fix should be straightforward once you understand the module name mismatch.