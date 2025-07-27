//! XML entity name to character code mappings
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/XMP.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (5 entries)
static XML_CHAR_NUMS_DATA: &[(&'static str, &'static str)] = &[
    ("amp", "38"),
    ("apos", "39"),
    ("gt", "62"),
    ("lt", "60"),
    ("quot", "34"),
];

/// Lookup table (lazy-initialized)
pub static XML_CHAR_NUMS: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| XML_CHAR_NUMS_DATA.iter().copied().collect());

/// Look up value by key
pub fn lookup_xml_char_nums(key: &str) -> Option<&'static str> {
    XML_CHAR_NUMS.get(key).copied()
}
