//! Nikon Z7/Z8/Z9 metering mode settings
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Nikon.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (4 entries)
static METERING_MODE_Z7_DATA: &[(u8, &'static str)] =
    &[(0, "Matrix"), (1, "Center"), (2, "Spot"), (3, "Highlight")];

/// Lookup table (lazy-initialized)
pub static METERING_MODE_Z7: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| METERING_MODE_Z7_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_metering_mode_z7(key: u8) -> Option<&'static str> {
    METERING_MODE_Z7.get(&key).copied()
}
