//! Inline PrintConv tables for CameraSettings table
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Olympus.pm (table: CameraSettings)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (1 entries)
static CAMERA_SETTINGS_FLASH_MODE_DATA: &[(u8, &'static str)] = &[(0, "Off")];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_FLASH_MODE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| CAMERA_SETTINGS_FLASH_MODE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_camera_settings__flash_mode(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS_FLASH_MODE.get(&key).copied()
}

/// Raw data (13 entries)
static CAMERA_SETTINGS_FLASH_REMOTE_CONTROL_DATA: &[(u8, &'static str)] = &[
    (0, "Off"),
    (1, "Channel 1, Low"),
    (2, "Channel 2, Low"),
    (3, "Channel 3, Low"),
    (4, "Channel 4, Low"),
    (9, "Channel 1, Mid"),
    (10, "Channel 2, Mid"),
    (11, "Channel 3, Mid"),
    (12, "Channel 4, Mid"),
    (17, "Channel 1, High"),
    (18, "Channel 2, High"),
    (19, "Channel 3, High"),
    (20, "Channel 4, High"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_FLASH_REMOTE_CONTROL: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        CAMERA_SETTINGS_FLASH_REMOTE_CONTROL_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_camera_settings__flash_remote_control(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS_FLASH_REMOTE_CONTROL.get(&key).copied()
}

/// Raw data (23 entries)
static CAMERA_SETTINGS_WHITE_BALANCE2_DATA: &[(u16, &'static str)] = &[
    (0, "Auto"),
    (1, "Auto (Keep Warm Color Off)"),
    (16, "7500K (Fine Weather with Shade)"),
    (17, "6000K (Cloudy)"),
    (18, "5300K (Fine Weather)"),
    (20, "3000K (Tungsten light)"),
    (21, "3600K (Tungsten light-like)"),
    (22, "Auto Setup"),
    (23, "5500K (Flash)"),
    (33, "6600K (Daylight fluorescent)"),
    (34, "4500K (Neutral white fluorescent)"),
    (35, "4000K (Cool white fluorescent)"),
    (36, "White Fluorescent"),
    (48, "3600K (Tungsten light-like)"),
    (67, "Underwater"),
    (256, "One Touch WB 1"),
    (257, "One Touch WB 2"),
    (258, "One Touch WB 3"),
    (259, "One Touch WB 4"),
    (512, "Custom WB 1"),
    (513, "Custom WB 2"),
    (514, "Custom WB 3"),
    (515, "Custom WB 4"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_WHITE_BALANCE2: LazyLock<HashMap<u16, &'static str>> =
    LazyLock::new(|| {
        CAMERA_SETTINGS_WHITE_BALANCE2_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_camera_settings__white_balance2(key: u16) -> Option<&'static str> {
    CAMERA_SETTINGS_WHITE_BALANCE2.get(&key).copied()
}

/// Raw data (5 entries)
static CAMERA_SETTINGS_MODIFIED_SATURATION_DATA: &[(u8, &'static str)] = &[
    (0, "Off"),
    (1, "CM1 (Red Enhance)"),
    (2, "CM2 (Green Enhance)"),
    (3, "CM3 (Blue Enhance)"),
    (4, "CM4 (Skin Tones)"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_MODIFIED_SATURATION: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        CAMERA_SETTINGS_MODIFIED_SATURATION_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_camera_settings__modified_saturation(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS_MODIFIED_SATURATION.get(&key).copied()
}

/// Raw data (3 entries)
static CAMERA_SETTINGS_COLOR_SPACE_DATA: &[(u8, &'static str)] =
    &[(0, "sRGB"), (1, "Adobe RGB"), (2, "Pro Photo RGB")];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_COLOR_SPACE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| CAMERA_SETTINGS_COLOR_SPACE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_camera_settings__color_space(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS_COLOR_SPACE.get(&key).copied()
}

/// Raw data (62 entries)
static CAMERA_SETTINGS_SCENE_MODE_DATA: &[(u8, &'static str)] = &[
    (0, "Standard"),
    (6, "Auto"),
    (7, "Sport"),
    (8, "Portrait"),
    (9, "Landscape+Portrait"),
    (10, "Landscape"),
    (11, "Night Scene"),
    (12, "Self Portrait"),
    (13, "Panorama"),
    (14, "2 in 1"),
    (15, "Movie"),
    (16, "Landscape+Portrait"),
    (17, "Night+Portrait"),
    (18, "Indoor"),
    (19, "Fireworks"),
    (20, "Sunset"),
    (21, "Beauty Skin"),
    (22, "Macro"),
    (23, "Super Macro"),
    (24, "Food"),
    (25, "Documents"),
    (26, "Museum"),
    (27, "Shoot & Select"),
    (28, "Beach & Snow"),
    (29, "Self Protrait+Timer"),
    (30, "Candle"),
    (31, "Available Light"),
    (32, "Behind Glass"),
    (33, "My Mode"),
    (34, "Pet"),
    (35, "Underwater Wide1"),
    (36, "Underwater Macro"),
    (37, "Shoot & Select1"),
    (38, "Shoot & Select2"),
    (39, "High Key"),
    (40, "Digital Image Stabilization"),
    (41, "Auction"),
    (42, "Beach"),
    (43, "Snow"),
    (44, "Underwater Wide2"),
    (45, "Low Key"),
    (46, "Children"),
    (47, "Vivid"),
    (48, "Nature Macro"),
    (49, "Underwater Snapshot"),
    (50, "Shooting Guide"),
    (54, "Face Portrait"),
    (57, "Bulb"),
    (59, "Smile Shot"),
    (60, "Quick Shutter"),
    (63, "Slow Shutter"),
    (64, "Bird Watching"),
    (65, "Multiple Exposure"),
    (66, "e-Portrait"),
    (67, "Soft Background Shot"),
    (142, "Hand-held Starlight"),
    (154, "HDR"),
    (197, "Panning"),
    (203, "Light Trails"),
    (204, "Backlight HDR"),
    (205, "Silent"),
    (206, "Multi Focus Shot"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_SCENE_MODE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| CAMERA_SETTINGS_SCENE_MODE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_camera_settings__scene_mode(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS_SCENE_MODE.get(&key).copied()
}

/// Raw data (1 entries)
static CAMERA_SETTINGS_NOISE_REDUCTION_DATA: &[(u8, &'static str)] = &[(0, "(none)")];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_NOISE_REDUCTION: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        CAMERA_SETTINGS_NOISE_REDUCTION_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_camera_settings__noise_reduction(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS_NOISE_REDUCTION.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS_DISTORTION_CORRECTION_DATA: &[(u8, &'static str)] = &[(0, "Off"), (1, "On")];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_DISTORTION_CORRECTION: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        CAMERA_SETTINGS_DISTORTION_CORRECTION_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_camera_settings__distortion_correction(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS_DISTORTION_CORRECTION.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS_SHADING_COMPENSATION_DATA: &[(u8, &'static str)] = &[(0, "Off"), (1, "On")];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_SHADING_COMPENSATION: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        CAMERA_SETTINGS_SHADING_COMPENSATION_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_camera_settings__shading_compensation(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS_SHADING_COMPENSATION.get(&key).copied()
}

/// Raw data (6 entries)
static CAMERA_SETTINGS_PICTURE_MODE_B_W_FILTER_DATA: &[(u8, &'static str)] = &[
    (0, "n/a"),
    (1, "Neutral"),
    (2, "Yellow"),
    (3, "Orange"),
    (4, "Red"),
    (5, "Green"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_PICTURE_MODE_B_W_FILTER: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        CAMERA_SETTINGS_PICTURE_MODE_B_W_FILTER_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_camera_settings__picture_mode_b_w_filter(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS_PICTURE_MODE_B_W_FILTER.get(&key).copied()
}

/// Raw data (6 entries)
static CAMERA_SETTINGS_PICTURE_MODE_TONE_DATA: &[(u8, &'static str)] = &[
    (0, "n/a"),
    (1, "Neutral"),
    (2, "Sepia"),
    (3, "Blue"),
    (4, "Purple"),
    (5, "Green"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_PICTURE_MODE_TONE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        CAMERA_SETTINGS_PICTURE_MODE_TONE_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_camera_settings__picture_mode_tone(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS_PICTURE_MODE_TONE.get(&key).copied()
}

/// Raw data (5 entries)
static CAMERA_SETTINGS_NOISE_FILTER_DATA: &[(&'static str, &'static str)] = &[
    ("-1 -2 1", "Low"),
    ("-2 -2 1", "Off"),
    ("0 -2 1", "Standard"),
    ("0 0 0", "n/a"),
    ("1 -2 1", "High"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_NOISE_FILTER: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| CAMERA_SETTINGS_NOISE_FILTER_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_camera_settings__noise_filter(key: &str) -> Option<&'static str> {
    CAMERA_SETTINGS_NOISE_FILTER.get(key).copied()
}

/// Raw data (4 entries)
static CAMERA_SETTINGS_PICTURE_MODE_EFFECT_DATA: &[(&'static str, &'static str)] = &[
    ("-1 -1 1", "Low"),
    ("0 -1 1", "Standard"),
    ("0 0 0", "n/a"),
    ("1 -1 1", "High"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_PICTURE_MODE_EFFECT: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        CAMERA_SETTINGS_PICTURE_MODE_EFFECT_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_camera_settings__picture_mode_effect(key: &str) -> Option<&'static str> {
    CAMERA_SETTINGS_PICTURE_MODE_EFFECT.get(key).copied()
}

/// Raw data (4 entries)
static CAMERA_SETTINGS_FILM_GRAIN_EFFECT_DATA: &[(u8, &'static str)] =
    &[(0, "Off"), (1, "Low"), (2, "Medium"), (3, "High")];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_FILM_GRAIN_EFFECT: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        CAMERA_SETTINGS_FILM_GRAIN_EFFECT_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_camera_settings__film_grain_effect(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS_FILM_GRAIN_EFFECT.get(&key).copied()
}

/// Raw data (6 entries)
static CAMERA_SETTINGS_MONOCHROME_COLOR_DATA: &[(u8, &'static str)] = &[
    (0, "(none)"),
    (1, "Normal"),
    (2, "Sepia"),
    (3, "Blue"),
    (4, "Purple"),
    (5, "Green"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_MONOCHROME_COLOR: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        CAMERA_SETTINGS_MONOCHROME_COLOR_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_camera_settings__monochrome_color(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS_MONOCHROME_COLOR.get(&key).copied()
}

/// Raw data (5 entries)
static CAMERA_SETTINGS_IMAGE_QUALITY2_DATA: &[(u8, &'static str)] =
    &[(1, "SQ"), (2, "HQ"), (3, "SHQ"), (4, "RAW"), (5, "SQ (5)")];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_IMAGE_QUALITY2: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        CAMERA_SETTINGS_IMAGE_QUALITY2_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_camera_settings__image_quality2(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS_IMAGE_QUALITY2.get(&key).copied()
}

/// Raw data (5 entries)
static CAMERA_SETTINGS_IMAGE_STABILIZATION_DATA: &[(u8, &'static str)] = &[
    (0, "Off"),
    (1, "On, Mode 1"),
    (2, "On, Mode 2"),
    (3, "On, Mode 3"),
    (4, "On, Mode 4"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_IMAGE_STABILIZATION: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        CAMERA_SETTINGS_IMAGE_STABILIZATION_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_camera_settings__image_stabilization(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS_IMAGE_STABILIZATION.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS_EXTENDED_W_B_DETECT_DATA: &[(u8, &'static str)] = &[(0, "Off"), (1, "On")];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_EXTENDED_W_B_DETECT: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        CAMERA_SETTINGS_EXTENDED_W_B_DETECT_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_camera_settings__extended_w_b_detect(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS_EXTENDED_W_B_DETECT.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS_PREVIEW_IMAGE_VALID_DATA: &[(u8, &'static str)] = &[(0, "No"), (1, "Yes")];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_PREVIEW_IMAGE_VALID: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        CAMERA_SETTINGS_PREVIEW_IMAGE_VALID_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_camera_settings__preview_image_valid(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS_PREVIEW_IMAGE_VALID.get(&key).copied()
}

/// Raw data (5 entries)
static CAMERA_SETTINGS_EXPOSURE_MODE_DATA: &[(u8, &'static str)] = &[
    (1, "Manual"),
    (2, "Program"),
    (3, "Aperture-priority AE"),
    (4, "Shutter speed priority AE"),
    (5, "Program-shift"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_EXPOSURE_MODE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| CAMERA_SETTINGS_EXPOSURE_MODE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_camera_settings__exposure_mode(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS_EXPOSURE_MODE.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS_A_E_LOCK_DATA: &[(u8, &'static str)] = &[(0, "Off"), (1, "On")];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_A_E_LOCK: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| CAMERA_SETTINGS_A_E_LOCK_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_camera_settings__a_e_lock(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS_A_E_LOCK.get(&key).copied()
}

/// Raw data (6 entries)
static CAMERA_SETTINGS_METERING_MODE_DATA: &[(u16, &'static str)] = &[
    (2, "Center-weighted average"),
    (3, "Spot"),
    (5, "ESP"),
    (261, "Pattern+AF"),
    (515, "Spot+Highlight control"),
    (1027, "Spot+Shadow control"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_METERING_MODE: LazyLock<HashMap<u16, &'static str>> =
    LazyLock::new(|| CAMERA_SETTINGS_METERING_MODE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_camera_settings__metering_mode(key: u16) -> Option<&'static str> {
    CAMERA_SETTINGS_METERING_MODE.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS_N_D_FILTER_DATA: &[(u8, &'static str)] = &[(0, "Off"), (1, "On")];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_N_D_FILTER: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| CAMERA_SETTINGS_N_D_FILTER_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_camera_settings__n_d_filter(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS_N_D_FILTER.get(&key).copied()
}

/// Raw data (3 entries)
static CAMERA_SETTINGS_MACRO_MODE_DATA: &[(u8, &'static str)] =
    &[(0, "Off"), (1, "On"), (2, "Super Macro")];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_MACRO_MODE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| CAMERA_SETTINGS_MACRO_MODE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_camera_settings__macro_mode(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS_MACRO_MODE.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS_A_F_SEARCH_DATA: &[(u8, &'static str)] = &[(0, "Not Ready"), (1, "Ready")];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_A_F_SEARCH: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| CAMERA_SETTINGS_A_F_SEARCH_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_camera_settings__a_f_search(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS_A_F_SEARCH.get(&key).copied()
}

/// Raw data (2 entries)
static CAMERA_SETTINGS_A_F_FINE_TUNE_DATA: &[(u8, &'static str)] = &[(0, "Off"), (1, "On")];

/// Lookup table (lazy-initialized)
pub static CAMERA_SETTINGS_A_F_FINE_TUNE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| CAMERA_SETTINGS_A_F_FINE_TUNE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_camera_settings__a_f_fine_tune(key: u8) -> Option<&'static str> {
    CAMERA_SETTINGS_A_F_FINE_TUNE.get(&key).copied()
}
