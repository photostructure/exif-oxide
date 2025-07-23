//! Picture style mode names
//! 
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Canon.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (24 entries)
static PICTURE_STYLES_DATA: &[(u16, &'static str)] = &[
    (0, "None"),
    (1, "Standard"),
    (2, "Portrait"),
    (3, "High Saturation"),
    (4, "Adobe RGB"),
    (5, "Low Saturation"),
    (6, "CM Set 1"),
    (7, "CM Set 2"),
    (33, "User Def. 1"),
    (34, "User Def. 2"),
    (35, "User Def. 3"),
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
    (136, "Fine Detail"),
    (255, "n/a"),
    (65535, "n/a"),
];

/// Lookup table (lazy-initialized)
pub static PICTURE_STYLES: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    PICTURE_STYLES_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_picture_styles(key: u16) -> Option<&'static str> {
    PICTURE_STYLES.get(&key).copied()
}
