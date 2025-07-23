//! EXIF Flash tag PrintConv values
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Exif.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (27 entries)
static FLASH_DATA: &[(u16, &'static str)] = &[
    (0, "No Flash"),
    (1, "Fired"),
    (5, "Fired, Return not detected"),
    (7, "Fired, Return detected"),
    (8, "On, Did not fire"),
    (9, "On, Fired"),
    (13, "On, Return not detected"),
    (15, "On, Return detected"),
    (16, "Off, Did not fire"),
    (20, "Off, Did not fire, Return not detected"),
    (24, "Auto, Did not fire"),
    (25, "Auto, Fired"),
    (29, "Auto, Fired, Return not detected"),
    (31, "Auto, Fired, Return detected"),
    (32, "No flash function"),
    (48, "Off, No flash function"),
    (65, "Fired, Red-eye reduction"),
    (69, "Fired, Red-eye reduction, Return not detected"),
    (71, "Fired, Red-eye reduction, Return detected"),
    (73, "On, Red-eye reduction"),
    (77, "On, Red-eye reduction, Return not detected"),
    (79, "On, Red-eye reduction, Return detected"),
    (80, "Off, Red-eye reduction"),
    (88, "Auto, Did not fire, Red-eye reduction"),
    (89, "Auto, Fired, Red-eye reduction"),
    (93, "Auto, Fired, Red-eye reduction, Return not detected"),
    (95, "Auto, Fired, Red-eye reduction, Return detected"),
];

/// Lookup table (lazy-initialized)
pub static FLASH: LazyLock<HashMap<u16, &'static str>> =
    LazyLock::new(|| FLASH_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_flash(key: u16) -> Option<&'static str> {
    FLASH.get(&key).copied()
}
