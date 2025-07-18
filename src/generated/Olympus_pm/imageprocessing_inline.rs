//! Inline PrintConv tables for ImageProcessing table
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Olympus.pm (table: ImageProcessing)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (1 entries)
static IMAGE_PROCESSING_NOISE_REDUCTION2_DATA: &[(u8, &'static str)] = &[(0, "(none)")];

/// Lookup table (lazy-initialized)
pub static IMAGE_PROCESSING_NOISE_REDUCTION2: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        IMAGE_PROCESSING_NOISE_REDUCTION2_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_image_processing__noise_reduction2(key: u8) -> Option<&'static str> {
    IMAGE_PROCESSING_NOISE_REDUCTION2.get(&key).copied()
}

/// Raw data (2 entries)
static IMAGE_PROCESSING_DISTORTION_CORRECTION2_DATA: &[(u8, &'static str)] =
    &[(0, "Off"), (1, "On")];

/// Lookup table (lazy-initialized)
pub static IMAGE_PROCESSING_DISTORTION_CORRECTION2: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        IMAGE_PROCESSING_DISTORTION_CORRECTION2_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_image_processing__distortion_correction2(key: u8) -> Option<&'static str> {
    IMAGE_PROCESSING_DISTORTION_CORRECTION2.get(&key).copied()
}

/// Raw data (2 entries)
static IMAGE_PROCESSING_SHADING_COMPENSATION2_DATA: &[(u8, &'static str)] =
    &[(0, "Off"), (1, "On")];

/// Lookup table (lazy-initialized)
pub static IMAGE_PROCESSING_SHADING_COMPENSATION2: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        IMAGE_PROCESSING_SHADING_COMPENSATION2_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_image_processing__shading_compensation2(key: u8) -> Option<&'static str> {
    IMAGE_PROCESSING_SHADING_COMPENSATION2.get(&key).copied()
}

/// Raw data (14 entries)
static IMAGE_PROCESSING_ASPECT_RATIO_DATA: &[(&'static str, &'static str)] = &[
    ("1 1", "4:3"),
    ("1 4", "1:1"),
    ("2 1", "3:2 (RAW)"),
    ("2 2", "3:2"),
    ("3 1", "16:9 (RAW)"),
    ("3 3", "16:9"),
    ("4 1", "1:1 (RAW)"),
    ("4 4", "6:6"),
    ("5 5", "5:4"),
    ("6 6", "7:6"),
    ("7 7", "6:5"),
    ("8 8", "7:5"),
    ("9 1", "3:4 (RAW)"),
    ("9 9", "3:4"),
];

/// Lookup table (lazy-initialized)
pub static IMAGE_PROCESSING_ASPECT_RATIO: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| IMAGE_PROCESSING_ASPECT_RATIO_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_image_processing__aspect_ratio(key: &str) -> Option<&'static str> {
    IMAGE_PROCESSING_ASPECT_RATIO.get(key).copied()
}

/// Raw data (2 entries)
static IMAGE_PROCESSING_KEYSTONE_COMPENSATION_DATA: &[(&'static str, &'static str)] =
    &[("0 0", "Off"), ("0 1", "On")];

/// Lookup table (lazy-initialized)
pub static IMAGE_PROCESSING_KEYSTONE_COMPENSATION: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        IMAGE_PROCESSING_KEYSTONE_COMPENSATION_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_image_processing__keystone_compensation(key: &str) -> Option<&'static str> {
    IMAGE_PROCESSING_KEYSTONE_COMPENSATION.get(key).copied()
}

/// Raw data (2 entries)
static IMAGE_PROCESSING_KEYSTONE_DIRECTION_DATA: &[(u8, &'static str)] =
    &[(0, "Vertical"), (1, "Horizontal")];

/// Lookup table (lazy-initialized)
pub static IMAGE_PROCESSING_KEYSTONE_DIRECTION: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        IMAGE_PROCESSING_KEYSTONE_DIRECTION_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_image_processing__keystone_direction(key: u8) -> Option<&'static str> {
    IMAGE_PROCESSING_KEYSTONE_DIRECTION.get(&key).copied()
}
