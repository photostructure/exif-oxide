//! Tag kits for interop category from Nikon.pm
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

/// Get tag definitions for interop category
pub fn get_interop_tags() -> Vec<(u32, TagKitDef)> {
    vec![
        (
            1,
            TagKitDef {
                id: 1,
                name: "MakerNoteVersion",
                format: "undef",
                groups: HashMap::new(),
                writable: true,
                notes: None,
                print_conv: PrintConvType::Manual("complex_expression_printconv"),
                value_conv: Some(
                    "$_=$val; /^[\\x00-\\x09]/ and $_=join(\"\",unpack(\"CCCC\",$_)); $_",
                ),
                subdirectory: None,
            },
        ),
        (
            2,
            TagKitDef {
                id: 2,
                name: "ISO",
                format: "int16u",
                groups: HashMap::new(),
                writable: true,
                notes: None,
                print_conv: PrintConvType::Manual("complex_expression_printconv"),
                value_conv: None,
                subdirectory: None,
            },
        ),
        (
            3,
            TagKitDef {
                id: 3,
                name: "ColorMode",
                format: "string",
                groups: HashMap::new(),
                writable: true,
                notes: None,
                print_conv: PrintConvType::None,
                value_conv: None,
                subdirectory: None,
            },
        ),
        (
            4,
            TagKitDef {
                id: 4,
                name: "Quality",
                format: "string",
                groups: HashMap::new(),
                writable: true,
                notes: None,
                print_conv: PrintConvType::None,
                value_conv: None,
                subdirectory: None,
            },
        ),
        (
            5,
            TagKitDef {
                id: 5,
                name: "WhiteBalance",
                format: "string",
                groups: HashMap::new(),
                writable: true,
                notes: None,
                print_conv: PrintConvType::None,
                value_conv: None,
                subdirectory: None,
            },
        ),
        (
            6,
            TagKitDef {
                id: 6,
                name: "Sharpness",
                format: "string",
                groups: HashMap::new(),
                writable: true,
                notes: None,
                print_conv: PrintConvType::None,
                value_conv: None,
                subdirectory: None,
            },
        ),
        (
            7,
            TagKitDef {
                id: 7,
                name: "FocusMode",
                format: "string",
                groups: HashMap::new(),
                writable: true,
                notes: None,
                print_conv: PrintConvType::None,
                value_conv: None,
                subdirectory: None,
            },
        ),
        (
            8,
            TagKitDef {
                id: 8,
                name: "FlashSetting",
                format: "string",
                groups: HashMap::new(),
                writable: true,
                notes: None,
                print_conv: PrintConvType::None,
                value_conv: None,
                subdirectory: None,
            },
        ),
        (
            9,
            TagKitDef {
                id: 9,
                name: "FlashType",
                format: "string",
                groups: HashMap::new(),
                writable: true,
                notes: None,
                print_conv: PrintConvType::None,
                value_conv: None,
                subdirectory: None,
            },
        ),
    ]
}
