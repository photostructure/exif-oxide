//! Panasonic white balance settings
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/PanasonicRaw.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (15 entries)
static PANASONIC_WHITE_BALANCE_DATA: &[(u8, &'static str)] = &[
    (0, "Auto"),
    (1, "Daylight"),
    (2, "Cloudy"),
    (3, "Tungsten"),
    (4, "n/a"),
    (5, "Flash"),
    (6, "n/a"),
    (7, "n/a"),
    (8, "Custom#1"),
    (9, "Custom#2"),
    (10, "Custom#3"),
    (11, "Custom#4"),
    (12, "Shade"),
    (13, "Kelvin"),
    (16, "AWBc"),
];

/// Lookup table (lazy-initialized)
pub static PANASONIC_WHITE_BALANCE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| PANASONIC_WHITE_BALANCE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_panasonic_white_balance(key: u8) -> Option<&'static str> {
    PANASONIC_WHITE_BALANCE.get(&key).copied()
}
