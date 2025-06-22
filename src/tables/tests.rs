//! Tests for tag table functionality

use crate::core::ExifFormat;
use crate::tables::lookup_tag;

#[test]
fn test_lookup_common_tags() {
    // Test Make tag
    let make_tag = lookup_tag(0x010F);
    assert!(make_tag.is_some());
    let make = make_tag.unwrap();
    assert_eq!(make.name, "Make");
    assert_eq!(make.format, ExifFormat::Ascii);
    assert_eq!(make.group, Some("Camera"));

    // Test Model tag
    let model_tag = lookup_tag(0x0110);
    assert!(model_tag.is_some());
    let model = model_tag.unwrap();
    assert_eq!(model.name, "Model");
    assert_eq!(model.format, ExifFormat::Ascii);
    assert_eq!(model.group, Some("Camera"));

    // Test Orientation tag
    let orientation_tag = lookup_tag(0x0112);
    assert!(orientation_tag.is_some());
    let orientation = orientation_tag.unwrap();
    assert_eq!(orientation.name, "Orientation");
    assert_eq!(orientation.format, ExifFormat::U16);

    // Test XResolution (rational type)
    let xres_tag = lookup_tag(0x011A);
    assert!(xres_tag.is_some());
    let xres = xres_tag.unwrap();
    assert_eq!(xres.name, "XResolution");
    assert_eq!(xres.format, ExifFormat::Rational);
}

#[test]
fn test_lookup_datetime_tags() {
    // Test DateTime
    let datetime_tag = lookup_tag(0x0132);
    assert!(datetime_tag.is_some());
    let datetime = datetime_tag.unwrap();
    assert_eq!(datetime.name, "ModifyDate");
    assert_eq!(datetime.format, ExifFormat::Ascii);
    assert_eq!(datetime.group, Some("Time"));

    // Test DateTimeOriginal
    let orig_tag = lookup_tag(0x9003);
    assert!(orig_tag.is_some());
    let orig = orig_tag.unwrap();
    assert_eq!(orig.name, "DateTimeOriginal");
    assert_eq!(orig.format, ExifFormat::Ascii);
    assert_eq!(orig.group, Some("Time"));
}

#[test]
fn test_lookup_exposure_tags() {
    // Test ExposureTime (rational)
    let exp_tag = lookup_tag(0x829A);
    assert!(exp_tag.is_some());
    let exp = exp_tag.unwrap();
    assert_eq!(exp.name, "ExposureTime");
    assert_eq!(exp.format, ExifFormat::Rational);

    // Test FNumber (rational)
    let fnum_tag = lookup_tag(0x829D);
    assert!(fnum_tag.is_some());
    let fnum = fnum_tag.unwrap();
    assert_eq!(fnum.name, "FNumber");
    assert_eq!(fnum.format, ExifFormat::Rational);
}

#[test]
fn test_lookup_signed_rational_tags() {
    // Test ExposureCompensation (signed rational)
    let ec_tag = lookup_tag(0x9204);
    assert!(ec_tag.is_some());
    let ec = ec_tag.unwrap();
    assert_eq!(ec.name, "ExposureCompensation");
    assert_eq!(ec.format, ExifFormat::SignedRational);
}

#[test]
fn test_lookup_unknown_tag() {
    // Test that unknown tags return None
    assert!(lookup_tag(0xDEAD).is_none());
    assert!(lookup_tag(0xBEEF).is_none());
    assert!(lookup_tag(0xCAFE).is_none());
}

#[test]
fn test_lookup_edge_cases() {
    // Test first tag in table
    let first_tag = lookup_tag(0x0001);
    assert!(first_tag.is_some());
    assert_eq!(first_tag.unwrap().name, "InteropIndex");

    // Test tags at boundaries
    assert!(lookup_tag(0x0000).is_none()); // Before first tag
    assert!(lookup_tag(0xFFFF).is_some()); // Last possible tag
}

#[test]
fn test_table_contains_expected_tags() {
    // Verify we have at least the essential EXIF tags
    let essential_tags = [
        (0x010E, "ImageDescription"),
        (0x010F, "Make"),
        (0x0110, "Model"),
        (0x0112, "Orientation"),
        (0x011A, "XResolution"),
        (0x011B, "YResolution"),
        (0x0128, "ResolutionUnit"),
        (0x0131, "Software"),
        (0x0132, "ModifyDate"),
        (0x013B, "Artist"),
        (0x8298, "Copyright"),
        (0x829A, "ExposureTime"),
        (0x829D, "FNumber"),
        (0x8769, "ExifOffset"),
        (0x8825, "GPSInfo"),
        (0x9003, "DateTimeOriginal"),
        (0x9004, "CreateDate"),
        (0x9204, "ExposureCompensation"),
        (0xA002, "ExifImageWidth"),
        (0xA003, "ExifImageHeight"),
    ];

    for (tag_id, expected_name) in &essential_tags {
        let tag = lookup_tag(*tag_id);
        assert!(
            tag.is_some(),
            "Tag 0x{:04X} ({}) should exist",
            tag_id,
            expected_name
        );
        assert_eq!(
            tag.unwrap().name,
            *expected_name,
            "Tag 0x{:04X} has wrong name",
            tag_id
        );
    }
}
