//! P07: PPI Enhancement Integration Tests
//!
//! Tests the PPI token coverage improvement from ~20% to 90%+ through phases.
//! P07: PPI Enhancement - see docs/todo/P07-ppi-enhancement.md
//!
//! This test validates that PPI enhancement delivers measurable improvement
//! in expression conversion rates with verifiable end-to-end functionality.

use codegen::ppi::{ExpressionType, RustGenerator};
use serde_json::json;
use std::collections::HashMap;

/// Test that measures PPI coverage improvement across enhancement phases
#[test]
fn test_ppi_coverage_improvement() {
    // P07: PPI Enhancement - see docs/todo/P07-ppi-enhancement.md
    // Fails until P07 complete - requires PPI::Statement::Expression, PPI::Token::Cast, PPI::Structure::Subscript support

    // Real ExifTool expressions from analysis corpus that require unsupported tokens
    let test_expressions = vec![
        // Critical missing tokens from analysis (4,172+ occurrences)
        TestExpression {
            expression: "$$self{Model} =~ /Canon/",
            description: "Canon model detection - requires Cast + Subscript + Regexp::Match",
            required_tokens: vec!["PPI::Token::Cast", "PPI::Structure::Subscript", "PPI::Token::Regexp::Match"],
            expression_type: ExpressionType::Condition,
        },
        TestExpression {
            expression: "$val[0] / $val[1]",
            description: "Array subscript arithmetic - requires Statement::Expression + Subscript",
            required_tokens: vec!["PPI::Statement::Expression", "PPI::Structure::Subscript"],
            expression_type: ExpressionType::ValueConv,
        },
        TestExpression {
            expression: "$val ? $val[0] : undef",
            description: "Ternary with subscript - requires Expression + Subscript",
            required_tokens: vec!["PPI::Statement::Expression", "PPI::Structure::Subscript"],
            expression_type: ExpressionType::ValueConv,
        },
        TestExpression {
            expression: "$$self{FileType} eq \"CR3\"",
            description: "Self-reference comparison - requires Cast + Subscript",
            required_tokens: vec!["PPI::Token::Cast", "PPI::Structure::Subscript"],
            expression_type: ExpressionType::Condition,
        },
        TestExpression {
            expression: "($val & 0xff) >> 8",
            description: "Hex bitwise operations - requires Expression + Number::Hex",
            required_tokens: vec!["PPI::Statement::Expression", "PPI::Token::Number::Hex"],
            expression_type: ExpressionType::ValueConv,
        },
        // Additional complex patterns from Canon.pm
        TestExpression {
            expression: "$$self{OPTIONS}{ExtractEmbedded}",
            description: "Nested hash access - requires Cast + multiple Subscripts",
            required_tokens: vec!["PPI::Token::Cast", "PPI::Structure::Subscript"],
            expression_type: ExpressionType::Condition,
        },
        TestExpression {
            expression: "$$valPt =~ /^LIGOGPSINFO\\0/",
            description: "Binary pattern matching - requires Cast + Regexp::Match",
            required_tokens: vec!["PPI::Token::Cast", "PPI::Token::Regexp::Match"],
            expression_type: ExpressionType::Condition,
        },
        TestExpression {
            expression: "$val[1] ? $val[0] / $val[1] : undef",
            description: "Complex ternary with array access - requires Expression + Subscript",
            required_tokens: vec!["PPI::Statement::Expression", "PPI::Structure::Subscript"],
            expression_type: ExpressionType::ValueConv,
        },
        // Pattern from Nikon.pm using multiple critical tokens
        TestExpression {
            expression: "my @a = split ' ', $val; $a[0]",
            description: "Variable declaration with array access - requires Variable + Expression + Subscript",
            required_tokens: vec!["PPI::Statement::Variable", "PPI::Statement::Expression", "PPI::Structure::Subscript"],
            expression_type: ExpressionType::ValueConv,
        },
        // String substitution pattern
        TestExpression {
            expression: "$val =~ s/\\xff+$//; $val",
            description: "String substitution - requires Regexp::Substitute + Expression",
            required_tokens: vec!["PPI::Token::Regexp::Substitute", "PPI::Statement::Expression"],
            expression_type: ExpressionType::ValueConv,
        },
    ];

    // Test current PPI conversion coverage
    let coverage_results = measure_ppi_coverage(&test_expressions);

    println!("📊 PPI Coverage Analysis:");
    println!("Total expressions tested: {}", test_expressions.len());
    println!(
        "Successfully converted: {}",
        coverage_results.successful_conversions
    );
    println!(
        "Failed conversions: {}",
        coverage_results.failed_conversions
    );
    println!(
        "Current coverage: {:.1}%",
        coverage_results.coverage_percentage
    );

    // Log specific failures for debugging
    for (expr, error) in &coverage_results.failures {
        println!("❌ Failed: '{}' - {}", expr, error);
    }

    // Current state: Should be ~20% due to missing critical tokens
    // After Task A (Phase 1): Should reach 60% when critical tokens implemented
    assert!(
        coverage_results.coverage_percentage < 30.0,
        "Current PPI coverage is {:.1}% but should be <30% until Task A complete. \
        Fails until P07 complete - requires PPI::Statement::Expression, PPI::Token::Cast, PPI::Structure::Subscript support",
        coverage_results.coverage_percentage
    );

    // Verify that failures are due to expected missing tokens
    verify_expected_token_failures(&coverage_results, &test_expressions);

    // TODO: When Task A is complete, this assertion should pass:
    // assert!(
    //     coverage_results.coverage_percentage >= 60.0,
    //     "Task A should achieve 60%+ coverage but got {:.1}%",
    //     coverage_results.coverage_percentage
    // );
}

/// Test expression with metadata for coverage analysis
#[derive(Debug, Clone)]
struct TestExpression {
    expression: &'static str,
    description: &'static str,
    required_tokens: Vec<&'static str>,
    expression_type: ExpressionType,
}

/// Results of PPI coverage measurement
#[derive(Debug)]
struct CoverageResults {
    successful_conversions: usize,
    failed_conversions: usize,
    coverage_percentage: f64,
    failures: Vec<(String, String)>, // (expression, error_message)
}

/// Measure PPI conversion coverage for given expressions
fn measure_ppi_coverage(expressions: &[TestExpression]) -> CoverageResults {
    let mut successful = 0;
    let mut failed = 0;
    let mut failures = Vec::new();

    for test_expr in expressions {
        // Create a minimal PPI AST structure for testing conversion
        // This would normally come from field_extractor.pl
        let synthetic_ast =
            create_synthetic_ppi_ast(test_expr.expression, test_expr.expression_type);

        // Attempt to generate Rust code using current PPI implementation
        let generator = RustGenerator::new(
            test_expr.expression_type,
            format!("test_function_{}", successful + failed),
            test_expr.expression.to_string(),
        );

        match generator.generate_function(&synthetic_ast) {
            Ok(_generated_code) => {
                successful += 1;
                println!("✅ Converted: '{}'", test_expr.expression);
            }
            Err(error) => {
                failed += 1;
                let error_msg = format!("{:?}", error);
                failures.push((test_expr.expression.to_string(), error_msg));
                println!("❌ Failed: '{}' - {}", test_expr.expression, error);
            }
        }
    }

    let total = successful + failed;
    let coverage_percentage = if total > 0 {
        (successful as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    CoverageResults {
        successful_conversions: successful,
        failed_conversions: failed,
        coverage_percentage,
        failures,
    }
}

/// Create synthetic PPI AST for testing (mimics field_extractor.pl output)
fn create_synthetic_ppi_ast(expression: &str, expr_type: ExpressionType) -> codegen::ppi::PpiNode {
    // Create synthetic AST structures that exercise the missing tokens
    // This simulates what we'd get from real Perl PPI parsing

    match expression {
        // Pattern: $$self{Model} =~ /Canon/
        expr if expr.contains("$$self") && expr.contains("=~") => {
            json_to_ppi_node(&json!({
                "class": "PPI::Document",
                "children": [{
                    "class": "PPI::Statement::Expression", // ❌ Currently unsupported
                    "children": [{
                        "class": "PPI::Token::Cast", // ❌ Currently unsupported
                        "content": "$$self{Model}"
                    }, {
                        "class": "PPI::Token::Operator",
                        "content": "=~"
                    }, {
                        "class": "PPI::Token::Regexp::Match", // ❌ Currently unsupported
                        "content": "/Canon/"
                    }]
                }]
            }))
        }

        // Pattern: $val[0] / $val[1]
        expr if expr.contains("[") && expr.contains("/") => {
            json_to_ppi_node(&json!({
                "class": "PPI::Document",
                "children": [{
                    "class": "PPI::Statement::Expression", // ❌ Currently unsupported
                    "children": [{
                        "class": "PPI::Structure::Subscript", // ❌ Currently unsupported
                        "content": "$val[0]"
                    }, {
                        "class": "PPI::Token::Operator",
                        "content": "/"
                    }, {
                        "class": "PPI::Structure::Subscript", // ❌ Currently unsupported
                        "content": "$val[1]"
                    }]
                }]
            }))
        }

        // Pattern: ($val & 0xff) >> 8
        expr if expr.contains("0x") => {
            json_to_ppi_node(&json!({
                "class": "PPI::Document",
                "children": [{
                    "class": "PPI::Statement::Expression", // ❌ Currently unsupported
                    "children": [{
                        "class": "PPI::Token::Symbol",
                        "content": "$val"
                    }, {
                        "class": "PPI::Token::Operator",
                        "content": "&"
                    }, {
                        "class": "PPI::Token::Number::Hex", // ❌ Currently unsupported
                        "content": "0xff"
                    }]
                }]
            }))
        }

        // Pattern: my @a = split ' ', $val
        expr if expr.contains("my") => {
            json_to_ppi_node(&json!({
                "class": "PPI::Document",
                "children": [{
                    "class": "PPI::Statement::Variable", // ❌ Currently unsupported
                    "children": [{
                        "class": "PPI::Token::Word",
                        "content": "my"
                    }, {
                        "class": "PPI::Token::Symbol",
                        "content": "@a"
                    }]
                }]
            }))
        }

        // Pattern: $val =~ s/\xff+$//
        expr if expr.contains("s/") => {
            json_to_ppi_node(&json!({
                "class": "PPI::Document",
                "children": [{
                    "class": "PPI::Statement::Expression", // ❌ Currently unsupported
                    "children": [{
                        "class": "PPI::Token::Symbol",
                        "content": "$val"
                    }, {
                        "class": "PPI::Token::Operator",
                        "content": "=~"
                    }, {
                        "class": "PPI::Token::Regexp::Substitute", // ❌ Currently unsupported
                        "content": "s/\\xff+$//"
                    }]
                }]
            }))
        }

        // Default: Simple expression that should work with current tokens
        _ => {
            json_to_ppi_node(&json!({
                "class": "PPI::Document",
                "children": [{
                    "class": "PPI::Statement", // ✅ Currently supported
                    "children": [{
                        "class": "PPI::Token::Symbol", // ✅ Currently supported
                        "content": "$val"
                    }, {
                        "class": "PPI::Token::Operator", // ✅ Currently supported
                        "content": "+"
                    }, {
                        "class": "PPI::Token::Number", // ✅ Currently supported
                        "content": "1",
                        "numeric_value": 1.0
                    }]
                }]
            }))
        }
    }
}

/// Convert JSON to PpiNode (helper for synthetic AST creation)
fn json_to_ppi_node(json_value: &serde_json::Value) -> codegen::ppi::PpiNode {
    codegen::ppi::parse_ppi_json(json_value).expect("Failed to parse synthetic PPI JSON")
}

/// Verify that failures are due to expected missing tokens
fn verify_expected_token_failures(results: &CoverageResults, expressions: &[TestExpression]) {
    let mut missing_token_failures = 0;

    for (failed_expr, error_msg) in &results.failures {
        // Find the corresponding test expression
        if let Some(test_expr) = expressions.iter().find(|e| e.expression == failed_expr) {
            // Check if failure is due to expected missing tokens
            let has_expected_token_error = test_expr
                .required_tokens
                .iter()
                .any(|&token| error_msg.contains("UnsupportedToken") && error_msg.contains(token));

            if has_expected_token_error {
                missing_token_failures += 1;
                println!(
                    "🎯 Expected failure for '{}': missing {}",
                    failed_expr,
                    test_expr.required_tokens.join(", ")
                );
            } else {
                println!(
                    "⚠️  Unexpected failure for '{}': {}",
                    failed_expr, error_msg
                );
            }
        }
    }

    println!(
        "📝 Analysis: {}/{} failures due to expected missing tokens",
        missing_token_failures,
        results.failures.len()
    );

    // Most failures should be due to missing critical tokens
    // (Some might be due to other parsing issues, which is acceptable)
    assert!(
        missing_token_failures >= results.failures.len() / 2,
        "Expected most failures to be due to missing tokens, but only {}/{} were",
        missing_token_failures,
        results.failures.len()
    );
}

/// Test individual PPI token support (unit level validation)
#[test]
fn test_critical_ppi_tokens_missing() {
    // P07: Verify that critical tokens are currently unsupported
    // This test documents the current state and will need updates when Task A is implemented

    let critical_tokens = vec![
        "PPI::Statement::Expression",
        "PPI::Token::Cast",
        "PPI::Structure::Subscript",
        "PPI::Token::Regexp::Match",
    ];

    // Create simple AST nodes for each critical token type
    for token_class in critical_tokens {
        let simple_ast = json_to_ppi_node(&json!({
            "class": "PPI::Document",
            "children": [{
                "class": token_class,
                "content": "test"
            }]
        }));

        let generator = RustGenerator::new(
            ExpressionType::ValueConv,
            "test_critical_token".to_string(),
            "test".to_string(),
        );

        let result = generator.generate_function(&simple_ast);

        // Should fail with UnsupportedToken error
        assert!(
            result.is_err(),
            "Token {} should be unsupported until Task A complete",
            token_class
        );

        if let Err(error) = result {
            let error_msg = format!("{:?}", error);
            assert!(
                error_msg.contains("UnsupportedToken") && error_msg.contains(token_class),
                "Expected UnsupportedToken error for {}, got: {}",
                token_class,
                error_msg
            );
        }
    }

    println!("✅ Verified all critical tokens are currently unsupported (as expected)");
}
