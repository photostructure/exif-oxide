//! Nikon Z7/Z8/Z9 focus mode settings
//! 
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Nikon.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (4 entries)
static FOCUS_MODE_Z7_DATA: &[(u8, &'static str)] = &[
    (0, "Manual"),
    (1, "AF-S"),
    (2, "AF-C"),
    (4, "AF-F"),
];

/// Lookup table (lazy-initialized)
pub static FOCUS_MODE_Z7: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    FOCUS_MODE_Z7_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_focus_mode_z7(key: u8) -> Option<&'static str> {
    FOCUS_MODE_Z7.get(&key).copied()
}
