//! ValueConv implementations for exif-oxide
//!
//! ValueConv functions perform mathematical conversions on raw tag values
//! to produce logical values. Unlike PrintConv which formats for display,
//! ValueConv maintains precision for further calculations and round-trip operations.
//!
//! All implementations are direct translations from ExifTool source code.

use crate::types::{ExifError, Result, TagValue};

/// GPS coordinate conversion from degrees/minutes/seconds to decimal degrees
///
/// ExifTool: lib/Image/ExifTool/GPS.pm ToDegrees function
/// GPS coordinates are stored as [degrees/1, minutes/1, seconds/100] arrays
pub fn gps_coordinate_value_conv(value: &TagValue) -> Result<TagValue> {
    match value {
        TagValue::RationalArray(rationals) if rationals.len() >= 3 => {
            let degrees = if rationals[0].1 != 0 {
                rationals[0].0 as f64 / rationals[0].1 as f64
            } else {
                0.0
            };

            let minutes = if rationals[1].1 != 0 {
                (rationals[1].0 as f64 / rationals[1].1 as f64) / 60.0
            } else {
                0.0
            };

            let seconds = if rationals[2].1 != 0 {
                (rationals[2].0 as f64 / rationals[2].1 as f64) / 3600.0
            } else {
                0.0
            };

            let decimal_degrees = degrees + minutes + seconds;
            Ok(TagValue::F64(decimal_degrees))
        }
        _ => Err(ExifError::ParseError(
            "GPS coordinate conversion requires rational array with at least 3 elements"
                .to_string(),
        )),
    }
}

/// GPS latitude ValueConv - uses common GPS coordinate conversion
/// ExifTool: lib/Image/ExifTool/GPS.pm GPSLatitude
/// Note: This produces the raw EXIF coordinate (always positive)
/// Composite tags will apply hemisphere reference for signed coordinates
pub fn gpslatitude_value_conv(value: &TagValue) -> Result<TagValue> {
    gps_coordinate_value_conv(value)
}

/// GPS longitude ValueConv - uses common GPS coordinate conversion  
/// ExifTool: lib/Image/ExifTool/GPS.pm GPSLongitude
/// Note: This produces the raw EXIF coordinate (always positive)
/// Composite tags will apply hemisphere reference for signed coordinates
pub fn gpslongitude_value_conv(value: &TagValue) -> Result<TagValue> {
    gps_coordinate_value_conv(value)
}

/// GPS destination latitude ValueConv
/// ExifTool: lib/Image/ExifTool/GPS.pm GPSDestLatitude
pub fn gpsdestlatitude_value_conv(value: &TagValue) -> Result<TagValue> {
    gps_coordinate_value_conv(value)
}

/// GPS destination longitude ValueConv
/// ExifTool: lib/Image/ExifTool/GPS.pm GPSDestLongitude  
pub fn gpsdestlongitude_value_conv(value: &TagValue) -> Result<TagValue> {
    gps_coordinate_value_conv(value)
}

/// APEX shutter speed conversion: 2^-val to actual shutter speed
///
/// ExifTool: lib/Image/ExifTool/Exif.pm line 3826
/// ShutterSpeedValue is stored as APEX value where actual_speed = 2^(-apex_value)
pub fn apex_shutter_speed_value_conv(value: &TagValue) -> Result<TagValue> {
    match value.as_f64() {
        Some(apex_val) => {
            let shutter_speed = (-apex_val).exp2(); // 2^(-val)
            Ok(TagValue::F64(shutter_speed))
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
        // Test GPS coordinate: 54 degrees, 59.38 minutes, 0 seconds
        // Should convert to: 54 + 59.38/60 + 0/3600 = 54 + 0.98966... = 54.98966...
        let rationals = vec![(54, 1), (5938, 100), (0, 1)]; // 54 degrees, 59.38 minutes, 0 seconds
        let gps_value = TagValue::RationalArray(rationals);

        let result = gps_coordinate_value_conv(&gps_value).unwrap();
        if let TagValue::F64(decimal) = result {
            let expected = 54.0 + (59.38 / 60.0) + (0.0 / 3600.0);
            assert!(
                (decimal - expected).abs() < 0.001,
                "Expected {expected}, got {decimal}"
            );
        } else {
            panic!("Expected F64 result");
        }
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
