//! ValueConv implementations for exif-oxide
//!
//! ValueConv functions perform mathematical conversions on raw tag values
//! to produce logical values. Unlike PrintConv which formats for display,
//! ValueConv maintains precision for further calculations and round-trip operations.
//!
//! All implementations are direct translations from ExifTool source code.

use crate::types::{ExifError, Result, TagValue};

/// GPS coordinate conversion to decimal degrees (unsigned)
///
/// ExifTool: lib/Image/ExifTool/GPS.pm lines 12-14 (%coordConv)
/// ExifTool: lib/Image/ExifTool/GPS.pm lines 364-374 (sub ToDegrees)
/// Formula: $deg = $d + (($m || 0) + ($s || 0)/60) / 60; (GPS.pm:380)
///
/// Converts rational array [degrees, minutes, seconds] to decimal degrees
/// NOTE: This produces UNSIGNED decimal degrees - hemisphere sign is applied
/// in Composite tags that combine coordinate + ref (e.g., Composite:GPSLatitude)
pub fn gps_coordinate_value_conv(value: &TagValue) -> Result<TagValue> {
    match value {
        TagValue::RationalArray(coords) if coords.len() >= 3 => {
            // ExifTool's ToDegrees extracts 3 numeric values using regex:
            // my ($d, $m, $s) = ($val =~ /((?:[+-]?)(?=\d|\.\d)\d*(?:\.\d*)?(?:[Ee][+-]\d+)?)/g);
            // For rational arrays, we can extract directly as decimals

            // Extract degrees (first rational)
            let degrees = if coords[0].1 != 0 {
                coords[0].0 as f64 / coords[0].1 as f64
            } else {
                0.0 // ExifTool uses 0 for undefined values
            };

            // Extract minutes (second rational)
            let minutes = if coords.len() > 1 && coords[1].1 != 0 {
                coords[1].0 as f64 / coords[1].1 as f64
            } else {
                0.0 // ExifTool: ($m || 0)
            };

            // Extract seconds (third rational)
            let seconds = if coords.len() > 2 && coords[2].1 != 0 {
                coords[2].0 as f64 / coords[2].1 as f64
            } else {
                0.0 // ExifTool: ($s || 0)/60
            };

            // ExifTool formula: $deg = $d + (($m || 0) + ($s || 0)/60) / 60;
            let decimal_degrees = degrees + ((minutes + seconds / 60.0) / 60.0);

            Ok(TagValue::F64(decimal_degrees))
        }
        _ => Err(ExifError::ParseError(
            "GPS coordinate conversion requires rational array with at least 3 elements"
                .to_string(),
        )),
    }
}

/// APEX shutter speed conversion: 2^-val to actual shutter speed
///
/// ExifTool: lib/Image/ExifTool/Exif.pm line 3826
/// ShutterSpeedValue is stored as APEX value where actual_speed = 2^(-apex_value)
/// ExifTool ValueConv: 'IsFloat($val) && abs($val)<100 ? 2**(-$val) : 0'
pub fn apex_shutter_speed_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_f64() {
        Some(apex_val) => {
            // Trust ExifTool: boundary check prevents computing 2^(very_large_number)
            // for invalid/corrupt APEX values (like -2147483648 in Canon.jpg)
            if apex_val.abs() < 100.0 {
                let shutter_speed = (-apex_val).exp2(); // 2^(-val)
                Ok(TagValue::F64(shutter_speed))
            } else {
                // ExifTool returns 0 for invalid APEX values (abs >= 100)
                Ok(TagValue::F64(0.0))
            }
        }
        None => Err(ExifError::ParseError(
            "APEX shutter speed conversion requires numeric value".to_string(),
        )),
    }
}

/// APEX aperture conversion: 2^(val/2) to f-number
///
/// ExifTool: lib/Image/ExifTool/Exif.pm line 3827  
/// ApertureValue is stored as APEX value where f_number = 2^(apex_value/2)
pub fn apex_aperture_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_f64() {
        Some(apex_val) => {
            let f_number = (apex_val / 2.0).exp2(); // 2^(val/2)
            Ok(TagValue::F64(f_number))
        }
        None => Err(ExifError::ParseError(
            "APEX aperture conversion requires numeric value".to_string(),
        )),
    }
}

/// APEX exposure compensation conversion
///
/// ExifTool: lib/Image/ExifTool/Exif.pm ExposureCompensation
/// Usually no conversion needed - value is already in EV stops
pub fn apex_exposure_compensation_value_conv(value: &TagValue) -> Result<TagValue> {
    // Most exposure compensation values are already in the correct format
    // Just ensure we have a consistent numeric representation
    match value.as_f64() {
        Some(ev_value) => Ok(TagValue::F64(ev_value)),
        None => Ok(value.clone()), // Pass through if not numeric
    }
}

/// FNumber conversion from rational to f-stop notation
///
/// ExifTool: lib/Image/ExifTool/Exif.pm FNumber
/// Converts rational like [4, 1] to decimal 4.0 for f/4.0 display
pub fn fnumber_value_conv(value: &TagValue) -> Result<TagValue> {
    match value {
        TagValue::Rational(num, denom) => {
            if *denom != 0 {
                let f_number = *num as f64 / *denom as f64;
                Ok(TagValue::F64(f_number))
            } else {
                Err(ExifError::ParseError(
                    "FNumber has zero denominator".to_string(),
                ))
            }
        }
        // Already converted or different format
        _ => Ok(value.clone()),
    }
}

/// GPS timestamp conversion
///
/// ExifTool: lib/Image/ExifTool/GPS.pm GPSTimeStamp
/// Converts rational array [hours/1, minutes/1, seconds/100] to "HH:MM:SS" format
pub fn gpstimestamp_value_conv(value: &TagValue) -> Result<TagValue> {
    match value {
        TagValue::RationalArray(rationals) if rationals.len() >= 3 => {
            let hours = if rationals[0].1 != 0 {
                rationals[0].0 / rationals[0].1
            } else {
                0
            };

            let minutes = if rationals[1].1 != 0 {
                rationals[1].0 / rationals[1].1
            } else {
                0
            };

            let seconds = if rationals[2].1 != 0 {
                rationals[2].0 / rationals[2].1
            } else {
                0
            };

            // Format as "HH:MM:SS"
            let time_string = format!("{hours:02}:{minutes:02}:{seconds:02}");
            Ok(TagValue::String(time_string))
        }
        _ => Err(ExifError::ParseError(
            "GPS timestamp conversion requires rational array with at least 3 elements".to_string(),
        )),
    }
}

/// GPS date stamp conversion (placeholder)
///
/// ExifTool: lib/Image/ExifTool/GPS.pm GPSDateStamp
/// TODO: Implement date parsing when we encounter actual GPS date formats
pub fn gpsdatestamp_value_conv(value: &TagValue) -> Result<TagValue> {
    // For now, pass through - implement when we see actual GPS date formats
    Ok(value.clone())
}

/// Canon AutoISO conversion: exp($val/32*log(2))*100
///
/// ExifTool: lib/Image/ExifTool/Canon.pm AutoISO ValueConv
/// Formula: exp($val/32*log(2))*100
pub fn canon_auto_iso_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_f64() {
        Some(val) => {
            // exp($val/32*log(2))*100 = exp($val * ln(2) / 32) * 100 = 2^($val/32) * 100
            let iso_value = (val / 32.0).exp2() * 100.0;
            Ok(TagValue::F64(iso_value))
        }
        None => Err(ExifError::ParseError(
            "Canon AutoISO conversion requires numeric value".to_string(),
        )),
    }
}

/// Canon BaseISO conversion: exp($val/32*log(2))*100/32
///
/// ExifTool: lib/Image/ExifTool/Canon.pm BaseISO ValueConv
/// Formula: exp($val/32*log(2))*100/32
pub fn canon_base_iso_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_f64() {
        Some(val) => {
            // exp($val/32*log(2))*100/32 = 2^($val/32) * 100 / 32
            let iso_value = (val / 32.0).exp2() * 100.0 / 32.0;
            Ok(TagValue::F64(iso_value))
        }
        None => Err(ExifError::ParseError(
            "Canon BaseISO conversion requires numeric value".to_string(),
        )),
    }
}

/// Canon simple division: $val / 32 + 5
///
/// ExifTool: lib/Image/ExifTool/Canon.pm various tags
/// Formula: $val / 32 + 5
pub fn canon_div_32_plus_5_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_f64() {
        Some(val) => Ok(TagValue::F64(val / 32.0 + 5.0)),
        None => Err(ExifError::ParseError(
            "Canon division conversion requires numeric value".to_string(),
        )),
    }
}

/// Canon simple division: $val / 10
///
/// ExifTool: lib/Image/ExifTool/Canon.pm various tags
/// Formula: $val / 10
pub fn canon_div_10_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_f64() {
        Some(val) => Ok(TagValue::F64(val / 10.0)),
        None => Err(ExifError::ParseError(
            "Canon division conversion requires numeric value".to_string(),
        )),
    }
}

/// Canon simple division: $val / 100
///
/// ExifTool: lib/Image/ExifTool/Canon.pm various tags
/// Formula: $val / 100
pub fn canon_div_100_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_f64() {
        Some(val) => Ok(TagValue::F64(val / 100.0)),
        None => Err(ExifError::ParseError(
            "Canon division conversion requires numeric value".to_string(),
        )),
    }
}

/// Canon addition: $val + 1
///
/// ExifTool: lib/Image/ExifTool/Canon.pm various tags
/// Formula: $val + 1
pub fn canon_plus_1_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_f64() {
        Some(val) => Ok(TagValue::F64(val + 1.0)),
        None => Err(ExifError::ParseError(
            "Canon addition conversion requires numeric value".to_string(),
        )),
    }
}

/// Canon millimeter conversion: $val * 25.4 / 1000
///
/// ExifTool: lib/Image/ExifTool/Canon.pm sensor size conversions
/// Formula: $val * 25.4 / 1000 (converts from unknown units to millimeters)
pub fn canon_millimeter_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_f64() {
        Some(val) => Ok(TagValue::F64(val * 25.4 / 1000.0)),
        None => Err(ExifError::ParseError(
            "Canon millimeter conversion requires numeric value".to_string(),
        )),
    }
}

/// Canon bit shift operations for file numbers: ($val>>16)|(($val&0xffff)<<16)
///
/// ExifTool: lib/Image/ExifTool/Canon.pm FileNumber ValueConv
/// Formula: ($val>>16)|(($val&0xffff)<<16) - swaps 16-bit halves
pub fn canon_file_number_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_u32() {
        Some(val) => {
            let result = (val >> 16) | ((val & 0xffff) << 16);
            Ok(TagValue::U32(result))
        }
        None => Err(ExifError::ParseError(
            "Canon file number conversion requires integer value".to_string(),
        )),
    }
}

/// Canon complex directory number conversion
///
/// ExifTool: lib/Image/ExifTool/Canon.pm DirectoryNumber ValueConv
/// Complex bit manipulation for directory numbers
pub fn canon_directory_number_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_u32() {
        Some(val) => {
            // (($val&0xffc0)>>6)*10000+(($val>>16)&0xff)+(($val&0x3f)<<8)
            let result = ((val & 0xffc0) >> 6) * 10000 + ((val >> 16) & 0xff) + ((val & 0x3f) << 8);
            Ok(TagValue::U32(result))
        }
        None => Err(ExifError::ParseError(
            "Canon directory number conversion requires integer value".to_string(),
        )),
    }
}

/// White balance ValueConv (placeholder)
///
/// ExifTool: lib/Image/ExifTool/Exif.pm WhiteBalance
/// TODO: Implement white balance conversion when we encounter specific formats
pub fn whitebalance_value_conv(value: &TagValue) -> Result<TagValue> {
    // For now, pass through - implement when we see actual white balance formats needing conversion
    Ok(value.clone())
}

/// ExposureTime ValueConv - converts rational to decimal seconds
/// ExifTool: lib/Image/ExifTool/Exif.pm ExposureTime ValueConv
pub fn exposuretime_value_conv(value: &TagValue) -> Result<TagValue> {
    match value {
        TagValue::Rational(num, denom) => {
            if *denom != 0 {
                let exposure_time = *num as f64 / *denom as f64;
                Ok(TagValue::F64(exposure_time))
            } else {
                Err(ExifError::ParseError(
                    "ExposureTime has zero denominator".to_string(),
                ))
            }
        }
        // Already converted or different format
        _ => Ok(value.clone()),
    }
}

/// FocalLength ValueConv - converts rational to decimal millimeters
/// ExifTool: lib/Image/ExifTool/Exif.pm FocalLength ValueConv
pub fn focallength_value_conv(value: &TagValue) -> Result<TagValue> {
    match value {
        TagValue::Rational(num, denom) => {
            if *denom != 0 {
                let focal_length = *num as f64 / *denom as f64;
                Ok(TagValue::F64(focal_length))
            } else {
                Err(ExifError::ParseError(
                    "FocalLength has zero denominator".to_string(),
                ))
            }
        }
        // Already converted or different format
        _ => Ok(value.clone()),
    }
}

/// Trim trailing whitespace from string values
/// ExifTool pattern: $val=~s/ +$//; $val
pub fn trim_whitespace_value_conv(value: &TagValue) -> Result<TagValue> {
    match value {
        TagValue::String(s) => Ok(TagValue::String(s.trim_end().to_string())),
        _ => Ok(value.clone()),
    }
}

/// Multiply value by 100
/// ExifTool pattern: $val * 100
pub fn multiply_100_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_f64() {
        Some(val) => Ok(TagValue::F64(val * 100.0)),
        None => Ok(value.clone()),
    }
}

/// Divide value by 8
/// ExifTool pattern: $val / 8
pub fn divide_8_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_f64() {
        Some(val) => Ok(TagValue::F64(val / 8.0)),
        None => Ok(value.clone()),
    }
}

/// Divide value by 256
/// ExifTool pattern: $val / 256
pub fn divide_256_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_f64() {
        Some(val) => Ok(TagValue::F64(val / 256.0)),
        None => Ok(value.clone()),
    }
}

/// Subtract 5 from value
/// ExifTool pattern: $val - 5
pub fn subtract_5_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_f64() {
        Some(val) => Ok(TagValue::F64(val - 5.0)),
        None => Ok(value.clone()),
    }
}

/// Add 3 to value
/// ExifTool pattern: $val + 3
pub fn add_3_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_f64() {
        Some(val) => Ok(TagValue::F64(val + 3.0)),
        None => Ok(value.clone()),
    }
}

/// Power function: 2 ** (-$val/3)
/// ExifTool pattern: 2 ** (-$val/3)
pub fn power_neg_div_3_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_f64() {
        Some(val) => Ok(TagValue::F64(2.0_f64.powf(-val / 3.0))),
        None => Ok(value.clone()),
    }
}

/// Divide value by 6
/// ExifTool pattern: $val/6
pub fn divide_6_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_f64() {
        Some(val) => Ok(TagValue::F64(val / 6.0)),
        None => Ok(value.clone()),
    }
}

/// Subtract 104 and divide by 8
/// ExifTool pattern: ($val-104)/8
pub fn subtract_104_divide_8_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_f64() {
        Some(val) => Ok(TagValue::F64((val - 104.0) / 8.0)),
        None => Ok(value.clone()),
    }
}

/// Reciprocal multiplied by 10: $val ? 10 / $val : 0
/// ExifTool pattern: $val ? 10 / $val : 0
pub fn reciprocal_10_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_f64() {
        Some(val) if val != 0.0 => Ok(TagValue::F64(10.0 / val)),
        _ => Ok(TagValue::F64(0.0)),
    }
}

/// Sony exposure time conversion: $val ? 2 ** (6 - $val/8) : 0
/// ExifTool pattern: $val ? 2 ** (6 - $val/8) : 0
pub fn sony_exposure_time_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_f64() {
        Some(val) if val != 0.0 => Ok(TagValue::F64(2.0_f64.powf(6.0 - val / 8.0))),
        _ => Ok(TagValue::F64(0.0)),
    }
}

/// Sony ISO conversion: $val ? exp(($val/8-6)*log(2))*100 : $val
/// ExifTool pattern: $val ? exp(($val/8-6)*log(2))*100 : $val
pub fn sony_iso_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_f64() {
        Some(val) if val != 0.0 => {
            let result = ((val / 8.0 - 6.0) * 2.0_f64.ln()).exp() * 100.0;
            Ok(TagValue::F64(result))
        }
        _ => Ok(value.clone()),
    }
}

/// Sony FNumber conversion: 2 ** (($val/8 - 1) / 2)
/// ExifTool pattern: 2 ** (($val/8 - 1) / 2)
pub fn sony_fnumber_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_f64() {
        Some(val) => Ok(TagValue::F64(2.0_f64.powf((val / 8.0 - 1.0) / 2.0))),
        None => Ok(value.clone()),
    }
}

/// EXIF date conversion
/// ExifTool pattern: Image::ExifTool::Exif::ExifDate($val)
pub fn exif_date_value_conv(value: &TagValue) -> Result<TagValue> {
    // For now, pass through - implement specific date conversion when needed
    Ok(value.clone())
}

/// XMP date conversion
/// ExifTool pattern: require Image::ExifTool::XMP; return Image::ExifTool::XMP::ConvertXMPDate($val);
pub fn xmp_date_value_conv(value: &TagValue) -> Result<TagValue> {
    // For now, pass through - implement specific XMP date conversion when needed
    Ok(value.clone())
}

/// Reference long strings (> 32 chars) to avoid duplication
/// ExifTool pattern: length($val) > 32 ? \$val : $val
pub fn reference_long_string_value_conv(value: &TagValue) -> Result<TagValue> {
    match value {
        TagValue::String(s) if s.len() > 32 => {
            // In Rust, we don't have references like Perl, so just return the string
            Ok(value.clone())
        }
        _ => Ok(value.clone()),
    }
}

/// Reference very long strings (> 64 chars) to avoid duplication
/// ExifTool pattern: length($val) > 64 ? \$val : $val
pub fn reference_very_long_string_value_conv(value: &TagValue) -> Result<TagValue> {
    match value {
        TagValue::String(s) if s.len() > 64 => {
            // In Rust, we don't have references like Perl, so just return the string
            Ok(value.clone())
        }
        _ => Ok(value.clone()),
    }
}

/// Remove prefix up to ": " from string values
/// ExifTool pattern: $val=~s/^.*: //;$val
pub fn remove_prefix_colon_value_conv(value: &TagValue) -> Result<TagValue> {
    match value {
        TagValue::String(s) => {
            if let Some(pos) = s.find(": ") {
                // Remove everything up to and including ": "
                let trimmed = &s[pos + 2..];
                Ok(TagValue::String(trimmed.to_string()))
            } else {
                // No ": " found, return original
                Ok(value.clone())
            }
        }
        _ => Ok(value.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gps_coordinate_conversion() {
        // Test typical GPS coordinate: 40° 26' 46.8" = 40.446333...
        let coords = vec![(40, 1), (26, 1), (468, 10)]; // 46.8 seconds as 468/10
        let coord_value = TagValue::RationalArray(coords);

        let result = gps_coordinate_value_conv(&coord_value).unwrap();
        if let TagValue::F64(decimal) = result {
            // 40 + 26/60 + 46.8/3600 = 40.446333...
            assert!((decimal - 40.446333333).abs() < 0.000001);
        } else {
            panic!("Expected F64 result");
        }
    }

    #[test]
    fn test_gps_coordinate_precision() {
        // Test high precision: 12° 34' 56.789"
        let coords = vec![(12, 1), (34, 1), (56789, 1000)]; // 56.789 seconds
        let coord_value = TagValue::RationalArray(coords);

        let result = gps_coordinate_value_conv(&coord_value).unwrap();
        if let TagValue::F64(decimal) = result {
            // 12 + 34/60 + 56.789/3600 = 12.582441388...
            let expected = 12.0 + 34.0 / 60.0 + 56.789 / 3600.0;
            assert!((decimal - expected).abs() < 0.0000001);
        } else {
            panic!("Expected F64 result");
        }
    }

    #[test]
    fn test_gps_coordinate_zero_values() {
        // Test coordinates at exactly 0° 0' 0"
        let coords = vec![(0, 1), (0, 1), (0, 1)];
        let coord_value = TagValue::RationalArray(coords);

        let result = gps_coordinate_value_conv(&coord_value).unwrap();
        if let TagValue::F64(decimal) = result {
            assert_eq!(decimal, 0.0);
        } else {
            panic!("Expected F64 result");
        }
    }

    #[test]
    fn test_gps_coordinate_only_degrees() {
        // Test coordinate with only degrees: 45° 0' 0"
        let coords = vec![(45, 1), (0, 1), (0, 1)];
        let coord_value = TagValue::RationalArray(coords);

        let result = gps_coordinate_value_conv(&coord_value).unwrap();
        if let TagValue::F64(decimal) = result {
            assert_eq!(decimal, 45.0);
        } else {
            panic!("Expected F64 result");
        }
    }

    #[test]
    fn test_gps_coordinate_zero_denominators() {
        // Test handling of zero denominators (should be treated as 0)
        let coords = vec![(40, 1), (30, 0), (45, 1)]; // minutes has zero denominator
        let coord_value = TagValue::RationalArray(coords);

        let result = gps_coordinate_value_conv(&coord_value).unwrap();
        if let TagValue::F64(decimal) = result {
            // 40 + 0/60 + 45/3600 = 40.0125
            assert!((decimal - 40.0125).abs() < 0.0001);
        } else {
            panic!("Expected F64 result");
        }
    }

    #[test]
    fn test_gps_coordinate_invalid_input() {
        // Test with wrong type
        let value = TagValue::String("40.446333".to_string());
        let result = gps_coordinate_value_conv(&value);
        assert!(matches!(result, Err(ExifError::ParseError(_))));

        // Test with too few elements
        let coords = vec![(40, 1), (26, 1)]; // Only 2 elements instead of 3
        let coord_value = TagValue::RationalArray(coords);
        let result = gps_coordinate_value_conv(&coord_value);
        assert!(matches!(result, Err(ExifError::ParseError(_))));

        // Test with empty array
        let coords = vec![];
        let coord_value = TagValue::RationalArray(coords);
        let result = gps_coordinate_value_conv(&coord_value);
        assert!(matches!(result, Err(ExifError::ParseError(_))));
    }

    #[test]
    fn test_apex_shutter_speed() {
        // APEX value 11 should give shutter speed of 2^(-11) = 1/2048 ≈ 0.00048828125
        let apex_value = TagValue::F64(11.0);
        let result = apex_shutter_speed_value_conv(&apex_value).unwrap();

        if let TagValue::F64(speed) = result {
            assert!((speed - 0.00048828125).abs() < 0.000001);
        } else {
            panic!("Expected F64 result");
        }
    }

    #[test]
    fn test_apex_aperture() {
        // APEX aperture value 4 should give f-number of 2^(4/2) = 2^2 = 4.0
        let apex_value = TagValue::F64(4.0);
        let result = apex_aperture_value_conv(&apex_value).unwrap();

        if let TagValue::F64(f_number) = result {
            assert!((f_number - 4.0).abs() < 0.001);
        } else {
            panic!("Expected F64 result");
        }
    }

    #[test]
    fn test_fnumber_conversion() {
        // Rational [4, 1] should convert to 4.0
        let fnumber_rational = TagValue::Rational(4, 1);
        let result = fnumber_value_conv(&fnumber_rational).unwrap();

        if let TagValue::F64(f_number) = result {
            assert!((f_number - 4.0).abs() < 0.001);
        } else {
            panic!("Expected F64 result");
        }
    }
}
