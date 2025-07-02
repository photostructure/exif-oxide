//! Metadata structures for EXIF data representation
//!
//! This module defines the core metadata structures including TagEntry,
//! ExifData, and TagSourceInfo that represent extracted EXIF information.

use crate::types::{ProcessorType, TagValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
