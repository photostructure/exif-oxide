//! Tag kits for color category from Sony.pm
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

static PRINT_CONV_1: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("0".to_string(), "Normal");
    map.insert("1".to_string(), "Continuous");
    map.insert("10".to_string(), "Continuous - Background defocus");
    map.insert("13".to_string(), "Continuous - 3D Sweep Panorama");
    map.insert("146".to_string(), "Single Frame - Movie Capture");
    map.insert(
        "15".to_string(),
        "Continuous - High Resolution Sweep Panorama",
    );
    map.insert("16".to_string(), "Continuous - 3D Image");
    map.insert("17".to_string(), "Continuous - Burst 2");
    map.insert("18".to_string(), "Normal - iAuto+");
    map.insert("19".to_string(), "Continuous - Speed/Advance Priority");
    map.insert("2".to_string(), "Continuous - Exposure Bracketing");
    map.insert("20".to_string(), "Continuous - Multi Frame NR");
    map.insert("23".to_string(), "Single-frame - Exposure Bracketing");
    map.insert("26".to_string(), "Continuous Low");
    map.insert("27".to_string(), "Continuous - High Sensitivity");
    map.insert("28".to_string(), "Smile Shutter");
    map.insert("29".to_string(), "Continuous - Tele-zoom Advance Priority");
    map.insert("3".to_string(), "DRO or White Balance Bracketing");
    map.insert("5".to_string(), "Continuous - Burst");
    map.insert("6".to_string(), "Single Frame - Capture During Movie");
    map.insert("7".to_string(), "Continuous - Sweep Panorama");
    map.insert(
        "8".to_string(),
        "Continuous - Anti-Motion Blur, Hand-held Twilight",
    );
    map.insert("9".to_string(), "Continuous - HDR");
    map
});

static PRINT_CONV_2: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("0".to_string(), "Normal");
    map.insert("1".to_string(), "Continuous");
    map.insert("10".to_string(), "Continuous - Background defocus");
    map.insert("13".to_string(), "Continuous - 3D Sweep Panorama");
    map.insert("146".to_string(), "Single Frame - Movie Capture");
    map.insert(
        "15".to_string(),
        "Continuous - High Resolution Sweep Panorama",
    );
    map.insert("16".to_string(), "Continuous - 3D Image");
    map.insert("17".to_string(), "Continuous - Burst 2");
    map.insert("18".to_string(), "Normal - iAuto+");
    map.insert("19".to_string(), "Continuous - Speed/Advance Priority");
    map.insert("2".to_string(), "Continuous - Exposure Bracketing");
    map.insert("20".to_string(), "Continuous - Multi Frame NR");
    map.insert("23".to_string(), "Single-frame - Exposure Bracketing");
    map.insert("26".to_string(), "Continuous Low");
    map.insert("27".to_string(), "Continuous - High Sensitivity");
    map.insert("28".to_string(), "Smile Shutter");
    map.insert("29".to_string(), "Continuous - Tele-zoom Advance Priority");
    map.insert("3".to_string(), "DRO or White Balance Bracketing");
    map.insert("5".to_string(), "Continuous - Burst");
    map.insert("6".to_string(), "Single Frame - Capture During Movie");
    map.insert("7".to_string(), "Continuous - Sweep Panorama");
    map.insert(
        "8".to_string(),
        "Continuous - Anti-Motion Blur, Hand-held Twilight",
    );
    map.insert("9".to_string(), "Continuous - HDR");
    map
});

static PRINT_CONV_3: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("0".to_string(), "Autoflash");
    map.insert("1".to_string(), "Fill-flash");
    map.insert("2".to_string(), "Flash Off");
    map.insert("3".to_string(), "Slow Sync");
    map.insert("4".to_string(), "Rear Sync");
    map.insert("6".to_string(), "Wireless");
    map
});

/// Get tag definitions for color category
pub fn get_color_tags() -> Vec<(u32, TagKitDef)> {
    vec![
        (
            528,
            TagKitDef {
                id: 528,
                name: "ReleaseMode2",
                format: "unknown",
                groups: HashMap::new(),
                writable: false,
                notes: None,
                print_conv: PrintConvType::Simple(&PRINT_CONV_1),
                value_conv: None,
                subdirectory: None,
            },
        ),
        (
            528,
            TagKitDef {
                id: 528,
                name: "ReleaseMode2",
                format: "unknown",
                groups: HashMap::new(),
                writable: false,
                notes: None,
                print_conv: PrintConvType::Simple(&PRINT_CONV_2),
                value_conv: None,
                subdirectory: None,
            },
        ),
        (
            529,
            TagKitDef {
                id: 529,
                name: "FlashMode",
                format: "unknown",
                groups: HashMap::new(),
                writable: false,
                notes: None,
                print_conv: PrintConvType::Simple(&PRINT_CONV_3),
                value_conv: None,
                subdirectory: None,
            },
        ),
        (
            535,
            TagKitDef {
                id: 535,
                name: "StopsAboveBaseISO",
                format: "int16u",
                groups: HashMap::new(),
                writable: false,
                notes: None,
                print_conv: PrintConvType::Manual("complex_expression_printconv"),
                value_conv: Some("16 - $val/256"),
                subdirectory: None,
            },
        ),
    ]
}
