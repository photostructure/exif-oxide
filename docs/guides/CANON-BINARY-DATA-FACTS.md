# Canon Binary Data Technical Facts from try1 Investigation

This document captures the hard technical facts about Canon's binary data structures discovered during try1's Canon lens investigation. These facts are implementation-agnostic and based on ExifTool source analysis.

## Canon CameraSettings Binary Structure

### Location and Storage
- **Tag**: 0x0001 in Canon MakerNote
- **Format**: Binary data processed via ProcessBinaryData
- **Source**: `third-party/exiftool/lib/Image/ExifTool/Canon.pm` line 2166

### Binary Table Definition (from Canon.pm)
```perl
%Image::ExifTool::Canon::CameraSettings = (
    %binaryDataAttrs,  # Inherits PROCESS_PROC => \&ProcessBinaryData
    FORMAT => 'int16s',     # Default format for table
    FIRST_ENTRY => 1,       # Entries start at index 1, not 0
    22 => {                 # Offset 22 (0x16 hex)
        Name => 'LensType',
        Format => 'int16u',  # Override table default (int16s → int16u)
        PrintConv => \%canonLensTypes,  # Reference to 524-entry lookup table
    },
    # ... ~140 other fields at various offsets
);
```

### LensType Field Details
- **Offset**: 22 bytes (0x16 hex) from start of binary data
- **Format**: unsigned 16-bit integer (int16u)
- **Byte Order**: Follows main EXIF endianness (typically little-endian)
- **Special Value**: 61182 (0xEEFE) indicates "Canon RF lens"

### Raw Binary Data Example (from try1 testing)
```
ExifTool verbose output shows:
LensType = 61182
- Tag 0x0016 (2 bytes, int16u[1]):
    06d8: fe ee                    [..]
```
- Raw bytes: 0xFE 0xEE (little-endian)
- Interpreted value: 61182 decimal
- PrintConv result: "Canon RF 50mm F1.2L USM or other Canon RF Lens"

## Canon RF Lens System

### RFLensType Field
- **Value**: 289 (from try1 investigation)
- **Location**: Separate binary structure at offset 0x003d (tag unknown)
- **Result**: "Canon RF 400mm F2.8L IS USM"
- **Purpose**: Specific RF lens identification (more precise than LensType)

### RF Lens Detection Pattern
1. LensType value 61182 (0xEEFE) indicates generic "Canon RF lens"
2. RFLensType provides specific lens model identification
3. Both fields required for complete RF lens information

## Canon Binary Data Processing Requirements

### ProcessBinaryData Pattern
1. **Extract binary blob** from Canon MakerNote tag 0x0001
2. **Apply binary table structure** to parse individual fields
3. **Handle format overrides** (table default vs field-specific format)
4. **Apply PrintConv conversions** to get human-readable values

### Offset Calculation
- All offsets relative to start of binary data in tag 0x0001
- FIRST_ENTRY=1 means offset 0 is unused, offsets start at 1
- LensType at offset 22 means: binary_data[22] and binary_data[23] (2 bytes)

### Format Handling
- Table default: `int16s` (signed 16-bit)
- LensType override: `int16u` (unsigned 16-bit)
- Must check for field-specific format overrides

## Canon MakerNote Structure Facts

### Canon MakerNote Format
- Uses standard IFD structure (same as main EXIF)
- No complex header or signature required
- Same byte order as main EXIF data
- May have 8-byte footer with offset information

### Tag Identification Issues (from try1 debugging)
- try1 investigation found tag 0x0001 contained ASCII "6", not binary data
- Tag 0x000d contained large binary data
- **Critical Finding**: Need to identify which tag actually contains CameraSettings data
- ExifTool verbose output shows correct binary data extraction, but tag mapping unclear

### Manufacturer Detection
```rust
// Pattern from try1
let manufacturer = Manufacturer::from_make("Canon");
// Triggers Canon-specific binary data processing
```

## Canon Lens Type Lookup Table

### canonLensTypes Table Facts
- **Size**: 524 entries (from try1 extraction attempts)
- **Format**: Hash lookup table (LensType value → human-readable string)
- **Source**: ExifTool Canon.pm contains full lookup table
- **Example Mapping**: 61182 → "Canon RF 50mm F1.2L USM or other Canon RF Lens"

### PrintConv Processing
- Raw LensType value (61182) looked up in canonLensTypes hash
- Returns human-readable lens description
- Critical for user-facing lens identification

## Binary Data Extraction Challenges

### try1 Implementation Issues
1. **Tag Identification**: Unclear which tag contains CameraSettings binary data
2. **Format Mapping**: Generated tables had incorrect format enum names
3. **Offset Calculations**: Negative offsets and compilation errors in generated code
4. **Tag Overflow**: Canon 0x4xxx tags were being skipped due to overflow protection

### ExifTool Source Analysis Requirements
- Must study Canon.pm ProcessBinaryData implementation
- Understand binary table inheritance from %binaryDataAttrs
- Map ExifTool format names to Rust enum variants
- Handle FIRST_ENTRY offset adjustments

## Key Implementation Requirements

### Binary Data Processing Pattern
1. **Detect Canon MakerNote** via Make field = "Canon"
2. **Locate CameraSettings tag** (investigation needed - not necessarily 0x0001)
3. **Extract binary blob** from tag value
4. **Apply CameraSettings binary table** with correct offsets
5. **Handle format overrides** for individual fields
6. **Apply PrintConv lookups** for human-readable output

### Critical Validation Points
- Binary data length must be sufficient for offset access
- Handle endianness consistently with main EXIF
- Validate LensType range (61182 for RF lenses)
- Graceful degradation if binary processing fails

### Format Enum Mapping (from try1 investigation)
- ExifTool `int16u` → Rust `U16`
- ExifTool `int16s` → Rust `I16`
- ExifTool `int32u` → Rust `U32`
- ExifTool `string` → Rust `Ascii`

## Other Canon Binary Structures

### Additional Binary Tables (from Canon.pm)
- **ShotInfo**: Various shot information
- **AFInfo**: Autofocus point data
- **CustomFunctions**: Camera customization settings
- **LensInfo**: Additional lens information

### Processing Requirements
- Each binary structure has its own table definition
- Different default formats and offset schemes
- Some may have conditional processing based on camera model

## Testing Requirements

### Validation Against ExifTool
```bash
# Expected output for Canon R5 Mark II
exiftool -LensType -LensModel -RFLensType -json test.jpg
# Should show:
# "LensType": "Canon RF 50mm F1.2L USM or other Canon RF Lens"
# "RFLensType": "Canon RF 400mm F2.8L IS USM"
```

### Debug Information Needed
- Raw binary data hex dump
- Offset calculations
- Format interpretations
- PrintConv lookup results

## Key Learnings from try1 Failure

### What Didn't Work
- Auto-generating binary tables from ExifTool source
- Generic binary processing without Canon-specific handling
- Assuming tag 0x0001 contains CameraSettings without verification

### What's Required
- Manual implementation of Canon binary processing
- Direct ExifTool source reference for offset calculations
- Canon-specific tag identification and validation
- Manual PrintConv implementation with canonLensTypes lookup

## Source References

### ExifTool Files to Study
- `lib/Image/ExifTool/Canon.pm:2166+` - CameraSettings binary table
- `lib/Image/ExifTool/Canon.pm:xxx` - canonLensTypes lookup table
- `lib/Image/ExifTool.pm:6000+` - ProcessBinaryData implementation

### Critical Code Locations (from try1)
- Canon parser: `src/maker/canon.rs`
- Binary processing: `src/core/binary_data.rs`
- PrintConv system: Various generated tables

## Conclusion

Canon lens extraction requires proper ProcessBinaryData implementation with:
1. Correct binary tag identification (not necessarily 0x0001)
2. Manual binary table processing following ExifTool's exact structure
3. canonLensTypes lookup table for human-readable output
4. RF lens special handling (LensType + RFLensType combination)

The try1 automated approach failed because Canon's binary structures have too many manufacturer-specific quirks for automatic extraction. Manual implementation with ExifTool source references is required.