//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

//! Sony ProcessBinaryData table CameraSettings2 generated from Sony.pm
//! ExifTool: Sony.pm %Sony::CameraSettings2

use std::collections::HashMap;
use std::sync::LazyLock;

/// Sony ProcessBinaryData table for CameraSettings2
/// Total tags: 45
#[derive(Debug, Clone)]
pub struct SonyCameraSettings2Table {
    pub default_format: &'static str,         // "int16u"
    pub first_entry: i32,                     // 0
    pub groups: (&'static str, &'static str), // ("MakerNotes", "Camera")
}

/// Tag definitions for Sony CameraSettings2
pub static CAMERASETTINGS2_TAGS: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(0, "ExposureTime"); // 0x00: ExposureTime
    map.insert(1, "FNumber"); // 0x01: FNumber
    map.insert(2, "HighSpeedSync"); // 0x02: HighSpeedSync
    map.insert(3, "ExposureCompensationSet"); // 0x03: ExposureCompensationSet
    map.insert(4, "WhiteBalanceSetting"); // 0x04: WhiteBalanceSetting
    map.insert(5, "WhiteBalanceFineTune"); // 0x05: WhiteBalanceFineTune
    map.insert(6, "ColorTemperatureSet"); // 0x06: ColorTemperatureSet
    map.insert(7, "ColorCompensationFilterSet"); // 0x07: ColorCompensationFilterSet
    map.insert(8, "CustomWB_RGBLevels"); // 0x08: CustomWB_RGBLevels
    map.insert(11, "ColorTemperatureCustom"); // 0x0b: ColorTemperatureCustom
    map.insert(12, "ColorCompensationFilterCustom"); // 0x0c: ColorCompensationFilterCustom
    map.insert(14, "WhiteBalance"); // 0x0e: WhiteBalance
    map.insert(15, "FocusModeSetting"); // 0x0f: FocusModeSetting
    map.insert(16, "AFAreaMode"); // 0x10: AFAreaMode
    map.insert(17, "AFPointSetting"); // 0x11: AFPointSetting
    map.insert(18, "FlashExposureCompSet"); // 0x12: FlashExposureCompSet
    map.insert(19, "MeteringMode"); // 0x13: MeteringMode
    map.insert(20, "ISOSetting"); // 0x14: ISOSetting
    map.insert(22, "DynamicRangeOptimizerMode"); // 0x16: DynamicRangeOptimizerMode
    map.insert(23, "DynamicRangeOptimizerLevel"); // 0x17: DynamicRangeOptimizerLevel
    map.insert(24, "CreativeStyle"); // 0x18: CreativeStyle
    map.insert(25, "Sharpness"); // 0x19: Sharpness
    map.insert(26, "Contrast"); // 0x1a: Contrast
    map.insert(27, "Saturation"); // 0x1b: Saturation
    map.insert(31, "FlashControl"); // 0x1f: FlashControl
    map.insert(37, "LongExposureNoiseReduction"); // 0x25: LongExposureNoiseReduction
    map.insert(38, "HighISONoiseReduction"); // 0x26: HighISONoiseReduction
    map.insert(39, "ImageStyle"); // 0x27: ImageStyle
    map.insert(40, "ShutterSpeedSetting"); // 0x28: ShutterSpeedSetting
    map.insert(41, "ApertureSetting"); // 0x29: ApertureSetting
    map.insert(60, "ExposureProgram"); // 0x3c: ExposureProgram
    map.insert(61, "ImageStabilizationSetting"); // 0x3d: ImageStabilizationSetting
    map.insert(62, "FlashAction"); // 0x3e: FlashAction
    map.insert(63, "Rotation"); // 0x3f: Rotation
    map.insert(64, "AELock"); // 0x40: AELock
    map.insert(76, "FlashAction2"); // 0x4c: FlashAction2
    map.insert(77, "FocusMode"); // 0x4d: FocusMode
    map.insert(83, "FocusStatus"); // 0x53: FocusStatus
    map.insert(84, "SonyImageSize"); // 0x54: SonyImageSize
    map.insert(85, "AspectRatio"); // 0x55: AspectRatio
    map.insert(86, "Quality"); // 0x56: Quality
    map.insert(88, "ExposureLevelIncrements"); // 0x58: ExposureLevelIncrements
    map.insert(126, "DriveMode"); // 0x7e: DriveMode
    map.insert(127, "FlashMode"); // 0x7f: FlashMode
    map.insert(131, "ColorSpace"); // 0x83: ColorSpace
    map
});

/// Format specifications for Sony CameraSettings2
pub static CAMERASETTINGS2_FORMATS: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(8, "int16u[3]"); // 0x08: CustomWB_RGBLevels
    map.insert(17, "int16u"); // 0x11: AFPointSetting
    map
});

impl SonyCameraSettings2Table {
    /// Create new table instance
    pub fn new() -> Self {
        Self {
            default_format: "int16u",
            first_entry: 0,
            groups: ("MakerNotes", "Camera"),
        }
    }

    /// Get tag name for offset
    pub fn get_tag_name(&self, offset: u16) -> Option<&'static str> {
        CAMERASETTINGS2_TAGS.get(&offset).copied()
    }

    /// Get format specification for offset
    pub fn get_format(&self, offset: u16) -> Option<&'static str> {
        CAMERASETTINGS2_FORMATS.get(&offset).copied()
    }

    /// Get all valid offsets for this table
    pub fn get_offsets(&self) -> Vec<u16> {
        CAMERASETTINGS2_TAGS.keys().copied().collect()
    }
}

impl Default for SonyCameraSettings2Table {
    fn default() -> Self {
        Self::new()
    }
}
