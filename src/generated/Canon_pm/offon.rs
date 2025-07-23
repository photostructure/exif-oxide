//! Simple Off/On mapping used throughout Canon tags
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Canon.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (2 entries)
static OFF_ON_DATA: &[(u8, &'static str)] = &[(0, "Off"), (1, "On")];

/// Lookup table (lazy-initialized)
pub static OFF_ON: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| OFF_ON_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_off_on(key: u8) -> Option<&'static str> {
    OFF_ON.get(&key).copied()
}
