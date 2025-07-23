//! Inline PrintConv tables for ExtraInfo3 table
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Sony.pm (table: ExtraInfo3)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (2 entries)
static EXTRA_INFO3_IMAGE_STABILIZATION_DATA: &[(u8, &'static str)] = &[(0, "Off"), (64, "On")];

/// Lookup table (lazy-initialized)
pub static EXTRA_INFO3_IMAGE_STABILIZATION: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        EXTRA_INFO3_IMAGE_STABILIZATION_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_extra_info3__image_stabilization(key: u8) -> Option<&'static str> {
    EXTRA_INFO3_IMAGE_STABILIZATION.get(&key).copied()
}

/// Raw data (4 entries)
static EXTRA_INFO3_CAMERA_ORIENTATION_DATA: &[(u8, &'static str)] = &[
    (0, "Horizontal (normal)"),
    (1, "Rotate 90 CW"),
    (2, "Rotate 270 CW"),
    (3, "Rotate 180"),
];

/// Lookup table (lazy-initialized)
pub static EXTRA_INFO3_CAMERA_ORIENTATION: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        EXTRA_INFO3_CAMERA_ORIENTATION_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_extra_info3__camera_orientation(key: u8) -> Option<&'static str> {
    EXTRA_INFO3_CAMERA_ORIENTATION.get(&key).copied()
}
