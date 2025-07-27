//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

//! Sony ProcessBinaryData table ShotInfo generated from Sony.pm
//! ExifTool: Sony.pm %Sony::ShotInfo

use std::collections::HashMap;
use std::sync::LazyLock;

/// Sony ProcessBinaryData table for ShotInfo
/// Total tags: 9
#[derive(Debug, Clone)]
pub struct SonyShotInfoTable {
    pub first_entry: i32,                     // 0
    pub groups: (&'static str, &'static str), // ("MakerNotes", "Image")
}

/// Tag definitions for Sony ShotInfo
pub static SHOTINFO_TAGS: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(2, "FaceInfoOffset"); // 0x02: FaceInfoOffset
    map.insert(6, "SonyDateTime"); // 0x06: SonyDateTime
    map.insert(26, "SonyImageHeight"); // 0x1a: SonyImageHeight
    map.insert(28, "SonyImageWidth"); // 0x1c: SonyImageWidth
    map.insert(48, "FacesDetected"); // 0x30: FacesDetected
    map.insert(50, "FaceInfoLength"); // 0x32: FaceInfoLength
    map.insert(52, "MetaVersion"); // 0x34: MetaVersion
    map.insert(72, "FaceInfo1"); // 0x48: FaceInfo1
    map.insert(94, "FaceInfo2"); // 0x5e: FaceInfo2
    map
});

/// Format specifications for Sony ShotInfo
pub static SHOTINFO_FORMATS: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(2, "int16u"); // 0x02: FaceInfoOffset
    map.insert(6, "string[20]"); // 0x06: SonyDateTime
    map.insert(26, "int16u"); // 0x1a: SonyImageHeight
    map.insert(28, "int16u"); // 0x1c: SonyImageWidth
    map.insert(48, "int16u"); // 0x30: FacesDetected
    map.insert(50, "int16u"); // 0x32: FaceInfoLength
    map.insert(52, "string[16]"); // 0x34: MetaVersion
    map
});

impl SonyShotInfoTable {
    /// Create new table instance
    pub fn new() -> Self {
        Self {
            first_entry: 0,
            groups: ("MakerNotes", "Image"),
        }
    }

    /// Get tag name for offset
    pub fn get_tag_name(&self, offset: u16) -> Option<&'static str> {
        SHOTINFO_TAGS.get(&offset).copied()
    }

    /// Get format specification for offset
    pub fn get_format(&self, offset: u16) -> Option<&'static str> {
        SHOTINFO_FORMATS.get(&offset).copied()
    }

    /// Get all valid offsets for this table
    pub fn get_offsets(&self) -> Vec<u16> {
        SHOTINFO_TAGS.keys().copied().collect()
    }
}

impl Default for SonyShotInfoTable {
    fn default() -> Self {
        Self::new()
    }
}
