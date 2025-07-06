//! Comprehensive tests for ValueConv implementations
//!
//! These tests ensure all ValueConv functions handle edge cases properly
//! without panics or runtime explosions, testing valid inputs, edge cases,
//! and error conditions.

use exif_oxide::implementations::value_conv::*;
use exif_oxide::types::{ExifError, TagValue};

/// GPS coordinate conversion tests - matches ExifTool behavior
#[test]
fn test_gps_coordinate_decimal_conversion() {
    // Test coordinate conversion: 40° 26' 46.8" = 40.446333...
    let coords = vec![(40, 1), (26, 1), (468, 10)];
    let coord_value = TagValue::RationalArray(coords);

    let result = gps_coordinate_value_conv(&coord_value).unwrap();
    if let TagValue::F64(decimal) = result {
        assert!((decimal - 40.446333333).abs() < 0.000001);
    } else {
        panic!("Expected F64 result");
    }
}

#[test]
fn test_gps_coordinate_unsigned_values() {
    // GPS coordinate ValueConv produces UNSIGNED values only
    // Sign is applied later in Composite tags using Ref values
    let coords = vec![(45, 1), (30, 1), (0, 1)]; // 45° 30' 0"
    let coord_value = TagValue::RationalArray(coords);

    let result = gps_coordinate_value_conv(&coord_value).unwrap();
    if let TagValue::F64(decimal) = result {
        assert!(decimal > 0.0); // Always positive from ValueConv
        assert!((decimal - 45.5).abs() < 0.001);
    } else {
        panic!("Expected F64 result");
    }
}

#[test]
fn test_apex_shutter_speed_valid() {
    // Normal APEX values
    let apex_value = TagValue::F64(5.0);
    let result = apex_shutter_speed_value_conv(&apex_value).unwrap();
    if let TagValue::F64(speed) = result {
        let expected = 2.0_f64.powf(-5.0); // 2^(-5) = 1/32
        assert!((speed - expected).abs() < 0.000001);
    } else {
        panic!("Expected F64 result");
    }

    // Zero APEX value
    let apex_value = TagValue::F64(0.0);
    let result = apex_shutter_speed_value_conv(&apex_value).unwrap();
    if let TagValue::F64(speed) = result {
        assert_eq!(speed, 1.0); // 2^0 = 1
    } else {
        panic!("Expected F64 result");
    }

    // Negative APEX value (slower shutter speeds)
    let apex_value = TagValue::F64(-2.0);
    let result = apex_shutter_speed_value_conv(&apex_value).unwrap();
    if let TagValue::F64(speed) = result {
        assert_eq!(speed, 4.0); // 2^(-(-2)) = 2^2 = 4
    } else {
        panic!("Expected F64 result");
    }
}

#[test]
fn test_apex_shutter_speed_edge_cases() {
    // Very large positive APEX (very fast shutter)
    let apex_value = TagValue::F64(20.0);
    let result = apex_shutter_speed_value_conv(&apex_value).unwrap();
    if let TagValue::F64(speed) = result {
        let expected = 2.0_f64.powf(-20.0); // Very small number
        assert!(speed > 0.0 && speed < 0.000001);
        assert!((speed - expected).abs() < 0.000001);
    } else {
        panic!("Expected F64 result");
    }

    // Very large negative APEX (very slow shutter)
    let apex_value = TagValue::F64(-10.0);
    let result = apex_shutter_speed_value_conv(&apex_value).unwrap();
    if let TagValue::F64(speed) = result {
        assert_eq!(speed, 1024.0); // 2^10
    } else {
        panic!("Expected F64 result");
    }
}

#[test]
fn test_apex_shutter_speed_invalid() {
    // Non-numeric values
    let value = "5.0".into();
    let result = apex_shutter_speed_value_conv(&value);
    assert!(matches!(result, Err(ExifError::ParseError(_))));

    // Array value
    let value = TagValue::U16Array(vec![5, 0]);
    let result = apex_shutter_speed_value_conv(&value);
    assert!(matches!(result, Err(ExifError::ParseError(_))));
}

#[test]
fn test_apex_aperture_valid() {
    // Normal APEX aperture values
    let apex_value = TagValue::F64(4.0);
    let result = apex_aperture_value_conv(&apex_value).unwrap();
    if let TagValue::F64(f_number) = result {
        assert_eq!(f_number, 4.0); // 2^(4/2) = 2^2 = 4
    } else {
        panic!("Expected F64 result");
    }

    // Zero APEX value
    let apex_value = TagValue::F64(0.0);
    let result = apex_aperture_value_conv(&apex_value).unwrap();
    if let TagValue::F64(f_number) = result {
        assert_eq!(f_number, 1.0); // 2^(0/2) = 2^0 = 1
    } else {
        panic!("Expected F64 result");
    }

    // Odd APEX values
    let apex_value = TagValue::F64(3.0);
    let result = apex_aperture_value_conv(&apex_value).unwrap();
    if let TagValue::F64(f_number) = result {
        let expected = 2.0_f64.powf(1.5); // 2^(3/2) ≈ 2.828
        assert!((f_number - expected).abs() < 0.001);
    } else {
        panic!("Expected F64 result");
    }
}

#[test]
fn test_apex_aperture_edge_cases() {
    // Large APEX value (very small aperture/large f-number)
    let apex_value = TagValue::F64(12.0);
    let result = apex_aperture_value_conv(&apex_value).unwrap();
    if let TagValue::F64(f_number) = result {
        assert_eq!(f_number, 64.0); // 2^(12/2) = 2^6 = 64
    } else {
        panic!("Expected F64 result");
    }

    // Negative APEX value (very large aperture/small f-number)
    let apex_value = TagValue::F64(-2.0);
    let result = apex_aperture_value_conv(&apex_value).unwrap();
    if let TagValue::F64(f_number) = result {
        assert_eq!(f_number, 0.5); // 2^(-2/2) = 2^(-1) = 0.5
    } else {
        panic!("Expected F64 result");
    }
}

#[test]
fn test_fnumber_valid() {
    // Standard f-numbers
    let fnumber = TagValue::Rational(4, 1);
    let result = fnumber_value_conv(&fnumber).unwrap();
    if let TagValue::F64(f) = result {
        assert_eq!(f, 4.0);
    } else {
        panic!("Expected F64 result");
    }

    // Fractional f-numbers
    let fnumber = TagValue::Rational(28, 10);
    let result = fnumber_value_conv(&fnumber).unwrap();
    if let TagValue::F64(f) = result {
        assert_eq!(f, 2.8);
    } else {
        panic!("Expected F64 result");
    }

    // Large denominator
    let fnumber = TagValue::Rational(560, 100);
    let result = fnumber_value_conv(&fnumber).unwrap();
    if let TagValue::F64(f) = result {
        assert_eq!(f, 5.6);
    } else {
        panic!("Expected F64 result");
    }
}

#[test]
fn test_fnumber_edge_cases() {
    // Zero numerator
    let fnumber = TagValue::Rational(0, 1);
    let result = fnumber_value_conv(&fnumber).unwrap();
    if let TagValue::F64(f) = result {
        assert_eq!(f, 0.0);
    } else {
        panic!("Expected F64 result");
    }

    // Zero denominator
    let fnumber = TagValue::Rational(4, 0);
    let result = fnumber_value_conv(&fnumber);
    assert!(matches!(result, Err(ExifError::ParseError(_))));

    // Very large values
    let fnumber = TagValue::Rational(1000000, 10000);
    let result = fnumber_value_conv(&fnumber).unwrap();
    if let TagValue::F64(f) = result {
        assert_eq!(f, 100.0);
    } else {
        panic!("Expected F64 result");
    }
}

#[test]
fn test_fnumber_passthrough() {
    // Already converted F64
    let fnumber = TagValue::F64(2.8);
    let result = fnumber_value_conv(&fnumber).unwrap();
    if let TagValue::F64(f) = result {
        assert_eq!(f, 2.8);
    } else {
        panic!("Expected F64 result");
    }

    // String value (should pass through)
    let fnumber = "f/2.8".into();
    let result = fnumber_value_conv(&fnumber).unwrap();
    if let TagValue::String(s) = result {
        assert_eq!(s, "f/2.8");
    } else {
        panic!("Expected String result");
    }
}

#[test]
fn test_gps_timestamp_valid() {
    // Standard timestamp: 14:30:45
    let rationals = vec![(14, 1), (30, 1), (45, 1)];
    let time_value = TagValue::RationalArray(rationals);

    let result = gpstimestamp_value_conv(&time_value).unwrap();
    if let TagValue::String(time_str) = result {
        assert_eq!(time_str, "14:30:45");
    } else {
        panic!("Expected String result");
    }

    // Timestamp with fractional seconds (should truncate)
    let rationals = vec![(14, 1), (30, 1), (455, 10)];
    let time_value = TagValue::RationalArray(rationals);

    let result = gpstimestamp_value_conv(&time_value).unwrap();
    if let TagValue::String(time_str) = result {
        assert_eq!(time_str, "14:30:45");
    } else {
        panic!("Expected String result");
    }
}

#[test]
fn test_gps_timestamp_edge_cases() {
    // Midnight
    let rationals = vec![(0, 1), (0, 1), (0, 1)];
    let time_value = TagValue::RationalArray(rationals);

    let result = gpstimestamp_value_conv(&time_value).unwrap();
    if let TagValue::String(time_str) = result {
        assert_eq!(time_str, "00:00:00");
    } else {
        panic!("Expected String result");
    }

    // Just before midnight
    let rationals = vec![(23, 1), (59, 1), (59, 1)];
    let time_value = TagValue::RationalArray(rationals);

    let result = gpstimestamp_value_conv(&time_value).unwrap();
    if let TagValue::String(time_str) = result {
        assert_eq!(time_str, "23:59:59");
    } else {
        panic!("Expected String result");
    }

    // Zero denominators (handled as 0)
    let rationals = vec![(14, 0), (30, 0), (45, 0)];
    let time_value = TagValue::RationalArray(rationals);

    let result = gpstimestamp_value_conv(&time_value).unwrap();
    if let TagValue::String(time_str) = result {
        assert_eq!(time_str, "00:00:00");
    } else {
        panic!("Expected String result");
    }
}

#[test]
fn test_gps_timestamp_invalid() {
    // Wrong type
    let value = "14:30:45".into();
    let result = gpstimestamp_value_conv(&value);
    assert!(matches!(result, Err(ExifError::ParseError(_))));

    // Too few elements
    let rationals = vec![(14, 1), (30, 1)];
    let time_value = TagValue::RationalArray(rationals);
    let result = gpstimestamp_value_conv(&time_value);
    assert!(matches!(result, Err(ExifError::ParseError(_))));

    // Empty array
    let rationals = vec![];
    let time_value = TagValue::RationalArray(rationals);
    let result = gpstimestamp_value_conv(&time_value);
    assert!(matches!(result, Err(ExifError::ParseError(_))));
}

#[test]
fn test_exposure_time_valid() {
    // Standard exposure times
    let exposure = TagValue::Rational(1, 60);
    let result = exposuretime_value_conv(&exposure).unwrap();
    if let TagValue::F64(time) = result {
        assert!((time - 1.0 / 60.0).abs() < 0.00001);
    } else {
        panic!("Expected F64 result");
    }

    // Long exposure
    let exposure = TagValue::Rational(30, 1);
    let result = exposuretime_value_conv(&exposure).unwrap();
    if let TagValue::F64(time) = result {
        assert_eq!(time, 30.0);
    } else {
        panic!("Expected F64 result");
    }

    // Very short exposure
    let exposure = TagValue::Rational(1, 8000);
    let result = exposuretime_value_conv(&exposure).unwrap();
    if let TagValue::F64(time) = result {
        assert!((time - 1.0 / 8000.0).abs() < 0.0000001);
    } else {
        panic!("Expected F64 result");
    }
}

#[test]
fn test_exposure_time_edge_cases() {
    // Zero exposure (shouldn't happen but handle gracefully)
    let exposure = TagValue::Rational(0, 1);
    let result = exposuretime_value_conv(&exposure).unwrap();
    if let TagValue::F64(time) = result {
        assert_eq!(time, 0.0);
    } else {
        panic!("Expected F64 result");
    }

    // Zero denominator
    let exposure = TagValue::Rational(1, 0);
    let result = exposuretime_value_conv(&exposure);
    assert!(matches!(result, Err(ExifError::ParseError(_))));

    // Already converted
    let exposure = TagValue::F64(0.5);
    let result = exposuretime_value_conv(&exposure).unwrap();
    if let TagValue::F64(time) = result {
        assert_eq!(time, 0.5);
    } else {
        panic!("Expected F64 result");
    }
}

#[test]
fn test_focal_length_valid() {
    // Standard focal lengths
    let focal = TagValue::Rational(50, 1);
    let result = focallength_value_conv(&focal).unwrap();
    if let TagValue::F64(length) = result {
        assert_eq!(length, 50.0);
    } else {
        panic!("Expected F64 result");
    }

    // Zoom lens at intermediate position
    let focal = TagValue::Rational(185, 10);
    let result = focallength_value_conv(&focal).unwrap();
    if let TagValue::F64(length) = result {
        assert_eq!(length, 18.5);
    } else {
        panic!("Expected F64 result");
    }

    // Telephoto
    let focal = TagValue::Rational(400, 1);
    let result = focallength_value_conv(&focal).unwrap();
    if let TagValue::F64(length) = result {
        assert_eq!(length, 400.0);
    } else {
        panic!("Expected F64 result");
    }
}

#[test]
fn test_focal_length_edge_cases() {
    // Zero focal length (shouldn't happen)
    let focal = TagValue::Rational(0, 1);
    let result = focallength_value_conv(&focal).unwrap();
    if let TagValue::F64(length) = result {
        assert_eq!(length, 0.0);
    } else {
        panic!("Expected F64 result");
    }

    // Zero denominator
    let focal = TagValue::Rational(50, 0);
    let result = focallength_value_conv(&focal);
    assert!(matches!(result, Err(ExifError::ParseError(_))));

    // Already converted
    let focal = TagValue::F64(85.0);
    let result = focallength_value_conv(&focal).unwrap();
    if let TagValue::F64(length) = result {
        assert_eq!(length, 85.0);
    } else {
        panic!("Expected F64 result");
    }
}

#[test]
fn test_apex_exposure_compensation() {
    // Standard EV values
    let ev = TagValue::F64(1.5);
    let result = apex_exposure_compensation_value_conv(&ev).unwrap();
    if let TagValue::F64(value) = result {
        assert_eq!(value, 1.5);
    } else {
        panic!("Expected F64 result");
    }

    // Negative compensation
    let ev = TagValue::F64(-2.0);
    let result = apex_exposure_compensation_value_conv(&ev).unwrap();
    if let TagValue::F64(value) = result {
        assert_eq!(value, -2.0);
    } else {
        panic!("Expected F64 result");
    }

    // Zero compensation
    let ev = TagValue::F64(0.0);
    let result = apex_exposure_compensation_value_conv(&ev).unwrap();
    if let TagValue::F64(value) = result {
        assert_eq!(value, 0.0);
    } else {
        panic!("Expected F64 result");
    }

    // Non-numeric passthrough
    let ev = "+1.5 EV".into();
    let result = apex_exposure_compensation_value_conv(&ev).unwrap();
    if let TagValue::String(s) = result {
        assert_eq!(s, "+1.5 EV");
    } else {
        panic!("Expected String result");
    }
}

#[test]
fn test_placeholder_functions() {
    // Test GPSDateStamp passthrough
    let date = "2024:01:15".into();
    let result = gpsdatestamp_value_conv(&date).unwrap();
    if let TagValue::String(s) = result {
        assert_eq!(s, "2024:01:15");
    } else {
        panic!("Expected String result");
    }

    // Test WhiteBalance passthrough
    let wb = TagValue::U16(1);
    let result = whitebalance_value_conv(&wb).unwrap();
    if let TagValue::U16(val) = result {
        assert_eq!(val, 1);
    } else {
        panic!("Expected U16 result");
    }
}

/// Test that no conversion function panics on any input type
#[test]
fn test_no_panics_on_any_input() {
    // Create a variety of TagValue types
    let test_values = vec![
        "test".into(),
        TagValue::U8(42),
        TagValue::U16(1000),
        TagValue::U32(100000),
        TagValue::I16(-500),
        TagValue::I32(-100000),
        TagValue::F64(std::f64::consts::E),
        TagValue::Rational(1, 2),
        TagValue::SRational(-1, 2),
        TagValue::U8Array(vec![1, 2, 3]),
        TagValue::U16Array(vec![100, 200]),
        TagValue::U32Array(vec![10000, 20000]),
        TagValue::F64Array(vec![3.3, 4.4]),
        TagValue::RationalArray(vec![(1, 2), (3, 4)]),
        TagValue::SRationalArray(vec![(-1, 2), (-3, 4)]),
        TagValue::Binary(vec![0xFF, 0xFE]),
    ];

    // Test each conversion function with each value type
    // None should panic - they should either succeed or return an error
    for value in &test_values {
        // GPS conversions
        let _ = gps_coordinate_value_conv(value);
        let _ = gpstimestamp_value_conv(value);
        let _ = gpsdatestamp_value_conv(value);

        // APEX conversions
        let _ = apex_shutter_speed_value_conv(value);
        let _ = apex_aperture_value_conv(value);
        let _ = apex_exposure_compensation_value_conv(value);

        // Other conversions
        let _ = fnumber_value_conv(value);
        let _ = exposuretime_value_conv(value);
        let _ = focallength_value_conv(value);
        let _ = whitebalance_value_conv(value);
    }
}

/// Integration test: Verify conversion chaining works properly
#[test]
fn test_value_conv_to_print_conv_chaining() {
    // GPS coordinate conversion removed in Milestone 8e
    // GPS coordinates now return raw rational arrays

    // Test APEX aperture -> f-number -> formatted string
    let apex_value = TagValue::F64(5.0);
    let f_number_result = apex_aperture_value_conv(&apex_value).unwrap();

    if let TagValue::F64(f_num) = f_number_result {
        // Would be formatted as "f/5.7" by PrintConv
        assert!((f_num - 5.656854).abs() < 0.00001);
    } else {
        panic!("Expected F64 from ValueConv");
    }
}

/// Stress test with extreme values
#[test]
fn test_extreme_values() {
    // GPS coordinate extreme value tests removed in Milestone 8e
    // GPS coordinates now return raw rational arrays

    // Test APEX with extreme values
    let extreme_apex = TagValue::F64(50.0);
    let result = apex_shutter_speed_value_conv(&extreme_apex);
    assert!(result.is_ok());

    let extreme_apex = TagValue::F64(-50.0);
    let result = apex_shutter_speed_value_conv(&extreme_apex);
    assert!(result.is_ok());
}
