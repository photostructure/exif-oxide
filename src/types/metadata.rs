//! Metadata structures for EXIF data representation
//!
//! This module defines the core metadata structures including TagEntry,
//! ExifData, and TagSourceInfo that represent extracted EXIF information.

use crate::types::TagValue;
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
///     group1: "ExifIFD".to_string(),  // Located in ExifIFD subdirectory
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
///     group1: "ExifIFD".to_string(),
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
    ///
    /// This corresponds to ExifTool's Group0 (format family).
    pub group: String,

    /// ExifTool Group1 (subdirectory location)
    ///
    /// Identifies the specific IFD or subdirectory where the tag was found:
    /// - "IFD0" - Main image IFD
    /// - "ExifIFD" - EXIF subdirectory (tag 0x8769)
    /// - "GPS" - GPS subdirectory (tag 0x8825)
    /// - "InteropIFD" - Interoperability subdirectory (tag 0xa005)
    /// - "MakerNotes" - Manufacturer-specific subdirectory (tag 0x927c)
    ///
    /// This field enables ExifTool-compatible group-based tag access patterns.
    pub group1: String,

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

    /// The display value after PrintConv processing.
    ///
    /// This can be either:
    /// - A string for human-readable formatting (e.g., "1/100", "24.0 mm", "Rotate 90 CW")
    /// - A numeric value for data that should remain numeric in JSON (e.g., ISO: 100, FNumber: 4.0)
    ///
    /// PrintConv functions decide the appropriate type based on the tag's semantics:
    /// - Display-oriented tags return strings
    /// - Data-oriented tags may pass through numeric values
    ///
    /// If no PrintConv exists, this equals the original `value`.
    ///
    /// # Design Note
    ///
    /// This differs from ExifTool where PrintConv always returns strings.
    /// We chose this approach to avoid regex-based type guessing during JSON serialization.
    /// See docs/design/PRINTCONV-DESIGN-DECISIONS.md for details.
    pub print: TagValue,
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
        use tracing::debug;

        self.legacy_tags.clear();

        for entry in &self.tags {
            let key = format!("{}:{}", entry.group, entry.name);

            // Determine whether to use value or print field
            let should_use_value = numeric_tags
                .map(|set| set.contains(&entry.name))
                .unwrap_or(false);

            if should_use_value {
                // Use value field for -# tags
                debug!("Tag {}: using numeric value {:?}", key, entry.value);
                self.legacy_tags.insert(key, entry.value.clone());
            } else {
                // Use PrintConv result directly - it already has the correct type
                // (string for display values, numeric for data values)
                debug!("Tag {}: using print value {:?}", key, entry.print);
                self.legacy_tags.insert(key, entry.print.clone());
            }
        }
    }

    /// Get all ExifIFD tags specifically
    /// ExifTool compatibility: access tags by Group1 location
    pub fn get_exif_ifd_tags(&self) -> Vec<&TagEntry> {
        self.tags
            .iter()
            .filter(|tag| tag.group1 == "ExifIFD")
            .collect()
    }

    /// Get all tags from a specific Group1 (subdirectory location)
    /// ExifTool: Group1-based filtering
    ///
    /// # Examples
    /// ```no_run
    /// use exif_oxide::formats::extract_metadata;
    ///
    /// let exif_data = extract_metadata(std::path::Path::new("image.jpg"), false).unwrap();
    ///
    /// // Get all GPS tags
    /// let gps_tags = exif_data.get_tags_by_group1("GPS");
    ///
    /// // Get all ExifIFD tags
    /// let exif_ifd_tags = exif_data.get_tags_by_group1("ExifIFD");
    /// ```
    pub fn get_tags_by_group1(&self, group1_name: &str) -> Vec<&TagEntry> {
        self.tags
            .iter()
            .filter(|tag| tag.group1 == group1_name)
            .collect()
    }

    /// ExifTool compatibility: get tag by group-qualified name
    /// Supports both Group0 and Group1 based access
    ///
    /// # Examples
    /// ```no_run
    /// use exif_oxide::formats::extract_metadata;
    ///
    /// let exif_data = extract_metadata(std::path::Path::new("image.jpg"), false).unwrap();
    ///
    /// // Access by Group1 (subdirectory location)
    /// let exposure_time = exif_data.get_tag_by_group("ExifIFD", "ExposureTime");
    ///
    /// // Access by Group0 (format family)
    /// let make = exif_data.get_tag_by_group("EXIF", "Make");
    /// ```
    pub fn get_tag_by_group(&self, group_name: &str, tag_name: &str) -> Option<&TagEntry> {
        self.tags.iter().find(|tag| {
            (tag.group == group_name || tag.group1 == group_name) && tag.name == tag_name
        })
    }

    /// ExifTool-style group access: EXIF:ExposureTime vs ExifIFD:ExposureTime
    /// Parses qualified tag names in "Group:TagName" format
    ///
    /// # Examples
    /// ```no_run
    /// use exif_oxide::formats::extract_metadata;
    ///
    /// let exif_data = extract_metadata(std::path::Path::new("image.jpg"), false).unwrap();
    ///
    /// let exposure_time = exif_data.get_tag_exiftool_style("ExifIFD:ExposureTime");
    /// let gps_lat = exif_data.get_tag_exiftool_style("GPS:GPSLatitude");
    /// ```
    pub fn get_tag_exiftool_style(&self, qualified_name: &str) -> Option<&TagEntry> {
        if let Some((group, name)) = qualified_name.split_once(':') {
            self.get_tag_by_group(group, name)
        } else {
            self.get_tag_by_name(qualified_name)
        }
    }

    /// Get tag by name (without group qualifier)
    /// Returns the first matching tag found
    pub fn get_tag_by_name(&self, tag_name: &str) -> Option<&TagEntry> {
        self.tags.iter().find(|tag| tag.name == tag_name)
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
    /// Processor name that handled this tag
    /// ExifTool: PROCESS_PROC information
    pub processor_name: String,
}

impl TagSourceInfo {
    /// Create new tag source info
    pub fn new(namespace: String, ifd_name: String, processor_name: String) -> Self {
        let priority = SourcePriority::from_namespace(&namespace);
        Self {
            namespace,
            ifd_name,
            priority,
            processor_name,
        }
    }

    /// Get the full tag name with namespace prefix
    /// ExifTool format: "Group:TagName"
    pub fn format_tag_name(&self, tag_name: &str) -> String {
        format!("{}:{}", self.namespace, tag_name)
    }

    /// Get ExifTool Group1 value based on IFD name
    /// ExifTool: Groups => { 1 => 'ExifIFD' } specification
    pub fn get_group1(&self) -> String {
        match self.ifd_name.as_str() {
            "ExifIFD" => "ExifIFD".to_string(),
            "GPS" => "GPS".to_string(),
            "InteropIFD" => "InteropIFD".to_string(),
            "MakerNotes" => "MakerNotes".to_string(),
            "IFD1" => "IFD1".to_string(),
            // Default to IFD0 for main IFD and unknown IFDs
            _ => "IFD0".to_string(),
        }
    }
}

/// Temporary placeholder for ProcessorDispatch during Phase 5 cleanup
/// TODO: Remove this once trait-based dispatch is fully integrated
#[derive(Debug, Clone, Default)]
pub struct ProcessorDispatch {
    pub subdirectory_overrides: HashMap<u16, String>,
    pub parameters: HashMap<String, String>,
}

impl ProcessorDispatch {
    pub fn with_table_processor(_processor: String) -> Self {
        Self::default()
    }
}
