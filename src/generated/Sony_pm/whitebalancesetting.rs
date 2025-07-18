//! Sony white balance settings with fine-tune adjustments
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Sony.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (51 entries)
static SONY_WHITE_BALANCE_SETTING_DATA: &[(u16, &'static str)] = &[
    (16, "Auto (-3)"),
    (17, "Auto (-2)"),
    (18, "Auto (-1)"),
    (19, "Auto (0)"),
    (20, "Auto (+1)"),
    (21, "Auto (+2)"),
    (22, "Auto (+3)"),
    (32, "Daylight (-3)"),
    (33, "Daylight (-2)"),
    (34, "Daylight (-1)"),
    (35, "Daylight (0)"),
    (36, "Daylight (+1)"),
    (37, "Daylight (+2)"),
    (38, "Daylight (+3)"),
    (48, "Shade (-3)"),
    (49, "Shade (-2)"),
    (50, "Shade (-1)"),
    (51, "Shade (0)"),
    (52, "Shade (+1)"),
    (53, "Shade (+2)"),
    (54, "Shade (+3)"),
    (64, "Cloudy (-3)"),
    (65, "Cloudy (-2)"),
    (66, "Cloudy (-1)"),
    (67, "Cloudy (0)"),
    (68, "Cloudy (+1)"),
    (69, "Cloudy (+2)"),
    (70, "Cloudy (+3)"),
    (80, "Tungsten (-3)"),
    (81, "Tungsten (-2)"),
    (82, "Tungsten (-1)"),
    (83, "Tungsten (0)"),
    (84, "Tungsten (+1)"),
    (85, "Tungsten (+2)"),
    (86, "Tungsten (+3)"),
    (96, "Fluorescent (-3)"),
    (97, "Fluorescent (-2)"),
    (98, "Fluorescent (-1)"),
    (99, "Fluorescent (0)"),
    (100, "Fluorescent (+1)"),
    (101, "Fluorescent (+2)"),
    (102, "Fluorescent (+3)"),
    (112, "Flash (-3)"),
    (113, "Flash (-2)"),
    (114, "Flash (-1)"),
    (115, "Flash (0)"),
    (116, "Flash (+1)"),
    (117, "Flash (+2)"),
    (118, "Flash (+3)"),
    (163, "Custom"),
    (243, "Color Temperature/Color Filter"),
];

/// Lookup table (lazy-initialized)
pub static SONY_WHITE_BALANCE_SETTING: LazyLock<HashMap<u16, &'static str>> =
    LazyLock::new(|| SONY_WHITE_BALANCE_SETTING_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_sony_white_balance_setting(key: u16) -> Option<&'static str> {
    SONY_WHITE_BALANCE_SETTING.get(&key).copied()
}
