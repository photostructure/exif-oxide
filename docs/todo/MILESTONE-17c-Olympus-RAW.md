# Milestone 17c: Olympus RAW Support

**Goal**: Implement Olympus ORF format leveraging existing RAW infrastructure and generated lookup tables  
**Status**: 90% Complete - Core infrastructure working, one architectural fix needed

## High Level Guidance

- **Follow Trust ExifTool**: Study how ExifTool processes ORF files exactly. See [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md)
- **Use Codegen**: Switch any manual lookup tables to generated code. See [EXIFTOOL-INTEGRATION.md](../design/EXIFTOOL-INTEGRATION.md)
- **Study ExifTool Sources**: [Olympus.pm](../../third-party/exiftool/lib/Image/ExifTool/Olympus.pm) and [module docs](../../third-party/exiftool/doc/modules/Olympus.md)

## ✅ Completed Infrastructure

All core Olympus processing infrastructure is implemented and working:

- **ORF Detection**: Added to RawFormat enum and file detection
- **Olympus Signature Detection**: Handles "OLYMPUS\0", "OLYMP\0", "OM SYSTEM\0" headers
- **FixFormat System**: Converts invalid Olympus TIFF formats to valid ones
- **Subdirectory Processing**: Equipment (0x2010), CameraSettings (0x2020), etc.
- **Offset Calculations**: Proper offset handling for Olympus MakerNotes
- **Equipment Processor**: Extracts CameraType2, SerialNumber, LensType
- **CLI Integration**: `cargo run -- file.orf` works

## ❌ Current Issue: ExifIFD Processing Architecture

**Root Cause**: ExifIFD is processed as binary data instead of standard IFD structure, preventing MakerNotes discovery.

**ExifTool's Approach** (from `exiftool -v3 test.orf`):
1. **ORF Detection**: `ProcessORF()` → `ProcessTIFF()` (delegates to standard TIFF)
2. **Standard IFD Parsing**: ExifIFD parsed as IFD structure with 31 entries
3. **MakerNotes Found**: `Tag 0x927c (1462284 bytes, undef[1462284])` in ExifIFD
4. **Signature Detection**: `MakerNoteOlympus2` pattern matches "OLYMPUS\0"
5. **Equipment Success**: Extracts CameraType2="E-M1", SerialNumber="BHP242330", etc.

**Our Current Issue**:
- ✅ IFD0 processing works correctly
- ✅ ExifIFD subdirectory found at offset 0x12e
- ❌ ExifIFD sent to `Olympus::CameraSettings` processor (wrong)
- ❌ No individual tag parsing - never finds tag 0x927C (MakerNotes)

## 🔧 Tasks Remaining

### 1. Fix ExifIFD Processor Selection (2-3 hours)

**Problem**: `src/exif/processors.rs` routes ExifIFD to manufacturer processors instead of standard IFD parsing.

**Debug Evidence**:
```bash
RUST_LOG=debug cargo run -- test-images/olympus/test.orf 2>&1 | grep "Selected processor.*ExifIFD"
# Shows: "Selected processor Olympus::CameraSettings for directory ExifIFD"
# Should: Parse ExifIFD as standard IFD to find individual tags
```

**Fix**: Ensure ExifIFD uses standard TIFF IFD processing, not manufacturer-specific processors.

**Files to Study**:
- `src/exif/processors.rs` - Fix processor selection logic
- `src/exif/ifd.rs` - Standard IFD parsing implementation
- Study how IFD0 processing works (which correctly finds individual tags)

### 2. Verify MakerNotes Processing (1 hour)

After ExifIFD fix, verify:
- Tag 0x927C found during standard IFD parsing
- Olympus signature detection triggers correctly
- Equipment subdirectory processing works
- Final tags show proper names: CameraType2, SerialNumber, LensType

### 3. Add ORF to Compatibility Testing (15 minutes)

Update `tools/generate_exiftool_json.sh` line 24:
```bash
SUPPORTED_EXTENSIONS=("jpg" "jpeg" "orf" "nef" "cr3" "arw" "rw2")
```

## 🧠 Tribal Knowledge

### Olympus Format Quirks
- **Invalid TIFF Formats**: Olympus violates TIFF spec with wrong format types (solved by FixFormat)
- **Signature Headers**: MakerNotes have manufacturer headers that must be detected and skipped
- **Offset Calculations**: Subdirectory offsets relative to original MakerNotes position
- **Dual Processing**: Equipment can be binary data OR IFD pointer depending on format

### ExifTool Processing Pattern
- **ProcessORF()**: Just delegates to `ProcessTIFF()` - no special ORF preprocessing
- **Standard Pipeline**: ORF files use standard TIFF processing to find MakerNotes
- **MakerNotes Detection**: Found via standard IFD parsing, then signature detection routes to Olympus tables

### Debug Tips
- **Tag 0x927C**: MakerNotes tag, should be found in ExifIFD during standard parsing
- **Entry Count**: If you see 12336 (0x3030), you're reading tag data as IFD header
- **Signature Detection**: Look for "OLYMPUS\0" in MakerNotes data
- **Equipment Processing**: Should show 25 entries, not 24,064

## 📋 Success Criteria

1. **ExifIFD Standard Processing**: Parsed as IFD structure showing individual tags
2. **Tag 0x927C Detection**: MakerNotes found during ExifIFD parsing
3. **Equipment Tag Names**: Show CameraType2, SerialNumber, LensType (not Tag_XXXX)
4. **ExifTool Compatibility**: Match `exiftool -j test-images/olympus/test.orf` output
5. **Compatibility Tests**: `make compat-test` passes for ORF files

## 🔍 Key References

### ExifTool Sources
- **Olympus.pm**: Main Olympus module with tag tables and processing logic
- **MakerNotes.pm**: Signature detection patterns (lines 557-589)
- **ProcessORF()**: Simple delegation to ProcessTIFF (Olympus.pm:4179)

### Our Implementation
- **Signature Detection**: `src/implementations/olympus.rs` - working correctly
- **FixFormat System**: `src/tiff_types.rs` - handles invalid TIFF formats
- **Equipment Processor**: `src/processor_registry/processors/olympus.rs` - working correctly
- **Equipment Tags**: `src/implementations/olympus/equipment_tags.rs` - tag name mappings

### Debug Commands
```bash
# Current issue - should show standard IFD parsing
RUST_LOG=debug cargo run -- test-images/olympus/test.orf 2>&1 | grep -E "(ExifIFD.*entries|Tag 0x927c)"

# Expected Equipment tags
exiftool -j test-images/olympus/test.orf | jq -r 'keys[]' | grep -E "(CameraType2|SerialNumber|LensType)"

# Compatibility testing
make compat-gen && make compat-test
```

## Estimated Completion Time

- **2-3 hours**: Fix ExifIFD processing architecture
- **1 hour**: Verify and test Equipment tag extraction
- **Total**: 3-4 hours to complete milestone

The core Olympus infrastructure is solid - this is purely an architectural routing fix to ensure ExifIFD gets standard IFD parsing instead of manufacturer-specific processing.