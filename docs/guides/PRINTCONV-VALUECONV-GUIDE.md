# PrintConv/ValueConv Implementation Guide

Complete guide to PrintConv and ValueConv functions in exif-oxide, including design decisions and implementation details.

## Design Philosophy 

**🚨 CRITICAL**: While we mostly follow [TRUST-EXIFTOOL.md](../TRUST-EXIFTOOL.md), PrintConv is a rare case where we improve presentation logic.

### The ExifTool Problem

ExifTool's PrintConv functions always return strings, then uses regex to guess JSON types:

```perl
# ExifTool's EscapeJSON (simplified)
if ($val =~ /^-?(\d|[1-9]\d{1,14})(\.\d{1,16})?(e[-+]?\d{1,3})?$/i) {
    return $val;  # Looks numeric → JSON number
} else {
    return "\"$val\"";  # Not numeric → JSON string  
}
```

**Problems**: 
- Type conversion chain: `value → string → regex → number`
- Fragile regex pattern matching
- Unpredictable JSON types

### Our Solution

PrintConv functions return `TagValue` for explicit JSON type control:

```rust
// String for human display
pub fn exposuretime_print_conv(val: &TagValue) -> TagValue {
    TagValue::String("1/100".to_string())  // Always string in JSON
}

// Numeric for data values  
pub fn fnumber_print_conv(val: &TagValue) -> TagValue {
    TagValue::F64(2.8)  // Always number in JSON
}
```

**Benefits**: Type safety, predictable behavior, clear intent.

## Architecture Overview

### Dual Registry System (`codegen/src/conv_registry.rs`)

**PrintConv Registry** - Maps expressions to display functions:
```rust
static PRINTCONV_REGISTRY: LazyLock<HashMap<&'static str, (&'static str, &'static str)>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("sprintf(\"%.1f mm\",$val)", ("crate::implementations::print_conv", "focallength_print_conv"));
    m.insert("Image::ExifTool::Exif::PrintExposureTime($val)", ("crate::implementations::print_conv", "exposuretime_print_conv"));
    m
});
```

**ValueConv Registry** - Maps expressions to data conversion functions:
```rust
static VALUECONV_REGISTRY: LazyLock<HashMap<&'static str, (&'static str, &'static str)>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    
    // Simple arithmetic patterns
    m.insert("$val * 100", ("crate::implementations::value_conv", "multiply_100_value_conv"));
    m.insert("$val=~s/ +$//; $val", ("crate::implementations::value_conv", "trim_whitespace_value_conv"));
    m.insert("$val=~s/^.*: //;$val", ("crate::implementations::value_conv", "remove_prefix_colon_value_conv"));
    
    // APEX conversions  
    m.insert("2**(-$val)", ("crate::implementations::value_conv", "apex_shutter_speed_value_conv"));
    m.insert("2**($val / 2)", ("crate::implementations::value_conv", "apex_aperture_value_conv"));
    
    // GPS coordinates
    m.insert("Image::ExifTool::GPS::ToDegrees($val)", ("crate::implementations::value_conv", "gps_coordinate_value_conv"));
    
    m
});
```

### Three-Tier Lookup System

The code generator checks in priority order:

1. **Tag-Specific Registry** (highest priority)
   - `Module::Tag` for module-specific implementations
   - `Tag` for universal implementations (e.g., Flash)

2. **Expression Registry** (PrintConv/ValueConv registries above)
   - **Exact match first** - tries expressions as-is from ExifTool
   - **Normalized match** - uses `normalize_expression.pl` for consistent formatting

3. **Raw Expression Fallback** (preserved for manual implementation)

### Perl Expression Normalization

ExifTool expressions have inconsistent whitespace. We use Perl::Tidy to normalize them:

```bash
# Before normalization
echo 'sprintf( "%.1f mm" , $val )' | perl normalize_expression.pl
# After normalization
sprintf("%.1f mm", $val)
```

**Why Perl::Tidy?**
- **Reliable**: Uses Perl's own proven parser (not manual Rust parsing)
- **Complete**: Handles all Perl syntax correctly
- **Consistent**: Standardizes whitespace, parentheses, operators
- **Maintainable**: 4 lines of code vs 200+ lines of manual parsing

**Tidy Options Used**:
- `-npro` - Don't read .perltidyrc files
- `-pt=2` - Tight parentheses (minimal spacing)
- `-bt=2 -sbt=2` - Tight brackets and braces
- `-ci=0` - No continuation indentation

Generated code embeds function names directly:
```rust
// Generated in src/generated/Exif_pm/tag_kit/datetime.rs
TagKitDef {
    name: "SubSecTime", 
    value_conv: Some("trim_whitespace_value_conv"),  // Function name!
    print_conv: PrintConvType::None,
}
```

## Function Implementation

### PrintConv Functions (`src/implementations/print_conv.rs`)

**Signature**: `fn(val: &TagValue) -> TagValue`
**Purpose**: Format logical values for display

```rust
/// FNumber PrintConv - returns numeric for JSON precision
pub fn fnumber_print_conv(val: &TagValue) -> TagValue {
    match val.as_f64() {
        Some(f) => TagValue::F64(f),  // Numeric in JSON
        None => TagValue::String(format!("Unknown ({val})"))
    }
}

/// ExposureTime PrintConv - returns string for human readability
pub fn exposuretime_print_conv(val: &TagValue) -> TagValue {
    TagValue::String("1/100".to_string())  // Always string
}
```

### ValueConv Functions (`src/implementations/value_conv.rs`)

**Signature**: `fn(val: &TagValue) -> Result<TagValue>`
**Purpose**: Convert raw binary data to logical values

```rust
/// Trim trailing whitespace - ExifTool: $val=~s/ +$//; $val
pub fn trim_whitespace_value_conv(value: &TagValue) -> Result<TagValue> {
    match value {
        TagValue::String(s) => Ok(TagValue::String(s.trim_end().to_string())),
        _ => Ok(value.clone()),
    }
}

/// GPS coordinate conversion - ExifTool: Image::ExifTool::GPS::ToDegrees
pub fn gps_coordinate_value_conv(value: &TagValue) -> Result<TagValue> {
    match value {
        TagValue::RationalArray(coords) if coords.len() >= 3 => {
            let degrees = coords[0].0 as f64 / coords[0].1 as f64;
            let minutes = coords[1].0 as f64 / coords[1].1 as f64;
            let seconds = coords[2].0 as f64 / coords[2].1 as f64;
            let decimal_degrees = degrees + ((minutes + seconds / 60.0) / 60.0);
            Ok(TagValue::F64(decimal_degrees))
        }
        _ => Err(ExifError::ParseError("GPS coordinate requires rational array".to_string())),
    }
}
```

## Adding New Conversions

### For Simple Patterns

1. **Add registry entry** in `codegen/src/conv_registry.rs`:
```rust
// For ValueConv - use normalized expressions (tight formatting)
m.insert("$val / 256", ("crate::implementations::value_conv", "divide_256_value_conv"));

// For PrintConv - use normalized expressions (no extra spaces)
m.insert("sprintf(\"ISO %d\",$val)", ("crate::implementations::print_conv", "iso_print_conv"));
```

**⚠️ Important**: Registry keys should use **normalized expressions** (run through `normalize_expression.pl`). The system tries exact match first, then normalization, so using normalized keys avoids the expensive Perl::Tidy step.

2. **Implement function**:
```rust
pub fn divide_256_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_f64() {
        Some(val) => Ok(TagValue::F64(val / 256.0)),
        None => Ok(value.clone()),
    }
}
```

3. **Regenerate**: `make codegen`

### For Complex Tags (Flash, ComplexHash)

Use tag-specific registry for tags needing special handling:

```rust
// In TAG_SPECIFIC_PRINTCONV
m.insert("Flash", ("crate::implementations::print_conv", "flash_print_conv"));
```

This overrides any expression-based lookup for Flash tags across all modules.

### For Inter-Tag Dependencies (DataMember References)

**⚠️ Special Case**: When ValueConv expressions reference `$self{DataMember}`, handle in binary processors, not registry.

**Problem Example**: Canon focal length conversion
```perl
# ExifTool Canon.pm:2463-2480
23 => {
    Name => 'MaxFocalLength',
    ValueConv => '$val / ($self{FocalUnits} || 1)',  # Needs FocalUnits from context!
    PrintConv => '"$val mm"',
},
```

**❌ Wrong Approach**: Extend ValueConv signature
```rust
// DON'T DO THIS - breaks pure function model
fn value_conv(value: &TagValue, context: &ExtractionContext) -> Result<TagValue>
```

**✅ Correct Approach**: Handle in binary data processor
```rust
// src/implementations/canon/binary_data.rs
pub fn process_camera_settings(data: &[u8]) -> Result<Vec<TagEntry>> {
    // Extract raw values with full context
    let focal_units = extract_u16_at_offset(data, FOCAL_UNITS_OFFSET)? as f64;
    let raw_min_focal = extract_u16_at_offset(data, MIN_FOCAL_OFFSET)? as f64;
    let raw_max_focal = extract_u16_at_offset(data, MAX_FOCAL_OFFSET)? as f64;
    
    // Apply conversion with context (like ExifTool does)
    let focal_divisor = if focal_units != 0.0 { focal_units } else { 1.0 };
    let min_focal_length = raw_min_focal / focal_divisor;  
    let max_focal_length = raw_max_focal / focal_divisor;
    
    vec![
        TagEntry::new("Canon", "MinFocalLength", TagValue::F64(min_focal_length)),
        TagEntry::new("Canon", "MaxFocalLength", TagValue::F64(max_focal_length)),
        // ... other camera settings with converted values
    ]
}
```

**Plus**: Add simple PrintConv pattern to registry for display:
```rust
// In PRINTCONV_REGISTRY for the display format
m.insert("\"$val mm\"", ("crate::implementations::print_conv", "focal_length_mm_print_conv"));
```

**Why This Approach**:
- **Mirrors ExifTool**: Conversion happens during binary processing where context exists
- **No Breaking Changes**: Registry system stays pure for isolated conversions
- **Handles Edge Cases**: Can implement `|| 1` fallback logic properly
- **Performance**: No double-processing of binary data

**Pattern for Future Dependencies**: When you see `$self{DataMember}` in ValueConv, it belongs in the binary processor, not the registry.

## Current Registry Coverage

**ValueConv Patterns Implemented**:
- `$val * 100`, `$val / 8`, `$val / 256` - Simple arithmetic
- `$val=~s/ +$//; $val` - Trim whitespace
- `$val=~s/^.*: //;$val` - Remove prefix 
- `2**(-$val)`, `2**($val/2)` - APEX conversions
- GPS coordinate conversions
- Canon-specific patterns (ISO, division, bit manipulation)

**PrintConv Patterns Implemented**:
- `sprintf` patterns for formatting
- ExifTool function calls
- Tag-specific implementations (Flash, GPS references)

## Performance Benefits

1. **Zero Runtime Overhead**: All expressions resolved at compile time
2. **Direct Function Calls**: No string matching or hash lookups
3. **Type Safety**: Compiler catches missing implementations
4. **Exact Match First**: Registry tries exact matches before expensive normalization

## Testing

### Unit Tests
```rust
#[test]
fn test_trim_whitespace_value_conv() {
    let val = TagValue::String("test   ".to_string());
    let result = trim_whitespace_value_conv(&val).unwrap();
    assert_eq!(result, TagValue::String("test".to_string()));
}
```

### Integration Tests
```bash
# Compare with ExifTool output
cargo run --bin compare-with-exiftool image.jpg

# Check specific conversions
cargo run -- test-images/canon/Canon_40D.jpg | grep FNumber
```

## Common Patterns Reference

**ValueConv Arithmetic**:
- `$val * 100` → `multiply_100_value_conv`
- `$val / 8` → `divide_8_value_conv`  
- `$val - 5` → `subtract_5_value_conv`

**ValueConv String Processing**:
- `$val=~s/ +$//; $val` → `trim_whitespace_value_conv`
- `$val=~s/^.*: //;$val` → `remove_prefix_colon_value_conv`

**PrintConv Formatting**:
- `sprintf("%.1f mm",$val)` → `focallength_print_conv`
- `sprintf("f/%.1f",$val)` → `fnumber_print_conv`

**APEX Conversions**:
- `2**(-$val)` → `apex_shutter_speed_value_conv` (ValueConv)
- `2**($val/2)` → `apex_aperture_value_conv` (ValueConv)

## API Structure

Our `TagEntry` provides both representations:

```rust
pub struct TagEntry {
    pub value: TagValue,  // Post-ValueConv: typed data
    pub print: TagValue,  // Post-PrintConv: display (string or numeric)
}
```

Consumers choose:
- `entry.value` - Always the logical data value
- `entry.print` - Display representation (may be string or numeric for JSON)

This design ensures predictable JSON output while maintaining ExifTool data extraction compatibility.

## Troubleshooting

### "Composite Tag Fails Due to Missing Dependencies"

**Symptoms**: Composite tags like `Composite:Lens` fail because required Canon/Nikon tags are missing or have raw values.

**Diagnosis**: Check if the missing tags have ValueConv expressions with `$self{DataMember}` references:
```bash
# Look for DataMember patterns in ExifTool source
grep -A5 -B5 '\$self{' third-party/exiftool/lib/Image/ExifTool/Canon.pm
```

**Solution**: Implement conversion in binary data processor, not registry. See "Inter-Tag Dependencies" section above.

### "Registry Lookup Fails Despite Correct Expression"

**Symptoms**: Expression shows as raw Perl code instead of function name in generated code.

**Common Causes**:
1. **Whitespace differences** - Try normalizing the expression:
   ```bash
   echo 'sprintf( "%.1f mm" , $val )' | perl codegen/extractors/normalize_expression.pl
   ```

2. **Missing registry entry** - Add the normalized expression to appropriate registry

3. **Module scoping** - Expression might need module-specific entry like `"Canon_pm::WhiteBalance"`

### "Wrong Conversion Applied"

**Symptoms**: Tag gets converted but produces incorrect values.

**Debug Steps**:
1. Check for module-scoped conflicts in registries
2. Verify tag ID mapping in generated code
3. Compare ExifTool source for exact conversion logic
4. Use debug logging to trace conversion path