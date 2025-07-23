//! Inline PrintConv tables for MyColors table
//! 
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Canon.pm (table: MyColors)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (14 entries)
static MY_COLORS_MY_COLOR_MODE_DATA: &[(u8, &'static str)] = &[
    (0, "Off"),
    (1, "Positive Film"),
    (2, "Light Skin Tone"),
    (3, "Dark Skin Tone"),
    (4, "Vivid Blue"),
    (5, "Vivid Green"),
    (6, "Vivid Red"),
    (7, "Color Accent"),
    (8, "Color Swap"),
    (9, "Custom"),
    (12, "Vivid"),
    (13, "Neutral"),
    (14, "Sepia"),
    (15, "B&W"),
];

/// Lookup table (lazy-initialized)
pub static MY_COLORS_MY_COLOR_MODE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    MY_COLORS_MY_COLOR_MODE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_my_colors__my_color_mode(key: u8) -> Option<&'static str> {
    MY_COLORS_MY_COLOR_MODE.get(&key).copied()
}
