//! Tag kits for color category from Canon.pm
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

/// Get tag definitions for color category
pub fn get_color_tags() -> Vec<(u32, TagKitDef)> {
    vec![
        (
            333,
            TagKitDef {
                id: 333,
                name: "PerChannelBlackLevel",
                format: "int16s[4]",
                groups: HashMap::new(),
                writable: false,
                notes: None,
                print_conv: PrintConvType::None,
                value_conv: None,
                subdirectory: None,
            },
        ),
        (
            326,
            TagKitDef {
                id: 326,
                name: "AverageBlackLevel",
                format: "int16u[4]",
                groups: HashMap::new(),
                writable: false,
                notes: None,
                print_conv: PrintConvType::None,
                value_conv: None,
                subdirectory: None,
            },
        ),
        (
            329,
            TagKitDef {
                id: 329,
                name: "PerChannelBlackLevel",
                format: "int16u[4]",
                groups: HashMap::new(),
                writable: false,
                notes: None,
                print_conv: PrintConvType::None,
                value_conv: None,
                subdirectory: None,
            },
        ),
        (
            320,
            TagKitDef {
                id: 320,
                name: "ColorCalib",
                format: "undef[120]",
                groups: HashMap::new(),
                writable: false,
                notes: None,
                print_conv: PrintConvType::None,
                value_conv: None,
                subdirectory: Some(SubDirectoryType::Binary {
                    processor: process_tag_0x140_subdirectory,
                }),
            },
        ),
    ]
}
