//! Inline PrintConv tables for ModifiedInfo table
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Canon.pm (table: ModifiedInfo)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (3 entries)
static MODIFIED_INFO_MODIFIED_TONE_CURVE_DATA: &[(u8, &'static str)] =
    &[(0, "Standard"), (1, "Manual"), (2, "Custom")];

/// Lookup table (lazy-initialized)
pub static MODIFIED_INFO_MODIFIED_TONE_CURVE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        MODIFIED_INFO_MODIFIED_TONE_CURVE_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_modified_info__modified_tone_curve(key: u8) -> Option<&'static str> {
    MODIFIED_INFO_MODIFIED_TONE_CURVE.get(&key).copied()
}

/// Raw data (24 entries)
static MODIFIED_INFO_MODIFIED_PICTURE_STYLE_DATA: &[(i32, &'static str)] = &[
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
pub static MODIFIED_INFO_MODIFIED_PICTURE_STYLE: LazyLock<HashMap<i32, &'static str>> =
    LazyLock::new(|| {
        MODIFIED_INFO_MODIFIED_PICTURE_STYLE_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_modified_info__modified_picture_style(key: i32) -> Option<&'static str> {
    MODIFIED_INFO_MODIFIED_PICTURE_STYLE.get(&key).copied()
}

/// Raw data (6 entries)
static MODIFIED_INFO_MODIFIED_SHARPNESS_FREQ_DATA: &[(u8, &'static str)] = &[
    (0, "n/a"),
    (1, "Lowest"),
    (2, "Low"),
    (3, "Standard"),
    (4, "High"),
    (5, "Highest"),
];

/// Lookup table (lazy-initialized)
pub static MODIFIED_INFO_MODIFIED_SHARPNESS_FREQ: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        MODIFIED_INFO_MODIFIED_SHARPNESS_FREQ_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_modified_info__modified_sharpness_freq(key: u8) -> Option<&'static str> {
    MODIFIED_INFO_MODIFIED_SHARPNESS_FREQ.get(&key).copied()
}

/// Raw data (22 entries)
static MODIFIED_INFO_MODIFIED_WHITE_BALANCE_DATA: &[(u8, &'static str)] = &[
    (0, "Auto"),
    (1, "Daylight"),
    (2, "Cloudy"),
    (3, "Tungsten"),
    (4, "Fluorescent"),
    (5, "Flash"),
    (6, "Custom"),
    (7, "Black & White"),
    (8, "Shade"),
    (9, "Manual Temperature (Kelvin)"),
    (10, "PC Set1"),
    (11, "PC Set2"),
    (12, "PC Set3"),
    (14, "Daylight Fluorescent"),
    (15, "Custom 1"),
    (16, "Custom 2"),
    (17, "Underwater"),
    (18, "Custom 3"),
    (19, "Custom 4"),
    (20, "PC Set4"),
    (21, "PC Set5"),
    (23, "Auto (ambience priority)"),
];

/// Lookup table (lazy-initialized)
pub static MODIFIED_INFO_MODIFIED_WHITE_BALANCE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        MODIFIED_INFO_MODIFIED_WHITE_BALANCE_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_modified_info__modified_white_balance(key: u8) -> Option<&'static str> {
    MODIFIED_INFO_MODIFIED_WHITE_BALANCE.get(&key).copied()
}
