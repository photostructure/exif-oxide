# Critical ExifTool Concepts

This guide explains the core concepts you need to understand when working with ExifTool source code and translating it to Rust.

## Essential Reading

Before diving into code, read these essential ExifTool documentation files:

- [PROCESS_PROC.md](../../third-party/exiftool/doc/concepts/PROCESS_PROC.md) - Processing procedures explained
- [VALUE_CONV.md](../../third-party/exiftool/doc/concepts/VALUE_CONV.md) - Value conversion system
- [PRINT_CONV.md](../../third-party/exiftool/doc/concepts/PRINT_CONV.md) - Human-readable conversions
- [BINARY_TAGS.md](../../third-party/exiftool/doc/concepts/BINARY_TAGS.md) - Binary data extraction
- [MAKERNOTE.md](../../third-party/exiftool/doc/concepts/MAKERNOTE.md) - Manufacturer-specific data

## 1. PROCESS_PROC - The Heart of Everything

```perl
# In ExifTool tables
PROCESS_PROC => \&ProcessCanon,  # Function reference
PROCESS_PROC => 'ProcessBinaryData',  # String name
```

PROCESS_PROC tells ExifTool how to parse a block of data. The most common is `ProcessBinaryData` (used 121+ times), which extracts fixed-offset binary structures. Understanding this is crucial.

## 2. Tag Tables - Not Just Data

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

## 3. The Conversion Pipeline

```
Raw Bytes → Format Parsing → ValueConv → PrintConv → Display
         ↑                 ↑           ↑
    (binary)        (logical value) (human readable)
```

- **ValueConv**: Converts raw data to logical values (e.g., APEX to f-stop)
- **PrintConv**: Converts logical values to human-readable strings

## 4. MakerNotes - The Wild West

Each camera manufacturer has proprietary data formats in MakerNotes. These often:

- Use different byte orders than the main file
- Have encrypted sections
- Calculate offsets differently
- Change format between firmware versions

## 5. Offset Calculations - The Biggest Gotcha

ExifTool uses this formula everywhere:

```
absolute_position = base + data_pos + relative_offset
```

But each manufacturer interprets this differently. Canon might use the TIFF header as base, Nikon might use the MakerNote start, Sony might use something else entirely.

For deep understanding of offsets, see:

- [OFFSET-BASE-MANAGEMENT.md](../OFFSET-BASE-MANAGEMENT.md) - Our offset management strategy
- [READING_AND_PARSING.md](../../third-party/exiftool/doc/concepts/READING_AND_PARSING.md) - ExifTool's approach

## Key Takeaways

1. **PROCESS_PROC determines parsing strategy** - Different data structures need different processors
2. **Tag tables contain both data and code** - ValueConv and PrintConv can be complex Perl expressions
3. **The conversion pipeline is multi-stage** - Raw → Logical → Human-readable
4. **MakerNotes are proprietary and quirky** - Each manufacturer does things differently
5. **Offset calculations are critical** - Small mistakes lead to reading wrong data

## Related Guides

- [READING-EXIFTOOL-SOURCE.md](READING-EXIFTOOL-SOURCE.md) - How to navigate ExifTool's Perl code
- [COMMON-PITFALLS.md](COMMON-PITFALLS.md) - Mistakes to avoid
- [TRIBAL-KNOWLEDGE.md](TRIBAL-KNOWLEDGE.md) - Undocumented quirks and mysteries