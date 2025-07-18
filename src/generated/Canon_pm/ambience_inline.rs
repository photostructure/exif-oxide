//! Inline PrintConv tables for Ambience table
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Canon.pm (table: Ambience)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (9 entries)
static AMBIENCE_AMBIENCE_SELECTION_DATA: &[(u8, &'static str)] = &[
    (0, "Standard"),
    (1, "Vivid"),
    (2, "Warm"),
    (3, "Soft"),
    (4, "Cool"),
    (5, "Intense"),
    (6, "Brighter"),
    (7, "Darker"),
    (8, "Monochrome"),
];

/// Lookup table (lazy-initialized)
pub static AMBIENCE_AMBIENCE_SELECTION: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| AMBIENCE_AMBIENCE_SELECTION_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_ambience__ambience_selection(key: u8) -> Option<&'static str> {
    AMBIENCE_AMBIENCE_SELECTION.get(&key).copied()
}
