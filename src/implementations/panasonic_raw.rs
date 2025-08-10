//! Panasonic RAW PrintConv implementations
//!
//! This module contains PrintConv functions for Panasonic RW2/RWL format tags.
//! All implementations use the generated tag system from the new universal extraction,
//! following the Trust ExifTool principle exactly.
//!
//! ExifTool Reference: lib/Image/ExifTool/PanasonicRaw.pm
//! Generated tags: src/generated/panasonic_raw/main_tags.rs

use crate::expressions::ExpressionEvaluator;
use crate::generated::PanasonicRaw_pm::main_tags::PANASONIC_RAW_MAIN_TAGS;
use crate::types::{PrintConv, TagValue};
use tracing::debug;

/// Panasonic RAW-specific PrintConv application using the generated tag table
/// ExifTool: PanasonicRaw.pm PrintConv processing with registry fallback
fn apply_panasonic_raw_print_conv(
    tag_id: u32,
    value: &TagValue,
    evaluator: &mut ExpressionEvaluator,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) -> TagValue {
    // Look up the tag in PanasonicRaw main tags table
    if let Some(tag_info) = PANASONIC_RAW_MAIN_TAGS.get(&(tag_id as u16)) {
        debug!("Found PanasonicRaw tag {}: {}", tag_id, tag_info.name);

        match &tag_info.print_conv {
            Some(PrintConv::Expression(expr)) => {
                debug!("Using PrintConv expression: {}", expr);
                // Use the expression evaluator for complex Perl expressions
                match evaluator.evaluate_expression(&expr, value) {
                    Ok(result) => result,
                    Err(e) => {
                        warnings.push(format!(
                            "Failed to evaluate PrintConv expression for tag {}: {}",
                            tag_id, e
                        ));
                        value.clone()
                    }
                }
            }
            Some(PrintConv::Complex) => {
                debug!(
                    "Complex PrintConv for tag {}, using generated module",
                    tag_id
                );
                // For complex conversions, use the generated module's apply_print_conv
                // TODO: This should be applied by the specific tag table that contains this tag
                value.clone() // Placeholder until proper tag table routing is implemented
            }
            Some(PrintConv::Simple(_table)) => {
                debug!(
                    "Simple PrintConv table for tag {} (not yet implemented)",
                    tag_id
                );
                // TODO: Handle simple lookup tables
                value.clone()
            }
            Some(PrintConv::None) | None => {
                debug!("No PrintConv for PanasonicRaw tag {}", tag_id);
                value.clone()
            }
        }
    } else {
        debug!("PanasonicRaw tag {} not found in main tags table", tag_id);
        value.clone()
    }
}

/// Panasonic Main Compression PrintConv
/// ExifTool: lib/Image/ExifTool/PanasonicRaw.pm Main hash (Compression, tag ID 11)
/// Tag kit: src/generated/panasonicraw_pm/tag_kit/other.rs (PRINT_CONV_3)
pub fn main_compression_print_conv(val: &TagValue) -> TagValue {
    let mut evaluator = ExpressionEvaluator::new();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    apply_panasonic_raw_print_conv(11, val, &mut evaluator, &mut errors, &mut warnings)
}

/// Panasonic Main Orientation PrintConv  
/// ExifTool: lib/Image/ExifTool/PanasonicRaw.pm Main hash (Orientation, tag ID 274)
/// Tag kit: src/generated/panasonicraw_pm/tag_kit/core.rs (PRINT_CONV_0)
pub fn main_orientation_print_conv(val: &TagValue) -> TagValue {
    let mut evaluator = ExpressionEvaluator::new();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    apply_panasonic_raw_print_conv(274, val, &mut evaluator, &mut errors, &mut warnings)
}

/// Panasonic Main Multishot PrintConv
/// ExifTool: lib/Image/ExifTool/PanasonicRaw.pm Main hash (Multishot, tag ID 289)
/// Tag kit: src/generated/panasonicraw_pm/tag_kit/document.rs (PRINT_CONV_1)
pub fn main_multishot_print_conv(val: &TagValue) -> TagValue {
    let mut evaluator = ExpressionEvaluator::new();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    apply_panasonic_raw_print_conv(289, val, &mut evaluator, &mut errors, &mut warnings)
}

/// Panasonic Main CFAPattern PrintConv
/// ExifTool: lib/Image/ExifTool/PanasonicRaw.pm Main hash (CFAPattern, tag ID 9)
/// Tag kit: src/generated/panasonicraw_pm/tag_kit/interop.rs (PRINT_CONV_2)
pub fn main_cfa_pattern_print_conv(val: &TagValue) -> TagValue {
    let mut evaluator = ExpressionEvaluator::new();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    apply_panasonic_raw_print_conv(9, val, &mut evaluator, &mut errors, &mut warnings)
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
