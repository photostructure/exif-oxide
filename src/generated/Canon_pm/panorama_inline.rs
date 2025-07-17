//! Inline PrintConv tables for Panorama table
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Canon.pm (table: Panorama)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (5 entries)
static PANORAMA_PANORAMA_DIRECTION_DATA: &[(u8, &'static str)] = &[
    (0, "Left to Right"),
    (1, "Right to Left"),
    (2, "Bottom to Top"),
    (3, "Top to Bottom"),
    (4, "2x2 Matrix (Clockwise)"),
];

/// Lookup table (lazy-initialized)
pub static PANORAMA_PANORAMA_DIRECTION: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| PANORAMA_PANORAMA_DIRECTION_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_panorama__panorama_direction(key: u8) -> Option<&'static str> {
    PANORAMA_PANORAMA_DIRECTION.get(&key).copied()
}
