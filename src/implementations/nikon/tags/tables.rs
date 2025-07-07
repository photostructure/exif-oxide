//! Static tag table definitions for Nikon cameras
//!
//! **Trust ExifTool**: This code translates ExifTool's Nikon tag tables verbatim.
//!
//! ExifTool Reference: lib/Image/ExifTool/Nikon.pm tag table definitions

use super::NikonTagTable;
use crate::implementations::nikon::tags::print_conv::{advanced::*, af::*, basic::*};

/// Primary Nikon tag mappings from Nikon::Main table
/// ExifTool: Nikon.pm %Image::ExifTool::Nikon::Main hash (lines 500-1200+)
pub static NIKON_MAIN_TAGS: NikonTagTable = NikonTagTable {
    name: "Nikon::Main",
    model_condition: None, // Applies to all Nikon cameras
    tags: &[
        // Core identification tags
        (0x0001, "MakerNoteVersion", None),
        (0x0002, "ISO", Some(nikon_iso_conv)),
        (0x0003, "ColorMode", Some(nikon_color_mode_conv)),
        (0x0004, "Quality", Some(nikon_quality_conv)),
        (0x0005, "WhiteBalance", Some(nikon_white_balance_conv)),
        (0x0006, "Sharpness", Some(nikon_sharpness_conv)),
        (0x0007, "FocusMode", Some(nikon_focus_mode_conv)),
        (0x0008, "FlashSetting", Some(nikon_flash_setting_conv)),
        (0x0009, "FlashType", None),
        (0x000B, "WhiteBalanceFineTune", None),
        (0x000C, "WB_RBLevels", None),
        (0x000D, "ProgramShift", None),
        (0x000E, "ExposureDifference", None),
        (0x000F, "ISOSelection", None),
        (0x0010, "DataDump", None),
        (0x0011, "PreviewIFD", None),
        (0x0012, "FlashExposureComp", None),
        (0x0013, "ISOSetting", None),
        (0x0014, "ColorBalanceA", None),
        // Encryption key tags (critical for Phase 2)
        (0x001D, "SerialNumber", None), // Encryption key source
        (0x001E, "ColorSpace", Some(nikon_color_space_conv)),
        (0x001F, "VRInfo", None),
        (0x0020, "ImageAuthentication", None),
        (0x0021, "FaceDetect", None),
        (
            0x0022,
            "ActiveDLighting",
            Some(nikon_active_d_lighting_conv),
        ),
        (0x0023, "PictureControlData", None),
        (0x0024, "WorldTime", None),
        (0x0025, "ISOInfo", None),
        // Lens and focus information
        (0x0080, "ImageAdjustment", None),
        (0x0081, "ToneComp", None),
        (0x0082, "AuxiliaryLens", None),
        (0x0083, "LensType", None),
        (0x0084, "Lens", None),
        (0x0085, "ManualFocusDistance", None),
        (0x0086, "DigitalZoom", None),
        (0x0087, "FlashMode", Some(nikon_flash_mode_conv)),
        (0x0088, "AFInfo", None),
        (0x0089, "ShootingMode", Some(nikon_shooting_mode_conv)),
        (0x008A, "AutoBracketRelease", None),
        (0x008B, "LensFStops", None),
        (0x008C, "ContrastCurve", None),
        (0x008D, "ColorHue", None),
        (0x008F, "SceneMode", Some(nikon_scene_mode_conv)),
        (0x0090, "LightSource", None),
        (0x0091, "ShotInfo", None), // Variable table based on camera model
        (0x0092, "HueAdjustment", None),
        (0x0093, "NEFCompression", Some(nikon_nef_compression_conv)),
        (0x0094, "Saturation", None),
        (0x0095, "NoiseReduction", None),
        (0x0096, "LinearizationTable", None),
        (0x0097, "ColorBalance", None),
        (0x0098, "LensData", None),
        (0x0099, "RawImageCenter", None),
        (0x009A, "SensorPixelSize", None),
        (0x009C, "SceneAssist", None),
        (0x009E, "RetouchHistory", None),
        (0x00A0, "SerialNumber2", None),
        (0x00A2, "ImageDataSize", None),
        (0x00A5, "ImageCount", None),
        (0x00A6, "DeletedImageCount", None),
        (0x00A7, "ShutterCount", None), // Second encryption key source
        (0x00A8, "FlashInfo", None),
        (
            0x00A9,
            "ImageOptimization",
            Some(nikon_image_optimization_conv),
        ),
        (0x00AA, "Saturation2", None),
        (0x00AB, "VariProgram", None),
        (
            0x00AC,
            "ImageStabilization",
            Some(nikon_image_stabilization_conv),
        ),
        (0x00AD, "AFResponse", None),
        (0x00B0, "MultiExposure", None),
        (
            0x00B1,
            "HighISONoiseReduction",
            Some(nikon_high_iso_nr_conv),
        ),
        (0x00B3, "ToningEffect", Some(nikon_toning_effect_conv)),
        (0x00B6, "PowerUpTime", None),
        (0x00B7, "AFInfo2", None),
        (0x00B8, "FileInfo", None),
        (0x00B9, "AFTune", None),
        (0x00BB, "RetouchInfo", None),
        (0x00BD, "PictureControlData2", None),
        (0x0103, "ShotInfoD300", None), // Model-specific ShotInfo variant
        (0x0104, "ShotInfoD300b", None),
        (0x0105, "ShotInfoD300s", None),
    ],
};

/// Model-specific tag table for Nikon Z9 camera
/// ExifTool: Nikon.pm ShotInfoZ9 table
pub static NIKON_Z9_SHOT_INFO: NikonTagTable = NikonTagTable {
    name: "Nikon::ShotInfoZ9",
    model_condition: Some("NIKON Z 9"), // Only for Z9 cameras
    tags: &[
        (0x0000, "ShotInfoVersion", None),
        (0x0004, "FirmwareVersion", None),
        (0x0130, "AFAreaMode", Some(nikon_af_area_mode_conv)),
        (0x0131, "VibrationReduction", Some(nikon_vr_conv)),
        (0x0135, "VRMode", Some(nikon_vr_mode_conv)),
        (
            0x0370,
            "SubjectDetection",
            Some(nikon_subject_detection_conv),
        ),
        (
            0x0371,
            "DynamicAFAreaSize",
            Some(nikon_dynamic_af_area_conv),
        ),
        (0x0372, "HDR", Some(nikon_hdr_conv)),
        (0x0373, "PixelShift", Some(nikon_pixel_shift_conv)),
    ],
};

/// Model-specific tag table for Nikon Z8 camera
/// ExifTool: Nikon.pm ShotInfoZ8 table
pub static NIKON_Z8_SHOT_INFO: NikonTagTable = NikonTagTable {
    name: "Nikon::ShotInfoZ8",
    model_condition: Some("NIKON Z 8"), // Only for Z8 cameras
    tags: &[
        (0x0000, "ShotInfoVersion", None),
        (0x0004, "FirmwareVersion", None),
        (0x0130, "AFAreaMode", Some(nikon_af_area_mode_conv)),
        (0x0131, "VibrationReduction", Some(nikon_vr_conv)),
        (0x0135, "VRMode", Some(nikon_vr_mode_conv)),
        (
            0x0370,
            "SubjectDetection",
            Some(nikon_subject_detection_conv),
        ),
        (
            0x0371,
            "DynamicAFAreaSize",
            Some(nikon_dynamic_af_area_conv),
        ),
        (0x0372, "HDR", Some(nikon_hdr_conv)),
        (0x0373, "PixelShift", Some(nikon_pixel_shift_conv)),
        (
            0x0374,
            "HighFrequencyFlickerReduction",
            Some(nikon_flicker_reduction_conv),
        ),
    ],
};

/// Model-specific tag table for Nikon D850 camera
/// ExifTool: Nikon.pm ShotInfoD850 table
pub static NIKON_D850_SHOT_INFO: NikonTagTable = NikonTagTable {
    name: "Nikon::ShotInfoD850",
    model_condition: Some("NIKON D850"), // Only for D850 cameras
    tags: &[
        (0x0000, "ShotInfoVersion", None),
        (0x0004, "FirmwareVersion", None),
        (0x0039, "ExposureMode", Some(nikon_exposure_mode_conv)),
        (0x0130, "AFAreaMode", Some(nikon_af_area_mode_conv)),
        (0x0131, "VibrationReduction", Some(nikon_vr_conv)),
        (0x0135, "VRMode", Some(nikon_vr_mode_conv)),
        (
            0x0200,
            "MultiSelectorShootMode",
            Some(nikon_multi_selector_conv),
        ),
        (
            0x0201,
            "FlashControlCommanderMode",
            Some(nikon_flash_commander_conv),
        ),
    ],
};

/// Model-specific tag table for Nikon Z6III camera
/// ExifTool: Nikon.pm ShotInfoZ6III table
pub static NIKON_Z6III_SHOT_INFO: NikonTagTable = NikonTagTable {
    name: "Nikon::ShotInfoZ6III",
    model_condition: Some("NIKON Z 6III"), // Only for Z6III cameras
    tags: &[
        (0x0000, "ShotInfoVersion", None),
        (0x0004, "FirmwareVersion", None),
        (0x0130, "AFAreaMode", Some(nikon_af_area_mode_conv)),
        (0x0131, "VibrationReduction", Some(nikon_vr_conv)),
        (0x0135, "VRMode", Some(nikon_vr_mode_conv)),
        (
            0x0370,
            "SubjectDetection",
            Some(nikon_subject_detection_conv),
        ),
        (
            0x0371,
            "DynamicAFAreaSize",
            Some(nikon_dynamic_af_area_conv),
        ),
        (0x0375, "PreReleaseCapture", Some(nikon_pre_capture_conv)),
    ],
};

/// Model-specific tag table for Nikon D6 camera
/// ExifTool: Nikon.pm ShotInfoD6 table
pub static NIKON_D6_SHOT_INFO: NikonTagTable = NikonTagTable {
    name: "Nikon::ShotInfoD6",
    model_condition: Some("NIKON D6"), // Only for D6 cameras
    tags: &[
        (0x0000, "ShotInfoVersion", None),
        (0x0004, "FirmwareVersion", None),
        (0x0039, "ExposureMode", Some(nikon_exposure_mode_conv)),
        (0x0130, "AFAreaMode", Some(nikon_af_area_mode_conv)),
        (0x0131, "VibrationReduction", Some(nikon_vr_conv)),
        (0x0135, "VRMode", Some(nikon_vr_mode_conv)),
        (
            0x0200,
            "MultiSelectorShootMode",
            Some(nikon_multi_selector_conv),
        ),
        (
            0x0376,
            "GroupAreaAFIllumination",
            Some(nikon_group_area_illumination_conv),
        ),
    ],
};
