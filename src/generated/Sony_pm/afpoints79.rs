//! Sony 79-point AF system point mappings
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Sony.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (79 entries)
static SONY_AF_POINTS_79_DATA: &[(u8, &'static str)] = &[
    (0, "A5"),
    (1, "A6"),
    (2, "A7"),
    (3, "B2"),
    (4, "B3"),
    (5, "B4"),
    (6, "B5"),
    (7, "B6"),
    (8, "B7"),
    (9, "B8"),
    (10, "B9"),
    (11, "B10"),
    (12, "C1"),
    (13, "C2"),
    (14, "C3"),
    (15, "C4"),
    (16, "C5"),
    (17, "C6"),
    (18, "C7"),
    (19, "C8"),
    (20, "C9"),
    (21, "C10"),
    (22, "C11"),
    (23, "D1"),
    (24, "D2"),
    (25, "D3"),
    (26, "D4"),
    (27, "D5"),
    (28, "D6"),
    (29, "D7"),
    (30, "D8"),
    (31, "D9"),
    (32, "D10"),
    (33, "D11"),
    (34, "E1"),
    (35, "E2"),
    (36, "E3"),
    (37, "E4"),
    (38, "E5"),
    (39, "E6"),
    (40, "E7"),
    (41, "E8"),
    (42, "E9"),
    (43, "E10"),
    (44, "E11"),
    (45, "F1"),
    (46, "F2"),
    (47, "F3"),
    (48, "F4"),
    (49, "F5"),
    (50, "F6"),
    (51, "F7"),
    (52, "F8"),
    (53, "F9"),
    (54, "F10"),
    (55, "F11"),
    (56, "G1"),
    (57, "G2"),
    (58, "G3"),
    (59, "G4"),
    (60, "G5"),
    (61, "G6"),
    (62, "G7"),
    (63, "G8"),
    (64, "G9"),
    (65, "G10"),
    (66, "G11"),
    (67, "H2"),
    (68, "H3"),
    (69, "H4"),
    (70, "H5"),
    (71, "H6"),
    (72, "H7"),
    (73, "H8"),
    (74, "H9"),
    (75, "H10"),
    (76, "I5"),
    (77, "I6"),
    (78, "I7"),
];

/// Lookup table (lazy-initialized)
pub static SONY_AF_POINTS_79: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| SONY_AF_POINTS_79_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_sony_af_points_79(key: u8) -> Option<&'static str> {
    SONY_AF_POINTS_79.get(&key).copied()
}
