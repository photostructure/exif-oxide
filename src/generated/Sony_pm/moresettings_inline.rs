//! Inline PrintConv tables for MoreSettings table
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Sony.pm (table: MoreSettings)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (11 entries)
static MORE_SETTINGS_DRIVE_MODE2_DATA: &[(u8, &'static str)] = &[
    (16, "Single Frame"),
    (33, "Continuous High"),
    (34, "Continuous Low"),
    (48, "Speed Priority Continuous"),
    (81, "Self-timer 10 sec"),
    (82, "Self-timer 2 sec, Mirror Lock-up"),
    (113, "Continuous Bracketing 0.3 EV"),
    (117, "Continuous Bracketing 0.7 EV"),
    (145, "White Balance Bracketing Low"),
    (146, "White Balance Bracketing High"),
    (192, "Remote Commander"),
];

/// Lookup table (lazy-initialized)
pub static MORE_SETTINGS_DRIVE_MODE2: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| MORE_SETTINGS_DRIVE_MODE2_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_more_settings__drive_mode2(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_DRIVE_MODE2.get(&key).copied()
}

/// Raw data (4 entries)
static MORE_SETTINGS_FLASH_ACTION2_DATA: &[(u8, &'static str)] = &[
    (0, "Did not fire"),
    (2, "External Flash fired (2)"),
    (3, "Built-in Flash fired"),
    (4, "External Flash fired (4)"),
];

/// Lookup table (lazy-initialized)
pub static MORE_SETTINGS_FLASH_ACTION2: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| MORE_SETTINGS_FLASH_ACTION2_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_more_settings__flash_action2(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_FLASH_ACTION2.get(&key).copied()
}

/// Raw data (3 entries)
static MORE_SETTINGS_FLASH_ACTION_EXTERNAL_DATA: &[(u8, &'static str)] =
    &[(121, "Fired"), (122, "Fired"), (136, "Did not fire")];

/// Lookup table (lazy-initialized)
pub static MORE_SETTINGS_FLASH_ACTION_EXTERNAL: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        MORE_SETTINGS_FLASH_ACTION_EXTERNAL_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_more_settings__flash_action_external(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_FLASH_ACTION_EXTERNAL.get(&key).copied()
}

/// Raw data (3 entries)
static MORE_SETTINGS_FLASH_ACTION_EXTERNAL_124_DATA: &[(u8, &'static str)] =
    &[(136, "Did not fire"), (167, "Fired"), (182, "Fired, HSS")];

/// Lookup table (lazy-initialized)
pub static MORE_SETTINGS_FLASH_ACTION_EXTERNAL_124: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        MORE_SETTINGS_FLASH_ACTION_EXTERNAL_124_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_more_settings__flash_action_external_124(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_FLASH_ACTION_EXTERNAL_124.get(&key).copied()
}

/// Raw data (51 entries)
static MORE_SETTINGS_WHITE_BALANCE_SETTING_DATA: &[(u8, &'static str)] = &[
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
pub static MORE_SETTINGS_WHITE_BALANCE_SETTING: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        MORE_SETTINGS_WHITE_BALANCE_SETTING_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_more_settings__white_balance_setting(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_WHITE_BALANCE_SETTING.get(&key).copied()
}

/// Raw data (2 entries)
static MORE_SETTINGS_FLASH_STATUS_DATA: &[(u8, &'static str)] = &[(0, "None"), (2, "External")];

/// Lookup table (lazy-initialized)
pub static MORE_SETTINGS_FLASH_STATUS: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| MORE_SETTINGS_FLASH_STATUS_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_more_settings__flash_status(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_FLASH_STATUS.get(&key).copied()
}

/// Raw data (3 entries)
static MORE_SETTINGS_FLASH_STATUS_134_DATA: &[(u8, &'static str)] =
    &[(0, "None"), (1, "Built-in"), (2, "External")];

/// Lookup table (lazy-initialized)
pub static MORE_SETTINGS_FLASH_STATUS_134: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        MORE_SETTINGS_FLASH_STATUS_134_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_more_settings__flash_status_134(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_FLASH_STATUS_134.get(&key).copied()
}

/// Raw data (6 entries)
static MORE_SETTINGS_FLASH_MODE_DATA: &[(u8, &'static str)] = &[
    (1, "Flash Off"),
    (16, "Autoflash"),
    (17, "Fill-flash"),
    (18, "Slow Sync"),
    (19, "Rear Sync"),
    (20, "Wireless"),
];

/// Lookup table (lazy-initialized)
pub static MORE_SETTINGS_FLASH_MODE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| MORE_SETTINGS_FLASH_MODE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_more_settings__flash_mode(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_FLASH_MODE.get(&key).copied()
}

/// Raw data (2 entries)
static MORE_SETTINGS_LONG_EXPOSURE_NOISE_REDUCTION_DATA: &[(u8, &'static str)] =
    &[(1, "Off"), (16, "On")];

/// Lookup table (lazy-initialized)
pub static MORE_SETTINGS_LONG_EXPOSURE_NOISE_REDUCTION: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        MORE_SETTINGS_LONG_EXPOSURE_NOISE_REDUCTION_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_more_settings__long_exposure_noise_reduction(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_LONG_EXPOSURE_NOISE_REDUCTION
        .get(&key)
        .copied()
}

/// Raw data (3 entries)
static MORE_SETTINGS_HIGH_I_S_O_NOISE_REDUCTION_DATA: &[(u8, &'static str)] =
    &[(16, "Low"), (17, "High"), (19, "Auto")];

/// Lookup table (lazy-initialized)
pub static MORE_SETTINGS_HIGH_I_S_O_NOISE_REDUCTION: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        MORE_SETTINGS_HIGH_I_S_O_NOISE_REDUCTION_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_more_settings__high_i_s_o_noise_reduction(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_HIGH_I_S_O_NOISE_REDUCTION.get(&key).copied()
}

/// Raw data (5 entries)
static MORE_SETTINGS_FOCUS_MODE_DATA: &[(u8, &'static str)] = &[
    (17, "AF-S"),
    (18, "AF-C"),
    (19, "AF-A"),
    (32, "Manual"),
    (48, "DMF"),
];

/// Lookup table (lazy-initialized)
pub static MORE_SETTINGS_FOCUS_MODE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| MORE_SETTINGS_FOCUS_MODE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_more_settings__focus_mode(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_FOCUS_MODE.get(&key).copied()
}

/// Raw data (32 entries)
static MORE_SETTINGS_EXPOSURE_PROGRAM_DATA: &[(u8, &'static str)] = &[
    (1, "Program AE"),
    (2, "Aperture-priority AE"),
    (3, "Shutter speed priority AE"),
    (4, "Manual"),
    (5, "Cont. Priority AE"),
    (16, "Auto"),
    (17, "Auto (no flash)"),
    (18, "Auto+"),
    (49, "Portrait"),
    (50, "Landscape"),
    (51, "Macro"),
    (52, "Sports"),
    (53, "Sunset"),
    (54, "Night view"),
    (55, "Night view/portrait"),
    (56, "Handheld Night Shot"),
    (57, "3D Sweep Panorama"),
    (64, "Auto 2"),
    (65, "Auto 2 (no flash)"),
    (80, "Sweep Panorama"),
    (96, "Anti Motion Blur"),
    (128, "Toy Camera"),
    (129, "Pop Color"),
    (130, "Posterization"),
    (131, "Posterization B/W"),
    (132, "Retro Photo"),
    (133, "High-key"),
    (134, "Partial Color Red"),
    (135, "Partial Color Green"),
    (136, "Partial Color Blue"),
    (137, "Partial Color Yellow"),
    (138, "High Contrast Monochrome"),
];

/// Lookup table (lazy-initialized)
pub static MORE_SETTINGS_EXPOSURE_PROGRAM: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        MORE_SETTINGS_EXPOSURE_PROGRAM_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_more_settings__exposure_program(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_EXPOSURE_PROGRAM.get(&key).copied()
}

/// Raw data (4 entries)
static MORE_SETTINGS_MULTI_FRAME_NOISE_REDUCTION_DATA: &[(u8, &'static str)] =
    &[(0, "n/a"), (1, "Off"), (16, "On"), (255, "None")];

/// Lookup table (lazy-initialized)
pub static MORE_SETTINGS_MULTI_FRAME_NOISE_REDUCTION: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        MORE_SETTINGS_MULTI_FRAME_NOISE_REDUCTION_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_more_settings__multi_frame_noise_reduction(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_MULTI_FRAME_NOISE_REDUCTION.get(&key).copied()
}

/// Raw data (3 entries)
static MORE_SETTINGS_H_D_R_SETTING_DATA: &[(u8, &'static str)] =
    &[(1, "Off"), (16, "On (Auto)"), (17, "On (Manual)")];

/// Lookup table (lazy-initialized)
pub static MORE_SETTINGS_H_D_R_SETTING: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| MORE_SETTINGS_H_D_R_SETTING_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_more_settings__h_d_r_setting(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_H_D_R_SETTING.get(&key).copied()
}

/// Raw data (9 entries)
static MORE_SETTINGS_H_D_R_LEVEL_DATA: &[(u8, &'static str)] = &[
    (33, "1 EV"),
    (34, "1.5 EV"),
    (35, "2 EV"),
    (36, "2.5 EV"),
    (37, "3 EV"),
    (38, "3.5 EV"),
    (39, "4 EV"),
    (40, "5 EV"),
    (41, "6 EV"),
];

/// Lookup table (lazy-initialized)
pub static MORE_SETTINGS_H_D_R_LEVEL: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| MORE_SETTINGS_H_D_R_LEVEL_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_more_settings__h_d_r_level(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_H_D_R_LEVEL.get(&key).copied()
}

/// Raw data (3 entries)
static MORE_SETTINGS_VIEWING_MODE_DATA: &[(u8, &'static str)] = &[
    (16, "ViewFinder"),
    (33, "Focus Check Live View"),
    (34, "Quick AF Live View"),
];

/// Lookup table (lazy-initialized)
pub static MORE_SETTINGS_VIEWING_MODE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| MORE_SETTINGS_VIEWING_MODE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_more_settings__viewing_mode(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_VIEWING_MODE.get(&key).copied()
}

/// Raw data (2 entries)
static MORE_SETTINGS_FACE_DETECTION_DATA: &[(u8, &'static str)] = &[(1, "Off"), (16, "On")];

/// Lookup table (lazy-initialized)
pub static MORE_SETTINGS_FACE_DETECTION: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| MORE_SETTINGS_FACE_DETECTION_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_more_settings__face_detection(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_FACE_DETECTION.get(&key).copied()
}

/// Raw data (3 entries)
static MORE_SETTINGS_METERING_MODE_DATA: &[(u8, &'static str)] = &[
    (1, "Multi-segment"),
    (2, "Center-weighted average"),
    (3, "Spot"),
];

/// Lookup table (lazy-initialized)
pub static MORE_SETTINGS_METERING_MODE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| MORE_SETTINGS_METERING_MODE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_more_settings__metering_mode(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_METERING_MODE.get(&key).copied()
}

/// Raw data (3 entries)
static MORE_SETTINGS_DYNAMIC_RANGE_OPTIMIZER_SETTING_DATA: &[(u8, &'static str)] =
    &[(1, "Off"), (16, "On (Auto)"), (17, "On (Manual)")];

/// Lookup table (lazy-initialized)
pub static MORE_SETTINGS_DYNAMIC_RANGE_OPTIMIZER_SETTING: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        MORE_SETTINGS_DYNAMIC_RANGE_OPTIMIZER_SETTING_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_more_settings__dynamic_range_optimizer_setting(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_DYNAMIC_RANGE_OPTIMIZER_SETTING
        .get(&key)
        .copied()
}

/// Raw data (4 entries)
static MORE_SETTINGS_ORIENTATION2_DATA: &[(u8, &'static str)] = &[
    (1, "Horizontal (normal)"),
    (2, "Rotate 180"),
    (6, "Rotate 90 CW"),
    (8, "Rotate 270 CW"),
];

/// Lookup table (lazy-initialized)
pub static MORE_SETTINGS_ORIENTATION2: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| MORE_SETTINGS_ORIENTATION2_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_more_settings__orientation2(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_ORIENTATION2.get(&key).copied()
}

/// Raw data (2 entries)
static MORE_SETTINGS_FLASH_ACTION_DATA: &[(u8, &'static str)] =
    &[(0, "Did not fire"), (1, "Fired")];

/// Lookup table (lazy-initialized)
pub static MORE_SETTINGS_FLASH_ACTION: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| MORE_SETTINGS_FLASH_ACTION_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_more_settings__flash_action(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_FLASH_ACTION.get(&key).copied()
}

/// Raw data (2 entries)
static MORE_SETTINGS_FOCUS_MODE2_DATA: &[(u8, &'static str)] = &[(0, "AF"), (1, "MF")];

/// Lookup table (lazy-initialized)
pub static MORE_SETTINGS_FOCUS_MODE2: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| MORE_SETTINGS_FOCUS_MODE2_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_more_settings__focus_mode2(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_FOCUS_MODE2.get(&key).copied()
}

/// Raw data (2 entries)
static MORE_SETTINGS_COLOR_SPACE_DATA: &[(u8, &'static str)] = &[(1, "sRGB"), (2, "Adobe RGB")];

/// Lookup table (lazy-initialized)
pub static MORE_SETTINGS_COLOR_SPACE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| MORE_SETTINGS_COLOR_SPACE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_more_settings__color_space(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_COLOR_SPACE.get(&key).copied()
}

/// Raw data (6 entries)
static MORE_SETTINGS_CREATIVE_STYLE_SETTING_DATA: &[(u8, &'static str)] = &[
    (16, "Standard"),
    (32, "Vivid"),
    (64, "Portrait"),
    (80, "Landscape"),
    (96, "B&W"),
    (160, "Sunset"),
];

/// Lookup table (lazy-initialized)
pub static MORE_SETTINGS_CREATIVE_STYLE_SETTING: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        MORE_SETTINGS_CREATIVE_STYLE_SETTING_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_more_settings__creative_style_setting(key: u8) -> Option<&'static str> {
    MORE_SETTINGS_CREATIVE_STYLE_SETTING.get(&key).copied()
}
