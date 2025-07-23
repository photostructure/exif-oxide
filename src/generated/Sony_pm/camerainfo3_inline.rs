//! Inline PrintConv tables for CameraInfo3 table
//! 
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Sony.pm (table: CameraInfo3)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (10 entries)
static CAMERA_INFO3_A_F_POINT_SELECTED_DATA: &[(u8, &'static str)] = &[
    (0, "Auto"),
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
pub static CAMERA_INFO3_A_F_POINT_SELECTED: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_INFO3_A_F_POINT_SELECTED_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_info3__a_f_point_selected(key: u8) -> Option<&'static str> {
    CAMERA_INFO3_A_F_POINT_SELECTED.get(&key).copied()
}

/// Raw data (4 entries)
static CAMERA_INFO3_FOCUS_MODE_DATA: &[(u8, &'static str)] = &[
    (0, "Manual"),
    (1, "AF-S"),
    (2, "AF-C"),
    (3, "AF-A"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_INFO3_FOCUS_MODE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_INFO3_FOCUS_MODE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_info3__focus_mode(key: u8) -> Option<&'static str> {
    CAMERA_INFO3_FOCUS_MODE.get(&key).copied()
}

/// Raw data (8 entries)
static CAMERA_INFO3_A_F_POINT_DATA: &[(u8, &'static str)] = &[
    (0, "Top-right"),
    (1, "Bottom-right"),
    (2, "Bottom"),
    (3, "Middle Horizontal"),
    (4, "Center Vertical"),
    (5, "Top"),
    (6, "Top-left"),
    (7, "Bottom-left"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_INFO3_A_F_POINT: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_INFO3_A_F_POINT_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_info3__a_f_point(key: u8) -> Option<&'static str> {
    CAMERA_INFO3_A_F_POINT.get(&key).copied()
}

/// Raw data (5 entries)
static CAMERA_INFO3_FOCUS_STATUS_DATA: &[(u8, &'static str)] = &[
    (0, "Manual - Not confirmed (0)"),
    (4, "Manual - Not confirmed (4)"),
    (16, "AF-C - Confirmed"),
    (24, "AF-C - Not Confirmed"),
    (64, "AF-S - Confirmed"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_INFO3_FOCUS_STATUS: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_INFO3_FOCUS_STATUS_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_info3__focus_status(key: u8) -> Option<&'static str> {
    CAMERA_INFO3_FOCUS_STATUS.get(&key).copied()
}

/// Raw data (16 entries)
static CAMERA_INFO3_A_F_POINT_SELECTED_28_DATA: &[(u8, &'static str)] = &[
    (0, "Auto"),
    (1, "Center"),
    (2, "Top"),
    (3, "Upper-right"),
    (4, "Right"),
    (5, "Lower-right"),
    (6, "Bottom"),
    (7, "Lower-left"),
    (8, "Left"),
    (9, "Upper-left"),
    (10, "Far Right"),
    (11, "Far Left"),
    (12, "Upper-middle"),
    (13, "Near Right"),
    (14, "Lower-middle"),
    (15, "Near Left"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_INFO3_A_F_POINT_SELECTED_28: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_INFO3_A_F_POINT_SELECTED_28_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_info3__a_f_point_selected_28(key: u8) -> Option<&'static str> {
    CAMERA_INFO3_A_F_POINT_SELECTED_28.get(&key).copied()
}

/// Raw data (19 entries)
static CAMERA_INFO3_A_F_POINT_32_DATA: &[(u8, &'static str)] = &[
    (0, "Upper-left"),
    (1, "Left"),
    (2, "Lower-left"),
    (3, "Far Left"),
    (4, "Top (horizontal)"),
    (5, "Near Right"),
    (6, "Center (horizontal)"),
    (7, "Near Left"),
    (8, "Bottom (horizontal)"),
    (9, "Top (vertical)"),
    (10, "Center (vertical)"),
    (11, "Bottom (vertical)"),
    (12, "Far Right"),
    (13, "Upper-right"),
    (14, "Right"),
    (15, "Lower-right"),
    (16, "Upper-middle"),
    (17, "Lower-middle"),
    (255, "(none)"),
];

/// Lookup table (lazy-initialized)
pub static CAMERA_INFO3_A_F_POINT_32: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    CAMERA_INFO3_A_F_POINT_32_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_camera_info3__a_f_point_32(key: u8) -> Option<&'static str> {
    CAMERA_INFO3_A_F_POINT_32.get(&key).copied()
}
