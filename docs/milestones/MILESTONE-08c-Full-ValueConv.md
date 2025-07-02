# Milestone 8c: Full ValueConv Implementation

**Duration**: 2 weeks  
**Goal**: Complete ValueConv system with all mathematical conversions

## Overview

ValueConv (Value Conversion) is ExifTool's system for converting raw metadata values into logical, programmatically useful forms. This milestone implements the remaining ValueConv functions, focusing on mathematical conversions, date/time parsing, and GPS coordinate transformation.

## Essential Reading Before Starting

### Core Documentation

1. **[VALUE_CONV.md](../../third-party/exiftool/doc/concepts/VALUE_CONV.md)** - Complete ExifTool ValueConv documentation

   - Read sections on expression evaluation and available variables
   - Pay attention to the processing order and type handling

2. **[API-DESIGN.md](../design/API-DESIGN.md)** - Understanding the TagEntry structure

   - Focus on the value/print field separation
   - Understand how ValueConv fits in the processing pipeline

3. **[IMPLEMENTATION-PALETTE.md](../design/IMPLEMENTATION-PALETTE.md)** - How to add implementations
   - See existing ValueConv examples in `src/implementations/value_conv.rs`
   - Follow the registration pattern

### ExifTool Source References

1. **lib/Image/ExifTool/Exif.pm** - APEX conversion functions

   - Search for `ConvertAperture`, `ConvertShutterSpeed`
   - Note the mathematical formulas used

2. **lib/Image/ExifTool/GPS.pm** - GPS coordinate conversions

   - Look for coordinate conversion logic
   - Understand hemisphere handling (N/S, E/W)

3. **lib/Image/ExifTool.pm** - Core conversion utilities
   - `ConvertUnixTime`, `ConvertDateTime` functions
   - General mathematical conversion patterns

## Key Deliverables

### 1. APEX Conversions

APEX (Additive System of Photographic Exposure) values need mathematical conversion:

```rust
// ShutterSpeedValue: APEX to seconds
// Formula: ShutterSpeed = 2^(-ShutterSpeedValue)
pub fn shutter_speed_value_conv(val: &TagValue) -> TagValue {
    match val {
        TagValue::Rational(num, den) if *den != 0 => {
            let apex_value = *num as f64 / *den as f64;
            let seconds = (2.0_f64).powf(-apex_value);
            TagValue::Float(seconds)
        }
        _ => val.clone(),
    }
}

// ApertureValue: APEX to f-number
// Formula: FNumber = sqrt(2^ApertureValue)
pub fn aperture_value_conv(val: &TagValue) -> TagValue {
    match val {
        TagValue::Rational(num, den) if *den != 0 => {
            let apex_value = *num as f64 / *den as f64;
            let f_number = (2.0_f64).powf(apex_value / 2.0).sqrt();
            TagValue::Float(f_number)
        }
        _ => val.clone(),
    }
}

// ExposureCompensation: Already in stops, but may need formatting
pub fn exposure_compensation_value_conv(val: &TagValue) -> TagValue {
    match val {
        TagValue::Rational(num, den) if *den != 0 => {
            TagValue::Float(*num as f64 / *den as f64)
        }
        _ => val.clone(),
    }
}
```

### 2. GPS Coordinate Conversion

Convert GPS coordinates from degrees/minutes/seconds to decimal degrees:

```rust
pub fn gps_coordinate_value_conv(val: &TagValue) -> TagValue {
    // GPS coordinates come as array of 3 rationals: [degrees, minutes, seconds]
    match val {
        TagValue::Array(coords) if coords.len() == 3 => {
            if let (Some(deg), Some(min), Some(sec)) = (
                rational_to_float(&coords[0]),
                rational_to_float(&coords[1]),
                rational_to_float(&coords[2]),
            ) {
                // Convert to decimal degrees
                let decimal = deg + (min / 60.0) + (sec / 3600.0);
                TagValue::Float(decimal)
            } else {
                val.clone()
            }
        }
        _ => val.clone(),
    }
}

// Helper function
fn rational_to_float(val: &TagValue) -> Option<f64> {
    match val {
        TagValue::Rational(num, den) if *den != 0 => {
            Some(*num as f64 / *den as f64)
        }
        _ => None,
    }
}
```

### 3. Date/Time Conversions

Parse EXIF date/time strings into standardized formats:

```rust
pub fn exif_datetime_value_conv(val: &TagValue) -> TagValue {
    match val {
        TagValue::String(s) => {
            // EXIF format: "YYYY:MM:DD HH:MM:SS"
            // Convert to ISO 8601: "YYYY-MM-DDTHH:MM:SS"
            if let Some(datetime) = parse_exif_datetime(s) {
                TagValue::String(datetime.to_rfc3339())
            } else {
                val.clone()
            }
        }
        _ => val.clone(),
    }
}

fn parse_exif_datetime(s: &str) -> Option<DateTime<FixedOffset>> {
    // Parse "2024:01:15 14:30:45" format
    // Handle timezone if present
    // Return standardized datetime
}
```

### 4. Complex Mathematical Conversions

Implement other mathematical conversions found in ExifTool:

```rust
// Focal length conversions
pub fn focal_length_value_conv(val: &TagValue) -> TagValue {
    // Convert from rational to float
    match val {
        TagValue::Rational(num, den) if *den != 0 => {
            TagValue::Float(*num as f64 / *den as f64)
        }
        _ => val.clone(),
    }
}

// Light value calculation
pub fn light_value_conv(fnumber: &TagValue, exposure_time: &TagValue) -> TagValue {
    // LV = log2(FNumber²/ExposureTime)
    // Implement the calculation
}
```

## Implementation Strategy

### Phase 1: Core Mathematical Functions (Week 1)

1. **APEX Conversions**

   - Implement shutter speed, aperture, and exposure compensation
   - Add comprehensive unit tests
   - Verify against ExifTool output

2. **GPS Conversions**
   - Implement coordinate conversion
   - Handle edge cases (0°, negative values)
   - Test with various GPS formats

### Phase 2: Date/Time and Complex Functions (Week 2)

1. **Date/Time Parsing**

   - EXIF datetime format
   - GPS date/time format
   - Subsecond handling

2. **Additional Conversions**
   - Any other mathematical conversions needed by test images
   - Use `--show-missing` to identify required conversions

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutter_speed_apex() {
        // APEX value 5 = 1/32 second
        let apex = TagValue::Rational(5, 1);
        let result = shutter_speed_value_conv(&apex);
        match result {
            TagValue::Float(f) => assert!((f - 0.03125).abs() < 0.0001),
            _ => panic!("Expected float result"),
        }
    }

    #[test]
    fn test_gps_coordinate_conversion() {
        // 40° 26' 46.8" = 40.446333...
        let coords = TagValue::Array(vec![
            TagValue::Rational(40, 1),
            TagValue::Rational(26, 1),
            TagValue::Rational(468, 10),
        ]);
        let result = gps_coordinate_value_conv(&coords);
        match result {
            TagValue::Float(f) => assert!((f - 40.446333).abs() < 0.000001),
            _ => panic!("Expected float result"),
        }
    }
}
```

### Integration Tests

1. Test with real images containing:

   - APEX values (professional cameras)
   - GPS coordinates (smartphones, GPS-enabled cameras)
   - Various date/time formats

2. Compare with ExifTool -# output:
   ```bash
   # ValueConv output
   exiftool -j -ShutterSpeedValue# -ApertureValue# image.jpg
   ```

## Success Criteria

- [ ] All APEX conversions match ExifTool output exactly
- [ ] GPS coordinates convert to decimal degrees correctly
- [ ] Date/time parsing handles all EXIF formats
- [ ] Integration tests pass for all test images
- [ ] Performance is acceptable (no complex calculations in hot paths)

## Common Pitfalls to Avoid

1. **Floating Point Precision**: Use appropriate epsilon for comparisons
2. **Division by Zero**: Always check denominators in rationals
3. **GPS Hemisphere**: Remember to handle N/S, E/W indicators if present
4. **Time Zones**: EXIF datetime often lacks timezone info
5. **Edge Cases**: Test with 0, negative values, and extreme values

## Registration Pattern

Add all new conversions to the registry:

```rust
// In src/implementations/value_conv.rs
pub fn init_value_conv_registry(registry: &mut Registry) {
    // Existing conversions...

    // APEX conversions
    registry.register_value_conv("ShutterSpeedValue", shutter_speed_value_conv);
    registry.register_value_conv("ApertureValue", aperture_value_conv);
    registry.register_value_conv("ExposureCompensation", exposure_compensation_value_conv);

    // GPS conversions
    registry.register_value_conv("GPSLatitude", gps_coordinate_value_conv);
    registry.register_value_conv("GPSLongitude", gps_coordinate_value_conv);

    // Date/time conversions
    registry.register_value_conv("DateTimeOriginal", exif_datetime_value_conv);
    registry.register_value_conv("CreateDate", exif_datetime_value_conv);
}
```

## Related Milestone Work

- **Milestone 8b**: TagEntry API (already completed) - provides the value/print separation
- **Milestone 8d**: Composite tags - some composites depend on these ValueConv results
- **Milestone 8f**: Composite infrastructure - ensures these conversions integrate properly

## Notes for Implementation

1. Start with APEX conversions as they're well-defined mathematically
2. GPS conversion is critical for many applications
3. Date/time parsing can be complex - consider using chrono crate
4. Always check ExifTool source for edge case handling
5. Some conversions may require access to other tag values (use composite tags for those)

Remember: We're translating ExifTool's behavior exactly. If ExifTool does something odd, we do it too!
