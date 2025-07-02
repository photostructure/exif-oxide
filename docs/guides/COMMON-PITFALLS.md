# Common Pitfalls and Solutions

This guide covers common mistakes when implementing ExifTool functionality in Rust and how to avoid them.

For additional context on these pitfalls, see [PATTERNS.md](../../third-party/exiftool/doc/concepts/PATTERNS.md).

## 1. Endianness Confusion

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

## 2. Offset Base Confusion

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

## 3. Format Strings

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

## 4. PrintConv Complexity

Some PrintConv functions are simple lookups, others are complex:

```perl
PrintConv => 'sprintf("%.1f", $val / 10)',  # Simple
PrintConv => \&CanonEv,  # Complex function with special cases
```

**Solution**: Implement only what you need. Use --show-missing to prioritize.

For more on conversions, see:

- [VALUE_CONV.md](../../third-party/exiftool/doc/concepts/VALUE_CONV.md) - Value conversion details
- [PRINT_CONV.md](../../third-party/exiftool/doc/concepts/PRINT_CONV.md) - Print conversion patterns

## 5. Tag ID Collisions

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

## 6. Missing Null Checks

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

## 7. Assuming Spec Compliance

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

## Related Guides

- [EXIFTOOL-CONCEPTS.md](EXIFTOOL-CONCEPTS.md) - Understand the fundamentals
- [TRIBAL-KNOWLEDGE.md](TRIBAL-KNOWLEDGE.md) - Undocumented quirks
- [READING-EXIFTOOL-SOURCE.md](READING-EXIFTOOL-SOURCE.md) - How to find answers in ExifTool