//! File types with weak magic number recognition (MP3)
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashSet;
use std::sync::LazyLock;

// Generated weak magic file types boolean set
// Source: ExifTool Image::ExifTool %weakMagic
// Description: File types with weak magic number recognition (MP3)

/// Static data for file types with weak magic number recognition (mp3) set (1 entries)
static WEAK_MAGIC_FILE_TYPES_DATA: &[&str] = &["MP3"];

/// File types with weak magic number recognition (MP3) boolean set table
/// Built from static data on first access
pub static WEAK_MAGIC_FILE_TYPES: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| WEAK_MAGIC_FILE_TYPES_DATA.iter().copied().collect());

/// Check if a file type is in the file types with weak magic number recognition (mp3) set
pub fn is_weak_magic_file(file_type: &str) -> bool {
    WEAK_MAGIC_FILE_TYPES.contains(file_type)
}
