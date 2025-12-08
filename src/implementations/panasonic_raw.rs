//! Panasonic RAW PrintConv implementations
//!
//! This module contains PrintConv functions for Panasonic RW2/RWL format tags.
//! All implementations use lookup tables from ExifTool source code,
//! following the Trust ExifTool principle exactly.
//!
//! ExifTool Reference: lib/Image/ExifTool/PanasonicRaw.pm

use crate::types::TagValue;

/// Panasonic Main Compression PrintConv
/// ExifTool: lib/Image/ExifTool/PanasonicRaw.pm:105-114
/// PrintConv => { 34316 => 'Panasonic RAW 1', 34826 => 'Panasonic RAW 2', 34828 => 'Panasonic RAW 3', 34830 => 'Panasonic RAW 4' }
pub fn main_compression_print_conv(val: &TagValue) -> TagValue {
    let Some(key) = val.as_i64() else {
        return val.clone();
    };

    match key {
        34316 => TagValue::string("Panasonic RAW 1"),
        34826 => TagValue::string("Panasonic RAW 2"),
        34828 => TagValue::string("Panasonic RAW 3"),
        34830 => TagValue::string("Panasonic RAW 4"),
        _ => TagValue::string(format!("Unknown ({})", key)),
    }
}

/// Panasonic Main Orientation PrintConv
/// ExifTool: lib/Image/ExifTool/PanasonicRaw.pm:248-251 (uses %Image::ExifTool::Exif::orientation)
/// Standard EXIF orientation values - reuses the generated EXIF orientation lookup table
pub fn main_orientation_print_conv(val: &TagValue) -> TagValue {
    use crate::generated::Exif_pm::orientation::lookup_orientation;

    let Some(key) = val.as_i64() else {
        return val.clone();
    };

    if !(0..=255).contains(&key) {
        return val.clone();
    }

    match lookup_orientation(key as u8) {
        Some(description) => TagValue::string(description),
        None => TagValue::string(format!("Unknown ({})", key)),
    }
}

/// Panasonic Main Multishot PrintConv
/// ExifTool: lib/Image/ExifTool/PanasonicRaw.pm:295-298
/// PrintConv => { 0 => 'Off', 65536 => 'Pixel Shift' }
pub fn main_multishot_print_conv(val: &TagValue) -> TagValue {
    let Some(key) = val.as_i64() else {
        return val.clone();
    };

    match key {
        0 => TagValue::string("Off"),
        65536 => TagValue::string("Pixel Shift"),
        _ => TagValue::string(format!("Unknown ({})", key)),
    }
}

/// Panasonic Main CFAPattern PrintConv
/// ExifTool: lib/Image/ExifTool/PanasonicRaw.pm:96-102
/// PrintConv => { 0 => 'n/a', 1 => '[Red,Green][Green,Blue]', 2 => '[Green,Red][Blue,Green]', 3 => '[Green,Blue][Red,Green]', 4 => '[Blue,Green][Green,Red]' }
pub fn main_cfa_pattern_print_conv(val: &TagValue) -> TagValue {
    let Some(key) = val.as_i64() else {
        return val.clone();
    };

    match key {
        0 => TagValue::string("n/a"),
        1 => TagValue::string("[Red,Green][Green,Blue]"),
        2 => TagValue::string("[Green,Red][Blue,Green]"),
        3 => TagValue::string("[Green,Blue][Red,Green]"),
        4 => TagValue::string("[Blue,Green][Green,Red]"),
        _ => TagValue::string(format!("Unknown ({})", key)),
    }
}

/// Apply PrintConv to Panasonic Main table tags
///
/// This function maps Main table tag names to their corresponding PrintConv functions.
/// ExifTool: PanasonicRaw.pm Main hash PrintConv fields
pub fn apply_main_print_conv(tag_name: &str, val: &TagValue) -> TagValue {
    match tag_name {
        "Compression" => main_compression_print_conv(val),
        "Orientation" => main_orientation_print_conv(val),
        "Multishot" => main_multishot_print_conv(val),
        "CFAPattern" => main_cfa_pattern_print_conv(val),
        _ => val.clone(), // No PrintConv for this tag
    }
}

/// Apply PrintConv to Panasonic RAW tags by tag ID
///
/// This function maps Panasonic RAW tag IDs to their corresponding PrintConv functions.
/// All tag IDs verified against ExifTool PanasonicRaw.pm and generated tag kits.
pub fn apply_panasonic_raw_print_conv_by_tag_id(tag_id: u16, val: &TagValue) -> TagValue {
    match tag_id {
        11 => main_compression_print_conv(val), // Compression (0x000B)
        274 => main_orientation_print_conv(val), // Orientation (0x0112)
        289 => main_multishot_print_conv(val),  // Multishot (0x0121)
        9 => main_cfa_pattern_print_conv(val),  // CFAPattern (0x0009)
        _ => val.clone(),                       // No PrintConv for this tag ID
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_compression_print_conv() {
        // Test known values from generated table
        let result = main_compression_print_conv(&TagValue::I32(34316));
        assert_eq!(result, TagValue::string("Panasonic RAW 1"));

        let result = main_compression_print_conv(&TagValue::I32(34826));
        assert_eq!(result, TagValue::string("Panasonic RAW 2"));

        let result = main_compression_print_conv(&TagValue::I32(34828));
        assert_eq!(result, TagValue::string("Panasonic RAW 3"));

        let result = main_compression_print_conv(&TagValue::I32(34830));
        assert_eq!(result, TagValue::string("Panasonic RAW 4"));

        // Test unknown value
        let result = main_compression_print_conv(&TagValue::I32(99999));
        assert_eq!(result, TagValue::string("Unknown (99999)"));
    }

    #[test]
    fn test_main_orientation_print_conv() {
        // Test known values from generated table
        let result = main_orientation_print_conv(&TagValue::U8(1));
        assert_eq!(result, TagValue::string("Horizontal (normal)"));

        let result = main_orientation_print_conv(&TagValue::U8(2));
        assert_eq!(result, TagValue::string("Mirror horizontal"));

        let result = main_orientation_print_conv(&TagValue::U8(3));
        assert_eq!(result, TagValue::string("Rotate 180"));

        let result = main_orientation_print_conv(&TagValue::U8(8));
        assert_eq!(result, TagValue::string("Rotate 270 CW"));

        // Test unknown value
        let result = main_orientation_print_conv(&TagValue::U8(99));
        assert_eq!(result, TagValue::string("Unknown (99)"));
    }

    #[test]
    fn test_main_multishot_print_conv() {
        // Test known values from generated table
        let result = main_multishot_print_conv(&TagValue::I32(0));
        assert_eq!(result, TagValue::string("Off"));

        let result = main_multishot_print_conv(&TagValue::I32(65536));
        assert_eq!(result, TagValue::string("Pixel Shift"));

        // Test unknown value
        let result = main_multishot_print_conv(&TagValue::I32(12345));
        assert_eq!(result, TagValue::string("Unknown (12345)"));
    }

    #[test]
    fn test_main_cfa_pattern_print_conv() {
        // Test known values from generated table
        let result = main_cfa_pattern_print_conv(&TagValue::U8(0));
        assert_eq!(result, TagValue::string("n/a"));

        let result = main_cfa_pattern_print_conv(&TagValue::U8(1));
        assert_eq!(result, TagValue::string("[Red,Green][Green,Blue]"));

        let result = main_cfa_pattern_print_conv(&TagValue::U8(4));
        assert_eq!(result, TagValue::string("[Blue,Green][Green,Red]"));

        // Test unknown value
        let result = main_cfa_pattern_print_conv(&TagValue::U8(99));
        assert_eq!(result, TagValue::string("Unknown (99)"));
    }

    #[test]
    fn test_apply_main_print_conv() {
        // Test Compression conversion
        let result = apply_main_print_conv("Compression", &TagValue::I32(34316));
        assert_eq!(result, TagValue::string("Panasonic RAW 1"));

        // Test Orientation conversion
        let result = apply_main_print_conv("Orientation", &TagValue::U8(6));
        assert_eq!(result, TagValue::string("Rotate 90 CW"));

        // Test CFAPattern conversion
        let result = apply_main_print_conv("CFAPattern", &TagValue::U8(2));
        assert_eq!(result, TagValue::string("[Green,Red][Blue,Green]"));

        // Test unknown tag - should return original value
        let original = TagValue::U8(42);
        let result = apply_main_print_conv("UnknownTag", &original);
        assert_eq!(result, original);
    }

    #[test]
    fn test_apply_panasonic_raw_print_conv_by_tag_id() {
        // Test Compression by tag ID (0x000B = 11)
        let result = apply_panasonic_raw_print_conv_by_tag_id(11, &TagValue::I32(34826));
        assert_eq!(result, TagValue::string("Panasonic RAW 2"));

        // Test Orientation by tag ID (0x0112 = 274)
        let result = apply_panasonic_raw_print_conv_by_tag_id(274, &TagValue::U8(5));
        assert_eq!(
            result,
            TagValue::string("Mirror horizontal and rotate 270 CW")
        );

        // Test CFAPattern by tag ID (0x0009 = 9)
        let result = apply_panasonic_raw_print_conv_by_tag_id(9, &TagValue::U8(2));
        assert_eq!(result, TagValue::string("[Green,Red][Blue,Green]"));

        // Test unknown tag ID - should return original value
        let original = TagValue::U8(42);
        let result = apply_panasonic_raw_print_conv_by_tag_id(9999, &original);
        assert_eq!(result, original);
    }
}
