//! Inline PrintConv tables for FocalLength table
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Canon.pm (table: FocalLength)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (2 entries)
static FOCAL_LENGTH_FOCAL_TYPE_DATA: &[(u8, &'static str)] = &[(1, "Fixed"), (2, "Zoom")];

/// Lookup table (lazy-initialized)
pub static FOCAL_LENGTH_FOCAL_TYPE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| FOCAL_LENGTH_FOCAL_TYPE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_focal_length__focal_type(key: u8) -> Option<&'static str> {
    FOCAL_LENGTH_FOCAL_TYPE.get(&key).copied()
}
