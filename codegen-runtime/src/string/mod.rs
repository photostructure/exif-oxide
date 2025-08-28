//! String function implementations for Rust code generation
//!
//! This module provides Rust implementations of Perl string functions commonly used
//! in ExifTool expressions. All functions follow Perl's exact behavior to
//! maintain compatibility with ExifTool's logic.

pub mod measurement;
pub mod transform;

// Re-export all functions for convenience
pub use measurement::{length_i32, length_string};
pub use transform::{regex_replace, regex_substitute_perl};