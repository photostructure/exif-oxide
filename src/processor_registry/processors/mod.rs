//! Concrete processor implementations
//!
//! This module contains concrete implementations of the `BinaryDataProcessor` trait
//! that delegate to existing Canon and Nikon processing logic while providing the
//! enhanced capabilities and context awareness of the new trait-based system.
//!
//! ## Phase 1 Implementation Strategy
//!
//! These processors delegate to existing implementations rather than reimplementing
//! logic, following the "Trust ExifTool" principle by reusing proven code.
//!
//! ## Architecture
//!
//! - **Canon processors**: Delegate to `implementations::canon` modules
//! - **Nikon processors**: Delegate to `implementations::nikon` modules
//! - **Capability assessment**: Model-specific and context-aware evaluation
//! - **Parameter passing**: Rich context through ProcessorContext system

pub mod canon;
pub mod fujifilm;
pub mod nikon;
pub mod olympus;

// Re-export processor implementations
pub use canon::*;
pub use fujifilm::*;
pub use nikon::*;
pub use olympus::*;
