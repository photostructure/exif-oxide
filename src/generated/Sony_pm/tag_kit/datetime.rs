//! Tag kits for datetime category from Sony.pm
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

static PRINT_CONV_7: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("0".to_string(), "Off");
    map.insert("1".to_string(), "Self-timer 10 s");
    map.insert("2".to_string(), "Self-timer 2 s");
    map
});

static PRINT_CONV_8: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("0".to_string(), "Off");
    map.insert("1".to_string(), "Self-timer 10 s");
    map.insert("2".to_string(), "Self-timer 2 s");
    map
});

static PRINT_CONV_9: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("0".to_string(), "Off");
    map.insert("1".to_string(), "Self-timer 10 s");
    map.insert("2".to_string(), "Self-timer 2 s");
    map
});

static PRINT_CONV_10: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("0".to_string(), "Off");
    map.insert("1".to_string(), "Self-timer 10 s");
    map.insert("2".to_string(), "Self-timer 2 s");
    map
});

static PRINT_CONV_11: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("0".to_string(), "Off");
    map.insert("1".to_string(), "Self-timer 10 s");
    map.insert("2".to_string(), "Self-timer 2 s");
    map
});

static PRINT_CONV_12: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("0".to_string(), "Off");
    map.insert("1".to_string(), "Self-timer 10 s");
    map.insert("2".to_string(), "Self-timer 2 s");
    map
});

static PRINT_CONV_13: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("0".to_string(), "Off");
    map.insert("1".to_string(), "Self-timer 10 s");
    map.insert("2".to_string(), "Self-timer 2 s");
    map
});

static PRINT_CONV_14: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("0".to_string(), "Off");
    map.insert("1".to_string(), "Self-timer 5 or 10 s");
    map.insert("2".to_string(), "Self-timer 2 s");
    map
});

static PRINT_CONV_15: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("0".to_string(), "Off");
    map.insert("1".to_string(), "Self-timer 5 or 10 s");
    map.insert("2".to_string(), "Self-timer 2 s");
    map
});

/// Get tag definitions for datetime category
pub fn get_datetime_tags() -> Vec<(u32, TagKitDef)> {
    vec![
        (0, TagKitDef {
            id: 0,
            name: "ExposureTime",
            format: "unknown",
            groups: HashMap::new(),
            writable: false,
            notes: None,
            print_conv: PrintConvType::Manual("complex_expression_printconv"),
            value_conv: Some("$val ? 2 ** (6 - $val/8) : 0"),
            subdirectory: None,
        }),
        (0, TagKitDef {
            id: 0,
            name: "ExposureTime",
            format: "unknown",
            groups: HashMap::new(),
            writable: false,
            notes: None,
            print_conv: PrintConvType::Manual("complex_expression_printconv"),
            value_conv: Some("$val ? 2 ** (6 - $val/8) : 0"),
            subdirectory: None,
        }),
        (33, TagKitDef {
            id: 33,
            name: "ExposureTime",
            format: "unknown",
            groups: HashMap::new(),
            writable: false,
            notes: Some("A450, A500 and A550"),
            print_conv: PrintConvType::Manual("complex_expression_printconv"),
            value_conv: Some("$val ? 2 ** (6 - $val/8) : 0"),
            subdirectory: None,
        }),
        (35, TagKitDef {
            id: 35,
            name: "ExposureTime",
            format: "unknown",
            groups: HashMap::new(),
            writable: false,
            notes: Some("NEX-3/5/5C"),
            print_conv: PrintConvType::Manual("complex_expression_printconv"),
            value_conv: Some("$val ? 2 ** (6 - $val/8) : 0"),
            subdirectory: None,
        }),
        (39, TagKitDef {
            id: 39,
            name: "ExposureTime",
            format: "unknown",
            groups: HashMap::new(),
            writable: false,
            notes: Some("models other than the A450, A500, A550 and NEX-3/5/5C"),
            print_conv: PrintConvType::Manual("complex_expression_printconv"),
            value_conv: Some("$val ? 2 ** (6 - $val/8) : 0"),
            subdirectory: None,
        }),
        (6, TagKitDef {
            id: 6,
            name: "SonyDateTime",
            format: "string[20]",
            groups: HashMap::new(),
            writable: false,
            notes: None,
            print_conv: PrintConvType::Manual("complex_expression_printconv"),
            value_conv: None,
            subdirectory: None,
        }),
        (4404, TagKitDef {
            id: 4404,
            name: "SelfTimer",
            format: "unknown",
            groups: HashMap::new(),
            writable: false,
            notes: None,
            print_conv: PrintConvType::Simple(&PRINT_CONV_7),
            value_conv: None,
            subdirectory: None,
        }),
        (438, TagKitDef {
            id: 438,
            name: "SonyDateTime",
            format: "undef[7]",
            groups: HashMap::new(),
            writable: false,
            notes: None,
            print_conv: PrintConvType::Manual("complex_expression_printconv"),
            value_conv: Some("\n        my @v = unpack('vC*', $val);\n        return sprintf(\"%.4d:%.2d:%.2d %.2d:%.2d:%.2d\", @v)\n    "),
            subdirectory: None,
        }),
        (4404, TagKitDef {
            id: 4404,
            name: "SelfTimer",
            format: "unknown",
            groups: HashMap::new(),
            writable: false,
            notes: None,
            print_conv: PrintConvType::Simple(&PRINT_CONV_8),
            value_conv: None,
            subdirectory: None,
        }),
        (4368, TagKitDef {
            id: 4368,
            name: "SelfTimer",
            format: "unknown",
            groups: HashMap::new(),
            writable: false,
            notes: None,
            print_conv: PrintConvType::Simple(&PRINT_CONV_9),
            value_conv: None,
            subdirectory: None,
        }),
        (528, TagKitDef {
            id: 528,
            name: "SonyDateTime",
            format: "undef[7]",
            groups: HashMap::new(),
            writable: false,
            notes: None,
            print_conv: PrintConvType::Manual("complex_expression_printconv"),
            value_conv: Some("\n        my @v = unpack('vC*', $val);\n        return sprintf(\"%.4d:%.2d:%.2d %.2d:%.2d:%.2d\", @v)\n    "),
            subdirectory: None,
        }),
        (4492, TagKitDef {
            id: 4492,
            name: "SelfTimer",
            format: "unknown",
            groups: HashMap::new(),
            writable: false,
            notes: None,
            print_conv: PrintConvType::Simple(&PRINT_CONV_10),
            value_conv: None,
            subdirectory: None,
        }),
        (510, TagKitDef {
            id: 510,
            name: "SonyDateTime",
            format: "undef[7]",
            groups: HashMap::new(),
            writable: false,
            notes: None,
            print_conv: PrintConvType::Manual("complex_expression_printconv"),
            value_conv: Some("\n        my @v = unpack('vC*', $val);\n        return sprintf(\"%.4d:%.2d:%.2d %.2d:%.2d:%.2d\", @v)\n    "),
            subdirectory: None,
        }),
        (4456, TagKitDef {
            id: 4456,
            name: "SelfTimer",
            format: "unknown",
            groups: HashMap::new(),
            writable: false,
            notes: None,
            print_conv: PrintConvType::Simple(&PRINT_CONV_11),
            value_conv: None,
            subdirectory: None,
        }),
        (556, TagKitDef {
            id: 556,
            name: "SonyDateTime",
            format: "undef[7]",
            groups: HashMap::new(),
            writable: false,
            notes: None,
            print_conv: PrintConvType::Manual("complex_expression_printconv"),
            value_conv: Some("\n        my @v = unpack('vC*', $val);\n        return sprintf(\"%.4d:%.2d:%.2d %.2d:%.2d:%.2d\", @v)\n    "),
            subdirectory: None,
        }),
        (4128, TagKitDef {
            id: 4128,
            name: "SelfTimer",
            format: "unknown",
            groups: HashMap::new(),
            writable: false,
            notes: None,
            print_conv: PrintConvType::Simple(&PRINT_CONV_12),
            value_conv: None,
            subdirectory: None,
        }),
        (536, TagKitDef {
            id: 536,
            name: "SelfTimer",
            format: "unknown",
            groups: HashMap::new(),
            writable: false,
            notes: None,
            print_conv: PrintConvType::Simple(&PRINT_CONV_13),
            value_conv: None,
            subdirectory: None,
        }),
        (536, TagKitDef {
            id: 536,
            name: "SelfTimer",
            format: "unknown",
            groups: HashMap::new(),
            writable: false,
            notes: None,
            print_conv: PrintConvType::Simple(&PRINT_CONV_14),
            value_conv: None,
            subdirectory: None,
        }),
        (528, TagKitDef {
            id: 528,
            name: "SelfTimer",
            format: "unknown",
            groups: HashMap::new(),
            writable: false,
            notes: None,
            print_conv: PrintConvType::Simple(&PRINT_CONV_15),
            value_conv: None,
            subdirectory: None,
        }),
    ]
}
