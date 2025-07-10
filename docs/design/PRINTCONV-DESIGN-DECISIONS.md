# PrintConv Design Decisions

**🚨 CRITICAL: While we mostly follow [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md), this is a rare case where we improve presentation logic.**

This document explains why exif-oxide diverges from ExifTool's PrintConv implementation and the design choices we made.

## The Problem

In ExifTool, PrintConv functions always return strings. When serializing to JSON, ExifTool then applies regex pattern matching to determine if a PrintConv result "looks numeric" and should be serialized as a JSON number or string:

```perl
# ExifTool's EscapeJSON function (simplified)
if ($val =~ /^-?(\d|[1-9]\d{1,14})(\.\d{1,16})?(e[-+]?\d{1,3})?$/i) {
    # Looks numeric - output as JSON number
    return $val;
} else {
    # Not numeric - output as JSON string
    return "\"$val\"";
}
```

This leads to:

- `ISO: "100"` → `"ISO": 100` (string matches numeric pattern)
- `FNumber: "2.8"` → `"FNumber": 2.8` (string matches numeric pattern)
- `ExposureTime: "1/100"` → `"ExposureTime": "1/100"` (contains "/" so doesn't match)

## Why This is Problematic

1. **Type Safety Lost**: The type conversion chain is: `value → string → regex match → parse back to number`
2. **Fragile**: Depends on regex pattern matching that could have false positives/negatives
3. **Inconsistent**: PrintConv is meant for human-readable display, yet sometimes becomes numeric in JSON
4. **Unpredictable**: Whether a value is numeric or string depends on its string representation

## Our Solution

In exif-oxide, PrintConv functions return `TagValue` instead of `String`. This allows each PrintConv function to explicitly decide the JSON serialization type:

```rust
// String for human-readable display
pub fn exposuretime_print_conv(val: &TagValue) -> TagValue {
    TagValue::String("1/100".to_string())  // Always a string in JSON
}

// Numeric passthrough for data values
pub fn fnumber_print_conv(val: &TagValue) -> TagValue {
    TagValue::F64(2.8)  // Always a number in JSON
}
```

## Benefits

1. **Type Safety**: No regex guessing - the PrintConv function explicitly chooses the type
2. **Predictable**: Tag behavior is defined in code, not by string patterns
3. **Clear Intent**: Display-oriented tags return strings, data-oriented tags return numbers
4. **Simpler**: No complex regex patterns or string parsing in the serialization layer

## API Design

Our `TagEntry` structure provides both representations:

```rust
pub struct TagEntry {
    pub value: TagValue,  // Post-ValueConv: typed data value
    pub print: TagValue,  // Post-PrintConv: display value (string or numeric)
}
```

Consumers can choose:

- `entry.value` - Always the typed data value
- `entry.print` - The display representation (which may be string or numeric)

## Compatibility Note

This design diverges from ExifTool's JSON output for some tags. We believe this is a defensible improvement because:

1. We fully trust ExifTool for **data extraction** (camera quirks, malformed data)
2. This is purely about **data presentation** - how to format already-extracted values
3. The API provides clear, predictable behavior rather than regex-based guessing

## Examples

| Tag          | ExifTool PrintConv | ExifTool JSON | Our PrintConv             | Our JSON |
| ------------ | ------------------ | ------------- | ------------------------- | -------- |
| ExposureTime | "1/100"            | "1/100"       | TagValue::String("1/100") | "1/100"  |
| FNumber      | "2.8"              | 2.8           | TagValue::F64(2.8)        | 2.8      |
| ISO          | "100"              | 100           | TagValue::U32(100)        | 100      |
| Flash        | "Fired"            | "Fired"       | TagValue::String("Fired") | "Fired"  |

The key difference: we don't rely on string patterns to determine numeric vs string serialization.
