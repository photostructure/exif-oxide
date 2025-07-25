//! Tag kits for thumbnail category from Exif.pm
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

/// Get tag definitions for thumbnail category
pub fn get_thumbnail_tags() -> Vec<(u32, TagKitDef)> {
    vec![
        (20507, TagKitDef {
            id: 20507,
            name: "ThumbnailData",
            format: "undef",
            groups: HashMap::new(),
            writable: false,
            notes: None,
            print_conv: PrintConvType::None,
            value_conv: None,
            subdirectory: None,
        }),
        (513, TagKitDef {
            id: 513,
            name: "ThumbnailOffset",
            format: "int32u",
            groups: HashMap::new(),
            writable: true,
            notes: Some("called JPEGInterchangeFormat in the specification, this is ThumbnailOffset\n                in IFD1 of JPEG and some TIFF-based images, IFD0 of MRW images and AVI and\n                MOV videos, and the SubIFD in IFD1 of SRW images; PreviewImageStart in\n                MakerNotes and IFD0 of ARW and SR2 images; JpgFromRawStart in SubIFD of NEF\n                images and IFD2 of PEF images; and OtherImageStart in everything else"),
            print_conv: PrintConvType::None,
            value_conv: None,
            subdirectory: None,
        }),
        (513, TagKitDef {
            id: 513,
            name: "ThumbnailOffset",
            format: "int32u",
            groups: HashMap::new(),
            writable: true,
            notes: None,
            print_conv: PrintConvType::None,
            value_conv: None,
            subdirectory: None,
        }),
        (513, TagKitDef {
            id: 513,
            name: "ThumbnailOffset",
            format: "int32u",
            groups: HashMap::new(),
            writable: true,
            notes: None,
            print_conv: PrintConvType::None,
            value_conv: None,
            subdirectory: None,
        }),
        (513, TagKitDef {
            id: 513,
            name: "PreviewImageStart",
            format: "int32u",
            groups: HashMap::new(),
            writable: true,
            notes: None,
            print_conv: PrintConvType::None,
            value_conv: None,
            subdirectory: None,
        }),
        (513, TagKitDef {
            id: 513,
            name: "PreviewImageStart",
            format: "int32u",
            groups: HashMap::new(),
            writable: true,
            notes: None,
            print_conv: PrintConvType::None,
            value_conv: None,
            subdirectory: None,
        }),
        (513, TagKitDef {
            id: 513,
            name: "JpgFromRawStart",
            format: "int32u",
            groups: HashMap::new(),
            writable: true,
            notes: None,
            print_conv: PrintConvType::None,
            value_conv: None,
            subdirectory: None,
        }),
        (513, TagKitDef {
            id: 513,
            name: "JpgFromRawStart",
            format: "int32u",
            groups: HashMap::new(),
            writable: true,
            notes: None,
            print_conv: PrintConvType::None,
            value_conv: None,
            subdirectory: None,
        }),
        (513, TagKitDef {
            id: 513,
            name: "OtherImageStart",
            format: "int32u",
            groups: HashMap::new(),
            writable: true,
            notes: None,
            print_conv: PrintConvType::None,
            value_conv: None,
            subdirectory: None,
        }),
        (513, TagKitDef {
            id: 513,
            name: "OtherImageStart",
            format: "int32u",
            groups: HashMap::new(),
            writable: true,
            notes: None,
            print_conv: PrintConvType::None,
            value_conv: None,
            subdirectory: None,
        }),
        (513, TagKitDef {
            id: 513,
            name: "OtherImageStart",
            format: "unknown",
            groups: HashMap::new(),
            writable: false,
            notes: None,
            print_conv: PrintConvType::None,
            value_conv: None,
            subdirectory: None,
        }),
        (514, TagKitDef {
            id: 514,
            name: "ThumbnailLength",
            format: "int32u",
            groups: HashMap::new(),
            writable: true,
            notes: Some("called JPEGInterchangeFormatLength in the specification, this is\n                ThumbnailLength in IFD1 of JPEG and some TIFF-based images, IFD0 of MRW\n                images and AVI and MOV videos, and the SubIFD in IFD1 of SRW images;\n                PreviewImageLength in MakerNotes and IFD0 of ARW and SR2 images;\n                JpgFromRawLength in SubIFD of NEF images, and IFD2 of PEF images; and\n                OtherImageLength in everything else"),
            print_conv: PrintConvType::None,
            value_conv: None,
            subdirectory: None,
        }),
        (514, TagKitDef {
            id: 514,
            name: "ThumbnailLength",
            format: "int32u",
            groups: HashMap::new(),
            writable: true,
            notes: None,
            print_conv: PrintConvType::None,
            value_conv: None,
            subdirectory: None,
        }),
        (514, TagKitDef {
            id: 514,
            name: "ThumbnailLength",
            format: "int32u",
            groups: HashMap::new(),
            writable: true,
            notes: None,
            print_conv: PrintConvType::None,
            value_conv: None,
            subdirectory: None,
        }),
        (514, TagKitDef {
            id: 514,
            name: "PreviewImageLength",
            format: "int32u",
            groups: HashMap::new(),
            writable: true,
            notes: None,
            print_conv: PrintConvType::None,
            value_conv: None,
            subdirectory: None,
        }),
        (514, TagKitDef {
            id: 514,
            name: "PreviewImageLength",
            format: "int32u",
            groups: HashMap::new(),
            writable: true,
            notes: None,
            print_conv: PrintConvType::None,
            value_conv: None,
            subdirectory: None,
        }),
        (514, TagKitDef {
            id: 514,
            name: "JpgFromRawLength",
            format: "int32u",
            groups: HashMap::new(),
            writable: true,
            notes: None,
            print_conv: PrintConvType::None,
            value_conv: None,
            subdirectory: None,
        }),
        (514, TagKitDef {
            id: 514,
            name: "JpgFromRawLength",
            format: "int32u",
            groups: HashMap::new(),
            writable: true,
            notes: None,
            print_conv: PrintConvType::None,
            value_conv: None,
            subdirectory: None,
        }),
        (514, TagKitDef {
            id: 514,
            name: "OtherImageLength",
            format: "int32u",
            groups: HashMap::new(),
            writable: true,
            notes: None,
            print_conv: PrintConvType::None,
            value_conv: None,
            subdirectory: None,
        }),
        (514, TagKitDef {
            id: 514,
            name: "OtherImageLength",
            format: "int32u",
            groups: HashMap::new(),
            writable: true,
            notes: None,
            print_conv: PrintConvType::None,
            value_conv: None,
            subdirectory: None,
        }),
        (514, TagKitDef {
            id: 514,
            name: "OtherImageLength",
            format: "unknown",
            groups: HashMap::new(),
            writable: false,
            notes: None,
            print_conv: PrintConvType::None,
            value_conv: None,
            subdirectory: None,
        }),
    ]
}
