//! Integration tests for tag kit system
//! Compares tag kit output with manual implementations to ensure parity

use exif_oxide::expressions::ExpressionEvaluator;
use exif_oxide::generated::Exif_pm::tag_kit::{apply_print_conv, TAG_KITS as EXIF_TAG_KITS};
use exif_oxide::implementations::print_conv;
use exif_oxide::types::TagValue;

#[test]
fn test_resolution_unit_tag_kit_parity() {
    // Test all ResolutionUnit values
    let test_cases = vec![
        (1u16, "None"),
        (2u16, "inches"),
        (3u16, "cm"),
        (999u16, "Unknown (999)"), // Unknown value
    ];

    for (value, expected) in test_cases {
        let tag_value = TagValue::U16(value);

        // Get manual implementation result
        let manual_result = print_conv::resolutionunit_print_conv(&tag_value);

        // Get tag kit result (tag 296 = ResolutionUnit)
        let mut evaluator = ExpressionEvaluator::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let tag_kit_result =
            apply_print_conv(296, &tag_value, &mut evaluator, &mut errors, &mut warnings);

        // Both should match
        assert_eq!(
            manual_result.to_string(),
            tag_kit_result.to_string(),
            "ResolutionUnit mismatch for value {value}: manual='{manual_result}', tag_kit='{tag_kit_result}'"
        );

        // Both should match expected
        assert_eq!(
            manual_result.to_string(),
            expected,
            "ResolutionUnit unexpected result for value {value}: got '{manual_result}', expected '{expected}'"
        );
    }
}

#[test]
fn test_ycbcr_positioning_tag_kit_parity() {
    // Test all YCbCrPositioning values
    let test_cases = vec![
        (1u16, "Centered"),
        (2u16, "Co-sited"),
        (999u16, "Unknown (999)"), // Unknown value
    ];

    for (value, expected) in test_cases {
        let tag_value = TagValue::U16(value);

        // Get manual implementation result
        let manual_result = print_conv::ycbcrpositioning_print_conv(&tag_value);

        // Get tag kit result (tag 531 = YCbCrPositioning)
        let mut evaluator = ExpressionEvaluator::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let tag_kit_result =
            apply_print_conv(531, &tag_value, &mut evaluator, &mut errors, &mut warnings);

        // Both should match
        assert_eq!(
            manual_result.to_string(),
            tag_kit_result.to_string(),
            "YCbCrPositioning mismatch for value {value}: manual='{manual_result}', tag_kit='{tag_kit_result}'"
        );

        // Both should match expected
        assert_eq!(
            manual_result.to_string(),
            expected,
            "YCbCrPositioning unexpected result for value {value}: got '{manual_result}', expected '{expected}'"
        );
    }
}

#[test]
fn test_orientation_tag_kit_parity() {
    // Test all Orientation values (1-8)
    let test_cases = vec![
        (1u8, "Horizontal (normal)"),
        (2u8, "Mirror horizontal"),
        (3u8, "Rotate 180"),
        (4u8, "Mirror vertical"),
        (5u8, "Mirror horizontal and rotate 270 CW"),
        (6u8, "Rotate 90 CW"),
        (7u8, "Mirror horizontal and rotate 90 CW"),
        (8u8, "Rotate 270 CW"),
        (99u8, "Unknown (99)"), // Unknown value
    ];

    for (value, expected) in test_cases {
        let tag_value = TagValue::U8(value);

        // Get manual implementation result
        let manual_result = print_conv::orientation_print_conv(&tag_value);

        // Get tag kit result (tag 274 = Orientation)
        let mut evaluator = ExpressionEvaluator::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let tag_kit_result =
            apply_print_conv(274, &tag_value, &mut evaluator, &mut errors, &mut warnings);

        // Both should match
        assert_eq!(
            manual_result.to_string(),
            tag_kit_result.to_string(),
            "Orientation mismatch for value {value}: manual='{manual_result}', tag_kit='{tag_kit_result}'"
        );

        // Both should match expected
        assert_eq!(
            manual_result.to_string(),
            expected,
            "Orientation unexpected result for value {value}: got '{manual_result}', expected '{expected}'"
        );
    }
}

#[test]
fn test_tag_kit_lookup_exists() {
    // Verify that the EXIF_TAG_KITS map contains expected tags
    let expected_tags = vec![
        296, // ResolutionUnit
        274, // Orientation
        531, // YCbCrPositioning
    ];

    for tag_id in expected_tags {
        assert!(
            EXIF_TAG_KITS.contains_key(&tag_id),
            "Tag {tag_id} not found in EXIF_TAG_KITS"
        );

        let tag_def = &EXIF_TAG_KITS[&tag_id];
        assert_eq!(tag_def.id, tag_id, "Tag ID mismatch");
    }
}
