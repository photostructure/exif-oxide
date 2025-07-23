//! Inline PrintConv tables for AFInfo2 table
//! 
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Canon.pm (table: AFInfo2)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (20 entries)
static A_F_INFO2_A_F_AREA_MODE_DATA: &[(u8, &'static str)] = &[
    (0, "Off (Manual Focus)"),
    (1, "AF Point Expansion (surround)"),
    (2, "Single-point AF"),
    (4, "Auto"),
    (5, "Face Detect AF"),
    (6, "Face + Tracking"),
    (7, "Zone AF"),
    (8, "AF Point Expansion (4 point)"),
    (9, "Spot AF"),
    (10, "AF Point Expansion (8 point)"),
    (11, "Flexizone Multi (49 point)"),
    (12, "Flexizone Multi (9 point)"),
    (13, "Flexizone Single"),
    (14, "Large Zone AF"),
    (16, "Large Zone AF (vertical)"),
    (17, "Large Zone AF (horizontal)"),
    (19, "Flexible Zone AF 1"),
    (20, "Flexible Zone AF 2"),
    (21, "Flexible Zone AF 3"),
    (22, "Whole Area AF"),
];

/// Lookup table (lazy-initialized)
pub static A_F_INFO2_A_F_AREA_MODE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    A_F_INFO2_A_F_AREA_MODE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_a_f_info2__a_f_area_mode(key: u8) -> Option<&'static str> {
    A_F_INFO2_A_F_AREA_MODE.get(&key).copied()
}
