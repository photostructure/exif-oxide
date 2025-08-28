//! Runtime support for Rust code generated from ExifTool Perl expressions
//!
//! This crate provides the core types and runtime functions that generated
//! Rust code depends on. It serves as a shared foundation between the main
//! exif-oxide crate and the codegen system.
//!
//! Key components:
//! - `TagValue` - Universal value type for EXIF data
//! - `ExifContext` - Expression evaluation context
//! - `fmt` module - Runtime functions for sprintf, unpack, arithmetic, etc.
//! - `test_support` - Utilities for testing generated code

pub mod fmt;
pub mod tag_value;
pub mod types;

// Re-export core types for convenience
pub use tag_value::TagValue;
pub use types::{ExifContext, ExifError};

// Re-export fmt functions commonly used by generated code
pub use fmt::{
    join_unpack_binary, pack_c_star_bit_extract, regex_substitute_perl, safe_binary_operation,
    safe_division, safe_reciprocal, sprintf_perl, sprintf_split_values, unpack_binary,
};

// Test support module - only available with test-helpers feature
#[cfg(feature = "test-helpers")]
pub mod test_support;

#[cfg(feature = "test-helpers")]
pub use test_support::*;
