//! PNG chunks that shouldn't be moved across during editing
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/PNG.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashSet;
use std::sync::LazyLock;

// Generated png no leapfrog chunks boolean set
// Source: ExifTool Image::ExifTool::PNG %noLeapFrog
// Description: PNG chunks that shouldn't be moved across during editing

/// Static data for png chunks that shouldn't be moved across during editing set (12 entries)
static PNG_NO_LEAPFROG_CHUNKS_DATA: &[&str] = &[
    "BASI", "CLON", "DHDR", "IEND", "IHDR", "JHDR", "MAGN", "MEND", "PAST", "SAVE", "SEEK", "SHOW",
];

/// PNG chunks that shouldn't be moved across during editing boolean set table
/// Built from static data on first access
pub static PNG_NO_LEAPFROG_CHUNKS: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| PNG_NO_LEAPFROG_CHUNKS_DATA.iter().copied().collect());

/// Check if a file type is in the png chunks that shouldn't be moved across during editing set
pub fn is_png_no_leapfrog_chunks(file_type: &str) -> bool {
    PNG_NO_LEAPFROG_CHUNKS.contains(file_type)
}
