//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

//! Canon ProcessBinaryData table Processing generated from Canon.pm
//! ExifTool: Canon.pm %Canon::Processing

use std::collections::HashMap;
use std::sync::LazyLock;

/// Canon ProcessBinaryData table for Processing
/// Total tags: 15
#[derive(Debug, Clone)]
pub struct CanonProcessingTable {
    pub default_format: &'static str,         // "int16s"
    pub first_entry: i32,                     // 1
    pub groups: (&'static str, &'static str), // ("MakerNotes", "Image")
}

/// Tag definitions for Canon Processing
pub static PROCESSING_TAGS: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(1, "ToneCurve"); // 0x01: ToneCurve
    map.insert(2, "Sharpness"); // 0x02: Sharpness
    map.insert(3, "SharpnessFrequency"); // 0x03: SharpnessFrequency
    map.insert(4, "SensorRedLevel"); // 0x04: SensorRedLevel
    map.insert(5, "SensorBlueLevel"); // 0x05: SensorBlueLevel
    map.insert(6, "WhiteBalanceRed"); // 0x06: WhiteBalanceRed
    map.insert(7, "WhiteBalanceBlue"); // 0x07: WhiteBalanceBlue
    map.insert(8, "WhiteBalance"); // 0x08: WhiteBalance
    map.insert(9, "ColorTemperature"); // 0x09: ColorTemperature
    map.insert(10, "PictureStyle"); // 0x0a: PictureStyle
    map.insert(11, "DigitalGain"); // 0x0b: DigitalGain
    map.insert(12, "WBShiftAB"); // 0x0c: WBShiftAB
    map.insert(13, "WBShiftGM"); // 0x0d: WBShiftGM
    map.insert(14, "UnsharpMaskFineness"); // 0x0e: UnsharpMaskFineness
    map.insert(15, "UnsharpMaskThreshold"); // 0x0f: UnsharpMaskThreshold
    map
});

impl CanonProcessingTable {
    /// Create new table instance
    pub fn new() -> Self {
        Self {
            default_format: "int16s",
            first_entry: 1,
            groups: ("MakerNotes", "Image"),
        }
    }

    /// Get tag name for offset
    pub fn get_tag_name(&self, offset: u16) -> Option<&'static str> {
        PROCESSING_TAGS.get(&offset).copied()
    }

    /// Get all valid offsets for this table
    pub fn get_offsets(&self) -> Vec<u16> {
        PROCESSING_TAGS.keys().copied().collect()
    }
}

impl Default for CanonProcessingTable {
    fn default() -> Self {
        Self::new()
    }
}
