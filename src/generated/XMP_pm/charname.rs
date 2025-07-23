//! XML character to entity name mappings
//! 
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/XMP.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (5 entries)
static XML_CHAR_NAMES_DATA: &[(&'static str, &'static str)] = &[
    ("\"", "quot"),
    ("&", "amp"),
    ("'", "#39"),
    ("<", "lt"),
    (">", "gt"),
];

/// Lookup table (lazy-initialized)
pub static XML_CHAR_NAMES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    XML_CHAR_NAMES_DATA.iter().copied().collect()
});

/// Look up value by key
pub fn lookup_xml_char_names(key: &str) -> Option<&'static str> {
    XML_CHAR_NAMES.get(key).copied()
}
