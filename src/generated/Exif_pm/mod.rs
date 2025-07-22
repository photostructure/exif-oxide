//! Generated lookup tables from Exif.pm
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Exif.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

pub mod flash;
pub mod orientation;

// Re-export all lookup functions and constants
pub use flash::*;
pub use orientation::*;
