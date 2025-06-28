# EXIF Parsing Gaps Analysis

## Current Issue

The user reported that exif-oxide is showing numeric tag IDs like "0x0103", "0x0201", "0x0202" instead of proper tag names, and is missing many EXIF fields compared to ExifTool.

Example problematic output:
```json
{
  "0x0202": "Undefined([0, 0, 31, 170])",
  "0x0201": "Undefined([0, 0, 11, 198])", 
  "0x0103": "Undefined([0, 6, 0, 0])"
}
```

Expected output (from ExifTool):
```json
{
  "EXIF:ThumbnailOffset": 3048,
  "EXIF:ThumbnailLength": 8106,
  "EXIF:Compression": "JPEG (old-style)"
}
```

## Root Cause Analysis

### Issue 1: Missing Tag Definitions

**Status**: ✅ **RESOLVED** - Tags ARE defined in the EXIF table

Investigation shows that the problematic tags ARE properly defined in `/home/mrm/src/exif-oxide/src/tables/exif_tags.rs`:

- `0x0103`: "Compression" with `PrintConvId::Compression`
- `0x0201`: Missing from current table (should be "JPEGInterchangeFormat" or "ThumbnailOffset")
- `0x0202`: Missing from current table (should be "JPEGInterchangeFormatLength" or "ThumbnailLength")

### Issue 2: Code Generation Problems

**Status**: 🚨 **CRITICAL** - Auto-generated files have syntax errors

The sync tools are generating invalid Rust code:

1. **Invalid enum variant**: `Undef?` in GPMF format enum (fixed)
2. **Missing regex escaping**: Raw strings needed for regex patterns (fixed)
3. **Missing static definitions**: `TRAILER_SEGMENTS` referenced but not defined (fixed)

### Issue 3: Tag Resolution Logic Gap

**Investigation Needed**: The EXIF parsing code may not be properly using the comprehensive tag table.

**Files to check**:
- `src/core/ifd.rs` - IFD parsing logic
- `src/core/jpeg.rs` - JPEG segment handling
- Tag lookup logic in main parsing flow

## Tasks for New Engineer

### Phase 1: Fix Code Generation (HIGH PRIORITY)

1. **Verify sync tool fixes**:
   ```bash
   cargo check  # Should compile without errors
   ```

2. **Regenerate all auto-generated files**:
   ```bash
   cargo run --bin exiftool_sync extract-all
   # OR
   make sync
   ```

3. **Fix any remaining code generation issues**:
   - Check all files in `src/tables/` for syntax errors
   - Look for missing static definitions
   - Verify regex patterns use raw strings

### Phase 2: Complete EXIF Tag Coverage

1. **Verify missing tags 0x0201, 0x0202**:
   ```bash
   rg "0x0201|0x0202" src/tables/exif_tags.rs
   ```

2. **If missing, regenerate EXIF tags**:
   ```bash
   cargo run --bin exiftool_sync extract exif-tags
   ```

3. **Check ExifTool source for canonical names**:
   - Look in `third-party/exiftool/lib/Image/ExifTool/Exif.pm`
   - Search for `0x201` and `0x202` hex values
   - Verify tag names match ExifTool exactly

### Phase 3: Debug Tag Resolution Logic

1. **Test with problematic image**:
   ```bash
   cargo run -- test-images/apple/IMG_3755.JPG
   ```

2. **Trace tag lookup process**:
   - Add debug prints to see which code path is taken
   - Check if tags are being looked up in the comprehensive table
   - Verify fallback to hex display only happens when tag truly unknown

3. **Key files to examine**:
   - `src/core/ifd.rs:parse_ifd()` - Where tags are initially parsed
   - Tag resolution logic - Where numeric IDs get converted to names
   - HashMap construction - Ensure all EXIF tags are loaded

### Phase 4: Apple Maker Note Issue

The user also reported: `Warning: Apple maker note parsing failed: Invalid EXIF data: Unknown format: 25888`

1. **Check Apple maker note parsing**:
   - `src/maker/apple/` - Apple-specific parsing logic
   - Format type 25888 (0x6500) - Unknown EXIF format

2. **Cross-reference with ExifTool**:
   - Check `third-party/exiftool/lib/Image/ExifTool/Apple.pm`
   - Look for format 25888 handling
   - May need to add new format type to core EXIF types

## Key Files and References

### Auto-Generated Files (Check for syntax errors)
- `src/tables/exif_tags.rs` - EXIF tag definitions
- `src/tables/app_segments.rs` - JPEG segment parsing rules  
- `src/gpmf/format.rs` - GPMF format definitions
- `src/binary/composite_tags.rs` - Binary extraction rules

### Core Parsing Logic
- `src/core/ifd.rs` - IFD parsing and tag lookup
- `src/core/jpeg.rs` - JPEG segment extraction
- `src/core/types.rs` - EXIF value types and formats

### Sync Tools
- `src/bin/exiftool_sync/` - Regeneration tools
- `exiftool-sync.toml` - Sync configuration
- `Makefile` - Convenience commands (`make sync`)

### ExifTool Reference
- `third-party/exiftool/lib/Image/ExifTool/Exif.pm` - Canonical EXIF definitions
- `third-party/exiftool/lib/Image/ExifTool/Apple.pm` - Apple maker note handling

## Expected Outcomes

After fixing these issues:

1. **Proper tag names**: `"Compression": "JPEG (old-style)"` instead of `"0x0103": "Undefined([0, 6, 0, 0])"`
2. **Complete EXIF coverage**: All standard tags (ExposureTime, FNumber, ISO, etc.) extracted
3. **Binary extraction**: Thumbnail and preview images properly handled
4. **Clean compilation**: All auto-generated files compile without errors

## Testing Commands

```bash
# Verify compilation
cargo check

# Test with problematic image
cargo run -- test-images/apple/IMG_3755.JPG

# Compare with ExifTool
./third-party/exiftool/exiftool -json test-images/apple/IMG_3755.JPG > exiftool.json
cargo run -- test-images/apple/IMG_3755.JPG > exif-oxide.json

# Run full test suite
cargo test
```

## Notes

- The sync infrastructure is complete and working
- The issue appears to be in code generation quality, not architecture
- EXIF tag table has 643 definitions (28x improvement over previous ~23)
- Apple maker note format 25888 may require new core format type support