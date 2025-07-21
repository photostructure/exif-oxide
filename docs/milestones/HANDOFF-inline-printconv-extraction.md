# Inline PrintConv Extraction - Handoff Document

## Goal

Create a codegen system to automatically extract and generate Rust lookup tables from inline PrintConv definitions found within ExifTool tag tables. This eliminates the need to manually maintain hundreds of lookup tables that are defined inline in ExifTool modules (e.g., Canon.pm has ~163 inline PrintConv tables).

## What Was Accomplished

### 1. Created Perl Extractor (`codegen/extractors/inline_printconv.pl`)
- Uses Perl interpreter to properly parse tag table structures (no regex parsing!)
- Extracts simple hash mappings from inline PrintConv definitions
- Skips complex structures (Perl code, BITMASK tables, references)
- Determines appropriate key types (u8, u16, i16, String, etc.)
- Outputs JSON files with extracted data

### 2. Updated Extraction Pipeline (`codegen/src/extraction.rs`)
- Added `inline_printconv.json` to supported config files
- Created `SpecialExtractor::InlinePrintConv` variant
- Implemented `run_inline_printconv_extractor` function
- Modified config parsing to handle table names instead of hash names for inline_printconv

### 3. Created Code Generator (`codegen/src/generators/lookup_tables/inline_printconv.rs`)
- Generates Rust lookup tables from extracted inline PrintConv data
- Handles different key types (numeric and string)
- Creates lookup functions with appropriate signatures
- Generates modular output files

### 4. Integrated into Lookup Tables System
- Updated `codegen/src/generators/lookup_tables/mod.rs`
- Added handling for `inline_printconv.json` configs in `process_config_directory`
- Created `generate_inline_printconv_file` function

### 5. Created Canon Configuration (`codegen/config/Canon_pm/inline_printconv.json`)
- Configured extraction for major Canon tables (CameraSettings, ShotInfo, etc.)
- Successfully extracts hundreds of inline PrintConv definitions

## Key Learnings & Context

### 1. Trust ExifTool Principle
- Every inline PrintConv exists for a reason (camera quirks discovered over 25 years)
- We translate exactly, never "improve" or "optimize"
- Some PrintConv values contain special entries like -1 for "n/a"

### 2. Extraction Challenges
- ExifTool tables can contain mixed PrintConv types:
  - Simple hashes: `PrintConv => { 1 => 'Macro', 2 => 'Normal' }`
  - References: `PrintConv => \%canonQuality`
  - Perl code: `PrintConv => q{ return $val ? 'On' : 'Off' }`
  - Complex structures: `PrintConv => { 0 => 'Off', BITMASK => { ... } }`
- Our extractor only handles simple hashes (which covers most cases)

### 3. Key Type Detection
- Keys can be numeric (u8, u16, i16, etc.) or strings
- Negative values indicate signed types
- Large values (>32767) need wider types
- String keys (like lens IDs "10.1", "10.2") need special handling

### 4. Module Patching Not Required
- Unlike simple_table extraction, inline PrintConv doesn't need module patching
- Tag tables are already package-scoped (`%Image::ExifTool::Canon::CameraSettings`)

## Remaining Tasks

### 1. Fix JSON Serialization Issue
The Perl extractor has a minor bug where `entry_count` isn't always included in the JSON output. The fix is already implemented but needs testing.

(But do we need entry_count included in the json payload in the first place? Wouldn't it be simpler to have rust just count the dictionary/map/set entries?)

### 2. Complete the Canon Module Generation
- Run `make clean` in codegen directory
- Run `make codegen` to regenerate all files
- Verify generated files in `src/generated/Canon_pm/`
- Check that inline PrintConv lookups are properly generated

### 3. Add More Manufacturer Modules
Create inline_printconv.json configs for other manufacturers:
- Nikon.pm (AFAreaMode, ISO settings, etc.)
- Sony.pm (exposure programs, white balance settings)
- Olympus.pm (art filters, scene modes)
- Fujifilm.pm (film simulations, dynamic range)

### 4. Update Documentation
- Add inline PrintConv extraction to `docs/CODEGEN.md`
- Update codegen README with new extractor type
- Document the generated function naming convention

### 5. Integration with Manual Implementations
Update manual PrintConv implementations to use generated inline tables:
```rust
// Before (manual)
match value {
    1 => "Macro",
    2 => "Normal",
    _ => "Unknown",
}

// After (using generated)
use crate::generated::Canon_pm::camera_settings_inline::lookup_camera_settings_macro_mode;
lookup_camera_settings_macro_mode(value).unwrap_or("Unknown")
```

### 6. Testing
- Create unit tests for key type detection
- Add integration tests comparing output with ExifTool
- Test edge cases (negative values, large numbers, string keys)

## File Locations

### Created/Modified Files
- `/home/mrm/src/exif-oxide/codegen/extractors/inline_printconv.pl` - Perl extractor
- `/home/mrm/src/exif-oxide/codegen/src/extraction.rs` - Pipeline integration
- `/home/mrm/src/exif-oxide/codegen/src/generators/lookup_tables/inline_printconv.rs` - Code generator
- `/home/mrm/src/exif-oxide/codegen/src/generators/lookup_tables/mod.rs` - Module updates
- `/home/mrm/src/exif-oxide/codegen/config/Canon_pm/inline_printconv.json` - Canon config

### Generated Output Examples
- `/home/mrm/src/exif-oxide/codegen/generated/extract/inline_printconv__camera_settings.json`
- `/home/mrm/src/exif-oxide/codegen/generated/extract/inline_printconv__shot_info.json`
- (Future) `/home/mrm/src/exif-oxide/src/generated/Canon_pm/camera_settings_inline.rs`

## Next Engineer Action Items

1. **Immediate**: Fix and test the JSON serialization (entry_count field)
2. **Short term**: Complete Canon module generation and verify output
3. **Medium term**: Add configs for other manufacturer modules
4. **Long term**: Update all manual PrintConv implementations to use generated tables

## Success Criteria

- All inline PrintConv tables from configured modules are automatically extracted
- Generated Rust code compiles without errors
- Lookup functions return correct values matching ExifTool output
- No manual maintenance required for inline PrintConv tables
- Monthly ExifTool updates automatically refresh lookup tables via `make codegen`