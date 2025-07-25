//! Integration layer between runtime condition evaluation and tag kit subdirectory dispatch
//!
//! This module provides the bridge between the runtime condition evaluator and the
//! generated tag kit subdirectory processors, enabling dynamic condition evaluation
//! during subdirectory processing.

use super::condition_evaluator::{SubdirectoryConditionEvaluator, SubdirectoryContext};
use crate::tiff_types::ByteOrder;
use crate::types::{Result, TagValue};
use std::collections::HashMap;
use tracing::{debug, warn};

/// Enhanced subdirectory processor that supports runtime condition evaluation
///
/// This trait extends the basic subdirectory processor pattern with runtime
/// condition evaluation capabilities, allowing for dynamic dispatch based on
/// complex conditions that cannot be resolved at code generation time.
pub trait RuntimeSubdirectoryProcessor {
    /// Process subdirectory with runtime condition evaluation
    ///
    /// This method provides the enhanced processing capabilities that include
    /// runtime condition evaluation for complex subdirectory dispatch patterns.
    fn process_with_runtime_conditions(
        &self,
        data: &[u8],
        byte_order: ByteOrder,
        context: &SubdirectoryContext,
        conditions: &[String],
    ) -> Result<Vec<(String, TagValue)>>;
}

/// Runtime subdirectory dispatcher
///
/// This dispatcher integrates the runtime condition evaluator with tag kit
/// subdirectory processors, enabling dynamic condition evaluation for complex
/// subdirectory patterns.
pub struct RuntimeSubdirectoryDispatcher {
    /// Runtime condition evaluator
    condition_evaluator: SubdirectoryConditionEvaluator,
}

impl RuntimeSubdirectoryDispatcher {
    /// Create a new runtime subdirectory dispatcher
    pub fn new() -> Self {
        Self {
            condition_evaluator: SubdirectoryConditionEvaluator::new(),
        }
    }

    /// Dispatch subdirectory processing with runtime condition evaluation
    ///
    /// This method evaluates the provided conditions against the context and
    /// dispatches to the appropriate subdirectory processor if conditions match.
    ///
    /// ## Arguments
    ///
    /// * `data` - The binary data to process
    /// * `byte_order` - Byte order for data interpretation
    /// * `context` - Runtime context for condition evaluation
    /// * `conditions` - List of conditions to evaluate
    /// * `processors` - Map of condition -> processor functions
    ///
    /// ## Returns
    ///
    /// Returns the result of the matching processor, or an empty result if no conditions match.
    pub fn dispatch_with_conditions(
        &mut self,
        data: &[u8],
        byte_order: ByteOrder,
        context: &SubdirectoryContext,
        conditions: &[(String, fn(&[u8], ByteOrder) -> Result<Vec<(String, TagValue)>>)],
    ) -> Result<Vec<(String, TagValue)>> {
        debug!(
            "Dispatching subdirectory with {} conditions and {} bytes of data",
            conditions.len(),
            data.len()
        );

        // Try each condition in order
        for (condition, processor) in conditions {
            debug!("Evaluating condition: {}", condition);
            
            match self.condition_evaluator.evaluate(condition, context) {
                Ok(true) => {
                    debug!("Condition matched: {}", condition);
                    return processor(data, byte_order);
                }
                Ok(false) => {
                    debug!("Condition did not match: {}", condition);
                    continue;
                }
                Err(e) => {
                    warn!("Error evaluating condition '{}': {:?}", condition, e);
                    continue;
                }
            }
        }

        debug!("No conditions matched, returning empty result");
        Ok(vec![])
    }

    /// Enhanced dispatch that supports mixed static and runtime conditions
    ///
    /// This method provides a more sophisticated dispatch pattern that can handle
    /// both statically determinable conditions (like count checks) and runtime
    /// conditions that require dynamic evaluation.
    pub fn dispatch_mixed_conditions(
        &mut self,
        data: &[u8],
        byte_order: ByteOrder,
        context: &SubdirectoryContext,
        static_conditions: &[(String, bool)], // Pre-evaluated static conditions
        runtime_conditions: &[(String, fn(&[u8], ByteOrder) -> Result<Vec<(String, TagValue)>>)],
    ) -> Result<Vec<(String, TagValue)>> {
        debug!(
            "Dispatching with {} static and {} runtime conditions",
            static_conditions.len(),
            runtime_conditions.len()
        );

        // First check static conditions (already evaluated at generation time)
        for (condition_name, matches) in static_conditions {
            if *matches {
                debug!("Static condition matched: {}", condition_name);
                // For static conditions, we need a different dispatch mechanism
                // This would be handled by the generated code calling specific processors
                warn!("Static condition matched but no processor mapping provided: {}", condition_name);
            }
        }

        // Then evaluate runtime conditions
        self.dispatch_with_conditions(data, byte_order, context, runtime_conditions)
    }
}

impl Default for RuntimeSubdirectoryDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create subdirectory context from common EXIF metadata
///
/// This function provides a convenient way to create a SubdirectoryContext
/// from commonly available EXIF metadata, simplifying integration with
/// existing EXIF processing code.
pub fn create_subdirectory_context_from_exif(
    data: &[u8],
    byte_order: ByteOrder,
    exif_metadata: &HashMap<String, TagValue>,
) -> SubdirectoryContext {
    let make = exif_metadata
        .get("Make")
        .and_then(|v| v.as_string().map(|s| s.to_string()));
    
    let model = exif_metadata
        .get("Model")
        .and_then(|v| v.as_string().map(|s| s.to_string()));

    SubdirectoryContext::from_data(data, make, model, byte_order)
        .with_count(data.len())
}

/// Macro to generate runtime-enhanced subdirectory processors
///
/// This macro simplifies the creation of subdirectory processors that support
/// runtime condition evaluation while maintaining compatibility with the
/// existing tag kit processor interface.
#[macro_export]
macro_rules! runtime_subdirectory_processor {
    (
        $fn_name:ident,
        $conditions:expr,
        $processors:expr
    ) => {
        pub fn $fn_name(
            data: &[u8],
            byte_order: crate::tiff_types::ByteOrder,
            make: Option<String>,
            model: Option<String>,
        ) -> crate::types::Result<Vec<(String, crate::types::TagValue)>> {
            use crate::runtime::integration::RuntimeSubdirectoryDispatcher;
            use crate::runtime::condition_evaluator::SubdirectoryContext;
            
            let context = SubdirectoryContext::from_data(data, make, model, byte_order);
            let mut dispatcher = RuntimeSubdirectoryDispatcher::new();
            
            dispatcher.dispatch_with_conditions(data, byte_order, &context, $conditions)
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::condition_evaluator::SubdirectoryContext;

    // Mock processor function for testing
    fn mock_processor(_data: &[u8], _byte_order: ByteOrder) -> Result<Vec<(String, TagValue)>> {
        Ok(vec![("TestTag".to_string(), TagValue::String("TestValue".to_string()))])
    }

    #[test]
    fn test_runtime_dispatch_with_matching_condition() {
        let mut dispatcher = RuntimeSubdirectoryDispatcher::new();
        
        let context = SubdirectoryContext {
            val_ptr: Some(vec![0x02, 0x04, 0x00, 0x01]),
            make: Some("Canon".to_string()),
            model: Some("EOS R5".to_string()),
            ..Default::default()
        };

        let conditions = vec![
            ("$$valPt =~ /^0204/".to_string(), mock_processor as fn(&[u8], ByteOrder) -> Result<Vec<(String, TagValue)>>),
        ];

        let result = dispatcher
            .dispatch_with_conditions(&[0x02, 0x04, 0x00, 0x01], ByteOrder::LittleEndian, &context, &conditions)
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "TestTag");
    }

    #[test]
    fn test_runtime_dispatch_with_non_matching_condition() {
        let mut dispatcher = RuntimeSubdirectoryDispatcher::new();
        
        let context = SubdirectoryContext {
            val_ptr: Some(vec![0x01, 0x02, 0x03, 0x04]),
            make: Some("Canon".to_string()),
            model: Some("EOS R5".to_string()),
            ..Default::default()
        };

        let conditions = vec![
            ("$$valPt =~ /^0204/".to_string(), mock_processor as fn(&[u8], ByteOrder) -> Result<Vec<(String, TagValue)>>),
        ];

        let result = dispatcher
            .dispatch_with_conditions(&[0x01, 0x02, 0x03, 0x04], ByteOrder::LittleEndian, &context, &conditions)
            .unwrap();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_create_context_from_exif() {
        let mut exif_metadata = HashMap::new();
        exif_metadata.insert("Make".to_string(), TagValue::String("Canon".to_string()));
        exif_metadata.insert("Model".to_string(), TagValue::String("EOS R5".to_string()));

        let data = &[0x01, 0x02, 0x03, 0x04];
        let context = create_subdirectory_context_from_exif(
            data, 
            ByteOrder::LittleEndian, 
            &exif_metadata
        );

        assert_eq!(context.make, Some("Canon".to_string()));
        assert_eq!(context.model, Some("EOS R5".to_string()));
        assert_eq!(context.count, Some(4));
        assert_eq!(context.val_ptr, Some(data.to_vec()));
    }
}