//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

//! Sony ProcessBinaryData table CameraSettings generated from Sony.pm
//! ExifTool: Sony.pm %Sony::CameraSettings

use std::collections::HashMap;
use std::sync::LazyLock;

/// Sony ProcessBinaryData table for CameraSettings
/// Total tags: 55
#[derive(Debug, Clone)]
pub struct SonyCameraSettingsTable {
    pub default_format: &'static str,         // "int16u"
    pub first_entry: i32,                     // 0
    pub groups: (&'static str, &'static str), // ("MakerNotes", "Camera")
}

/// Tag definitions for Sony CameraSettings
pub static CAMERASETTINGS_TAGS: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(0, "ExposureTime"); // 0x00: ExposureTime
    map.insert(1, "FNumber"); // 0x01: FNumber
    map.insert(2, "HighSpeedSync"); // 0x02: HighSpeedSync
    map.insert(3, "ExposureCompensationSet"); // 0x03: ExposureCompensationSet
    map.insert(4, "DriveMode"); // 0x04: DriveMode
    map.insert(5, "WhiteBalanceSetting"); // 0x05: WhiteBalanceSetting
    map.insert(6, "WhiteBalanceFineTune"); // 0x06: WhiteBalanceFineTune
    map.insert(7, "ColorTemperatureSet"); // 0x07: ColorTemperatureSet
    map.insert(8, "ColorCompensationFilterSet"); // 0x08: ColorCompensationFilterSet
    map.insert(12, "ColorTemperatureCustom"); // 0x0c: ColorTemperatureCustom
    map.insert(13, "ColorCompensationFilterCustom"); // 0x0d: ColorCompensationFilterCustom
    map.insert(15, "WhiteBalance"); // 0x0f: WhiteBalance
    map.insert(16, "FocusModeSetting"); // 0x10: FocusModeSetting
    map.insert(17, "AFAreaMode"); // 0x11: AFAreaMode
    map.insert(18, "AFPointSetting"); // 0x12: AFPointSetting
    map.insert(19, "FlashMode"); // 0x13: FlashMode
    map.insert(20, "FlashExposureCompSet"); // 0x14: FlashExposureCompSet
    map.insert(21, "MeteringMode"); // 0x15: MeteringMode
    map.insert(22, "ISOSetting"); // 0x16: ISOSetting
    map.insert(24, "DynamicRangeOptimizerMode"); // 0x18: DynamicRangeOptimizerMode
    map.insert(25, "DynamicRangeOptimizerLevel"); // 0x19: DynamicRangeOptimizerLevel
    map.insert(26, "CreativeStyle"); // 0x1a: CreativeStyle
    map.insert(27, "ColorSpace"); // 0x1b: ColorSpace
    map.insert(28, "Sharpness"); // 0x1c: Sharpness
    map.insert(29, "Contrast"); // 0x1d: Contrast
    map.insert(30, "Saturation"); // 0x1e: Saturation
    map.insert(31, "ZoneMatchingValue"); // 0x1f: ZoneMatchingValue
    map.insert(34, "Brightness"); // 0x22: Brightness
    map.insert(35, "FlashControl"); // 0x23: FlashControl
    map.insert(40, "PrioritySetupShutterRelease"); // 0x28: PrioritySetupShutterRelease
    map.insert(41, "AFIlluminator"); // 0x29: AFIlluminator
    map.insert(42, "AFWithShutter"); // 0x2a: AFWithShutter
    map.insert(43, "LongExposureNoiseReduction"); // 0x2b: LongExposureNoiseReduction
    map.insert(44, "HighISONoiseReduction"); // 0x2c: HighISONoiseReduction
    map.insert(45, "ImageStyle"); // 0x2d: ImageStyle
    map.insert(46, "FocusModeSwitch"); // 0x2e: FocusModeSwitch
    map.insert(47, "ShutterSpeedSetting"); // 0x2f: ShutterSpeedSetting
    map.insert(48, "ApertureSetting"); // 0x30: ApertureSetting
    map.insert(60, "ExposureProgram"); // 0x3c: ExposureProgram
    map.insert(61, "ImageStabilizationSetting"); // 0x3d: ImageStabilizationSetting
    map.insert(62, "FlashAction"); // 0x3e: FlashAction
    map.insert(63, "Rotation"); // 0x3f: Rotation
    map.insert(64, "AELock"); // 0x40: AELock
    map.insert(76, "FlashAction2"); // 0x4c: FlashAction2
    map.insert(77, "FocusMode"); // 0x4d: FocusMode
    map.insert(80, "BatteryState"); // 0x50: BatteryState
    map.insert(81, "BatteryLevel"); // 0x51: BatteryLevel
    map.insert(83, "FocusStatus"); // 0x53: FocusStatus
    map.insert(84, "SonyImageSize"); // 0x54: SonyImageSize
    map.insert(85, "AspectRatio"); // 0x55: AspectRatio
    map.insert(86, "Quality"); // 0x56: Quality
    map.insert(88, "ExposureLevelIncrements"); // 0x58: ExposureLevelIncrements
    map.insert(106, "RedEyeReduction"); // 0x6a: RedEyeReduction
    map.insert(154, "FolderNumber"); // 0x9a: FolderNumber
    map.insert(155, "ImageNumber"); // 0x9b: ImageNumber
    map
});

/// Format specifications for Sony CameraSettings
pub static CAMERASETTINGS_FORMATS: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(18, "int16u"); // 0x12: AFPointSetting
    map
});

impl SonyCameraSettingsTable {
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
        CAMERASETTINGS_TAGS.get(&offset).copied()
    }

    /// Get format specification for offset
    pub fn get_format(&self, offset: u16) -> Option<&'static str> {
        CAMERASETTINGS_FORMATS.get(&offset).copied()
    }

    /// Get all valid offsets for this table
    pub fn get_offsets(&self) -> Vec<u16> {
        CAMERASETTINGS_TAGS.keys().copied().collect()
    }
}

impl Default for SonyCameraSettingsTable {
    fn default() -> Self {
        Self::new()
    }
}
