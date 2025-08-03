//! Integration test for GPS ValueConv registry fix (P16a)
//!
//! This test verifies that GPS coordinate conversion uses the registry-delegated
//! functions instead of trying to compile GPS function expressions.

use exif_oxide::types::TagValue;

#[test]
fn test_gps_latitude_valueconv_delegation() {
    // Test GPS latitude coordinate conversion using generated GPS module
    // This verifies our registry fix works in the actual codegen pipeline

    let coords = vec![(40, 1), (26, 1), (468, 10)]; // 40° 26' 46.8"
    let coord_value = TagValue::RationalArray(coords);

    // Call the generated GPS module's ValueConv function directly
    let result = exif_oxide::generated::GPS_pm::tag_kit::apply_value_conv(
        2, // GPSLatitude tag ID
        &coord_value,
        &mut Vec::new(), // errors
    )
    .unwrap();

    // Verify it returns decimal degrees, not missing function call
    match result {
        TagValue::F64(decimal) => {
            // Verify correct decimal conversion: 40° 26' 46.8" = 40.446333...
            assert!(
                (decimal - 40.446333333).abs() < 0.000001,
                "Expected ~40.446333, got {}",
                decimal
            );
            assert!(decimal > 0.0, "GPS ValueConv should return unsigned values");
        }
        other => panic!("Expected F64 decimal degrees, got {:?}", other),
    }
}

#[test]
fn test_gps_longitude_valueconv_delegation() {
    // Test GPS longitude coordinate conversion
    let coords = vec![(118, 1), (14, 1), (2208, 100)]; // 118° 14' 22.08"
    let coord_value = TagValue::RationalArray(coords);

    // Call the generated GPS module's ValueConv function for longitude
    let result = exif_oxide::generated::GPS_pm::tag_kit::apply_value_conv(
        4, // GPSLongitude tag ID
        &coord_value,
        &mut Vec::new(),
    )
    .unwrap();

    match result {
        TagValue::F64(decimal) => {
            // Verify correct decimal conversion: 118° 14' 22.08" = 118.2394...
            assert!(
                (decimal - 118.2394444).abs() < 0.0001,
                "Expected ~118.2394444, got {}",
                decimal
            );
            assert!(decimal > 0.0, "GPS ValueConv should return unsigned values");
        }
        other => panic!("Expected F64 decimal degrees, got {:?}", other),
    }
}

#[test]
fn test_gps_dest_latitude_valueconv_delegation() {
    // Test GPS destination latitude uses same delegation
    let coords = vec![(34, 1), (3, 1), (84, 10)]; // 34° 3' 8.4"
    let coord_value = TagValue::RationalArray(coords);

    let result = exif_oxide::generated::GPS_pm::tag_kit::apply_value_conv(
        20, // GPSDestLatitude tag ID
        &coord_value,
        &mut Vec::new(),
    )
    .unwrap();

    match result {
        TagValue::F64(decimal) => {
            // Verify correct conversion: 34° 3' 8.4" = 34.0523333...
            assert!(
                (decimal - 34.052333333).abs() < 0.000001,
                "Expected ~34.052333, got {}",
                decimal
            );
        }
        other => panic!("Expected F64 decimal degrees, got {:?}", other),
    }
}

#[test]
fn test_gps_dest_longitude_valueconv_delegation() {
    // Test GPS destination longitude
    let coords = vec![(118, 1), (24, 1), (1385, 100)]; // 118° 24' 13.85"
    let coord_value = TagValue::RationalArray(coords);

    let result = exif_oxide::generated::GPS_pm::tag_kit::apply_value_conv(
        22, // GPSDestLongitude tag ID
        &coord_value,
        &mut Vec::new(),
    )
    .unwrap();

    match result {
        TagValue::F64(decimal) => {
            // Verify correct conversion: 118° 24' 13.85" = 118.4038472...
            assert!(
                (decimal - 118.4038472).abs() < 0.0001,
                "Expected ~118.4038472, got {}",
                decimal
            );
        }
        other => panic!("Expected F64 decimal degrees, got {:?}", other),
    }
}

#[test]
fn test_gps_timestamp_valueconv_delegation() {
    // Test GPS timestamp conversion also uses registry delegation
    let timestamp = vec![(14, 1), (32, 1), (2540, 100)]; // 14:32:25.40
    let timestamp_value = TagValue::RationalArray(timestamp);

    let result = exif_oxide::generated::GPS_pm::tag_kit::apply_value_conv(
        7, // GPSTimeStamp tag ID
        &timestamp_value,
        &mut Vec::new(),
    )
    .unwrap();

    // GPS timestamp should be converted to a formatted string by our implementation
    match result {
        TagValue::String(time_str) => {
            // Should be formatted as HH:MM:SS.ss
            assert!(
                time_str.starts_with("14:32:25"),
                "Expected time starting with '14:32:25', got '{}'",
                time_str
            );
        }
        other => panic!("Expected String timestamp, got {:?}", other),
    }
}

#[test]
fn test_registry_fix_prevents_missing_function_calls() {
    // Verify that we DON'T get missing function calls for GPS expressions
    // This is the core regression test for P16a

    let coords = vec![(51, 1), (30, 1), (0, 1)]; // 51° 30' 0"
    let coord_value = TagValue::RationalArray(coords);

    let result = exif_oxide::generated::GPS_pm::tag_kit::apply_value_conv(
        2, // GPSLatitude
        &coord_value,
        &mut Vec::new(),
    )
    .unwrap();

    // Critical: Should NOT be a missing function string
    match result {
        TagValue::String(s) if s.contains("missing_print_conv") => {
            panic!("Registry fix failed! Got missing function call: {}", s);
        }
        TagValue::String(s) if s.contains("Image::ExifTool::GPS::ToDegrees") => {
            panic!("Registry fix failed! Got uncompiled expression: {}", s);
        }
        TagValue::F64(decimal) => {
            // This is what we want - actual decimal conversion
            assert!((decimal - 51.5).abs() < 0.001);
            println!("✅ Registry fix working: GPS coordinates return decimal degrees");
        }
        other => panic!("Unexpected result type: {:?}", other),
    }
}
