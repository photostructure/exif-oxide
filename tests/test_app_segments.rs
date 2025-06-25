//! Tests for APP segment table functionality

use exif_oxide::tables::app_segments::{
    get_app_segment_rules, identify_app_segment, FormatHandler,
};

#[test]
fn test_app0_jfif_identification() {
    // Test JFIF APP0 segment identification
    let jfif_data = b"JFIF\x00\x01\x01\x01\x00H\x00H\x00\x00";

    let rule = identify_app_segment(0, jfif_data).expect("Should identify JFIF");
    assert_eq!(rule.name, "JFIF");
    assert!(matches!(rule.format_handler, FormatHandler::JFIF));
}

#[test]
fn test_app1_exif_identification() {
    // Test EXIF APP1 segment identification
    let exif_data = b"Exif\x00\x00MM\x00*\x00\x00\x00\x08";

    let rule = identify_app_segment(1, exif_data).expect("Should identify EXIF");
    assert_eq!(rule.name, "EXIF");
    assert!(matches!(rule.format_handler, FormatHandler::EXIF));
}

#[test]
fn test_app1_xmp_identification() {
    // Test XMP APP1 segment identification
    let xmp_data = b"http://ns.adobe.com/xap/1.0/\x00<?xml version=\"1.0\"";

    let rule = identify_app_segment(1, xmp_data).expect("Should identify XMP");
    assert_eq!(rule.name, "XMP");
    assert!(matches!(rule.format_handler, FormatHandler::XMP));
}

#[test]
fn test_app2_mpf_identification() {
    // Test MPF APP2 segment identification
    let mpf_data = b"MPF\x00MM\x00*\x00\x00\x00\x08";

    let rule = identify_app_segment(2, mpf_data).expect("Should identify MPF");
    assert_eq!(rule.name, "MPF");
    assert!(matches!(rule.format_handler, FormatHandler::MPF));
}

#[test]
fn test_app6_gopro_identification() {
    // Test GoPro APP6 segment identification
    let gopro_data = b"GoPro\x00DEVC";

    let rule = identify_app_segment(6, gopro_data).expect("Should identify GoPro");
    assert_eq!(rule.name, "GoPro");
    assert!(matches!(rule.format_handler, FormatHandler::GoPro));
}

#[test]
fn test_app13_photoshop_identification() {
    // Test Photoshop APP13 segment identification
    let photoshop_data = b"Photoshop 3.0\x008BIM";

    let rule = identify_app_segment(13, photoshop_data).expect("Should identify Photoshop");
    assert_eq!(rule.name, "Photoshop");
    assert!(matches!(rule.format_handler, FormatHandler::Photoshop));
}

#[test]
fn test_unknown_app_segment() {
    // Test unknown APP segment
    let unknown_data = b"UNKNOWN\x00\x01\x02\x03";

    let result = identify_app_segment(0, unknown_data);
    assert!(result.is_none(), "Should not identify unknown segment");
}

#[test]
fn test_app_segment_rules_coverage() {
    // Test that we have rules for all major APP segments
    for app_num in 0..=15 {
        let rules = get_app_segment_rules(app_num).expect("Should have rules for all APP segments");

        // Some segments might be empty, but the function should still return successfully
        if app_num == 0 {
            // APP0 should have JFIF rules
            assert!(!rules.is_empty(), "APP0 should have JFIF rules");
            assert!(rules.iter().any(|r| r.name == "JFIF"));
        }

        if app_num == 1 {
            // APP1 should have EXIF and XMP rules
            assert!(!rules.is_empty(), "APP1 should have EXIF/XMP rules");
            assert!(rules.iter().any(|r| r.name == "EXIF"));
            assert!(rules.iter().any(|r| r.name == "XMP"));
        }

        if app_num == 2 {
            // APP2 should have MPF rules
            assert!(!rules.is_empty(), "APP2 should have MPF rules");
            assert!(rules.iter().any(|r| r.name == "MPF"));
        }

        if app_num == 6 {
            // APP6 should have GoPro rules
            assert!(!rules.is_empty(), "APP6 should have GoPro rules");
            assert!(rules.iter().any(|r| r.name == "GoPro"));
        }
    }
}

#[test]
fn test_segment_signature_accuracy() {
    // Test that signatures are accurate byte representations
    let rules = get_app_segment_rules(0).unwrap();
    let jfif_rule = rules.iter().find(|r| r.name == "JFIF").unwrap();

    // JFIF signature should be "JFIF\0"
    assert_eq!(jfif_rule.signature, b"JFIF\0");

    let app1_rules = get_app_segment_rules(1).unwrap();
    let exif_rule = app1_rules.iter().find(|r| r.name == "EXIF").unwrap();

    // EXIF signature should be "Exif\0"
    assert_eq!(exif_rule.signature, b"Exif\0");
}

#[test]
fn test_multiple_app1_segments() {
    // Test that we can identify multiple different formats in APP1
    let app1_rules = get_app_segment_rules(1).unwrap();

    // Should have at least EXIF, XMP, and ExtendedXMP
    let format_names: Vec<&str> = app1_rules.iter().map(|r| r.name).collect();
    assert!(format_names.contains(&"EXIF"));
    assert!(format_names.contains(&"XMP"));
    assert!(format_names.contains(&"ExtendedXMP"));
    assert!(format_names.contains(&"QVCI"));
    assert!(format_names.contains(&"FLIR"));
}

#[test]
fn test_condition_types() {
    // Test different condition types are properly classified
    let app1_rules = get_app_segment_rules(1).unwrap();

    // EXIF should use StartsWith
    let exif_rule = app1_rules.iter().find(|r| r.name == "EXIF").unwrap();
    assert!(matches!(
        exif_rule.condition_type,
        exif_oxide::tables::app_segments::ConditionType::StartsWith
    ));

    // ExtendedXMP uses a complex condition
    let extended_xmp_rule = app1_rules.iter().find(|r| r.name == "ExtendedXMP").unwrap();
    assert!(matches!(
        extended_xmp_rule.condition_type,
        exif_oxide::tables::app_segments::ConditionType::Custom(_)
    ));
}
