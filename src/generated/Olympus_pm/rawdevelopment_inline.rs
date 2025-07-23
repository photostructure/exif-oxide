//! Inline PrintConv tables for RawDevelopment table
//! 
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Olympus.pm (table: RawDevelopment)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (3 entries)
static RAW_DEVELOPMENT_RAW_DEV_COLOR_SPACE_DATA: &[(u8, &'static str)] = &[
    (0, "sRGB"),
    (1, "Adobe RGB"),
    (2, "Pro Photo RGB"),
];

/// Lookup table (lazy-initialized)
pub static RAW_DEVELOPMENT_RAW_DEV_COLOR_SPACE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    RAW_DEVELOPMENT_RAW_DEV_COLOR_SPACE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_raw_development__raw_dev_color_space(key: u8) -> Option<&'static str> {
    RAW_DEVELOPMENT_RAW_DEV_COLOR_SPACE.get(&key).copied()
}

/// Raw data (4 entries)
static RAW_DEVELOPMENT_RAW_DEV_ENGINE_DATA: &[(u8, &'static str)] = &[
    (0, "High Speed"),
    (1, "High Function"),
    (2, "Advanced High Speed"),
    (3, "Advanced High Function"),
];

/// Lookup table (lazy-initialized)
pub static RAW_DEVELOPMENT_RAW_DEV_ENGINE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    RAW_DEVELOPMENT_RAW_DEV_ENGINE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_raw_development__raw_dev_engine(key: u8) -> Option<&'static str> {
    RAW_DEVELOPMENT_RAW_DEV_ENGINE.get(&key).copied()
}

/// Raw data (1 entries)
static RAW_DEVELOPMENT_RAW_DEV_NOISE_REDUCTION_DATA: &[(u8, &'static str)] = &[
    (0, "(none)"),
];

/// Lookup table (lazy-initialized)
pub static RAW_DEVELOPMENT_RAW_DEV_NOISE_REDUCTION: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    RAW_DEVELOPMENT_RAW_DEV_NOISE_REDUCTION_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_raw_development__raw_dev_noise_reduction(key: u8) -> Option<&'static str> {
    RAW_DEVELOPMENT_RAW_DEV_NOISE_REDUCTION.get(&key).copied()
}

/// Raw data (4 entries)
static RAW_DEVELOPMENT_RAW_DEV_EDIT_STATUS_DATA: &[(u8, &'static str)] = &[
    (0, "Original"),
    (1, "Edited (Landscape)"),
    (6, "Edited (Portrait)"),
    (8, "Edited (Portrait)"),
];

/// Lookup table (lazy-initialized)
pub static RAW_DEVELOPMENT_RAW_DEV_EDIT_STATUS: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    RAW_DEVELOPMENT_RAW_DEV_EDIT_STATUS_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_raw_development__raw_dev_edit_status(key: u8) -> Option<&'static str> {
    RAW_DEVELOPMENT_RAW_DEV_EDIT_STATUS.get(&key).copied()
}

/// Raw data (1 entries)
static RAW_DEVELOPMENT_RAW_DEV_SETTINGS_DATA: &[(u8, &'static str)] = &[
    (0, "(none)"),
];

/// Lookup table (lazy-initialized)
pub static RAW_DEVELOPMENT_RAW_DEV_SETTINGS: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    RAW_DEVELOPMENT_RAW_DEV_SETTINGS_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_raw_development__raw_dev_settings(key: u8) -> Option<&'static str> {
    RAW_DEVELOPMENT_RAW_DEV_SETTINGS.get(&key).copied()
}
