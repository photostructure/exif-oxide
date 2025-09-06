//! Demonstration of how placeholder functions use missing conversion tracking
//!
//! This simulates what the generated placeholder functions do when they
//! encounter expressions that couldn't be translated from Perl to Rust.

use codegen_runtime::missing::{
    clear_missing_conversions, get_missing_conversions, missing_print_conv, missing_value_conv,
};
use codegen_runtime::types::{ExifContext, ExifError};
use codegen_runtime::TagValue;

/// This simulates a generated placeholder function for a PrintConv expression
/// that couldn't be translated (e.g., uses complex Perl regex or function calls)
pub fn placeholder_print_conv_example(val: &TagValue, _ctx: Option<&ExifContext>) -> TagValue {
    // In real usage, these values would be passed from the tag processing context
    // For now, we're using placeholder values as the generator does
    codegen_runtime::missing::missing_print_conv(
        0,                                                    // tag_id (would be filled at runtime)
        "UnknownTag",   // tag_name (would be filled at runtime)
        "UnknownGroup", // group (would be filled at runtime)
        "Image::ExifTool::GPS::ToDMS($self, $val, 1, \"E\")", // original Perl expression
        val,
    )
}

/// This simulates a generated placeholder function for a ValueConv expression
pub fn placeholder_value_conv_example(
    val: &TagValue,
    _ctx: Option<&ExifContext>,
) -> Result<TagValue, ExifError> {
    Ok(codegen_runtime::missing::missing_value_conv(
        0,                                                         // tag_id
        "UnknownTag",                                              // tag_name
        "UnknownGroup",                                            // group
        "my @a=split(' ',$val); $_/=500 foreach @a; join(' ',@a)", // original Perl expression
        val,
    ))
}

#[test]
fn test_placeholder_functions_track_missing_conversions() {
    clear_missing_conversions();

    // Simulate calling placeholder functions during tag processing
    let test_value = TagValue::String("45.5 123.7".to_string());

    // Call the PrintConv placeholder
    let result1 = placeholder_print_conv_example(&test_value, None);
    assert_eq!(result1, test_value); // Should return unchanged

    // Call the ValueConv placeholder
    let result2 = placeholder_value_conv_example(&test_value, None).unwrap();
    assert_eq!(result2, test_value); // Should return unchanged

    // Check that both were tracked
    let missing = get_missing_conversions();
    assert_eq!(missing.len(), 2);

    // Verify the expressions were captured
    let expressions: Vec<&str> = missing.iter().map(|m| m.expression.as_str()).collect();
    assert!(expressions.contains(&"Image::ExifTool::GPS::ToDMS($self, $val, 1, \"E\")"));
    assert!(expressions.contains(&"my @a=split(' ',$val); $_/=500 foreach @a; join(' ',@a)"));
}

#[test]
fn test_show_missing_output_format() {
    clear_missing_conversions();

    // Simulate processing multiple tags with missing conversions
    let value = TagValue::F64(123.45);

    // Different tags using the same expression
    missing_print_conv(
        0x0001,
        "GPSLatitude",
        "GPS",
        "Image::ExifTool::GPS::ToDMS($self, $val, 1, \"N\")",
        &value,
    );

    // Another expression type
    missing_value_conv(
        0x0002,
        "SensorSize",
        "Pentax",
        "my @a=split(' ',$val); $_/=500 foreach @a; join(' ',@a)",
        &value,
    );

    // Get the missing conversions as would be shown with --show-missing
    let missing = get_missing_conversions();

    // Format them for display (this is what the main app would do)
    for m in &missing {
        let conv_type = match m.conv_type {
            codegen_runtime::missing::ConversionType::PrintConv => "PrintConv",
            codegen_runtime::missing::ConversionType::ValueConv => "ValueConv",
        };

        println!(
            "{}: {} [used by {}:{}]",
            conv_type, m.expression, m.group, m.tag_name
        );
    }

    assert_eq!(missing.len(), 2);
}
