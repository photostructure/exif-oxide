//! File types that can be created from scratch (XMP, ICC, MIE, VRD, etc.)
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashSet;
use std::sync::LazyLock;

// Generated creatable file types boolean set
// Source: ExifTool Image::ExifTool %createTypes
// Description: File types that can be created from scratch (XMP, ICC, MIE, VRD, etc.)

/// Static data for file types that can be created from scratch (xmp, icc, mie, vrd, etc.) set (7 entries)
static CREATABLE_FILE_TYPES_DATA: &[&str] = &["DR4", "EXIF", "EXV", "ICC", "MIE", "VRD", "XMP"];

/// File types that can be created from scratch (XMP, ICC, MIE, VRD, etc.) boolean set table
/// Built from static data on first access
pub static CREATABLE_FILE_TYPES: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| CREATABLE_FILE_TYPES_DATA.iter().copied().collect());

/// Check if a file type is in the file types that can be created from scratch (xmp, icc, mie, vrd, etc.) set
pub fn is_creatable_file(file_type: &str) -> bool {
    CREATABLE_FILE_TYPES.contains(file_type)
}
