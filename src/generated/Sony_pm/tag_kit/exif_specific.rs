//! Tag kits for exif_specific category from Sony.pm
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

/// Get tag definitions for exif_specific category
pub fn get_exif_specific_tags() -> Vec<(u32, TagKitDef)> {
    vec![
        (
            36875,
            TagKitDef {
                id: 36875,
                name: "Tag900b",
                format: "unknown",
                groups: HashMap::new(),
                writable: false,
                notes: None,
                print_conv: PrintConvType::None,
                value_conv: None,
                subdirectory: Some(SubDirectoryType::Binary {
                    processor: process_tag_0x900b_subdirectory,
                }),
            },
        ),
        (
            36944,
            TagKitDef {
                id: 36944,
                name: "Tag9050a",
                format: "unknown",
                groups: HashMap::new(),
                writable: false,
                notes: None,
                print_conv: PrintConvType::None,
                value_conv: None,
                subdirectory: Some(SubDirectoryType::Binary {
                    processor: process_tag_0x9050_subdirectory,
                }),
            },
        ),
        (
            36944,
            TagKitDef {
                id: 36944,
                name: "Tag9050b",
                format: "unknown",
                groups: HashMap::new(),
                writable: false,
                notes: None,
                print_conv: PrintConvType::None,
                value_conv: None,
                subdirectory: Some(SubDirectoryType::Binary {
                    processor: process_tag_0x9050_subdirectory,
                }),
            },
        ),
        (
            36944,
            TagKitDef {
                id: 36944,
                name: "Tag9050c",
                format: "unknown",
                groups: HashMap::new(),
                writable: false,
                notes: None,
                print_conv: PrintConvType::None,
                value_conv: None,
                subdirectory: Some(SubDirectoryType::Binary {
                    processor: process_tag_0x9050_subdirectory,
                }),
            },
        ),
        (
            36944,
            TagKitDef {
                id: 36944,
                name: "Tag9050d",
                format: "unknown",
                groups: HashMap::new(),
                writable: false,
                notes: None,
                print_conv: PrintConvType::None,
                value_conv: None,
                subdirectory: Some(SubDirectoryType::Binary {
                    processor: process_tag_0x9050_subdirectory,
                }),
            },
        ),
        (
            36944,
            TagKitDef {
                id: 36944,
                name: "Sony_0x9050",
                format: "unknown",
                groups: HashMap::new(),
                writable: false,
                notes: None,
                print_conv: PrintConvType::Manual("code_ref_printconv"),
                value_conv: Some("PrintHex($val)"),
                subdirectory: None,
            },
        ),
    ]
}
