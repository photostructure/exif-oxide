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

    // rational_to_decimal REMOVED in Milestone 8e
    // GPS coordinate conversion moved to Composite tag system

    // gps_to_decimal_with_ref REMOVED in Milestone 8e
    // GPS coordinate conversion moved to Composite tag system
}

/// A single extracted metadata tag with both its converted value and display string.
///
/// This structure provides access to both the logical value (after ValueConv)
/// and the human-readable display string (after PrintConv), allowing consumers
/// to choose the most appropriate representation.
///
/// # Examples
///
/// ```
/// use exif_oxide::types::{TagEntry, TagValue};
///
/// // A typical EXIF tag entry
/// let entry = TagEntry {
///     group: "EXIF".to_string(),
///     name: "FNumber".to_string(),
///     value: TagValue::F64(4.0),      // Post-ValueConv: 4/1 → 4.0
///     print: "4.0".to_string(),       // Post-PrintConv: formatted for display
/// };
///
/// assert_eq!(entry.name, "FNumber");
///
/// // A tag with units in the display string
/// let focal_entry = TagEntry {
///     group: "EXIF".to_string(),
///     name: "FocalLength".to_string(),
///     value: TagValue::F64(24.0),     // Numeric value
///     print: "24 mm".to_string(),     // Human-readable with units
/// };
///
/// assert_eq!(focal_entry.print, "24 mm");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagEntry {
    /// Tag group name (e.g., "EXIF", "GPS", "Canon", "MakerNotes")
    ///
    /// Groups follow ExifTool's naming conventions:
    /// - Main IFDs: "EXIF", "GPS", "IFD0", "IFD1"
    /// - Manufacturer: "Canon", "Nikon", "Sony", etc.
    /// - Sub-groups: "Canon::CameraSettings", etc.
    pub group: String,

    /// Tag name without group prefix (e.g., "FNumber", "ExposureTime")
    ///
    /// Names match ExifTool's tag naming exactly for compatibility.
    pub name: String,

    /// The logical value after ValueConv processing.
    ///
    /// This is the value you get with ExifTool's -# flag:
    /// - Rational values converted to floats (4/1 → 4.0)
    /// - APEX values converted to real units
    /// - Raw value if no ValueConv exists
    ///
    /// # Examples
    ///
    /// - FNumber: `TagValue::F64(4.0)` (from rational 4/1)
    /// - ExposureTime: `TagValue::F64(0.0005)` (from rational 1/2000)
    /// - Make: `TagValue::String("Canon")` (no ValueConv needed)
    pub value: TagValue,

    /// The display string after PrintConv processing.
    ///
    /// This is the human-readable representation:
    /// - Numbers may be formatted ("4.0" not "4")
    /// - Units may be added ("24.0 mm")
    /// - Coded values decoded ("Rotate 90 CW" not "6")
    ///
    /// If no PrintConv exists, this equals `value.to_string()`.
    ///
    /// # ExifTool JSON Compatibility
    ///
    /// When serializing to JSON, some numeric PrintConv results
    /// (like FNumber's "4.0") are encoded as JSON numbers, not strings.
    /// The CLI handles this compatibility layer.
    pub print: String,
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

    /// All extracted tags with both value and print representations
    #[serde(skip)]
    pub tags: Vec<TagEntry>,

    /// Legacy field for backward compatibility - will be populated during serialization
    /// TODO: Remove this once all consumers are updated to use TagEntry
    #[serde(flatten)]
    pub legacy_tags: HashMap<String, TagValue>,

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

impl ExifData {
    /// Create a new ExifData with empty tags
    pub fn new(source_file: String, exif_tool_version: String) -> Self {
        Self {
            source_file,
            exif_tool_version,
            tags: Vec::new(),
            legacy_tags: HashMap::new(),
            errors: Vec::new(),
            missing_implementations: None,
        }
    }

    /// Convert tags to legacy format for JSON serialization
    /// This populates legacy_tags from the TagEntry vector
    pub fn prepare_for_serialization(
        &mut self,
        numeric_tags: Option<&std::collections::HashSet<String>>,
    ) {
        self.legacy_tags.clear();

        for entry in &self.tags {
            let key = format!("{}:{}", entry.group, entry.name);

            // Determine whether to use value or print field
            let should_use_value = numeric_tags
                .map(|set| set.contains(&entry.name))
                .unwrap_or(false);

            if should_use_value {
                // Use value field for -# tags
                self.legacy_tags.insert(key, entry.value.clone());
            } else {
                // Use PrintConv (human-readable string) unless PrintConv is missing/invalid
                // If PrintConv provides meaningful conversion, use it; otherwise use ValueConv
                let value_as_string = format!("{}", entry.value);
                if entry.print != value_as_string && !entry.print.is_empty() {
                    // PrintConv provides meaningful conversion (like "Rotate 270 CW" instead of "6")
                    self.legacy_tags
                        .insert(key, TagValue::String(entry.print.clone()));
                } else {
                    // No meaningful PrintConv available, use ValueConv to preserve data types
                    self.legacy_tags.insert(key, entry.value.clone());
                }
            }
        }
    }
}

/// Error types for exif-oxide
// TODO: Enhance error types to match ExifTool's sophisticated error classification system (warnings, errors, fatal)
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
    /// Sony manufacturer-specific processing
    Sony(SonyProcessor),
    /// Generic manufacturer processing
    Generic(String),
}

/// Canon-specific processor variants
/// ExifTool: Canon.pm has multiple processing procedures
#[derive(Debug, Clone, PartialEq)]
pub enum CanonProcessor {
    /// Standard Canon EXIF processing
    Main,
    /// Canon CameraSettings processing
    /// ExifTool: ProcessBinaryData for CameraSettings table
    CameraSettings,
    /// Canon AFInfo processing
    /// ExifTool: ProcessSerialData for AFInfo table
    AfInfo,
    /// Canon AFInfo2 processing  
    /// ExifTool: ProcessSerialData for AFInfo2 table
    AfInfo2,
    /// Canon serial data processing (generic)
    /// ExifTool: ProcessSerialData
    SerialData,
    /// Canon binary data processing (generic)
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

/// Sony-specific processor variants
/// ExifTool: Sony.pm has multiple processing procedures and signature detection
#[derive(Debug, Clone, PartialEq)]
pub enum SonyProcessor {
    /// Standard Sony EXIF processing with MakerNotes namespace
    /// ExifTool: Image::ExifTool::Sony::Main
    Main,
    /// Sony PIC format processing
    /// ExifTool: Image::ExifTool::Sony::PIC (DSC-H200/J20/W370/W510, MHS-TS20)
    Pic,
    /// Sony SRF format processing  
    /// ExifTool: Image::ExifTool::Sony::SRF
    Srf,
    /// Sony Ericsson mobile phone format
    /// ExifTool: Image::ExifTool::Sony::Ericsson
    Ericsson,
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

/// Source priority for tag conflict resolution
/// Higher numbers take precedence over lower numbers
/// ExifTool behavior: Main EXIF tags override MakerNote tags with same ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SourcePriority {
    /// Unknown or unrecognized source (lowest priority)
    Unknown = 10,
    /// MakerNote tags (manufacturer-specific data)
    MakerNotes = 50,
    /// GPS IFD tags
    Gps = 80,
    /// Main EXIF tags (highest priority)
    /// ExifTool: IFD0, IFD1, ExifIFD, etc.
    Exif = 100,
}

impl SourcePriority {
    /// Get priority for a namespace string
    /// Matches ExifTool's group hierarchy behavior
    pub fn from_namespace(namespace: &str) -> Self {
        match namespace {
            "EXIF" | "IFD0" | "IFD1" | "ExifIFD" | "SubIFD" => SourcePriority::Exif,
            "GPS" => SourcePriority::Gps,
            "MakerNotes" => SourcePriority::MakerNotes,
            _ => SourcePriority::Unknown,
        }
    }
}

/// Enhanced tag source information for conflict resolution
/// Tracks where each tag came from and its processing context
#[derive(Debug, Clone)]
pub struct TagSourceInfo {
    /// Namespace/group for the tag (e.g., "EXIF", "MakerNotes", "GPS")
    /// ExifTool: Group 0 in tag name "Group:TagName"
    pub namespace: String,
    /// Specific IFD or table name (e.g., "IFD0", "ExifIFD", "Canon::Main")
    /// ExifTool: Directory path for debugging and processing context
    pub ifd_name: String,
    /// Source priority for conflict resolution
    /// ExifTool: Main EXIF tags take precedence over MakerNote tags
    pub priority: SourcePriority,
    /// Processor type that handled this tag
    /// ExifTool: PROCESS_PROC information
    pub processor_type: ProcessorType,
}

impl TagSourceInfo {
    /// Create new tag source info
    pub fn new(namespace: String, ifd_name: String, processor_type: ProcessorType) -> Self {
        let priority = SourcePriority::from_namespace(&namespace);
        Self {
            namespace,
            ifd_name,
            priority,
            processor_type,
        }
    }

    /// Get the full tag name with namespace prefix
    /// ExifTool format: "Group:TagName"
    pub fn format_tag_name(&self, tag_name: &str) -> String {
        format!("{}:{}", self.namespace, tag_name)
    }
}
