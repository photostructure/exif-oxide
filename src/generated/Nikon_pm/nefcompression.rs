//! Nikon NEF (RAW) compression modes
//! 
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Nikon.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (12 entries)
static NEF_COMPRESSION_DATA: &[(u8, &'static str)] = &[
    (1, "Lossy (type 1)"),
    (2, "Uncompressed"),
    (3, "Lossless"),
    (4, "Lossy (type 2)"),
    (5, "Striped packed 12 bits"),
    (6, "Uncompressed (reduced to 12 bit)"),
    (7, "Unpacked 12 bits"),
    (8, "Small"),
    (9, "Packed 12 bits"),
    (10, "Packed 14 bits"),
    (13, "High Efficiency"),
    (14, "High Efficiency*"),
];

/// Lookup table (lazy-initialized)
pub static NEF_COMPRESSION: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    NEF_COMPRESSION_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_nef_compression(key: u8) -> Option<&'static str> {
    NEF_COMPRESSION.get(&key).copied()
}
