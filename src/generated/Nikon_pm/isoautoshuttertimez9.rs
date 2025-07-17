//! Z9 ISO auto shutter time mappings
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Nikon.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (50 entries)
static ISO_AUTO_SHUTTER_TIME_Z9_DATA: &[(&'static str, &'static str)] = &[
    ("-12", "15 s"),
    ("-15", "Auto"),
    ("-3", "2 s"),
    ("-6", "4 s"),
    ("-9", "8 s"),
    ("0", "1 s"),
    ("1", "1/1.3 s"),
    ("10", "1/10 s"),
    ("11", "1/13 s"),
    ("12", "1/15 s"),
    ("13", "1/20 s"),
    ("14", "1/25 s"),
    ("15", "1/30 s"),
    ("16", "1/40 s"),
    ("17", "1/50 s"),
    ("18", "1/60 s"),
    ("19", "1/80 s"),
    ("2", "1/1.6 s"),
    ("20", "1/100 s"),
    ("21", "1/120 s"),
    ("22", "1/160 s"),
    ("23", "1/200 s"),
    ("24", "1/250 s"),
    ("25", "1/320 s"),
    ("26", "1/400 s"),
    ("27", "1/500 s"),
    ("28", "1/640 s"),
    ("29", "1/800 s"),
    ("3", "1/2 s"),
    ("30", "1/1000 s"),
    ("31", "1/1250 s"),
    ("32", "1/1600 s"),
    ("33", "1/2000 s"),
    ("34", "1/2500 s"),
    ("35", "1/3200 s"),
    ("36", "1/4000 s"),
    ("37", "1/5000 s"),
    ("37.5", "1/6000 s"),
    ("38", "1/6400 s"),
    ("39", "1/8000 s"),
    ("4", "1/2.5 s"),
    ("40", "1/10000 s"),
    ("40.5", "1/12000 s"),
    ("41", "1/13000 s"),
    ("42", "1/16000 s"),
    ("5", "1/3 s"),
    ("6", "1/4 s"),
    ("7", "1/5 s"),
    ("8", "1/6 s"),
    ("9", "1/8 s"),
];

/// Lookup table (lazy-initialized)
pub static ISO_AUTO_SHUTTER_TIME_Z9: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| ISO_AUTO_SHUTTER_TIME_Z9_DATA.iter().copied().collect());

/// Look up value by key
pub fn lookup_iso_auto_shutter_time_z9(key: &str) -> Option<&'static str> {
    ISO_AUTO_SHUTTER_TIME_Z9.get(key).copied()
}
