//! Test ExposureTime PrintConv formatting

use exif_oxide::types::TagValue;

#[test]
fn test_exposuretime_printconv_json_formatting() {
    // Test the JSON serialization of ExposureTime values
    // This tests the fix for ExposureTime showing as [1, 100] instead of "1/100"

    // Create ExifData with an ExposureTime tag entry
    let mut exif_data =
        exif_oxide::types::ExifData::new("test.jpg".to_string(), "0.1.0-oxide".to_string());

    // Add an ExposureTime tag entry with rational value
    let exposure_entry = exif_oxide::types::TagEntry {
        group: "EXIF".to_string(),
        group1: "ExifIFD".to_string(),
        name: "ExposureTime".to_string(),
        value: TagValue::Rational(1, 100),
        print: "1/100".into(), // PrintConv produces a string for ExposureTime
    };

    exif_data.tags = vec![exposure_entry];

    // Prepare for serialization (without numeric tags)
    exif_data.prepare_for_serialization(None);

    // Check that the legacy_tags contains the print string
    let exposure_value = exif_data
        .legacy_tags
        .get("EXIF:ExposureTime")
        .expect("EXIF:ExposureTime not found in legacy_tags");

    // It should be a String containing "1/100", not a rational array
    match exposure_value {
        TagValue::String(s) => assert_eq!(
            s, "1/100",
            "ExposureTime should serialize as string '1/100'"
        ),
        _ => panic!("ExposureTime should be a String, got {exposure_value:?}"),
    }

    // Test JSON serialization to ensure it's not an array
    let json = serde_json::to_string(&exif_data).expect("Failed to serialize");

    // The JSON should contain "ExposureTime":"1/100" not "ExposureTime":[1,100]
    assert!(
        json.contains(r#""EXIF:ExposureTime":"1/100""#),
        "JSON should contain ExposureTime as string '1/100', got: {json}"
    );
    assert!(
        !json.contains("[1,100]"),
        "JSON should not contain ExposureTime as array [1,100], got: {json}"
    );
}

#[test]
fn test_various_exposuretime_values() {
    use exif_oxide::implementations::print_conv::exposuretime_print_conv;

    // Test various exposure times match ExifTool formatting

    // Fast shutter speeds (< 0.25001) should be fractional
    assert_eq!(
        exposuretime_print_conv(&TagValue::F64(0.0005)),
        "1/2000".into()
    );
    assert_eq!(
        exposuretime_print_conv(&TagValue::F64(0.01)),
        "1/100".into()
    );
    assert_eq!(exposuretime_print_conv(&TagValue::F64(0.125)), "1/8".into());
    assert_eq!(exposuretime_print_conv(&TagValue::F64(0.25)), "1/4".into());

    // Slower speeds (>= 0.25001) should be decimal
    assert_eq!(exposuretime_print_conv(&TagValue::F64(0.5)), "0.5".into());
    assert_eq!(exposuretime_print_conv(&TagValue::F64(1.0)), "1".into()); // .0 stripped
    assert_eq!(exposuretime_print_conv(&TagValue::F64(2.0)), "2".into()); // .0 stripped
    assert_eq!(exposuretime_print_conv(&TagValue::F64(2.5)), "2.5".into());

    // Test with rational values
    assert_eq!(
        exposuretime_print_conv(&TagValue::Rational(1, 2000)),
        "1/2000".into()
    );
    assert_eq!(
        exposuretime_print_conv(&TagValue::Rational(1, 100)),
        "1/100".into()
    );
    assert_eq!(
        exposuretime_print_conv(&TagValue::Rational(1, 2)),
        "0.5".into()
    );
    assert_eq!(
        exposuretime_print_conv(&TagValue::Rational(2, 1)),
        "2".into()
    );
}
