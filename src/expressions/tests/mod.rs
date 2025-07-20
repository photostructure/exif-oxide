//! Tests for the expression evaluation system

use super::*;
use crate::expressions::{parse_expression, types::Expression};
use crate::formats::FileFormat;
use crate::processor_registry::ProcessorContext;

#[test]
fn test_simple_equality_condition() {
    let mut evaluator = ExpressionEvaluator::new();
    let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string())
        .with_manufacturer("Canon".to_string());

    let result = evaluator
        .evaluate_context_condition(&context, "$manufacturer eq 'Canon'")
        .unwrap();
    assert!(result);

    let result = evaluator
        .evaluate_context_condition(&context, "$manufacturer eq 'Nikon'")
        .unwrap();
    assert!(!result);
}

#[test]
fn test_regex_condition() {
    let mut evaluator = ExpressionEvaluator::new();
    let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string())
        .with_model("Canon EOS R5".to_string());

    let result = evaluator
        .evaluate_context_condition(&context, "$model =~ /EOS R5/")
        .unwrap();
    assert!(result);

    let result = evaluator
        .evaluate_context_condition(&context, "$model =~ /R6/")
        .unwrap();
    assert!(!result);
}

#[test]
fn test_exists_condition() {
    let mut evaluator = ExpressionEvaluator::new();
    let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string())
        .with_manufacturer("Canon".to_string());

    let result = evaluator
        .evaluate_context_condition(&context, "exists($manufacturer)")
        .unwrap();
    assert!(result);

    let result = evaluator
        .evaluate_context_condition(&context, "exists($model)")
        .unwrap();
    assert!(!result);
}

#[test]
fn test_complex_logical_operators() {
    let mut evaluator = ExpressionEvaluator::new();

    // Test AND operator
    let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string())
        .with_manufacturer("Canon".to_string())
        .with_model("EOS R5".to_string());

    let result = evaluator
        .evaluate_context_condition(&context, "$manufacturer eq 'Canon' and $model =~ /EOS R5/")
        .unwrap();
    assert!(result);

    let result = evaluator
        .evaluate_context_condition(&context, "$manufacturer eq 'Canon' and $model =~ /R6/")
        .unwrap();
    assert!(!result);

    // Test OR operator
    let result = evaluator
        .evaluate_context_condition(&context, "$model =~ /R5/ or $model =~ /R6/")
        .unwrap();
    assert!(result);

    let result = evaluator
        .evaluate_context_condition(&context, "$model =~ /R6/ or $model =~ /R3/")
        .unwrap();
    assert!(!result);

    // Test NOT operator
    let result = evaluator
        .evaluate_context_condition(&context, "not $manufacturer eq 'Nikon'")
        .unwrap();
    assert!(result);

    let result = evaluator
        .evaluate_context_condition(&context, "!($model =~ /R6/)")
        .unwrap();
    assert!(result);
}

#[test]
fn test_data_pattern_conditions() {
    let mut evaluator = ExpressionEvaluator::new();

    // Test Nikon encryption pattern
    let nikon_encrypted_data = vec![0x02, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04];

    let result = evaluator
        .evaluate_data_condition(&nikon_encrypted_data, "$$valPt =~ /^0200/")
        .unwrap();
    assert!(result);

    // Test pattern that doesn't match
    let result = evaluator
        .evaluate_data_condition(&nikon_encrypted_data, "$$valPt =~ /^0400/")
        .unwrap();
    assert!(!result);

    // Test different encryption patterns
    let nikon_204_data = vec![0x02, 0x04, 0x00, 0x01];
    let result = evaluator
        .evaluate_data_condition(&nikon_204_data, "$$valPt =~ /^0204/")
        .unwrap();
    assert!(result);

    let nikon_402_data = vec![0x04, 0x02, 0x00, 0x01];
    let result = evaluator
        .evaluate_data_condition(&nikon_402_data, "$$valPt =~ /^0402/")
        .unwrap();
    assert!(result);
}

#[test]
fn test_tag_id_conditions() {
    let mut evaluator = ExpressionEvaluator::new();

    // Test hex tag ID
    let context = ProcessorContext::new(FileFormat::Jpeg, "Nikon::Main".to_string())
        .with_manufacturer("NIKON CORPORATION".to_string())
        .with_tag_id(0x001d);

    let result = evaluator
        .evaluate_context_condition(&context, "$tagID == 0x001d")
        .unwrap();
    assert!(result);

    let result = evaluator
        .evaluate_context_condition(&context, "$tagID == 0x00a7")
        .unwrap();
    assert!(!result);

    // Test decimal tag ID
    let context =
        ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string()).with_tag_id(29); // 0x001d in decimal

    let result = evaluator
        .evaluate_context_condition(&context, "$tag_id == 29")
        .unwrap();
    assert!(result);
}

#[test]
fn test_numeric_comparisons() {
    let mut evaluator = ExpressionEvaluator::new();

    let mut context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string());
    context.add_parent_tag("AFInfoVersion".to_string(), TagValue::U16(0x0002));

    // Test greater than
    let result = evaluator
        .evaluate_context_condition(&context, "$AFInfoVersion > 0x0001")
        .unwrap();
    assert!(result);

    // Test greater than or equal
    let result = evaluator
        .evaluate_context_condition(&context, "$AFInfoVersion >= 0x0002")
        .unwrap();
    assert!(result);

    // Test less than
    let result = evaluator
        .evaluate_context_condition(&context, "$AFInfoVersion < 0x0003")
        .unwrap();
    assert!(result);

    // Test less than or equal
    let result = evaluator
        .evaluate_context_condition(&context, "$AFInfoVersion <= 0x0002")
        .unwrap();
    assert!(result);
}

#[test]
fn test_inequality_conditions() {
    let mut evaluator = ExpressionEvaluator::new();

    let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string())
        .with_manufacturer("Canon".to_string());

    // Test not equal (!=)
    let result = evaluator
        .evaluate_context_condition(&context, "$manufacturer != 'Nikon'")
        .unwrap();
    assert!(result);

    // Test not equal (ne)
    let result = evaluator
        .evaluate_context_condition(&context, "$manufacturer ne 'Nikon'")
        .unwrap();
    assert!(result);

    let result = evaluator
        .evaluate_context_condition(&context, "$manufacturer != 'Canon'")
        .unwrap();
    assert!(!result);
}

#[test]
fn test_parentheses_grouping() {
    let mut evaluator = ExpressionEvaluator::new();

    let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string())
        .with_manufacturer("Canon".to_string())
        .with_model("EOS R5".to_string());

    // Test simple parentheses
    let result = evaluator
        .evaluate_context_condition(&context, "($manufacturer eq 'Canon')")
        .unwrap();
    assert!(result);

    // Test AND with parentheses
    let result = evaluator
        .evaluate_context_condition(
            &context,
            "($manufacturer eq 'Canon' and $model eq 'EOS R5')",
        )
        .unwrap();
    assert!(result);

    // Test OR with simple conditions
    let result = evaluator
        .evaluate_context_condition(
            &context,
            "$manufacturer eq 'Canon' or $manufacturer eq 'Nikon'",
        )
        .unwrap();
    assert!(result);

    let result = evaluator
        .evaluate_context_condition(
            &context,
            "$manufacturer eq 'Nikon' or $manufacturer eq 'Sony'",
        )
        .unwrap();
    assert!(!result);
}

#[test]
fn test_regex_negation() {
    let mut evaluator = ExpressionEvaluator::new();

    let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string())
        .with_model("Canon EOS R5".to_string());

    // Test regex negation (!~)
    let result = evaluator
        .evaluate_context_condition(&context, "$model !~ /R6/")
        .unwrap();
    assert!(result);

    let result = evaluator
        .evaluate_context_condition(&context, "$model !~ /R5/")
        .unwrap();
    assert!(!result);
}

#[test]
fn test_binary_data_complex_patterns() {
    let mut evaluator = ExpressionEvaluator::new();

    // Test complex Nikon data patterns
    let complex_data = vec![0x02, 0x04, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06];

    // Test multiple pattern matching attempts
    let result = evaluator
        .evaluate_data_condition(&complex_data, "$$valPt =~ /^0204/ and $$valPt =~ /0102/")
        .unwrap();
    assert!(result);

    let result = evaluator
        .evaluate_data_condition(&complex_data, "$$valPt =~ /^0300/ or $$valPt =~ /^0204/")
        .unwrap();
    assert!(result);
}

#[test]
fn test_error_handling() {
    let mut evaluator = ExpressionEvaluator::new();

    // Test invalid syntax
    assert!(evaluator
        .evaluate_context_condition(&ProcessorContext::default(), "invalid syntax")
        .is_err());

    // Test invalid regex pattern
    assert!(evaluator
        .evaluate_data_condition(&[0u8; 4], "$$valPt =~ /[/")
        .is_err());

    // Test truly unsupported syntax - malformed expression
    let result = evaluator.evaluate_context_condition(
        &ProcessorContext::default(),
        "malformed & invalid #% syntax",
    );
    assert!(
        result.is_err(),
        "Expected error for malformed syntax but got: {result:?}"
    );
}

/// Test conditional tag specific expressions
#[test]
fn test_conditional_tag_expressions() {
    let mut evaluator = ExpressionEvaluator::new();

    // Test Canon ColorData count conditions
    let mut context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string());
    context
        .parent_tags
        .insert("count".to_string(), TagValue::U32(582));

    let result = evaluator
        .evaluate_context_condition(&context, "$count == 582")
        .unwrap();
    assert!(result, "ColorData1 count condition should match");

    let result = evaluator
        .evaluate_context_condition(&context, "$count == 692")
        .unwrap();
    assert!(!result, "ColorData4 count condition should not match");

    // Test format conditions
    context.format_version = Some("int32u".to_string());
    let result = evaluator
        .evaluate_context_condition(&context, "$formatVersion eq \"int32u\"")
        .unwrap();
    assert!(result, "Format condition should match int32u");
}

/// Test conditional tag model detection
#[test]
fn test_canon_model_conditions() {
    let mut evaluator = ExpressionEvaluator::new();

    let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string())
        .with_model("Canon EOS D30".to_string());

    // Test specific model condition from Canon conditional tags
    // Note: $$self{Model} should map to the model field
    let result = evaluator
        .evaluate_context_condition(&context, "$model =~ /EOS D30/")
        .unwrap();
    assert!(result, "EOS D30 model condition should match");

    let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string())
        .with_model("Canon EOS 5D".to_string());

    let result = evaluator
        .evaluate_context_condition(&context, "$model =~ /EOS D30/")
        .unwrap();
    assert!(!result, "EOS D30 model condition should not match EOS 5D");
}

/// Test VignettingCorr binary pattern conditions
#[test]
fn test_vignetting_corr_conditions() {
    let mut evaluator = ExpressionEvaluator::new();

    // Test the complex VignettingCorr condition from Canon conditional tags
    // For binary pattern testing, we need to use hex patterns that work with regex

    // Data that starts with null byte (hex 00)
    let valid_data = vec![0x00, 0x01, 0x02, 0x03];
    let result = evaluator
        .evaluate_data_condition(&valid_data, "$$valPt =~ /^00/")
        .unwrap();
    assert!(
        result,
        "VignettingCorr data should start with null byte (hex 00)"
    );

    // Data that should be excluded (all zeros)
    let excluded_data = vec![0x00, 0x00, 0x00, 0x00];
    let result = evaluator
        .evaluate_data_condition(&excluded_data, "$$valPt =~ /^00/")
        .unwrap();
    assert!(result, "Data starts with null byte");

    // Test that we can parse the exclusion pattern (all zeros)
    let result = evaluator
        .evaluate_data_condition(&excluded_data, "$$valPt =~ /^00000000/")
        .unwrap();
    assert!(result, "Should match all-zero pattern");
}

/// Test count range conditions for different ColorData versions
#[test]
fn test_colordata_count_conditions() {
    let mut evaluator = ExpressionEvaluator::new();

    // Test various ColorData count conditions from Canon
    let test_cases = vec![
        (582, "$count == 582", true),   // ColorData1
        (653, "$count == 653", true),   // ColorData2
        (796, "$count == 796", true),   // ColorData3
        (692, "$count == 692", true),   // ColorData4
        (674, "$count == 674", true),   // ColorData4
        (702, "$count == 702", true),   // ColorData4
        (1227, "$count == 1227", true), // ColorData4
        (999, "$count == 582", false),  // No match
    ];

    for (count_value, condition, expected) in test_cases {
        let mut context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string());
        context
            .parent_tags
            .insert("count".to_string(), TagValue::U32(count_value));

        let result = evaluator
            .evaluate_context_condition(&context, condition)
            .unwrap();
        assert_eq!(
            result, expected,
            "Count {count_value} with condition '{condition}' should be {expected}"
        );
    }
}

#[test]
fn test_parse_value_reference_expression() {
    // Test simple $val{N} reference parsing
    let expression = parse_expression("$val{0}").expect("Failed to parse $val{0}");

    match expression {
        Expression::ValueReference(index) => {
            assert_eq!(index, 0, "Value reference should parse index 0");
        }
        _ => panic!("Expected ValueReference expression, got {expression:?}"),
    }

    // Test different indices
    let expression = parse_expression("$val{15}").expect("Failed to parse $val{15}");
    match expression {
        Expression::ValueReference(index) => {
            assert_eq!(index, 15, "Value reference should parse index 15");
        }
        _ => panic!("Expected ValueReference expression, got {expression:?}"),
    }
}

#[test]
fn test_parse_ceiling_division_expression() {
    // Test complex ceiling division: int(($val{0}+15)/16)
    let expression = parse_expression("int(($val{0}+15)/16)")
        .expect("Failed to parse ceiling division expression");

    match expression {
        Expression::CeilingDivision {
            val_index,
            addend,
            divisor,
        } => {
            assert_eq!(val_index, 0, "Value index should be 0");
            assert_eq!(addend, 15, "Addend should be 15");
            assert_eq!(divisor, 16, "Divisor should be 16");
        }
        _ => panic!("Expected CeilingDivision expression, got {expression:?}"),
    }

    // Test another example: int(($val{3}+7)/8)
    let expression = parse_expression("int(($val{3}+7)/8)")
        .expect("Failed to parse second ceiling division expression");

    match expression {
        Expression::CeilingDivision {
            val_index,
            addend,
            divisor,
        } => {
            assert_eq!(val_index, 3, "Value index should be 3");
            assert_eq!(addend, 7, "Addend should be 7");
            assert_eq!(divisor, 8, "Divisor should be 8");
        }
        _ => panic!("Expected CeilingDivision expression, got {expression:?}"),
    }
}

#[test]
fn test_mathematical_expression_error_handling() {
    // Test division by zero detection
    let result = parse_expression("int(($val{0}+15)/0)");
    assert!(result.is_err(), "Division by zero should return error");

    // Test invalid $val syntax
    let result = parse_expression("$val{invalid}");
    assert!(result.is_err(), "Invalid index should return error");

    // Test incomplete ceiling division
    let result = parse_expression("int(($val{0}+15)");
    assert!(result.is_err(), "Incomplete expression should return error");
}

#[test]
fn test_comprehensive_expression_type_coverage() {
    // Test all Expression enum variants are parseable

    // Exists
    let expr = parse_expression("exists($manufacturer)").unwrap();
    assert!(matches!(expr, Expression::Exists(_)));

    // Equals
    let expr = parse_expression("$count == 582").unwrap();
    assert!(matches!(expr, Expression::Equals(_, _)));

    // GreaterThan
    let expr = parse_expression("$count > 500").unwrap();
    assert!(matches!(expr, Expression::GreaterThan(_, _)));

    // GreaterThanOrEqual
    let expr = parse_expression("$count >= 500").unwrap();
    assert!(matches!(expr, Expression::GreaterThanOrEqual(_, _)));

    // LessThan
    let expr = parse_expression("$count < 1000").unwrap();
    assert!(matches!(expr, Expression::LessThan(_, _)));

    // LessThanOrEqual
    let expr = parse_expression("$count <= 1000").unwrap();
    assert!(matches!(expr, Expression::LessThanOrEqual(_, _)));

    // RegexMatch
    let expr = parse_expression("$model =~ /EOS/").unwrap();
    assert!(matches!(expr, Expression::RegexMatch(_, _)));

    // DataPattern
    let expr = parse_expression("$$valPt =~ /^0204/").unwrap();
    assert!(matches!(expr, Expression::DataPattern(_)));

    // And
    let expr = parse_expression("$count == 582 and $format eq \"int32u\"").unwrap();
    assert!(matches!(expr, Expression::And(_)));

    // Or
    let expr = parse_expression("$count == 582 or $count == 692").unwrap();
    assert!(matches!(expr, Expression::Or(_)));

    // Not
    let expr = parse_expression("not $count == 582").unwrap();
    assert!(matches!(expr, Expression::Not(_)));

    // ValueReference
    let expr = parse_expression("$val{0}").unwrap();
    assert!(matches!(expr, Expression::ValueReference(_)));

    // CeilingDivision
    let expr = parse_expression("int(($val{0}+15)/16)").unwrap();
    assert!(matches!(expr, Expression::CeilingDivision { .. }));
}

#[test]
fn test_mathematical_expressions_fail_in_context_evaluation() {
    let mut evaluator = ExpressionEvaluator::new();
    let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::Main".to_string());

    // Mathematical expressions should fail in context evaluation
    let result = evaluator.evaluate_context_condition(&context, "$val{0}");
    assert!(
        result.is_err(),
        "Value reference should fail in context evaluation"
    );

    let result = evaluator.evaluate_context_condition(&context, "int(($val{0}+15)/16)");
    assert!(
        result.is_err(),
        "Ceiling division should fail in context evaluation"
    );
}
