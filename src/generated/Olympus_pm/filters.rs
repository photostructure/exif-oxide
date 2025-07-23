//! Olympus art filter modes
//! 
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Olympus.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (42 entries)
static OLYMPUS_FILTERS_DATA: &[(u8, &'static str)] = &[
    (0, "Off"),
    (1, "Soft Focus"),
    (2, "Pop Art"),
    (3, "Pale & Light Color"),
    (4, "Light Tone"),
    (5, "Pin Hole"),
    (6, "Grainy Film"),
    (8, "Underwater"),
    (9, "Diorama"),
    (10, "Cross Process"),
    (12, "Fish Eye"),
    (13, "Drawing"),
    (14, "Gentle Sepia"),
    (15, "Pale & Light Color II"),
    (16, "Pop Art II"),
    (17, "Pin Hole II"),
    (18, "Pin Hole III"),
    (19, "Grainy Film II"),
    (20, "Dramatic Tone"),
    (21, "Punk"),
    (22, "Soft Focus 2"),
    (23, "Sparkle"),
    (24, "Watercolor"),
    (25, "Key Line"),
    (26, "Key Line II"),
    (27, "Miniature"),
    (28, "Reflection"),
    (29, "Fragmented"),
    (31, "Cross Process II"),
    (32, "Dramatic Tone II"),
    (33, "Watercolor I"),
    (34, "Watercolor II"),
    (35, "Diorama II"),
    (36, "Vintage"),
    (37, "Vintage II"),
    (38, "Vintage III"),
    (39, "Partial Color"),
    (40, "Partial Color II"),
    (41, "Partial Color III"),
    (42, "Bleach Bypass"),
    (43, "Bleach Bypass II"),
    (44, "Instant Film"),
];

/// Lookup table (lazy-initialized)
pub static OLYMPUS_FILTERS: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    OLYMPUS_FILTERS_DATA.iter().cloned().collect()
});

/// Look up value by key
pub fn lookup_olympus_filters(key: u8) -> Option<&'static str> {
    OLYMPUS_FILTERS.get(&key).copied()
}
