//! Mathematical functions for Perl-to-Rust code generation
//!
//! This module provides Rust implementations of Perl mathematical functions
//! that are commonly used in ExifTool expressions.

pub mod basic;
pub mod camera;
pub mod safe;

// Re-export all functions for convenience
pub use basic::{abs, atan2, cos, exp, int, log, negate, sin, sqrt, IsFloat};
pub use camera::{
    aperture_from_nikon_lens, exposure_compensation, iso_from_exposure_value, pow2, power,
    round_to_places, shutter_speed_from_exposure,
};
pub use safe::{safe_division, safe_reciprocal};
