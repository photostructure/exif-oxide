//! Integration tests for unsupported expression handling
//!
//! These tests verify that expressions we cannot translate from Perl to Rust
//! generate proper placeholder functions that track missing conversions.

use codegen::ppi::fn_registry::{PpiFunctionRegistry, UsageContext};
use codegen::ppi::shared_pipeline::call_ppi_ast_script;
use codegen::ppi::{ExpressionType, PpiNode};

#[test]
fn test_foreach_expression_generates_placeholder() {
    let mut registry = PpiFunctionRegistry::new();

    // Perl foreach expression that we can't translate
    let expression = "my @a=split(' ',$val); $_/=500 foreach @a; join(' ',@a)";

    // Parse to AST
    let ast_json = call_ppi_ast_script(expression).expect("Failed to parse expression");
    let ast: PpiNode = serde_json::from_str(&ast_json).expect("Failed to parse AST JSON");

    // Register the expression
    registry
        .register_ast(
            &ast,
            ExpressionType::ValueConv,
            expression,
            Some(UsageContext {
                module: "test".to_string(),
                table: "test".to_string(),
                tag: "SensorSize".to_string(),
            }),
        )
        .expect("Failed to register AST");

    // Generate function files
    let files = registry
        .generate_function_files_with_imports(
            "use crate::core::{TagValue, missing}; use crate::core::types::{ExifContext, ExifError}"
        )
        .expect("Failed to generate function files");

    // Verify a placeholder was generated
    let has_placeholder = files
        .iter()
        .any(|f| f.content.contains("missing_value_conv") && f.content.contains("foreach"));

    assert!(
        has_placeholder,
        "Should generate placeholder for foreach expression"
    );
}

#[test]
fn test_tr_operator_generates_placeholder() {
    let mut registry = PpiFunctionRegistry::new();

    // Perl tr/// operator that we can't translate
    let expression = "$val=~tr/ /:/; $val";

    // Parse to AST
    let ast_json = call_ppi_ast_script(expression).expect("Failed to parse expression");
    let ast: PpiNode = serde_json::from_str(&ast_json).expect("Failed to parse AST JSON");

    // Register the expression
    registry
        .register_ast(
            &ast,
            ExpressionType::PrintConv,
            expression,
            Some(UsageContext {
                module: "test".to_string(),
                table: "test".to_string(),
                tag: "AspectRatio".to_string(),
            }),
        )
        .expect("Failed to register AST");

    // Generate function files
    let files = registry
        .generate_function_files_with_imports(
            "use crate::core::{TagValue, missing}; use crate::core::types::{ExifContext, ExifError}"
        )
        .expect("Failed to generate function files");

    // Verify a placeholder was generated
    let has_placeholder = files
        .iter()
        .any(|f| f.content.contains("missing_print_conv") && f.content.contains("tr/"));

    assert!(
        has_placeholder,
        "Should generate placeholder for tr/// operator"
    );
}

#[test]
fn test_complex_multiline_generates_placeholder() {
    let mut registry = PpiFunctionRegistry::new();

    // Complex multi-line Perl that we can't translate
    let expression = r#"my ($a,$b) = split ' ',$val;
return 'Off' unless $a;
my %a = (
    1 => 'Left to Right',
    2 => 'Right to Left',
);
return(($a{$a} || "Unknown ($a)") . ', Shot ' . $b);"#;

    // Parse to AST
    let ast_json = call_ppi_ast_script(expression).expect("Failed to parse expression");
    let ast: PpiNode = serde_json::from_str(&ast_json).expect("Failed to parse AST JSON");

    // Register the expression
    registry
        .register_ast(
            &ast,
            ExpressionType::PrintConv,
            expression,
            Some(UsageContext {
                module: "test".to_string(),
                table: "test".to_string(),
                tag: "PanoramaMode".to_string(),
            }),
        )
        .expect("Failed to register AST");

    // Generate function files
    let files = registry
        .generate_function_files_with_imports(
            "use crate::core::{TagValue, missing}; use crate::core::types::{ExifContext, ExifError}"
        )
        .expect("Failed to generate function files");

    // Should generate a placeholder for this complex expression
    let has_placeholder = files.iter().any(|f| {
        f.content.contains("missing_print_conv")
            && (f.content.contains("return") || f.content.contains("split"))
    });

    assert!(
        has_placeholder,
        "Should generate placeholder for complex multiline expression"
    );
}

#[test]
fn test_exiftool_function_call_generates_placeholder() {
    let mut registry = PpiFunctionRegistry::new();

    // ExifTool function call that we can't translate
    let expression = r#"Image::ExifTool::GPS::ToDMS($self, $val, 1, "E")"#;

    // Parse to AST
    let ast_json = call_ppi_ast_script(expression).expect("Failed to parse expression");
    let ast: PpiNode = serde_json::from_str(&ast_json).expect("Failed to parse AST JSON");

    // Register the expression
    registry
        .register_ast(
            &ast,
            ExpressionType::PrintConv,
            expression,
            Some(UsageContext {
                module: "test".to_string(),
                table: "test".to_string(),
                tag: "GPSLongitude".to_string(),
            }),
        )
        .expect("Failed to register AST");

    // Generate function files
    let files = registry
        .generate_function_files_with_imports(
            "use crate::core::{TagValue, missing}; use crate::core::types::{ExifContext, ExifError}"
        )
        .expect("Failed to generate function files");

    // Should generate a placeholder
    let has_placeholder = files
        .iter()
        .any(|f| f.content.contains("missing_print_conv") && f.content.contains("Image::ExifTool"));

    assert!(
        has_placeholder,
        "Should generate placeholder for ExifTool function calls"
    );
}
