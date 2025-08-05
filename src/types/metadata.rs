//! Metadata structures for EXIF data representation
//!
//! This module defines the core metadata structures including TagEntry,
//! ExifData, and TagSourceInfo that represent extracted EXIF information.

use crate::types::TagValue;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Configuration for filtering which tags to extract and how to format them
///
/// This struct controls both tag selection (filtering) and value formatting (PrintConv vs ValueConv).
/// It enables performance optimization by extracting only requested tags and early termination
/// when simple tags (like File group tags) are requested.
///
/// # Examples
///
/// ```
/// use exif_oxide::types::FilterOptions;
/// use std::collections::HashSet;
///
/// // Extract only MIMEType tag (performance optimized - no EXIF parsing needed)
/// let mime_only = FilterOptions {
///     requested_tags: vec!["MIMEType".to_string()],
///     requested_groups: vec![],
///     group_all_patterns: vec![],
///     extract_all: false,
///     numeric_tags: HashSet::new(),
///     glob_patterns: vec![],
/// };
///
/// // Extract all EXIF group tags with some numeric values
/// let mut numeric_tags = HashSet::new();
/// numeric_tags.insert("Orientation".to_string());
/// let exif_with_numeric = FilterOptions {
///     requested_tags: vec![],
///     requested_groups: vec![],
///     group_all_patterns: vec!["EXIF:all".to_string()],
///     extract_all: false,
///     numeric_tags,
///     glob_patterns: vec![],
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct FilterOptions {
    /// Specific tags to extract (case-insensitive)
    /// Examples: ["MIMEType", "Orientation", "FNumber"]
    pub requested_tags: Vec<String>,

    /// Group filters without :all suffix (case-insensitive)  
    /// Examples: ["EXIF", "File", "GPS"]
    pub requested_groups: Vec<String>,

    /// Group:all patterns (case-insensitive)
    /// Examples: ["File:all", "EXIF:all", "GPS:all"]
    pub group_all_patterns: Vec<String>,

    /// Extract all available tags (default behavior for backward compatibility)
    /// When true, all other filters are ignored
    pub extract_all: bool,

    /// Tags that should use numeric values instead of PrintConv
    /// These correspond to ExifTool's -TagName# syntax
    pub numeric_tags: HashSet<String>,

    /// Glob patterns for tag/group matching (case-insensitive)
    /// Examples: ["GPS*", "*tude", "*Date*", "Canon*"]
    /// Supports prefix (*), suffix (*), and middle (*) wildcards
    pub glob_patterns: Vec<String>,
}

impl Default for FilterOptions {
    fn default() -> Self {
        Self {
            requested_tags: Vec::new(),
            requested_groups: Vec::new(),
            group_all_patterns: Vec::new(),
            extract_all: true, // Default to extracting all tags for backward compatibility
            numeric_tags: HashSet::new(),
            glob_patterns: Vec::new(),
        }
    }
}

impl FilterOptions {
    /// Create FilterOptions that extracts all tags (backward compatibility)
    pub fn extract_all() -> Self {
        Self::default()
    }

    /// Create FilterOptions for specific tags only
    pub fn tags_only(tags: Vec<String>) -> Self {
        Self {
            requested_tags: tags,
            requested_groups: Vec::new(),
            group_all_patterns: Vec::new(),
            extract_all: false,
            numeric_tags: HashSet::new(),
            glob_patterns: Vec::new(),
        }
    }

    /// Create FilterOptions for specific groups
    pub fn groups_only(groups: Vec<String>) -> Self {
        Self {
            requested_tags: Vec::new(),
            requested_groups: groups,
            group_all_patterns: Vec::new(),
            extract_all: false,
            numeric_tags: HashSet::new(),
            glob_patterns: Vec::new(),
        }
    }

    /// Check if we should extract all tags (ignoring filters)
    pub fn should_extract_all(&self) -> bool {
        self.extract_all
    }

    /// Check if any specific tags or groups are requested
    pub fn has_specific_requests(&self) -> bool {
        !self.requested_tags.is_empty()
            || !self.requested_groups.is_empty()
            || !self.group_all_patterns.is_empty()
            || !self.glob_patterns.is_empty()
    }

    /// Check if a tag should be extracted based on current filters
    /// Uses case-insensitive matching to match ExifTool behavior
    pub fn should_extract_tag(&self, tag_name: &str, tag_group: &str) -> bool {
        if self.extract_all {
            return true;
        }

        let tag_name_lower = tag_name.to_lowercase();
        let tag_group_lower = tag_group.to_lowercase();

        // Check specific tag requests (case-insensitive)
        if self
            .requested_tags
            .iter()
            .any(|t| t.to_lowercase() == tag_name_lower)
        {
            return true;
        }

        // Check group filters (case-insensitive)
        if self
            .requested_groups
            .iter()
            .any(|g| g.to_lowercase() == tag_group_lower)
        {
            return true;
        }

        // Check group:all patterns (case-insensitive)
        for pattern in &self.group_all_patterns {
            let pattern_lower = pattern.to_lowercase();
            if let Some((group_part, all_part)) = pattern_lower.split_once(':') {
                if all_part == "all" && group_part == tag_group_lower {
                    return true;
                }
            }
        }

        // Check glob patterns (case-insensitive)
        // Test against both tag name and group:tag format like ExifTool
        for pattern in &self.glob_patterns {
            if Self::matches_glob_pattern(tag_name, pattern) {
                return true;
            }
            // Also check group:tag format for patterns like "EXIF:*"
            let qualified_name = format!("{}:{}", tag_group, tag_name);
            if Self::matches_glob_pattern(&qualified_name, pattern) {
                return true;
            }
        }

        false
    }

    /// Check if a tag should use numeric output (ValueConv instead of PrintConv)
    pub fn should_use_numeric(&self, tag_name: &str) -> bool {
        self.numeric_tags.contains(tag_name)
    }

    /// Check if a string matches a glob pattern (case-insensitive)
    /// Supports ExifTool-style wildcards: prefix (*), suffix (*), and middle (*)
    /// Examples: "GPS*" matches "GPSLatitude", "*tude" matches "Latitude", "*Date*" matches "CreateDate"
    fn matches_glob_pattern(text: &str, pattern: &str) -> bool {
        let text_lower = text.to_lowercase();
        let pattern_lower = pattern.to_lowercase();

        if pattern_lower == "*" {
            return true; // * matches everything
        }

        if !pattern_lower.contains('*') {
            return text_lower == pattern_lower; // Exact match if no wildcards
        }

        // Handle different wildcard patterns
        if pattern_lower.starts_with('*') && pattern_lower.ends_with('*') {
            // Middle wildcard: "*Date*" matches anything containing "date"
            let middle = &pattern_lower[1..pattern_lower.len() - 1];
            text_lower.contains(middle)
        } else if let Some(suffix) = pattern_lower.strip_prefix('*') {
            // Suffix wildcard: "*tude" matches anything ending with "tude"
            text_lower.ends_with(suffix)
        } else if pattern_lower.ends_with('*') {
            // Prefix wildcard: "GPS*" matches anything starting with "gps"
            let prefix = &pattern_lower[..pattern_lower.len() - 1];
            text_lower.starts_with(prefix)
        } else {
            // Multiple * in pattern - for now, treat as exact match
            // TODO: Could implement full glob matching if needed
            text_lower == pattern_lower
        }
    }

    /// Determine if only File group tags are requested (performance optimization)
    /// This enables early return without expensive EXIF/MakerNotes parsing
    pub fn is_file_group_only(&self) -> bool {
        if self.extract_all {
            return false;
        }

        // Check if all requested items are File group related
        let all_file_related = self.requested_tags.iter().all(|tag| {
            // Common File group tags that don't require format-specific parsing
            matches!(
                tag.to_lowercase().as_str(),
                "filename"
                    | "directory"
                    | "filesize"
                    | "filemodifydate"
                    | "fileaccessdate"
                    | "fileinodechangedate"
                    | "filecreatedate"
                    | "filepermissions"
                    | "filetype"
                    | "filetypeextension"
                    | "mimetype"
            )
        }) && self
            .requested_groups
            .iter()
            .all(|group| group.to_lowercase() == "file")
            && self
                .group_all_patterns
                .iter()
                .all(|pattern| pattern.to_lowercase() == "file:all")
            && self.glob_patterns.iter().all(|pattern| {
                // Check if glob pattern could match non-File group tags
                // Only File group patterns or pure File tag patterns should be considered file-only
                let pattern_lower = pattern.to_lowercase();
                pattern_lower == "file:*"
                    || pattern_lower.starts_with("file")
                    || matches!(
                        pattern_lower.as_str(),
                        "filename*"
                            | "directory*"
                            | "filesize*"
                            | "filemodifydate*"
                            | "fileaccessdate*"
                            | "fileinodechangedate*"
                            | "filecreatedate*"
                            | "filepermissions*"
                            | "filetype*"
                            | "filetypeextension*"
                            | "mimetype*"
                    )
            });

        all_file_related && self.has_specific_requests()
    }
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
///     group1: "ExifIFD".to_string(),  // Located in ExifIFD subdirectory
///     name: "FNumber".to_string(),
///     value: TagValue::F64(4.0),      // Post-ValueConv: 4/1 → 4.0
///     print: TagValue::String("4.0".to_string()),       // Post-PrintConv: formatted for display
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
///     print: TagValue::String("24 mm".to_string()),     // Human-readable with units
/// };
///
/// assert_eq!(focal_entry.print, TagValue::String("24 mm".to_string()));
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
    #[serde(rename = "ExifToolVersion", skip_serializing_if = "String::is_empty")]
    pub exif_tool_version: String,

    /// All extracted tags with both value and print representations
    #[serde(skip)]
    pub tags: Vec<TagEntry>,

    /// Legacy field for backward compatibility - will be populated during serialization
    /// TODO: Remove this once all consumers are updated to use TagEntry
    #[serde(flatten)]
    pub legacy_tags: IndexMap<String, TagValue>,

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
            legacy_tags: IndexMap::new(),
            errors: Vec::new(),
            missing_implementations: None,
        }
    }

    /// Get group priority for ExifTool-compatible ordering
    /// Returns lower numbers for groups that should appear first
    fn get_group_priority(tag_key: &str) -> u8 {
        if tag_key == "SourceFile" {
            return 0;
        }
        if tag_key == "ExifToolVersion" {
            return 1;
        }

        // Extract group prefix from "Group:TagName" format
        if let Some(group) = tag_key.split(':').next() {
            match group {
                "File" => 2,
                "JFIF" | "APP" | "APP0" | "APP1" | "APP2" | "APP3" | "APP4" | "APP5" | "APP6"
                | "APP7" | "APP8" | "APP9" | "APP10" | "APP11" | "APP12" | "APP13" | "APP14"
                | "APP15" => 3,
                "EXIF" => 4,
                "MakerNotes" => 5,
                "Composite" => 255, // Always last
                // Other groups (XMP, IPTC, Photoshop, PrintIM, MPF, ICC_Profile, etc.)
                _ => 50,
            }
        } else {
            // Tags without group prefix get high priority (like SourceFile, ExifToolVersion)
            10
        }
    }

    /// Convert tags to legacy format for JSON serialization
    /// This populates legacy_tags from the TagEntry vector
    pub fn prepare_for_serialization(
        &mut self,
        numeric_tags: Option<&std::collections::HashSet<String>>,
    ) {
        use tracing::debug;

        // Preserve existing legacy_tags (like System: and Warning: tags) before clearing
        let existing_legacy_tags = self.legacy_tags.clone();
        self.legacy_tags.clear();

        // Re-add preserved legacy tags that don't come from TagEntry
        for (key, value) in existing_legacy_tags {
            if key.starts_with("System:") || key.starts_with("Warning:") {
                self.legacy_tags.insert(key, value);
            }
        }

        // Create a sorted list of (tag_key, tag_entry) pairs for ordered insertion
        let mut tag_pairs: Vec<(String, &TagEntry)> = self
            .tags
            .iter()
            .map(|entry| (format!("{}:{}", entry.group, entry.name), entry))
            .collect();

        // Sort by group priority first, then alphabetically within group
        tag_pairs.sort_by(|(key_a, _), (key_b, _)| {
            let priority_a = Self::get_group_priority(key_a);
            let priority_b = Self::get_group_priority(key_b);

            match priority_a.cmp(&priority_b) {
                std::cmp::Ordering::Equal => key_a.cmp(key_b), // Alphabetical within group
                other => other,
            }
        });

        // Insert tags in the sorted order
        for (key, entry) in tag_pairs {
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
    /// let exif_data = extract_metadata(std::path::Path::new("image.jpg"), false, false).unwrap();
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
    /// let exif_data = extract_metadata(std::path::Path::new("image.jpg"), false, false).unwrap();
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
    /// let exif_data = extract_metadata(std::path::Path::new("image.jpg"), false, false).unwrap();
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
    /// Returns the highest priority matching tag found
    /// ExifTool behavior: EXIF tags take precedence over MakerNotes tags
    pub fn get_tag_by_name(&self, tag_name: &str) -> Option<&TagEntry> {
        let matching_tags: Vec<&TagEntry> = self
            .tags
            .iter()
            .filter(|tag| tag.name == tag_name)
            .collect();

        if matching_tags.is_empty() {
            return None;
        }

        // If only one match, return it
        if matching_tags.len() == 1 {
            return Some(matching_tags[0]);
        }

        // Multiple matches - use priority-based selection
        // ExifTool behavior: EXIF group takes precedence over MakerNotes group
        matching_tags
            .into_iter()
            .max_by_key(|tag| SourcePriority::from_namespace(&tag.group))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_glob_pattern() {
        // Test prefix wildcard
        assert!(FilterOptions::matches_glob_pattern("GPSAltitude", "GPS*"));
        assert!(FilterOptions::matches_glob_pattern("GPSLatitude", "GPS*"));
        assert!(!FilterOptions::matches_glob_pattern("Altitude", "GPS*"));

        // Test suffix wildcard
        assert!(FilterOptions::matches_glob_pattern("GPSLatitude", "*tude"));
        assert!(FilterOptions::matches_glob_pattern("Altitude", "*tude"));
        assert!(!FilterOptions::matches_glob_pattern("GPSAltitu", "*tude"));

        // Test middle wildcard
        assert!(FilterOptions::matches_glob_pattern("CreateDate", "*Date*"));
        assert!(FilterOptions::matches_glob_pattern(
            "DateTimeOriginal",
            "*Date*"
        ));
        assert!(!FilterOptions::matches_glob_pattern("CreateTime", "*Date*"));

        // Test case insensitive
        assert!(FilterOptions::matches_glob_pattern("gpsaltitude", "GPS*"));
        assert!(FilterOptions::matches_glob_pattern("GPSAltitude", "gps*"));
    }

    #[test]
    fn test_should_extract_tag_with_glob_patterns() {
        let filter_opts = FilterOptions {
            requested_tags: Vec::new(),
            requested_groups: Vec::new(),
            group_all_patterns: Vec::new(),
            extract_all: false,
            numeric_tags: HashSet::new(),
            glob_patterns: vec!["GPS*".to_string()],
        };

        // Should match GPS tags
        assert!(filter_opts.should_extract_tag("GPSAltitude", "GPS"));
        assert!(filter_opts.should_extract_tag("GPSLatitude", "GPS"));
        assert!(filter_opts.should_extract_tag("GPSVersionID", "EXIF"));

        // Should not match non-GPS tags
        assert!(!filter_opts.should_extract_tag("Make", "EXIF"));
        // Note: "Altitude" in GPS group creates "GPS:Altitude" which DOES match "GPS*" pattern
        // This is correct ExifTool behavior - group-qualified names are checked
        assert!(filter_opts.should_extract_tag("Altitude", "GPS"));
        assert!(!filter_opts.should_extract_tag("Altitude", "EXIF")); // Different group
    }

    #[test]
    fn test_is_file_group_only_with_glob_patterns() {
        // GPS glob pattern should NOT be file-only
        let gps_filter = FilterOptions {
            requested_tags: Vec::new(),
            requested_groups: Vec::new(),
            group_all_patterns: Vec::new(),
            extract_all: false,
            numeric_tags: HashSet::new(),
            glob_patterns: vec!["GPS*".to_string()],
        };
        assert!(!gps_filter.is_file_group_only());

        // File glob pattern SHOULD be file-only
        let file_filter = FilterOptions {
            requested_tags: Vec::new(),
            requested_groups: Vec::new(),
            group_all_patterns: Vec::new(),
            extract_all: false,
            numeric_tags: HashSet::new(),
            glob_patterns: vec!["File*".to_string()],
        };
        assert!(file_filter.is_file_group_only());

        // MIMEType glob pattern SHOULD be file-only (File group tag)
        let mime_filter = FilterOptions {
            requested_tags: Vec::new(),
            requested_groups: Vec::new(),
            group_all_patterns: Vec::new(),
            extract_all: false,
            numeric_tags: HashSet::new(),
            glob_patterns: vec!["MIMEType*".to_string()],
        };
        assert!(mime_filter.is_file_group_only());
    }
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
            "KyoceraRaw" => "KyoceraRaw".to_string(),
            // Canon MakerNote subdirectory processing
            // ExifTool: MakerNotes.pm MakerNoteCanon -> Canon.pm Main table
            // The directory name becomes Group1 per ExifTool's SetGroup logic
            name if name.starts_with("Canon") => "Canon".to_string(),
            // Other manufacturer MakerNote subdirectories follow the same pattern
            name if name.starts_with("Nikon") => "Nikon".to_string(),
            name if name.starts_with("Sony") => "Sony".to_string(),
            name if name.starts_with("Olympus") => "Olympus".to_string(),
            name if name.starts_with("Panasonic") => "Panasonic".to_string(),
            name if name.starts_with("Pentax") => "Pentax".to_string(),
            name if name.starts_with("Fujifilm") => "Fujifilm".to_string(),
            // Default to IFD0 for main IFD and unknown IFDs
            _ => "IFD0".to_string(),
        }
    }

    /// Get ExifTool Group1 value with tag-specific overrides for correct context assignment
    /// ExifTool: Certain tags belong to specific contexts regardless of processing order
    /// Fixes issue where Canon MakerNotes processing steals ExifIFD tags like ColorSpace
    pub fn get_group1_with_tag_override(&self, tag_id: u16) -> String {
        // ExifIFD-specific tags should always have group1="ExifIFD" regardless of processing context
        // ExifTool: These tags are defined in Exif.pm ExifIFD table, not manufacturer tables
        match tag_id {
            // Core ExifIFD tags that should never be assigned to manufacturer context
            0x9000 => "ExifIFD".to_string(), // ExifVersion - Always in ExifIFD
            0xA000 => "ExifIFD".to_string(), // FlashpixVersion - Always in ExifIFD
            0xA001 => "ExifIFD".to_string(), // ColorSpace - Always in ExifIFD
            0xA002 => "ExifIFD".to_string(), // ExifImageWidth - Always in ExifIFD
            0xA003 => "ExifIFD".to_string(), // ExifImageHeight - Always in ExifIFD
            0xA005 => "ExifIFD".to_string(), // InteropIFD pointer - Always in ExifIFD
            // GPS IFD pointer should always have GPS group1
            0x8825 => "GPS".to_string(), // GPSInfo - Always GPS context
            // For all other tags, use normal context-based assignment
            _ => self.get_group1(),
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
