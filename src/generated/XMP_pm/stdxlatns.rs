//! Standard XMP namespace prefix translations
//! 
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/XMP.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (7 entries)
static STD_XLAT_NS_DATA: &[(&'static str, &'static str)] = &[
    ("GettyImagesGIFT", "getty"),
    ("Iptc4xmpCore", "iptcCore"),
    ("Iptc4xmpExt", "iptcExt"),
    ("MicrosoftPhoto", "microsoft"),
    ("hdr_metadata", "hdr"),
    ("photomechanic", "photomech"),
    ("prismusagerights", "pur"),
];

/// Lookup table (lazy-initialized)
pub static STD_XLAT_NS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    STD_XLAT_NS_DATA.iter().copied().collect()
});

/// Look up value by key
pub fn lookup_std_xlat_ns(key: &str) -> Option<&'static str> {
    STD_XLAT_NS.get(key).copied()
}
