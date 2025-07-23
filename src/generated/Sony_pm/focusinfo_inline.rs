//! Inline PrintConv tables for FocusInfo table
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Olympus.pm (table: FocusInfo)
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (2 entries)
static FOCUS_INFO_EXTERNAL_FLASH_DATA: &[(&'static str, &'static str)] =
    &[("0 0", "Off"), ("1 0", "On")];

/// Lookup table (lazy-initialized)
pub static FOCUS_INFO_EXTERNAL_FLASH: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| FOCUS_INFO_EXTERNAL_FLASH_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_focus_info__external_flash(key: &str) -> Option<&'static str> {
    FOCUS_INFO_EXTERNAL_FLASH.get(key).copied()
}

/// Raw data (2 entries)
static FOCUS_INFO_EXTERNAL_FLASH_BOUNCE_DATA: &[(u8, &'static str)] =
    &[(0, "Bounce or Off"), (1, "Direct")];

/// Lookup table (lazy-initialized)
pub static FOCUS_INFO_EXTERNAL_FLASH_BOUNCE: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| {
        FOCUS_INFO_EXTERNAL_FLASH_BOUNCE_DATA
            .iter()
            .cloned()
            .collect()
    });

/// Look up value by key
pub fn lookup_focus_info__external_flash_bounce(key: u8) -> Option<&'static str> {
    FOCUS_INFO_EXTERNAL_FLASH_BOUNCE.get(&key).copied()
}

/// Raw data (4 entries)
static FOCUS_INFO_INTERNAL_FLASH_DATA: &[(&'static str, &'static str)] =
    &[("0", "Off"), ("0 0", "Off"), ("1", "On"), ("1 0", "On")];

/// Lookup table (lazy-initialized)
pub static FOCUS_INFO_INTERNAL_FLASH: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| FOCUS_INFO_INTERNAL_FLASH_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_focus_info__internal_flash(key: &str) -> Option<&'static str> {
    FOCUS_INFO_INTERNAL_FLASH.get(key).copied()
}

/// Raw data (2 entries)
static FOCUS_INFO_MACRO_L_E_D_DATA: &[(u8, &'static str)] = &[(0, "Off"), (1, "On")];

/// Lookup table (lazy-initialized)
pub static FOCUS_INFO_MACRO_L_E_D: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| FOCUS_INFO_MACRO_L_E_D_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_focus_info__macro_l_e_d(key: u8) -> Option<&'static str> {
    FOCUS_INFO_MACRO_L_E_D.get(&key).copied()
}

/// Raw data (2 entries)
static FOCUS_INFO_AUTO_FOCUS_DATA: &[(u8, &'static str)] = &[(0, "Off"), (1, "On")];

/// Lookup table (lazy-initialized)
pub static FOCUS_INFO_AUTO_FOCUS: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| FOCUS_INFO_AUTO_FOCUS_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_focus_info__auto_focus(key: u8) -> Option<&'static str> {
    FOCUS_INFO_AUTO_FOCUS.get(&key).copied()
}
