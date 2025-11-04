//! Core type definitions for exif-oxide
//!
//! This module provides a unified interface to all type definitions
//! used throughout the library.

// Re-export core types from codegen-runtime for API compatibility
pub use codegen_runtime::{ExifContext, TagValue};

pub mod binary_data;
mod context;
mod errors;
mod metadata;
mod tag_info;

// Re-export everything for backwards compatibility
pub use binary_data::*;
pub use context::*;
pub use errors::*;
pub use metadata::*;
pub use tag_info::*;
