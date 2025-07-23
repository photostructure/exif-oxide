//! Inline PrintConv tables for RawDevelopment2 table
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Olympus.pm (table: RawDevelopment2)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (2 entries)
static RAW_DEVELOPMENT2_RAW_DEV_WHITE_BALANCE_DATA: &[(u8, &'static str)] =
    &[(1, "Color Temperature"), (2, "Gray Point")];

/// Lookup table (lazy-initialized)
pub static RAW_DEVELOPMENT2_RAW_DEV_WHITE_BALANCE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        RAW_DEVELOPMENT2_RAW_DEV_WHITE_BALANCE_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_raw_development2__raw_dev_white_balance(key: u8) -> Option<&'static str> {
    RAW_DEVELOPMENT2_RAW_DEV_WHITE_BALANCE.get(&key).copied()
}

/// Raw data (3 entries)
static RAW_DEVELOPMENT2_RAW_DEV_COLOR_SPACE_DATA: &[(u8, &'static str)] =
    &[(0, "sRGB"), (1, "Adobe RGB"), (2, "Pro Photo RGB")];

/// Lookup table (lazy-initialized)
pub static RAW_DEVELOPMENT2_RAW_DEV_COLOR_SPACE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        RAW_DEVELOPMENT2_RAW_DEV_COLOR_SPACE_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_raw_development2__raw_dev_color_space(key: u8) -> Option<&'static str> {
    RAW_DEVELOPMENT2_RAW_DEV_COLOR_SPACE.get(&key).copied()
}

/// Raw data (1 entries)
static RAW_DEVELOPMENT2_RAW_DEV_NOISE_REDUCTION_DATA: &[(u8, &'static str)] = &[(0, "(none)")];

/// Lookup table (lazy-initialized)
pub static RAW_DEVELOPMENT2_RAW_DEV_NOISE_REDUCTION: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        RAW_DEVELOPMENT2_RAW_DEV_NOISE_REDUCTION_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_raw_development2__raw_dev_noise_reduction(key: u8) -> Option<&'static str> {
    RAW_DEVELOPMENT2_RAW_DEV_NOISE_REDUCTION.get(&key).copied()
}

/// Raw data (2 entries)
static RAW_DEVELOPMENT2_RAW_DEV_ENGINE_DATA: &[(u8, &'static str)] =
    &[(0, "High Speed"), (1, "High Function")];

/// Lookup table (lazy-initialized)
pub static RAW_DEVELOPMENT2_RAW_DEV_ENGINE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        RAW_DEVELOPMENT2_RAW_DEV_ENGINE_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_raw_development2__raw_dev_engine(key: u8) -> Option<&'static str> {
    RAW_DEVELOPMENT2_RAW_DEV_ENGINE.get(&key).copied()
}

/// Raw data (5 entries)
static RAW_DEVELOPMENT2_RAW_DEV_PICTURE_MODE_DATA: &[(u16, &'static str)] = &[
    (1, "Vivid"),
    (2, "Natural"),
    (3, "Muted"),
    (256, "Monotone"),
    (512, "Sepia"),
];

/// Lookup table (lazy-initialized)
pub static RAW_DEVELOPMENT2_RAW_DEV_PICTURE_MODE: LazyLock<HashMap<u16, &'static str>> =
    LazyLock::new(|| {
        RAW_DEVELOPMENT2_RAW_DEV_PICTURE_MODE_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_raw_development2__raw_dev_picture_mode(key: u16) -> Option<&'static str> {
    RAW_DEVELOPMENT2_RAW_DEV_PICTURE_MODE.get(&key).copied()
}

/// Raw data (5 entries)
static RAW_DEVELOPMENT2_RAW_DEV_P_M__B_W_FILTER_DATA: &[(u8, &'static str)] = &[
    (1, "Neutral"),
    (2, "Yellow"),
    (3, "Orange"),
    (4, "Red"),
    (5, "Green"),
];

/// Lookup table (lazy-initialized)
pub static RAW_DEVELOPMENT2_RAW_DEV_P_M__B_W_FILTER: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        RAW_DEVELOPMENT2_RAW_DEV_P_M__B_W_FILTER_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_raw_development2__raw_dev_p_m__b_w_filter(key: u8) -> Option<&'static str> {
    RAW_DEVELOPMENT2_RAW_DEV_P_M__B_W_FILTER.get(&key).copied()
}

/// Raw data (5 entries)
static RAW_DEVELOPMENT2_RAW_DEV_P_M_PICTURE_TONE_DATA: &[(u8, &'static str)] = &[
    (1, "Neutral"),
    (2, "Sepia"),
    (3, "Blue"),
    (4, "Purple"),
    (5, "Green"),
];

/// Lookup table (lazy-initialized)
pub static RAW_DEVELOPMENT2_RAW_DEV_P_M_PICTURE_TONE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        RAW_DEVELOPMENT2_RAW_DEV_P_M_PICTURE_TONE_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_raw_development2__raw_dev_p_m_picture_tone(key: u8) -> Option<&'static str> {
    RAW_DEVELOPMENT2_RAW_DEV_P_M_PICTURE_TONE.get(&key).copied()
}

/// Raw data (2 entries)
static RAW_DEVELOPMENT2_RAW_DEV_AUTO_GRADATION_DATA: &[(u8, &'static str)] =
    &[(0, "Off"), (1, "On")];

/// Lookup table (lazy-initialized)
pub static RAW_DEVELOPMENT2_RAW_DEV_AUTO_GRADATION: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        RAW_DEVELOPMENT2_RAW_DEV_AUTO_GRADATION_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_raw_development2__raw_dev_auto_gradation(key: u8) -> Option<&'static str> {
    RAW_DEVELOPMENT2_RAW_DEV_AUTO_GRADATION.get(&key).copied()
}
