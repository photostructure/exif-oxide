//! Autofocus and Vibration Reduction PrintConv functions for Nikon tags
//!
//! **Trust ExifTool**: This code translates ExifTool's AF/VR PrintConv functions verbatim.
//!
//! ExifTool Reference: lib/Image/ExifTool/Nikon.pm AF/VR tag PrintConv definitions

use std::collections::HashMap;

/// PrintConv function for Nikon AFAreaMode tag
/// ExifTool: Nikon.pm lines 876-906 %aFAreaModePD (Phase Detect)
pub fn nikon_af_area_mode_conv(value: &crate::types::TagValue) -> Result<String, String> {
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
pub fn nikon_vr_conv(value: &crate::types::TagValue) -> Result<String, String> {
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
pub fn nikon_vr_mode_conv(value: &crate::types::TagValue) -> Result<String, String> {
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

/// PrintConv function for Nikon DynamicAFAreaSize tag
/// ExifTool: Nikon.pm DynamicAFAreaSize PrintConv
pub fn nikon_dynamic_af_area_conv(value: &crate::types::TagValue) -> Result<String, String> {
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

/// PrintConv function for Nikon ImageStabilization tag
/// ExifTool: Nikon.pm - VR (Vibration Reduction) settings
pub fn nikon_image_stabilization_conv(value: &crate::types::TagValue) -> Result<String, String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TagValue;

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
    fn test_unknown_af_values_fallback() {
        // Test that unknown values fall back gracefully
        let unknown_af_area = TagValue::I32(999);
        let result = nikon_af_area_mode_conv(&unknown_af_area).unwrap();
        assert_eq!(result, "Unknown");
    }

    #[test]
    fn test_dynamic_af_area_conv() {
        // Test DynamicAFAreaSize conversion
        let points_51 = TagValue::I32(2);
        let result = nikon_dynamic_af_area_conv(&points_51).unwrap();
        assert_eq!(result, "51 Points");

        let points_153 = TagValue::I32(4);
        let result = nikon_dynamic_af_area_conv(&points_153).unwrap();
        assert_eq!(result, "153 Points");
    }
}
