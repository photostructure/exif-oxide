# TODO: Fix File Type Detection System

## Summary

**STATUS**: Codegen extraction is working correctly, but the main file type detection system is not using the generated lookup tables properly.

`make compat` is finding 20 MIME type compatibility issues. The extraction and code generation pipeline is working correctly and producing the right data, but the main library's file type detection logic needs to be updated to use the generated tables.

## ✅ RESOLVED: Codegen Extraction Issues

~~The codegen extraction is now working correctly:~~

- ✅ `%magicNumber` extracted successfully (110 patterns in `src/generated/file_types/magic_number_patterns.rs`)
- ✅ `%fileTypeLookup` extracted successfully (343 lookups in `src/generated/file_types/file_type_lookup.rs`)

**Note**: The warnings about "Skipping %magicNumber" and "Skipping %fileTypeLookup" are spurious. These are handled by specialized extractors (`codegen/extractors/regex_patterns.pl` and `codegen/extractors/file_type_lookup.pl`) rather than the simple table extraction system, so the warnings are misleading.

## Background

The file type lookup system extracts data from ExifTool's `%fileTypeLookup` and `%magicNumber` hashes. This system provides:

- Magic number patterns for content-based detection
- Extension to file type mappings
- Alias resolution (e.g., 3GP2 → 3G2)
- Format descriptions
- Multiple format support for extensions

## Current Test Failures (20 issues)

From latest `make compat` run, these MIME types have detection issues:

### XMP Files (10 files - all failing to detect `application/rdf+xml`)

- [ ] `third-party/exiftool/t/images/XMP.xmp`
- [ ] `third-party/exiftool/t/images/XMP2.xmp`
- [ ] `third-party/exiftool/t/images/XMP3.xmp`
- [ ] `third-party/exiftool/t/images/XMP4.xmp`
- [ ] `third-party/exiftool/t/images/XMP5.xmp`
- [ ] `third-party/exiftool/t/images/XMP6.xmp`
- [ ] `third-party/exiftool/t/images/XMP7.xmp`
- [ ] `third-party/exiftool/t/images/XMP8.xmp`
- [ ] `third-party/exiftool/t/images/XMP9.xmp`
- [ ] `third-party/exiftool/t/images/PLUS.xmp`

### Video Files (4 files)

- [ ] `test-images/gopro/jump.mp4` - should detect `video/mp4`
- [ ] `third-party/exiftool/t/images/Flash.flv` - should detect `video/x-flv`
- [ ] `third-party/exiftool/t/images/M2TS.mts` - should detect `video/m2ts`
- [ ] `third-party/exiftool/t/images/ASF.wmv` - should detect `video/x-ms-wmv` (detecting `image/jpeg`)

### Image Files (4 files - misdetected as TIFF)

- [ ] `third-party/exiftool/t/images/CanonRaw.cr3` - should detect `image/x-canon-cr3` (detecting `image/tiff`)
- [ ] `test-images/canon/canon_eos_r50v_01.cr3` - should detect `image/x-canon-cr3` (detecting `image/tiff`)
- [ ] `third-party/exiftool/t/images/Jpeg2000.jp2` - should detect `image/jp2` (detecting `image/tiff`)
- [ ] `third-party/exiftool/t/images/Jpeg2000.j2c` - should detect `image/x-j2c`

### Other Files (2 files)

- [ ] `third-party/exiftool/t/images/Photoshop.psd` - should detect `application/vnd.adobe.photoshop`
- [ ] `third-party/exiftool/t/images/PostScript.eps` - should detect `application/postscript`

## Files to Study

### 1. Generated Files (✅ Working Correctly)

- **`src/generated/file_types/magic_number_patterns.rs`** - Contains 110 magic number patterns from ExifTool (✅ generated correctly)
- **`src/generated/file_types/file_type_lookup.rs`** - Contains 343 file type lookups and extension mappings (✅ generated correctly)  
- **`src/generated/file_types/mime_types.rs`** - MIME type mappings (✅ generated)

### 2. Main Library File Detection (🔍 Needs Investigation)

The core issue is likely in how the main library uses the generated data:

- **Main file type detection logic** - Need to find where MIME types are determined in the main library
- **Integration with generated tables** - The generated data exists but may not be properly integrated
- **Magic number pattern matching** - Check if magic number patterns from `magic_number_patterns.rs` are being used
- **Extension-based fallback** - Check if extension-based detection is working correctly

### 3. Test Files  

- **`tests/mime_type_compatibility_tests.rs`** - The failing compatibility tests (lines around 498 show failure)
- **File samples** - Use the specific failing files listed above for testing

### 4. ExifTool Source (Reference)

- **`third-party/exiftool/lib/Image/ExifTool.pm`** - `%fileTypeLookup` hash around line 258-404, `%magicNumber` hash around line 912-1027
- **`third-party/exiftool/doc/concepts/FILE_TYPES.md`** - Comprehensive documentation on ExifTool's file detection

### 5. Specialized Extractors (✅ Working Correctly)

- **`codegen/extractors/regex_patterns.pl`** - Extracts `%magicNumber` hash (✅ working)
- **`codegen/extractors/file_type_lookup.pl`** - Extracts `%fileTypeLookup` hash (✅ working)
- **`codegen/Makefile.modular`** - Lines 70-76 show how specialized extractors are orchestrated

## Investigation Steps

### 1. Find Main File Type Detection Logic

The main library must have logic that determines MIME types. Find where this happens:

```bash
# Look for MIME type detection in main library
grep -r "mime.*type\|MIME.*Type" src/ --exclude-dir=generated
grep -r "FileTypeDetector\|file.*type.*detect" src/ --exclude-dir=generated
grep -r "magic.*number\|header.*detect" src/ --exclude-dir=generated
```

### 2. Check Integration with Generated Data

Verify if the main library is importing and using the generated lookup tables:

```bash
# Check if generated modules are imported
grep -r "use.*file_types::" src/ --exclude-dir=generated
grep -r "magic_number_patterns\|file_type_lookup" src/ --exclude-dir=generated
```

### 3. Test Specific Failing Cases

Use the specific failing files to test patterns:

```bash
# Test ExifTool detection on specific failing files
exiftool -MIMEType third-party/exiftool/t/images/XMP.xmp
exiftool -MIMEType test-images/gopro/jump.mp4
exiftool -MIMEType third-party/exiftool/t/images/CanonRaw.cr3
```

## Root Cause Analysis

### Likely Issues

1. **Missing Integration**: Main library file detection logic doesn't use generated lookup tables
2. **Priority Order**: Extension-based detection may be overriding magic number detection incorrectly  
3. **Pattern Matching**: Magic number patterns may not be compiled into usable regex engines
4. **MIME Type Mapping**: Generated data may not include proper MIME type mappings

### Evidence  

- ✅ Extraction working: Both specialized extractors run successfully
- ✅ Generation working: Generated files contain correct data (110 patterns, 343 lookups)
- ❌ Integration failing: 20 MIME type compatibility failures in `make compat`

## How to Fix

### Option 1: Find and Fix Main File Detection Logic

1. **Locate main detection code** - Find where the library determines file types and MIME types
2. **Check integration** - Verify it imports and uses the generated lookup tables  
3. **Add missing integration** - If not integrated, add imports and usage of generated data
4. **Test incrementally** - Fix one category at a time (e.g., start with XMP files)

### Option 2: Fix Magic Number Pattern Usage

1. **Check pattern compilation** - Verify magic number patterns are compiled into working regex engines
2. **Test pattern matching** - Ensure patterns match file headers correctly
3. **Check priority** - Magic number detection should typically override extension-based detection

### Option 3: Fix MIME Type Mapping  

1. **Check MIME type generation** - Verify `mime_types.rs` contains correct mappings
2. **Check MIME type lookup** - Ensure detected file types are properly mapped to MIME types
3. **Add missing MIME types** - Some file types may need additional MIME type mappings

## Testing

After making changes:

```bash
# Run the specific MIME type compatibility tests
make test-mime-compat

# Full compatibility suite
make compat
```

## Important Notes

1. **DO NOT manually edit generated files** in `src/generated/`
2. **Trust ExifTool** - If ExifTool doesn't define something, we shouldn't add it
3. **The codegen is working** - Focus on main library integration, not extraction
4. **Use git diff** to see what changed after regenerating

## Related Context  

The codegen system recently went through a major refactoring (Milestone 16) where "simple_tables" was renamed to "extract". The file type lookup is part of the file detection module in the new modular architecture.

**Key Discovery**: The warnings about "Skipping %magicNumber" and "Skipping %fileTypeLookup" can be ignored - they're handled by specialized extractors, not the simple table system.

## Getting Help

If you get stuck:

1. Check existing documentation in `docs/`
2. Use `git grep` to find similar patterns in the codebase  
3. The `TRUST-EXIFTOOL.md` principle is paramount - when in doubt, match ExifTool exactly
4. See `third-party/exiftool/doc/concepts/FILE_TYPES.md` for ExifTool's file detection deep dive
