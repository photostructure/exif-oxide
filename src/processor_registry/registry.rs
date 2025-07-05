//! ProcessorRegistry - Central registry for processor management and dispatch
//!
//! This module implements the core registry that manages all processors and
//! provides sophisticated capability-based selection with dispatch rules.

use crate::types::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, trace, warn};

use super::{
    BinaryDataProcessor, DispatchRule, ProcessorCapability, ProcessorContext, ProcessorKey,
    ProcessorMetadata, ProcessorResult, SharedProcessor,
};

/// Central registry for processor management and dispatch
///
/// The ProcessorRegistry manages all available processors and provides
/// sophisticated selection logic based on processor capabilities and
/// dispatch rules. It implements ExifTool's processor dispatch system
/// with enhanced type safety and performance.
///
/// ## ExifTool Reference
///
/// This registry implements the logic from ExifTool's processor dispatch:
/// ```perl
/// my $proc = $$subdir{ProcessProc} || $$tagTablePtr{PROCESS_PROC} || \&ProcessExif;
/// ```
///
/// Combined with conditional evaluation and capability assessment.
pub struct ProcessorRegistry {
    /// Registered processors indexed by their keys
    processors: HashMap<ProcessorKey, SharedProcessor>,

    /// Dispatch rules for sophisticated processor selection
    dispatch_rules: Vec<Box<dyn DispatchRule>>,

    /// Fallback chain when no specific processor matches
    fallback_chain: Vec<ProcessorKey>,

    /// Registry statistics for monitoring and debugging
    stats: RegistryStats,
}

impl ProcessorRegistry {
    /// Create a new processor registry
    pub fn new() -> Self {
        Self {
            processors: HashMap::new(),
            dispatch_rules: Vec::new(),
            fallback_chain: Vec::new(),
            stats: RegistryStats::new(),
        }
    }

    /// Register a processor in the registry
    ///
    /// This adds a processor implementation to the registry, making it
    /// available for selection during processing.
    pub fn register_processor<P: BinaryDataProcessor + 'static>(
        &mut self,
        key: ProcessorKey,
        processor: P,
    ) {
        debug!("Registering processor: {}", key);
        self.processors.insert(key, Arc::new(processor));
        self.stats.processors_registered += 1;
    }

    /// Add a dispatch rule to the registry
    ///
    /// Dispatch rules provide sophisticated logic for processor selection
    /// that goes beyond simple capability assessment.
    pub fn add_dispatch_rule<R: DispatchRule + 'static>(&mut self, rule: R) {
        debug!("Adding dispatch rule: {}", rule.description());
        self.dispatch_rules.push(Box::new(rule));
    }

    /// Set the fallback processor chain
    ///
    /// This defines the order of processors to try when no specific
    /// processor is found through normal selection.
    pub fn set_fallback_chain(&mut self, chain: Vec<ProcessorKey>) {
        debug!("Setting fallback chain: {:?}", chain);
        self.fallback_chain = chain;
    }

    /// Find the best processor for the given context
    ///
    /// This is the main selection method that evaluates all registered
    /// processors and returns the most appropriate one based on capabilities
    /// and dispatch rules.
    ///
    /// ## Selection Algorithm
    ///
    /// 1. Evaluate all processors for capability
    /// 2. Filter out incompatible processors
    /// 3. Apply dispatch rules for tie-breaking and preference
    /// 4. Sort by capability and rule preferences
    /// 5. Return the best candidate
    pub fn find_best_processor(
        &self,
        context: &ProcessorContext,
    ) -> Option<(ProcessorKey, SharedProcessor)> {
        self.stats.increment_selections();

        trace!(
            "Finding best processor for context: {:?}",
            context.table_name
        );

        // Step 1: Evaluate all processors for capability
        let mut candidates = Vec::new();
        for (key, processor) in &self.processors {
            let capability = processor.can_process(context);

            trace!("Processor {} capability: {:?}", key, capability);

            if capability.is_compatible() {
                candidates.push((key.clone(), processor.clone(), capability));
            }
        }

        if candidates.is_empty() {
            debug!("No compatible processors found for {}", context.table_name);
            return None;
        }

        // Step 2: Apply dispatch rules for processor selection
        for rule in &self.dispatch_rules {
            if rule.applies_to(context) {
                if let Some((selected_key, selected_processor)) =
                    rule.select_processor(&candidates, context)
                {
                    debug!(
                        "Dispatch rule '{}' selected processor: {}",
                        rule.description(),
                        selected_key
                    );
                    self.stats.increment_rule_selections();
                    return Some((selected_key, selected_processor));
                }
            }
        }

        // Step 3: Sort by capability and return best candidate
        candidates.sort_by(|a, b| {
            // Sort by capability (higher priority first)
            b.2.priority_score()
                .cmp(&a.2.priority_score())
                // Then by processor key for deterministic results
                .then_with(|| a.0.to_string().cmp(&b.0.to_string()))
        });

        let (best_key, best_processor, best_capability) = candidates.into_iter().next()?;

        debug!(
            "Selected processor: {} (capability: {:?})",
            best_key, best_capability
        );

        self.stats.increment_capability_selections();
        Some((best_key, best_processor))
    }

    /// Find a specific processor by key
    ///
    /// This method allows direct lookup of processors by their key,
    /// useful for nested processing where a specific processor is requested.
    pub fn find_processor(&self, key: &ProcessorKey) -> Option<(ProcessorKey, SharedProcessor)> {
        self.processors
            .get(key)
            .map(|processor| (key.clone(), processor.clone()))
    }

    /// Process data using the best available processor
    ///
    /// This is a convenience method that combines processor selection
    /// and processing in a single call.
    pub fn process_data(&self, data: &[u8], context: &ProcessorContext) -> Result<ProcessorResult> {
        if let Some((key, processor)) = self.find_best_processor(context) {
            debug!("Processing data with processor: {}", key);
            processor.process_data(data, context)
        } else {
            // Try fallback chain
            for fallback_key in &self.fallback_chain {
                if let Some((_, processor)) = self.find_processor(fallback_key) {
                    debug!("Using fallback processor: {}", fallback_key);
                    return processor.process_data(data, context);
                }
            }

            warn!("No processor available for context: {}", context.table_name);
            Ok(ProcessorResult::new())
        }
    }

    /// Get all registered processors
    ///
    /// Returns a list of all processor keys and their metadata for
    /// debugging and introspection.
    pub fn list_processors(&self) -> Vec<(ProcessorKey, ProcessorMetadata)> {
        self.processors
            .iter()
            .map(|(key, processor)| (key.clone(), processor.get_metadata()))
            .collect()
    }

    /// Get processors that can handle a specific context
    ///
    /// Returns all compatible processors with their capability assessments
    /// for analysis and debugging.
    pub fn get_compatible_processors(
        &self,
        context: &ProcessorContext,
    ) -> Vec<(ProcessorKey, ProcessorCapability)> {
        self.processors
            .iter()
            .filter_map(|(key, processor)| {
                let capability = processor.can_process(context);
                if capability.is_compatible() {
                    Some((key.clone(), capability))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get registry statistics
    pub fn get_stats(&self) -> &RegistryStats {
        &self.stats
    }

    /// Reset registry statistics
    pub fn reset_stats(&mut self) {
        self.stats = RegistryStats::new();
    }

    /// Get the number of registered processors
    ///
    /// This method returns the total count of registered processors,
    /// useful for debugging and initialization verification.
    pub fn processor_count(&self) -> usize {
        self.processors.len()
    }

    /// Setup standard processors that are always available
    ///
    /// This method registers the core processors that handle basic
    /// EXIF processing and provide fallback functionality.
    pub fn register_standard_processors(&mut self) {
        // Register core EXIF processor
        self.register_processor(
            ProcessorKey::new("EXIF".to_string(), "Main".to_string()),
            StandardExifProcessor,
        );

        // Register binary data processor
        self.register_processor(
            ProcessorKey::new("EXIF".to_string(), "BinaryData".to_string()),
            StandardBinaryDataProcessor,
        );

        // Set up basic fallback chain
        self.set_fallback_chain(vec![
            ProcessorKey::new("EXIF".to_string(), "BinaryData".to_string()),
            ProcessorKey::new("EXIF".to_string(), "Main".to_string()),
        ]);
    }
}

impl Default for ProcessorRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.register_standard_processors();
        registry
    }
}

/// Registry statistics for monitoring and debugging
#[derive(Debug, Clone)]
pub struct RegistryStats {
    /// Number of processors registered
    pub processors_registered: usize,

    /// Number of processor selections performed
    pub selections_performed: usize,

    /// Number of selections made by dispatch rules
    pub rule_selections: usize,

    /// Number of selections made by capability assessment
    pub capability_selections: usize,

    /// Number of fallback processor uses
    pub fallback_uses: usize,
}

impl RegistryStats {
    fn new() -> Self {
        Self {
            processors_registered: 0,
            selections_performed: 0,
            rule_selections: 0,
            capability_selections: 0,
            fallback_uses: 0,
        }
    }

    fn increment_selections(&self) {
        // Note: In a real implementation, we'd use atomic counters
        // For now, this is a placeholder for the statistics interface
    }

    fn increment_rule_selections(&self) {
        // Note: In a real implementation, we'd use atomic counters
    }

    fn increment_capability_selections(&self) {
        // Note: In a real implementation, we'd use atomic counters
    }
}

/// Standard EXIF processor for basic IFD processing
///
/// This processor handles standard EXIF IFD structures and serves as
/// the fallback when no manufacturer-specific processor is available.
struct StandardExifProcessor;

impl BinaryDataProcessor for StandardExifProcessor {
    fn can_process(&self, _context: &ProcessorContext) -> ProcessorCapability {
        // Can always process as fallback
        ProcessorCapability::Fallback
    }

    fn process_data(&self, _data: &[u8], _context: &ProcessorContext) -> Result<ProcessorResult> {
        // Temporary fix: return error to force fallback to existing processing
        debug!("StandardExifProcessor: Returning error to force fallback");
        Err(crate::types::ExifError::ParseError(
            "Forcing fallback to existing processing".to_string(),
        ))
    }

    fn get_metadata(&self) -> ProcessorMetadata {
        ProcessorMetadata::new(
            "Standard EXIF Processor".to_string(),
            "Handles standard EXIF IFD structures".to_string(),
        )
    }
}

/// Standard binary data processor for ProcessBinaryData tables
///
/// This processor handles ExifTool's ProcessBinaryData function equivalent
/// and serves as a fallback for binary data processing.
struct StandardBinaryDataProcessor;

impl BinaryDataProcessor for StandardBinaryDataProcessor {
    fn can_process(&self, context: &ProcessorContext) -> ProcessorCapability {
        // Good for tables that use ProcessBinaryData
        if context.table_name.contains("BinaryData") || context.parameters.contains_key("format") {
            ProcessorCapability::Good
        } else {
            ProcessorCapability::Fallback
        }
    }

    fn process_data(&self, _data: &[u8], _context: &ProcessorContext) -> Result<ProcessorResult> {
        // Temporary fix: return error to force fallback to existing processing
        debug!("StandardBinaryDataProcessor: Returning error to force fallback");
        Err(crate::types::ExifError::ParseError(
            "Forcing fallback to existing processing".to_string(),
        ))
    }

    fn get_metadata(&self) -> ProcessorMetadata {
        ProcessorMetadata::new(
            "Standard Binary Data Processor".to_string(),
            "Handles ProcessBinaryData table processing".to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::FileFormat;

    struct TestProcessor {
        name: String,
        capability: ProcessorCapability,
    }

    impl BinaryDataProcessor for TestProcessor {
        fn can_process(&self, _context: &ProcessorContext) -> ProcessorCapability {
            self.capability.clone()
        }

        fn process_data(
            &self,
            _data: &[u8],
            _context: &ProcessorContext,
        ) -> Result<ProcessorResult> {
            let mut result = ProcessorResult::new();
            result.add_tag(
                "TestTag".to_string(),
                crate::types::TagValue::String(self.name.clone()),
            );
            Ok(result)
        }

        fn get_metadata(&self) -> super::ProcessorMetadata {
            super::ProcessorMetadata::new(self.name.clone(), "Test processor".to_string())
        }
    }

    #[test]
    fn test_processor_registration() {
        let mut registry = ProcessorRegistry::new();
        let key = ProcessorKey::new("Test".to_string(), "Processor".to_string());

        registry.register_processor(
            key.clone(),
            TestProcessor {
                name: "Test".to_string(),
                capability: ProcessorCapability::Good,
            },
        );

        assert!(registry.find_processor(&key).is_some());
    }

    #[test]
    fn test_processor_selection() {
        let mut registry = ProcessorRegistry::new();

        // Register processors with different capabilities
        registry.register_processor(
            ProcessorKey::new("Test".to_string(), "Perfect".to_string()),
            TestProcessor {
                name: "Perfect".to_string(),
                capability: ProcessorCapability::Perfect,
            },
        );

        registry.register_processor(
            ProcessorKey::new("Test".to_string(), "Good".to_string()),
            TestProcessor {
                name: "Good".to_string(),
                capability: ProcessorCapability::Good,
            },
        );

        let context = ProcessorContext::new(FileFormat::Jpeg, "Test::Table".to_string());
        let (selected_key, _) = registry.find_best_processor(&context).unwrap();

        // Should select the Perfect capability processor
        assert_eq!(selected_key.processor_name, "Perfect");
    }

    #[test]
    fn test_compatible_processors() {
        let mut registry = ProcessorRegistry::new();

        registry.register_processor(
            ProcessorKey::new("Test".to_string(), "Compatible".to_string()),
            TestProcessor {
                name: "Compatible".to_string(),
                capability: ProcessorCapability::Good,
            },
        );

        registry.register_processor(
            ProcessorKey::new("Test".to_string(), "Incompatible".to_string()),
            TestProcessor {
                name: "Incompatible".to_string(),
                capability: ProcessorCapability::Incompatible,
            },
        );

        let context = ProcessorContext::new(FileFormat::Jpeg, "Test::Table".to_string());
        let compatible = registry.get_compatible_processors(&context);

        assert_eq!(compatible.len(), 1);
        assert_eq!(compatible[0].0.processor_name, "Compatible");
    }

    #[test]
    fn test_fallback_chain() {
        let mut registry = ProcessorRegistry::new();
        let fallback_key = ProcessorKey::new("Test".to_string(), "Fallback".to_string());

        registry.register_processor(
            fallback_key.clone(),
            TestProcessor {
                name: "Fallback".to_string(),
                capability: ProcessorCapability::Fallback,
            },
        );

        registry.set_fallback_chain(vec![fallback_key.clone()]);

        let context = ProcessorContext::new(FileFormat::Jpeg, "Unknown::Table".to_string());

        // Even if we register an incompatible processor
        registry.register_processor(
            ProcessorKey::new("Test".to_string(), "Incompatible".to_string()),
            TestProcessor {
                name: "Incompatible".to_string(),
                capability: ProcessorCapability::Incompatible,
            },
        );

        let result = registry.process_data(&[], &context).unwrap();
        // Should have used fallback and extracted a tag
        assert!(!result.extracted_tags.is_empty());
    }
}
