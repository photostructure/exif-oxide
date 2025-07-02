# Tribal Knowledge

This document captures the undocumented quirks, mysteries, and "tribal knowledge" discovered while working with ExifTool and camera metadata.

## "Magic" Constants

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

## Double-UTF8 Encoding

Some cameras encode UTF-8 strings twice. ExifTool silently fixes this. You should too.

For character encoding complexities, see [CHARSETS.md](../../third-party/exiftool/doc/concepts/CHARSETS.md):

```rust
// Some Sony cameras double-encode UTF-8
// ExifTool: XMP.pm:4567
if looks_like_double_utf8(&string) {
    string = decode_utf8_twice(string);
}
```

## Manufacturer Quirks

### Canon

- Uses 0xdeadbeef as a sentinel value
- Different offset schemes based on camera model (4, 6, 16, or 28 bytes)
- Some models have TIFF footer validation

### Nikon

- TIFF header at offset 0x0a in maker notes
- Multiple encryption schemes
- Format changes between firmware versions

### Sony

- Seven different maker note detection patterns
- Some models double-encode UTF-8
- Encryption on newer models

### Samsung

- NX200 has incorrect entry count (reports 23, actually has 21)
- Requires manual fixup in code

## Byte Order Weirdness

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

## The "n/a" vs undef Pattern

Many PrintConv definitions use `'n/a'` strings instead of undef for missing values:

```perl
0x7fff => 'n/a',    # Canon.pm:2625
0xffff => 'n/a',    # Canon.pm:6520
```

This suggests camera firmware explicitly sets these "not available" sentinel values rather than leaving fields empty.

## Inverse Conversion Complexity

Some PrintConvInv functions are more complex than their forward counterparts:

```perl
# Forward: simple sprintf
PrintConv => 'sprintf("%.4x%.5d",$val>>16,$val&0xffff)',

# Inverse: complex parsing
PrintConvInv => sub {
    # Complex regex and parsing logic...
},
```

This reflects the asymmetric complexity of parsing vs. formatting.

## Performance Hacks

### Size Limits

```perl
# ProcessBinaryData protection against memory exhaustion
my $sizeLimit = $size < 65536 ? $size : 65536;
```

ExifTool limits unknown tag generation to prevent memory exhaustion with corrupted files.

### Fast Scan Mode

```perl
if ($self->{OPTIONS}{FastScan} >= 4) {
    # Skip actual data reading, use format detection only
}
```

Different levels of "fast scan" trade accuracy for speed.

## Undocumented Behaviors

1. **BITMASK Zero Handling**: When a bitmask value is 0 (no bits set), ExifTool displays "(none)" rather than an empty string

2. **JSON Type Quirks**: Some PrintConv functions return numeric strings that become JSON numbers (e.g., FNumber: "4.0" → 4.0)

3. **Evaluation Context**: PrintConv strings have access to `$self`, `$tag`, and other variables beyond just `$val`

4. **Hook Timing**: ProcessBinaryData Hooks execute before format parsing, allowing dynamic format changes

## Remember

These quirks exist for reasons - usually a specific camera model that does something weird. When you encounter something that seems wrong:

1. **Don't fix it** - It's probably correct for some camera
2. **Document it** - Add comments with ExifTool references
3. **Test it** - Make sure your implementation matches ExifTool
4. **Ask why** - But accept that sometimes nobody knows

The metadata world is messy because cameras are messy. Embrace the chaos!

## Related Guides

- [EXIFTOOL-CONCEPTS.md](EXIFTOOL-CONCEPTS.md) - The fundamentals
- [COMMON-PITFALLS.md](COMMON-PITFALLS.md) - Mistakes to avoid
- [READING-EXIFTOOL-SOURCE.md](READING-EXIFTOOL-SOURCE.md) - Finding answers
