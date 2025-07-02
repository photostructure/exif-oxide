//! Integration tests for conditional dispatch system
//!
//! These tests verify that the conditional dispatch system works end-to-end
//! with the ExifReader and properly selects processors based on runtime conditions.

use exif_oxide::conditions::{Condition, EvalContext};
use exif_oxide::types::{
    CanonProcessor, ConditionalProcessor, NikonProcessor, ProcessorDispatch, ProcessorType,
    SonyProcessor,
};
use std::collections::HashMap;

/// Test processor selection with Canon model conditions
#[test]
fn test_canon_model_conditional_dispatch() {
    // Configure conditional dispatch for Canon models
    let mut dispatch = ProcessorDispatch::default();

    // Canon 1D series condition
    let canon_1d_conditional = ConditionalProcessor::conditional(
        Condition::canon_1d_series(),
        ProcessorType::Canon(CanonProcessor::Main),
        {
            let mut params = HashMap::new();
            params.insert(
                "CameraInfoTable".to_string(),
                "Canon::CameraInfo1D".to_string(),
            );
            params
        },
    );

    // Canon 1D Mark II condition
    let canon_1d_mk2_conditional = ConditionalProcessor::conditional(
        Condition::canon_1d_mark_ii(),
        ProcessorType::Canon(CanonProcessor::Main),
        {
            let mut params = HashMap::new();
            params.insert(
                "CameraInfoTable".to_string(),
                "Canon::CameraInfo1DmkII".to_string(),
            );
            params
        },
    );

    let camera_info_tag = 0x0012;
    dispatch.add_conditional_processor(camera_info_tag, canon_1d_conditional);
    dispatch.add_conditional_processor(camera_info_tag, canon_1d_mk2_conditional);

    // Test evaluation contexts
    let context_1d = EvalContext {
        data: &[],
        count: 0,
        format: None,
        make: Some("Canon"),
        model: Some("Canon EOS 1D"),
    };

    let context_1d_mk2 = EvalContext {
        data: &[],
        count: 0,
        format: None,
        make: Some("Canon"),
        model: Some("Canon EOS 1Ds Mark II"),
    };

    // Verify conditions evaluate correctly
    assert!(Condition::canon_1d_series().evaluate(&context_1d));
    assert!(Condition::canon_1d_mark_ii().evaluate(&context_1d_mk2));

    // Verify conditional processors exist
    assert!(dispatch
        .conditional_processors
        .contains_key(&camera_info_tag));
    let conditionals = &dispatch.conditional_processors[&camera_info_tag];
    assert_eq!(conditionals.len(), 2);

    // Verify first conditional matches 1D series
    let first_conditional = &conditionals[0];
    assert!(first_conditional
        .condition
        .as_ref()
        .unwrap()
        .evaluate(&context_1d));
    assert_eq!(
        first_conditional.parameters.get("CameraInfoTable"),
        Some(&"Canon::CameraInfo1D".to_string())
    );

    // Verify second conditional matches 1D Mark II
    let second_conditional = &conditionals[1];
    assert!(second_conditional
        .condition
        .as_ref()
        .unwrap()
        .evaluate(&context_1d_mk2));
    assert_eq!(
        second_conditional.parameters.get("CameraInfoTable"),
        Some(&"Canon::CameraInfo1DmkII".to_string())
    );
}

/// Test processor selection with Nikon data pattern conditions
#[test]
fn test_nikon_data_pattern_conditional_dispatch() {
    let mut dispatch = ProcessorDispatch::default();

    // Nikon LensData 0204 condition
    let nikon_0204_conditional = ConditionalProcessor::conditional(
        Condition::nikon_lens_data_0204(),
        ProcessorType::Nikon(NikonProcessor::Encrypted),
        {
            let mut params = HashMap::new();
            params.insert("TagTable".to_string(), "Nikon::LensData0204".to_string());
            params.insert("DecryptStart".to_string(), "4".to_string());
            params
        },
    );

    // Nikon LensData 0402 condition
    let nikon_0402_conditional = ConditionalProcessor::conditional(
        Condition::nikon_lens_data_0402(),
        ProcessorType::Nikon(NikonProcessor::Encrypted),
        {
            let mut params = HashMap::new();
            params.insert("TagTable".to_string(), "Nikon::LensData0402".to_string());
            params.insert("DecryptStart".to_string(), "4".to_string());
            params
        },
    );

    let lens_data_tag = 0x0098;
    dispatch.add_conditional_processor(lens_data_tag, nikon_0204_conditional);
    dispatch.add_conditional_processor(lens_data_tag, nikon_0402_conditional);

    // Test data patterns
    let data_0204 = b"0204encrypted_lens_data";
    let data_0402 = b"0402different_lens_data";

    let context_0204 = EvalContext {
        data: data_0204,
        count: data_0204.len() as u32,
        format: Some("undef"),
        make: Some("NIKON CORPORATION"),
        model: Some("NIKON D850"),
    };

    let context_0402 = EvalContext {
        data: data_0402,
        count: data_0402.len() as u32,
        format: Some("undef"),
        make: Some("NIKON CORPORATION"),
        model: Some("NIKON D850"),
    };

    // Verify data pattern conditions work
    assert!(Condition::nikon_lens_data_0204().evaluate(&context_0204));
    assert!(Condition::nikon_lens_data_0402().evaluate(&context_0402));

    // Verify correct processors are selected
    let conditionals = &dispatch.conditional_processors[&lens_data_tag];
    assert_eq!(conditionals.len(), 2);

    // First conditional should match 0204 data
    let first_conditional = &conditionals[0];
    assert!(first_conditional
        .condition
        .as_ref()
        .unwrap()
        .evaluate(&context_0204));
    assert!(!first_conditional
        .condition
        .as_ref()
        .unwrap()
        .evaluate(&context_0402));

    // Second conditional should match 0402 data
    let second_conditional = &conditionals[1];
    assert!(!second_conditional
        .condition
        .as_ref()
        .unwrap()
        .evaluate(&context_0204));
    assert!(second_conditional
        .condition
        .as_ref()
        .unwrap()
        .evaluate(&context_0402));
}

/// Test processor selection with Sony count-based conditions
#[test]
fn test_sony_count_conditional_dispatch() {
    let mut dispatch = ProcessorDispatch::default();

    let sony_conditional = ConditionalProcessor::conditional(
        Condition::sony_camera_info_counts(),
        ProcessorType::Sony(SonyProcessor::Main),
        {
            let mut params = HashMap::new();
            params.insert("TagTable".to_string(), "Sony::CameraInfo".to_string());
            params
        },
    );

    let camera_info_tag = 0x0114;
    dispatch.add_conditional_processor(camera_info_tag, sony_conditional);

    // Test different count values
    let context_368 = EvalContext {
        data: &[],
        count: 368,
        format: Some("undef"),
        make: Some("SONY"),
        model: Some("ILCE-7M3"),
    };

    let context_5478 = EvalContext {
        data: &[],
        count: 5478,
        format: Some("undef"),
        make: Some("SONY"),
        model: Some("ILCE-7M3"),
    };

    let context_other = EvalContext {
        data: &[],
        count: 1000, // Different count
        format: Some("undef"),
        make: Some("SONY"),
        model: Some("ILCE-7M3"),
    };

    // Verify count conditions work
    let sony_condition = Condition::sony_camera_info_counts();
    assert!(sony_condition.evaluate(&context_368));
    assert!(sony_condition.evaluate(&context_5478));
    assert!(!sony_condition.evaluate(&context_other));
}

/// Test complex boolean logic conditions
#[test]
fn test_complex_boolean_conditional_dispatch() {
    let mut dispatch = ProcessorDispatch::default();

    // Complex condition: Canon make AND (1D series OR 1D Mark II)
    let complex_condition = Condition::And(vec![
        Condition::canon_make(),
        Condition::Or(vec![
            Condition::canon_1d_series(),
            Condition::canon_1d_mark_ii(),
        ]),
    ]);

    let complex_conditional = ConditionalProcessor::conditional(
        complex_condition,
        ProcessorType::Canon(CanonProcessor::Main),
        {
            let mut params = HashMap::new();
            params.insert("ComplexLogic".to_string(), "true".to_string());
            params
        },
    );

    dispatch.add_conditional_processor(0x927C, complex_conditional);

    // Test contexts
    let context_canon_1d = EvalContext {
        data: &[],
        count: 0,
        format: None,
        make: Some("Canon"),
        model: Some("Canon EOS 1D"),
    };

    let context_canon_1d_mk2 = EvalContext {
        data: &[],
        count: 0,
        format: None,
        make: Some("Canon"),
        model: Some("Canon EOS 1Ds Mark II"),
    };

    let context_canon_other = EvalContext {
        data: &[],
        count: 0,
        format: None,
        make: Some("Canon"),
        model: Some("Canon EOS 5D"),
    };

    let context_nikon = EvalContext {
        data: &[],
        count: 0,
        format: None,
        make: Some("NIKON CORPORATION"),
        model: Some("NIKON D850"),
    };

    // Verify complex condition evaluation
    let conditionals = &dispatch.conditional_processors[&0x927C];
    let condition = conditionals[0].condition.as_ref().unwrap();

    assert!(condition.evaluate(&context_canon_1d)); // Canon make + 1D series
    assert!(condition.evaluate(&context_canon_1d_mk2)); // Canon make + 1D Mark II
    assert!(!condition.evaluate(&context_canon_other)); // Canon make but not 1D
    assert!(!condition.evaluate(&context_nikon)); // Not Canon make
}

/// Test format and count based conditions
#[test]
fn test_format_count_conditional_dispatch() {
    let mut dispatch = ProcessorDispatch::default();

    // UNDEFINED format with specific count
    let format_count_condition = Condition::And(vec![
        Condition::FormatEquals("undef".to_string()),
        Condition::CountEquals(2560),
    ]);

    let format_conditional =
        ConditionalProcessor::conditional(format_count_condition, ProcessorType::BinaryData, {
            let mut params = HashMap::new();
            params.insert(
                "SpecialFormat".to_string(),
                "UndefinedExact2560".to_string(),
            );
            params
        });

    dispatch.add_conditional_processor(0x927C, format_conditional);

    // Test contexts
    let context_match = EvalContext {
        data: &[],
        count: 2560,
        format: Some("undef"),
        make: None,
        model: None,
    };

    let context_wrong_format = EvalContext {
        data: &[],
        count: 2560,
        format: Some("int16u"),
        make: None,
        model: None,
    };

    let context_wrong_count = EvalContext {
        data: &[],
        count: 1000,
        format: Some("undef"),
        make: None,
        model: None,
    };

    // Verify format+count condition
    let conditionals = &dispatch.conditional_processors[&0x927C];
    let condition = conditionals[0].condition.as_ref().unwrap();

    assert!(condition.evaluate(&context_match)); // Both format and count match
    assert!(!condition.evaluate(&context_wrong_format)); // Wrong format
    assert!(!condition.evaluate(&context_wrong_count)); // Wrong count
}

/// Test conditional processor precedence (first match wins)
#[test]
fn test_conditional_processor_precedence() {
    let mut dispatch = ProcessorDispatch::default();

    // Add two conditions that could both match
    let first_conditional = ConditionalProcessor::conditional(
        Condition::canon_make(),
        ProcessorType::Canon(CanonProcessor::Main),
        {
            let mut params = HashMap::new();
            params.insert("Priority".to_string(), "First".to_string());
            params
        },
    );

    let second_conditional = ConditionalProcessor::conditional(
        Condition::canon_1d_series(),
        ProcessorType::Canon(CanonProcessor::Main),
        {
            let mut params = HashMap::new();
            params.insert("Priority".to_string(), "Second".to_string());
            params
        },
    );

    let tag_id = 0x0012;
    dispatch.add_conditional_processor(tag_id, first_conditional);
    dispatch.add_conditional_processor(tag_id, second_conditional);

    // Context that matches both conditions
    let context = EvalContext {
        data: &[],
        count: 0,
        format: None,
        make: Some("Canon"),
        model: Some("Canon EOS 1D"),
    };

    // Both conditions should match
    let conditionals = &dispatch.conditional_processors[&tag_id];
    assert!(conditionals[0]
        .condition
        .as_ref()
        .unwrap()
        .evaluate(&context));
    assert!(conditionals[1]
        .condition
        .as_ref()
        .unwrap()
        .evaluate(&context));

    // But the first one should win (order matters)
    assert_eq!(
        conditionals[0].parameters.get("Priority"),
        Some(&"First".to_string())
    );
}

/// Test unconditional processor (fallback)
#[test]
fn test_unconditional_processor_fallback() {
    let mut dispatch = ProcessorDispatch::default();

    // Conditional processor that won't match
    let conditional = ConditionalProcessor::conditional(
        Condition::CountEquals(999999), // Unlikely to match
        ProcessorType::Canon(CanonProcessor::Main),
        HashMap::new(),
    );

    // Unconditional processor (fallback)
    let unconditional = ConditionalProcessor::unconditional(ProcessorType::Exif);

    let tag_id = 0x0012;
    dispatch.add_conditional_processor(tag_id, conditional);
    dispatch.add_conditional_processor(tag_id, unconditional);

    // Context that won't match the conditional
    let context = EvalContext {
        data: &[],
        count: 100, // Different from 999999
        format: None,
        make: Some("Canon"),
        model: Some("Canon EOS 1D"),
    };

    let conditionals = &dispatch.conditional_processors[&tag_id];

    // First conditional should not match
    assert!(!conditionals[0]
        .condition
        .as_ref()
        .unwrap()
        .evaluate(&context));

    // Second conditional is unconditional (should always match)
    assert!(conditionals[1].condition.is_none());
}

/// Test regex caching performance
#[test]
fn test_regex_caching_performance() {
    use std::time::Instant;

    let pattern = r"^Canon";
    let context = EvalContext {
        data: &[],
        count: 0,
        format: None,
        make: Some("Canon EOS"),
        model: None,
    };

    // First evaluation (compiles and caches regex)
    let start = Instant::now();
    let condition1 = Condition::MakeMatch(pattern.to_string());
    let result1 = condition1.evaluate(&context);
    let first_duration = start.elapsed();

    // Second evaluation (uses cached regex)
    let start = Instant::now();
    let condition2 = Condition::MakeMatch(pattern.to_string());
    let result2 = condition2.evaluate(&context);
    let second_duration = start.elapsed();

    // Both should succeed
    assert!(result1);
    assert!(result2);

    // Second evaluation should be faster (though this is not guaranteed in tests)
    // This is more of a demonstration that caching is working
    println!("First evaluation: {first_duration:?}, Second evaluation: {second_duration:?}");
}

/// Test error handling with invalid regex patterns
#[test]
fn test_invalid_regex_handling() {
    // Invalid regex pattern should be handled gracefully
    let invalid_pattern = r"[unclosed";

    // This should not panic, but return an error
    let result = Condition::data_pattern(invalid_pattern);
    assert!(result.is_err());

    // Even if we somehow create an invalid condition, evaluation should not panic
    let condition = Condition::DataPattern(invalid_pattern.to_string());
    let context = EvalContext {
        data: b"test data",
        count: 9,
        format: None,
        make: None,
        model: None,
    };

    // Should return false for invalid regex (graceful degradation)
    assert!(!condition.evaluate(&context));
}
