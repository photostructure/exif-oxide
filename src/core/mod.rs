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

pub mod array_helpers;
pub mod composite_fallbacks;
pub mod data;
pub mod fmt;
pub mod math;
pub mod missing;
pub mod string;
pub mod tag_value;
pub mod types;
pub mod xmp_tag_info;

// Re-export core types for convenience
pub use tag_value::TagValue;
pub use types::{ExifContext, ExifError};
pub use xmp_tag_info::{XmpListType, XmpTagInfo};

// Re-export composite tag function types
pub use types::{CompositePrintConvFn, CompositeValueConvFn};

// Re-export array helpers for generated code
pub use array_helpers::get_array_element;

// Re-export data functions commonly used by generated code
pub use data::{join_unpack_binary, join_vec, pack_c_star_bit_extract, unpack_binary};

// Re-export fmt functions commonly used by generated code
pub use fmt::{sprintf_perl, sprintf_split_values, sprintf_with_string_concat_repeat};

// Re-export math functions commonly used by generated code
pub use math::{
    abs, atan2, cos, exp, int, log, negate, power, safe_division, safe_reciprocal, sin, sqrt,
    IsFloat,
};

// Re-export string functions commonly used by generated code
pub use string::{
    chr, concat, index_2arg, index_3arg, length_i32, length_string, regex_replace,
    regex_substitute_perl, substr_2arg, substr_3arg, uc,
};

// Re-export composite fallbacks registry for codegen and orchestration
pub use composite_fallbacks::COMPOSITE_FALLBACKS;

// Test support module - only available with test-helpers feature
#[cfg(feature = "test-helpers")]
pub mod test_support;

#[cfg(feature = "test-helpers")]
pub use test_support::*;
