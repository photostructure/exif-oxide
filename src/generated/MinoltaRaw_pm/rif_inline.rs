//! Inline PrintConv tables for RIF table
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/MinoltaRaw.pm (table: RIF)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (6 entries)
static R_I_F_PROGRAM_MODE_DATA: &[(u8, &'static str)] = &[
    (0, "None"),
    (1, "Portrait"),
    (2, "Text"),
    (3, "Night Portrait"),
    (4, "Sunset"),
    (5, "Sports"),
];

/// Lookup table (lazy-initialized)
pub static R_I_F_PROGRAM_MODE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| R_I_F_PROGRAM_MODE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_r_i_f__program_mode(key: u8) -> Option<&'static str> {
    R_I_F_PROGRAM_MODE.get(&key).copied()
}

/// Raw data (3 entries)
static R_I_F_ZONE_MATCHING_DATA: &[(u8, &'static str)] =
    &[(0, "ISO Setting Used"), (1, "High Key"), (2, "Low Key")];

/// Lookup table (lazy-initialized)
pub static R_I_F_ZONE_MATCHING: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| R_I_F_ZONE_MATCHING_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_r_i_f__zone_matching(key: u8) -> Option<&'static str> {
    R_I_F_ZONE_MATCHING.get(&key).copied()
}

/// Raw data (3 entries)
static R_I_F_ZONE_MATCHING_74_DATA: &[(u8, &'static str)] =
    &[(0, "ISO Setting Used"), (1, "High Key"), (2, "Low Key")];

/// Lookup table (lazy-initialized)
pub static R_I_F_ZONE_MATCHING_74: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| R_I_F_ZONE_MATCHING_74_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_r_i_f__zone_matching_74(key: u8) -> Option<&'static str> {
    R_I_F_ZONE_MATCHING_74.get(&key).copied()
}
