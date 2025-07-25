//! AF point indices for 153-point AF system
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Nikon.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Raw data (153 entries)
static AF_POINTS_153_DATA: &[(u8, &'static str)] = &[
    (1, "E9"),
    (2, "D9"),
    (3, "C9"),
    (4, "B9"),
    (5, "A9"),
    (6, "F9"),
    (7, "G9"),
    (8, "H9"),
    (9, "I9"),
    (10, "E10"),
    (11, "D10"),
    (12, "C10"),
    (13, "B10"),
    (14, "A10"),
    (15, "F10"),
    (16, "G10"),
    (17, "H10"),
    (18, "I10"),
    (19, "E11"),
    (20, "D11"),
    (21, "C11"),
    (22, "B11"),
    (23, "A11"),
    (24, "F11"),
    (25, "G11"),
    (26, "H11"),
    (27, "I11"),
    (28, "E8"),
    (29, "D8"),
    (30, "C8"),
    (31, "B8"),
    (32, "A8"),
    (33, "F8"),
    (34, "G8"),
    (35, "H8"),
    (36, "I8"),
    (37, "E7"),
    (38, "D7"),
    (39, "C7"),
    (40, "B7"),
    (41, "A7"),
    (42, "F7"),
    (43, "G7"),
    (44, "H7"),
    (45, "I7"),
    (46, "E12"),
    (47, "D12"),
    (48, "C12"),
    (49, "B12"),
    (50, "A12"),
    (51, "F12"),
    (52, "G12"),
    (53, "H12"),
    (54, "I12"),
    (55, "E13"),
    (56, "D13"),
    (57, "C13"),
    (58, "B13"),
    (59, "A13"),
    (60, "F13"),
    (61, "G13"),
    (62, "H13"),
    (63, "I13"),
    (64, "E14"),
    (65, "D14"),
    (66, "C14"),
    (67, "B14"),
    (68, "A14"),
    (69, "F14"),
    (70, "G14"),
    (71, "H14"),
    (72, "I14"),
    (73, "E15"),
    (74, "D15"),
    (75, "C15"),
    (76, "B15"),
    (77, "A15"),
    (78, "F15"),
    (79, "G15"),
    (80, "H15"),
    (81, "I15"),
    (82, "E16"),
    (83, "D16"),
    (84, "C16"),
    (85, "B16"),
    (86, "A16"),
    (87, "F16"),
    (88, "G16"),
    (89, "H16"),
    (90, "I16"),
    (91, "E17"),
    (92, "D17"),
    (93, "C17"),
    (94, "B17"),
    (95, "A17"),
    (96, "F17"),
    (97, "G17"),
    (98, "H17"),
    (99, "I17"),
    (100, "E6"),
    (101, "D6"),
    (102, "C6"),
    (103, "B6"),
    (104, "A6"),
    (105, "F6"),
    (106, "G6"),
    (107, "H6"),
    (108, "I6"),
    (109, "E5"),
    (110, "D5"),
    (111, "C5"),
    (112, "B5"),
    (113, "A5"),
    (114, "F5"),
    (115, "G5"),
    (116, "H5"),
    (117, "I5"),
    (118, "E4"),
    (119, "D4"),
    (120, "C4"),
    (121, "B4"),
    (122, "A4"),
    (123, "F4"),
    (124, "G4"),
    (125, "H4"),
    (126, "I4"),
    (127, "E3"),
    (128, "D3"),
    (129, "C3"),
    (130, "B3"),
    (131, "A3"),
    (132, "F3"),
    (133, "G3"),
    (134, "H3"),
    (135, "I3"),
    (136, "E2"),
    (137, "D2"),
    (138, "C2"),
    (139, "B2"),
    (140, "A2"),
    (141, "F2"),
    (142, "G2"),
    (143, "H2"),
    (144, "I2"),
    (145, "E1"),
    (146, "D1"),
    (147, "C1"),
    (148, "B1"),
    (149, "A1"),
    (150, "F1"),
    (151, "G1"),
    (152, "H1"),
    (153, "I1"),
];

/// Lookup table (lazy-initialized)
pub static AF_POINTS_153: LazyLock<HashMap<u8, &'static str>> =
    LazyLock::new(|| AF_POINTS_153_DATA.iter().cloned().collect());

/// Look up value by key
pub fn lookup_af_points_153(key: u8) -> Option<&'static str> {
    AF_POINTS_153.get(&key).copied()
}
