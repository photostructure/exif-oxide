//! File types determined by process proc during FastScan == 3
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashSet;
use std::sync::LazyLock;

// Generated process determined types boolean set
// Source: ExifTool Image::ExifTool %processType
// Description: File types determined by process proc during FastScan == 3

/// Static data for file types determined by process proc during fastscan == 3 set (10 entries)
static PROCESS_DETERMINED_TYPES_DATA: &[&str] = &[
    "AIFF", "EXE", "Font", "JPEG", "PS", "Real", "TIFF", "TXT", "VCard", "XMP",
];

/// File types determined by process proc during FastScan == 3 boolean set table
/// Built from static data on first access
pub static PROCESS_DETERMINED_TYPES: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| PROCESS_DETERMINED_TYPES_DATA.iter().copied().collect());

/// Check if a file type is in the file types determined by process proc during fastscan == 3 set
pub fn is_process_determined(file_type: &str) -> bool {
    PROCESS_DETERMINED_TYPES.contains(file_type)
}
