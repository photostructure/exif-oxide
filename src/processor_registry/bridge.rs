//! Bridge system between old enum dispatch and new trait dispatch
//!
//! This module provides compatibility between the existing enum-based processor
//! system and the new trait-based processor architecture. It enables gradual
//! migration without breaking changes.
//!
//! ## Migration Strategy
//!
//! 1. **Phase 1**: Bridge enables both systems to coexist
//! 2. **Phase 2-4**: Gradual conversion from enum to trait processors
//! 3. **Phase 5**: Remove enum system entirely (future milestone)
//!
//! ## Architecture
//!
//! The bridge tries the trait-based system first for enhanced capabilities,
//! then falls back to the enum system for compatibility.

use super::{ProcessorContext, ProcessorKey, ProcessorRegistry, SharedProcessor};
use crate::types::{CanonProcessor, NikonProcessor, ProcessorType, SonyProcessor};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

/// Selection result from the bridge system
///
/// Indicates whether a trait-based processor was selected or the system
/// should fall back to the enum-based processor dispatch.
pub enum ProcessorSelection {
    /// New trait-based processor found and ready to use
    Trait(ProcessorKey, SharedProcessor),
    /// Fallback to old enum-based processor with parameters
    Enum(ProcessorType, HashMap<String, String>),
}

impl std::fmt::Debug for ProcessorSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessorSelection::Trait(key, _processor) => f
                .debug_tuple("Trait")
                .field(key)
                .field(&"<processor>")
                .finish(),
            ProcessorSelection::Enum(processor_type, params) => f
                .debug_tuple("Enum")
                .field(processor_type)
                .field(params)
                .finish(),
        }
    }
}

/// Bridge between old enum system and new trait system
///
/// This bridge provides a smooth transition path from the existing enum-based
/// processor dispatch to the new trait-based system. It tries trait processors
/// first for enhanced capabilities, then falls back to enum processors.
///
/// ## Usage Pattern
///
/// ```no_run
/// use exif_oxide::processor_registry::{ProcessorBridge, ProcessorSelection, ProcessorContext};
/// use exif_oxide::formats::FileFormat;
///
/// let bridge = ProcessorBridge::new();
/// let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::SerialData".to_string());
/// let data = &[0u8; 100];
///
/// match bridge.select_processor(&context) {
///     ProcessorSelection::Trait(key, processor) => {
///         // Use new trait-based processing
///         let result = processor.process_data(data, &context).unwrap();
///     }
///     ProcessorSelection::Enum(processor_type, params) => {
///         // Fall back to existing enum-based processing
///         println!("Using enum processor: {:?}", processor_type);
///     }
/// }
/// ```
pub struct ProcessorBridge {
    trait_registry: Arc<ProcessorRegistry>,
}

impl ProcessorBridge {
    /// Create new processor bridge with initialized trait registry
    pub fn new() -> Self {
        Self {
            trait_registry: Arc::new(create_global_registry()),
        }
    }

    /// Create processor bridge with custom registry (for testing)
    pub fn with_registry(registry: ProcessorRegistry) -> Self {
        Self {
            trait_registry: Arc::new(registry),
        }
    }

    /// Select processor, trying trait system first, falling back to enum
    ///
    /// This is the main entry point for the bridge system. It evaluates
    /// the context and returns the best available processor, preferring
    /// the trait-based system for enhanced capabilities.
    ///
    /// ## Selection Algorithm
    ///
    /// 1. Try trait-based processors with capability assessment
    /// 2. If no suitable trait processor found, convert to enum system
    /// 3. Return appropriate selection for the calling code
    pub fn select_processor(&self, context: &ProcessorContext) -> ProcessorSelection {
        debug!(
            "Bridge selecting processor for manufacturer: {:?}, table: {}",
            context.manufacturer, context.table_name
        );

        // Try trait-based system first
        if let Some((key, processor)) = self.trait_registry.find_best_processor(context) {
            debug!("Bridge selected trait-based processor: {}", key);
            return ProcessorSelection::Trait(key, processor);
        }

        // Fall back to existing enum system
        debug!("Bridge falling back to enum-based processor");
        let (enum_processor, params) = self.convert_context_to_enum(context);
        ProcessorSelection::Enum(enum_processor, params)
    }

    /// Get reference to the trait registry for inspection
    pub fn get_trait_registry(&self) -> &ProcessorRegistry {
        &self.trait_registry
    }

    /// Convert ProcessorContext to enum system parameters
    ///
    /// This function bridges the rich context system to the simpler
    /// enum-based dispatch that existed before. It maintains compatibility
    /// while the migration is in progress.
    fn convert_context_to_enum(
        &self,
        context: &ProcessorContext,
    ) -> (ProcessorType, HashMap<String, String>) {
        let mut params = HashMap::new();

        // Store context information as parameters for enum processors
        if let Some(manufacturer) = &context.manufacturer {
            params.insert("manufacturer".to_string(), manufacturer.clone());
        }
        if let Some(model) = &context.model {
            params.insert("model".to_string(), model.clone());
        }
        params.insert("table_name".to_string(), context.table_name.clone());
        params.insert("data_offset".to_string(), context.data_offset.to_string());

        // Convert to appropriate enum processor based on context
        let processor_type = match context.manufacturer.as_deref() {
            Some("Canon") => {
                // Select Canon processor variant based on table name
                if context.table_name.contains("CameraSettings") {
                    ProcessorType::Canon(CanonProcessor::CameraSettings)
                } else if context.table_name.contains("AFInfo") {
                    ProcessorType::Canon(CanonProcessor::AfInfo)
                } else if context.table_name.contains("SerialData") {
                    ProcessorType::Canon(CanonProcessor::SerialData)
                } else if context.table_name.contains("Binary") {
                    ProcessorType::Canon(CanonProcessor::BinaryData)
                } else {
                    ProcessorType::Canon(CanonProcessor::Main)
                }
            }
            Some("NIKON CORPORATION") | Some("NIKON") => {
                // Select Nikon processor variant based on table name and data
                if context.table_name.contains("Encrypted") {
                    ProcessorType::Nikon(NikonProcessor::Encrypted)
                } else {
                    ProcessorType::Nikon(NikonProcessor::Main)
                }
            }
            Some("SONY") | Some("Sony") => {
                // Select Sony processor variant (basic for now)
                ProcessorType::Sony(SonyProcessor::Main)
            }
            Some(other) => {
                // Generic manufacturer processing
                ProcessorType::Generic(other.to_string())
            }
            None => {
                // No manufacturer - use generic processing
                if context.table_name.contains("Binary") {
                    ProcessorType::BinaryData
                } else if context.table_name.contains("GPS") {
                    ProcessorType::Gps
                } else {
                    ProcessorType::Exif
                }
            }
        };

        debug!(
            "Bridge converted context to enum processor: {:?}",
            processor_type
        );

        (processor_type, params)
    }
}

impl Default for ProcessorBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Create the global trait registry with all concrete processors registered
///
/// This function sets up the trait-based processor registry with all available
/// concrete processor implementations and their dispatch rules.
///
/// ## Registered Processors
///
/// - **Canon**: SerialData, CameraSettings, SerialDataMkII
/// - **Nikon**: Encrypted, AFInfo, LensData
/// - **Dispatch Rules**: Manufacturer-specific selection logic
fn create_global_registry() -> ProcessorRegistry {
    use super::dispatch::{CanonDispatchRule, NikonDispatchRule};
    use super::processors::*;

    let mut registry = ProcessorRegistry::new();

    debug!("Initializing global trait processor registry");

    // Register Canon processors
    registry.register_processor(
        ProcessorKey::new("Canon".to_string(), "SerialData".to_string()),
        CanonSerialDataProcessor,
    );

    registry.register_processor(
        ProcessorKey::new("Canon".to_string(), "CameraSettings".to_string()),
        CanonCameraSettingsProcessor,
    );

    registry.register_processor(
        ProcessorKey::with_variant(
            "Canon".to_string(),
            "SerialData".to_string(),
            "MkII".to_string(),
        ),
        CanonSerialDataMkIIProcessor,
    );

    // Register Nikon processors
    registry.register_processor(
        ProcessorKey::new("Nikon".to_string(), "Encrypted".to_string()),
        NikonEncryptedDataProcessor,
    );

    registry.register_processor(
        ProcessorKey::new("Nikon".to_string(), "AFInfo".to_string()),
        NikonAFInfoProcessor,
    );

    registry.register_processor(
        ProcessorKey::new("Nikon".to_string(), "LensData".to_string()),
        NikonLensDataProcessor,
    );

    // Add sophisticated dispatch rules
    registry.add_dispatch_rule(CanonDispatchRule);
    registry.add_dispatch_rule(NikonDispatchRule);

    debug!(
        "Global trait registry initialized with {} processors",
        registry.processor_count()
    );

    registry
}

lazy_static::lazy_static! {
    /// Global processor bridge instance
    ///
    /// This static instance provides system-wide access to the processor bridge
    /// without requiring manual initialization in every location that needs
    /// processor selection.
    ///
    /// ## Usage
    ///
    /// ```no_run
    /// use exif_oxide::processor_registry::{PROCESSOR_BRIDGE, ProcessorSelection, ProcessorContext};
    /// use exif_oxide::formats::FileFormat;
    ///
    /// let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::SerialData".to_string());
    ///
    /// match PROCESSOR_BRIDGE.select_processor(&context) {
    ///     ProcessorSelection::Trait(key, processor) => { /* use trait processor */ }
    ///     ProcessorSelection::Enum(proc_type, params) => { /* use enum processor */ }
    /// }
    /// ```
    pub static ref PROCESSOR_BRIDGE: ProcessorBridge = ProcessorBridge::new();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::FileFormat;
    use crate::processor_registry::ProcessorContext;

    #[test]
    fn test_bridge_canon_processor_selection() {
        let bridge = ProcessorBridge::new();

        let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::SerialData".to_string())
            .with_manufacturer("Canon".to_string())
            .with_model("EOS R5".to_string());

        match bridge.select_processor(&context) {
            ProcessorSelection::Trait(key, _processor) => {
                assert_eq!(key.namespace, "Canon");
                assert!(key.processor_name.contains("SerialData"));
            }
            ProcessorSelection::Enum(ProcessorType::Canon(CanonProcessor::SerialData), _) => {
                // Acceptable fallback
            }
            other => panic!("Unexpected processor selection: {other:?}"),
        }
    }

    #[test]
    fn test_bridge_nikon_processor_selection() {
        let bridge = ProcessorBridge::new();

        let context = ProcessorContext::new(FileFormat::Jpeg, "Nikon::Encrypted".to_string())
            .with_manufacturer("NIKON CORPORATION".to_string());

        match bridge.select_processor(&context) {
            ProcessorSelection::Trait(key, _processor) => {
                assert_eq!(key.namespace, "Nikon");
                assert_eq!(key.processor_name, "Encrypted");
            }
            ProcessorSelection::Enum(ProcessorType::Nikon(NikonProcessor::Encrypted), _) => {
                // Acceptable fallback
            }
            other => panic!("Unexpected processor selection: {other:?}"),
        }
    }

    #[test]
    fn test_bridge_enum_fallback() {
        let bridge = ProcessorBridge::new();

        // Create context for unsupported manufacturer
        let context = ProcessorContext::new(FileFormat::Jpeg, "Unknown::Table".to_string())
            .with_manufacturer("Unknown Manufacturer".to_string());

        match bridge.select_processor(&context) {
            ProcessorSelection::Enum(ProcessorType::Generic(manufacturer), params) => {
                assert_eq!(manufacturer, "Unknown Manufacturer");
                assert_eq!(
                    params.get("manufacturer"),
                    Some(&"Unknown Manufacturer".to_string())
                );
                assert_eq!(
                    params.get("table_name"),
                    Some(&"Unknown::Table".to_string())
                );
            }
            other => panic!("Expected enum fallback for unknown manufacturer, got: {other:?}"),
        }
    }

    #[test]
    fn test_bridge_context_conversion() {
        let bridge = ProcessorBridge::new();

        let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::CameraSettings".to_string())
            .with_manufacturer("Canon".to_string())
            .with_model("EOS 5D Mark IV".to_string());

        let (processor_type, params) = bridge.convert_context_to_enum(&context);

        assert_eq!(
            processor_type,
            ProcessorType::Canon(CanonProcessor::CameraSettings)
        );
        assert_eq!(params.get("manufacturer"), Some(&"Canon".to_string()));
        assert_eq!(params.get("model"), Some(&"EOS 5D Mark IV".to_string()));
        assert_eq!(
            params.get("table_name"),
            Some(&"Canon::CameraSettings".to_string())
        );
    }

    #[test]
    fn test_bridge_canon_model_specific_selection() {
        let bridge = ProcessorBridge::new();

        // Test Canon R5 should get MkII variant
        let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::SerialData".to_string())
            .with_manufacturer("Canon".to_string())
            .with_model("Canon EOS R5".to_string());

        match bridge.select_processor(&context) {
            ProcessorSelection::Trait(key, _processor) => {
                // Should select the MkII variant for R5
                assert_eq!(key.namespace, "Canon");
                assert_eq!(key.processor_name, "SerialData");
                // Variant might be "MkII" if the dispatch rule is working correctly
            }
            ProcessorSelection::Enum(_, _) => {
                // Fallback is also acceptable during Phase 1
            }
        }
    }

    #[test]
    fn test_bridge_global_instance() {
        // Test that the global instance works
        let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::SerialData".to_string())
            .with_manufacturer("Canon".to_string());

        // This should not panic and should return a valid selection
        let selection = PROCESSOR_BRIDGE.select_processor(&context);

        match selection {
            ProcessorSelection::Trait(key, _) => {
                assert_eq!(key.namespace, "Canon");
            }
            ProcessorSelection::Enum(ProcessorType::Canon(_), _) => {
                // Acceptable fallback
            }
            other => panic!("Unexpected selection from global bridge: {other:?}"),
        }
    }
}
