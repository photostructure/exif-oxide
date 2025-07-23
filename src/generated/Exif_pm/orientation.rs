//! EXIF Orientation tag PrintConv values
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Exif.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (8 entries)
static ORIENTATION_DATA: &[(u8, &'static str)] = &[
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
pub static ORIENTATION: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| ORIENTATION_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_orientation(key: u8) -> Option<&'static str> {
    ORIENTATION.get(&key).copied()
}
