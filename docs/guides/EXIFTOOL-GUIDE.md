# Complete ExifTool Guide for exif-oxide

**🚨 CRITICAL: Read [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) first - it's the fundamental law of this project.**

This guide helps you understand and work with ExifTool source code when implementing features in exif-oxide. Every section assumes you've internalized the "Trust ExifTool" principle.

## Section 1: Essential ExifTool Concepts

> **Remember:** We translate ExifTool exactly, we don't innovate. See [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md).

### Essential Reading

Before diving into code, read these essential ExifTool documentation files:

- [PROCESS_PROC.md](../../third-party/exiftool/doc/concepts/PROCESS_PROC.md) - Processing procedures explained
- [VALUE_CONV.md](../../third-party/exiftool/doc/concepts/VALUE_CONV.md) - Value conversion system
- [PRINT_CONV.md](../../third-party/exiftool/doc/concepts/PRINT_CONV.md) - Human-readable conversions
- [BINARY_TAGS.md](../../third-party/exiftool/doc/concepts/BINARY_TAGS.md) - Binary data extraction
- [MAKERNOTE.md](../../third-party/exiftool/doc/concepts/MAKERNOTE.md) - Manufacturer-specific data

### 1.1 PROCESS_PROC - The Heart of Everything

```perl
# In ExifTool tables
PROCESS_PROC => \&ProcessCanon,  # Function reference
PROCESS_PROC => 'ProcessBinaryData',  # String name
```

PROCESS_PROC tells ExifTool how to parse a block of data. The most common is `ProcessBinaryData` (used 121+ times), which extracts fixed-offset binary structures. Understanding this is crucial.

### 1.2 Tag Tables - Not Just Data

ExifTool tag tables look like simple hashes but contain code. See [MODULE_OVERVIEW.md](../../third-party/exiftool/doc/concepts/MODULE_OVERVIEW.md) for the big picture:

```perl
0x0112 => {
    Name => 'Orientation',
    PrintConv => {
        1 => 'Horizontal (normal)',
        2 => 'Mirror horizontal',
        # ...
    },
    ValueConv => '$val > 8 ? undef : $val',  # Perl code!
}
```

### 1.3 The Conversion Pipeline

```
Raw Bytes → Format Parsing → ValueConv → PrintConv → Display
         ↑                 ↑           ↑
    (binary)        (logical value) (human readable)
```

- **ValueConv**: Converts raw data to logical values (e.g., APEX to f-stop)
- **PrintConv**: Converts logical values to human-readable strings

### 1.4 MakerNotes - The Wild West

Each camera manufacturer has proprietary data formats in MakerNotes. These often:

- Use different byte orders than the main file
- Have encrypted sections
- Calculate offsets differently
- Change format between firmware versions

### 1.5 Offset Calculations - The Biggest Gotcha

ExifTool uses this formula everywhere:

```
absolute_position = base + data_pos + relative_offset
```

But each manufacturer interprets this differently. Canon might use the TIFF header as base, Nikon might use the MakerNote start, Sony might use something else entirely.

For deep understanding of offsets, see [OFFSET_BASE_MANAGEMENT.md](../../third-party/exiftool/doc/concepts/OFFSET_BASE_MANAGEMENT.md)

- [ARCHITECTURE.md](../ARCHITECTURE.md) - Our offset management strategy (Core Runtime Architecture section)
- [READING_AND_PARSING.md](../../third-party/exiftool/doc/concepts/READING_AND_PARSING.md) - ExifTool's approach
- [CODEGEN.md](../CODEGEN.md) for complete strategy system documentation.

## Section 2: Reading ExifTool Source Code

> **Warning:** ExifTool source files can be extraordinarily long. Read the relevant ExifTool doc summaries first. Follow [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - don't try to "improve" what you find.

### Essential Files to Understand

1. **lib/Image/ExifTool.pm** - Core API and state management
2. **lib/Image/ExifTool/Exif.pm** - EXIF/TIFF parsing (study ProcessExif) - see [Exif.md](../../third-party/exiftool/doc/modules/Exif.md)
3. **lib/Image/ExifTool/Canon.pm** - Good example of manufacturer complexity - see [Canon.md](../../third-party/exiftool/doc/modules/Canon.md)
4. **lib/Image/ExifTool/README** - Documents all special table keys

Study the concepts documentation:

- [PROCESS_PROC.md](../../third-party/exiftool/doc/concepts/PROCESS_PROC.md) - Processing procedures explained
- [VALUE_CONV.md](../../third-party/exiftool/doc/concepts/VALUE_CONV.md) - Value conversion system
- [PRINT_CONV.md](../../third-party/exiftool/doc/concepts/PRINT_CONV.md) - Human-readable conversions
- [BINARY_TAGS.md](../../third-party/exiftool/doc/concepts/BINARY_TAGS.md) - Binary data extraction
- [MAKERNOTE.md](../../third-party/exiftool/doc/concepts/MAKERNOTE.md) - Manufacturer-specific data
- [MODULE_OVERVIEW.md](../../third-party/exiftool/doc/concepts/MODULE_OVERVIEW.md) - ExifTool module architecture
- [READING_AND_PARSING.md](../../third-party/exiftool/doc/concepts/READING_AND_PARSING.md) - File reading and parsing strategies
- [FILE_TYPES.md](../../third-party/exiftool/doc/concepts/FILE_TYPES.md) - File format detection
- [COMPOSITE_TAGS.md](../../third-party/exiftool/doc/concepts/COMPOSITE_TAGS.md) - Derived tag values
- [CHARSETS.md](../../third-party/exiftool/doc/concepts/CHARSETS.md) - Character encoding handling
- [PATTERNS.md](../../third-party/exiftool/doc/concepts/PATTERNS.md) - Common patterns across modules
- [WRITE_PROC.md](../../third-party/exiftool/doc/concepts/WRITE_PROC.md) - Writing procedures
- [CONTAINER_FORMATS.md](../../third-party/exiftool/doc/concepts/CONTAINER_FORMATS.md) - Container format handling
- [ERROR_HANDLING.md](../../third-party/exiftool/doc/concepts/ERROR_HANDLING.md) - Error handling strategies
- [EXIFIFD.md](../../third-party/exiftool/doc/concepts/EXIFIFD.md) - EXIF IFD structure and handling
- [GROUP_SYSTEM.md](../../third-party/exiftool/doc/concepts/GROUP_SYSTEM.md) - Tag group organization
- [OFFSET_BASE_MANAGEMENT.md](../../third-party/exiftool/doc/concepts/OFFSET_BASE_MANAGEMENT.md) - Offset calculation patterns
- [SUBDIRECTORY_SYSTEM.md](../../third-party/exiftool/doc/concepts/SUBDIRECTORY_SYSTEM.md) - Subdirectory processing
- [TAG_INFO_HASH.md](../../third-party/exiftool/doc/concepts/TAG_INFO_HASH.md) - Tag information structure
- [WRITE_SYSTEM.md](../../third-party/exiftool/doc/concepts/WRITE_SYSTEM.md) - Writing system architecture

Also study the module documentation:

- [Nikon.md](../../third-party/exiftool/doc/modules/Nikon.md) - Complex encryption and versions
- [Sony.md](../../third-party/exiftool/doc/modules/Sony.md) - Another encryption approach
- [MakerNotes.md](../../third-party/exiftool/doc/modules/MakerNotes.md) - Central dispatcher
- [Apple.md](../../third-party/exiftool/doc/modules/Apple.md) - Apple device formats and metadata
- [Casio.md](../../third-party/exiftool/doc/modules/Casio.md) - Casio camera formats
- [FujiFilm.md](../../third-party/exiftool/doc/modules/FujiFilm.md) - Fujifilm camera specifics
- [GPS.md](../../third-party/exiftool/doc/modules/GPS.md) - GPS metadata handling
- [GoPro.md](../../third-party/exiftool/doc/modules/GoPro.md) - GoPro action camera formats
- [JPEG.md](../../third-party/exiftool/doc/modules/JPEG.md) - JPEG file structure and metadata
- [Kodak.md](../../third-party/exiftool/doc/modules/Kodak.md) - Kodak camera formats
- [Minolta.md](../../third-party/exiftool/doc/modules/Minolta.md) - Minolta camera specifics
- [Olympus.md](../../third-party/exiftool/doc/modules/Olympus.md) - Olympus camera formats
- [Panasonic.md](../../third-party/exiftool/doc/modules/Panasonic.md) - Panasonic camera specifics
- [Pentax.md](../../third-party/exiftool/doc/modules/Pentax.md) - Pentax camera formats
- [TIFF.md](../../third-party/exiftool/doc/modules/TIFF.md) - TIFF file format details
- [XMP.md](../../third-party/exiftool/doc/modules/XMP.md) - Adobe XMP metadata standard

### 2.1 How to Read Tag Tables

```perl
%Image::ExifTool::Canon::CameraSettings = (
    PROCESS_PROC => \&ProcessBinaryData,
    FIRST_ENTRY => 1,
    FORMAT => 'int16s',
    GROUPS => { 0 => 'MakerNotes', 2 => 'Camera' },
    1 => {
        Name => 'MacroMode',
        PrintConv => {
            1 => 'Macro',
            2 => 'Normal',
        },
    },
    # ... hundreds more
);
```

Key points:

- `PROCESS_PROC` determines how to parse
- `FORMAT` sets default for all tags
- `FIRST_ENTRY` means array is 1-indexed (not 0)
- `GROUPS` affects how tags are categorized

### 2.2 Common Perl Patterns to Recognize

```perl
# Conditional value
ValueConv => '$val =~ /^\d+$/ ? $val : undef',

# Bit extraction
ValueConv => '($val >> 4) & 0x0f',

# Multiple values
ValueConv => '[ split " ", $val ]',

# Reference to previously extracted value
Format => 'string[$val{3}]',  # Length from tag 3

# DataMember - tag needed by other tags
DataMember => 'CameraType',
RawConv => '$$self{CameraType} = $val',
```

### 2.3 Search Strategies

#### Finding PrintConv Implementations

```bash
# Search for a specific PrintConv
grep -r "orientation.*PrintConv\|Orientation.*{" third-party/exiftool/lib/

# Find the actual hash definition
less third-party/exiftool/lib/Image/ExifTool/Exif.pm
/Orientation
```

#### Finding ProcessProc Usage

```bash
# See all uses of a processor
grep -r "ProcessSerialData" third-party/exiftool/lib/

# Find where it's defined
grep -r "sub ProcessSerialData" third-party/exiftool/lib/
```

#### Understanding Tag Dependencies

```bash
# Find DataMember usage
grep -r "DataMember.*NumAFPoints" third-party/exiftool/lib/

# See what uses that DataMember
grep -r '\$val{NumAFPoints}\|NumAFPoints}' third-party/exiftool/lib/
```

### 2.4 Navigation Tips

1. **Use the module documentation first** - Much shorter than source files
2. **Search for tag names** - They're usually unique
3. **Follow PROCESS_PROC references** - Understand the parsing strategy
4. **Check for SubDirectory** - Tags might reference other tables
5. **Look for Condition** - Runtime dispatch logic

### 2.5 Understanding Complex Tables

Some tables have special behaviors:

```perl
# Conditional subdirectories
{
    Condition => '$$self{Model} =~ /\b1DS?$/',
    Name => 'CameraInfo1D',
    SubDirectory => { TagTable => 'Image::ExifTool::Canon::CameraInfo1D' },
},

# Hook mechanism for dynamic behavior
{
    Name => 'Tag123',
    Format => 'int16u',
    Hook => '$format = $size > 2 ? "int16u" : "int8u"',
}
```

## Section 3: Common Pitfalls and Solutions

> **Critical:** Every pitfall listed here violates [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) in some way. When in doubt, copy ExifTool exactly.

For additional context on these pitfalls, see [PATTERNS.md](../../third-party/exiftool/doc/concepts/PATTERNS.md).

### 3.1 Endianness Confusion

ExifTool tracks byte order per directory. A Canon file might be little-endian overall but have big-endian values in certain maker note sections.

**Solution**: Always use the ExifReader's current byte order, not the file's global order.

```rust
// Wrong
let value = u16::from_le_bytes([data[0], data[1]]); // Assumes little-endian

// Right
let value = match reader.byte_order {
    ByteOrder::LittleEndian => u16::from_le_bytes([data[0], data[1]]),
    ByteOrder::BigEndian => u16::from_be_bytes([data[0], data[1]]),
};
```

### 3.2 Offset Base Confusion

When you see `SubDirectory => { Start => '$val' }`, the base for that subdirectory is NOT obvious. It could be relative to:

- The TIFF header
- The current directory start
- The tag's position
- Something manufacturer-specific

**Solution**: Study ProcessExif carefully. When in doubt, add debug logging to track actual vs expected positions.

```rust
// Add debug logging
trace!("Subdirectory at offset {:#x}, base={:#x}, data_pos={:#x}", 
       offset, self.base, self.data_pos);
```

### 3.3 Format Strings

ExifTool format strings can be dynamic:

```perl
Format => 'string[$val{3}]'  # Length from tag 3
Format => 'var_string'        # Variable null-terminated
```

**Solution**: Start with fixed formats only. Add variable format support incrementally.

```rust
// Phase 1: Support fixed formats
match format {
    "int16u" => read_u16(data, offset),
    "string[32]" => read_fixed_string(data, offset, 32),
    _ => return Err("Unsupported format"),
}

// Phase 2: Add variable formats later
```

### 3.4 PrintConv Complexity

Some PrintConv functions are simple lookups, others are complex:

```perl
PrintConv => 'sprintf("%.1f", $val / 10)',  # Simple
PrintConv => \&CanonEv,  # Complex function with special cases
```

**Solution**: Implement only what you need. Use --show-missing to prioritize.

For more on conversions, see:

- [VALUE_CONV.md](../../third-party/exiftool/doc/concepts/VALUE_CONV.md) - Value conversion details
- [PRINT_CONV.md](../../third-party/exiftool/doc/concepts/PRINT_CONV.md) - Print conversion patterns

### 3.5 Tag ID Collisions

Different IFDs can have tags with the same ID:

```
EXIF:0x0112 = Orientation
Sony:0x0112 = SonyOrientation (different!)
```

**Solution**: Always track which IFD/table you're in. Use fully-qualified names.

```rust
// Use group-prefixed names
"EXIF:Orientation" vs "Sony:Orientation"
```

### 3.6 Missing Null Checks

Many cameras write invalid data. Always validate:

```rust
// Wrong
let denominator = u32::from_bytes(&data[4..8]);
let value = numerator / denominator;

// Right
let denominator = u32::from_bytes(&data[4..8]);
if denominator == 0 {
    return TagValue::Invalid;
}
let value = numerator as f64 / denominator as f64;
```

### 3.7 Assuming Spec Compliance

**The spec is a lie**. Cameras don't follow it.

```rust
// Wrong: Assuming GPS coordinates are always 3 rationals
let coords = [
    read_rational(&data[0..8]),
    read_rational(&data[8..16]),
    read_rational(&data[16..24]),
];

// Right: Check actual data
let count = entry.count;
if count != 3 {
    warn!("GPS coordinate has {} values, expected 3", count);
}
```

## Section 4: Technical Reference

> **Foundation:** These technical facts were validated through extensive testing against ExifTool's behavior. They represent hard constraints that must be respected per [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md).

### 4.1 Critical ExifTool Behavior Facts

#### JPEG Segment Structure

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

#### TIFF/IFD Structure Facts

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

#### String Handling Edge Cases

**EXIF String Quirks**:

- EXIF strings are null-terminated BUT buffer may contain garbage after null terminator
- Some manufacturers pad with spaces instead of null bytes
- UTF-8 encoding not guaranteed - may need charset detection
- Always scan for null terminator or use full count length, don't assume clean strings

### 4.2 Manufacturer-Specific Facts

#### Canon MakerNote Structure

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

### 4.3 ProcessBinaryData Technical Facts

#### Binary Data Table Structure (from ExifTool source analysis)

- Uses `%binaryDataAttrs` inheritance for PROCESS_PROC
- Default format specified at table level (e.g., `FORMAT => 'int16s'`)
- Individual entries can override format
- `FIRST_ENTRY` specifies starting offset index

#### Canon CameraSettings Example (Canon.pm:2166):

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

#### Binary Data Processing Pattern

1. Extract raw binary data from maker note tag
2. Apply binary data table structure to parse individual fields
3. Each field has offset, format, and optional PrintConv
4. Field values become individual tags in output

### 4.4 Error Handling and Edge Cases

#### Bounds Checking Requirements

- ALWAYS bounds-check before reading (use `get()` not direct indexing)
- Validate offset + length ≤ file size before extraction
- Handle integer overflow in offset calculations
- Graceful degradation when data is malformed

#### Real-World File Quirks

- Some cameras create files with incorrect IFD entry counts
- Maker note data may be encrypted or obfuscated
- Files may have circular IFD references (requires cycle detection)
- Zero-length values and missing required fields occur in practice

## Section 5: Tribal Knowledge

> **Remember:** These quirks exist for camera-specific reasons. Follow [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) - don't "fix" seemingly odd behavior.

### 5.1 "Magic" Constants

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

### 5.2 Double-UTF8 Encoding

Some cameras encode UTF-8 strings twice. ExifTool silently fixes this. You should too.

For character encoding complexities, see [CHARSETS.md](../../third-party/exiftool/doc/concepts/CHARSETS.md):

```rust
// Some Sony cameras double-encode UTF-8
// ExifTool: XMP.pm:4567
if looks_like_double_utf8(&string) {
    string = decode_utf8_twice(string);
}
```

### 5.3 Manufacturer Quirks

#### Canon

- Uses 0xdeadbeef as a sentinel value
- Different offset schemes based on camera model (4, 6, 16, or 28 bytes)
- Some models have TIFF footer validation

#### Nikon

- TIFF header at offset 0x0a in maker notes
- Multiple encryption schemes
- Format changes between firmware versions

#### Sony

- Seven different maker note detection patterns
- Some models double-encode UTF-8
- Encryption on newer models

#### Samsung

- NX200 has incorrect entry count (reports 23, actually has 21)
- Requires manual fixup in code

### 5.4 Byte Order Weirdness

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

### 5.5 The "n/a" vs undef Pattern

Many PrintConv definitions use `'n/a'` strings instead of undef for missing values:

```perl
0x7fff => 'n/a',    # Canon.pm:2625
0xffff => 'n/a',    # Canon.pm:6520
```

This suggests camera firmware explicitly sets these "not available" sentinel values rather than leaving fields empty.

### 5.6 Undocumented Behaviors

1. **BITMASK Zero Handling**: When a bitmask value is 0 (no bits set), ExifTool displays "(none)" rather than an empty string

2. **JSON Type Quirks**: Some PrintConv functions return numeric strings that become JSON numbers (e.g., FNumber: "4.0" → 4.0)

3. **Evaluation Context**: PrintConv strings have access to `$self`, `$tag`, and other variables beyond just `$val`

4. **Hook Timing**: ProcessBinaryData Hooks execute before format parsing, allowing dynamic format changes

### 5.7 Remember

These quirks exist for reasons - usually a specific camera model that does something weird. When you encounter something that seems wrong:

1. **Don't fix it** - It's probably correct for some camera
2. **Document it** - Add comments with ExifTool references
3. **Test it** - Make sure your implementation matches ExifTool
4. **Ask why** - But accept that sometimes nobody knows

The metadata world is messy because cameras are messy. Embrace the chaos!

## Debugging Tips

### 1. Use ExifTool for Ground Truth

```bash
# See exactly what ExifTool extracts
exiftool -v3 image.jpg > exiftool_verbose.txt

# Get JSON output for comparison
exiftool -j image.jpg > expected.json

# See hex dump of specific tag
exiftool -htmlDump image.jpg > dump.html
```

### 2. Add Trace Logging

```rust
use tracing::trace;

trace!("ProcessExif: offset={:#x}, entries={}", offset, count);
trace!("Tag {:04x}: raw_value={:?}", tag_id, raw_value);
```

### 3. Verify Offsets

When implementing a new format, always verify your offset calculations:

```rust
assert_eq!(
    calculated_offset,
    expected_offset,
    "Offset mismatch for tag {}: calculated {:#x} != expected {:#x}",
    tag_name, calculated_offset, expected_offset
);
```

### 4. Test Incrementally

Don't try to parse an entire Canon maker note at once. Start with:

1. Detecting it exists
2. Reading the header
3. Parsing one simple tag
4. Adding more tags
5. Handling special cases

## Key Takeaways

1. **PROCESS_PROC determines parsing strategy** - Different data structures need different processors
2. **Tag tables contain both data and code** - ValueConv and PrintConv can be complex Perl expressions
3. **The conversion pipeline is multi-stage** - Raw → Logical → Human-readable
4. **MakerNotes are proprietary and quirky** - Each manufacturer does things differently
5. **Offset calculations are critical** - Small mistakes lead to reading wrong data
6. **Trust ExifTool completely** - See [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md)

**Most Important:** Every odd, confusing, seemingly "wrong" piece of ExifTool code exists for a reason. Follow [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md) religiously.