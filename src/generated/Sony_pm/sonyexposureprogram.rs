//! Sony exposure program modes
//! 
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Sony.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (14 entries)
static SONY_EXPOSURE_PROGRAM_DATA: &[(u8, &'static str)] = &[
    (0, "Auto"),
    (1, "Manual"),
    (2, "Program AE"),
    (3, "Aperture-priority AE"),
    (4, "Shutter speed priority AE"),
    (8, "Program Shift A"),
    (9, "Program Shift S"),
    (16, "Portrait"),
    (17, "Sports"),
    (18, "Sunset"),
    (19, "Night Portrait"),
    (20, "Landscape"),
    (21, "Macro"),
    (35, "Auto No Flash"),
];

/// Lookup table (lazy-initialized)
pub static SONY_EXPOSURE_PROGRAM: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    SONY_EXPOSURE_PROGRAM_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_sony_exposure_program(key: u8) -> Option<&'static str> {
    SONY_EXPOSURE_PROGRAM.get(&key).copied()
}
