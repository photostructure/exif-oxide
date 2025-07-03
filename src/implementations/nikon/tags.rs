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
        // ... additional Z9-specific tags will be added in Phase 3
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

// Skeleton PrintConv functions for Phase 1 - will be implemented in Phase 3
fn nikon_iso_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // TODO: Implement Nikon ISO conversion in Phase 3
    Ok(format!("ISO {value}"))
}

fn nikon_color_mode_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // TODO: Implement Nikon ColorMode conversion in Phase 3
    Ok(format!("ColorMode {value}"))
}

fn nikon_sharpness_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // TODO: Implement Nikon Sharpness conversion in Phase 3
    Ok(format!("Sharpness {value}"))
}

fn nikon_focus_mode_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // TODO: Implement Nikon FocusMode conversion in Phase 3
    Ok(format!("FocusMode {value}"))
}

fn nikon_flash_setting_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // TODO: Implement Nikon FlashSetting conversion in Phase 3
    Ok(format!("FlashSetting {value}"))
}

fn nikon_color_space_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // TODO: Implement Nikon ColorSpace conversion in Phase 3
    Ok(format!("ColorSpace {value}"))
}

fn nikon_active_d_lighting_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // TODO: Implement Nikon ActiveDLighting conversion in Phase 3
    Ok(format!("ActiveDLighting {value}"))
}

fn nikon_flash_mode_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // TODO: Implement Nikon FlashMode conversion in Phase 3
    Ok(format!("FlashMode {value}"))
}

fn nikon_shooting_mode_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // TODO: Implement Nikon ShootingMode conversion in Phase 3
    Ok(format!("ShootingMode {value}"))
}

fn nikon_scene_mode_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // TODO: Implement Nikon SceneMode conversion in Phase 3
    Ok(format!("SceneMode {value}"))
}

fn nikon_nef_compression_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // TODO: Implement Nikon NEFCompression conversion in Phase 3
    Ok(format!("NEFCompression {value}"))
}

fn nikon_image_optimization_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // TODO: Implement Nikon ImageOptimization conversion in Phase 3
    Ok(format!("ImageOptimization {value}"))
}

fn nikon_image_stabilization_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // TODO: Implement Nikon ImageStabilization conversion in Phase 3
    Ok(format!("ImageStabilization {value}"))
}

fn nikon_high_iso_nr_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // TODO: Implement Nikon HighISONoiseReduction conversion in Phase 3
    Ok(format!("HighISONoiseReduction {value}"))
}

fn nikon_toning_effect_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // TODO: Implement Nikon ToningEffect conversion in Phase 3
    Ok(format!("ToningEffect {value}"))
}

fn nikon_af_area_mode_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // TODO: Implement Nikon AFAreaMode conversion in Phase 3
    Ok(format!("AFAreaMode {value}"))
}

fn nikon_vr_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // TODO: Implement Nikon VibrationReduction conversion in Phase 3
    Ok(format!("VibrationReduction {value}"))
}

fn nikon_vr_mode_conv(value: &crate::types::TagValue) -> Result<String, String> {
    // TODO: Implement Nikon VRMode conversion in Phase 3
    Ok(format!("VRMode {value}"))
}

/// Get appropriate tag table for camera model
/// ExifTool: Model-specific tag table selection logic
pub fn select_nikon_tag_table(model: &str) -> &'static NikonTagTable {
    // Model-specific table selection
    // ExifTool: Condition evaluation for model-specific tables
    if model.contains("Z 9") {
        debug!("Selected Nikon Z9 ShotInfo table for model: {}", model);
        &NIKON_Z9_SHOT_INFO
    } else {
        // Default to main table for all other Nikon cameras
        debug!("Selected Nikon Main table for model: {}", model);
        &NIKON_MAIN_TAGS
    }
}

/// Look up Nikon tag name by ID
/// ExifTool: Tag table lookup functionality
pub fn get_nikon_tag_name(tag_id: u16, model: &str) -> Option<&'static str> {
    let table = select_nikon_tag_table(model);

    table
        .tags
        .iter()
        .find(|(id, _, _)| *id == tag_id)
        .map(|(_, name, _)| *name)
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
        let table = select_nikon_tag_table("NIKON D850");
        assert_eq!(table.name, "Nikon::Main");
    }

    #[test]
    fn test_tag_name_lookup() {
        let name = get_nikon_tag_name(0x0004, "NIKON D850");
        assert_eq!(name, Some("Quality"));
    }

    #[test]
    fn test_encryption_key_tags_present() {
        // Verify encryption key tags are included
        let serial_tag = get_nikon_tag_name(0x001D, "NIKON D850");
        assert_eq!(serial_tag, Some("SerialNumber"));

        let count_tag = get_nikon_tag_name(0x00A7, "NIKON D850");
        assert_eq!(count_tag, Some("ShutterCount"));
    }
}
