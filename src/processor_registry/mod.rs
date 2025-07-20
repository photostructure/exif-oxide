//! Enhanced Processor Architecture for exif-oxide
//!
//! This module implements the sophisticated processor dispatch system designed to handle
//! ExifTool's 121+ ProcessBinaryData variants with trait-based architecture.
//!
//! ## Architecture Overview
//!
//! - **BinaryDataProcessor trait**: Core abstraction for all processors
//! - **ProcessorRegistry**: Central registry with capability-based selection
//! - **ProcessorContext**: Rich context passing with metadata
//! - **DispatchRule trait**: Sophisticated conditional dispatch logic
//! - **ProcessorCapability**: Assessment system (Perfect/Good/Fallback/Incompatible)
//!
//! ## Migration Strategy
//!
//! This system is built alongside the existing enum-based processor system to avoid
//! breaking changes. The migration follows these phases:
//!
//! 1. **Phase 1**: Build trait system (this module)
//! 2. **Phase 2**: Create compatibility bridge
//! 3. **Phase 3**: Convert existing processors
//! 4. **Phase 4**: Integration testing
//! 5. **Phase 5**: Remove deprecated enum system
//!
//! ## ExifTool Reference
//!
//! Based on ExifTool's PROCESS_PROC dispatch system and conditional SubDirectory
//! processing patterns found throughout Canon.pm, Nikon.pm, and other manufacturer modules.

pub mod capability;
// Note: conditions module moved to src/expressions/
pub mod context;
pub mod dispatch;
pub mod processors;
pub mod registry;
pub mod traits;

// Re-export core types for convenience
pub use crate::expressions::ExpressionEvaluator as ConditionEvaluator;
pub use capability::ProcessorCapability;
pub use context::ProcessorContext;
pub use dispatch::DispatchRule;
pub use processors::*;
pub use registry::ProcessorRegistry;
pub use traits::{
    BinaryDataProcessor, ProcessorKey, ProcessorMetadata, ProcessorResult, SharedProcessor,
};

// Import dispatch rules for the global registry
use dispatch::*;

// Global processor registry instance
use std::sync::LazyLock;

/// Global processor registry - initialized with all available processors
/// This provides the main entry point for processor dispatch throughout the application
static PROCESSOR_REGISTRY: LazyLock<ProcessorRegistry> = LazyLock::new(|| {
    let mut registry = ProcessorRegistry::new();

    // Register standard processors
    registry.register_standard_processors();

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
        ProcessorKey::new("Nikon".to_string(), "EncryptedData".to_string()),
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

    // Register Olympus processors
    registry.register_processor(
        ProcessorKey::new("Olympus".to_string(), "Equipment".to_string()),
        OlympusEquipmentProcessor,
    );

    registry.register_processor(
        ProcessorKey::new("Olympus".to_string(), "CameraSettings".to_string()),
        OlympusCameraSettingsProcessor,
    );

    registry.register_processor(
        ProcessorKey::new("Olympus".to_string(), "FocusInfo".to_string()),
        OlympusFocusInfoProcessor,
    );

    // Register FujiFilm processors (demonstrates generated ProcessBinaryData table integration)
    registry.register_processor(
        ProcessorKey::new("FUJIFILM".to_string(), "FFMV".to_string()),
        FujiFilmFFMVProcessor::new(),
    );

    // Add dispatch rules for sophisticated processor selection
    registry.add_dispatch_rule(CanonDispatchRule);
    registry.add_dispatch_rule(NikonDispatchRule);
    registry.add_dispatch_rule(OlympusDispatchRule);
    registry.add_dispatch_rule(FormatDispatchRule);
    registry.add_dispatch_rule(TableDispatchRule);

    registry
});

/// Get access to the global processor registry
/// This is the main function that ExifReader and other components use to access processors
pub fn get_global_registry() -> &'static ProcessorRegistry {
    &PROCESSOR_REGISTRY
}
