//! Kyocera ISO lookup table
//!
//! Auto-generated from ExifTool KyoceraRaw.pm ISO PrintConv data
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen
//!
//! Source: third-party/exiftool/lib/Image/ExifTool/KyoceraRaw.pm:56-70

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw ISO lookup data (13 entries)
static KYOCERA_ISO_DATA: &[(u32, u32)] = &[
    (7, 25),
    (8, 32),
    (9, 40),
    (10, 50),
    (11, 64),
    (12, 80),
    (13, 100),
    (14, 125),
    (15, 160),
    (16, 200),
    (17, 250),
    (18, 320),
    (19, 400),
];

/// Lookup table (lazy-initialized)
pub static KYOCERA_ISO_LOOKUP: LazyLock<HashMap<u32, u32>> =
    LazyLock::new(|| KYOCERA_ISO_DATA.iter().cloned().collect());

/// Look up Kyocera internal ISO value to standard ISO speed
/// ExifTool: KyoceraRaw.pm %isoLookup hash
/// Maps internal values 7-19 to ISO speeds 25-400
pub fn lookup_kyocera_iso(key: u32) -> Option<u32> {
    KYOCERA_ISO_LOOKUP.get(&key).copied()
}
