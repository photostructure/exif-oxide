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
    /// Rational number as numerator/denominator pair (RATIONAL format)
    /// ExifTool: Format type 5 (rational64u) - 2x uint32
    Rational(u32, u32),
    /// Signed rational number as numerator/denominator pair (SRATIONAL format)  
    /// ExifTool: Format type 10 (rational64s) - 2x int32
    SRational(i32, i32),
    /// Array of rational numbers for multi-value tags like GPS coordinates
    /// ExifTool: GPSLatitude/GPSLongitude arrays [degrees/1, minutes/1, seconds/100]
    RationalArray(Vec<(u32, u32)>),
    /// Array of signed rational numbers
    SRationalArray(Vec<(i32, i32)>),
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
            TagValue::Rational(num, denom) => {
                if *denom != 0 {
                    Some(*num as f64 / *denom as f64)
                } else {
                    None // Division by zero
                }
            }
            TagValue::SRational(num, denom) => {
                if *denom != 0 {
                    Some(*num as f64 / *denom as f64)
                } else {
                    None // Division by zero
                }
            }
            _ => None,
        }
    }

    /// Convert to rational tuple if possible
    pub fn as_rational(&self) -> Option<(u32, u32)> {
        match self {
            TagValue::Rational(num, denom) => Some((*num, *denom)),
            _ => None,
        }
    }

    /// Convert to signed rational tuple if possible
    pub fn as_srational(&self) -> Option<(i32, i32)> {
        match self {
            TagValue::SRational(num, denom) => Some((*num, *denom)),
            _ => None,
        }
    }

    /// Convert rational to decimal degrees (for GPS coordinates)
    /// GPS coordinates use degrees/1, minutes/1, seconds/100 format
    /// ExifTool: GPS.pm ToDegrees function for coordinate conversion
    pub fn rational_to_decimal(&self) -> Option<f64> {
        match self {
            TagValue::RationalArray(rationals) if rationals.len() >= 3 => {
                let degrees = if rationals[0].1 != 0 {
                    rationals[0].0 as f64 / rationals[0].1 as f64
                } else {
                    0.0
                };
                let minutes = if rationals[1].1 != 0 {
                    rationals[1].0 as f64 / rationals[1].1 as f64 / 60.0
                } else {
                    0.0
                };
                let seconds = if rationals[2].1 != 0 {
                    rationals[2].0 as f64 / rationals[2].1 as f64 / 3600.0
                } else {
                    0.0
                };
                Some(degrees + minutes + seconds)
            }
            _ => None,
        }
    }

    /// Convert GPS coordinate array to signed decimal with hemisphere
    /// ExifTool: Composite GPSLatitude/GPSLongitude with reference direction
    pub fn gps_to_decimal_with_ref(coord: &TagValue, reference: &TagValue) -> Option<f64> {
        let decimal = coord.rational_to_decimal()?;

        match reference.as_string() {
            Some("S") | Some("W") => Some(-decimal), // South/West are negative
            Some("N") | Some("E") => Some(decimal),  // North/East are positive
            _ => Some(decimal),                      // Default to positive if no valid reference
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
            TagValue::Rational(num, denom) => {
                if *denom == 0 {
                    write!(f, "{num}/0 (inf)")
                } else if *denom == 1 {
                    write!(f, "{num}")
                } else {
                    write!(f, "{num}/{denom}")
                }
            }
            TagValue::SRational(num, denom) => {
                if *denom == 0 {
                    write!(f, "{num}/0 (inf)")
                } else if *denom == 1 {
                    write!(f, "{num}")
                } else {
                    write!(f, "{num}/{denom}")
                }
            }
            TagValue::RationalArray(arr) => {
                write!(f, "[")?;
                for (i, (num, denom)) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    if *denom == 0 {
                        write!(f, "{num}/0")?;
                    } else if *denom == 1 {
                        write!(f, "{num}")?;
                    } else {
                        write!(f, "{num}/{denom}")?;
                    }
                }
                write!(f, "]")
            }
            TagValue::SRationalArray(arr) => {
                write!(f, "[")?;
                for (i, (num, denom)) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    if *denom == 0 {
                        write!(f, "{num}/0")?;
                    } else if *denom == 1 {
                        write!(f, "{num}")?;
                    } else {
                        write!(f, "{num}/{denom}")?;
                    }
                }
                write!(f, "]")
            }
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
