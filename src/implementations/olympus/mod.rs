//! Olympus-specific implementations for exif-oxide
//!
//! This module provides Olympus-specific functionality including:
//! - MakerNotes signature detection and offset calculation
//! - Equipment tag definitions and lookups
//!
//! ## ExifTool Reference
//!
//! Based on ExifTool's Olympus.pm module which handles the complex
//! Olympus MakerNotes format with manufacturer signature headers.

mod equipment_tags;
pub use equipment_tags::get_equipment_tag_name;

use crate::types::TagValue;
use std::collections::HashMap;

/// Check if a Make string indicates this is an Olympus camera
/// ExifTool: lib/Image/ExifTool/Olympus.pm line 1089
pub fn is_olympus_makernote(make: &str) -> bool {
    make.to_uppercase().contains("OLYMPUS")
}

/// Get Olympus MakerNotes signature info if present
/// ExifTool: lib/Image/ExifTool/Olympus.pm lines 1157-1168
///
/// Returns Some((signature_string, header_length)) if a signature is found
pub fn get_olympus_signature(data: &[u8]) -> Option<(&str, usize)> {
    // Check for "OLYMPUS\0II\x03\0" signature (12 bytes)
    if data.len() >= 12 
        && data.starts_with(b"OLYMPUS\0")
        && &data[8..10] == b"II" 
        && data[10] == 0x03 
        && data[11] == 0x00 {
        return Some(("OLYMPUS", 12));
    }
    
    // Check for "OLYMP\0" signature (8 bytes)
    if data.len() >= 8 && data.starts_with(b"OLYMP\0") {
        return Some(("OLYMP", 8));
    }
    
    None
}

/// Calculate proper offset for Olympus MakerNotes subdirectories
/// ExifTool: lib/Image/ExifTool/Olympus.pm - subdirectory offset handling
///
/// When Olympus MakerNotes have a signature header, subdirectory offsets
/// are relative to the original MakerNotes position, not the adjusted position
pub fn calculate_olympus_subdirectory_offset(
    subdirectory_value: u32,
    maker_notes_offset: usize,
    _signature_length: usize,
) -> usize {
    // Subdirectory offsets are relative to the original MakerNotes position
    maker_notes_offset + subdirectory_value as usize
}