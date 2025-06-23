//! Core type definitions for EXIF parsing

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm"]

use serde::Serialize;
use serde_json;

/// EXIF data formats as defined in TIFF specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ExifFormat {
    /// 8-bit unsigned integer
    U8 = 1,
    /// ASCII string (null-terminated)
    Ascii = 2,
    /// 16-bit unsigned integer
    U16 = 3,
    /// 32-bit unsigned integer
    U32 = 4,
    /// Rational (2 x U32: numerator, denominator)
    Rational = 5,
    /// 8-bit signed integer
    I8 = 6,
    /// Undefined (raw bytes)
    Undefined = 7,
    /// 16-bit signed integer
    I16 = 8,
    /// 32-bit signed integer
    I32 = 9,
    /// Signed rational (2 x I32)
    SignedRational = 10,
    /// 32-bit float
    F32 = 11,
    /// 64-bit float
    F64 = 12,
}

impl ExifFormat {
    /// Get the size in bytes of one component of this format
    pub fn size(&self) -> usize {
        match self {
            Self::U8 | Self::Ascii | Self::I8 | Self::Undefined => 1,
            Self::U16 | Self::I16 => 2,
            Self::U32 | Self::I32 | Self::F32 => 4,
            Self::Rational | Self::SignedRational | Self::F64 => 8,
        }
    }

    /// Create format from u16 value
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            1 => Some(Self::U8),
            2 => Some(Self::Ascii),
            3 => Some(Self::U16),
            4 => Some(Self::U32),
            5 => Some(Self::Rational),
            6 => Some(Self::I8),
            7 => Some(Self::Undefined),
            8 => Some(Self::I16),
            9 => Some(Self::I32),
            10 => Some(Self::SignedRational),
            11 => Some(Self::F32),
            12 => Some(Self::F64),
            _ => None,
        }
    }
}

/// A single IFD (Image File Directory) entry
#[derive(Debug, Clone)]
pub struct IfdEntry {
    pub tag: u16,
    pub format: ExifFormat,
    pub count: u32,
    pub value_offset: u32,
}

/// Parsed EXIF value
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ExifValue {
    /// ASCII string
    Ascii(String),
    /// Single unsigned 8-bit integer
    U8(u8),
    /// Multiple unsigned 8-bit integers
    U8Array(Vec<u8>),
    /// Single unsigned 16-bit integer
    U16(u16),
    /// Multiple unsigned 16-bit integers
    U16Array(Vec<u16>),
    /// Single unsigned 32-bit integer
    U32(u32),
    /// Multiple unsigned 32-bit integers
    U32Array(Vec<u32>),
    /// Single signed 16-bit integer
    I16(i16),
    /// Multiple signed 16-bit integers
    I16Array(Vec<i16>),
    /// Single signed 32-bit integer
    I32(i32),
    /// Multiple signed 32-bit integers
    I32Array(Vec<i32>),
    /// Single unsigned rational (numerator, denominator)
    Rational(u32, u32),
    /// Multiple unsigned rationals
    RationalArray(Vec<(u32, u32)>),
    /// Single signed rational (numerator, denominator)
    SignedRational(i32, i32),
    /// Multiple signed rationals
    SignedRationalArray(Vec<(i32, i32)>),
    /// Raw byte data
    Undefined(Vec<u8>),
    /// Binary data with length information (instead of actual data)
    BinaryData(usize),
}

impl ExifValue {
    /// Convert ExifValue to JSON string representation for compatibility with main.rs output
    pub fn to_json_string(&self) -> String {
        match self {
            ExifValue::Ascii(s) => serde_json::json!({"Ascii": s}).to_string(),
            ExifValue::U8(n) => serde_json::json!({"U8": n}).to_string(),
            ExifValue::U8Array(arr) => serde_json::json!({"U8Array": arr}).to_string(),
            ExifValue::U16(n) => serde_json::json!({"U16": n}).to_string(),
            ExifValue::U16Array(arr) => serde_json::json!({"U16Array": arr}).to_string(),
            ExifValue::U32(n) => serde_json::json!({"U32": n}).to_string(),
            ExifValue::U32Array(arr) => serde_json::json!({"U32Array": arr}).to_string(),
            ExifValue::I16(n) => serde_json::json!({"I16": n}).to_string(),
            ExifValue::I16Array(arr) => serde_json::json!({"I16Array": arr}).to_string(),
            ExifValue::I32(n) => serde_json::json!({"I32": n}).to_string(),
            ExifValue::I32Array(arr) => serde_json::json!({"I32Array": arr}).to_string(),
            ExifValue::Rational(num, den) => {
                serde_json::json!({"Rational": [num, den]}).to_string()
            }
            ExifValue::RationalArray(arr) => {
                let tuples: Vec<[u32; 2]> = arr.iter().map(|(n, d)| [*n, *d]).collect();
                serde_json::json!({"RationalArray": tuples}).to_string()
            }
            ExifValue::SignedRational(num, den) => {
                serde_json::json!({"SignedRational": [num, den]}).to_string()
            }
            ExifValue::SignedRationalArray(arr) => {
                let tuples: Vec<[i32; 2]> = arr.iter().map(|(n, d)| [*n, *d]).collect();
                serde_json::json!({"SignedRationalArray": tuples}).to_string()
            }
            ExifValue::Undefined(data) => serde_json::json!({"Undefined": data}).to_string(),
            ExifValue::BinaryData(len) => serde_json::json!({"BinaryData": len}).to_string(),
        }
    }
}

/// Tag information from ExifTool tables
#[derive(Debug, Clone)]
pub struct TagInfo {
    pub name: &'static str,
    pub format: ExifFormat,
    pub group: Option<&'static str>,
}
