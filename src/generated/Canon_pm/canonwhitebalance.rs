//! White balance mode names
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Canon.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (22 entries)
static CANON_WHITE_BALANCE_DATA: &[(u8, &str)] = &[
    (0, "Auto"),
    (1, "Daylight"),
    (2, "Cloudy"),
    (3, "Tungsten"),
    (4, "Fluorescent"),
    (5, "Flash"),
    (6, "Custom"),
    (7, "Black & White"),
    (8, "Shade"),
    (9, "Manual Temperature (Kelvin)"),
    (10, "PC Set1"),
    (11, "PC Set2"),
    (12, "PC Set3"),
    (14, "Daylight Fluorescent"),
    (15, "Custom 1"),
    (16, "Custom 2"),
    (17, "Underwater"),
    (18, "Custom 3"),
    (19, "Custom 4"),
    (20, "PC Set4"),
    (21, "PC Set5"),
    (23, "Auto (ambience priority)"),
];

/// Lookup table (lazy-initialized)
pub static CANON_WHITE_BALANCE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| CANON_WHITE_BALANCE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_canon_white_balance(key: u8) -> Option<&'static str> {
    CANON_WHITE_BALANCE.get(&key).copied()
}
