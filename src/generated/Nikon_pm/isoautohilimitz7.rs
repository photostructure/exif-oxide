//! Nikon Z7/Z8/Z9 ISO auto high limit settings
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Nikon.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (3 entries)
static ISO_AUTO_HI_LIMIT_Z7_DATA: &[(u8, &'static str)] = &[
    (Format, "int16u"),
    (SeparateTable, "ISOAutoHiLimitZ7"),
    (Unknown, "1"),
];

/// Lookup table (lazy-initialized)
pub static ISO_AUTO_HI_LIMIT_Z7: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| ISO_AUTO_HI_LIMIT_Z7_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_iso_auto_hi_limit_z7(key: u8) -> Option<&'static str> {
    ISO_AUTO_HI_LIMIT_Z7.get(&key).copied()
}
