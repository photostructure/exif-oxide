# PrintConv/ValueConv Implementation Guide

This guide explains how PrintConv and ValueConv functions are implemented in exif-oxide using a compile-time registry system.

## Overview

PrintConv and ValueConv are ExifTool's mechanisms for transforming raw tag values:
- **ValueConv**: Transforms raw binary data into meaningful values (e.g., rational [39, 10] → 3.9)
- **PrintConv**: Formats values for human-readable display (e.g., 3.9 → "f/3.9")

In exif-oxide, these conversions are:
1. Extracted from ExifTool source during codegen
2. Mapped to Rust implementations via a compile-time registry
3. Generated as direct function calls with zero runtime overhead

## Architecture

### 1. Codegen Registry (`codegen/src/conv_registry.rs`)

The registry maps Perl expressions to Rust function paths:

```rust
static PRINTCONV_REGISTRY: LazyLock<HashMap<&'static str, (&'static str, &'static str)>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    
    // Common sprintf patterns
    m.insert("sprintf(\"%.1f mm\",$val)", ("crate::implementations::print_conv", "focallength_print_conv"));
    m.insert("sprintf(\"%.1f\",$val)", ("crate::implementations::print_conv", "decimal_1_print_conv"));
    
    // ExifTool function calls
    m.insert("Image::ExifTool::Exif::PrintExposureTime($val)", ("crate::implementations::print_conv", "exposuretime_print_conv"));
    
    m
});
```

### 2. Tag Extraction (`codegen/extractors/tag_kit.pl`)

The extractor identifies PrintConv/ValueConv types:
- **Simple**: Hash lookup tables (extracted directly)
- **Expression**: Perl expressions (preserved for registry lookup)
- **Manual**: Complex code references

### 3. Code Generation (`codegen/src/generators/tag_kit_modular.rs`)

Generates direct function calls based on registry lookups:

```rust
// Generated code in src/generated/Canon_pm/tag_kit/mod.rs
pub fn apply_print_conv(tag_id: u32, value: &TagValue, ...) -> TagValue {
    match tag_id {
        6 => crate::implementations::print_conv::print_fraction(value),
        // Direct function calls - no runtime lookup!
    }
}
```

### 4. Implementations (`src/implementations/`)

Actual conversion functions:

```rust
// src/implementations/print_conv.rs
pub fn fnumber_print_conv(val: &TagValue) -> TagValue {
    match val.as_f64() {
        Some(f) => TagValue::String(format!("f/{:.1}", f)),
        None => val.clone(),
    }
}
```

## Adding New Conversions

### Step 1: Implement the Function

Add to `src/implementations/print_conv.rs` or `value_conv.rs`:

```rust
/// GPS Altitude PrintConv
/// ExifTool: lib/Image/ExifTool/GPS.pm:124 - '$val =~ /^(inf|undef)$/ ? $val : "$val m"'
pub fn gpsaltitude_print_conv(val: &TagValue) -> TagValue {
    match val.as_f64() {
        Some(v) if v.is_infinite() => "inf".into(),
        Some(v) if v.is_nan() => "undef".into(),
        Some(v) => TagValue::string(format!("{v:.1} m")),
        None => TagValue::string(format!("Unknown ({val})")),
    }
}
```

### Step 2: Add Registry Entry

Update `codegen/src/conv_registry.rs`:

```rust
// Add to PRINTCONV_REGISTRY
m.insert("$val =~ /^(inf|undef)$/ ? $val : \"$val m\"", 
    ("crate::implementations::print_conv", "gpsaltitude_print_conv"));
```

### Step 3: Regenerate Code

```bash
make codegen
```

## Module-Scoped Functions

Some ExifTool functions exist in multiple modules:

```rust
// GPS.pm has ConvertTimeStamp
m.insert("GPS.pm::ConvertTimeStamp($val)", 
    ("crate::implementations::value_conv", "gpstimestamp_value_conv"));

// ID3.pm also has ConvertTimeStamp - different implementation!
m.insert("ID3.pm::ConvertTimeStamp($val)", 
    ("crate::implementations::value_conv", "id3_timestamp_value_conv"));
```

The registry tries module-scoped lookup first, then falls back to unscoped.

## Missing Conversion Tracking

Use `--show-missing` to find unimplemented conversions:

```bash
$ cargo run -- --show-missing image.jpg
# Output:
"MissingImplementations": [
    "PrintConv: sprintf(\"ISO %d\", $val) [used by tags: 0x8827, 0x8832]",
    "ValueConv: $val * 100 [used by tag: 0x9204]"
]
```

Missing conversions are tracked in `src/implementations/missing.rs` and shown grouped by expression with the tags that use them.

## Common Patterns

### sprintf Expressions
```perl
sprintf("%.1f mm", $val)     # One decimal place with units
sprintf("%.2f", $val)         # Two decimal places
sprintf("%+d", $val)          # Signed integer
sprintf("0x%x", $val)         # Hexadecimal
```

### Conditional Expressions
```perl
$val ? $val : "Auto"          # Simple ternary
$val > 0 ? "+$val" : $val     # Conditional formatting
```

### String Manipulation
```perl
"$val mm"                     # Simple concatenation
$val =~ s/\s+$//; $val        # Regex substitution
```

### APEX Conversions
```perl
2 ** ($val / 2)               # Aperture value
2 ** (-$val)                  # Shutter speed
```

## Testing

### Unit Tests
Test individual conversion functions:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fnumber_print_conv() {
        let val = TagValue::F64(3.9);
        let result = fnumber_print_conv(&val);
        assert_eq!(result, TagValue::String("f/3.9".to_string()));
    }
}
```

### Integration Tests
Use real images to verify conversions:

```bash
cargo run -- test-images/canon/Canon_40D.jpg | grep FNumber
# Should show: "EXIF:FNumber": "f/3.9"
```

### Compatibility Tests
Compare with ExifTool output:

```bash
cargo run --bin compare-with-exiftool image.jpg
```

## Performance Considerations

1. **Zero Runtime Overhead**: All expressions resolved at compile time
2. **Direct Function Calls**: No string matching or hash lookups at runtime
3. **Type Safety**: Compiler catches missing implementations
4. **Incremental Builds**: Only affected modules regenerated

## Future Improvements

1. **Expression Evaluator**: Handle simple sprintf patterns without manual implementation
2. **Auto-generation**: Generate implementations from compatibility test results
3. **BITMASK Support**: See [P15c-bitmask-printconv-implementation.md](../todo/P15c-bitmask-printconv-implementation.md)

## Troubleshooting

### Expression Not Found
If a conversion shows as missing but you've added it:
1. Check expression normalization - whitespace matters
2. Verify module name format (GPS_pm vs GPS.pm)
3. Run `make codegen` after registry changes

### Wrong Conversion Applied
1. Check for module-scoped conflicts
2. Verify tag ID mapping is correct
3. Use debug logging to trace conversion path

### Performance Issues
1. Ensure using release builds for benchmarking
2. Check for accidental runtime lookups
3. Profile with `cargo flamegraph`