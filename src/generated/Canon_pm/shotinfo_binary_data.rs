//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

//! Canon ProcessBinaryData table ShotInfo generated from Canon.pm
//! ExifTool: Canon.pm %Canon::ShotInfo

use crate::types::{BinaryDataTag, BinaryDataTagVariant};
use std::collections::HashMap;
use std::sync::LazyLock;

/// Canon ProcessBinaryData table for ShotInfo
/// Total tags: 28
#[derive(Debug, Clone)]
pub struct CanonShotInfoTable {
    pub default_format: &'static str,         // "int16s"
    pub first_entry: i32,                     // 1
    pub groups: (&'static str, &'static str), // ("MakerNotes", "Image")
}

/// Tag definitions for Canon ShotInfo
pub static SHOTINFO_TAGS: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(1, "AutoISO"); // 0x01: AutoISO
    map.insert(2, "BaseISO"); // 0x02: BaseISO
    map.insert(3, "MeasuredEV"); // 0x03: MeasuredEV
    map.insert(4, "TargetAperture"); // 0x04: TargetAperture
    map.insert(5, "TargetExposureTime"); // 0x05: TargetExposureTime
    map.insert(6, "ExposureCompensation"); // 0x06: ExposureCompensation
    map.insert(7, "WhiteBalance"); // 0x07: WhiteBalance
    map.insert(8, "SlowShutter"); // 0x08: SlowShutter
    map.insert(9, "SequenceNumber"); // 0x09: SequenceNumber
    map.insert(10, "OpticalZoomCode"); // 0x0a: OpticalZoomCode
    map.insert(12, "CameraTemperature"); // 0x0c: CameraTemperature
    map.insert(13, "FlashGuideNumber"); // 0x0d: FlashGuideNumber
    map.insert(14, "AFPointsInFocus"); // 0x0e: AFPointsInFocus
    map.insert(15, "FlashExposureComp"); // 0x0f: FlashExposureComp
    map.insert(16, "AutoExposureBracketing"); // 0x10: AutoExposureBracketing
    map.insert(17, "AEBBracketValue"); // 0x11: AEBBracketValue
    map.insert(18, "ControlMode"); // 0x12: ControlMode
    map.insert(19, "FocusDistanceUpper"); // 0x13: FocusDistanceUpper
    map.insert(20, "FocusDistanceLower"); // 0x14: FocusDistanceLower
    map.insert(21, "FNumber"); // 0x15: FNumber
    map.insert(22, "ExposureTime"); // 0x16: ExposureTime
    map.insert(23, "MeasuredEV2"); // 0x17: MeasuredEV2
    map.insert(24, "BulbDuration"); // 0x18: BulbDuration
    map.insert(26, "CameraType"); // 0x1a: CameraType
    map.insert(27, "AutoRotate"); // 0x1b: AutoRotate
    map.insert(28, "NDFilter"); // 0x1c: NDFilter
    map.insert(29, "SelfTimer2"); // 0x1d: SelfTimer2
    map.insert(33, "FlashOutput"); // 0x21: FlashOutput
    map
});

/// Conditional variants for Canon ShotInfo
pub static SHOTINFO_VARIANTS: LazyLock<HashMap<u16, BinaryDataTag>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    // Tag 22: ExposureTime (conditional)
    map.insert(
        22,
        BinaryDataTag {
            name: "ExposureTime".to_string(),
            variants: vec![
                BinaryDataTagVariant {
                    name: "ExposureTime".to_string(),
                    condition: Some("$model =~ /(20D|350D|REBEL XT|Kiss Digital N)/".to_string()),
                    format_spec: None,
                    format: None,
                    mask: None,
                    print_conv: None,
                    value_conv: Some(
                        "exp(-Image::ExifTool::Canon::CanonEv($val)*log(2))*1000/32".to_string(),
                    ),
                    print_conv_expr: Some(
                        "Image::ExifTool::Exif::PrintExposureTime($val)".to_string(),
                    ),
                    data_member: None,
                    group: None,
                    priority: Some(0),
                },
                BinaryDataTagVariant {
                    name: "ExposureTime".to_string(),
                    condition: None,
                    format_spec: None,
                    format: None,
                    mask: None,
                    print_conv: None,
                    value_conv: Some(
                        "exp(-Image::ExifTool::Canon::CanonEv($val)*log(2))".to_string(),
                    ),
                    print_conv_expr: Some(
                        "Image::ExifTool::Exif::PrintExposureTime($val)".to_string(),
                    ),
                    data_member: None,
                    group: None,
                    priority: Some(0),
                },
            ],
            format_spec: None,
            format: None,
            mask: None,
            print_conv: None,
            data_member: None,
            group: None,
        },
    );

    map
});

/// Format specifications for Canon ShotInfo
pub static SHOTINFO_FORMATS: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(19, "int16u"); // 0x13: FocusDistanceUpper
    map.insert(20, "int16u"); // 0x14: FocusDistanceLower
    map
});

impl CanonShotInfoTable {
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

    /// Get conditional variants for a tag if available
    pub fn get_conditional_tag(&self, offset: u16) -> Option<&BinaryDataTag> {
        SHOTINFO_VARIANTS.get(&offset)
    }

    /// Check if a tag has conditional variants
    pub fn is_conditional(&self, offset: u16) -> bool {
        SHOTINFO_VARIANTS.contains_key(&offset)
    }
}

impl Default for CanonShotInfoTable {
    fn default() -> Self {
        Self::new()
    }
}
