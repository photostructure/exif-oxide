# ExifTool Technical Facts from try1 Investigation

This document captures the hard technical facts about ExifTool's behavior and file format details discovered during the try1 investigation. These facts are valuable regardless of implementation approach.

## Critical ExifTool Behavior Facts

### JPEG Segment Structure

**APP1 Segment Limits**:

- APP1 segments are limited to 64KB (65533 bytes after length field)
- Segment length includes the length bytes themselves (subtract 2 for actual data size)
- EXIF data starts 6 bytes after "Exif\0\0" marker in APP1 segment
- Multiple APP1 segments possible (EXIF + XMP can coexist)

**JPEG Marker Handling**:

- Markers can be padded with unlimited 0xFF bytes
- Must check for "Exif\0\0" signature, not just APP1 presence
- 0xD9 = End of Image, 0xDA = Start of Scan (no more metadata after SOS)
- JPEG files often have padding after EOI marker - search last 32 bytes for validation

### TIFF/IFD Structure Facts

**IFD Parsing Rules**:

- Next IFD offset of 0xFFFFFFFF means "no next IFD" (not 0x00000000)
- Value fits inline if `size × count ≤ 4 bytes`
- Offsets are ALWAYS relative to TIFF header start, not file start
- IFD1 (thumbnail) uses tags 0x0201 (ThumbnailOffset) and 0x0202 (ThumbnailLength)

**Endianness Detection**:

- "II" = little-endian (Intel), "MM" = big-endian (Motorola)
- Magic number is always 42 (0x002A little-endian, 0x2A00 big-endian)
- Endianness applies to ALL numeric values in the file

**Structural Tag Overrides**:

- Tag 0x8769 (ExifOffset) incorrectly defined as Ascii in ExifTool tables but must be parsed as U32
- Tag 0x8825 (GPSOffset) and 0x014A (SubIFDs) also require U32 format override
- These are "structural" tags that point to other IFD locations

### String Handling Edge Cases

**EXIF String Quirks**:

- EXIF strings are null-terminated BUT buffer may contain garbage after null terminator
- Some manufacturers pad with spaces instead of null bytes
- UTF-8 encoding not guaranteed - may need charset detection
- Always scan for null terminator or use full count length, don't assume clean strings

**Manufacturer-Specific String Handling**:

- Canon: Generally well-formed strings
- Nikon: May use different character encodings
- Sony: Some encrypted sections affect string parsing

### Binary Data Value Storage

**Inline vs Offset Storage**:

- Values ≤ 4 bytes: Stored inline in IFD entry
- Values > 4 bytes: Offset points to data elsewhere in file
- For inline values, unused bytes are zero-padded
- Rational values (8 bytes) always use offset storage

**Format Code Mapping** (from try1 testing):

```
1 = BYTE (U8)
2 = ASCII (null-terminated string)
3 = SHORT (U16)
4 = LONG (U32)
5 = RATIONAL (two U32: numerator/denominator)
6 = SBYTE (I8)
7 = UNDEFINED (raw bytes)
8 = SSHORT (I16)
9 = SLONG (I32)
10 = SRATIONAL (two I32: numerator/denominator)
11 = FLOAT (F32)
12 = DOUBLE (F64)
```

## Manufacturer-Specific Facts

### Canon MakerNote Structure

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

**Canon Offset Schemes**:

- Model-specific logic for 4/6/16/28 byte offset variants
- Uses TIFF footer validation and offset base adjustment
- Fallback mechanisms for offset calculation failures

### Binary Tag Extraction Facts

**Thumbnail Extraction**:

- Thumbnails typically stored in IFD1 (second IFD after IFD0)
- Use tags 0x0201 (ThumbnailOffset) + 0x0202 (ThumbnailLength)
- Offset calculation differs between JPEG and TIFF formats:
  - JPEG: Offset from TIFF header position (after "Exif\0\0")
  - TIFF/RAW: Offset from file start
- May have 12-byte header before actual JPEG data at offset

**JPEG Validation for Embedded Images**:

- Valid JPEG must start with SOI marker (0xFFD8)
- Valid JPEG should end with EOI marker (0xFFD9)
- Search for markers within extracted data, don't assume exact positioning
- Real-world files may have padding or structure variations

**Format Variations in Binary Tags** (discovered during testing):

- ThumbnailOffset may be stored as U32, U16, or even Undefined format
- Must use flexible parsing that handles multiple format types
- Little-endian default for Undefined data is common

## Container Format Facts

### PNG eXIf Chunks

- PNG uses length-prefixed chunks with CRC validation
- eXIf chunk contains raw TIFF/EXIF data (no "Exif\0\0" wrapper)
- Must validate PNG signature (8-byte header) before chunk parsing
- Stop parsing at IDAT (image data) chunks - no metadata after

### RIFF Containers (WebP, AVI)

- RIFF uses little-endian chunk sizes exclusively
- Chunks must be word-aligned (pad odd sizes)
- WebP has 4-byte padding before TIFF header in EXIF chunks
- WebP can contain both EXIF and XMP chunks

### QuickTime/MP4 Atoms

- QuickTime uses big-endian exclusively for all numeric values
- Atom sizes can be 32-bit or 64-bit (size=1 indicates 64-bit follows)
- Size 0 means "to end of file"
- ftyp atom contains brand codes for format validation
- Metadata typically in moov/meta or moov/udta atoms

## ProcessBinaryData Technical Facts

### Binary Data Table Structure (from ExifTool source analysis)

- Uses `%binaryDataAttrs` inheritance for PROCESS_PROC
- Default format specified at table level (e.g., `FORMAT => 'int16s'`)
- Individual entries can override format
- `FIRST_ENTRY` specifies starting offset index

### Canon CameraSettings Example (Canon.pm:2166):

```perl
%Image::ExifTool::Canon::CameraSettings = (
    %binaryDataAttrs,  # Inherits PROCESS_PROC => \&ProcessBinaryData
    FORMAT => 'int16s',
    FIRST_ENTRY => 1,
    22 => {            # Offset 22 (0x16)
        Name => 'LensType',
        Format => 'int16u',   # Override table default
        PrintConv => \%canonLensTypes,
    },
    # ... 140+ other fields
);
```

### Binary Data Processing Pattern

1. Extract raw binary data from maker note tag
2. Apply binary data table structure to parse individual fields
3. Each field has offset, format, and optional PrintConv
4. Field values become individual tags in output

## Error Handling and Edge Cases

### Bounds Checking Requirements

- ALWAYS bounds-check before reading (use `get()` not direct indexing)
- Validate offset + length ≤ file size before extraction
- Handle integer overflow in offset calculations
- Graceful degradation when data is malformed

### Real-World File Quirks

- Some cameras create files with incorrect IFD entry counts
- Maker note data may be encrypted or obfuscated
- Files may have circular IFD references (requires cycle detection)
- Zero-length values and missing required fields occur in practice

### Memory Considerations

- Large embedded images/videos in maker notes
- TIFF-based RAW files can be hundreds of MB
- Stream large binary data instead of loading into memory
- Memory-mapped files for large format support

## Performance Characteristics from try1 Testing

### Parsing Speed Benchmarks (from try1)

- JPEG: ~8-9 microseconds for basic EXIF extraction
- TIFF: ~5-6 microseconds (faster due to no segment search)
- PNG: ~7 microseconds
- Container formats: ~8-10 microseconds

### Memory Usage Patterns

- Metadata-only mode: ~64KB max for IFD chains
- Full file mode: Required for binary extraction
- String values: Variable length, need careful handling
- Static table lookups: ~40KB for tag definitions

## Critical Implementation Requirements

### Security Considerations

- Bounds checking for all offset calculations
- Resource limits for binary extraction sizes
- Timeout for parsing operations to prevent DoS
- Validation of extracted binary data (magic numbers, structure)

### ExifTool Compatibility Requirements

- Exact tag name matching (case-sensitive)
- Same numeric value interpretation
- Identical PrintConv output formatting
- Same error handling behavior (continue vs fail)

### Testing Requirements

- Must validate against ExifTool output byte-for-byte
- Test with ExifTool's own test images (t/images/)
- Cross-manufacturer compatibility testing
- Edge case handling (malformed files, unusual formats)

## Conclusion

These technical facts were validated through try1's extensive testing against ExifTool's behavior. They represent hard constraints that any ExifTool-compatible implementation must respect, regardless of architecture choices.

The key insight: ExifTool's behavior includes many manufacturer-specific quirks and edge cases that exist for good reasons. Any implementation must handle these same edge cases to achieve compatibility.
