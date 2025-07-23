//! Inline PrintConv tables for AFConfig table
//! 
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Canon.pm (table: AFConfig)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (2 entries)
static A_F_CONFIG_AUTO_A_F_POINT_SEL_E_O_SI_T_R_A_F_DATA: &[(u8, &'static str)] = &[
    (0, "Enable"),
    (1, "Disable"),
];

/// Lookup table (lazy-initialized)
pub static A_F_CONFIG_AUTO_A_F_POINT_SEL_E_O_SI_T_R_A_F: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    A_F_CONFIG_AUTO_A_F_POINT_SEL_E_O_SI_T_R_A_F_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_a_f_config__auto_a_f_point_sel_e_o_si_t_r_a_f(key: u8) -> Option<&'static str> {
    A_F_CONFIG_AUTO_A_F_POINT_SEL_E_O_SI_T_R_A_F.get(&key).copied()
}

/// Raw data (2 entries)
static A_F_CONFIG_LENS_DRIVE_WHEN_A_F_IMPOSSIBLE_DATA: &[(u8, &'static str)] = &[
    (0, "Continue Focus Search"),
    (1, "Stop Focus Search"),
];

/// Lookup table (lazy-initialized)
pub static A_F_CONFIG_LENS_DRIVE_WHEN_A_F_IMPOSSIBLE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    A_F_CONFIG_LENS_DRIVE_WHEN_A_F_IMPOSSIBLE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_a_f_config__lens_drive_when_a_f_impossible(key: u8) -> Option<&'static str> {
    A_F_CONFIG_LENS_DRIVE_WHEN_A_F_IMPOSSIBLE.get(&key).copied()
}

/// Raw data (2 entries)
static A_F_CONFIG_A_F_AREA_SELECTION_METHOD_DATA: &[(u8, &'static str)] = &[
    (0, "M-Fn Button"),
    (1, "Main Dial"),
];

/// Lookup table (lazy-initialized)
pub static A_F_CONFIG_A_F_AREA_SELECTION_METHOD: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    A_F_CONFIG_A_F_AREA_SELECTION_METHOD_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_a_f_config__a_f_area_selection_method(key: u8) -> Option<&'static str> {
    A_F_CONFIG_A_F_AREA_SELECTION_METHOD.get(&key).copied()
}

/// Raw data (3 entries)
static A_F_CONFIG_ORIENTATION_LINKED_A_F_DATA: &[(u8, &'static str)] = &[
    (0, "Same for Vert/Horiz Points"),
    (1, "Separate Vert/Horiz Points"),
    (2, "Separate Area+Points"),
];

/// Lookup table (lazy-initialized)
pub static A_F_CONFIG_ORIENTATION_LINKED_A_F: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    A_F_CONFIG_ORIENTATION_LINKED_A_F_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_a_f_config__orientation_linked_a_f(key: u8) -> Option<&'static str> {
    A_F_CONFIG_ORIENTATION_LINKED_A_F.get(&key).copied()
}

/// Raw data (2 entries)
static A_F_CONFIG_MANUAL_A_F_POINT_SEL_PATTERN_DATA: &[(u8, &'static str)] = &[
    (0, "Stops at AF Area Edges"),
    (1, "Continuous"),
];

/// Lookup table (lazy-initialized)
pub static A_F_CONFIG_MANUAL_A_F_POINT_SEL_PATTERN: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    A_F_CONFIG_MANUAL_A_F_POINT_SEL_PATTERN_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_a_f_config__manual_a_f_point_sel_pattern(key: u8) -> Option<&'static str> {
    A_F_CONFIG_MANUAL_A_F_POINT_SEL_PATTERN.get(&key).copied()
}

/// Raw data (5 entries)
static A_F_CONFIG_A_F_POINT_DISPLAY_DURING_FOCUS_DATA: &[(u8, &'static str)] = &[
    (0, "Selected (constant)"),
    (1, "All (constant)"),
    (2, "Selected (pre-AF, focused)"),
    (3, "Selected (focused)"),
    (4, "Disabled"),
];

/// Lookup table (lazy-initialized)
pub static A_F_CONFIG_A_F_POINT_DISPLAY_DURING_FOCUS: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    A_F_CONFIG_A_F_POINT_DISPLAY_DURING_FOCUS_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_a_f_config__a_f_point_display_during_focus(key: u8) -> Option<&'static str> {
    A_F_CONFIG_A_F_POINT_DISPLAY_DURING_FOCUS.get(&key).copied()
}

/// Raw data (3 entries)
static A_F_CONFIG_V_F_DISPLAY_ILLUMINATION_DATA: &[(u8, &'static str)] = &[
    (0, "Auto"),
    (1, "Enable"),
    (2, "Disable"),
];

/// Lookup table (lazy-initialized)
pub static A_F_CONFIG_V_F_DISPLAY_ILLUMINATION: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    A_F_CONFIG_V_F_DISPLAY_ILLUMINATION_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_a_f_config__v_f_display_illumination(key: u8) -> Option<&'static str> {
    A_F_CONFIG_V_F_DISPLAY_ILLUMINATION.get(&key).copied()
}

/// Raw data (2 entries)
static A_F_CONFIG_A_F_STATUS_VIEWFINDER_DATA: &[(u8, &'static str)] = &[
    (0, "Show in Field of View"),
    (1, "Show Outside View"),
];

/// Lookup table (lazy-initialized)
pub static A_F_CONFIG_A_F_STATUS_VIEWFINDER: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    A_F_CONFIG_A_F_STATUS_VIEWFINDER_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_a_f_config__a_f_status_viewfinder(key: u8) -> Option<&'static str> {
    A_F_CONFIG_A_F_STATUS_VIEWFINDER.get(&key).copied()
}

/// Raw data (3 entries)
static A_F_CONFIG_INITIAL_A_F_POINT_IN_SERVO_DATA: &[(u8, &'static str)] = &[
    (0, "Initial AF Point Selected"),
    (1, "Manual AF Point"),
    (2, "Auto"),
];

/// Lookup table (lazy-initialized)
pub static A_F_CONFIG_INITIAL_A_F_POINT_IN_SERVO: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    A_F_CONFIG_INITIAL_A_F_POINT_IN_SERVO_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_a_f_config__initial_a_f_point_in_servo(key: u8) -> Option<&'static str> {
    A_F_CONFIG_INITIAL_A_F_POINT_IN_SERVO.get(&key).copied()
}

/// Raw data (4 entries)
static A_F_CONFIG_SUBJECT_TO_DETECT_DATA: &[(u8, &'static str)] = &[
    (0, "None"),
    (1, "People"),
    (2, "Animals"),
    (3, "Vehicles"),
];

/// Lookup table (lazy-initialized)
pub static A_F_CONFIG_SUBJECT_TO_DETECT: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    A_F_CONFIG_SUBJECT_TO_DETECT_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_a_f_config__subject_to_detect(key: u8) -> Option<&'static str> {
    A_F_CONFIG_SUBJECT_TO_DETECT.get(&key).copied()
}

/// Raw data (2 entries)
static A_F_CONFIG_EYE_DETECTION_DATA: &[(u8, &'static str)] = &[
    (0, "Off"),
    (1, "On"),
];

/// Lookup table (lazy-initialized)
pub static A_F_CONFIG_EYE_DETECTION: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    A_F_CONFIG_EYE_DETECTION_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_a_f_config__eye_detection(key: u8) -> Option<&'static str> {
    A_F_CONFIG_EYE_DETECTION.get(&key).copied()
}

/// Raw data (3 entries)
static A_F_CONFIG_A_I_SERVO_FIRST_IMAGE_DATA: &[(u8, &'static str)] = &[
    (0, "Equal Priority"),
    (1, "Release Priority"),
    (2, "Focus Priority"),
];

/// Lookup table (lazy-initialized)
pub static A_F_CONFIG_A_I_SERVO_FIRST_IMAGE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    A_F_CONFIG_A_I_SERVO_FIRST_IMAGE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_a_f_config__a_i_servo_first_image(key: u8) -> Option<&'static str> {
    A_F_CONFIG_A_I_SERVO_FIRST_IMAGE.get(&key).copied()
}

/// Raw data (5 entries)
static A_F_CONFIG_A_I_SERVO_SECOND_IMAGE_DATA: &[(u8, &'static str)] = &[
    (0, "Equal Priority"),
    (1, "Release Priority"),
    (2, "Focus Priority"),
    (3, "Release High Priority"),
    (4, "Focus High Priority"),
];

/// Lookup table (lazy-initialized)
pub static A_F_CONFIG_A_I_SERVO_SECOND_IMAGE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    A_F_CONFIG_A_I_SERVO_SECOND_IMAGE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_a_f_config__a_i_servo_second_image(key: u8) -> Option<&'static str> {
    A_F_CONFIG_A_I_SERVO_SECOND_IMAGE.get(&key).copied()
}

/// Raw data (4 entries)
static A_F_CONFIG_A_F_ASSIST_BEAM_DATA: &[(u8, &'static str)] = &[
    (0, "Enable"),
    (1, "Disable"),
    (2, "IR AF Assist Beam Only"),
    (3, "LED AF Assist Beam Only"),
];

/// Lookup table (lazy-initialized)
pub static A_F_CONFIG_A_F_ASSIST_BEAM: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    A_F_CONFIG_A_F_ASSIST_BEAM_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_a_f_config__a_f_assist_beam(key: u8) -> Option<&'static str> {
    A_F_CONFIG_A_F_ASSIST_BEAM.get(&key).copied()
}

/// Raw data (2 entries)
static A_F_CONFIG_ONE_SHOT_A_F_RELEASE_DATA: &[(u8, &'static str)] = &[
    (0, "Focus Priority"),
    (1, "Release Priority"),
];

/// Lookup table (lazy-initialized)
pub static A_F_CONFIG_ONE_SHOT_A_F_RELEASE: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    A_F_CONFIG_ONE_SHOT_A_F_RELEASE_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_a_f_config__one_shot_a_f_release(key: u8) -> Option<&'static str> {
    A_F_CONFIG_ONE_SHOT_A_F_RELEASE.get(&key).copied()
}
