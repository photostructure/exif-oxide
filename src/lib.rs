//! exif-oxide: High-performance Rust implementation of ExifTool
//!
//! This library provides metadata extraction from image files with ExifTool compatibility.
//! The architecture uses runtime registries for PrintConv/ValueConv implementations to avoid
//! code generation bloat while maintaining flexible extensibility.

pub mod formats;
pub mod generated;
pub mod registry;
pub mod types;

pub use generated::*;
pub use registry::Registry;
pub use types::{ExifData, ExifError, TagValue};
