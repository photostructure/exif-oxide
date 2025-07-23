//! Sony ISO setting values to actual ISO numbers
//! 
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Sony.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (43 entries)
static SONY_ISO_SETTING_2010_DATA: &[(u8, &'static str)] = &[
    (0, "Auto"),
    (5, "25"),
    (7, "40"),
    (8, "50"),
    (9, "64"),
    (10, "80"),
    (11, "100"),
    (12, "125"),
    (13, "160"),
    (14, "200"),
    (15, "250"),
    (16, "320"),
    (17, "400"),
    (18, "500"),
    (19, "640"),
    (20, "800"),
    (21, "1000"),
    (22, "1250"),
    (23, "1600"),
    (24, "2000"),
    (25, "2500"),
    (26, "3200"),
    (27, "4000"),
    (28, "5000"),
    (29, "6400"),
    (30, "8000"),
    (31, "10000"),
    (32, "12800"),
    (33, "16000"),
    (34, "20000"),
    (35, "25600"),
    (36, "32000"),
    (37, "40000"),
    (38, "51200"),
    (39, "64000"),
    (40, "80000"),
    (41, "102400"),
    (42, "128000"),
    (43, "160000"),
    (44, "204800"),
    (45, "256000"),
    (46, "320000"),
    (47, "409600"),
];

/// Lookup table (lazy-initialized)
pub static SONY_ISO_SETTING_2010: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    SONY_ISO_SETTING_2010_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_sony_iso_setting_2010(key: u8) -> Option<&'static str> {
    SONY_ISO_SETTING_2010.get(&key).copied()
}
