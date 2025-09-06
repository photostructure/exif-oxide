//! Test that expressions with special characters are properly handled
//!
//! This verifies that the generated placeholder functions properly escape
//! Perl expressions containing quotes, backslashes, and newlines.

use codegen_runtime::missing::{
    clear_missing_conversions, get_missing_conversions, missing_print_conv,
};
use codegen_runtime::TagValue;

#[test]
fn test_expression_with_quotes_and_backslashes() {
    clear_missing_conversions();

    // Expression with single quotes, double quotes, and backslashes
    // This is what would be in the generated placeholder function
    let expression = r#"$val =~ s/(\d{2})(\d{2})/$1:$2:/; $val"#;

    let value = TagValue::String("1234".to_string());

    // Call missing_print_conv with the complex expression
    let result = missing_print_conv(0, "TimeFormat", "TestGroup", expression, &value);

    // Should return unchanged
    assert_eq!(result, value);

    // Verify it was tracked with the correct expression
    let missing = get_missing_conversions();
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].expression, expression);
}

#[test]
fn test_multiline_perl_expression() {
    clear_missing_conversions();

    // Multi-line Perl expression as it would appear in generated code
    let expression = "my ($a,$b) = split ' ',$val;\n            return 'Off' unless $a;\n            my %a = (\n                1 => 'Left to Right',\n                2 => 'Right to Left',\n                3 => 'Bottom to Top',\n                4 => 'Top to Bottom',\n            );\n            return(($a{$a} || \"Unknown ($a)\") . ', Shot ' . $b);";

    let value = TagValue::String("1 5".to_string());

    let result = missing_print_conv(0, "PanoramaMode", "Olympus", expression, &value);

    assert_eq!(result, value);

    let missing = get_missing_conversions();
    assert_eq!(missing.len(), 1);
    // Verify the newlines are preserved in the tracked expression
    assert!(missing[0].expression.contains('\n'));
}

#[test]
fn test_expression_with_perl_variables() {
    clear_missing_conversions();

    // Expression with Perl special variables and function calls
    let expression = "Image::ExifTool::Nikon::PrintPC($val,\"No Sharpening\",\"%d\")";

    let value = TagValue::I32(5);

    let result = missing_print_conv(0, "Sharpness", "Nikon", expression, &value);

    assert_eq!(result, value);

    let missing = get_missing_conversions();
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].expression, expression);
    assert_eq!(missing[0].tag_name, "Sharpness");
}
