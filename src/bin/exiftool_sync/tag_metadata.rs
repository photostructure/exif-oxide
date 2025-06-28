//! Tag Metadata Module
//!
//! This module provides access to tag priority information based on frequency and mainstream
//! usage data extracted from ExifTool usage statistics. The data helps prioritize which tags
//! should be implemented first in exif-oxide synchronization efforts.
//!
//! Priority levels are assigned based on ExifTool usage data:
//! - High: Mainstream tags or frequency > 0.8 (most commonly used tags)  
//! - Medium: Frequency > 0.25 (moderately used tags)
//! - Low: All other tags (rarely used or specialized tags)

use crate::extractors::Priority;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Tag metadata entry containing usage statistics from ExifTool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagMetadataEntry {
    /// Usage frequency (0.0 to 1.0) - how often this tag appears in real-world images
    pub frequency: f64,
    /// Whether this tag is considered mainstream/essential for photography
    pub mainstream: bool,
    /// Groups/contexts where this tag appears (EXIF, MakerNotes, XMP, etc.)
    pub groups: Vec<String>,
}

/// Tag metadata manager that loads and provides access to ExifTool usage statistics
#[derive(Debug)]
pub struct TagMetadata {
    /// Map from tag name to metadata entry
    metadata: HashMap<String, TagMetadataEntry>,
}

impl TagMetadata {
    /// Create a new TagMetadata instance by loading data from TagMetadata.json
    ///
    /// # Returns
    ///
    /// Returns `Ok(TagMetadata)` on success, or an error if the file cannot be loaded
    /// or parsed. If the file doesn't exist, returns an empty metadata set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let tag_metadata = TagMetadata::new()?;
    /// let priority = tag_metadata.get_priority("Make");
    /// ```
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let metadata = Self::load_tag_metadata()?;
        Ok(Self { metadata })
    }

    /// Create a new empty TagMetadata instance for fallback scenarios
    ///
    /// This is useful when TagMetadata.json cannot be loaded but operation should continue
    /// with default priority assignments.
    pub fn empty() -> Self {
        Self {
            metadata: HashMap::new(),
        }
    }

    /// Load TagMetadata.json from the exiftool-vendored.js data directory
    ///
    /// # Returns
    ///
    /// Returns a HashMap mapping tag names to their metadata entries.
    /// If the file doesn't exist, returns an empty HashMap without error.
    fn load_tag_metadata() -> Result<HashMap<String, TagMetadataEntry>, Box<dyn std::error::Error>>
    {
        let metadata_path = Path::new("third-party/exiftool-vendored.js/data/TagMetadata.json");

        if !metadata_path.exists() {
            // Return empty metadata if file doesn't exist - this is not an error condition
            return Ok(HashMap::new());
        }

        let content = std::fs::read_to_string(metadata_path)
            .map_err(|e| format!("Failed to read TagMetadata.json: {}", e))?;

        let metadata: HashMap<String, TagMetadataEntry> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse TagMetadata.json: {}", e))?;

        Ok(metadata)
    }

    /// Get the priority level for a given tag name based on usage statistics
    ///
    /// Priority is determined using the same logic as analyze_printconv_safety.rs:
    /// - High: mainstream tags OR frequency > 0.8 (most important/common tags)
    /// - Medium: frequency > 0.25 (moderately important tags)  
    /// - Low: all other tags (less common or specialized tags)
    ///
    /// # Arguments
    ///
    /// * `tag_name` - The ExifTool tag name (e.g., "Make", "Model", "ExposureTime")
    ///
    /// # Returns
    ///
    /// Returns the Priority level for the tag. If the tag is not found in the metadata,
    /// returns Priority::Low as a safe default.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let tag_metadata = TagMetadata::new()?;
    ///
    /// // Common photography tags should be High priority
    /// assert_eq!(tag_metadata.get_priority("Make"), Priority::High);
    /// assert_eq!(tag_metadata.get_priority("ExposureTime"), Priority::High);
    ///
    /// // Less common tags should be Medium or Low priority
    /// assert_eq!(tag_metadata.get_priority("SomeRareTag"), Priority::Low);
    /// ```
    pub fn get_priority(&self, tag_name: &str) -> Priority {
        if let Some(metadata) = self.metadata.get(tag_name) {
            if metadata.mainstream || metadata.frequency > 0.8 {
                Priority::High
            } else if metadata.frequency > 0.25 {
                Priority::Medium
            } else {
                Priority::Low
            }
        } else {
            // Default to Low priority for unknown tags
            Priority::Low
        }
    }

    /// Get the metadata entry for a specific tag
    ///
    /// # Arguments
    ///
    /// * `tag_name` - The ExifTool tag name
    ///
    /// # Returns
    ///
    /// Returns `Some(TagMetadataEntry)` if the tag exists in the metadata,
    /// or `None` if not found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let tag_metadata = TagMetadata::new()?;
    ///
    /// if let Some(entry) = tag_metadata.get_metadata("Make") {
    ///     println!("Make frequency: {}", entry.frequency);
    ///     println!("Make is mainstream: {}", entry.mainstream);
    ///     println!("Make groups: {:?}", entry.groups);
    /// }
    /// ```
    #[allow(dead_code)]
    pub fn get_metadata(&self, tag_name: &str) -> Option<&TagMetadataEntry> {
        self.metadata.get(tag_name)
    }

    /// Get the total number of tags in the metadata
    ///
    /// # Returns
    ///
    /// Returns the number of tags loaded from TagMetadata.json
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.metadata.len()
    }

    /// Check if the metadata is empty
    ///
    /// # Returns
    ///
    /// Returns `true` if no metadata was loaded (e.g., file doesn't exist)
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.metadata.is_empty()
    }

    /// Get an iterator over all tag names in the metadata
    ///
    /// # Returns
    ///
    /// Returns an iterator over tag name strings
    #[allow(dead_code)]
    pub fn tag_names(&self) -> impl Iterator<Item = &String> {
        self.metadata.keys()
    }

    /// Get tags filtered by priority level
    ///
    /// # Arguments
    ///
    /// * `priority` - The priority level to filter by
    ///
    /// # Returns
    ///
    /// Returns a vector of tag names that match the specified priority level
    ///
    /// # Examples
    ///
    /// ```rust
    /// let tag_metadata = TagMetadata::new()?;
    ///
    /// // Get all high-priority tags for implementation
    /// let high_priority_tags = tag_metadata.get_tags_by_priority(Priority::High);
    /// println!("Found {} high-priority tags", high_priority_tags.len());
    /// ```
    #[allow(dead_code)]
    pub fn get_tags_by_priority(&self, priority: Priority) -> Vec<&String> {
        self.metadata
            .keys()
            .filter(|tag_name| {
                matches!(self.get_priority(tag_name), p if std::mem::discriminant(&p) == std::mem::discriminant(&priority))
            })
            .collect()
    }
}

/// Convenience function to get priority for a tag name
///
/// This function creates a new TagMetadata instance and queries it for the priority.
/// For better performance when making multiple queries, create a TagMetadata instance
/// and reuse it.
///
/// # Arguments
///
/// * `tag_name` - The ExifTool tag name
///
/// # Returns
///
/// Returns the Priority level for the tag, or Priority::Low if metadata cannot be loaded
/// or the tag is not found.
///
/// # Examples
///
/// ```rust
/// let priority = get_priority("Make");
/// assert_eq!(priority, Priority::High);
/// ```
#[allow(dead_code)]
pub fn get_priority(tag_name: &str) -> Priority {
    TagMetadata::new()
        .map(|metadata| metadata.get_priority(tag_name))
        .unwrap_or(Priority::Low) // Safe fallback if metadata can't be loaded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_metadata_loading() {
        // This test will pass even if TagMetadata.json doesn't exist
        let result = TagMetadata::new();
        assert!(
            result.is_ok(),
            "TagMetadata::new() should not fail even if file missing"
        );
    }

    #[test]
    fn test_priority_logic() {
        // Test the priority assignment logic with mock data
        let mut metadata = HashMap::new();

        // High priority: mainstream tag
        metadata.insert(
            "Make".to_string(),
            TagMetadataEntry {
                frequency: 0.5,
                mainstream: true,
                groups: vec!["EXIF".to_string()],
            },
        );

        // High priority: high frequency
        metadata.insert(
            "Model".to_string(),
            TagMetadataEntry {
                frequency: 0.9,
                mainstream: false,
                groups: vec!["EXIF".to_string()],
            },
        );

        // Medium priority
        metadata.insert(
            "ExposureTime".to_string(),
            TagMetadataEntry {
                frequency: 0.3,
                mainstream: false,
                groups: vec!["EXIF".to_string()],
            },
        );

        // Low priority
        metadata.insert(
            "RareTag".to_string(),
            TagMetadataEntry {
                frequency: 0.1,
                mainstream: false,
                groups: vec!["MakerNotes".to_string()],
            },
        );

        let tag_metadata = TagMetadata { metadata };

        assert_eq!(tag_metadata.get_priority("Make"), Priority::High);
        assert_eq!(tag_metadata.get_priority("Model"), Priority::High);
        assert_eq!(tag_metadata.get_priority("ExposureTime"), Priority::Medium);
        assert_eq!(tag_metadata.get_priority("RareTag"), Priority::Low);
        assert_eq!(tag_metadata.get_priority("UnknownTag"), Priority::Low);
    }

    #[test]
    fn test_convenience_function() {
        // Test the convenience function doesn't panic
        let priority = get_priority("Make");
        // Should return some priority level without panicking
        assert!(matches!(
            priority,
            Priority::Low | Priority::Medium | Priority::High
        ));
    }

    #[test]
    fn test_filtering_by_priority() {
        let mut metadata = HashMap::new();

        metadata.insert(
            "HighTag1".to_string(),
            TagMetadataEntry {
                frequency: 0.9,
                mainstream: false,
                groups: vec!["EXIF".to_string()],
            },
        );

        metadata.insert(
            "HighTag2".to_string(),
            TagMetadataEntry {
                frequency: 0.1,
                mainstream: true,
                groups: vec!["EXIF".to_string()],
            },
        );

        metadata.insert(
            "MediumTag".to_string(),
            TagMetadataEntry {
                frequency: 0.3,
                mainstream: false,
                groups: vec!["EXIF".to_string()],
            },
        );

        let tag_metadata = TagMetadata { metadata };

        let high_tags = tag_metadata.get_tags_by_priority(Priority::High);
        let medium_tags = tag_metadata.get_tags_by_priority(Priority::Medium);
        let low_tags = tag_metadata.get_tags_by_priority(Priority::Low);

        assert_eq!(high_tags.len(), 2);
        assert_eq!(medium_tags.len(), 1);
        assert_eq!(low_tags.len(), 0);

        assert!(high_tags.contains(&&"HighTag1".to_string()));
        assert!(high_tags.contains(&&"HighTag2".to_string()));
        assert!(medium_tags.contains(&&"MediumTag".to_string()));
    }
}
