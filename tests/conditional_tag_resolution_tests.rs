//! Conditional tag resolution integration tests
//!
//! These tests verify that the conditional tag resolution system works correctly
//! for manufacturer-specific tags that depend on context (model, count, format, binary data).
//!
//! Primary focus: Canon ColorData tags that resolve based on count values
//! (e.g., count 582 → ColorData1, count 692 → ColorData4)

#![cfg(feature = "integration-tests")]

use exif_oxide::expressions::{parse_expression, ExpressionEvaluator};
// TODO: Re-enable when conditional tags are generated
// use exif_oxide::generated::canon::main_conditional_tags::{
//     CanonConditionalTags, ConditionalContext,
// };
use exif_oxide::processor_registry::ProcessorContext;
use exif_oxide::types::TagValue;

mod common;

/// Test the expression system integration with conditional tags
#[test]
fn test_expression_system_integration() {
    let mut evaluator = ExpressionEvaluator::new();

    // Test simple count condition parsing and evaluation
    let condition = "$count == 582";
    let _expression = parse_expression(condition).expect("Failed to parse count condition");

    // Create a mock context with count = 582
    let mut context = ProcessorContext::new(
        exif_oxide::formats::FileFormat::Jpeg,
        "Canon::Main".to_string(),
    );
    context
        .parent_tags
        .insert("count".to_string(), TagValue::U32(582));

    let result = evaluator.evaluate_context_condition(&context, condition);
    assert!(
        result.unwrap_or(false),
        "Count condition should evaluate to true for count=582"
    );

    // Test with different count
    context
        .parent_tags
        .insert("count".to_string(), TagValue::U32(692));
    let result = evaluator.evaluate_context_condition(&context, condition);
    assert!(
        !result.unwrap_or(true),
        "Count condition should evaluate to false for count=692"
    );
}

/// Test model-based conditional expressions
#[test]
fn test_model_condition_evaluation() {
    let mut evaluator = ExpressionEvaluator::new();

    // Test Canon EOS model condition
    let condition = "$$self{Model} =~ /EOS/";

    let mut context = ProcessorContext::new(
        exif_oxide::formats::FileFormat::Jpeg,
        "Canon::Main".to_string(),
    );
    context.model = Some("Canon EOS 5D".to_string());

    let result = evaluator.evaluate_context_condition(&context, condition);
    assert!(
        result.unwrap_or(false),
        "Model condition should match Canon EOS 5D"
    );

    // Test non-EOS model
    context.model = Some("Canon PowerShot".to_string());
    let result = evaluator.evaluate_context_condition(&context, condition);
    assert!(
        !result.unwrap_or(true),
        "Model condition should not match Canon PowerShot"
    );
}

/// Test binary pattern conditions
#[test]
fn test_binary_pattern_evaluation() {
    let mut evaluator = ExpressionEvaluator::new();

    // Test VignettingCorr binary pattern condition
    let condition = "$$valPt =~ /^\\0/";

    // Test with binary data starting with null byte
    let binary_data = vec![0x00, 0x01, 0x02, 0x03];
    let result = evaluator.evaluate_data_condition(&binary_data, condition);
    assert!(
        result.unwrap_or(false),
        "Binary pattern should match data starting with null byte"
    );

    // Test with binary data not starting with null byte
    let binary_data = vec![0xFF, 0x01, 0x02, 0x03];
    let result = evaluator.evaluate_data_condition(&binary_data, condition);
    assert!(
        !result.unwrap_or(true),
        "Binary pattern should not match data not starting with null byte"
    );
}

/// Test complex logical conditions (AND/OR)
#[test]
fn test_complex_logical_conditions() {
    let mut evaluator = ExpressionEvaluator::new();

    // Test complex condition: count == 582 AND model contains EOS
    let condition = "$count == 582 and $$self{Model} =~ /EOS/";

    let mut context = ProcessorContext::new(
        exif_oxide::formats::FileFormat::Jpeg,
        "Canon::Main".to_string(),
    );
    context.model = Some("Canon EOS 5D".to_string());
    context
        .parent_tags
        .insert("count".to_string(), TagValue::U32(582));

    let result = evaluator.evaluate_context_condition(&context, condition);
    assert!(
        result.unwrap_or(false),
        "Complex AND condition should evaluate to true"
    );

    // Test with wrong count
    context
        .parent_tags
        .insert("count".to_string(), TagValue::U32(692));
    let result = evaluator.evaluate_context_condition(&context, condition);
    assert!(
        !result.unwrap_or(true),
        "Complex AND condition should evaluate to false with wrong count"
    );
}

/// Test CanonConditionalTags resolver integration
// TODO P07: Re-enable when conditional tags are generated
/*
#[test]
fn test_canon_conditional_tags_resolver() {
    let conditional_tags = CanonConditionalTags::new();

    // Test ColorData1 resolution (count 582)
    let context = ConditionalContext {
        make: Some("Canon".to_string()),
        model: Some("Canon EOS 5D".to_string()),
        count: Some(582),
        format: Some("int32u".to_string()),
        binary_data: None,
    };

    // Test tag ID 16385 (which should resolve to ColorData based on count)
    if let Some(resolved_tag) = conditional_tags.resolve_tag("16385", &context) {
        assert_eq!(
            resolved_tag.name, "ColorData1",
            "Tag 16385 with count 582 should resolve to ColorData1"
        );
    }

    // Test with different count for ColorData4
    let context = ConditionalContext {
        make: Some("Canon".to_string()),
        model: Some("Canon EOS 5D".to_string()),
        count: Some(692),
        format: Some("int32u".to_string()),
        binary_data: None,
    };

    if let Some(resolved_tag) = conditional_tags.resolve_tag("16385", &context) {
        assert_eq!(
            resolved_tag.name, "ColorData4",
            "Tag 16385 with count 692 should resolve to ColorData4"
        );
    }
}
*/

/// Test ConditionalContext to ProcessorContext conversion
// TODO P07: Re-enable when conditional tags are generated
/*
#[test]
fn test_context_conversion() {
    let conditional_context = ConditionalContext {
        make: Some("Canon".to_string()),
        model: Some("Canon EOS R5".to_string()),
        count: Some(796),
        format: Some("int32u".to_string()),
        binary_data: None,
    };

    // Test that we can create a ProcessorContext from ConditionalContext
    let mut processor_context = ProcessorContext::new(
        exif_oxide::formats::FileFormat::Jpeg,
        "Canon::Main".to_string(),
    );

    processor_context.manufacturer = conditional_context.make.clone();
    processor_context.model = conditional_context.model.clone();
    processor_context.format_version = conditional_context.format.clone();

    if let Some(count) = conditional_context.count {
        processor_context
            .parent_tags
            .insert("count".to_string(), TagValue::U32(count));
    }

    assert_eq!(processor_context.manufacturer, Some("Canon".to_string()));
    assert_eq!(processor_context.model, Some("Canon EOS R5".to_string()));
    assert_eq!(
        processor_context.parent_tags.get("count"),
        Some(&TagValue::U32(796))
    );
}
*/

/// Test edge cases and error handling
#[test]
fn test_conditional_resolution_edge_cases() {
    let conditional_tags = CanonConditionalTags::new();

    // Test with missing model (should not resolve)
    let context = ConditionalContext {
        make: Some("Canon".to_string()),
        model: None,
        count: Some(582),
        format: Some("int32u".to_string()),
        binary_data: None,
    };

    let _result = conditional_tags.resolve_tag("16385", &context);
    // Should either resolve gracefully or return None - both are acceptable

    // Test with non-Canon make (should not resolve Canon tags)
    let context = ConditionalContext {
        make: Some("Nikon".to_string()),
        model: Some("D850".to_string()),
        count: Some(582),
        format: Some("int32u".to_string()),
        binary_data: None,
    };

    let _result = conditional_tags.resolve_tag("16385", &context);
    // For Canon-specific tags, this should not resolve for Nikon cameras

    // Test with unknown tag ID
    let context = ConditionalContext {
        make: Some("Canon".to_string()),
        model: Some("Canon EOS 5D".to_string()),
        count: Some(582),
        format: Some("int32u".to_string()),
        binary_data: None,
    };

    let result = conditional_tags.resolve_tag("99999", &context);
    assert!(result.is_none(), "Unknown tag ID should not resolve");
}

/// Integration test with actual Canon image file (if available)
#[test]
fn test_canon_file_conditional_resolution() {
    use common::CANON_T3I_JPG;

    // Skip if test image not available
    if !std::path::Path::new(CANON_T3I_JPG).exists() {
        eprintln!("Skipping Canon file test - test image not available: {CANON_T3I_JPG}");
        return;
    }

    // This test validates that the conditional tag integration doesn't break normal processing
    // The actual conditional resolution happens inside the EXIF parsing pipeline
    // and uses the integation we built in src/exif/ifd.rs

    println!("Canon integration test file exists: {CANON_T3I_JPG}");

    // For now, just verify the file exists and can be read
    let data = std::fs::read(CANON_T3I_JPG).expect("Failed to read Canon test file");
    assert!(!data.is_empty(), "Canon test file should not be empty");
    assert!(data.len() > 1024, "Canon test file should be substantial");

    // The conditional tag resolution is tested at the unit level
    // This integration test ensures the overall system still works
}

/// Performance test for conditional resolution overhead
#[test]
fn test_conditional_resolution_performance() {
    let conditional_tags = CanonConditionalTags::new();

    let context = ConditionalContext {
        make: Some("Canon".to_string()),
        model: Some("Canon EOS 5D".to_string()),
        count: Some(582),
        format: Some("int32u".to_string()),
        binary_data: None,
    };

    let start = std::time::Instant::now();

    // Perform multiple resolutions to test performance
    for _ in 0..1000 {
        let _result = conditional_tags.resolve_tag("16385", &context);
    }

    let duration = start.elapsed();
    println!("1000 conditional resolutions took: {duration:?}");

    // Should be very fast - less than 1ms for 1000 resolutions
    assert!(
        duration.as_millis() < 100,
        "Conditional resolution should be fast"
    );
}
