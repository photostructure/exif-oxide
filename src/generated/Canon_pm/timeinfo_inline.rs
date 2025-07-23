//! Inline PrintConv tables for TimeInfo table
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Canon.pm (table: TimeInfo)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (35 entries)
static TIME_INFO_TIME_ZONE_CITY_DATA: &[(u16, &'static str)] = &[
    (0, "n/a"),
    (1, "Chatham Islands"),
    (2, "Wellington"),
    (3, "Solomon Islands"),
    (4, "Sydney"),
    (5, "Adelaide"),
    (6, "Tokyo"),
    (7, "Hong Kong"),
    (8, "Bangkok"),
    (9, "Yangon"),
    (10, "Dhaka"),
    (11, "Kathmandu"),
    (12, "Delhi"),
    (13, "Karachi"),
    (14, "Kabul"),
    (15, "Dubai"),
    (16, "Tehran"),
    (17, "Moscow"),
    (18, "Cairo"),
    (19, "Paris"),
    (20, "London"),
    (21, "Azores"),
    (22, "Fernando de Noronha"),
    (23, "Sao Paulo"),
    (24, "Newfoundland"),
    (25, "Santiago"),
    (26, "Caracas"),
    (27, "New York"),
    (28, "Chicago"),
    (29, "Denver"),
    (30, "Los Angeles"),
    (31, "Anchorage"),
    (32, "Honolulu"),
    (33, "Samoa"),
    (32766, "(not set)"),
];

/// Lookup table (lazy-initialized)
pub static TIME_INFO_TIME_ZONE_CITY: LazyLock<HashMap<u16, &'static str>> =
    LazyLock::new(|| TIME_INFO_TIME_ZONE_CITY_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_time_info__time_zone_city(key: u16) -> Option<&'static str> {
    TIME_INFO_TIME_ZONE_CITY.get(&key).copied()
}

/// Raw data (2 entries)
static TIME_INFO_DAYLIGHT_SAVINGS_DATA: &[(u8, &'static str)] = &[(0, "Off"), (60, "On")];

/// Lookup table (lazy-initialized)
pub static TIME_INFO_DAYLIGHT_SAVINGS: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| TIME_INFO_DAYLIGHT_SAVINGS_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_time_info__daylight_savings(key: u8) -> Option<&'static str> {
    TIME_INFO_DAYLIGHT_SAVINGS.get(&key).copied()
}
