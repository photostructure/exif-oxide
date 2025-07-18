//! PNG chunks containing text metadata (tEXt, zTXt, iTXt, eXIf)
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/PNG.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashSet;
use std::sync::LazyLock;

// Generated png text chunks boolean set
// Source: ExifTool Image::ExifTool::PNG %isTxtChunk
// Description: PNG chunks containing text metadata (tEXt, zTXt, iTXt, eXIf)

/// Static data for png chunks containing text metadata (text, ztxt, itxt, exif) set (4 entries)
static PNG_TEXT_CHUNKS_DATA: &[&str] = &["eXIf", "iTXt", "tEXt", "zTXt"];

/// PNG chunks containing text metadata (tEXt, zTXt, iTXt, eXIf) boolean set table
/// Built from static data on first access
pub static PNG_TEXT_CHUNKS: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| PNG_TEXT_CHUNKS_DATA.iter().copied().collect());

/// Check if a file type is in the png chunks containing text metadata (text, ztxt, itxt, exif) set
pub fn is_png_text_chunks(file_type: &str) -> bool {
    PNG_TEXT_CHUNKS.contains(file_type)
}
