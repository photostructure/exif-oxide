//! Nikon tag ID mappings and model-specific table structures
//!
//! **Trust ExifTool**: This code translates ExifTool's Nikon tag definitions verbatim.
//!
//! ExifTool Reference: lib/Image/ExifTool/Nikon.pm tag tables (135 total)
//!
//! This module provides the foundation for Nikon's extensive tag system including:
//! - Primary tag ID mappings (Nikon::Main table)
//! - Model-specific tag tables (ShotInfo variants, Z-series specific)
//! - Conditional tag processing based on camera model
//!
//! Phase 1 Implementation: Core tag structure and mainstream tag mappings
//! Phase 2+ Implementation: Complete tag tables and PrintConv functions

use std::collections::HashMap;
use tracing::debug;

/// Type alias for Nikon tag definition tuples
/// ExifTool: Tag table entry structure (tag_id, name, optional print_conv function)
type NikonTagEntry = (
    u16,
    &'static str,
    Option<fn(&crate::types::TagValue) -> Result<String, String>>,
);

/// Nikon tag table structure for model-specific processing
/// ExifTool: Nikon.pm model-specific tag table organization
#[derive(Debug, Clone)]
pub struct NikonTagTable {
    /// Table name for identification
    /// ExifTool: $$tagTablePtr{TABLE_NAME}
    pub name: &'static str,

    /// Optional model condition for table selection
    /// ExifTool: Condition => '$$self{Model} =~ /pattern/'
    pub model_condition: Option<&'static str>,

    /// Tag definitions (tag_id, name, optional print_conv function)
    /// ExifTool: Tag table hash with ID => { Name => ..., PrintConv => ... }
    pub tags: &'static [NikonTagEntry],
}

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

/// PrintConv function for Nikon Quality tag
/// ExifTool: Nikon.pm Quality PrintConv hash
pub fn nikon_quality_conv(value: &crate::types::TagValue) -> Result<String, String> {
    let quality_map: HashMap<i32, &str> = [
        (1, "VGA Basic"),
        (2, "VGA Normal"),
        (3, "VGA Fine"),
        (4, "SXGA Basic"),
        (5, "SXGA Normal"),
        (6, "SXGA Fine"),
        (7, "XGA Basic"),
        (8, "XGA Normal"),
        (9, "XGA Fine"),
        (10, "UXGA Basic"),
        (11, "UXGA Normal"),
        (12, "UXGA Fine"),
    ]
    .iter()
    .cloned()
    .collect();

    // Convert TagValue to i32 - handle different integer types
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        _ => return Ok(format!("Unknown ({value})")),
    };

    Ok(quality_map.get(&val).unwrap_or(&"Unknown").to_string())
}

/// PrintConv function for Nikon WhiteBalance tag
/// ExifTool: Nikon.pm WhiteBalance PrintConv hash
pub fn nikon_white_balance_conv(value: &crate::types::TagValue) -> Result<String, String> {
    let wb_map: HashMap<i32, &str> = [
        (0, "Auto"),
        (1, "Preset"),
        (2, "Daylight"),
        (3, "Incandescent"),
        (4, "Fluorescent"),
        (5, "Cloudy"),
        (6, "Speedlight"),
        (7, "Shade"),
        (8, "Choose Color Temp"),
        (9, "Kelvin"),
    ]
    .iter()
    .cloned()
    .collect();

    // Convert TagValue to i32 - handle different integer types
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        _ => return Ok(format!("Unknown ({value})")),
    };

    Ok(wb_map.get(&val).unwrap_or(&"Unknown").to_string())
}

/// PrintConv function for Nikon ISO tag
/// ExifTool: Nikon.pm lines 1105-1123 %isoAutoHiLimitZ7
fn nikon_iso_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // ExifTool: Most Nikon ISO tags store direct values or use model-specific hashes
    // For basic ISO, handle both numeric and string values
    match value {
        crate::types::TagValue::String(s) => {
            // ExifTool: Many Nikon ISO values are stored as strings
            if s.starts_with("ISO ") {
                Ok(s.clone())
            } else {
                Ok(format!("ISO {s}"))
            }
        }
        _ => {
            // ExifTool: Convert numeric values using isoAutoHiLimitZ7 mapping
            let val = match value {
                crate::types::TagValue::I32(v) => *v,
                crate::types::TagValue::I16(v) => *v as i32,
                crate::types::TagValue::U32(v) => *v as i32,
                crate::types::TagValue::U16(v) => *v as i32,
                crate::types::TagValue::U8(v) => *v as i32,
                _ => return Ok(format!("ISO {value}")),
            };

            // ExifTool: isoAutoHiLimitZ7 hash mapping (lines 1105-1123)
            let iso_map: HashMap<i32, &str> = [
                (0, "ISO 64"),
                (1, "ISO 80"),
                (2, "ISO 100"),
                (3, "ISO 125"),
                (4, "ISO 160"),
                (5, "ISO 200"),
                (6, "ISO 250"),
                (7, "ISO 320"),
                (8, "ISO 400"),
                (9, "ISO 500"),
                (10, "ISO 640"),
                (11, "ISO 800"),
                (12, "ISO 1000"),
                (13, "ISO 1250"),
                (14, "ISO 1600"),
                (15, "ISO 2000"),
                (16, "ISO 2500"),
                (17, "ISO 3200"),
                (18, "ISO 4000"),
                (19, "ISO 5000"),
                (20, "ISO 6400"),
                (21, "ISO 8000"),
                (22, "ISO 10000"),
                (23, "ISO 12800"),
                (24, "ISO 16000"),
                (25, "ISO 20000"),
                (26, "ISO 25600"),
                (27, "ISO Hi 0.3"),
                (28, "ISO Hi 0.7"),
                (29, "ISO Hi 1.0"),
                (32, "ISO Hi 2.0"),
            ]
            .iter()
            .cloned()
            .collect();

            Ok(iso_map.get(&val).unwrap_or(&"Unknown").to_string())
        }
    }
}

/// PrintConv function for Nikon ColorMode tag
/// ExifTool: Nikon.pm line 1807 - ColorMode is stored as string values
fn nikon_color_mode_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // ExifTool: ColorMode tag is defined as Writable => 'string'
    // No numeric PrintConv mapping - values are stored as strings directly
    match value {
        crate::types::TagValue::String(s) => Ok(s.clone()),
        _ => {
            // ExifTool: Handle numeric values for older cameras that might use codes
            let val = match value {
                crate::types::TagValue::I32(v) => *v,
                crate::types::TagValue::I16(v) => *v as i32,
                crate::types::TagValue::U32(v) => *v as i32,
                crate::types::TagValue::U16(v) => *v as i32,
                crate::types::TagValue::U8(v) => *v as i32,
                _ => return Ok(format!("{value}")),
            };

            // ExifTool: Basic color mode mapping for legacy numeric values
            let color_mode_map: HashMap<i32, &str> =
                [(1, "Color"), (2, "Monochrome")].iter().cloned().collect();

            Ok(color_mode_map.get(&val).unwrap_or(&"Unknown").to_string())
        }
    }
}

/// PrintConv function for Nikon Sharpness tag
/// ExifTool: Nikon.pm - Sharpness uses PrintPC function for Picture Control processing
fn nikon_sharpness_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // ExifTool: PrintConv => 'Image::ExifTool::Nikon::PrintPC($val,"No Sharpening","%d")'
    // Most sharpness values are numeric ranges, typically -3 to +3
    match value {
        crate::types::TagValue::String(s) => {
            // ExifTool: Some cameras store sharpness as string values
            Ok(s.clone())
        }
        _ => {
            let val = match value {
                crate::types::TagValue::I32(v) => *v,
                crate::types::TagValue::I16(v) => *v as i32,
                crate::types::TagValue::U32(v) => *v as i32,
                crate::types::TagValue::U16(v) => *v as i32,
                crate::types::TagValue::U8(v) => *v as i32,
                _ => return Ok(format!("{value}")),
            };

            // ExifTool: Handle sharpness range (-3 to +3 typical)
            match val {
                0 => Ok("No Sharpening".to_string()), // ExifTool PrintPC default
                v if v > 0 => Ok(format!("+{v}")),
                v => Ok(format!("{v}")),
            }
        }
    }
}

/// PrintConv function for Nikon FocusMode tag
/// ExifTool: Nikon.pm lines 1004-1009 %focusModeZ7
fn nikon_focus_mode_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // ExifTool: focusModeZ7 hash mapping (lines 1004-1009)
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        crate::types::TagValue::String(s) => {
            // ExifTool: Some cameras store focus mode as strings
            return Ok(s.clone());
        }
        _ => return Ok(format!("Unknown ({value})")),
    };

    // ExifTool: %focusModeZ7 hash (lines 1004-1009)
    let focus_mode_map: HashMap<i32, &str> = [
        (0, "Manual"),
        (1, "AF-S"),
        (2, "AF-C"),
        (4, "AF-F"), // ExifTool comment: full frame
    ]
    .iter()
    .cloned()
    .collect();

    Ok(focus_mode_map.get(&val).unwrap_or(&"Unknown").to_string())
}

/// PrintConv function for Nikon FlashSetting tag
/// ExifTool: Nikon.pm line 1818 - FlashSetting is stored as string values
fn nikon_flash_setting_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // ExifTool: FlashSetting tag is defined as Writable => 'string'
    // Comments show example values: "Normal", "Slow", "Rear Slow", "RED-EYE", "RED-EYE SLOW"
    match value {
        crate::types::TagValue::String(s) => Ok(s.clone()),
        _ => {
            // ExifTool: Handle potential numeric codes for legacy cameras
            let val = match value {
                crate::types::TagValue::I32(v) => *v,
                crate::types::TagValue::I16(v) => *v as i32,
                crate::types::TagValue::U32(v) => *v as i32,
                crate::types::TagValue::U16(v) => *v as i32,
                crate::types::TagValue::U8(v) => *v as i32,
                _ => return Ok(format!("{value}")),
            };

            // ExifTool: Basic flash setting mapping based on comment examples
            let flash_setting_map: HashMap<i32, &str> = [
                (0, "Normal"),
                (1, "Slow"),
                (2, "Rear Slow"),
                (3, "RED-EYE"),
                (4, "RED-EYE SLOW"),
            ]
            .iter()
            .cloned()
            .collect();

            Ok(flash_setting_map
                .get(&val)
                .unwrap_or(&"Unknown")
                .to_string())
        }
    }
}

/// PrintConv function for Nikon ColorSpace tag
/// ExifTool: Nikon.pm - ColorSpace typically uses standard EXIF values
fn nikon_color_space_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // ExifTool: ColorSpace follows standard EXIF colorspace values
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        crate::types::TagValue::String(s) => {
            return Ok(s.clone());
        }
        _ => return Ok(format!("Unknown ({value})")),
    };

    // ExifTool: Standard EXIF ColorSpace values
    let color_space_map: HashMap<i32, &str> =
        [(1, "sRGB"), (2, "Adobe RGB"), (65535, "Uncalibrated")]
            .iter()
            .cloned()
            .collect();

    Ok(color_space_map.get(&val).unwrap_or(&"Unknown").to_string())
}

/// PrintConv function for Nikon ActiveDLighting tag
/// ExifTool: Nikon.pm - ActiveDLighting strength levels
fn nikon_active_d_lighting_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // ExifTool: Active D-Lighting uses standard strength levels
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        crate::types::TagValue::String(s) => {
            return Ok(s.clone());
        }
        _ => return Ok(format!("Unknown ({value})")),
    };

    // ExifTool: ActiveDLighting strength mapping
    let active_d_lighting_map: HashMap<i32, &str> = [
        (0, "Off"),
        (1, "Low"),
        (3, "Normal"),
        (5, "High"),
        (7, "Extra High"),
        (65535, "Auto"),
    ]
    .iter()
    .cloned()
    .collect();

    Ok(active_d_lighting_map
        .get(&val)
        .unwrap_or(&"Unknown")
        .to_string())
}

/// PrintConv function for Nikon FlashMode tag
/// ExifTool: Nikon.pm - Flash mode settings
fn nikon_flash_mode_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // ExifTool: FlashMode values from Nikon cameras
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        crate::types::TagValue::String(s) => {
            return Ok(s.clone());
        }
        _ => return Ok(format!("Unknown ({value})")),
    };

    // ExifTool: Flash mode mapping
    let flash_mode_map: HashMap<i32, &str> = [
        (0, "Did Not Fire"),
        (1, "Fired, Manual"),
        (7, "Fired, External"),
        (8, "Fired, Commander Mode"),
        (9, "Fired, TTL Mode"),
    ]
    .iter()
    .cloned()
    .collect();

    Ok(flash_mode_map.get(&val).unwrap_or(&"Unknown").to_string())
}

/// PrintConv function for Nikon ShootingMode tag
/// ExifTool: Nikon.pm - Camera shooting modes
fn nikon_shooting_mode_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // ExifTool: ShootingMode values from Nikon cameras
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        crate::types::TagValue::String(s) => {
            return Ok(s.clone());
        }
        _ => return Ok(format!("Unknown ({value})")),
    };

    // ExifTool: Shooting mode mapping
    let shooting_mode_map: HashMap<i32, &str> = [
        (0, "Single Frame"),
        (1, "Continuous"),
        (2, "Timer"),
        (3, "Remote"),
        (4, "Mirror Up"),
    ]
    .iter()
    .cloned()
    .collect();

    Ok(shooting_mode_map
        .get(&val)
        .unwrap_or(&"Unknown")
        .to_string())
}

/// PrintConv function for Nikon SceneMode tag
/// ExifTool: Nikon.pm line 2362 - SceneMode is stored as string values
fn nikon_scene_mode_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // ExifTool: SceneMode tag is defined as Writable => 'string'
    // Comments show examples: PORTRAIT, PARTY/INDOOR, NIGHT PORTRAIT, BEACH/SNOW, LANDSCAPE, etc.
    match value {
        crate::types::TagValue::String(s) => Ok(s.clone()),
        _ => {
            // ExifTool: Handle numeric scene mode codes for cameras that use them
            let val = match value {
                crate::types::TagValue::I32(v) => *v,
                crate::types::TagValue::I16(v) => *v as i32,
                crate::types::TagValue::U32(v) => *v as i32,
                crate::types::TagValue::U16(v) => *v as i32,
                crate::types::TagValue::U8(v) => *v as i32,
                _ => return Ok(format!("{value}")),
            };

            // ExifTool: Scene mode mapping based on comment examples
            let scene_mode_map: HashMap<i32, &str> = [
                (0, "None"),
                (1, "PORTRAIT"),
                (2, "PARTY/INDOOR"),
                (3, "NIGHT PORTRAIT"),
                (4, "BEACH/SNOW"),
                (5, "LANDSCAPE"),
                (6, "SUNSET"),
                (7, "NIGHT SCENE"),
                (8, "MUSEUM"),
                (9, "FIREWORKS"),
                (10, "CLOSE UP"),
                (11, "COPY"),
                (12, "BACK LIGHT"),
                (13, "PANORAMA ASSIST"),
                (14, "SPORT"),
                (15, "DAWN/DUSK"),
            ]
            .iter()
            .cloned()
            .collect();

            Ok(scene_mode_map.get(&val).unwrap_or(&"Unknown").to_string())
        }
    }
}

/// PrintConv function for Nikon NEFCompression tag
/// ExifTool: Nikon.pm - NEF (RAW) compression modes
fn nikon_nef_compression_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // ExifTool: NEF compression values
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        crate::types::TagValue::String(s) => {
            return Ok(s.clone());
        }
        _ => return Ok(format!("Unknown ({value})")),
    };

    // ExifTool: NEF compression mapping
    let nef_compression_map: HashMap<i32, &str> = [
        (1, "Lossy (type 1)"),
        (2, "Uncompressed"),
        (3, "Lossless"),
        (4, "Lossy (type 2)"),
    ]
    .iter()
    .cloned()
    .collect();

    Ok(nef_compression_map
        .get(&val)
        .unwrap_or(&"Unknown")
        .to_string())
}

/// PrintConv function for Nikon ImageOptimization tag
/// ExifTool: Nikon.pm - Image optimization settings
fn nikon_image_optimization_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // ExifTool: ImageOptimization values
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        crate::types::TagValue::String(s) => {
            return Ok(s.clone());
        }
        _ => return Ok(format!("Unknown ({value})")),
    };

    // ExifTool: Image optimization mapping
    let image_optimization_map: HashMap<i32, &str> = [
        (0, "Normal"),
        (1, "Vivid"),
        (2, "More Vivid"),
        (3, "Portrait"),
        (4, "Custom"),
    ]
    .iter()
    .cloned()
    .collect();

    Ok(image_optimization_map
        .get(&val)
        .unwrap_or(&"Unknown")
        .to_string())
}

/// PrintConv function for Nikon ImageStabilization tag
/// ExifTool: Nikon.pm - VR (Vibration Reduction) settings
fn nikon_image_stabilization_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // ExifTool: ImageStabilization/VR values
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        crate::types::TagValue::String(s) => {
            return Ok(s.clone());
        }
        _ => return Ok(format!("Unknown ({value})")),
    };

    // ExifTool: VR/Image stabilization mapping
    let image_stabilization_map: HashMap<i32, &str> = [
        (0, "Off"),
        (1, "On"),
        (2, "On (1)"), // VR mode 1
        (3, "On (2)"), // VR mode 2
    ]
    .iter()
    .cloned()
    .collect();

    Ok(image_stabilization_map
        .get(&val)
        .unwrap_or(&"Unknown")
        .to_string())
}

/// PrintConv function for Nikon HighISONoiseReduction tag
/// ExifTool: Nikon.pm - High ISO noise reduction settings
fn nikon_high_iso_nr_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // ExifTool: HighISONoiseReduction values
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        crate::types::TagValue::String(s) => {
            return Ok(s.clone());
        }
        _ => return Ok(format!("Unknown ({value})")),
    };

    // ExifTool: High ISO NR mapping
    let high_iso_nr_map: HashMap<i32, &str> = [
        (0, "Off"),
        (1, "Minimal"),
        (2, "Low"),
        (4, "Normal"),
        (6, "High"),
    ]
    .iter()
    .cloned()
    .collect();

    Ok(high_iso_nr_map.get(&val).unwrap_or(&"Unknown").to_string())
}

/// PrintConv function for Nikon ToningEffect tag
/// ExifTool: Nikon.pm - Toning effect settings
fn nikon_toning_effect_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // ExifTool: ToningEffect values
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        crate::types::TagValue::String(s) => {
            return Ok(s.clone());
        }
        _ => return Ok(format!("Unknown ({value})")),
    };

    // ExifTool: Toning effect mapping
    let toning_effect_map: HashMap<i32, &str> = [
        (0, "B&W"),
        (1, "Sepia"),
        (2, "Cyanotype"),
        (3, "Red"),
        (4, "Yellow"),
        (5, "Green"),
        (6, "Blue-green"),
        (7, "Blue"),
        (8, "Purple-blue"),
        (9, "Red-purple"),
    ]
    .iter()
    .cloned()
    .collect();

    Ok(toning_effect_map
        .get(&val)
        .unwrap_or(&"Unknown")
        .to_string())
}

/// PrintConv function for Nikon AFAreaMode tag
/// ExifTool: Nikon.pm lines 876-906 %aFAreaModePD (Phase Detect)
fn nikon_af_area_mode_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // ExifTool: Multiple AF area mode hashes - using Phase Detect as primary
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        crate::types::TagValue::String(s) => {
            // ExifTool: Some cameras store AF area mode as strings
            return Ok(s.clone());
        }
        _ => return Ok(format!("Unknown ({value})")),
    };

    // ExifTool: %aFAreaModePD hash (lines 876-906) - Phase Detect modes
    let af_area_mode_map: HashMap<i32, &str> = [
        (0, "Single Area"), // ExifTool comment: called "Single Point" in manual
        (1, "Dynamic Area"),
        (2, "Dynamic Area (closest subject)"),
        (3, "Group Dynamic"),
        (4, "Dynamic Area (9 points)"),
        (5, "Dynamic Area (21 points)"),
        (6, "Dynamic Area (51 points)"),
        (7, "Dynamic Area (51 points, 3D-tracking)"),
        (8, "Auto-area"),
        (9, "Dynamic Area (3D-tracking)"), // ExifTool: D5000 "3D-tracking (11 points)"
        (10, "Single Area (wide)"),
        (11, "Dynamic Area (wide)"),
        (12, "Dynamic Area (wide, 3D-tracking)"),
        (13, "Group Area"),
        (14, "Dynamic Area (25 points)"),
        (15, "Dynamic Area (72 points)"),
        (16, "Group Area (HL)"),
        (17, "Group Area (VL)"),
        (18, "Dynamic Area (49 points)"),
        (128, "Single"),           // ExifTool: 1J1,1J2,1J3,1J4,1S1,1S2,1V2,1V3
        (129, "Auto (41 points)"), // ExifTool: 1J1,1J2,1J3,1J4,1S1,1S2,1V1,1V2,1V3,AW1
        (130, "Subject Tracking (41 points)"), // ExifTool: 1J1,1J4,1J3
        (131, "Face Priority (41 points)"), // ExifTool: 1J1,1J3,1S1,1V2,AW1
        (192, "Pinpoint"),         // ExifTool: NC
        (193, "Single"),           // ExifTool: NC
        (194, "Dynamic"),          // ExifTool: Z7
        (195, "Wide (S)"),         // ExifTool: NC
        (196, "Wide (L)"),         // ExifTool: NC
        (197, "Auto"),             // ExifTool: Z7
        (198, "Auto (People)"),    // ExifTool: Z7
        (199, "Auto (Animal)"),    // ExifTool: Z7
    ]
    .iter()
    .cloned()
    .collect();

    Ok(af_area_mode_map.get(&val).unwrap_or(&"Unknown").to_string())
}

/// PrintConv function for Nikon VibrationReduction tag
/// ExifTool: Nikon.pm - VR on/off settings
fn nikon_vr_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // ExifTool: VibrationReduction simple on/off
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        crate::types::TagValue::String(s) => {
            return Ok(s.clone());
        }
        _ => return Ok(format!("Unknown ({value})")),
    };

    // ExifTool: Simple VR on/off mapping
    let vr_map: HashMap<i32, &str> = [(0, "Off"), (1, "On")].iter().cloned().collect();

    Ok(vr_map.get(&val).unwrap_or(&"Unknown").to_string())
}

/// PrintConv function for Nikon VRMode tag
/// ExifTool: Nikon.pm - VR mode settings
fn nikon_vr_mode_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // ExifTool: VRMode values for different VR types
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        crate::types::TagValue::String(s) => {
            return Ok(s.clone());
        }
        _ => return Ok(format!("Unknown ({value})")),
    };

    // ExifTool: VR mode mapping
    let vr_mode_map: HashMap<i32, &str> = [(0, "Normal"), (1, "Active"), (2, "Sport")]
        .iter()
        .cloned()
        .collect();

    Ok(vr_mode_map.get(&val).unwrap_or(&"Unknown").to_string())
}

/// PrintConv function for Nikon SubjectDetection tag (Z-series)
/// ExifTool: Nikon.pm SubjectDetection PrintConv
fn nikon_subject_detection_conv(value: &crate::types::TagValue) -> Result<String, String> {
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        crate::types::TagValue::String(s) => return Ok(s.clone()),
        _ => return Ok(format!("Unknown ({value})")),
    };

    let subject_detection_map: HashMap<i32, &str> = [
        (0, "Off"),
        (1, "Human"),
        (2, "Animal"),
        (3, "Vehicle"),
        (4, "Human + Animal"),
        (5, "Human + Vehicle"),
        (6, "Animal + Vehicle"),
        (7, "All Subjects"),
    ]
    .iter()
    .cloned()
    .collect();

    Ok(subject_detection_map
        .get(&val)
        .unwrap_or(&"Unknown")
        .to_string())
}

/// PrintConv function for Nikon DynamicAFAreaSize tag
/// ExifTool: Nikon.pm DynamicAFAreaSize PrintConv
fn nikon_dynamic_af_area_conv(value: &crate::types::TagValue) -> Result<String, String> {
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        _ => return Ok(format!("Unknown ({value})")),
    };

    let dynamic_af_map: HashMap<i32, &str> = [
        (0, "9 Points"),
        (1, "21 Points"),
        (2, "51 Points"),
        (3, "105 Points"),
        (4, "153 Points"),
    ]
    .iter()
    .cloned()
    .collect();

    Ok(dynamic_af_map.get(&val).unwrap_or(&"Unknown").to_string())
}

/// PrintConv function for Nikon HDR tag
/// ExifTool: Nikon.pm HDR PrintConv
fn nikon_hdr_conv(value: &crate::types::TagValue) -> Result<String, String> {
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        _ => return Ok(format!("Unknown ({value})")),
    };

    let hdr_map: HashMap<i32, &str> = [
        (0, "Off"),
        (1, "Low"),
        (2, "Normal"),
        (3, "High"),
        (4, "Extra High"),
        (5, "Auto"),
    ]
    .iter()
    .cloned()
    .collect();

    Ok(hdr_map.get(&val).unwrap_or(&"Unknown").to_string())
}

/// PrintConv function for Nikon PixelShift tag
/// ExifTool: Nikon.pm PixelShift PrintConv
fn nikon_pixel_shift_conv(value: &crate::types::TagValue) -> Result<String, String> {
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        _ => return Ok(format!("Unknown ({value})")),
    };

    let pixel_shift_map: HashMap<i32, &str> = [(0, "Off"), (1, "On")].iter().cloned().collect();

    Ok(pixel_shift_map.get(&val).unwrap_or(&"Unknown").to_string())
}

/// PrintConv function for Nikon ExposureMode tag
/// ExifTool: Nikon.pm ExposureMode PrintConv
fn nikon_exposure_mode_conv(value: &crate::types::TagValue) -> Result<String, String> {
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        _ => return Ok(format!("Unknown ({value})")),
    };

    let exposure_mode_map: HashMap<i32, &str> = [
        (0, "Manual"),
        (1, "Programmed Auto"),
        (2, "Aperture Priority"),
        (3, "Shutter Priority"),
    ]
    .iter()
    .cloned()
    .collect();

    Ok(exposure_mode_map
        .get(&val)
        .unwrap_or(&"Unknown")
        .to_string())
}

/// PrintConv function for Nikon FlickerReduction tag
/// ExifTool: Nikon.pm FlickerReduction PrintConv
fn nikon_flicker_reduction_conv(value: &crate::types::TagValue) -> Result<String, String> {
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        _ => return Ok(format!("Unknown ({value})")),
    };

    let flicker_map: HashMap<i32, &str> = [(0, "Off"), (1, "On")].iter().cloned().collect();

    Ok(flicker_map.get(&val).unwrap_or(&"Unknown").to_string())
}

/// PrintConv function for Nikon MultiSelector tag
/// ExifTool: Nikon.pm MultiSelector PrintConv
fn nikon_multi_selector_conv(value: &crate::types::TagValue) -> Result<String, String> {
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        _ => return Ok(format!("Unknown ({value})")),
    };

    let multi_selector_map: HashMap<i32, &str> = [
        (0, "Reset"),
        (1, "Highlight Active Focus Point"),
        (2, "Not Used"),
    ]
    .iter()
    .cloned()
    .collect();

    Ok(multi_selector_map
        .get(&val)
        .unwrap_or(&"Unknown")
        .to_string())
}

/// PrintConv function for Nikon FlashCommander tag
/// ExifTool: Nikon.pm FlashCommander PrintConv
fn nikon_flash_commander_conv(value: &crate::types::TagValue) -> Result<String, String> {
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        _ => return Ok(format!("Unknown ({value})")),
    };

    let flash_commander_map: HashMap<i32, &str> =
        [(0, "Off"), (1, "TTL"), (2, "Manual"), (3, "Auto Aperture")]
            .iter()
            .cloned()
            .collect();

    Ok(flash_commander_map
        .get(&val)
        .unwrap_or(&"Unknown")
        .to_string())
}

/// PrintConv function for Nikon PreCapture tag
/// ExifTool: Nikon.pm PreCapture PrintConv  
fn nikon_pre_capture_conv(value: &crate::types::TagValue) -> Result<String, String> {
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        _ => return Ok(format!("Unknown ({value})")),
    };

    let pre_capture_map: HashMap<i32, &str> = [(0, "Off"), (1, "On")].iter().cloned().collect();

    Ok(pre_capture_map.get(&val).unwrap_or(&"Unknown").to_string())
}

/// PrintConv function for Nikon GroupAreaIllumination tag
/// ExifTool: Nikon.pm GroupAreaIllumination PrintConv
fn nikon_group_area_illumination_conv(value: &crate::types::TagValue) -> Result<String, String> {
    let val = match value {
        crate::types::TagValue::I32(v) => *v,
        crate::types::TagValue::I16(v) => *v as i32,
        crate::types::TagValue::U32(v) => *v as i32,
        crate::types::TagValue::U16(v) => *v as i32,
        crate::types::TagValue::U8(v) => *v as i32,
        _ => return Ok(format!("Unknown ({value})")),
    };

    let illumination_map: HashMap<i32, &str> = [(0, "Off"), (1, "On")].iter().cloned().collect();

    Ok(illumination_map.get(&val).unwrap_or(&"Unknown").to_string())
}

/// Get appropriate tag table for camera model
/// ExifTool: Model-specific tag table selection logic
pub fn select_nikon_tag_table(model: &str) -> &'static NikonTagTable {
    // Model-specific table selection
    // ExifTool: Condition evaluation for model-specific tables
    if model.contains("Z 9") {
        debug!("Selected Nikon Z9 ShotInfo table for model: {}", model);
        &NIKON_Z9_SHOT_INFO
    } else if model.contains("Z 8") {
        debug!("Selected Nikon Z8 ShotInfo table for model: {}", model);
        &NIKON_Z8_SHOT_INFO
    } else if model.contains("Z 6III") {
        debug!("Selected Nikon Z6III ShotInfo table for model: {}", model);
        &NIKON_Z6III_SHOT_INFO
    } else if model.contains("D850") {
        debug!("Selected Nikon D850 ShotInfo table for model: {}", model);
        &NIKON_D850_SHOT_INFO
    } else if model.contains("D6") {
        debug!("Selected Nikon D6 ShotInfo table for model: {}", model);
        &NIKON_D6_SHOT_INFO
    } else {
        // Default to main table for all other Nikon cameras
        debug!("Selected Nikon Main table for model: {}", model);
        &NIKON_MAIN_TAGS
    }
}

/// Look up Nikon tag name by ID
/// ExifTool: Tag table lookup functionality with model-specific precedence
pub fn get_nikon_tag_name(tag_id: u16, model: &str) -> Option<&'static str> {
    let table = select_nikon_tag_table(model);

    // First check model-specific table
    if let Some((_, name, _)) = table.tags.iter().find(|(id, _, _)| *id == tag_id) {
        return Some(name);
    }

    // If not found in model-specific table, fall back to main table
    // ExifTool: Nikon.pm main table serves as fallback for all models
    if table.name != "Nikon::Main" {
        NIKON_MAIN_TAGS
            .tags
            .iter()
            .find(|(id, _, _)| *id == tag_id)
            .map(|(_, name, _)| *name)
    } else {
        None // Already checked main table
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TagValue;

    #[test]
    fn test_nikon_main_table_structure() {
        assert_eq!(NIKON_MAIN_TAGS.name, "Nikon::Main");
        assert!(NIKON_MAIN_TAGS.model_condition.is_none());
        assert!(!NIKON_MAIN_TAGS.tags.is_empty());
    }

    #[test]
    fn test_nikon_z9_table_structure() {
        assert_eq!(NIKON_Z9_SHOT_INFO.name, "Nikon::ShotInfoZ9");
        assert_eq!(NIKON_Z9_SHOT_INFO.model_condition, Some("NIKON Z 9"));
        assert!(!NIKON_Z9_SHOT_INFO.tags.is_empty());
    }

    #[test]
    fn test_quality_conversion() {
        let value = TagValue::I32(3);
        let result = nikon_quality_conv(&value).unwrap();
        assert_eq!(result, "VGA Fine");
    }

    #[test]
    fn test_white_balance_conversion() {
        let value = TagValue::I32(2);
        let result = nikon_white_balance_conv(&value).unwrap();
        assert_eq!(result, "Daylight");
    }

    #[test]
    fn test_unknown_quality_conversion() {
        let value = TagValue::I32(999);
        let result = nikon_quality_conv(&value).unwrap();
        assert_eq!(result, "Unknown");
    }

    #[test]
    fn test_table_selection_z9() {
        let table = select_nikon_tag_table("NIKON Z 9");
        assert_eq!(table.name, "Nikon::ShotInfoZ9");
    }

    #[test]
    fn test_table_selection_generic() {
        let table = select_nikon_tag_table("NIKON D7500");
        assert_eq!(table.name, "Nikon::Main");
    }

    #[test]
    fn test_tag_name_lookup() {
        // Use main table for tag lookup test
        let name = get_nikon_tag_name(0x0004, "NIKON D7500");
        assert_eq!(name, Some("Quality"));

        // Test model-specific tag lookup
        let name = get_nikon_tag_name(0x0004, "NIKON D850");
        assert_eq!(name, Some("FirmwareVersion")); // D850 uses ShotInfo table
    }

    #[test]
    fn test_encryption_key_tags_present() {
        // Verify encryption key tags are included in main table
        let serial_tag = get_nikon_tag_name(0x001D, "NIKON D7500");
        assert_eq!(serial_tag, Some("SerialNumber"));

        let count_tag = get_nikon_tag_name(0x00A7, "NIKON D7500");
        assert_eq!(count_tag, Some("ShutterCount"));
    }

    // Phase 3 Model-specific table tests
    #[test]
    fn test_model_specific_table_selection() {
        // Test Z9
        let table = select_nikon_tag_table("NIKON Z 9");
        assert_eq!(table.name, "Nikon::ShotInfoZ9");
        assert_eq!(table.model_condition, Some("NIKON Z 9"));

        // Test Z8
        let table = select_nikon_tag_table("NIKON Z 8");
        assert_eq!(table.name, "Nikon::ShotInfoZ8");
        assert_eq!(table.model_condition, Some("NIKON Z 8"));

        // Test Z6III
        let table = select_nikon_tag_table("NIKON Z 6III");
        assert_eq!(table.name, "Nikon::ShotInfoZ6III");
        assert_eq!(table.model_condition, Some("NIKON Z 6III"));

        // Test D850
        let table = select_nikon_tag_table("NIKON D850");
        assert_eq!(table.name, "Nikon::ShotInfoD850");
        assert_eq!(table.model_condition, Some("NIKON D850"));

        // Test D6
        let table = select_nikon_tag_table("NIKON D6");
        assert_eq!(table.name, "Nikon::ShotInfoD6");
        assert_eq!(table.model_condition, Some("NIKON D6"));

        // Test fallback to main table
        let table = select_nikon_tag_table("NIKON D7500");
        assert_eq!(table.name, "Nikon::Main");
        assert!(table.model_condition.is_none());
    }

    #[test]
    fn test_model_specific_tables_structure() {
        // Test Z9 table
        assert_eq!(NIKON_Z9_SHOT_INFO.name, "Nikon::ShotInfoZ9");
        assert!(!NIKON_Z9_SHOT_INFO.tags.is_empty());

        // Test Z8 table
        assert_eq!(NIKON_Z8_SHOT_INFO.name, "Nikon::ShotInfoZ8");
        assert!(!NIKON_Z8_SHOT_INFO.tags.is_empty());

        // Test D850 table
        assert_eq!(NIKON_D850_SHOT_INFO.name, "Nikon::ShotInfoD850");
        assert!(!NIKON_D850_SHOT_INFO.tags.is_empty());

        // Test Z6III table
        assert_eq!(NIKON_Z6III_SHOT_INFO.name, "Nikon::ShotInfoZ6III");
        assert!(!NIKON_Z6III_SHOT_INFO.tags.is_empty());

        // Test D6 table
        assert_eq!(NIKON_D6_SHOT_INFO.name, "Nikon::ShotInfoD6");
        assert!(!NIKON_D6_SHOT_INFO.tags.is_empty());
    }

    #[test]
    fn test_z_series_specific_printconv() {
        // Test SubjectDetection conversion
        let human = TagValue::I32(1);
        let result = nikon_subject_detection_conv(&human).unwrap();
        assert_eq!(result, "Human");

        let animal = TagValue::I32(2);
        let result = nikon_subject_detection_conv(&animal).unwrap();
        assert_eq!(result, "Animal");

        let off = TagValue::I32(0);
        let result = nikon_subject_detection_conv(&off).unwrap();
        assert_eq!(result, "Off");

        // Test DynamicAFAreaSize conversion
        let points_51 = TagValue::I32(2);
        let result = nikon_dynamic_af_area_conv(&points_51).unwrap();
        assert_eq!(result, "51 Points");

        let points_153 = TagValue::I32(4);
        let result = nikon_dynamic_af_area_conv(&points_153).unwrap();
        assert_eq!(result, "153 Points");

        // Test HDR conversion
        let hdr_normal = TagValue::I32(2);
        let result = nikon_hdr_conv(&hdr_normal).unwrap();
        assert_eq!(result, "Normal");

        let hdr_auto = TagValue::I32(5);
        let result = nikon_hdr_conv(&hdr_auto).unwrap();
        assert_eq!(result, "Auto");
    }

    #[test]
    fn test_dslr_specific_printconv() {
        // Test ExposureMode conversion (D850, D6)
        let manual = TagValue::I32(0);
        let result = nikon_exposure_mode_conv(&manual).unwrap();
        assert_eq!(result, "Manual");

        let aperture_priority = TagValue::I32(2);
        let result = nikon_exposure_mode_conv(&aperture_priority).unwrap();
        assert_eq!(result, "Aperture Priority");

        // Test MultiSelector conversion
        let highlight = TagValue::I32(1);
        let result = nikon_multi_selector_conv(&highlight).unwrap();
        assert_eq!(result, "Highlight Active Focus Point");

        // Test FlashCommander conversion
        let ttl = TagValue::I32(1);
        let result = nikon_flash_commander_conv(&ttl).unwrap();
        assert_eq!(result, "TTL");
    }

    #[test]
    fn test_advanced_feature_printconv() {
        // Test PixelShift conversion
        let pixel_shift_on = TagValue::I32(1);
        let result = nikon_pixel_shift_conv(&pixel_shift_on).unwrap();
        assert_eq!(result, "On");

        // Test FlickerReduction conversion
        let flicker_on = TagValue::I32(1);
        let result = nikon_flicker_reduction_conv(&flicker_on).unwrap();
        assert_eq!(result, "On");

        // Test PreCapture conversion
        let pre_capture_on = TagValue::I32(1);
        let result = nikon_pre_capture_conv(&pre_capture_on).unwrap();
        assert_eq!(result, "On");

        // Test GroupAreaIllumination conversion
        let illumination_on = TagValue::I32(1);
        let result = nikon_group_area_illumination_conv(&illumination_on).unwrap();
        assert_eq!(result, "On");
    }

    // Phase 3 PrintConv function tests
    #[test]
    fn test_nikon_iso_conv() {
        // Test ISO numeric mapping from isoAutoHiLimitZ7
        let iso_100 = TagValue::I32(2);
        let result = nikon_iso_conv(&iso_100).unwrap();
        assert_eq!(result, "ISO 100");

        let iso_3200 = TagValue::U16(17);
        let result = nikon_iso_conv(&iso_3200).unwrap();
        assert_eq!(result, "ISO 3200");

        // Test high ISO values
        let iso_hi = TagValue::I32(27);
        let result = nikon_iso_conv(&iso_hi).unwrap();
        assert_eq!(result, "ISO Hi 0.3");

        // Test string ISO values (ExifTool: direct string storage)
        let iso_string = TagValue::String("ISO 6400".to_string());
        let result = nikon_iso_conv(&iso_string).unwrap();
        assert_eq!(result, "ISO 6400");
    }

    #[test]
    fn test_nikon_focus_mode_conv() {
        // Test focusModeZ7 mapping
        let manual = TagValue::I32(0);
        let result = nikon_focus_mode_conv(&manual).unwrap();
        assert_eq!(result, "Manual");

        let af_s = TagValue::U16(1);
        let result = nikon_focus_mode_conv(&af_s).unwrap();
        assert_eq!(result, "AF-S");

        let af_c = TagValue::I32(2);
        let result = nikon_focus_mode_conv(&af_c).unwrap();
        assert_eq!(result, "AF-C");

        let af_f = TagValue::I32(4); // ExifTool comment: full frame
        let result = nikon_focus_mode_conv(&af_f).unwrap();
        assert_eq!(result, "AF-F");

        // Test string focus mode
        let string_mode = TagValue::String("AF-A".to_string());
        let result = nikon_focus_mode_conv(&string_mode).unwrap();
        assert_eq!(result, "AF-A");
    }

    #[test]
    fn test_nikon_af_area_mode_conv() {
        // Test aFAreaModePD mapping (Phase Detect)
        let single_area = TagValue::I32(0);
        let result = nikon_af_area_mode_conv(&single_area).unwrap();
        assert_eq!(result, "Single Area");

        let dynamic_area = TagValue::U16(1);
        let result = nikon_af_area_mode_conv(&dynamic_area).unwrap();
        assert_eq!(result, "Dynamic Area");

        let auto_area = TagValue::I32(8);
        let result = nikon_af_area_mode_conv(&auto_area).unwrap();
        assert_eq!(result, "Auto-area");

        // Test mirrorless camera values
        let pinpoint = TagValue::I32(192); // ExifTool: NC
        let result = nikon_af_area_mode_conv(&pinpoint).unwrap();
        assert_eq!(result, "Pinpoint");

        let z7_auto = TagValue::I32(197); // ExifTool: Z7
        let result = nikon_af_area_mode_conv(&z7_auto).unwrap();
        assert_eq!(result, "Auto");

        // Test Z7 specific modes
        let auto_people = TagValue::I32(198);
        let result = nikon_af_area_mode_conv(&auto_people).unwrap();
        assert_eq!(result, "Auto (People)");

        let auto_animal = TagValue::I32(199);
        let result = nikon_af_area_mode_conv(&auto_animal).unwrap();
        assert_eq!(result, "Auto (Animal)");
    }

    #[test]
    fn test_nikon_color_mode_conv() {
        // Test string color mode (ExifTool: stored as strings)
        let color_string = TagValue::String("MODE1".to_string());
        let result = nikon_color_mode_conv(&color_string).unwrap();
        assert_eq!(result, "MODE1");

        // Test numeric color mode for legacy cameras
        let color_numeric = TagValue::I32(1);
        let result = nikon_color_mode_conv(&color_numeric).unwrap();
        assert_eq!(result, "Color");

        let monochrome = TagValue::I32(2);
        let result = nikon_color_mode_conv(&monochrome).unwrap();
        assert_eq!(result, "Monochrome");
    }

    #[test]
    fn test_nikon_sharpness_conv() {
        // Test sharpness values (typically -3 to +3)
        let no_sharpening = TagValue::I32(0);
        let result = nikon_sharpness_conv(&no_sharpening).unwrap();
        assert_eq!(result, "No Sharpening");

        let positive_sharp = TagValue::I32(2);
        let result = nikon_sharpness_conv(&positive_sharp).unwrap();
        assert_eq!(result, "+2");

        let negative_sharp = TagValue::I32(-1);
        let result = nikon_sharpness_conv(&negative_sharp).unwrap();
        assert_eq!(result, "-1");

        // Test string sharpness
        let string_sharp = TagValue::String("Normal".to_string());
        let result = nikon_sharpness_conv(&string_sharp).unwrap();
        assert_eq!(result, "Normal");
    }

    #[test]
    fn test_nikon_active_d_lighting_conv() {
        // Test Active D-Lighting strength levels
        let off = TagValue::I32(0);
        let result = nikon_active_d_lighting_conv(&off).unwrap();
        assert_eq!(result, "Off");

        let low = TagValue::U16(1);
        let result = nikon_active_d_lighting_conv(&low).unwrap();
        assert_eq!(result, "Low");

        let normal = TagValue::I32(3);
        let result = nikon_active_d_lighting_conv(&normal).unwrap();
        assert_eq!(result, "Normal");

        let high = TagValue::I32(5);
        let result = nikon_active_d_lighting_conv(&high).unwrap();
        assert_eq!(result, "High");

        let auto = TagValue::I32(65535);
        let result = nikon_active_d_lighting_conv(&auto).unwrap();
        assert_eq!(result, "Auto");
    }

    #[test]
    fn test_nikon_nef_compression_conv() {
        // Test NEF compression modes
        let lossy_1 = TagValue::I32(1);
        let result = nikon_nef_compression_conv(&lossy_1).unwrap();
        assert_eq!(result, "Lossy (type 1)");

        let uncompressed = TagValue::I32(2);
        let result = nikon_nef_compression_conv(&uncompressed).unwrap();
        assert_eq!(result, "Uncompressed");

        let lossless = TagValue::I32(3);
        let result = nikon_nef_compression_conv(&lossless).unwrap();
        assert_eq!(result, "Lossless");

        let lossy_2 = TagValue::I32(4);
        let result = nikon_nef_compression_conv(&lossy_2).unwrap();
        assert_eq!(result, "Lossy (type 2)");
    }

    #[test]
    fn test_nikon_vr_conv() {
        // Test VR on/off
        let vr_off = TagValue::I32(0);
        let result = nikon_vr_conv(&vr_off).unwrap();
        assert_eq!(result, "Off");

        let vr_on = TagValue::U16(1);
        let result = nikon_vr_conv(&vr_on).unwrap();
        assert_eq!(result, "On");
    }

    #[test]
    fn test_nikon_vr_mode_conv() {
        // Test VR mode settings
        let normal = TagValue::I32(0);
        let result = nikon_vr_mode_conv(&normal).unwrap();
        assert_eq!(result, "Normal");

        let active = TagValue::I32(1);
        let result = nikon_vr_mode_conv(&active).unwrap();
        assert_eq!(result, "Active");

        let sport = TagValue::I32(2);
        let result = nikon_vr_mode_conv(&sport).unwrap();
        assert_eq!(result, "Sport");
    }

    #[test]
    fn test_unknown_values_fallback() {
        // Test that unknown values fall back gracefully
        let unknown_iso = TagValue::I32(999);
        let result = nikon_iso_conv(&unknown_iso).unwrap();
        assert_eq!(result, "Unknown");

        let unknown_focus = TagValue::I32(999);
        let result = nikon_focus_mode_conv(&unknown_focus).unwrap();
        assert_eq!(result, "Unknown");

        let unknown_af_area = TagValue::I32(999);
        let result = nikon_af_area_mode_conv(&unknown_af_area).unwrap();
        assert_eq!(result, "Unknown");
    }
}
