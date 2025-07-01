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
}
