//! Compatibility aliases for generated modules - DO NOT GENERATE
//!
//! This file provides lowercase aliases for the PascalCase module names
//! to maintain compatibility with existing code that expects the old names.
//!
//! This is a temporary solution until all code is updated to use the new
//! module names directly.

// Module aliases to support old import paths
pub use super::Canon_pm as canon;
pub use super::ExifTool_pm as exiftool;
pub use super::Exif_pm as exif_pm; // Avoid conflict with crate::exif module
pub use super::Nikon_pm as nikon;
pub use super::XMP_pm as xmp_pm; // Avoid conflict with crate::xmp module

// Re-export the generated file_types module
pub use super::file_types;
