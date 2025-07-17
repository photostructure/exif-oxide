//! Image quality setting names
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Canon.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (9 entries)
static CANON_QUALITY_DATA: &[(i16, &'static str)] = &[
    (-1, "n/a"),
    (1, "Economy"),
    (2, "Normal"),
    (3, "Fine"),
    (4, "RAW"),
    (5, "Superfine"),
    (7, "CRAW"),
    (130, "Light (RAW)"),
    (131, "Standard (RAW)"),
];

/// Lookup table (lazy-initialized)
pub static CANON_QUALITY: LazyLock<HashMap<i16, &'static str>> =
    LazyLock::new(|| CANON_QUALITY_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_canon_quality(key: i16) -> Option<&'static str> {
    CANON_QUALITY.get(&key).copied()
}
