//! Inline PrintConv tables for FileInfo table
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Canon.pm (table: FileInfo)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (5 entries)
static FILE_INFO_FILTER_EFFECT_DATA: &[(u8, &'static str)] = &[
    (0, "None"),
    (1, "Yellow"),
    (2, "Orange"),
    (3, "Red"),
    (4, "Green"),
];

/// Lookup table (lazy-initialized)
pub static FILE_INFO_FILTER_EFFECT: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| FILE_INFO_FILTER_EFFECT_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_file_info__filter_effect(key: u8) -> Option<&'static str> {
    FILE_INFO_FILTER_EFFECT.get(&key).copied()
}

/// Raw data (5 entries)
static FILE_INFO_TONING_EFFECT_DATA: &[(u8, &'static str)] = &[
    (0, "None"),
    (1, "Sepia"),
    (2, "Blue"),
    (3, "Purple"),
    (4, "Green"),
];

/// Lookup table (lazy-initialized)
pub static FILE_INFO_TONING_EFFECT: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| FILE_INFO_TONING_EFFECT_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_file_info__toning_effect(key: u8) -> Option<&'static str> {
    FILE_INFO_TONING_EFFECT.get(&key).copied()
}

/// Raw data (2 entries)
static FILE_INFO_LIVE_VIEW_SHOOTING_DATA: &[(u8, &'static str)] = &[(0, "Off"), (1, "On")];

/// Lookup table (lazy-initialized)
pub static FILE_INFO_LIVE_VIEW_SHOOTING: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| FILE_INFO_LIVE_VIEW_SHOOTING_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_file_info__live_view_shooting(key: u8) -> Option<&'static str> {
    FILE_INFO_LIVE_VIEW_SHOOTING.get(&key).copied()
}

/// Raw data (3 entries)
static FILE_INFO_SHUTTER_MODE_DATA: &[(u8, &'static str)] = &[
    (0, "Mechanical"),
    (1, "Electronic First Curtain"),
    (2, "Electronic"),
];

/// Lookup table (lazy-initialized)
pub static FILE_INFO_SHUTTER_MODE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| FILE_INFO_SHUTTER_MODE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_file_info__shutter_mode(key: u8) -> Option<&'static str> {
    FILE_INFO_SHUTTER_MODE.get(&key).copied()
}

/// Raw data (2 entries)
static FILE_INFO_FLASH_EXPOSURE_LOCK_DATA: &[(u8, &'static str)] = &[(0, "Off"), (1, "On")];

/// Lookup table (lazy-initialized)
pub static FILE_INFO_FLASH_EXPOSURE_LOCK: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| FILE_INFO_FLASH_EXPOSURE_LOCK_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_file_info__flash_exposure_lock(key: u8) -> Option<&'static str> {
    FILE_INFO_FLASH_EXPOSURE_LOCK.get(&key).copied()
}

/// Raw data (5 entries)
static FILE_INFO_BRACKET_MODE_DATA: &[(u8, &'static str)] =
    &[(0, "Off"), (1, "AEB"), (2, "FEB"), (3, "ISO"), (4, "WB")];

/// Lookup table (lazy-initialized)
pub static FILE_INFO_BRACKET_MODE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| FILE_INFO_BRACKET_MODE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_file_info__bracket_mode(key: u8) -> Option<&'static str> {
    FILE_INFO_BRACKET_MODE.get(&key).copied()
}

/// Raw data (2 entries)
static FILE_INFO_ANTI_FLICKER_DATA: &[(u8, &'static str)] = &[(0, "Off"), (1, "On")];

/// Lookup table (lazy-initialized)
pub static FILE_INFO_ANTI_FLICKER: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| FILE_INFO_ANTI_FLICKER_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_file_info__anti_flicker(key: u8) -> Option<&'static str> {
    FILE_INFO_ANTI_FLICKER.get(&key).copied()
}

/// Raw data (9 entries)
static FILE_INFO_RAW_JPG_QUALITY_DATA: &[(i16, &'static str)] = &[
    (-1, "n/a"),
    (1, "Economy"),
    (2, "Normal"),
    (3, "Fine"),
    (4, "RAW"),
    (5, "Superfine"),
    (7, "CRAW"),
    (130, "Light (RAW)"),
    (131, "Standard (RAW)"),
];

/// Lookup table (lazy-initialized)
pub static FILE_INFO_RAW_JPG_QUALITY: LazyLock<HashMap<i16, &'static str>> =
    LazyLock::new(|| FILE_INFO_RAW_JPG_QUALITY_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_file_info__raw_jpg_quality(key: i16) -> Option<&'static str> {
    FILE_INFO_RAW_JPG_QUALITY.get(&key).copied()
}

/// Raw data (71 entries)
static FILE_INFO_R_F_LENS_TYPE_DATA: &[(u16, &'static str)] = &[
    (0, "n/a"),
    (257, "Canon RF 50mm F1.2L USM"),
    (258, "Canon RF 24-105mm F4L IS USM"),
    (259, "Canon RF 28-70mm F2L USM"),
    (260, "Canon RF 35mm F1.8 MACRO IS STM"),
    (261, "Canon RF 85mm F1.2L USM"),
    (262, "Canon RF 85mm F1.2L USM DS"),
    (263, "Canon RF 24-70mm F2.8L IS USM"),
    (264, "Canon RF 15-35mm F2.8L IS USM"),
    (265, "Canon RF 24-240mm F4-6.3 IS USM"),
    (266, "Canon RF 70-200mm F2.8L IS USM"),
    (267, "Canon RF 85mm F2 MACRO IS STM"),
    (268, "Canon RF 600mm F11 IS STM"),
    (269, "Canon RF 600mm F11 IS STM + RF1.4x"),
    (270, "Canon RF 600mm F11 IS STM + RF2x"),
    (271, "Canon RF 800mm F11 IS STM"),
    (272, "Canon RF 800mm F11 IS STM + RF1.4x"),
    (273, "Canon RF 800mm F11 IS STM + RF2x"),
    (274, "Canon RF 24-105mm F4-7.1 IS STM"),
    (275, "Canon RF 100-500mm F4.5-7.1L IS USM"),
    (276, "Canon RF 100-500mm F4.5-7.1L IS USM + RF1.4x"),
    (277, "Canon RF 100-500mm F4.5-7.1L IS USM + RF2x"),
    (278, "Canon RF 70-200mm F4L IS USM"),
    (279, "Canon RF 100mm F2.8L MACRO IS USM"),
    (280, "Canon RF 50mm F1.8 STM"),
    (281, "Canon RF 14-35mm F4L IS USM"),
    (282, "Canon RF-S 18-45mm F4.5-6.3 IS STM"),
    (283, "Canon RF 100-400mm F5.6-8 IS USM"),
    (284, "Canon RF 100-400mm F5.6-8 IS USM + RF1.4x"),
    (285, "Canon RF 100-400mm F5.6-8 IS USM + RF2x"),
    (286, "Canon RF-S 18-150mm F3.5-6.3 IS STM"),
    (287, "Canon RF 24mm F1.8 MACRO IS STM"),
    (288, "Canon RF 16mm F2.8 STM"),
    (289, "Canon RF 400mm F2.8L IS USM"),
    (290, "Canon RF 400mm F2.8L IS USM + RF1.4x"),
    (291, "Canon RF 400mm F2.8L IS USM + RF2x"),
    (292, "Canon RF 600mm F4L IS USM"),
    (293, "Canon RF 600mm F4L IS USM + RF1.4x"),
    (294, "Canon RF 600mm F4L IS USM + RF2x"),
    (295, "Canon RF 800mm F5.6L IS USM"),
    (296, "Canon RF 800mm F5.6L IS USM + RF1.4x"),
    (297, "Canon RF 800mm F5.6L IS USM + RF2x"),
    (298, "Canon RF 1200mm F8L IS USM"),
    (299, "Canon RF 1200mm F8L IS USM + RF1.4x"),
    (300, "Canon RF 1200mm F8L IS USM + RF2x"),
    (301, "Canon RF 5.2mm F2.8L Dual Fisheye 3D VR"),
    (302, "Canon RF 15-30mm F4.5-6.3 IS STM"),
    (303, "Canon RF 135mm F1.8 L IS USM"),
    (304, "Canon RF 24-50mm F4.5-6.3 IS STM"),
    (305, "Canon RF-S 55-210mm F5-7.1 IS STM"),
    (306, "Canon RF 100-300mm F2.8L IS USM"),
    (307, "Canon RF 100-300mm F2.8L IS USM + RF1.4x"),
    (308, "Canon RF 100-300mm F2.8L IS USM + RF2x"),
    (309, "Canon RF 200-800mm F6.3-9 IS USM"),
    (310, "Canon RF 200-800mm F6.3-9 IS USM + RF1.4x"),
    (311, "Canon RF 200-800mm F6.3-9 IS USM + RF2x"),
    (312, "Canon RF 10-20mm F4 L IS STM"),
    (313, "Canon RF 28mm F2.8 STM"),
    (314, "Canon RF 24-105mm F2.8 L IS USM Z"),
    (315, "Canon RF-S 10-18mm F4.5-6.3 IS STM"),
    (316, "Canon RF 35mm F1.4 L VCM"),
    (317, "Canon RF-S 3.9mm F3.5 STM DUAL FISHEYE"),
    (318, "Canon RF 28-70mm F2.8 IS STM"),
    (319, "Canon RF 70-200mm F2.8 L IS USM Z"),
    (320, "Canon RF 70-200mm F2.8 L IS USM Z + RF1.4x"),
    (321, "Canon RF 70-200mm F2.8 L IS USM Z + RF2x"),
    (323, "Canon RF 16-28mm F2.8 IS STM"),
    (324, "Canon RF-S 14-30mm F4-6.3 IS STM PZ"),
    (325, "Canon RF 50mm F1.4 L VCM"),
    (326, "Canon RF 24mm F1.4 L VCM"),
    (327, "Canon RF 20mm F1.4 L VCM"),
];

/// Lookup table (lazy-initialized)
pub static FILE_INFO_R_F_LENS_TYPE: LazyLock<HashMap<u16, &'static str>> =
    LazyLock::new(|| FILE_INFO_R_F_LENS_TYPE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_file_info__r_f_lens_type(key: u16) -> Option<&'static str> {
    FILE_INFO_R_F_LENS_TYPE.get(&key).copied()
}

/// Raw data (19 entries)
static FILE_INFO_RAW_JPG_SIZE_DATA: &[(i16, &'static str)] = &[
    (-1, "n/a"),
    (0, "Large"),
    (1, "Medium"),
    (2, "Small"),
    (5, "Medium 1"),
    (6, "Medium 2"),
    (7, "Medium 3"),
    (8, "Postcard"),
    (9, "Widescreen"),
    (10, "Medium Widescreen"),
    (14, "Small 1"),
    (15, "Small 2"),
    (16, "Small 3"),
    (128, "640x480 Movie"),
    (129, "Medium Movie"),
    (130, "Small Movie"),
    (137, "1280x720 Movie"),
    (142, "1920x1080 Movie"),
    (143, "4096x2160 Movie"),
];

/// Lookup table (lazy-initialized)
pub static FILE_INFO_RAW_JPG_SIZE: LazyLock<HashMap<i16, &'static str>> =
    LazyLock::new(|| FILE_INFO_RAW_JPG_SIZE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_file_info__raw_jpg_size(key: i16) -> Option<&'static str> {
    FILE_INFO_RAW_JPG_SIZE.get(&key).copied()
}

/// Raw data (4 entries)
static FILE_INFO_LONG_EXPOSURE_NOISE_REDUCTION2_DATA: &[(u8, &'static str)] =
    &[(0, "Off"), (1, "On (1D)"), (3, "On"), (4, "Auto")];

/// Lookup table (lazy-initialized)
pub static FILE_INFO_LONG_EXPOSURE_NOISE_REDUCTION2: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        FILE_INFO_LONG_EXPOSURE_NOISE_REDUCTION2_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_file_info__long_exposure_noise_reduction2(key: u8) -> Option<&'static str> {
    FILE_INFO_LONG_EXPOSURE_NOISE_REDUCTION2.get(&key).copied()
}

/// Raw data (3 entries)
static FILE_INFO_W_B_BRACKET_MODE_DATA: &[(u8, &'static str)] =
    &[(0, "Off"), (1, "On (shift AB)"), (2, "On (shift GM)")];

/// Lookup table (lazy-initialized)
pub static FILE_INFO_W_B_BRACKET_MODE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| FILE_INFO_W_B_BRACKET_MODE_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_file_info__w_b_bracket_mode(key: u8) -> Option<&'static str> {
    FILE_INFO_W_B_BRACKET_MODE.get(&key).copied()
}
