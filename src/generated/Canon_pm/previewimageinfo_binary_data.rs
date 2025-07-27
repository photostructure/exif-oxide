//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

//! Canon ProcessBinaryData table PreviewImageInfo generated from Canon.pm
//! ExifTool: Canon.pm %Canon::PreviewImageInfo

use std::collections::HashMap;
use std::sync::LazyLock;

/// Canon ProcessBinaryData table for PreviewImageInfo
/// Total tags: 5
#[derive(Debug, Clone)]
pub struct CanonPreviewImageInfoTable {
    pub default_format: &'static str,         // "int32u"
    pub first_entry: i32,                     // 1
    pub groups: (&'static str, &'static str), // ("MakerNotes", "Image")
}

/// Tag definitions for Canon PreviewImageInfo
pub static PREVIEWIMAGEINFO_TAGS: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(1, "PreviewQuality"); // 0x01: PreviewQuality
    map.insert(2, "PreviewImageLength"); // 0x02: PreviewImageLength
    map.insert(3, "PreviewImageWidth"); // 0x03: PreviewImageWidth
    map.insert(4, "PreviewImageHeight"); // 0x04: PreviewImageHeight
    map.insert(5, "PreviewImageStart"); // 0x05: PreviewImageStart
    map
});

impl CanonPreviewImageInfoTable {
    /// Create new table instance
    pub fn new() -> Self {
        Self {
            default_format: "int32u",
            first_entry: 1,
            groups: ("MakerNotes", "Image"),
        }
    }

    /// Get tag name for offset
    pub fn get_tag_name(&self, offset: u16) -> Option<&'static str> {
        PREVIEWIMAGEINFO_TAGS.get(&offset).copied()
    }

    /// Get all valid offsets for this table
    pub fn get_offsets(&self) -> Vec<u16> {
        PREVIEWIMAGEINFO_TAGS.keys().copied().collect()
    }
}

impl Default for CanonPreviewImageInfoTable {
    fn default() -> Self {
        Self::new()
    }
}
