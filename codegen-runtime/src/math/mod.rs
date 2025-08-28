//! Mathematical functions for Perl-to-Rust code generation
//!
//! This module provides Rust implementations of Perl mathematical functions
//! that are commonly used in ExifTool expressions.

pub mod basic;
pub mod safe;

// Re-export all functions for convenience
pub use basic::{exp, int, log};
pub use safe::{safe_division, safe_reciprocal};