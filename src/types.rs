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

/// Directory processing context for nested IFD processing
/// Matches ExifTool's $dirInfo hash structure
#[derive(Debug, Clone)]
pub struct DirectoryInfo {
    /// Directory name for debugging and PATH tracking
    pub name: String,
    /// Start offset of directory within data
    pub dir_start: usize,
    /// Length of directory data
    pub dir_len: usize,
    /// Base offset for pointer calculations (ExifTool's Base)
    pub base: u64,
    /// File position of data block (ExifTool's DataPos)
    pub data_pos: u64,
    /// Whether this directory allows reprocessing (ALLOW_REPROCESS)
    pub allow_reprocess: bool,
}

/// Data member value for tag dependencies
/// ExifTool: DataMember mechanism for inter-tag dependencies
#[derive(Debug, Clone, PartialEq)]
pub enum DataMemberValue {
    U8(u8),
    U16(u16),
    U32(u32),
    String(String),
}

impl DataMemberValue {
    pub fn as_u16(&self) -> Option<u16> {
        match self {
            DataMemberValue::U16(v) => Some(*v),
            DataMemberValue::U8(v) => Some(*v as u16),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self {
            DataMemberValue::U32(v) => Some(*v),
            DataMemberValue::U16(v) => Some(*v as u32),
            DataMemberValue::U8(v) => Some(*v as u32),
            _ => None,
        }
    }

    pub fn as_usize(&self) -> Option<usize> {
        match self {
            DataMemberValue::U32(v) => Some(*v as usize),
            DataMemberValue::U16(v) => Some(*v as usize),
            DataMemberValue::U8(v) => Some(*v as usize),
            _ => None,
        }
    }
}

/// Processor types for PROCESS_PROC dispatch system
/// ExifTool: Different processing procedures for different data formats
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessorType {
    /// Standard EXIF IFD processing (default)
    /// ExifTool: ProcessExif function
    Exif,
    /// Binary data processing with format tables
    /// ExifTool: ProcessBinaryData function
    BinaryData,
    /// GPS IFD processing
    /// ExifTool: Uses ProcessExif but with GPS-specific context
    Gps,
    /// Canon manufacturer-specific processing
    Canon(CanonProcessor),
    /// Nikon manufacturer-specific processing  
    Nikon(NikonProcessor),
    /// Generic manufacturer processing
    Generic(String),
}

/// Canon-specific processor variants
/// ExifTool: Canon.pm has multiple processing procedures
#[derive(Debug, Clone, PartialEq)]
pub enum CanonProcessor {
    /// Standard Canon EXIF processing
    Main,
    /// Canon serial data processing
    /// ExifTool: ProcessSerialData
    SerialData,
    /// Canon binary data processing
    BinaryData,
}

/// Nikon-specific processor variants
/// ExifTool: Nikon.pm has multiple processing procedures
#[derive(Debug, Clone, PartialEq)]
pub enum NikonProcessor {
    /// Standard Nikon EXIF processing
    Main,
    /// Nikon encrypted data processing
    /// ExifTool: ProcessNikonEncrypted
    Encrypted,
}

/// Processor dispatch configuration
/// ExifTool: Combination of table PROCESS_PROC and SubDirectory ProcessProc
#[derive(Debug, Clone)]
pub struct ProcessorDispatch {
    /// Table-level default processor
    /// ExifTool: $$tagTablePtr{PROCESS_PROC}
    pub table_processor: Option<ProcessorType>,
    /// SubDirectory-specific processor overrides
    /// ExifTool: $$subdir{ProcessProc}
    pub subdirectory_overrides: std::collections::HashMap<u16, ProcessorType>,
    /// Parameters passed to processor
    /// ExifTool: Additional SubDirectory parameters
    pub parameters: std::collections::HashMap<String, String>,
}

impl Default for ProcessorDispatch {
    fn default() -> Self {
        Self {
            table_processor: Some(ProcessorType::Exif), // Default fallback
            subdirectory_overrides: std::collections::HashMap::new(),
            parameters: std::collections::HashMap::new(),
        }
    }
}
