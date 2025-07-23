//! Canonical file extensions for file types
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/ExifTool.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (9 entries)
static FILE_TYPE_EXTENSIONS_DATA: &[(&'static str, &'static str)] = &[
    ("Canon 1D RAW", "tif"),
    ("DICOM", "dcm"),
    ("FLIR", "fff"),
    ("GZIP", "gz"),
    ("JPEG", "jpg"),
    ("M2TS", "mts"),
    ("MPEG", "mpg"),
    ("TIFF", "tif"),
    ("VCard", "vcf"),
];

/// Lookup table (lazy-initialized)
pub static FILE_TYPE_EXTENSIONS: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| FILE_TYPE_EXTENSIONS_DATA.iter().copied().collect());

/// Look up value by key
pub fn lookup_file_type_extensions(key: &str) -> Option<&'static str> {
    FILE_TYPE_EXTENSIONS.get(key).copied()
}
