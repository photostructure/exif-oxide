//! ExifTool XMP family 1 group name translations
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/XMP.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (5 entries)
static XMP_GROUP_NAMES_DATA: &[(&str, &str)] = &[
    ("getty", "GettyImagesGIFT"),
    ("iptcCore", "Iptc4xmpCore"),
    ("iptcExt", "Iptc4xmpExt"),
    ("microsoft", "MicrosoftPhoto"),
    ("photomech", "photomechanic"),
];

/// Lookup table (lazy-initialized)
pub static XMP_GROUP_NAMES: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| XMP_GROUP_NAMES_DATA.iter().copied().collect());

/// Look up value by key
pub fn lookup_xmp_group_names(key: &str) -> Option<&'static str> {
    XMP_GROUP_NAMES.get(key).copied()
}
