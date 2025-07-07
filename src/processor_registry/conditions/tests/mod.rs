//! Tests for the condition evaluation system

use super::*;
use crate::formats::FileFormat;

#[test]
fn test_simple_equality_condition() {
    let mut evaluator = ConditionEvaluator::new();
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
    let mut evaluator = ConditionEvaluator::new();
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
    let mut evaluator = ConditionEvaluator::new();
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
    let mut evaluator = ConditionEvaluator::new();

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
    let mut evaluator = ConditionEvaluator::new();

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
    let mut evaluator = ConditionEvaluator::new();

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
    let mut evaluator = ConditionEvaluator::new();

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
    let mut evaluator = ConditionEvaluator::new();

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
    let mut evaluator = ConditionEvaluator::new();

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
    let mut evaluator = ConditionEvaluator::new();

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
    let mut evaluator = ConditionEvaluator::new();

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
    let mut evaluator = ConditionEvaluator::new();

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
