//! Inline PrintConv tables for Main table
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/PanasonicRaw.pm (table: Main)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (4 entries)
static MAIN_COMPRESSION_DATA: &[(i32, &'static str)] = &[
    (34316, "Panasonic RAW 1"),
    (34826, "Panasonic RAW 2"),
    (34828, "Panasonic RAW 3"),
    (34830, "Panasonic RAW 4"),
];

/// Lookup table (lazy-initialized)
pub static MAIN_COMPRESSION: LazyLock<HashMap<i32, &'static str>> =
    LazyLock::new(|| MAIN_COMPRESSION_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_main__compression(key: i32) -> Option<&'static str> {
    MAIN_COMPRESSION.get(&key).copied()
}

/// Raw data (8 entries)
static MAIN_ORIENTATION_DATA: &[(u8, &'static str)] = &[
    (1, "Horizontal (normal)"),
    (2, "Mirror horizontal"),
    (3, "Rotate 180"),
    (4, "Mirror vertical"),
    (5, "Mirror horizontal and rotate 270 CW"),
    (6, "Rotate 90 CW"),
    (7, "Mirror horizontal and rotate 90 CW"),
    (8, "Rotate 270 CW"),
];

/// Lookup table (lazy-initialized)
pub static MAIN_ORIENTATION: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| MAIN_ORIENTATION_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_main__orientation(key: u8) -> Option<&'static str> {
    MAIN_ORIENTATION.get(&key).copied()
}

/// Raw data (2 entries)
static MAIN_MULTISHOT_DATA: &[(i32, &'static str)] = &[(0, "Off"), (65536, "Pixel Shift")];

/// Lookup table (lazy-initialized)
pub static MAIN_MULTISHOT: LazyLock<HashMap<i32, &'static str>> =
    LazyLock::new(|| MAIN_MULTISHOT_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_main__multishot(key: i32) -> Option<&'static str> {
    MAIN_MULTISHOT.get(&key).copied()
}

/// Raw data (5 entries)
static MAIN_C_F_A_PATTERN_DATA: &[(u8, &'static str)] = &[
    (0, "n/a"),
    (1, "[Red,Green][Green,Blue]"),
    (2, "[Green,Red][Blue,Green]"),
    (3, "[Green,Blue][Red,Green]"),
    (4, "[Blue,Green][Green,Red]"),
];

/// Lookup table (lazy-initialized)
pub static MAIN_C_F_A_PATTERN: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| MAIN_C_F_A_PATTERN_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_main__c_f_a_pattern(key: u8) -> Option<&'static str> {
    MAIN_C_F_A_PATTERN.get(&key).copied()
}
