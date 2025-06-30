//! Core data types for exif-oxide
//!
//! This module defines the fundamental types used throughout the system,
//! including tag values, metadata structures, and error types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Represents a tag value that can be of various types
///
/// ExifTool handles many different data types. This enum represents
/// the possible values a tag can contain after parsing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TagValue {
    /// Unsigned 8-bit integer
    U8(u8),
    /// Unsigned 16-bit integer  
    U16(u16),
    /// Unsigned 32-bit integer
    U32(u32),
    /// Signed 16-bit integer
    I16(i16),
    /// Signed 32-bit integer
    I32(i32),
    /// Floating point number
    F64(f64),
    /// Text string
    String(String),
    /// Array of unsigned 8-bit integers (for binary data)
    U8Array(Vec<u8>),
    /// Array of unsigned 16-bit integers
    U16Array(Vec<u16>),
    /// Array of unsigned 32-bit integers  
    U32Array(Vec<u32>),
    /// Array of floating point numbers (for rational arrays)
    F64Array(Vec<f64>),
    /// Raw binary data when type is unknown
    Binary(Vec<u8>),
}

impl TagValue {
    /// Convert to u8 if possible
    pub fn as_u8(&self) -> Option<u8> {
        match self {
            TagValue::U8(v) => Some(*v),
            _ => None,
        }
    }

    /// Convert to u16 if possible
    pub fn as_u16(&self) -> Option<u16> {
        match self {
            TagValue::U16(v) => Some(*v),
            TagValue::U8(v) => Some(*v as u16),
            _ => None,
        }
    }

    /// Convert to u32 if possible
    pub fn as_u32(&self) -> Option<u32> {
        match self {
            TagValue::U32(v) => Some(*v),
            TagValue::U16(v) => Some(*v as u32),
            TagValue::U8(v) => Some(*v as u32),
            _ => None,
        }
    }

    /// Convert to string if possible
    pub fn as_string(&self) -> Option<&str> {
        match self {
            TagValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Convert to f64 if possible
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            TagValue::F64(v) => Some(*v),
            TagValue::U32(v) => Some(*v as f64),
            TagValue::U16(v) => Some(*v as f64),
            TagValue::U8(v) => Some(*v as f64),
            _ => None,
        }
    }
}

impl std::fmt::Display for TagValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TagValue::U8(v) => write!(f, "{v}"),
            TagValue::U16(v) => write!(f, "{v}"),
            TagValue::U32(v) => write!(f, "{v}"),
            TagValue::I16(v) => write!(f, "{v}"),
            TagValue::I32(v) => write!(f, "{v}"),
            TagValue::F64(v) => write!(f, "{v}"),
            TagValue::String(s) => write!(f, "{s}"),
            TagValue::U8Array(arr) => write!(f, "{arr:?}"),
            TagValue::U16Array(arr) => write!(f, "{arr:?}"),
            TagValue::U32Array(arr) => write!(f, "{arr:?}"),
            TagValue::F64Array(arr) => write!(f, "{arr:?}"),
            TagValue::Binary(data) => write!(f, "[{} bytes of binary data]", data.len()),
        }
    }
}

/// Represents extracted EXIF data from an image
///
/// This matches ExifTool's JSON output structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExifData {
    /// Source file path
    #[serde(rename = "SourceFile")]
    pub source_file: String,

    /// Version of exif-oxide
    #[serde(rename = "ExifToolVersion")]
    pub exif_tool_version: String,

    /// All extracted tags as key-value pairs
    #[serde(flatten)]
    pub tags: HashMap<String, TagValue>,

    /// Any errors encountered during processing
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,

    /// Missing implementations (only included with --show-missing)
    #[serde(
        rename = "MissingImplementations",
        skip_serializing_if = "Option::is_none"
    )]
    pub missing_implementations: Option<Vec<String>>,
}

/// Error types for exif-oxide
#[derive(Error, Debug)]
pub enum ExifError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid file format: {0}")]
    InvalidFormat(String),

    #[error("Parsing error: {0}")]
    ParseError(String),

    #[error("Unsupported feature: {0}")]
    Unsupported(String),

    #[error("Registry error: {0}")]
    Registry(String),
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, ExifError>;
