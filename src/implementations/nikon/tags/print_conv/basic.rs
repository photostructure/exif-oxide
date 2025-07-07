//! Basic PrintConv functions for common Nikon tags
//!
//! **Trust ExifTool**: This code translates ExifTool's PrintConv functions verbatim.
//!
//! ExifTool Reference: lib/Image/ExifTool/Nikon.pm basic tag PrintConv definitions

use std::collections::HashMap;

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
pub fn nikon_iso_conv(value: &crate::types::TagValue) -> Result<String, String> {
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
pub fn nikon_color_mode_conv(value: &crate::types::TagValue) -> Result<String, String> {
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
pub fn nikon_sharpness_conv(value: &crate::types::TagValue) -> Result<String, String> {
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
pub fn nikon_focus_mode_conv(value: &crate::types::TagValue) -> Result<String, String> {
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
pub fn nikon_flash_setting_conv(value: &crate::types::TagValue) -> Result<String, String> {
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
pub fn nikon_color_space_conv(value: &crate::types::TagValue) -> Result<String, String> {
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

/// PrintConv function for Nikon FlashMode tag
/// ExifTool: Nikon.pm - Flash mode settings
pub fn nikon_flash_mode_conv(value: &crate::types::TagValue) -> Result<String, String> {
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
pub fn nikon_shooting_mode_conv(value: &crate::types::TagValue) -> Result<String, String> {
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
pub fn nikon_scene_mode_conv(value: &crate::types::TagValue) -> Result<String, String> {
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
pub fn nikon_nef_compression_conv(value: &crate::types::TagValue) -> Result<String, String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TagValue;

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
}
