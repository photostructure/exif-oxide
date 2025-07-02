//! Example configurations for conditional dispatch
//!
//! This module demonstrates how to configure conditional processor dispatch
//! for various camera models and data patterns, matching ExifTool's behavior.

use crate::conditions::Condition;
use crate::types::{
    CanonProcessor, ConditionalProcessor, NikonProcessor, ProcessorDispatch, ProcessorType,
};
use std::collections::HashMap;

/// Configure Canon model-specific processor dispatch
/// ExifTool: Canon.pm conditional CameraInfo table selection
pub fn configure_canon_conditional_dispatch() -> ProcessorDispatch {
    let mut dispatch =
        ProcessorDispatch::with_table_processor(ProcessorType::Canon(CanonProcessor::Main));

    // Canon CameraInfo tag (hypothetical tag ID for demonstration)
    let camera_info_tag = 0x0012; // Example tag ID

    // Canon EOS 1D series CameraInfo table selection
    // ExifTool: $$self{Model} =~ /\b1DS?$/
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

    // Canon EOS 1D Mark II series
    // ExifTool: $$self{Model} =~ /\b1Ds? Mark II$/
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

    // Canon EOS 1D Mark III series
    // ExifTool: $$self{Model} =~ /\b1Ds? Mark III$/
    let canon_1d_mk3_conditional = ConditionalProcessor::conditional(
        Condition::canon_1d_mark_iii(),
        ProcessorType::Canon(CanonProcessor::Main),
        {
            let mut params = HashMap::new();
            params.insert(
                "CameraInfoTable".to_string(),
                "Canon::CameraInfo1DmkIII".to_string(),
            );
            params
        },
    );

    // Add conditional processors in order (first match wins)
    dispatch.add_conditional_processor(camera_info_tag, canon_1d_conditional);
    dispatch.add_conditional_processor(camera_info_tag, canon_1d_mk2_conditional);
    dispatch.add_conditional_processor(camera_info_tag, canon_1d_mk3_conditional);

    dispatch
}

/// Configure Nikon data pattern-based processor dispatch
/// ExifTool: Nikon.pm conditional LensData version selection
pub fn configure_nikon_conditional_dispatch() -> ProcessorDispatch {
    let mut dispatch =
        ProcessorDispatch::with_table_processor(ProcessorType::Nikon(NikonProcessor::Main));

    // Nikon LensData tag (hypothetical tag ID for demonstration)
    let lens_data_tag = 0x0098; // Example tag ID

    // Nikon LensData version 0204
    // ExifTool: $$valPt =~ /^0204/
    let nikon_lens_0204_conditional = ConditionalProcessor::conditional(
        Condition::nikon_lens_data_0204(),
        ProcessorType::Nikon(NikonProcessor::Encrypted),
        {
            let mut params = HashMap::new();
            params.insert("TagTable".to_string(), "Nikon::LensData0204".to_string());
            params.insert("DecryptStart".to_string(), "4".to_string());
            params.insert("ByteOrder".to_string(), "LittleEndian".to_string());
            params
        },
    );

    // Nikon LensData version 0402
    // ExifTool: $$valPt =~ /^0402/
    let nikon_lens_0402_conditional = ConditionalProcessor::conditional(
        Condition::nikon_lens_data_0402(),
        ProcessorType::Nikon(NikonProcessor::Encrypted),
        {
            let mut params = HashMap::new();
            params.insert("TagTable".to_string(), "Nikon::LensData0402".to_string());
            params.insert("DecryptStart".to_string(), "4".to_string());
            params.insert("ByteOrder".to_string(), "LittleEndian".to_string());
            params
        },
    );

    // Add conditional processors in order
    dispatch.add_conditional_processor(lens_data_tag, nikon_lens_0204_conditional);
    dispatch.add_conditional_processor(lens_data_tag, nikon_lens_0402_conditional);

    dispatch
}

/// Configure Sony count-based processor dispatch
/// ExifTool: Sony.pm CameraInfo count variants
pub fn configure_sony_conditional_dispatch() -> ProcessorDispatch {
    use crate::types::SonyProcessor;

    let mut dispatch =
        ProcessorDispatch::with_table_processor(ProcessorType::Sony(SonyProcessor::Main));

    // Sony CameraInfo tag (hypothetical tag ID for demonstration)
    let camera_info_tag = 0x0114; // Example tag ID

    // Sony CameraInfo with specific count values
    // ExifTool: $count == 368 or $count == 5478
    let sony_camera_info_conditional = ConditionalProcessor::conditional(
        Condition::sony_camera_info_counts(),
        ProcessorType::Sony(SonyProcessor::Main),
        {
            let mut params = HashMap::new();
            params.insert("TagTable".to_string(), "Sony::CameraInfo".to_string());
            params.insert("ProcessType".to_string(), "BinaryData".to_string());
            params
        },
    );

    dispatch.add_conditional_processor(camera_info_tag, sony_camera_info_conditional);

    dispatch
}

/// Configure complex conditional dispatch with multiple conditions
/// ExifTool: Complex boolean logic examples
pub fn configure_complex_conditional_dispatch() -> ProcessorDispatch {
    let mut dispatch = ProcessorDispatch::default();

    // Complex condition: Canon make AND specific model range
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
            params.insert(
                "ProcessorVariant".to_string(),
                "ComplexCanonLogic".to_string(),
            );
            params
        },
    );

    dispatch.add_conditional_processor(0x927C, complex_conditional); // MakerNotes tag

    dispatch
}

/// Configure format-based conditional dispatch
/// ExifTool: Format and count based conditions
pub fn configure_format_conditional_dispatch() -> ProcessorDispatch {
    let mut dispatch = ProcessorDispatch::default();

    // UNDEFINED format with specific count
    let format_count_condition = Condition::And(vec![
        Condition::FormatEquals("undef".to_string()),
        Condition::CountRange(2560, 2560), // Exactly 2560 bytes
    ]);

    let format_conditional =
        ConditionalProcessor::conditional(format_count_condition, ProcessorType::BinaryData, {
            let mut params = HashMap::new();
            params.insert("DataFormat".to_string(), "SpecialUndefined".to_string());
            params
        });

    dispatch.add_conditional_processor(0x927C, format_conditional);

    dispatch
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conditions::EvalContext;

    #[test]
    fn test_canon_conditional_dispatch_configuration() {
        let dispatch = configure_canon_conditional_dispatch();

        // Verify that conditional processors were added
        assert!(dispatch.conditional_processors.contains_key(&0x0012));
        let conditionals = &dispatch.conditional_processors[&0x0012];
        assert_eq!(conditionals.len(), 3);

        // Verify first conditional is for 1D series
        let first_conditional = &conditionals[0];
        assert!(first_conditional.condition.is_some());
        assert!(matches!(
            first_conditional.processor,
            ProcessorType::Canon(CanonProcessor::Main)
        ));
    }

    #[test]
    fn test_nikon_conditional_dispatch_configuration() {
        let dispatch = configure_nikon_conditional_dispatch();

        // Verify that conditional processors were added
        assert!(dispatch.conditional_processors.contains_key(&0x0098));
        let conditionals = &dispatch.conditional_processors[&0x0098];
        assert_eq!(conditionals.len(), 2);

        // Verify parameters are set correctly
        let lens_0204_conditional = &conditionals[0];
        assert_eq!(
            lens_0204_conditional.parameters.get("DecryptStart"),
            Some(&"4".to_string())
        );
        assert_eq!(
            lens_0204_conditional.parameters.get("ByteOrder"),
            Some(&"LittleEndian".to_string())
        );
    }

    #[test]
    fn test_condition_evaluation_with_canon_model() {
        let context = EvalContext {
            data: &[],
            count: 0,
            format: None,
            make: Some("Canon"),
            model: Some("Canon EOS 1D"),
        };

        let condition = Condition::canon_1d_series();
        assert!(condition.evaluate(&context));

        let context_mark_ii = EvalContext {
            model: Some("Canon EOS 1Ds Mark II"),
            ..context
        };

        let condition_mark_ii = Condition::canon_1d_mark_ii();
        assert!(condition_mark_ii.evaluate(&context_mark_ii));
    }

    #[test]
    fn test_condition_evaluation_with_nikon_data() {
        // Test with binary data that matches Nikon pattern
        let data_0204 = b"0204encrypted_lens_data";
        let context = EvalContext {
            data: data_0204,
            count: data_0204.len() as u32,
            format: Some("undef"),
            make: Some("NIKON CORPORATION"),
            model: Some("NIKON D850"),
        };

        let condition = Condition::nikon_lens_data_0204();
        assert!(condition.evaluate(&context));

        // Test with different version
        let data_0402 = b"0402encrypted_lens_data";
        let context_0402 = EvalContext {
            data: data_0402,
            ..context
        };

        let condition_0402 = Condition::nikon_lens_data_0402();
        assert!(condition_0402.evaluate(&context_0402));
    }

    #[test]
    fn test_complex_conditional_logic() {
        let context = EvalContext {
            data: &[],
            count: 0,
            format: None,
            make: Some("Canon"),
            model: Some("Canon EOS 1D Mark II"),
        };

        // Test complex AND/OR logic
        let complex_condition = Condition::And(vec![
            Condition::canon_make(),
            Condition::Or(vec![
                Condition::canon_1d_series(),
                Condition::canon_1d_mark_ii(),
            ]),
        ]);

        assert!(complex_condition.evaluate(&context));
    }
}
