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

pub mod bridge;
pub mod capability;
pub mod conditions;
pub mod context;
pub mod dispatch;
pub mod processors;
pub mod registry;
pub mod traits;

// Re-export core types for convenience
pub use bridge::{ProcessorBridge, ProcessorSelection, PROCESSOR_BRIDGE};
pub use capability::ProcessorCapability;
pub use conditions::ConditionEvaluator;
pub use context::ProcessorContext;
pub use dispatch::DispatchRule;
pub use processors::*;
pub use registry::ProcessorRegistry;
pub use traits::{
    BinaryDataProcessor, ProcessorKey, ProcessorMetadata, ProcessorResult, SharedProcessor,
};
