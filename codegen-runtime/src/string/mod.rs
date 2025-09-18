//! String function implementations for Rust code generation
//!
//! This module provides Rust implementations of Perl string functions commonly used
//! in ExifTool expressions. All functions follow Perl's exact behavior to
//! maintain compatibility with ExifTool's logic.

pub mod extraction;
pub mod format;
pub mod measurement;
pub mod transform;

// Re-export all functions for convenience
pub use extraction::{index_2arg, index_3arg, substr_2arg, substr_3arg};
pub use format::{
    concat, default_if_empty, format_tag, is_defined, repeat_string, stringify, tag_string,
};
pub use measurement::{length_i32, length_string};
pub use transform::{chr, regex_replace, regex_substitute_perl, uc};
