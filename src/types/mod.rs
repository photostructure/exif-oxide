//! Core type definitions for exif-oxide
//!
//! This module provides a unified interface to all type definitions
//! used throughout the library.

// Re-export core types from crate::core for API compatibility
pub use crate::core::{ExifContext, TagValue};

pub mod binary_data;
mod context;
mod errors;
mod metadata;
mod tag_info;

// Re-export everything for backwards compatibility
pub use binary_data::*;
#[allow(unused_imports)]
pub use context::*;
pub use errors::{ExifError, Result}; // ExifError comes from crate::core via errors module
pub use metadata::*;
pub use tag_info::*;
