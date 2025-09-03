//! Mathematical functions for Perl-to-Rust code generation
//!
//! This module provides Rust implementations of Perl mathematical functions
//! that are commonly used in ExifTool expressions.

pub mod basic;
pub mod safe;

// Re-export all functions for convenience
pub use basic::{abs, atan2, cos, exp, int, log, sin, sqrt};
pub use safe::{safe_division, safe_reciprocal};
