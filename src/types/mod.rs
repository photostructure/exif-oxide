//! Core type definitions for exif-oxide
//!
//! This module provides a unified interface to all type definitions
//! used throughout the library.

pub mod binary_data;
mod context;
mod errors;
mod metadata;
mod tag_info;
mod values;

// Re-export everything for backwards compatibility
pub use binary_data::*;
pub use context::*;
pub use errors::*;
pub use metadata::*;
pub use tag_info::*;
pub use values::*;
