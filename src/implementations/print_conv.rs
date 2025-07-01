//! PrintConv implementations for exif-oxide
//!
//! This module contains manual implementations of ExifTool's PrintConv functions.
//! Each function converts a raw tag value to a human-readable string.
//!
//! All implementations are direct translations from ExifTool source code,
//! with comments pointing back to the original ExifTool references.

use crate::types::TagValue;

/// EXIF Orientation PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:2719-2728
pub fn orientation_print_conv(val: &TagValue) -> String {
    match val.as_u16() {
        Some(1) => "Horizontal (normal)",
        Some(2) => "Mirror horizontal",
        Some(3) => "Rotate 180",
        Some(4) => "Mirror vertical",
        Some(5) => "Mirror horizontal and rotate 270 CW",
        Some(6) => "Rotate 90 CW",
        Some(7) => "Mirror horizontal and rotate 90 CW",
        Some(8) => "Rotate 270 CW",
        _ => return format!("Unknown ({val})"),
    }
    .to_string()
}

/// EXIF ResolutionUnit PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:2778-2782
pub fn resolutionunit_print_conv(val: &TagValue) -> String {
    match val.as_u16() {
        Some(1) => "None",
        Some(2) => "inches",
        Some(3) => "cm",
        _ => return format!("Unknown ({val})"),
    }
    .to_string()
}

/// EXIF YCbCrPositioning PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:2802-2805
pub fn ycbcrpositioning_print_conv(val: &TagValue) -> String {
    match val.as_u16() {
        Some(1) => "Centered",
        Some(2) => "Co-sited",
        _ => return format!("Unknown ({val})"),
    }
    .to_string()
}

/// GPS AltitudeRef PrintConv
/// ExifTool: lib/Image/ExifTool/GPS.pm GPSAltitudeRef tag definition
pub fn gpsaltituderef_print_conv(val: &TagValue) -> String {
    match val.as_u8() {
        Some(0) => "Above Sea Level",
        Some(1) => "Below Sea Level",
        _ => return format!("Unknown ({val})"),
    }
    .to_string()
}

/// GPS LatitudeRef PrintConv
/// ExifTool: lib/Image/ExifTool/GPS.pm GPSLatitudeRef tag definition
pub fn gpslatituderef_print_conv(val: &TagValue) -> String {
    match val.as_string() {
        Some("N") => "North",
        Some("S") => "South",
        _ => return format!("Unknown ({val})"),
    }
    .to_string()
}

/// GPS LongitudeRef PrintConv  
/// ExifTool: lib/Image/ExifTool/GPS.pm GPSLongitudeRef tag definition
pub fn gpslongituderef_print_conv(val: &TagValue) -> String {
    match val.as_string() {
        Some("E") => "East",
        Some("W") => "West",
        _ => return format!("Unknown ({val})"),
    }
    .to_string()
}

/// EXIF Flash PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:164-197, tag definition lines 2379-2386
/// NOTE: This is NOT a bitmask conversion - ExifTool uses direct hash lookup for specific combined values
pub fn flash_print_conv(val: &TagValue) -> String {
    match val.as_u16() {
        Some(0x00) => "No Flash",
        Some(0x01) => "Fired",
        Some(0x05) => "Fired, Return not detected",
        Some(0x07) => "Fired, Return detected",
        Some(0x08) => "On, Did not fire",
        Some(0x09) => "On, Fired",
        Some(0x0d) => "On, Return not detected",
        Some(0x0f) => "On, Return detected",
        Some(0x10) => "Off, Did not fire",
        Some(0x14) => "Off, Did not fire, Return not detected",
        Some(0x18) => "Auto, Did not fire",
        Some(0x19) => "Auto, Fired",
        Some(0x1d) => "Auto, Fired, Return not detected",
        Some(0x1f) => "Auto, Fired, Return detected",
        Some(0x20) => "No flash function",
        Some(0x30) => "Off, No flash function",
        Some(0x41) => "Fired, Red-eye reduction",
        Some(0x45) => "Fired, Red-eye reduction, Return not detected",
        Some(0x47) => "Fired, Red-eye reduction, Return detected",
        Some(0x49) => "On, Red-eye reduction",
        Some(0x4d) => "On, Red-eye reduction, Return not detected",
        Some(0x4f) => "On, Red-eye reduction, Return detected",
        Some(0x50) => "Off, Red-eye reduction",
        Some(0x58) => "Auto, Did not fire, Red-eye reduction",
        Some(0x59) => "Auto, Fired, Red-eye reduction",
        Some(0x5d) => "Auto, Fired, Red-eye reduction, Return not detected",
        Some(0x5f) => "Auto, Fired, Red-eye reduction, Return detected",
        // Unknown values shown in parentheses (ExifTool format)
        // TODO: Standardize hex formatting - some functions use decimal, others need hex (0x1a vs 26)
        _ => {
            if let Some(num) = val.as_u16() {
                return format!("Unknown ({num})");
            } else {
                return format!("Unknown ({val})");
            }
        }
    }
    .to_string()
}

/// EXIF ColorSpace PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:2620-2638
pub fn colorspace_print_conv(val: &TagValue) -> String {
    match val.as_u16() {
        Some(1) => "sRGB",
        Some(2) => "Adobe RGB",
        Some(0xffff) => "Uncalibrated",
        // Sony-specific non-standard values (ref JD)
        Some(0xfffe) => "ICC Profile",
        Some(0xfffd) => "Wide Gamut RGB",
        // Unknown values shown in parentheses (ExifTool format)
        _ => {
            if let Some(num) = val.as_u16() {
                return format!("Unknown ({num})");
            } else {
                return format!("Unknown ({val})");
            }
        }
    }
    .to_string()
}

/// EXIF WhiteBalance PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:2809-2821
// TODO: Add manufacturer-specific handling - Canon uses "Evaluative" vs "Multi-segment" for MeteringMode
pub fn whitebalance_print_conv(val: &TagValue) -> String {
    match val.as_u16() {
        Some(0) => "Auto",
        Some(1) => "Manual",
        _ => return format!("Unknown ({val})"),
    }
    .to_string()
}

/// EXIF MeteringMode PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:2357-2371
// TODO: Add manufacturer-specific handling - Canon uses "Evaluative" instead of "Multi-segment"
pub fn meteringmode_print_conv(val: &TagValue) -> String {
    match val.as_u16() {
        Some(0) => "Unknown",
        Some(1) => "Average",
        Some(2) => "Center-weighted average",
        Some(3) => "Spot",
        Some(4) => "Multi-spot",
        Some(5) => "Multi-segment",
        Some(6) => "Partial",
        Some(255) => "Other",
        _ => return format!("Unknown ({val})"),
    }
    .to_string()
}

/// EXIF ExposureProgram PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:2082-2097
/// NOTE: Value 9 is not standard EXIF but used by some Canon models
pub fn exposureprogram_print_conv(val: &TagValue) -> String {
    match val.as_u16() {
        Some(0) => "Not Defined",
        Some(1) => "Manual",
        Some(2) => "Program AE",
        Some(3) => "Aperture-priority AE",
        Some(4) => "Shutter speed priority AE",
        Some(5) => "Creative (Slow speed)",
        Some(6) => "Action (High speed)",
        Some(7) => "Portrait",
        Some(8) => "Landscape",
        Some(9) => "Bulb", // Canon-specific non-standard value
        _ => return format!("Unknown ({val})"),
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orientation_print_conv() {
        assert_eq!(
            orientation_print_conv(&TagValue::U16(1)),
            "Horizontal (normal)"
        );
        assert_eq!(orientation_print_conv(&TagValue::U16(8)), "Rotate 270 CW");
        assert_eq!(orientation_print_conv(&TagValue::U16(99)), "Unknown (99)");
    }

    #[test]
    fn test_resolutionunit_print_conv() {
        assert_eq!(resolutionunit_print_conv(&TagValue::U16(1)), "None");
        assert_eq!(resolutionunit_print_conv(&TagValue::U16(2)), "inches");
        assert_eq!(resolutionunit_print_conv(&TagValue::U16(3)), "cm");
        assert_eq!(
            resolutionunit_print_conv(&TagValue::U16(99)),
            "Unknown (99)"
        );
    }

    #[test]
    fn test_ycbcrpositioning_print_conv() {
        assert_eq!(ycbcrpositioning_print_conv(&TagValue::U16(1)), "Centered");
        assert_eq!(ycbcrpositioning_print_conv(&TagValue::U16(2)), "Co-sited");
        assert_eq!(
            ycbcrpositioning_print_conv(&TagValue::U16(99)),
            "Unknown (99)"
        );
    }

    #[test]
    fn test_flash_print_conv() {
        // Test standard values
        assert_eq!(flash_print_conv(&TagValue::U16(0x00)), "No Flash");
        assert_eq!(flash_print_conv(&TagValue::U16(0x01)), "Fired");
        assert_eq!(flash_print_conv(&TagValue::U16(0x19)), "Auto, Fired");
        assert_eq!(flash_print_conv(&TagValue::U16(0x20)), "No flash function");

        // Test red-eye reduction values
        assert_eq!(
            flash_print_conv(&TagValue::U16(0x41)),
            "Fired, Red-eye reduction"
        );
        assert_eq!(
            flash_print_conv(&TagValue::U16(0x59)),
            "Auto, Fired, Red-eye reduction"
        );

        // Test unknown value
        assert_eq!(flash_print_conv(&TagValue::U16(0x99)), "Unknown (153)");
    }

    #[test]
    fn test_colorspace_print_conv() {
        assert_eq!(colorspace_print_conv(&TagValue::U16(1)), "sRGB");
        assert_eq!(colorspace_print_conv(&TagValue::U16(2)), "Adobe RGB");
        assert_eq!(
            colorspace_print_conv(&TagValue::U16(0xffff)),
            "Uncalibrated"
        );

        // Test Sony-specific values
        assert_eq!(colorspace_print_conv(&TagValue::U16(0xfffe)), "ICC Profile");
        assert_eq!(
            colorspace_print_conv(&TagValue::U16(0xfffd)),
            "Wide Gamut RGB"
        );

        // Test unknown value
        assert_eq!(colorspace_print_conv(&TagValue::U16(0x99)), "Unknown (153)");
    }

    #[test]
    fn test_whitebalance_print_conv() {
        assert_eq!(whitebalance_print_conv(&TagValue::U16(0)), "Auto");
        assert_eq!(whitebalance_print_conv(&TagValue::U16(1)), "Manual");
        assert_eq!(whitebalance_print_conv(&TagValue::U16(99)), "Unknown (99)");
    }

    #[test]
    fn test_meteringmode_print_conv() {
        assert_eq!(meteringmode_print_conv(&TagValue::U16(0)), "Unknown");
        assert_eq!(meteringmode_print_conv(&TagValue::U16(1)), "Average");
        assert_eq!(
            meteringmode_print_conv(&TagValue::U16(2)),
            "Center-weighted average"
        );
        assert_eq!(meteringmode_print_conv(&TagValue::U16(3)), "Spot");
        assert_eq!(meteringmode_print_conv(&TagValue::U16(5)), "Multi-segment");
        assert_eq!(meteringmode_print_conv(&TagValue::U16(255)), "Other");
        assert_eq!(meteringmode_print_conv(&TagValue::U16(99)), "Unknown (99)");
    }

    #[test]
    fn test_exposureprogram_print_conv() {
        assert_eq!(exposureprogram_print_conv(&TagValue::U16(0)), "Not Defined");
        assert_eq!(exposureprogram_print_conv(&TagValue::U16(1)), "Manual");
        assert_eq!(exposureprogram_print_conv(&TagValue::U16(2)), "Program AE");
        assert_eq!(
            exposureprogram_print_conv(&TagValue::U16(3)),
            "Aperture-priority AE"
        );
        assert_eq!(exposureprogram_print_conv(&TagValue::U16(7)), "Portrait");

        // Test Canon-specific non-standard value
        assert_eq!(exposureprogram_print_conv(&TagValue::U16(9)), "Bulb");

        assert_eq!(
            exposureprogram_print_conv(&TagValue::U16(99)),
            "Unknown (99)"
        );
    }
}
