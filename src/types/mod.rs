//! Core type definitions for exif-oxide
//!
//! This module provides a unified interface to all type definitions
//! used throughout the library.

mod binary_data;
mod errors;
mod metadata;
mod processors;
mod values;

// Re-export everything for backwards compatibility
pub use binary_data::*;
pub use errors::*;
pub use metadata::*;
pub use processors::*;
pub use values::*;
