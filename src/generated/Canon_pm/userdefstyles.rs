//! Canon user-defined picture styles
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Canon.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (10 entries)
static CANON_USER_DEF_STYLES_DATA: &[(u8, &str)] = &[
    (65, "PC 1"),
    (66, "PC 2"),
    (67, "PC 3"),
    (129, "Standard"),
    (130, "Portrait"),
    (131, "Landscape"),
    (132, "Neutral"),
    (133, "Faithful"),
    (134, "Monochrome"),
    (135, "Auto"),
];

/// Lookup table (lazy-initialized)
pub static CANON_USER_DEF_STYLES: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| CANON_USER_DEF_STYLES_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_canon_user_def_styles(key: u8) -> Option<&'static str> {
    CANON_USER_DEF_STYLES.get(&key).copied()
}
