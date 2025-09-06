//! Tests for missing conversion tracking in placeholder functions
//!
//! This tests that when the PPI generator can't translate a Perl expression,
//! it generates a placeholder function that properly tracks the missing conversion
//! for --show-missing functionality.

use codegen::ppi::fn_registry::{PpiFunctionRegistry, UsageContext};
use codegen::ppi::shared_pipeline::call_ppi_ast_script;
use codegen::ppi::types::{ExpressionType, PpiNode};
use codegen_runtime::missing::clear_missing_conversions;

#[test]
fn test_placeholder_function_tracks_missing_valueconv() {
    // Clear any previous missing conversions
    clear_missing_conversions();

    // Create a registry and register an expression that can't be translated
    let mut registry = PpiFunctionRegistry::new();

    // This expression uses Perl's foreach which we can't translate
    let expression = "my @a=split(' ',$val); $_/=500 foreach @a; join(' ',@a)";

    // Parse to AST
    let ast_json = call_ppi_ast_script(expression).expect("Failed to parse expression");
    let ast: PpiNode = serde_json::from_str(&ast_json).expect("Failed to parse AST JSON");

    registry
        .register_ast(
            &ast,
            ExpressionType::ValueConv,
            expression,
            Some(UsageContext {
                module: "test_module".to_string(),
                table: "TestTable".to_string(),
                tag: "TestTag".to_string(),
            }),
        )
        .expect("Failed to register AST");

    // Generate the function files
    let files = registry
        .generate_function_files_with_imports("use codegen_runtime::{TagValue, missing}; use codegen_runtime::types::{ExifContext, ExifError}")
        .expect("Failed to generate function files");

    // The generated file should contain a placeholder function
    let function_file = files
        .iter()
        .find(|f| f.path.starts_with("functions/hash_"))
        .expect("Should have generated a function file");

    // Verify the placeholder function calls missing_value_conv
    assert!(
        function_file
            .content
            .contains("codegen_runtime::missing::missing_value_conv"),
        "Placeholder function should call missing_value_conv"
    );

    // Verify the original expression is included (escaped)
    assert!(
        function_file.content.contains("foreach"),
        "Original expression should be preserved in the placeholder"
    );
}

#[test]
fn test_placeholder_function_tracks_missing_printconv() {
    // Clear any previous missing conversions
    clear_missing_conversions();

    // Create a registry and register a PrintConv expression that can't be translated
    let mut registry = PpiFunctionRegistry::new();

    // This expression uses a complex Perl function call we can't translate
    let expression = "Image::ExifTool::GPS::ToDMS($self, $val, 1, \"E\")";

    // Parse to AST
    let ast_json = call_ppi_ast_script(expression).expect("Failed to parse expression");
    let ast: PpiNode = serde_json::from_str(&ast_json).expect("Failed to parse AST JSON");

    registry
        .register_ast(
            &ast,
            ExpressionType::PrintConv,
            expression,
            Some(UsageContext {
                module: "test_module".to_string(),
                table: "TestTable".to_string(),
                tag: "GPSLongitude".to_string(),
            }),
        )
        .expect("Failed to register AST");

    // Generate the function files
    let files = registry
        .generate_function_files_with_imports("use codegen_runtime::{TagValue, missing}; use codegen_runtime::types::{ExifContext, ExifError}")
        .expect("Failed to generate function files");

    // The generated file should contain a placeholder function
    let function_file = files
        .iter()
        .find(|f| f.path.starts_with("functions/hash_"))
        .expect("Should have generated a function file");

    // Verify the placeholder function calls missing_print_conv
    assert!(
        function_file
            .content
            .contains("codegen_runtime::missing::missing_print_conv"),
        "Placeholder function should call missing_print_conv"
    );

    // Verify the original expression is included (escaped)
    assert!(
        function_file.content.contains("ToDMS"),
        "Original expression should be preserved in the placeholder"
    );
}

#[test]
fn test_placeholder_escapes_expression_properly() {
    // Clear any previous missing conversions
    clear_missing_conversions();

    // Create a registry and register an expression with quotes and backslashes
    let mut registry = PpiFunctionRegistry::new();

    // Expression with quotes, backslashes, and newlines
    let expression = r#"$val =~ s/(\d{2})(\d{2})/$1:$2:/; $val"#;

    // Parse to AST
    let ast_json = call_ppi_ast_script(expression).expect("Failed to parse expression");
    let ast: PpiNode = serde_json::from_str(&ast_json).expect("Failed to parse AST JSON");

    registry
        .register_ast(
            &ast,
            ExpressionType::ValueConv,
            expression,
            Some(UsageContext {
                module: "test_module".to_string(),
                table: "TestTable".to_string(),
                tag: "TimeFormat".to_string(),
            }),
        )
        .expect("Failed to register AST");

    // Generate the function files
    let files = registry
        .generate_function_files_with_imports("use codegen_runtime::{TagValue, missing}; use codegen_runtime::types::{ExifContext, ExifError}")
        .expect("Failed to generate function files");

    // The generated file should contain properly escaped expression
    let function_file = files
        .iter()
        .find(|f| f.path.starts_with("functions/hash_"))
        .expect("Should have generated a function file");

    // Verify the expression is properly escaped in the string literal
    // Should have escaped backslashes: \\d instead of \d
    assert!(
        function_file.content.contains(r"\\d"),
        "Backslashes should be escaped in the expression string"
    );
}
