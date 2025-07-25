//! Image size setting names
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Canon.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (19 entries)
static CANON_IMAGE_SIZE_DATA: &[(i16, &'static str)] = &[
    (-1, "n/a"),
    (0, "Large"),
    (1, "Medium"),
    (2, "Small"),
    (5, "Medium 1"),
    (6, "Medium 2"),
    (7, "Medium 3"),
    (8, "Postcard"),
    (9, "Widescreen"),
    (10, "Medium Widescreen"),
    (14, "Small 1"),
    (15, "Small 2"),
    (16, "Small 3"),
    (128, "640x480 Movie"),
    (129, "Medium Movie"),
    (130, "Small Movie"),
    (137, "1280x720 Movie"),
    (142, "1920x1080 Movie"),
    (143, "4096x2160 Movie"),
];

/// Lookup table (lazy-initialized)
pub static CANON_IMAGE_SIZE: LazyLock<HashMap<i16, &'static str>> =
    LazyLock::new(|| CANON_IMAGE_SIZE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_canon_image_size(key: i16) -> Option<&'static str> {
    CANON_IMAGE_SIZE.get(&key).copied()
}
