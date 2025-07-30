# Manufacturer-Specific Facts and Quirks

**🚨 CRITICAL: All manufacturer quirks follow [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - we implement exactly what ExifTool does, even if it seems wrong.**

This document captures manufacturer-specific technical facts, quirks, and implementation requirements discovered through ExifTool analysis and real-world testing. Every quirk exists to handle specific camera firmware bugs or non-standard behaviors.

> **Last Verified**: 2025-07-10 against ExifTool source (version 12.93+)

## Section 1: Canon

> **Source:** Consolidated from Canon binary data investigation and ExifTool Canon.pm analysis

### 1.1 Canon MakerNote Structure

**Canon Binary Data Facts**:

- Canon uses standard IFD structure for maker notes (no complex header)
- Uses same byte order as main EXIF data
- May have 8-byte footer with offset information
- Canon CameraSettings stored in tag 0x0001 as binary data

**Canon ProcessBinaryData Structure** (from Canon.pm analysis):

- CameraSettings uses format `int16s` with `FIRST_ENTRY => 1`
- LensType is at offset 22 (0x16) as `int16u` format
- LensType value 61182 (0xEEFE) indicates "Canon RF lens"
- Binary structure contains ~140 fields with camera settings

### 1.2 Canon CameraSettings Binary Structure

#### Location and Storage
- **Tag**: 0x0001 in Canon MakerNote
- **Format**: Binary data processed via ProcessBinaryData
- **Source**: `third-party/exiftool/lib/Image/ExifTool/Canon.pm` lines 2166-2869

#### Binary Table Definition (from Canon.pm)
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

#### LensType Field Details
- **Offset**: 22 bytes (0x16 hex) from start of binary data
- **Format**: unsigned 16-bit integer (int16u)
- **Byte Order**: Follows main EXIF endianness (typically little-endian)
- **Special Value**: 61182 (0xEEFE) indicates "Canon RF lens"

#### Raw Binary Data Example
```
ExifTool verbose output shows:
LensType = 61182
- Tag 0x0016 (2 bytes, int16u[1]):
    06d8: fe ee                    [..]
```
- Raw bytes: 0xFE 0xEE (little-endian)
- Interpreted value: 61182 decimal
- PrintConv result: "Canon RF 50mm F1.2L USM or other Canon RF Lens"

### 1.3 Canon RF Lens System

#### RFLensType Field
- **Value**: 289 (from investigation)
- **Location**: Separate binary structure at offset 0x003d (tag unknown)
- **Result**: "Canon RF 400mm F2.8L IS USM"
- **Purpose**: Specific RF lens identification (more precise than LensType)

#### RF Lens Detection Pattern
1. LensType value 61182 (0xEEFE) indicates generic "Canon RF lens"
2. RFLensType provides specific lens model identification
3. Both fields required for complete RF lens information

### 1.4 Canon Offset Schemes

**Canon Offset Schemes**:

- Model-specific logic for 4/6/16/28 byte offset variants
- Uses TIFF footer validation and offset base adjustment
- Fallback mechanisms for offset calculation failures

**Canon-specific quirks** (from ExifTool source):

- Uses 0xdeadbeef as a sentinel value
- Different offset schemes based on camera model (4, 6, 16, or 28 bytes)
- Some models have TIFF footer validation

### 1.5 Canon Binary Data Processing Requirements

#### ProcessBinaryData Pattern
1. **Extract binary blob** from Canon MakerNote tag 0x0001
2. **Apply binary table structure** to parse individual fields
3. **Handle format overrides** (table default vs field-specific format)
4. **Apply PrintConv conversions** to get human-readable values

#### Offset Calculation
- All offsets relative to start of binary data in tag 0x0001
- FIRST_ENTRY=1 means offset 0 is unused, offsets start at 1
- LensType at offset 22 means: binary_data[22] and binary_data[23] (2 bytes)

#### Format Handling
- Table default: `int16s` (signed 16-bit)
- LensType override: `int16u` (unsigned 16-bit)
- Must check for field-specific format overrides

### 1.6 Canon Lens Type Lookup Table

#### canonLensTypes Table Facts
- **Size**: 534 entries (as of ExifTool 12.93+)
- **Format**: Hash lookup table (LensType value → human-readable string)
- **Source**: ExifTool Canon.pm lines 97-643 contains full lookup table
- **Example Mapping**: 61182 → "Canon RF 50mm F1.2L USM or other Canon RF Lens"
- **RF Lens Entries**: 65 sub-entries under 61182 (61182.1 through 61182.64)

#### PrintConv Processing
- Raw LensType value (61182) looked up in canonLensTypes hash
- Returns human-readable lens description
- Critical for user-facing lens identification

### 1.7 Additional Canon Binary Structures

#### Additional Binary Tables (from Canon.pm)
- **ShotInfo**: Various shot information
- **AFInfo**: Autofocus point data
- **CustomFunctions**: Camera customization settings
- **LensInfo**: Additional lens information

#### Processing Requirements
- Each binary structure has its own table definition
- Different default formats and offset schemes
- Some may have conditional processing based on camera model

### 1.8 Byte Order Weirdness

```perl
# Canon.pm:1161 - Focus distance with odd byte ordering
my %focusDistanceByteSwap = (
    # this is very odd (little-endian number on odd boundary),
    # but it does seem to work better with my sample images - PH
    Format => 'int16uRev',
    # ...
);
```

This reveals Canon's inconsistent byte ordering in some fields, requiring special handling.

## Section 2: Nikon

> **Planning:** Content to be consolidated when Nikon support is implemented

### 2.1 Nikon MakerNote Structure

**Nikon-specific quirks**:

- TIFF header at offset 0x0a in maker notes
- Multiple encryption schemes
- Format changes between firmware versions

### 2.2 NEF vs NRW File Type Detection

**NEF/NRW File Type Strategy** (updated 2025-07-28):

ExifTool uses a complex multi-stage detection process:

1. **Initial Detection**: Based on file extension (.nef or .nrw)
2. **IFD0 Override**: If NEF file has JPEG compression (value 6) in IFD0 → change to NRW
3. **MakerNotes Override**: If NRW file has NEFLinearizationTable (tag 0x0096) → change back to NEF

**Key Facts**:
- Both NEF and NRW use standard TIFF structure
- IFD0 compression alone is NOT sufficient to distinguish them
- NEF files can have JPEG compression in IFD0 (for thumbnails)
- The main image is usually in SubIFD1 with compression 34713 (Nikon NEF Compressed)
- NEFLinearizationTable is located in MakerNotes, not IFD0

**exif-oxide Implementation**:
We trust file extensions for NEF/NRW distinction. This design choice:
- Provides predictable, documented behavior
- Avoids false positives from incomplete content analysis
- Eliminates the complexity of parsing MakerNotes during file detection
- Matches common industry practice of trusting extensions for initial type detection

**Rationale**:
- ExifTool's complete detection requires parsing MakerNotes, which adds significant complexity
- Many NEF files have JPEG compression in IFD0 (for thumbnails) but are still NEF files
- Without checking NEFLinearizationTable in MakerNotes, content-based detection produces false positives
- File extensions are reliable for camera-generated files

**References**:
- ExifTool Exif.pm:1138-1141 (NRW detection from JPEG compression)
- ExifTool.pm:8672-8674 (NEF recovery from NEFLinearizationTable)

### 2.3 Nikon Encryption

**Nikon Encryption Facts**:

- Multiple encryption schemes across different camera models
- Key derivation based on camera serial number and other factors
- Some sections encrypted, others in plaintext
- Required for full metadata extraction on newer models

## Section 3: Sony

### 3.1 Sony MakerNote Structure

**Sony-specific quirks**:

- Seven different maker note detection patterns
- Some models double-encode UTF-8
- Encryption on newer models

### 3.2 Sony Data Processing

**Sony-specific quirks**:
- Multiple maker note formats requiring detection
- Encrypted data on newer models
- Various UTF-8 handling requirements

## Section 4: Samsung

### 4.1 Samsung Quirks

**Samsung-specific handling**:

- Encrypted tone curve data (tags 0xa040-0xa043)
- Count values of 23 for tone curve entries
- Various model-specific processing requirements

## Section 5: Other Manufacturers

### 5.1 Leica

**Leica Format Variations**:

- 9 different maker note formats!
- Complex base offset calculations
- Model-specific detection required

### 5.2 General Manufacturer Patterns

#### The "n/a" vs undef Pattern

Many PrintConv definitions use `'n/a'` strings instead of undef for missing values:

```perl
0x7fff => 'n/a',    # Canon.pm:2625
0xffff => 'n/a',    # Canon.pm:6520
```

This suggests camera firmware explicitly sets these "not available" sentinel values rather than leaving fields empty.

#### "Magic" Constants

When you see unexplained constants in ExifTool:

```perl
$offset += 0x1a;  # What is 0x1a?
```

These are usually manufacturer-specific quirks discovered through reverse engineering. Document them but don't change them:

```rust
// Add 0x1a to offset for Canon 5D
// ExifTool: Canon.pm:1234 (no explanation given)
offset += 0x1a;
```

## Implementation Guidelines

### For Each Manufacturer

When implementing manufacturer support:

1. **Study ExifTool source** - Understand their specific quirks
2. **Start with detection** - Identify the manufacturer reliably
3. **Implement binary processors** - Handle their specific data formats
4. **Add PrintConv/ValueConv** - Provide human-readable output
5. **Test extensively** - Use real camera files

### Critical Validation Points

- Binary data length must be sufficient for offset access
- Handle endianness consistently with main EXIF
- Validate value ranges for known constants
- Graceful degradation if processing fails

### Key Implementation Requirements

#### Binary Data Processing Pattern
1. **Detect Manufacturer** via Make field
2. **Locate specific tags** (investigation needed for each manufacturer)
3. **Extract binary blob** from tag value
4. **Apply manufacturer binary tables** with correct offsets
5. **Handle format overrides** for individual fields
6. **Apply PrintConv lookups** for human-readable output

#### Format Enum Mapping
- ExifTool `int16u` → Rust `U16`
- ExifTool `int16s` → Rust `I16`
- ExifTool `int32u` → Rust `U32`
- ExifTool `string` → Rust `Ascii`

### Testing Requirements

#### Validation Against ExifTool
```bash
# Expected output for specific camera models
exiftool -LensType -LensModel -json test.jpg
# Should show manufacturer-specific tag values
```

#### Debug Information Needed
- Raw binary data hex dump
- Offset calculations
- Format interpretations
- PrintConv lookup results

## Remember the Prime Directive

> **From [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md):** These quirks exist for camera-specific reasons. Don't "fix" seemingly odd behavior.

When you encounter something that seems wrong:

1. **Don't fix it** - It's probably correct for some camera
2. **Document it** - Add comments with ExifTool references
3. **Test it** - Make sure your implementation matches ExifTool
4. **Ask why** - But accept that sometimes nobody knows

The metadata world is messy because cameras are messy. Embrace the chaos!

## Source References

### ExifTool Files to Study by Manufacturer

#### Canon
- `lib/Image/ExifTool/Canon.pm:2166-2869` - CameraSettings binary table
- `lib/Image/ExifTool/Canon.pm:97-643` - canonLensTypes lookup table (534 entries)
- `lib/Image/ExifTool.pm:9750+` - ProcessBinaryData implementation

#### Nikon
- `lib/Image/ExifTool/Nikon.pm` - Nikon-specific processing
- Encryption handling sections

#### Sony
- `lib/Image/ExifTool/Sony.pm` - Sony-specific processing
- UTF-8 encoding handling

### General
- `lib/Image/ExifTool/MakerNotes.pm` - Central dispatcher
- `lib/Image/ExifTool.pm` - Core processing logic

## Verification Summary

### Confirmed Facts (Verified 2025-07-10)
✅ Canon CameraSettings structure at Canon.pm:2166 with FORMAT='int16s', FIRST_ENTRY=1
✅ Canon LensType at offset 22, format int16u, PrintConv to canonLensTypes
✅ Canon RF lens special value 61182 (0xEEFE) with 65 sub-entries for specific models
✅ canonLensTypes table contains 534 entries (not 524 as originally stated)
✅ Canon uses 0xdeadbeef (decimal -559038737) as sentinel value for 'n/a'
✅ focusDistanceByteSwap with Format 'int16uRev' for odd byte boundary handling

### Unverified/Updated Claims
❓ Samsung NX200 entry count bug (23 vs 21) - not found in source
❓ Sony double UTF-8 encoding at XMP.pm:4567 - line doesn't contain this code
✏️ Updated canonLensTypes count from 524 to 534 entries
✏️ Added specific line ranges for ExifTool source references

## Conclusion

Each manufacturer has evolved their own quirks over decades of firmware development. ExifTool has painstakingly documented and handled each of these through reverse engineering and community contributions. Our job is to faithfully translate this accumulated knowledge to Rust, preserving every quirk and edge case.