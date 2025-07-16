//! PNG chunks containing image data (IDAT, JDAT, JDAA)
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/PNG.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashSet;
use std::sync::LazyLock;

// Generated png data chunks boolean set
// Source: ExifTool Image::ExifTool::PNG %isDatChunk
// Description: PNG chunks containing image data (IDAT, JDAT, JDAA)

/// Static data for png chunks containing image data (idat, jdat, jdaa) set (3 entries)
static PNG_DATA_CHUNKS_DATA: &[&str] = &["IDAT", "JDAA", "JDAT"];

/// PNG chunks containing image data (IDAT, JDAT, JDAA) boolean set table
/// Built from static data on first access
pub static PNG_DATA_CHUNKS: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| PNG_DATA_CHUNKS_DATA.iter().copied().collect());

/// Check if a file type is in the png chunks containing image data (idat, jdat, jdaa) set
pub fn is_png_data_chunks(file_type: &str) -> bool {
    PNG_DATA_CHUNKS.contains(file_type)
}
