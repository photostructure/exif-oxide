# Technical Project Plan: Olympus Required Tags Support

**Related Milestone:** P3-MILESTONE-17c-Olympus-RAW

## Project Overview

- **Goal:** Support all "required: true" tags in tag-metadata.json for Olympus JPEG and RAW files
- **Problem:** Current implementation extracts only ~50% of required tags for Olympus images due to infrastructure gaps and missing implementations

## Background & Context

- PhotoStructure requires 124 tags marked as "required: true" for proper functionality
- Olympus images contain these tags across multiple groups: EXIF, MakerNotes (including Equipment subdirectory), Composite, File
- Current blocking issues prevent extraction of critical manufacturer-specific tags

## Technical Foundation

### Key Codebases
- `src/exif/tags.rs` - Tag storage system (currently uses HashMap<u16, TagValue>)
- `src/exif/ifd.rs` - IFD parsing and Equipment tag resolution
- `src/processor_registry/dispatch.rs` - Olympus-specific dispatch rules
- `src/implementations/olympus/` - Olympus-specific implementations
- `src/generated/Olympus_pm/` - Generated code from ExifTool extraction

### Documentation
- `docs/todo/P3-MILESTONE-17c-Olympus-RAW.md` - Parent milestone tracking
- `docs/TRUST-EXIFTOOL.md` - Critical: We translate ExifTool logic exactly
- `third-party/exiftool/doc/modules/Olympus.md` - ExifTool Olympus module overview

### ExifTool References
- `third-party/exiftool/lib/Image/ExifTool/Olympus.pm` - Source of truth
- Equipment table at line ~1590
- Main table with 576+ tag definitions
- olympusLensTypes hash with 200+ lens definitions

## Work Completed

### Infrastructure
- ✅ MakerNotes IFD parsing fixed (was using binary processor)
- ✅ Equipment subdirectory discovery at tag 0x2010
- ✅ Equipment IFD parsing (extracts 25 entries)
- ✅ Equipment codegen integration (generates lookup functions)
- ✅ Basic EXIF tag extraction for standard tags

### Generated Code
- ✅ `equipment_tag_structure.rs` - Equipment tag name lookups
- ✅ `olympuslenstypes.rs` - Lens type database (200+ entries)
- ✅ `olympuscameratypes.rs` - Camera model lookups
- ✅ Various inline PrintConv tables for subdirectories

## Remaining Tasks

### 1. Fix Tag ID Conflict System (CRITICAL BLOCKER)
**Blocks:** ~20 required tags including CameraType2, SerialNumber, LensType

**Problem:** Current HashMap<u16, TagValue> storage prevents tags with same ID from different contexts
- Equipment 0x0100 (CameraType2) conflicts with EXIF 0x0100 (ImageWidth)
- Equipment 0x0101 (SerialNumber) conflicts with EXIF 0x0101 (ImageHeight)

**Implementation:**
```rust
// Change from:
extracted_tags: HashMap<u16, TagValue>
// To:
extracted_tags: HashMap<(String, u16), TagValue>  // (namespace, tag_id)
```

**Files to update:**
- `src/exif/tags.rs:16-55` - store_tag_with_precedence() method
- `src/exif/mod.rs:37` - extracted_tags field definition
- All code accessing extracted_tags (search for `.get(&tag_id)`)

### 2. Olympus MakerNotes Required Tags

**High Priority Equipment Tags:**
- `CameraID` (0x0209) - freq 0.068, required [BLOCKED by conflict system]
- `CameraType2` (0x0100) - Model name [BLOCKED by conflict system]  
- `SerialNumber` (0x0101) - Camera serial [BLOCKED by conflict system]
- `LensType` (0x0201) - freq 0.18, required [BLOCKED by conflict system]
- `DateTimeUTC` (0x1001) - freq 0.007, required

**Implementation Notes:**
- All Equipment tags already have codegen support
- Just need namespace-aware storage to access them

### 3. Standard EXIF Tags (Olympus writes these)

**Missing Required Tags:**
- `Copyright` (0x8298) - freq 0.20, required
- `Artist` (0x013b) - Empty but required field
- `ExifVersion` (0x9000) - Usually "0230"
- `SubSecTime` (0x9290) - freq 0.083, required
- `UserComment` (0x9286) - Often empty

**Implementation:** Add to EXIF tag definitions in generated code

### 4. Composite Tag Calculations

**High Priority:**
- `Aperture` - Calculate from FNumber (freq 0.85)
- `ShutterSpeed` - Format ExposureTime (freq 0.86)
- `LensID` - Lookup from LensType using olympusLensTypes (freq 0.20)
- `SubSecDateTimeOriginal` - Combine DateTimeOriginal + SubSecTime

**Implementation:** Add composite tag registry entries

### 5. File System Metadata

**Missing Tags:**
- `FilePermissions` - Use std::fs::metadata()
- `FileAccessDate` - Use metadata.accessed()
- `FileInodeChangeDate` - Unix-specific, use metadata.created()

### 6. GPS Tags (Lower frequency but required)

**Tags to implement:**
- `GPSLatitude/Longitude` - Convert from rational degrees
- `GPSAltitude` - Handle above/below sea level  
- `GPSDateTime` - Combine GPSDateStamp + GPSTimeStamp

## Prerequisites

- Complete namespace-aware tag storage implementation
- Fix codegen processing order (Main table before subdirectories)
- **Tag Kit Migration**: Complete [tag kit migration and retrofit](../done/20250723-tag-kit-migration-and-retrofit.md) for Olympus module
  - Olympus has inline_printconv config that needs migration
  - Ensures consistent extraction approach across all modules

## Testing Strategy

### Unit Tests
- Test Equipment tag extraction with known values
- Verify tag conflict resolution
- Test composite tag calculations

### Integration Tests
- Compare output with ExifTool using compare-with-exiftool tool
- Test both JPEG (C2000Z.jpg) and ORF (test.orf) files
- Verify all required tags present in output

### Test Commands
```bash
# Check Equipment tags extracted correctly
RUST_LOG=debug cargo run -- test-images/olympus/test.orf 2>&1 | grep -E "(CameraType2|SerialNumber|LensType)"

# Compare with ExifTool
cargo run --bin compare-with-exiftool test-images/olympus/test.orf

# Verify no tag conflicts
RUST_LOG=debug cargo run -- test-images/olympus/test.orf 2>&1 | grep "Ignoring.*priority"
```

## Success Criteria & Quality Gates

- All 124 required tags extractable from Olympus test images
- No tag ID conflicts between EXIF and MakerNotes
- Output matches ExifTool -j -struct -G format
- All existing tests continue to pass
- Performance remains within 10% of current baseline

## Gotchas & Tribal Knowledge

### Tag Conflict Architecture
- This issue affects ALL manufacturers, not just Olympus
- Canon, Nikon, Sony all have similar subdirectory conflicts
- Solution must be generic, not Olympus-specific

### Equipment Subdirectory
- Tag 0x2010 can be either offset (IFD) or binary data
- Must check format to determine processing method
- Equipment has WRITE_PROC => WriteExif (it's an IFD structure)

### Lens Type Complexities
- LensType combines manufacturer code + lens ID
- Some lenses share IDs across manufacturers
- Teleconverter detection affects lens identification

### Olympus-Specific Quirks
- MakerNotes start with "OLYMPUS\0II\x03\0"
- Some models use different MakerNote structures
- Art filters affect various tag interpretations

### Common Mistakes
- Don't assume tag names match between Main and Equipment tables
- Equipment BodyFirmwareVersion != Main FirmwareVersion  
- Some tags are duplicated with different meanings