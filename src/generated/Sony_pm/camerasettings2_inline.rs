//! Inline PrintConv tables for CameraSettings2 table
//! 
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Sony.pm (table: CameraSettings2)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (7 entries)
static CAMERA_SETTINGS2_DRIVE_MODE_DATA: &[(u8, &'static str)] = &[
    (1, "Single Frame"),
    (2, "Continuous High"),
    (4, "Self-timer 10 sec"),
    (5, "Self-timer 2 sec, Mirror Lock-up"),
    (7, "Continuous Bracketing"),
    (10, "Remote Commander"),
    (11, "Continuous Self-timer"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_DRIVE_MODE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_DRIVE_MODE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__drive_mode(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_DRIVE_MODE.get(&key).copied()
}

/// Raw data (6 entries)
static CAMERA_SETTINGS2_FLASH_MODE_DATA: &[(u8, &'static str)] = &[
    (0, "Autoflash"),
    (2, "Rear Sync"),
    (3, "Wireless"),
    (4, "Fill-flash"),
    (5, "Flash Off"),
    (6, "Slow Sync"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_FLASH_MODE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_FLASH_MODE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__flash_mode(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_FLASH_MODE.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS2_COLOR_SPACE_DATA: &[(u8, &'static str)] = &[
    (5, "Adobe RGB"),
    (6, "sRGB"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_COLOR_SPACE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_COLOR_SPACE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__color_space(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_COLOR_SPACE.get(&key).copied()
}

/// Raw data (10 entries)
static CAMERA_SETTINGS2_WHITE_BALANCE_DATA: &[(u8, &'static str)] = &[
    (2, "Auto"),
    (4, "Daylight"),
    (5, "Fluorescent"),
    (6, "Tungsten"),
    (7, "Flash"),
    (12, "Color Temperature"),
    (13, "Color Filter"),
    (14, "Custom"),
    (16, "Cloudy"),
    (17, "Shade"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_WHITE_BALANCE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_WHITE_BALANCE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__white_balance(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_WHITE_BALANCE.get(&key).copied()
}

/// Raw data (4 entries)
static CAMERA_SETTINGS2_FOCUS_MODE_SETTING_DATA: &[(u8, &'static str)] = &[
    (0, "Manual"),
    (1, "AF-S"),
    (2, "AF-C"),
    (3, "AF-A"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_FOCUS_MODE_SETTING: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_FOCUS_MODE_SETTING_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__focus_mode_setting(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_FOCUS_MODE_SETTING.get(&key).copied()
}

/// Raw data (3 entries)
static CAMERA_SETTINGS2_A_F_AREA_MODE_DATA: &[(u8, &'static str)] = &[
    (0, "Wide"),
    (1, "Local"),
    (2, "Spot"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_A_F_AREA_MODE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_A_F_AREA_MODE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__a_f_area_mode(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_A_F_AREA_MODE.get(&key).copied()
}

/// Raw data (9 entries)
static CAMERA_SETTINGS2_A_F_POINT_SETTING_DATA: &[(u8, &'static str)] = &[
    (1, "Center"),
    (2, "Top"),
    (3, "Upper-right"),
    (4, "Right"),
    (5, "Lower-right"),
    (6, "Bottom"),
    (7, "Lower-left"),
    (8, "Left"),
    (9, "Upper-left"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_A_F_POINT_SETTING: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_A_F_POINT_SETTING_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__a_f_point_setting(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_A_F_POINT_SETTING.get(&key).copied()
}

/// Raw data (3 entries)
static CAMERA_SETTINGS2_METERING_MODE_DATA: &[(u8, &'static str)] = &[
    (1, "Multi-segment"),
    (2, "Center-weighted average"),
    (4, "Spot"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_METERING_MODE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_METERING_MODE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__metering_mode(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_METERING_MODE.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS2_HIGH_SPEED_SYNC_DATA: &[(u8, &'static str)] = &[
    (0, "Off"),
    (1, "On"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_HIGH_SPEED_SYNC: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_HIGH_SPEED_SYNC_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__high_speed_sync(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_HIGH_SPEED_SYNC.get(&key).copied()
}

/// Raw data (4 entries)
static CAMERA_SETTINGS2_DYNAMIC_RANGE_OPTIMIZER_MODE_DATA: &[(u8, &'static str)] = &[
    (0, "Off"),
    (1, "Standard"),
    (2, "Advanced Auto"),
    (3, "Advanced Level"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_DYNAMIC_RANGE_OPTIMIZER_MODE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_DYNAMIC_RANGE_OPTIMIZER_MODE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__dynamic_range_optimizer_mode(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_DYNAMIC_RANGE_OPTIMIZER_MODE.get(&key).copied()
}

/// Raw data (7 entries)
static CAMERA_SETTINGS2_CREATIVE_STYLE_DATA: &[(u8, &'static str)] = &[
    (1, "Standard"),
    (2, "Vivid"),
    (3, "Portrait"),
    (4, "Landscape"),
    (5, "Sunset"),
    (6, "Night View/Portrait"),
    (8, "B&W"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_CREATIVE_STYLE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_CREATIVE_STYLE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__creative_style(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_CREATIVE_STYLE.get(&key).copied()
}

/// Raw data (3 entries)
static CAMERA_SETTINGS2_FLASH_CONTROL_DATA: &[(u8, &'static str)] = &[
    (0, "ADI"),
    (1, "Pre-flash TTL"),
    (2, "Manual"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_FLASH_CONTROL: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_FLASH_CONTROL_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__flash_control(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_FLASH_CONTROL.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS2_LONG_EXPOSURE_NOISE_REDUCTION_DATA: &[(u8, &'static str)] = &[
    (0, "Off"),
    (1, "On"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_LONG_EXPOSURE_NOISE_REDUCTION: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_LONG_EXPOSURE_NOISE_REDUCTION_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__long_exposure_noise_reduction(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_LONG_EXPOSURE_NOISE_REDUCTION.get(&key).copied()
}

/// Raw data (4 entries)
static CAMERA_SETTINGS2_HIGH_I_S_O_NOISE_REDUCTION_DATA: &[(u8, &'static str)] = &[
    (0, "Off"),
    (1, "Low"),
    (2, "Normal"),
    (3, "High"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_HIGH_I_S_O_NOISE_REDUCTION: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_HIGH_I_S_O_NOISE_REDUCTION_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__high_i_s_o_noise_reduction(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_HIGH_I_S_O_NOISE_REDUCTION.get(&key).copied()
}

/// Raw data (7 entries)
static CAMERA_SETTINGS2_IMAGE_STYLE_DATA: &[(u8, &'static str)] = &[
    (1, "Standard"),
    (2, "Vivid"),
    (3, "Portrait"),
    (4, "Landscape"),
    (5, "Sunset"),
    (7, "Night View/Portrait"),
    (8, "B&W"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_IMAGE_STYLE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_IMAGE_STYLE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__image_style(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_IMAGE_STYLE.get(&key).copied()
}

/// Raw data (11 entries)
static CAMERA_SETTINGS2_WHITE_BALANCE_SETTING_DATA: &[(u8, &'static str)] = &[
    (2, "Auto"),
    (4, "Daylight"),
    (5, "Fluorescent"),
    (6, "Tungsten"),
    (7, "Flash"),
    (16, "Cloudy"),
    (17, "Shade"),
    (18, "Color Temperature/Color Filter"),
    (32, "Custom 1"),
    (33, "Custom 2"),
    (34, "Custom 3"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_WHITE_BALANCE_SETTING: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_WHITE_BALANCE_SETTING_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__white_balance_setting(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_WHITE_BALANCE_SETTING.get(&key).copied()
}

/// Raw data (14 entries)
static CAMERA_SETTINGS2_EXPOSURE_PROGRAM_DATA: &[(u8, &'static str)] = &[
    (0, "Auto"),
    (1, "Manual"),
    (2, "Program AE"),
    (3, "Aperture-priority AE"),
    (4, "Shutter speed priority AE"),
    (8, "Program Shift A"),
    (9, "Program Shift S"),
    (16, "Portrait"),
    (17, "Sports"),
    (18, "Sunset"),
    (19, "Night Portrait"),
    (20, "Landscape"),
    (21, "Macro"),
    (35, "Auto No Flash"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_EXPOSURE_PROGRAM: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_EXPOSURE_PROGRAM_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__exposure_program(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_EXPOSURE_PROGRAM.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS2_IMAGE_STABILIZATION_SETTING_DATA: &[(u8, &'static str)] = &[
    (0, "Off"),
    (1, "On"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_IMAGE_STABILIZATION_SETTING: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_IMAGE_STABILIZATION_SETTING_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__image_stabilization_setting(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_IMAGE_STABILIZATION_SETTING.get(&key).copied()
}

/// Raw data (4 entries)
static CAMERA_SETTINGS2_FLASH_ACTION_DATA: &[(u8, &'static str)] = &[
    (0, "Did not fire"),
    (1, "Fired"),
    (2, "External Flash, Did not fire"),
    (3, "External Flash, Fired"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_FLASH_ACTION: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_FLASH_ACTION_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__flash_action(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_FLASH_ACTION.get(&key).copied()
}

/// Raw data (3 entries)
static CAMERA_SETTINGS2_ROTATION_DATA: &[(u8, &'static str)] = &[
    (0, "Horizontal (normal)"),
    (1, "Rotate 90 CW"),
    (2, "Rotate 270 CW"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_ROTATION: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_ROTATION_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__rotation(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_ROTATION.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS2_A_E_LOCK_DATA: &[(u8, &'static str)] = &[
    (1, "Off"),
    (2, "On"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_A_E_LOCK: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_A_E_LOCK_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__a_e_lock(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_A_E_LOCK.get(&key).copied()
}

/// Raw data (9 entries)
static CAMERA_SETTINGS2_FLASH_ACTION2_DATA: &[(u8, &'static str)] = &[
    (1, "Fired, Autoflash"),
    (2, "Fired, Fill-flash"),
    (3, "Fired, Rear Sync"),
    (4, "Fired, Wireless"),
    (5, "Did not fire"),
    (6, "Fired, Slow Sync"),
    (17, "Fired, Autoflash, Red-eye reduction"),
    (18, "Fired, Fill-flash, Red-eye reduction"),
    (34, "Fired, Fill-flash, HSS"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_FLASH_ACTION2: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_FLASH_ACTION2_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__flash_action2(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_FLASH_ACTION2.get(&key).copied()
}

/// Raw data (4 entries)
static CAMERA_SETTINGS2_FOCUS_MODE_DATA: &[(u8, &'static str)] = &[
    (0, "Manual"),
    (1, "AF-S"),
    (2, "AF-C"),
    (3, "AF-A"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_FOCUS_MODE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_FOCUS_MODE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__focus_mode(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_FOCUS_MODE.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS2_FOCUS_STATUS_DATA: &[(u8, &'static str)] = &[
    (0, "Not confirmed"),
    (4, "Not confirmed, Tracking"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_FOCUS_STATUS: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_FOCUS_STATUS_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__focus_status(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_FOCUS_STATUS.get(&key).copied()
}

/// Raw data (3 entries)
static CAMERA_SETTINGS2_SONY_IMAGE_SIZE_DATA: &[(u8, &'static str)] = &[
    (1, "Large"),
    (2, "Medium"),
    (3, "Small"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_SONY_IMAGE_SIZE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_SONY_IMAGE_SIZE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__sony_image_size(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_SONY_IMAGE_SIZE.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS2_ASPECT_RATIO_DATA: &[(u8, &'static str)] = &[
    (1, "3:2"),
    (2, "16:9"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_ASPECT_RATIO: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_ASPECT_RATIO_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__aspect_ratio(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_ASPECT_RATIO.get(&key).copied()
}

/// Raw data (7 entries)
static CAMERA_SETTINGS2_QUALITY_DATA: &[(u8, &'static str)] = &[
    (0, "RAW"),
    (2, "CRAW"),
    (16, "Extra Fine"),
    (32, "Fine"),
    (34, "RAW + JPEG"),
    (35, "CRAW + JPEG"),
    (48, "Standard"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_QUALITY: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_QUALITY_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__quality(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_QUALITY.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS2_EXPOSURE_LEVEL_INCREMENTS_DATA: &[(u8, &'static str)] = &[
    (33, "1/3 EV"),
    (50, "1/2 EV"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS2_EXPOSURE_LEVEL_INCREMENTS: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_SETTINGS2_EXPOSURE_LEVEL_INCREMENTS_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_settings2__exposure_level_increments(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS2_EXPOSURE_LEVEL_INCREMENTS.get(&key).copied()
}
