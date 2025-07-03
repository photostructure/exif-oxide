//! Unit tests for Canon-specific EXIF processing
//!
//! These tests validate Canon MakerNote processing, offset scheme detection,
//! TIFF footer parsing, AF Info processing, and camera settings extraction.

#[cfg(test)]
use super::*;
#[cfg(test)]
use crate::implementations::canon::binary_data::create_camera_settings_table;
#[cfg(test)]
use crate::tiff_types::ByteOrder;
#[cfg(test)]
use crate::types::TagValue;
#[cfg(test)]
use std::collections::HashMap;

#[test]
fn test_canon_signature_detection() {
    assert!(detect_canon_signature("Canon"));
    assert!(detect_canon_signature("Canon EOS REBEL T3i"));
    assert!(!detect_canon_signature("Nikon"));
    assert!(!detect_canon_signature("Sony"));
}

#[test]
fn test_offset_scheme_detection() {
    // Test default case
    assert_eq!(
        detect_offset_scheme("Canon EOS 5D"),
        CanonOffsetScheme::FourByte
    );

    // Test 6-byte models
    assert_eq!(
        detect_offset_scheme("Canon EOS 20D"),
        CanonOffsetScheme::SixByte
    );
    assert_eq!(
        detect_offset_scheme("Canon EOS 350D"),
        CanonOffsetScheme::SixByte
    );
    assert_eq!(
        detect_offset_scheme("Canon EOS REBEL XT"),
        CanonOffsetScheme::SixByte
    );

    // Test 28-byte models
    assert_eq!(
        detect_offset_scheme("Canon FV-M30"),
        CanonOffsetScheme::TwentyEightByte
    );
    assert_eq!(
        detect_offset_scheme("Canon OPTURA 60"),
        CanonOffsetScheme::TwentyEightByte
    );

    // Test 16-byte models
    assert_eq!(
        detect_offset_scheme("Canon PowerShot S70"),
        CanonOffsetScheme::SixteenByte
    );
    assert_eq!(
        detect_offset_scheme("Canon IXUS 400"),
        CanonOffsetScheme::SixteenByte
    );
}

#[test]
fn test_canon_tiff_footer_parse() {
    use crate::implementations::canon::tiff_footer::CanonTiffFooter;

    // Test little-endian footer
    let le_footer = [0x49, 0x49, 0x2a, 0x00, 0x10, 0x00, 0x00, 0x00];
    let footer = CanonTiffFooter::parse(&le_footer, ByteOrder::LittleEndian).unwrap();
    assert_eq!(footer.original_offset, 0x10);

    // Test big-endian footer
    let be_footer = [0x4d, 0x4d, 0x00, 0x2a, 0x00, 0x00, 0x00, 0x10];
    let footer = CanonTiffFooter::parse(&be_footer, ByteOrder::BigEndian).unwrap();
    assert_eq!(footer.original_offset, 0x10);

    // Test invalid header
    let invalid_footer = [0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00];
    assert!(CanonTiffFooter::parse(&invalid_footer, ByteOrder::LittleEndian).is_err());

    // Test byte order mismatch
    assert!(CanonTiffFooter::parse(&le_footer, ByteOrder::BigEndian).is_err());
}

#[test]
fn test_camera_settings_table() {
    let table = create_camera_settings_table();

    // Test MacroMode tag
    let macro_tag = table.get(&1).unwrap();
    assert_eq!(macro_tag.name, "MacroMode");
    assert_eq!(
        macro_tag.print_conv.as_ref().unwrap().get(&1),
        Some(&"Macro".to_string())
    );

    // Test FocusMode tag
    let focus_tag = table.get(&7).unwrap();
    assert_eq!(focus_tag.name, "FocusMode");
    assert_eq!(
        focus_tag.print_conv.as_ref().unwrap().get(&0),
        Some(&"One-shot AF".to_string())
    );
}

#[test]
fn test_af_size_expr_calculation() {
    let mut extracted_values = HashMap::new();
    extracted_values.insert(0, 9); // NumAFPoints = 9
    extracted_values.insert(2, 9); // NumAFPoints for AFInfo2 = 9

    // Test fixed size
    let fixed_expr = CanonAfSizeExpr::Fixed(5);
    assert_eq!(fixed_expr.calculate_size(&extracted_values), 5);

    // Test value reference
    let val_ref_expr = CanonAfSizeExpr::ValueRef(0);
    assert_eq!(val_ref_expr.calculate_size(&extracted_values), 9);

    // Test ceiling division: int((9+15)/16) = int(24/16) = 1
    let ceil_div_expr = CanonAfSizeExpr::CeilDiv(0, 16);
    assert_eq!(ceil_div_expr.calculate_size(&extracted_values), 1);

    // Test with 45 AF points: int((45+15)/16) = int(60/16) = 3
    extracted_values.insert(0, 45);
    assert_eq!(ceil_div_expr.calculate_size(&extracted_values), 3);
}

#[test]
fn test_af_info_table_structure() {
    let table = create_af_info_table();

    // Verify table has expected number of tags
    assert_eq!(table.len(), 12);

    // Test NumAFPoints (sequence 0)
    let num_af_points = &table[0];
    assert_eq!(num_af_points.sequence, 0);
    assert_eq!(num_af_points.name, "NumAFPoints");
    assert!(matches!(num_af_points.format, CanonAfFormat::Int16u));

    // Test AFAreaXPositions (sequence 8) - variable array
    let x_positions = &table[8];
    assert_eq!(x_positions.sequence, 8);
    assert_eq!(x_positions.name, "AFAreaXPositions");
    assert!(matches!(x_positions.format, CanonAfFormat::Int16sArray(_)));
    assert!(matches!(
        x_positions.size_expr,
        CanonAfSizeExpr::ValueRef(0)
    ));

    // Test AFPointsInFocus (sequence 10) - ceiling division
    let points_in_focus = &table[10];
    assert_eq!(points_in_focus.sequence, 10);
    assert_eq!(points_in_focus.name, "AFPointsInFocus");
    assert!(matches!(
        points_in_focus.format,
        CanonAfFormat::Int16sArray(_)
    ));
    assert!(matches!(
        points_in_focus.size_expr,
        CanonAfSizeExpr::CeilDiv(0, 16)
    ));
}

#[test]
fn test_af_info2_table_structure() {
    let table = create_af_info2_table();

    // Verify table has expected number of tags
    assert_eq!(table.len(), 14);

    // Test AFAreaMode (sequence 1) with PrintConv
    let af_area_mode = &table[1];
    assert_eq!(af_area_mode.sequence, 1);
    assert_eq!(af_area_mode.name, "AFAreaMode");
    assert!(af_area_mode.print_conv.is_some());
    assert_eq!(
        af_area_mode.print_conv.as_ref().unwrap().get(&2),
        Some(&"Single-point AF".to_string())
    );

    // Test AFAreaWidths (sequence 8) - uses $val{2} for AFInfo2
    let af_area_widths = &table[8];
    assert_eq!(af_area_widths.sequence, 8);
    assert_eq!(af_area_widths.name, "AFAreaWidths");
    assert!(matches!(
        af_area_widths.size_expr,
        CanonAfSizeExpr::ValueRef(2)
    ));

    // Test AFPointsSelected (sequence 13) - EOS only
    let points_selected = &table[13];
    assert_eq!(points_selected.sequence, 13);
    assert_eq!(points_selected.name, "AFPointsSelected");
    assert!(matches!(
        points_selected.condition,
        Some(CanonAfCondition::ModelIsEos)
    ));
}

#[test]
fn test_process_serial_data_basic() {
    // Simulate minimal AFInfo data: NumAFPoints=2, ValidAFPoints=2, sizes...
    let test_data = vec![
        // NumAFPoints = 2 (little-endian)
        0x02, 0x00, // ValidAFPoints = 2
        0x02, 0x00, // CanonImageWidth = 5184 (0x1440)
        0x40, 0x14, // CanonImageHeight = 3456 (0x0d80)
        0x80, 0x0d, // AFImageWidth = 5184
        0x40, 0x14, // AFImageHeight = 3456
        0x80, 0x0d, // AFAreaWidth = 139
        0x8b, 0x00, // AFAreaHeight = 186
        0xba, 0x00, // AFAreaXPositions[2]: -1365, 1365 (signed)
        0xab, 0xfa, // -1365 in two's complement little-endian
        0x55, 0x05, // 1365
        // AFAreaYPositions[2]: 0, 0
        0x00, 0x00, 0x00, 0x00,
        // AFPointsInFocus: ceil(2/16) = 1 value: 3 (points 0,1 set)
        0x03, 0x00,
    ];

    let table = create_af_info_table();
    let byte_order = ByteOrder::LittleEndian;
    let model = "Canon EOS REBEL T3i"; // EOS model

    let results =
        process_serial_data(&test_data, 0, test_data.len(), byte_order, &table, model).unwrap();

    // Verify basic values
    assert_eq!(
        results.get("MakerNotes:NumAFPoints"),
        Some(&TagValue::U16(2))
    );
    assert_eq!(
        results.get("MakerNotes:ValidAFPoints"),
        Some(&TagValue::U16(2))
    );
    assert_eq!(
        results.get("MakerNotes:CanonImageWidth"),
        Some(&TagValue::U16(5184))
    );
    assert_eq!(
        results.get("MakerNotes:CanonImageHeight"),
        Some(&TagValue::U16(3456))
    );

    // Verify variable arrays
    assert_eq!(
        results.get("MakerNotes:AFAreaXPositions"),
        Some(&TagValue::String("-1365 1365".to_string()))
    );
    assert_eq!(
        results.get("MakerNotes:AFAreaYPositions"),
        Some(&TagValue::String("0 0".to_string()))
    );
    assert_eq!(
        results.get("MakerNotes:AFPointsInFocus"),
        Some(&TagValue::String("3".to_string()))
    );

    // PrimaryAFPoint should be present for EOS model (but not in our test data)
    assert!(!results.contains_key("MakerNotes:PrimaryAFPoint")); // Not enough data
}
