//! Inline PrintConv tables for RawInfo table
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Olympus.pm (table: RawInfo)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (12 entries)
static RAW_INFO_LIGHT_SOURCE_DATA: &[(u16, &'static str)] = &[
    (0, "Unknown"),
    (16, "Shade"),
    (17, "Cloudy"),
    (18, "Fine Weather"),
    (20, "Tungsten (Incandescent)"),
    (22, "Evening Sunlight"),
    (33, "Daylight Fluorescent"),
    (34, "Day White Fluorescent"),
    (35, "Cool White Fluorescent"),
    (36, "White Fluorescent"),
    (256, "One Touch White Balance"),
    (512, "Custom 1-4"),
];

/// Lookup table (lazy-initialized)
pub static RAW_INFO_LIGHT_SOURCE: LazyLock<HashMap<u16, &'static str>> =
    LazyLock::new(|| RAW_INFO_LIGHT_SOURCE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_raw_info__light_source(key: u16) -> Option<&'static str> {
    RAW_INFO_LIGHT_SOURCE.get(&key).copied()
}
