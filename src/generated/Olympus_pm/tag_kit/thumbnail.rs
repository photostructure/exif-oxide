//! Tag kits for thumbnail category from Olympus.pm
//!
//! This file is automatically generated by codegen.
//! DO NOT EDIT MANUALLY - changes will be overwritten.

#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_variables)]

use super::*;
use super::{PrintConvType, SubDirectoryType, TagKitDef};
use crate::types::TagValue;
use std::collections::HashMap;
use std::sync::LazyLock;

static PRINT_CONV_71: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("0 00 00".to_string(), "None");
    map.insert(
        "0 01 00".to_string(),
        "Olympus Zuiko Digital ED 50mm F2.0 Macro",
    );
    map.insert(
        "0 01 01".to_string(),
        "Olympus Zuiko Digital 40-150mm F3.5-4.5",
    );
    map.insert(
        "0 01 10".to_string(),
        "Olympus M.Zuiko Digital ED 14-42mm F3.5-5.6",
    );
    map.insert("0 02 00".to_string(), "Olympus Zuiko Digital ED 150mm F2.0");
    map.insert(
        "0 02 10".to_string(),
        "Olympus M.Zuiko Digital 17mm F2.8 Pancake",
    );
    map.insert("0 03 00".to_string(), "Olympus Zuiko Digital ED 300mm F2.8");
    map.insert(
        "0 03 10".to_string(),
        "Olympus M.Zuiko Digital ED 14-150mm F4.0-5.6 [II]",
    );
    map.insert(
        "0 04 10".to_string(),
        "Olympus M.Zuiko Digital ED 9-18mm F4.0-5.6",
    );
    map.insert(
        "0 05 00".to_string(),
        "Olympus Zuiko Digital 14-54mm F2.8-3.5",
    );
    map.insert(
        "0 05 01".to_string(),
        "Olympus Zuiko Digital Pro ED 90-250mm F2.8",
    );
    map.insert(
        "0 05 10".to_string(),
        "Olympus M.Zuiko Digital ED 14-42mm F3.5-5.6 L",
    );
    map.insert(
        "0 06 00".to_string(),
        "Olympus Zuiko Digital ED 50-200mm F2.8-3.5",
    );
    map.insert(
        "0 06 01".to_string(),
        "Olympus Zuiko Digital ED 8mm F3.5 Fisheye",
    );
    map.insert(
        "0 06 10".to_string(),
        "Olympus M.Zuiko Digital ED 40-150mm F4.0-5.6",
    );
    map.insert(
        "0 07 00".to_string(),
        "Olympus Zuiko Digital 11-22mm F2.8-3.5",
    );
    map.insert(
        "0 07 01".to_string(),
        "Olympus Zuiko Digital 18-180mm F3.5-6.3",
    );
    map.insert(
        "0 07 10".to_string(),
        "Olympus M.Zuiko Digital ED 12mm F2.0",
    );
    map.insert(
        "0 08 01".to_string(),
        "Olympus Zuiko Digital 70-300mm F4.0-5.6",
    );
    map.insert(
        "0 08 10".to_string(),
        "Olympus M.Zuiko Digital ED 75-300mm F4.8-6.7",
    );
    map.insert(
        "0 09 10".to_string(),
        "Olympus M.Zuiko Digital 14-42mm F3.5-5.6 II",
    );
    map.insert(
        "0 10 01".to_string(),
        "Kenko Tokina Reflex 300mm F6.3 MF Macro",
    );
    map.insert(
        "0 10 10".to_string(),
        "Olympus M.Zuiko Digital ED 12-50mm F3.5-6.3 EZ",
    );
    map.insert("0 11 10".to_string(), "Olympus M.Zuiko Digital 45mm F1.8");
    map.insert(
        "0 12 10".to_string(),
        "Olympus M.Zuiko Digital ED 60mm F2.8 Macro",
    );
    map.insert(
        "0 13 10".to_string(),
        "Olympus M.Zuiko Digital 14-42mm F3.5-5.6 II R",
    );
    map.insert(
        "0 14 10".to_string(),
        "Olympus M.Zuiko Digital ED 40-150mm F4.0-5.6 R",
    );
    map.insert(
        "0 15 00".to_string(),
        "Olympus Zuiko Digital ED 7-14mm F4.0",
    );
    map.insert(
        "0 15 10".to_string(),
        "Olympus M.Zuiko Digital ED 75mm F1.8",
    );
    map.insert("0 16 10".to_string(), "Olympus M.Zuiko Digital 17mm F1.8");
    map.insert(
        "0 17 00".to_string(),
        "Olympus Zuiko Digital Pro ED 35-100mm F2.0",
    );
    map.insert(
        "0 18 00".to_string(),
        "Olympus Zuiko Digital 14-45mm F3.5-5.6",
    );
    map.insert(
        "0 18 10".to_string(),
        "Olympus M.Zuiko Digital ED 75-300mm F4.8-6.7 II",
    );
    map.insert(
        "0 19 10".to_string(),
        "Olympus M.Zuiko Digital ED 12-40mm F2.8 Pro",
    );
    map.insert(
        "0 20 00".to_string(),
        "Olympus Zuiko Digital 35mm F3.5 Macro",
    );
    map.insert(
        "0 20 10".to_string(),
        "Olympus M.Zuiko Digital ED 40-150mm F2.8 Pro",
    );
    map.insert(
        "0 21 10".to_string(),
        "Olympus M.Zuiko Digital ED 14-42mm F3.5-5.6 EZ",
    );
    map.insert(
        "0 22 00".to_string(),
        "Olympus Zuiko Digital 17.5-45mm F3.5-5.6",
    );
    map.insert("0 22 10".to_string(), "Olympus M.Zuiko Digital 25mm F1.8");
    map.insert(
        "0 23 00".to_string(),
        "Olympus Zuiko Digital ED 14-42mm F3.5-5.6",
    );
    map.insert(
        "0 23 10".to_string(),
        "Olympus M.Zuiko Digital ED 7-14mm F2.8 Pro",
    );
    map.insert(
        "0 24 00".to_string(),
        "Olympus Zuiko Digital ED 40-150mm F4.0-5.6",
    );
    map.insert(
        "0 24 10".to_string(),
        "Olympus M.Zuiko Digital ED 300mm F4.0 IS Pro",
    );
    map.insert(
        "0 25 10".to_string(),
        "Olympus M.Zuiko Digital ED 8mm F1.8 Fisheye Pro",
    );
    map.insert(
        "0 26 10".to_string(),
        "Olympus M.Zuiko Digital ED 12-100mm F4.0 IS Pro",
    );
    map.insert(
        "0 27 10".to_string(),
        "Olympus M.Zuiko Digital ED 30mm F3.5 Macro",
    );
    map.insert(
        "0 28 10".to_string(),
        "Olympus M.Zuiko Digital ED 25mm F1.2 Pro",
    );
    map.insert(
        "0 29 10".to_string(),
        "Olympus M.Zuiko Digital ED 17mm F1.2 Pro",
    );
    map.insert(
        "0 30 00".to_string(),
        "Olympus Zuiko Digital ED 50-200mm F2.8-3.5 SWD",
    );
    map.insert(
        "0 30 10".to_string(),
        "Olympus M.Zuiko Digital ED 45mm F1.2 Pro",
    );
    map.insert(
        "0 31 00".to_string(),
        "Olympus Zuiko Digital ED 12-60mm F2.8-4.0 SWD",
    );
    map.insert(
        "0 32 00".to_string(),
        "Olympus Zuiko Digital ED 14-35mm F2.0 SWD",
    );
    map.insert(
        "0 32 10".to_string(),
        "Olympus M.Zuiko Digital ED 12-200mm F3.5-6.3",
    );
    map.insert("0 33 00".to_string(), "Olympus Zuiko Digital 25mm F2.8");
    map.insert(
        "0 33 10".to_string(),
        "Olympus M.Zuiko Digital 150-400mm F4.5 TC1.25x IS Pro",
    );
    map.insert(
        "0 34 00".to_string(),
        "Olympus Zuiko Digital ED 9-18mm F4.0-5.6",
    );
    map.insert(
        "0 34 10".to_string(),
        "Olympus M.Zuiko Digital ED 12-45mm F4.0 Pro",
    );
    map.insert(
        "0 35 00".to_string(),
        "Olympus Zuiko Digital 14-54mm F2.8-3.5 II",
    );
    map.insert("0 35 10".to_string(), "Olympus M.Zuiko 100-400mm F5.0-6.3");
    map.insert(
        "0 36 10".to_string(),
        "Olympus M.Zuiko Digital ED 8-25mm F4 Pro",
    );
    map.insert(
        "0 37 10".to_string(),
        "Olympus M.Zuiko Digital ED 40-150mm F4.0 Pro",
    );
    map.insert(
        "0 39 10".to_string(),
        "Olympus M.Zuiko Digital ED 90mm F3.5 Macro IS Pro",
    );
    map.insert(
        "0 40 10".to_string(),
        "Olympus M.Zuiko Digital ED 150-600mm F5.0-6.3",
    );
    map.insert("1 01 00".to_string(), "Sigma 18-50mm F3.5-5.6 DC");
    map.insert("1 01 10".to_string(), "Sigma 30mm F2.8 EX DN");
    map.insert("1 02 00".to_string(), "Sigma 55-200mm F4.0-5.6 DC");
    map.insert("1 02 10".to_string(), "Sigma 19mm F2.8 EX DN");
    map.insert("1 03 00".to_string(), "Sigma 18-125mm F3.5-5.6 DC");
    map.insert("1 03 10".to_string(), "Sigma 30mm F2.8 DN | A");
    map.insert("1 04 00".to_string(), "Sigma 18-125mm F3.5-5.6 DC");
    map.insert("1 04 10".to_string(), "Sigma 19mm F2.8 DN | A");
    map.insert("1 05 00".to_string(), "Sigma 30mm F1.4 EX DC HSM");
    map.insert("1 05 10".to_string(), "Sigma 60mm F2.8 DN | A");
    map.insert(
        "1 06 00".to_string(),
        "Sigma APO 50-500mm F4.0-6.3 EX DG HSM",
    );
    map.insert("1 06 10".to_string(), "Sigma 30mm F1.4 DC DN | C");
    map.insert("1 07 00".to_string(), "Sigma Macro 105mm F2.8 EX DG");
    map.insert("1 07 10".to_string(), "Sigma 16mm F1.4 DC DN | C (017)");
    map.insert(
        "1 08 00".to_string(),
        "Sigma APO Macro 150mm F2.8 EX DG HSM",
    );
    map.insert("1 09 00".to_string(), "Sigma 18-50mm F2.8 EX DC Macro");
    map.insert(
        "1 10 00".to_string(),
        "Sigma 24mm F1.8 EX DG Aspherical Macro",
    );
    map.insert("1 11 00".to_string(), "Sigma APO 135-400mm F4.5-5.6 DG");
    map.insert("1 12 00".to_string(), "Sigma APO 300-800mm F5.6 EX DG HSM");
    map.insert("1 13 00".to_string(), "Sigma 30mm F1.4 EX DC HSM");
    map.insert(
        "1 14 00".to_string(),
        "Sigma APO 50-500mm F4.0-6.3 EX DG HSM",
    );
    map.insert("1 15 00".to_string(), "Sigma 10-20mm F4.0-5.6 EX DC HSM");
    map.insert(
        "1 16 00".to_string(),
        "Sigma APO 70-200mm F2.8 II EX DG Macro HSM",
    );
    map.insert("1 17 00".to_string(), "Sigma 50mm F1.4 EX DG HSM");
    map.insert(
        "2 01 00".to_string(),
        "Leica D Vario Elmarit 14-50mm F2.8-3.5 Asph.",
    );
    map.insert(
        "2 01 10".to_string(),
        "Lumix G Vario 14-45mm F3.5-5.6 Asph. Mega OIS",
    );
    map.insert("2 02 00".to_string(), "Leica D Summilux 25mm F1.4 Asph.");
    map.insert(
        "2 02 10".to_string(),
        "Lumix G Vario 45-200mm F4.0-5.6 Mega OIS",
    );
    map.insert(
        "2 03 00".to_string(),
        "Leica D Vario Elmar 14-50mm F3.8-5.6 Asph. Mega OIS",
    );
    map.insert(
        "2 03 01".to_string(),
        "Leica D Vario Elmar 14-50mm F3.8-5.6 Asph.",
    );
    map.insert(
        "2 03 10".to_string(),
        "Lumix G Vario HD 14-140mm F4.0-5.8 Asph. Mega OIS",
    );
    map.insert(
        "2 04 00".to_string(),
        "Leica D Vario Elmar 14-150mm F3.5-5.6",
    );
    map.insert("2 04 10".to_string(), "Lumix G Vario 7-14mm F4.0 Asph.");
    map.insert("2 05 10".to_string(), "Lumix G 20mm F1.7 Asph.");
    map.insert(
        "2 06 10".to_string(),
        "Leica DG Macro-Elmarit 45mm F2.8 Asph. Mega OIS",
    );
    map.insert(
        "2 07 10".to_string(),
        "Lumix G Vario 14-42mm F3.5-5.6 Asph. Mega OIS",
    );
    map.insert("2 08 10".to_string(), "Lumix G Fisheye 8mm F3.5");
    map.insert(
        "2 09 10".to_string(),
        "Lumix G Vario 100-300mm F4.0-5.6 Mega OIS",
    );
    map.insert("2 10 10".to_string(), "Lumix G 14mm F2.5 Asph.");
    map.insert("2 11 10".to_string(), "Lumix G 12.5mm F12 3D");
    map.insert("2 12 10".to_string(), "Leica DG Summilux 25mm F1.4 Asph.");
    map.insert(
        "2 13 10".to_string(),
        "Lumix G X Vario PZ 45-175mm F4.0-5.6 Asph. Power OIS",
    );
    map.insert(
        "2 14 10".to_string(),
        "Lumix G X Vario PZ 14-42mm F3.5-5.6 Asph. Power OIS",
    );
    map.insert(
        "2 15 10".to_string(),
        "Lumix G X Vario 12-35mm F2.8 Asph. Power OIS",
    );
    map.insert(
        "2 16 10".to_string(),
        "Lumix G Vario 45-150mm F4.0-5.6 Asph. Mega OIS",
    );
    map.insert(
        "2 17 10".to_string(),
        "Lumix G X Vario 35-100mm F2.8 Power OIS",
    );
    map.insert(
        "2 18 10".to_string(),
        "Lumix G Vario 14-42mm F3.5-5.6 II Asph. Mega OIS",
    );
    map.insert(
        "2 19 10".to_string(),
        "Lumix G Vario 14-140mm F3.5-5.6 Asph. Power OIS",
    );
    map.insert(
        "2 20 10".to_string(),
        "Lumix G Vario 12-32mm F3.5-5.6 Asph. Mega OIS",
    );
    map.insert(
        "2 21 10".to_string(),
        "Leica DG Nocticron 42.5mm F1.2 Asph. Power OIS",
    );
    map.insert("2 22 10".to_string(), "Leica DG Summilux 15mm F1.7 Asph.");
    map.insert(
        "2 23 10".to_string(),
        "Lumix G Vario 35-100mm F4.0-5.6 Asph. Mega OIS",
    );
    map.insert(
        "2 24 10".to_string(),
        "Lumix G Macro 30mm F2.8 Asph. Mega OIS",
    );
    map.insert("2 25 10".to_string(), "Lumix G 42.5mm F1.7 Asph. Power OIS");
    map.insert("2 26 10".to_string(), "Lumix G 25mm F1.7 Asph.");
    map.insert(
        "2 27 10".to_string(),
        "Leica DG Vario-Elmar 100-400mm F4.0-6.3 Asph. Power OIS",
    );
    map.insert(
        "2 28 10".to_string(),
        "Lumix G Vario 12-60mm F3.5-5.6 Asph. Power OIS",
    );
    map.insert("2 29 10".to_string(), "Leica DG Summilux 12mm F1.4 Asph.");
    map.insert(
        "2 30 10".to_string(),
        "Leica DG Vario-Elmarit 12-60mm F2.8-4 Asph. Power OIS",
    );
    map.insert("2 31 10".to_string(), "Lumix G Vario 45-200mm F4.0-5.6 II");
    map.insert("2 32 10".to_string(), "Lumix G Vario 100-300mm F4.0-5.6 II");
    map.insert(
        "2 33 10".to_string(),
        "Lumix G X Vario 12-35mm F2.8 II Asph. Power OIS",
    );
    map.insert("2 34 10".to_string(), "Lumix G Vario 35-100mm F2.8 II");
    map.insert(
        "2 35 10".to_string(),
        "Leica DG Vario-Elmarit 8-18mm F2.8-4 Asph.",
    );
    map.insert(
        "2 36 10".to_string(),
        "Leica DG Elmarit 200mm F2.8 Power OIS",
    );
    map.insert(
        "2 37 10".to_string(),
        "Leica DG Vario-Elmarit 50-200mm F2.8-4 Asph. Power OIS",
    );
    map.insert(
        "2 38 10".to_string(),
        "Leica DG Vario-Summilux 10-25mm F1.7 Asph.",
    );
    map.insert(
        "2 39 10".to_string(),
        "Leica DG Summilux 25mm F1.4 II Asph.",
    );
    map.insert(
        "2 40 10".to_string(),
        "Leica DG Vario-Summilux 25-50mm F1.7 Asph.",
    );
    map.insert("2 41 10".to_string(), "Leica DG Summilux 9mm F1.7 Asph.");
    map.insert(
        "24 01 10".to_string(),
        "Venus Optics Laowa 50mm F2.8 2x Macro",
    );
    map.insert(
        "3 01 00".to_string(),
        "Leica D Vario Elmarit 14-50mm F2.8-3.5 Asph.",
    );
    map.insert("3 02 00".to_string(), "Leica D Summilux 25mm F1.4 Asph.");
    map.insert("5 01 10".to_string(), "Tamron 14-150mm F3.5-5.8 Di III");
    map.insert("f7 03 10".to_string(), "LAOWA C&D-Dreamer MFT 7.5mm F2.0");
    map
});

static PRINT_CONV_72: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("0".to_string(), "Off");
    map.insert("1".to_string(), "On");
    map
});

static PRINT_CONV_73: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("1027".to_string(), "Spot+Shadow control");
    map.insert("2".to_string(), "Center-weighted average");
    map.insert("261".to_string(), "Pattern+AF");
    map.insert("3".to_string(), "Spot");
    map.insert("5".to_string(), "ESP");
    map.insert("515".to_string(), "Spot+Highlight control");
    map
});

static PRINT_CONV_74: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("0".to_string(), "Off");
    map.insert("1".to_string(), "On");
    map.insert("2".to_string(), "Super Macro");
    map
});

/// Get tag definitions for thumbnail category
pub fn get_thumbnail_tags() -> Vec<(u32, TagKitDef)> {
    vec![
        (513, TagKitDef {
            id: 513,
            name: "LensType",
            format: "int8u",
            groups: HashMap::new(),
            writable: true,
            notes: Some("6 numbers: 1. Make, 2. Unknown, 3. Model, 4. Sub-model, 5-6. Unknown.  Only\n            the Make, Model and Sub-model are used to identify the lens type"),
            print_conv: PrintConvType::Simple(&PRINT_CONV_71),
            value_conv: Some("my @a=split(\" \",$val); sprintf(\"%x %.2x %.2x\",@a[0,2,3])"),
            subdirectory: None,
        }),
        (514, TagKitDef {
            id: 514,
            name: "LensSerialNumber",
            format: "string",
            groups: HashMap::new(),
            writable: true,
            notes: None,
            print_conv: PrintConvType::Manual("complex_expression_printconv"),
            value_conv: None,
            subdirectory: None,
        }),
        (513, TagKitDef {
            id: 513,
            name: "AELock",
            format: "int16u",
            groups: HashMap::new(),
            writable: true,
            notes: None,
            print_conv: PrintConvType::Simple(&PRINT_CONV_72),
            value_conv: None,
            subdirectory: None,
        }),
        (514, TagKitDef {
            id: 514,
            name: "MeteringMode",
            format: "int16u",
            groups: HashMap::new(),
            writable: true,
            notes: None,
            print_conv: PrintConvType::Simple(&PRINT_CONV_73),
            value_conv: None,
            subdirectory: None,
        }),
        (256, TagKitDef {
            id: 256,
            name: "ThumbnailImage",
            format: "undef",
            groups: HashMap::new(),
            writable: true,
            notes: None,
            print_conv: PrintConvType::None,
            value_conv: None,
            subdirectory: None,
        }),
        (513, TagKitDef {
            id: 513,
            name: "Quality",
            format: "int16u",
            groups: HashMap::new(),
            writable: true,
            notes: Some("Quality values are decoded based on the CameraType tag. All types\n            represent SQ, HQ and SHQ as sequential integers, but in general\n            SX-type cameras start with a value of 0 for SQ while others start\n            with 1"),
            print_conv: PrintConvType::Manual("code_ref_printconv"),
            value_conv: None,
            subdirectory: None,
        }),
        (514, TagKitDef {
            id: 514,
            name: "Macro",
            format: "int16u",
            groups: HashMap::new(),
            writable: true,
            notes: None,
            print_conv: PrintConvType::Simple(&PRINT_CONV_74),
            value_conv: None,
            subdirectory: None,
        }),
    ]
}
