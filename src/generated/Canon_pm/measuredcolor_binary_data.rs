//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

//! Canon ProcessBinaryData table MeasuredColor generated from Canon.pm
//! ExifTool: Canon.pm %Canon::MeasuredColor

use std::collections::HashMap;
use std::sync::LazyLock;

/// Canon ProcessBinaryData table for MeasuredColor
/// Total tags: 1
#[derive(Debug, Clone)]
pub struct CanonMeasuredColorTable {
    pub default_format: &'static str,         // "int16u"
    pub first_entry: i32,                     // 1
    pub groups: (&'static str, &'static str), // ("MakerNotes", "Camera")
}

/// Tag definitions for Canon MeasuredColor
pub static MEASUREDCOLOR_TAGS: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(1, "MeasuredRGGB"); // 0x01: MeasuredRGGB
    map
});

/// Format specifications for Canon MeasuredColor
pub static MEASUREDCOLOR_FORMATS: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(1, "int16u[4]"); // 0x01: MeasuredRGGB
    map
});

impl CanonMeasuredColorTable {
    /// Create new table instance
    pub fn new() -> Self {
        Self {
            default_format: "int16u",
            first_entry: 1,
            groups: ("MakerNotes", "Camera"),
        }
    }

    /// Get tag name for offset
    pub fn get_tag_name(&self, offset: u16) -> Option<&'static str> {
        MEASUREDCOLOR_TAGS.get(&offset).copied()
    }

    /// Get format specification for offset
    pub fn get_format(&self, offset: u16) -> Option<&'static str> {
        MEASUREDCOLOR_FORMATS.get(&offset).copied()
    }

    /// Get all valid offsets for this table
    pub fn get_offsets(&self) -> Vec<u16> {
        MEASUREDCOLOR_TAGS.keys().copied().collect()
    }
}

impl Default for CanonMeasuredColorTable {
    fn default() -> Self {
        Self::new()
    }
}
