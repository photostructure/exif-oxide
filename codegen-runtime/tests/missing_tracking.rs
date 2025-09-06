//! Tests for missing conversion tracking functionality
//!
//! This module tests that the missing_print_conv and missing_value_conv
//! functions properly track expressions that couldn't be translated.

use codegen_runtime::missing::{
    clear_missing_conversions, get_missing_conversions, missing_print_conv, missing_value_conv,
    ConversionType,
};
use codegen_runtime::TagValue;

#[test]
fn test_missing_print_conv_tracking() {
    // Clear any previous conversions
    clear_missing_conversions();

    // Create a test value
    let value = TagValue::String("test value".to_string());

    // Call missing_print_conv
    let result = missing_print_conv(
        42,                                  // tag_id
        "TestTag",                           // tag_name
        "TestGroup",                         // group
        "complex perl expression goes here", // expression
        &value,
    );

    // Should return the value unchanged
    assert_eq!(result, value);

    // Check that it was tracked
    let missing = get_missing_conversions();
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].tag_id, 42);
    assert_eq!(missing[0].tag_name, "TestTag");
    assert_eq!(missing[0].group, "TestGroup");
    assert_eq!(missing[0].expression, "complex perl expression goes here");
    assert!(matches!(missing[0].conv_type, ConversionType::PrintConv));
}

#[test]
fn test_missing_value_conv_tracking() {
    // Clear any previous conversions
    clear_missing_conversions();

    // Create a test value
    let value = TagValue::I32(100);

    // Call missing_value_conv
    let result = missing_value_conv(
        123,                        // tag_id
        "AnotherTag",               // tag_name
        "AnotherGroup",             // group
        "$val =~ s/foo/bar/; $val", // expression
        &value,
    );

    // Should return the value unchanged
    assert_eq!(result, value);

    // Check that it was tracked
    let missing = get_missing_conversions();
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].tag_id, 123);
    assert_eq!(missing[0].tag_name, "AnotherTag");
    assert_eq!(missing[0].group, "AnotherGroup");
    assert_eq!(missing[0].expression, "$val =~ s/foo/bar/; $val");
    assert!(matches!(missing[0].conv_type, ConversionType::ValueConv));
}

#[test]
fn test_duplicate_expressions_tracked_once() {
    // Clear any previous conversions
    clear_missing_conversions();

    let value = TagValue::String("test".to_string());
    let expression = "same expression";

    // Call the same missing conversion multiple times
    missing_print_conv(1, "Tag1", "Group1", expression, &value);
    missing_print_conv(2, "Tag2", "Group2", expression, &value);
    missing_print_conv(3, "Tag3", "Group3", expression, &value);

    // Should only track once per unique expression + type
    let missing = get_missing_conversions();
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].expression, expression);
}

#[test]
fn test_different_conv_types_tracked_separately() {
    // Clear any previous conversions
    clear_missing_conversions();

    let value = TagValue::String("test".to_string());
    let expression = "same expression";

    // Call with same expression but different conversion types
    missing_print_conv(1, "Tag1", "Group1", expression, &value);
    missing_value_conv(2, "Tag2", "Group2", expression, &value);

    // Should track both since they're different conversion types
    let missing = get_missing_conversions();
    assert_eq!(missing.len(), 2);

    // Check we have one of each type
    let has_print = missing
        .iter()
        .any(|m| matches!(m.conv_type, ConversionType::PrintConv));
    let has_value = missing
        .iter()
        .any(|m| matches!(m.conv_type, ConversionType::ValueConv));
    assert!(has_print);
    assert!(has_value);
}

#[test]
fn test_clear_missing_conversions() {
    // Add some conversions
    let value = TagValue::String("test".to_string());
    missing_print_conv(1, "Tag", "Group", "expr1", &value);
    missing_value_conv(2, "Tag2", "Group2", "expr2", &value);

    // Verify they're tracked
    assert_eq!(get_missing_conversions().len(), 2);

    // Clear them
    clear_missing_conversions();

    // Verify they're gone
    assert_eq!(get_missing_conversions().len(), 0);
}
