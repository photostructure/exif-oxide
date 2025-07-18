//! Panasonic RAW PrintConv implementations
//!
//! This module contains PrintConv functions for Panasonic RW2/RWL format tags.
//! All implementations use generated lookup tables from ExifTool source code,
//! following the Trust ExifTool principle exactly.
//!
//! ExifTool Reference: lib/Image/ExifTool/PanasonicRaw.pm
//! Generated tables: src/generated/PanasonicRaw_pm/

use crate::types::TagValue;

/// Panasonic Main Compression PrintConv
/// ExifTool: lib/Image/ExifTool/PanasonicRaw.pm Main hash (Compression)
/// Generated table: src/generated/PanasonicRaw_pm/main_inline.rs
pub fn main_compression_print_conv(val: &TagValue) -> TagValue {
    use crate::generated::PanasonicRaw_pm::lookup_main__compression;

    // Handle both i32 and u16 types since compression values can be large
    let compression_val = match val {
        TagValue::I32(v) => Some(*v),
        TagValue::U32(v) if *v <= i32::MAX as u32 => Some(*v as i32),
        TagValue::U16(v) => Some(*v as i32),
        _ => None,
    };

    match compression_val {
        Some(val) => {
            if let Some(description) = lookup_main__compression(val) {
                TagValue::string(description)
            } else {
                TagValue::string(format!("Unknown ({val})"))
            }
        }
        None => TagValue::string(format!("Unknown ({val})")),
    }
}

/// Panasonic Main Orientation PrintConv
/// ExifTool: lib/Image/ExifTool/PanasonicRaw.pm Main hash (Orientation)
/// Generated table: src/generated/PanasonicRaw_pm/main_inline.rs
pub fn main_orientation_print_conv(val: &TagValue) -> TagValue {
    use crate::generated::PanasonicRaw_pm::lookup_main__orientation;

    // Handle both u8 and u16 types - orientation values are 1-8 so fit in u8
    let orientation_val = match val {
        TagValue::U8(v) => Some(*v),
        TagValue::U16(v) if *v <= 255 => Some(*v as u8),
        _ => None,
    };

    match orientation_val {
        Some(val) => {
            if let Some(description) = lookup_main__orientation(val) {
                TagValue::string(description)
            } else {
                TagValue::string(format!("Unknown ({val})"))
            }
        }
        None => TagValue::string(format!("Unknown ({val})")),
    }
}

/// Panasonic Main Multishot PrintConv
/// ExifTool: lib/Image/ExifTool/PanasonicRaw.pm Main hash (Multishot)
/// Generated table: src/generated/PanasonicRaw_pm/main_inline.rs
pub fn main_multishot_print_conv(val: &TagValue) -> TagValue {
    use crate::generated::PanasonicRaw_pm::lookup_main__multishot;

    // Handle both i32 and u32 types
    let multishot_val = match val {
        TagValue::I32(v) => Some(*v),
        TagValue::U32(v) if *v <= i32::MAX as u32 => Some(*v as i32),
        TagValue::U16(v) => Some(*v as i32),
        _ => None,
    };

    match multishot_val {
        Some(val) => {
            if let Some(description) = lookup_main__multishot(val) {
                TagValue::string(description)
            } else {
                TagValue::string(format!("Unknown ({val})"))
            }
        }
        None => TagValue::string(format!("Unknown ({val})")),
    }
}

/// Panasonic Main CFAPattern PrintConv
/// ExifTool: lib/Image/ExifTool/PanasonicRaw.pm Main hash (CFAPattern)
/// Generated table: src/generated/PanasonicRaw_pm/main_inline.rs
pub fn main_cfa_pattern_print_conv(val: &TagValue) -> TagValue {
    use crate::generated::PanasonicRaw_pm::lookup_main__c_f_a_pattern;

    if let Some(pattern_val) = val.as_u8() {
        if let Some(description) = lookup_main__c_f_a_pattern(pattern_val) {
            return TagValue::string(description);
        }
    }
    TagValue::string(format!("Unknown ({val})"))
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
/// ExifTool: PanasonicRaw.pm tag ID to name mappings
pub fn apply_panasonic_raw_print_conv_by_tag_id(tag_id: u16, val: &TagValue) -> TagValue {
    match tag_id {
        0x0103 => main_compression_print_conv(val), // Compression
        0x0112 => main_orientation_print_conv(val), // Orientation
        0x010F => main_multishot_print_conv(val),   // Multishot (if this tag exists)
        0x829D => main_cfa_pattern_print_conv(val), // CFAPattern (ColorSpace in EXIF range)
        _ => val.clone(),                           // No PrintConv for this tag ID
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
        // Test Compression by tag ID
        let result = apply_panasonic_raw_print_conv_by_tag_id(0x0103, &TagValue::I32(34826));
        assert_eq!(result, TagValue::string("Panasonic RAW 2"));

        // Test Orientation by tag ID
        let result = apply_panasonic_raw_print_conv_by_tag_id(0x0112, &TagValue::U8(5));
        assert_eq!(
            result,
            TagValue::string("Mirror horizontal and rotate 270 CW")
        );

        // Test CFAPattern by tag ID
        let result = apply_panasonic_raw_print_conv_by_tag_id(0x829D, &TagValue::U8(2));
        assert_eq!(result, TagValue::string("[Green,Red][Blue,Green]"));

        // Test unknown tag ID - should return original value
        let original = TagValue::U8(42);
        let result = apply_panasonic_raw_print_conv_by_tag_id(0x9999, &original);
        assert_eq!(result, original);
    }
}
