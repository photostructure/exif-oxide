//! Integration tests for impl_registry fallback in PPI function registry
//!
//! Tests the three-tier fallback system: PPI → Registry → Placeholder
//! This module validates that registry fallback works correctly when PPI generation fails.

#[cfg(test)]
mod tests {
    use super::super::{FunctionSpec, PpiFunctionRegistry};
    use crate::impl_registry::{classify_valueconv_expression, lookup_printconv, ValueConvType};
    use crate::ppi::{ExpressionType, PpiNode};
    use anyhow::Result;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

    /// Helper to create a minimal FunctionSpec for testing
    fn create_test_function_spec(
        expression_type: ExpressionType,
        original_expression: &str,
        source_module: Option<&str>,
    ) -> FunctionSpec {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let hash_prefix = format!("{:02x}", counter % 256);
        FunctionSpec {
            function_name: format!("test_function_{}", counter),
            module_path: format!("crate::generated::functions::hash_{}", hash_prefix),
            hash_prefix: hash_prefix,
            original_expression: original_expression.to_string(),
            expression_type,
            source_module: source_module.map(|s| s.to_string()),
        }
    }

    /// Test that registry fallback works for PrintConv expressions
    #[test]
    fn test_printconv_registry_fallback() -> Result<()> {
        let mut registry = PpiFunctionRegistry::new();

        // Use a known PrintConv expression that should be in the registry
        let expression = "fnumber_print_conv"; // Should be in printconv_registry
        let function_spec =
            create_test_function_spec(ExpressionType::PrintConv, expression, Some("Exif_pm"));

        // Verify this expression is actually in the registry
        let registry_result = lookup_printconv(expression, "Exif_pm");
        assert!(
            registry_result.is_some(),
            "Test expression should be in printconv registry: {}",
            expression
        );

        // Generate function using registry fallback
        let ast_hash = "test_hash";
        let result =
            registry.generate_registry_or_placeholder_function(&function_spec, ast_hash)?;

        // Should generate registry call, not placeholder
        assert!(
            result.contains("Registry fallback: PrintConv implementation found"),
            "Should generate registry fallback function"
        );
        assert!(
            result.contains("crate::implementations::print_conv"),
            "Should call implementation function"
        );
        assert!(
            !result.contains("Missing implementation"),
            "Should not generate placeholder"
        );

        // Verify statistics were updated correctly
        let stats = registry.stats().conversion_stats;
        assert_eq!(
            stats.print_conv_registry_successes, 1,
            "Should record registry success"
        );
        assert_eq!(
            stats.print_conv_placeholder_fallbacks, 0,
            "Should not record placeholder fallback"
        );

        Ok(())
    }

    /// Test that registry fallback works for ValueConv expressions
    #[test]
    fn test_valueconv_registry_fallback() -> Result<()> {
        let mut registry = PpiFunctionRegistry::new();

        // Use a known ValueConv expression that should be in the registry
        let expression = "Image::ExifTool::GPS::ConvertTimeStamp($val)";
        let function_spec =
            create_test_function_spec(ExpressionType::ValueConv, expression, Some("GPS_pm"));

        // Verify this expression is classified as a custom function
        match classify_valueconv_expression(expression, "GPS_pm") {
            ValueConvType::CustomFunction(_, _) => {
                // Expected - GPS functions should use registry
            }
            _ => panic!("GPS timestamp conversion should use registry, not PPI generation!"),
        }

        // Generate function using registry fallback
        let ast_hash = "test_hash";
        let result =
            registry.generate_registry_or_placeholder_function(&function_spec, ast_hash)?;

        // Should generate registry call, not placeholder
        assert!(
            result.contains("Registry fallback: ValueConv implementation found"),
            "Should generate registry fallback function"
        );
        assert!(
            result.contains("crate::implementations::value_conv"),
            "Should call implementation function"
        );
        assert!(
            !result.contains("Missing implementation"),
            "Should not generate placeholder"
        );

        // Verify statistics
        let stats = registry.stats().conversion_stats;
        assert_eq!(
            stats.value_conv_registry_successes, 1,
            "Should record registry success"
        );

        Ok(())
    }

    /// Test that registry fallback gracefully falls back to placeholders when registry misses
    #[test]
    fn test_registry_miss_falls_back_to_placeholder() -> Result<()> {
        let mut registry = PpiFunctionRegistry::new();

        // Use an expression that should NOT be in any registry
        let expression = "nonexistent_custom_expression_12345";
        let function_spec = create_test_function_spec(
            ExpressionType::PrintConv,
            expression,
            Some("NonExistent_pm"),
        );

        // Verify this expression is NOT in the registry
        let registry_result = lookup_printconv(expression, "NonExistent_pm");
        assert!(
            registry_result.is_none(),
            "Test expression should NOT be in registry: {}",
            expression
        );

        // Generate function - should fall back to placeholder
        let ast_hash = "test_hash";
        let result =
            registry.generate_registry_or_placeholder_function(&function_spec, ast_hash)?;

        // Should generate placeholder, not registry call
        assert!(
            result.contains("PLACEHOLDER: Unsupported expression"),
            "Should generate placeholder function"
        );
        assert!(
            result.contains("Missing implementation"),
            "Should contain missing implementation warning"
        );
        assert!(
            !result.contains("Registry fallback"),
            "Should not indicate registry fallback"
        );

        // Verify statistics
        let stats = registry.stats().conversion_stats;
        assert_eq!(
            stats.print_conv_placeholder_fallbacks, 1,
            "Should record placeholder fallback"
        );
        assert_eq!(
            stats.print_conv_registry_successes, 0,
            "Should not record registry success"
        );

        Ok(())
    }

    /// Test that registry fallback works when source_module is missing
    #[test]
    fn test_no_source_module_falls_back_to_placeholder() -> Result<()> {
        let mut registry = PpiFunctionRegistry::new();

        // Create function spec without source module
        let expression = "some_expression";
        let function_spec = create_test_function_spec(
            ExpressionType::PrintConv,
            expression,
            None, // No source module
        );

        // Generate function - should fall back to placeholder
        let ast_hash = "test_hash";
        let result =
            registry.generate_registry_or_placeholder_function(&function_spec, ast_hash)?;

        // Should generate placeholder since no source module provided
        assert!(
            result.contains("PLACEHOLDER: Unsupported expression"),
            "Should generate placeholder when no source module"
        );

        Ok(())
    }

    /// Test that statistics correctly distinguish between PPI success, registry success, and placeholder fallback
    #[test]
    fn test_statistics_categorization() -> Result<()> {
        let mut registry = PpiFunctionRegistry::new();

        // Test registry success
        let registry_expr = "fnumber_print_conv";
        let registry_spec =
            create_test_function_spec(ExpressionType::PrintConv, registry_expr, Some("Exif_pm"));
        // Record attempt for statistics
        registry.record_conversion_attempt(ExpressionType::PrintConv);
        let _registry_result =
            registry.generate_registry_or_placeholder_function(&registry_spec, "hash1")?;

        // Test placeholder fallback
        let placeholder_expr = "nonexistent_expression";
        let placeholder_spec = create_test_function_spec(
            ExpressionType::PrintConv,
            placeholder_expr,
            Some("NonExistent_pm"),
        );
        // Record attempt for statistics
        registry.record_conversion_attempt(ExpressionType::PrintConv);
        let _placeholder_result =
            registry.generate_registry_or_placeholder_function(&placeholder_spec, "hash2")?;

        // Verify statistics
        let stats = registry.stats().conversion_stats;
        assert_eq!(
            stats.print_conv_registry_successes, 1,
            "Should track one registry success"
        );
        assert_eq!(
            stats.print_conv_placeholder_fallbacks, 1,
            "Should track one placeholder fallback"
        );
        assert_eq!(
            stats.print_conv_ppi_successes, 0,
            "Should not track PPI successes (not tested here)"
        );

        // Test success rates
        assert!(
            stats.registry_success_rate(ExpressionType::PrintConv) > 0.0,
            "Registry success rate should be > 0"
        );
        assert!(
            stats.total_success_rate(ExpressionType::PrintConv) > 0.0,
            "Total success rate should include registry successes"
        );

        Ok(())
    }

    /// Test that existing placeholder behavior is preserved (regression test)
    #[test]
    fn test_placeholder_behavior_unchanged() -> Result<()> {
        let mut registry = PpiFunctionRegistry::new();

        // Test multiple expression types to ensure placeholder generation is consistent
        let test_cases = vec![
            (ExpressionType::PrintConv, "test_printconv_expr"),
            (ExpressionType::ValueConv, "test_valueconv_expr"),
            (ExpressionType::Condition, "test_condition_expr"),
        ];

        for (expr_type, expression) in test_cases {
            let function_spec = create_test_function_spec(expr_type, expression, Some("Test_pm"));

            let result =
                registry.generate_registry_or_placeholder_function(&function_spec, "test_hash")?;

            // All should generate placeholders (since these are fake expressions)
            assert!(
                result.contains("PLACEHOLDER: Unsupported expression"),
                "Should generate placeholder for expression type: {:?}",
                expr_type
            );
            assert!(
                result.contains("Missing implementation"),
                "Should contain missing implementation warning for: {:?}",
                expr_type
            );

            // Check type-specific behavior
            match expr_type {
                ExpressionType::PrintConv => {
                    assert!(
                        result.contains("missing_print_conv"),
                        "PrintConv placeholder should call missing_print_conv"
                    );
                }
                ExpressionType::ValueConv => {
                    assert!(
                        result.contains("missing_value_conv"),
                        "ValueConv placeholder should call missing_value_conv"
                    );
                }
                ExpressionType::Condition => {
                    assert!(
                        result.contains("false"),
                        "Condition placeholder should return false"
                    );
                }
            }
        }

        Ok(())
    }

    /// Test that the integration works end-to-end with real PPI AST failures
    #[test]
    fn test_ppi_failure_to_registry_integration() -> Result<()> {
        let mut registry = PpiFunctionRegistry::new();

        // Create a minimal PPI AST that will likely fail generation
        // This simulates the real workflow where PPI generation fails and we fall back to registry
        let ppi_ast = PpiNode {
            class: "PPI::Token::Unknown".to_string(),
            content: Some("unknown_token".to_string()),
            children: vec![],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        // Use a known registry expression
        let expression = "fnumber_print_conv";
        let function_spec =
            create_test_function_spec(ExpressionType::PrintConv, expression, Some("Exif_pm"));

        // This should trigger: PPI failure → Registry success
        let result =
            registry.generate_function_code_with_stats(&ppi_ast, &function_spec, "test_hash")?;

        // Should generate registry fallback (since PPI will fail on unknown token)
        assert!(
            result.contains("Registry fallback: PrintConv implementation found"),
            "Should fall back to registry after PPI failure"
        );

        // Verify statistics show the right progression
        let stats = registry.stats().conversion_stats;
        assert_eq!(
            stats.print_conv_registry_successes, 1,
            "Should record registry fallback success"
        );
        assert_eq!(
            stats.print_conv_ppi_successes, 0,
            "Should not record PPI success (it failed)"
        );

        Ok(())
    }
}
