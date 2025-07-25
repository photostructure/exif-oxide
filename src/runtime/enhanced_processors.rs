//! Enhanced subdirectory processors with runtime condition evaluation
//!
//! This module provides examples of how existing tag kit subdirectory processors
//! can be enhanced with runtime condition evaluation capabilities while maintaining
//! backward compatibility with the existing processor interface.

use super::condition_evaluator::SubdirectoryContext;
use super::integration::{
    create_subdirectory_context_from_exif, ConditionProcessorPair, RuntimeSubdirectoryDispatcher,
};
use crate::tiff_types::ByteOrder;
use crate::types::{Result, TagValue};
use std::collections::HashMap;
use tracing::debug;

/// Type alias for enhanced processor wrapper function signature
pub type EnhancedProcessorWrapper = Box<
    dyn Fn(&[u8], ByteOrder, Option<&HashMap<String, TagValue>>) -> Result<Vec<(String, TagValue)>>,
>;

/// Enhanced version of a Canon subdirectory processor with runtime condition support
///
/// This demonstrates how to integrate runtime condition evaluation with existing
/// tag kit subdirectory processors. The function maintains the same signature as
/// the generated processors but adds internal runtime condition evaluation.
pub fn enhanced_canon_subdirectory_processor(
    data: &[u8],
    byte_order: ByteOrder,
    exif_context: Option<&HashMap<String, TagValue>>,
) -> Result<Vec<(String, TagValue)>> {
    debug!("Processing Canon subdirectory with enhanced runtime evaluation");

    // Create subdirectory context from available metadata
    let context = if let Some(exif_metadata) = exif_context {
        create_subdirectory_context_from_exif(data, byte_order, exif_metadata)
    } else {
        SubdirectoryContext::from_data(data, None, None, byte_order)
    };

    let mut dispatcher = RuntimeSubdirectoryDispatcher::new();

    // Define conditions and their corresponding processors
    // These conditions would come from the ExifTool source analysis
    let conditions: Vec<ConditionProcessorPair> = vec![
        // Example: Process encrypted data for specific models
        (
            "$$self{Model} =~ /EOS.*R5/".to_string(),
            process_canon_encrypted_data,
        ),
        // Example: Handle different data formats based on binary patterns
        ("$$valPt =~ /^0204/".to_string(), process_canon_format_v2),
        // Example: Default processor for all other cases
        (
            "1".to_string(), // Always true condition as fallback
            process_canon_default,
        ),
    ];

    dispatcher.dispatch_with_conditions(data, byte_order, &context, &conditions)
}

/// Example processor for Canon encrypted data
fn process_canon_encrypted_data(
    data: &[u8],
    _byte_order: ByteOrder,
) -> Result<Vec<(String, TagValue)>> {
    debug!("Processing Canon encrypted data ({} bytes)", data.len());
    // This would contain the actual Canon encrypted data processing logic
    Ok(vec![
        (
            "EncryptionType".to_string(),
            TagValue::String("Canon_EOS_R5".to_string()),
        ),
        (
            "ProcessedBy".to_string(),
            TagValue::String("enhanced_runtime_processor".to_string()),
        ),
    ])
}

/// Example processor for Canon format v2
fn process_canon_format_v2(data: &[u8], _byte_order: ByteOrder) -> Result<Vec<(String, TagValue)>> {
    debug!("Processing Canon format v2 data ({} bytes)", data.len());
    // This would contain the actual Canon format v2 processing logic
    Ok(vec![
        (
            "FormatVersion".to_string(),
            TagValue::String("2.04".to_string()),
        ),
        ("DataLength".to_string(), TagValue::U32(data.len() as u32)),
    ])
}

/// Example default processor for Canon data
fn process_canon_default(data: &[u8], _byte_order: ByteOrder) -> Result<Vec<(String, TagValue)>> {
    debug!(
        "Processing Canon data with default processor ({} bytes)",
        data.len()
    );
    // This would contain the actual Canon default processing logic
    Ok(vec![
        (
            "ProcessorType".to_string(),
            TagValue::String("default".to_string()),
        ),
        ("DataLength".to_string(), TagValue::U32(data.len() as u32)),
    ])
}

/// Demonstrates how to wrap an existing tag kit processor with runtime capabilities
///
/// This function shows how the generated tag kit processors can be enhanced
/// without modifying the generated code, by creating wrapper functions that
/// add runtime condition evaluation on top of the existing static dispatch.
pub fn wrap_existing_processor<F>(
    original_processor: F,
    runtime_conditions: Vec<ConditionProcessorPair>,
) -> EnhancedProcessorWrapper
where
    F: Fn(&[u8], ByteOrder) -> Result<Vec<(String, TagValue)>> + 'static,
{
    Box::new(
        move |data: &[u8],
              byte_order: ByteOrder,
              exif_context: Option<&HashMap<String, TagValue>>| {
            // First try runtime conditions
            if !runtime_conditions.is_empty() {
                let context = if let Some(exif_metadata) = exif_context {
                    create_subdirectory_context_from_exif(data, byte_order, exif_metadata)
                } else {
                    SubdirectoryContext::from_data(data, None, None, byte_order)
                };

                let mut dispatcher = RuntimeSubdirectoryDispatcher::new();
                let result = dispatcher.dispatch_with_conditions(
                    data,
                    byte_order,
                    &context,
                    &runtime_conditions,
                )?;

                // If runtime conditions produced results, use them
                if !result.is_empty() {
                    return Ok(result);
                }
            }

            // Fall back to original processor
            original_processor(data, byte_order)
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_canon_processor_with_model_condition() {
        let mut exif_metadata = HashMap::new();
        exif_metadata.insert("Make".to_string(), TagValue::String("Canon".to_string()));
        exif_metadata.insert("Model".to_string(), TagValue::String("EOS R5".to_string()));

        let data = &[0x01, 0x02, 0x03, 0x04];
        let result = enhanced_canon_subdirectory_processor(
            data,
            ByteOrder::LittleEndian,
            Some(&exif_metadata),
        )
        .unwrap();

        assert!(!result.is_empty());
        assert_eq!(result[0].0, "EncryptionType");
        assert_eq!(result[0].1, TagValue::String("Canon_EOS_R5".to_string()));
    }

    #[test]
    fn test_enhanced_canon_processor_with_data_pattern() {
        let mut exif_metadata = HashMap::new();
        exif_metadata.insert("Make".to_string(), TagValue::String("Canon".to_string()));
        exif_metadata.insert("Model".to_string(), TagValue::String("EOS 5D".to_string())); // Not R5

        let data = &[0x02, 0x04, 0x00, 0x01]; // Matches ^0204 pattern
        let result = enhanced_canon_subdirectory_processor(
            data,
            ByteOrder::LittleEndian,
            Some(&exif_metadata),
        )
        .unwrap();

        assert!(!result.is_empty());
        assert_eq!(result[0].0, "FormatVersion");
        assert_eq!(result[0].1, TagValue::String("2.04".to_string()));
    }

    #[test]
    fn test_enhanced_canon_processor_fallback() {
        let mut exif_metadata = HashMap::new();
        exif_metadata.insert("Make".to_string(), TagValue::String("Canon".to_string()));
        exif_metadata.insert("Model".to_string(), TagValue::String("EOS 5D".to_string())); // Not R5

        let data = &[0x01, 0x02, 0x03, 0x04]; // Doesn't match ^0204 pattern
        let result = enhanced_canon_subdirectory_processor(
            data,
            ByteOrder::LittleEndian,
            Some(&exif_metadata),
        )
        .unwrap();

        assert!(!result.is_empty());
        assert_eq!(result[0].0, "ProcessorType");
        assert_eq!(result[0].1, TagValue::String("default".to_string()));
    }
}
