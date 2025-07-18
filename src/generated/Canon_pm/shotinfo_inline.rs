//! Inline PrintConv tables for ShotInfo table
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Canon.pm (table: ShotInfo)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (8 entries)
static SHOT_INFO_A_F_POINTS_IN_FOCUS_DATA: &[(u16, &'static str)] = &[
    (12288, "None (MF)"),
    (12289, "Right"),
    (12290, "Center"),
    (12291, "Center+Right"),
    (12292, "Left"),
    (12293, "Left+Right"),
    (12294, "Left+Center"),
    (12295, "All"),
];

/// Lookup table (lazy-initialized)
pub static SHOT_INFO_A_F_POINTS_IN_FOCUS: LazyLock<HashMap<u16, &'static str>> =
    LazyLock::new(|| SHOT_INFO_A_F_POINTS_IN_FOCUS_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_shot_info__a_f_points_in_focus(key: u16) -> Option<&'static str> {
    SHOT_INFO_A_F_POINTS_IN_FOCUS.get(&key).copied()
}

/// Raw data (5 entries)
static SHOT_INFO_AUTO_EXPOSURE_BRACKETING_DATA: &[(i16, &'static str)] = &[
    (-1, "On"),
    (0, "Off"),
    (1, "On (shot 1)"),
    (2, "On (shot 2)"),
    (3, "On (shot 3)"),
];

/// Lookup table (lazy-initialized)
pub static SHOT_INFO_AUTO_EXPOSURE_BRACKETING: LazyLock<HashMap<i16, &'static str>> =
    LazyLock::new(|| {
        SHOT_INFO_AUTO_EXPOSURE_BRACKETING_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_shot_info__auto_exposure_bracketing(key: i16) -> Option<&'static str> {
    SHOT_INFO_AUTO_EXPOSURE_BRACKETING.get(&key).copied()
}

/// Raw data (3 entries)
static SHOT_INFO_CONTROL_MODE_DATA: &[(u8, &'static str)] = &[
    (0, "n/a"),
    (1, "Camera Local Control"),
    (3, "Computer Remote Control"),
];

/// Lookup table (lazy-initialized)
pub static SHOT_INFO_CONTROL_MODE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| SHOT_INFO_CONTROL_MODE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_shot_info__control_mode(key: u8) -> Option<&'static str> {
    SHOT_INFO_CONTROL_MODE.get(&key).copied()
}

/// Raw data (5 entries)
static SHOT_INFO_CAMERA_TYPE_DATA: &[(u8, &'static str)] = &[
    (0, "n/a"),
    (248, "EOS High-end"),
    (250, "Compact"),
    (252, "EOS Mid-range"),
    (255, "DV Camera"),
];

/// Lookup table (lazy-initialized)
pub static SHOT_INFO_CAMERA_TYPE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| SHOT_INFO_CAMERA_TYPE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_shot_info__camera_type(key: u8) -> Option<&'static str> {
    SHOT_INFO_CAMERA_TYPE.get(&key).copied()
}

/// Raw data (5 entries)
static SHOT_INFO_AUTO_ROTATE_DATA: &[(i16, &'static str)] = &[
    (-1, "n/a"),
    (0, "None"),
    (1, "Rotate 90 CW"),
    (2, "Rotate 180"),
    (3, "Rotate 270 CW"),
];

/// Lookup table (lazy-initialized)
pub static SHOT_INFO_AUTO_ROTATE: LazyLock<HashMap<i16, &'static str>> =
    LazyLock::new(|| SHOT_INFO_AUTO_ROTATE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_shot_info__auto_rotate(key: i16) -> Option<&'static str> {
    SHOT_INFO_AUTO_ROTATE.get(&key).copied()
}

/// Raw data (3 entries)
static SHOT_INFO_N_D_FILTER_DATA: &[(i16, &'static str)] = &[(-1, "n/a"), (0, "Off"), (1, "On")];

/// Lookup table (lazy-initialized)
pub static SHOT_INFO_N_D_FILTER: LazyLock<HashMap<i16, &'static str>> =
    LazyLock::new(|| SHOT_INFO_N_D_FILTER_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_shot_info__n_d_filter(key: i16) -> Option<&'static str> {
    SHOT_INFO_N_D_FILTER.get(&key).copied()
}

/// Raw data (22 entries)
static SHOT_INFO_WHITE_BALANCE_DATA: &[(u8, &'static str)] = &[
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
pub static SHOT_INFO_WHITE_BALANCE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| SHOT_INFO_WHITE_BALANCE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_shot_info__white_balance(key: u8) -> Option<&'static str> {
    SHOT_INFO_WHITE_BALANCE.get(&key).copied()
}

/// Raw data (5 entries)
static SHOT_INFO_SLOW_SHUTTER_DATA: &[(i16, &'static str)] = &[
    (-1, "n/a"),
    (0, "Off"),
    (1, "Night Scene"),
    (2, "On"),
    (3, "None"),
];

/// Lookup table (lazy-initialized)
pub static SHOT_INFO_SLOW_SHUTTER: LazyLock<HashMap<i16, &'static str>> =
    LazyLock::new(|| SHOT_INFO_SLOW_SHUTTER_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_shot_info__slow_shutter(key: i16) -> Option<&'static str> {
    SHOT_INFO_SLOW_SHUTTER.get(&key).copied()
}
