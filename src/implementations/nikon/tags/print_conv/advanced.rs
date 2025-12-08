//! Advanced feature PrintConv functions for Nikon tags
//!
//! **Trust ExifTool**: This code translates ExifTool's advanced feature PrintConv functions verbatim.
//!
//! ExifTool Reference: lib/Image/ExifTool/Nikon.pm advanced feature tag PrintConv definitions

use std::collections::HashMap;

/// PrintConv function for Nikon ActiveDLighting tag
/// ExifTool: Nikon.pm - ActiveDLighting strength levels
pub fn nikon_active_d_lighting_conv(value: &crate::types::TagValue) -> Result<String, String> {
    if let crate::types::TagValue::String(s) = value {
        return Ok(s.clone());
    }

    let Some(val) = value.as_i32() else {
        return Ok(format!("Unknown ({value})"));
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

/// PrintConv function for Nikon ImageOptimization tag
/// ExifTool: Nikon.pm - Image optimization settings
pub fn nikon_image_optimization_conv(value: &crate::types::TagValue) -> Result<String, String> {
    if let crate::types::TagValue::String(s) = value {
        return Ok(s.clone());
    }

    let Some(val) = value.as_i32() else {
        return Ok(format!("Unknown ({value})"));
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

/// PrintConv function for Nikon HighISONoiseReduction tag
/// ExifTool: Nikon.pm - High ISO noise reduction settings
pub fn nikon_high_iso_nr_conv(value: &crate::types::TagValue) -> Result<String, String> {
    if let crate::types::TagValue::String(s) = value {
        return Ok(s.clone());
    }

    let Some(val) = value.as_i32() else {
        return Ok(format!("Unknown ({value})"));
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
pub fn nikon_toning_effect_conv(value: &crate::types::TagValue) -> Result<String, String> {
    if let crate::types::TagValue::String(s) = value {
        return Ok(s.clone());
    }

    let Some(val) = value.as_i32() else {
        return Ok(format!("Unknown ({value})"));
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

/// PrintConv function for Nikon SubjectDetection tag (Z-series)
/// ExifTool: Nikon.pm SubjectDetection PrintConv
pub fn nikon_subject_detection_conv(value: &crate::types::TagValue) -> Result<String, String> {
    if let crate::types::TagValue::String(s) = value {
        return Ok(s.clone());
    }

    let Some(val) = value.as_i32() else {
        return Ok(format!("Unknown ({value})"));
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

/// PrintConv function for Nikon HDR tag
/// ExifTool: Nikon.pm HDR PrintConv
pub fn nikon_hdr_conv(value: &crate::types::TagValue) -> Result<String, String> {
    let Some(val) = value.as_i32() else {
        return Ok(format!("Unknown ({value})"));
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
pub fn nikon_pixel_shift_conv(value: &crate::types::TagValue) -> Result<String, String> {
    let Some(val) = value.as_i32() else {
        return Ok(format!("Unknown ({value})"));
    };

    let pixel_shift_map: HashMap<i32, &str> = [(0, "Off"), (1, "On")].iter().cloned().collect();

    Ok(pixel_shift_map.get(&val).unwrap_or(&"Unknown").to_string())
}

/// PrintConv function for Nikon ExposureMode tag
/// ExifTool: Nikon.pm ExposureMode PrintConv
pub fn nikon_exposure_mode_conv(value: &crate::types::TagValue) -> Result<String, String> {
    let Some(val) = value.as_i32() else {
        return Ok(format!("Unknown ({value})"));
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
pub fn nikon_flicker_reduction_conv(value: &crate::types::TagValue) -> Result<String, String> {
    let Some(val) = value.as_i32() else {
        return Ok(format!("Unknown ({value})"));
    };

    let flicker_map: HashMap<i32, &str> = [(0, "Off"), (1, "On")].iter().cloned().collect();

    Ok(flicker_map.get(&val).unwrap_or(&"Unknown").to_string())
}

/// PrintConv function for Nikon MultiSelector tag
/// ExifTool: Nikon.pm MultiSelector PrintConv
pub fn nikon_multi_selector_conv(value: &crate::types::TagValue) -> Result<String, String> {
    let Some(val) = value.as_i32() else {
        return Ok(format!("Unknown ({value})"));
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
pub fn nikon_flash_commander_conv(value: &crate::types::TagValue) -> Result<String, String> {
    let Some(val) = value.as_i32() else {
        return Ok(format!("Unknown ({value})"));
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
pub fn nikon_pre_capture_conv(value: &crate::types::TagValue) -> Result<String, String> {
    let Some(val) = value.as_i32() else {
        return Ok(format!("Unknown ({value})"));
    };

    let pre_capture_map: HashMap<i32, &str> = [(0, "Off"), (1, "On")].iter().cloned().collect();

    Ok(pre_capture_map.get(&val).unwrap_or(&"Unknown").to_string())
}

/// PrintConv function for Nikon GroupAreaIllumination tag
/// ExifTool: Nikon.pm GroupAreaIllumination PrintConv
pub fn nikon_group_area_illumination_conv(
    value: &crate::types::TagValue,
) -> Result<String, String> {
    let Some(val) = value.as_i32() else {
        return Ok(format!("Unknown ({value})"));
    };

    let illumination_map: HashMap<i32, &str> = [(0, "Off"), (1, "On")].iter().cloned().collect();

    Ok(illumination_map.get(&val).unwrap_or(&"Unknown").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TagValue;

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

    #[test]
    fn test_unknown_values_fallback() {
        // Test that unknown values fall back gracefully
        let unknown_hdr = TagValue::I32(999);
        let result = nikon_hdr_conv(&unknown_hdr).unwrap();
        assert_eq!(result, "Unknown");
    }
}
