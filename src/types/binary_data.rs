//! Binary data processing types for ProcessBinaryData
//!
//! This module defines the types used for ExifTool's ProcessBinaryData
//! functionality, including format definitions and table structures.

use crate::types::ExifError;
use std::collections::HashMap;

/// Binary data formats for ProcessBinaryData
/// ExifTool: lib/Image/ExifTool.pm %formatSize and @formatName arrays
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryDataFormat {
    /// Unsigned 8-bit integer
    /// ExifTool: int8u
    Int8u,
    /// Signed 8-bit integer
    /// ExifTool: int8s
    Int8s,
    /// Unsigned 16-bit integer
    /// ExifTool: int16u
    Int16u,
    /// Signed 16-bit integer
    /// ExifTool: int16s
    Int16s,
    /// Unsigned 32-bit integer
    /// ExifTool: int32u
    Int32u,
    /// Signed 32-bit integer
    /// ExifTool: int32s
    Int32s,
    /// 32-bit floating point
    /// ExifTool: float
    Float,
    /// 64-bit floating point
    /// ExifTool: double
    Double,
    /// Null-terminated string
    /// ExifTool: string
    String,
    /// Pascal string (first byte is length)
    /// ExifTool: pstring
    PString,
    /// Binary/undefined data
    /// ExifTool: undef
    Undef,
}

impl BinaryDataFormat {
    /// Get byte size for this format
    /// ExifTool: lib/Image/ExifTool.pm %formatSize array
    pub fn byte_size(self) -> usize {
        match self {
            BinaryDataFormat::Int8u | BinaryDataFormat::Int8s | BinaryDataFormat::Undef => 1,
            BinaryDataFormat::Int16u | BinaryDataFormat::Int16s => 2,
            BinaryDataFormat::Int32u | BinaryDataFormat::Int32s | BinaryDataFormat::Float => 4,
            BinaryDataFormat::Double => 8,
            BinaryDataFormat::String | BinaryDataFormat::PString => 1, // Variable length
        }
    }

    /// Parse format string to enum
    /// ExifTool: lib/Image/ExifTool.pm format name lookup
    pub fn parse_format(format: &str) -> std::result::Result<Self, ExifError> {
        match format {
            "int8u" => Ok(BinaryDataFormat::Int8u),
            "int8s" => Ok(BinaryDataFormat::Int8s),
            "int16u" => Ok(BinaryDataFormat::Int16u),
            "int16s" => Ok(BinaryDataFormat::Int16s),
            "int32u" => Ok(BinaryDataFormat::Int32u),
            "int32s" => Ok(BinaryDataFormat::Int32s),
            "float" => Ok(BinaryDataFormat::Float),
            "double" => Ok(BinaryDataFormat::Double),
            "string" => Ok(BinaryDataFormat::String),
            "pstring" => Ok(BinaryDataFormat::PString),
            "undef" => Ok(BinaryDataFormat::Undef),
            _ => Err(ExifError::ParseError(format!(
                "Unknown binary data format: {format}"
            ))),
        }
    }
}

/// Binary data table configuration
/// ExifTool: Tag table with PROCESS_PROC => \&ProcessBinaryData
#[derive(Debug, Clone)]
pub struct BinaryDataTable {
    /// Default format for entries (ExifTool: FORMAT key)
    pub default_format: BinaryDataFormat,
    /// Starting index for unknown tag generation (ExifTool: FIRST_ENTRY key)
    pub first_entry: Option<u32>,
    /// Group hierarchy for tags (ExifTool: GROUPS key)
    pub groups: HashMap<u8, String>,
    /// Tag definitions indexed by position
    pub tags: HashMap<u32, BinaryDataTag>,
}

/// Individual tag definition in binary data table
/// ExifTool: Tag info hash structure
#[derive(Debug, Clone)]
pub struct BinaryDataTag {
    /// Tag name
    pub name: String,
    /// Data format override (None uses table default)
    pub format: Option<BinaryDataFormat>,
    /// Bit mask for extracting value
    pub mask: Option<u32>,
    /// PrintConv lookup table
    pub print_conv: Option<HashMap<u32, String>>,
}

impl Default for BinaryDataTable {
    fn default() -> Self {
        Self {
            default_format: BinaryDataFormat::Int8u,
            first_entry: None,
            groups: HashMap::new(),
            tags: HashMap::new(),
        }
    }
}
