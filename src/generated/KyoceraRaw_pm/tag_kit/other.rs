//! Tag kits for other category from KyoceraRaw.pm
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

static PRINT_CONV_0: LazyLock<HashMap<String, &'static str>> = LazyLock::new(HashMap::new);

/// Get tag definitions for other category
pub fn get_other_tags() -> Vec<(u32, TagKitDef)> {
    vec![
        (
            104,
            TagKitDef {
                id: 104,
                name: "MaxAperture",
                format: "int32u",
                groups: HashMap::new(),
                writable: false,
                notes: None,
                print_conv: PrintConvType::Expression("sprintf(\"%.2g\",$val)"),
                value_conv: Some("2**($val/16)"),
                subdirectory: None,
            },
        ),
        (
            112,
            TagKitDef {
                id: 112,
                name: "FocalLength",
                format: "int32u",
                groups: HashMap::new(),
                writable: false,
                notes: None,
                print_conv: PrintConvType::Expression("\"$val mm\""),
                value_conv: None,
                subdirectory: None,
            },
        ),
        (
            12,
            TagKitDef {
                id: 12,
                name: "Model",
                format: "string[12]",
                groups: HashMap::new(),
                writable: false,
                notes: None,
                print_conv: PrintConvType::None,
                value_conv: None,
                subdirectory: None,
            },
        ),
        (
            124,
            TagKitDef {
                id: 124,
                name: "Lens",
                format: "string[32]",
                groups: HashMap::new(),
                writable: false,
                notes: None,
                print_conv: PrintConvType::None,
                value_conv: None,
                subdirectory: None,
            },
        ),
        (
            25,
            TagKitDef {
                id: 25,
                name: "Make",
                format: "string[7]",
                groups: HashMap::new(),
                writable: false,
                notes: None,
                print_conv: PrintConvType::None,
                value_conv: None,
                subdirectory: None,
            },
        ),
        (
            52,
            TagKitDef {
                id: 52,
                name: "ISO",
                format: "int32u",
                groups: HashMap::new(),
                writable: false,
                notes: None,
                print_conv: PrintConvType::Simple(&PRINT_CONV_0),
                value_conv: None,
                subdirectory: None,
            },
        ),
        (
            60,
            TagKitDef {
                id: 60,
                name: "WB_RGGBLevels",
                format: "int32u[4]",
                groups: HashMap::new(),
                writable: false,
                notes: None,
                print_conv: PrintConvType::None,
                value_conv: None,
                subdirectory: None,
            },
        ),
        (
            88,
            TagKitDef {
                id: 88,
                name: "FNumber",
                format: "int32u",
                groups: HashMap::new(),
                writable: false,
                notes: None,
                print_conv: PrintConvType::Expression("sprintf(\"%.2g\",$val)"),
                value_conv: Some("2**($val/16)"),
                subdirectory: None,
            },
        ),
    ]
}
