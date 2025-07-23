//! Inline PrintConv tables for PRD table
//! 
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/MinoltaRaw.pm (table: PRD)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (2 entries)
static P_R_D_STORAGE_METHOD_DATA: &[(u8, &'static str)] = &[
    (82, "Padded"),
    (89, "Linear"),
];

/// Lookup table (lazy-initialized)
pub static P_R_D_STORAGE_METHOD: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    P_R_D_STORAGE_METHOD_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_p_r_d__storage_method(key: u8) -> Option<&'static str> {
    P_R_D_STORAGE_METHOD.get(&key).copied()
}

/// Raw data (2 entries)
static P_R_D_BAYER_PATTERN_DATA: &[(u8, &'static str)] = &[
    (1, "RGGB"),
    (4, "GBRG"),
];

/// Lookup table (lazy-initialized)
pub static P_R_D_BAYER_PATTERN: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    P_R_D_BAYER_PATTERN_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_p_r_d__bayer_pattern(key: u8) -> Option<&'static str> {
    P_R_D_BAYER_PATTERN.get(&key).copied()
}
