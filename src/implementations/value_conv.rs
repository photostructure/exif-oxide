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
