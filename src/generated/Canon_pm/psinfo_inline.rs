//! Inline PrintConv tables for PSInfo table
//! 
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Canon.pm (table: PSInfo)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (6 entries)
static P_S_INFO_FILTER_EFFECT_MONOCHROME_DATA: &[(i32, &'static str)] = &[
    (-559038737, "n/a"),
    (0, "None"),
    (1, "Yellow"),
    (2, "Orange"),
    (3, "Red"),
    (4, "Green"),
];

/// Lookup table (lazy-initialized)
pub static P_S_INFO_FILTER_EFFECT_MONOCHROME: LazyLock<HashMap<i32, &'static str>> = LazyLock::new(|| {
    P_S_INFO_FILTER_EFFECT_MONOCHROME_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_p_s_info__filter_effect_monochrome(key: i32) -> Option<&'static str> {
    P_S_INFO_FILTER_EFFECT_MONOCHROME.get(&key).copied()
}

/// Raw data (6 entries)
static P_S_INFO_TONING_EFFECT_MONOCHROME_DATA: &[(i32, &'static str)] = &[
    (-559038737, "n/a"),
    (0, "None"),
    (1, "Sepia"),
    (2, "Blue"),
    (3, "Purple"),
    (4, "Green"),
];

/// Lookup table (lazy-initialized)
pub static P_S_INFO_TONING_EFFECT_MONOCHROME: LazyLock<HashMap<i32, &'static str>> = LazyLock::new(|| {
    P_S_INFO_TONING_EFFECT_MONOCHROME_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_p_s_info__toning_effect_monochrome(key: i32) -> Option<&'static str> {
    P_S_INFO_TONING_EFFECT_MONOCHROME.get(&key).copied()
}

/// Raw data (6 entries)
static P_S_INFO_FILTER_EFFECT_USER_DEF1_DATA: &[(i32, &'static str)] = &[
    (-559038737, "n/a"),
    (0, "None"),
    (1, "Yellow"),
    (2, "Orange"),
    (3, "Red"),
    (4, "Green"),
];

/// Lookup table (lazy-initialized)
pub static P_S_INFO_FILTER_EFFECT_USER_DEF1: LazyLock<HashMap<i32, &'static str>> = LazyLock::new(|| {
    P_S_INFO_FILTER_EFFECT_USER_DEF1_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_p_s_info__filter_effect_user_def1(key: i32) -> Option<&'static str> {
    P_S_INFO_FILTER_EFFECT_USER_DEF1.get(&key).copied()
}

/// Raw data (6 entries)
static P_S_INFO_TONING_EFFECT_USER_DEF1_DATA: &[(i32, &'static str)] = &[
    (-559038737, "n/a"),
    (0, "None"),
    (1, "Sepia"),
    (2, "Blue"),
    (3, "Purple"),
    (4, "Green"),
];

/// Lookup table (lazy-initialized)
pub static P_S_INFO_TONING_EFFECT_USER_DEF1: LazyLock<HashMap<i32, &'static str>> = LazyLock::new(|| {
    P_S_INFO_TONING_EFFECT_USER_DEF1_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_p_s_info__toning_effect_user_def1(key: i32) -> Option<&'static str> {
    P_S_INFO_TONING_EFFECT_USER_DEF1.get(&key).copied()
}

/// Raw data (6 entries)
static P_S_INFO_FILTER_EFFECT_USER_DEF2_DATA: &[(i32, &'static str)] = &[
    (-559038737, "n/a"),
    (0, "None"),
    (1, "Yellow"),
    (2, "Orange"),
    (3, "Red"),
    (4, "Green"),
];

/// Lookup table (lazy-initialized)
pub static P_S_INFO_FILTER_EFFECT_USER_DEF2: LazyLock<HashMap<i32, &'static str>> = LazyLock::new(|| {
    P_S_INFO_FILTER_EFFECT_USER_DEF2_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_p_s_info__filter_effect_user_def2(key: i32) -> Option<&'static str> {
    P_S_INFO_FILTER_EFFECT_USER_DEF2.get(&key).copied()
}

/// Raw data (6 entries)
static P_S_INFO_TONING_EFFECT_USER_DEF2_DATA: &[(i32, &'static str)] = &[
    (-559038737, "n/a"),
    (0, "None"),
    (1, "Sepia"),
    (2, "Blue"),
    (3, "Purple"),
    (4, "Green"),
];

/// Lookup table (lazy-initialized)
pub static P_S_INFO_TONING_EFFECT_USER_DEF2: LazyLock<HashMap<i32, &'static str>> = LazyLock::new(|| {
    P_S_INFO_TONING_EFFECT_USER_DEF2_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_p_s_info__toning_effect_user_def2(key: i32) -> Option<&'static str> {
    P_S_INFO_TONING_EFFECT_USER_DEF2.get(&key).copied()
}

/// Raw data (6 entries)
static P_S_INFO_FILTER_EFFECT_USER_DEF3_DATA: &[(i32, &'static str)] = &[
    (-559038737, "n/a"),
    (0, "None"),
    (1, "Yellow"),
    (2, "Orange"),
    (3, "Red"),
    (4, "Green"),
];

/// Lookup table (lazy-initialized)
pub static P_S_INFO_FILTER_EFFECT_USER_DEF3: LazyLock<HashMap<i32, &'static str>> = LazyLock::new(|| {
    P_S_INFO_FILTER_EFFECT_USER_DEF3_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_p_s_info__filter_effect_user_def3(key: i32) -> Option<&'static str> {
    P_S_INFO_FILTER_EFFECT_USER_DEF3.get(&key).copied()
}

/// Raw data (6 entries)
static P_S_INFO_TONING_EFFECT_USER_DEF3_DATA: &[(i32, &'static str)] = &[
    (-559038737, "n/a"),
    (0, "None"),
    (1, "Sepia"),
    (2, "Blue"),
    (3, "Purple"),
    (4, "Green"),
];

/// Lookup table (lazy-initialized)
pub static P_S_INFO_TONING_EFFECT_USER_DEF3: LazyLock<HashMap<i32, &'static str>> = LazyLock::new(|| {
    P_S_INFO_TONING_EFFECT_USER_DEF3_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_p_s_info__toning_effect_user_def3(key: i32) -> Option<&'static str> {
    P_S_INFO_TONING_EFFECT_USER_DEF3.get(&key).copied()
}

/// Raw data (10 entries)
static P_S_INFO_USER_DEF1_PICTURE_STYLE_DATA: &[(u8, &'static str)] = &[
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
];

/// Lookup table (lazy-initialized)
pub static P_S_INFO_USER_DEF1_PICTURE_STYLE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    P_S_INFO_USER_DEF1_PICTURE_STYLE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_p_s_info__user_def1_picture_style(key: u8) -> Option<&'static str> {
    P_S_INFO_USER_DEF1_PICTURE_STYLE.get(&key).copied()
}

/// Raw data (10 entries)
static P_S_INFO_USER_DEF2_PICTURE_STYLE_DATA: &[(u8, &'static str)] = &[
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
];

/// Lookup table (lazy-initialized)
pub static P_S_INFO_USER_DEF2_PICTURE_STYLE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    P_S_INFO_USER_DEF2_PICTURE_STYLE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_p_s_info__user_def2_picture_style(key: u8) -> Option<&'static str> {
    P_S_INFO_USER_DEF2_PICTURE_STYLE.get(&key).copied()
}

/// Raw data (10 entries)
static P_S_INFO_USER_DEF3_PICTURE_STYLE_DATA: &[(u8, &'static str)] = &[
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
];

/// Lookup table (lazy-initialized)
pub static P_S_INFO_USER_DEF3_PICTURE_STYLE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    P_S_INFO_USER_DEF3_PICTURE_STYLE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_p_s_info__user_def3_picture_style(key: u8) -> Option<&'static str> {
    P_S_INFO_USER_DEF3_PICTURE_STYLE.get(&key).copied()
}
