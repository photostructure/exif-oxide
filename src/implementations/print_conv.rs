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

/// GPS Altitude PrintConv
/// ExifTool: lib/Image/ExifTool/GPS.pm:124 - '$val =~ /^(inf|undef)$/ ? $val : "$val m"'
pub fn gpsaltitude_print_conv(val: &TagValue) -> String {
    match val.as_f64() {
        Some(v) if v.is_infinite() => "inf".to_string(),
        Some(v) if v.is_nan() => "undef".to_string(),
        Some(v) => format!("{v:.1} m"), // Round to 0.1m - GPS accuracy limit
        None => format!("Unknown ({val})"),
    }
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

/// FNumber PrintConv - formats f-stop values
/// ExifTool: lib/Image/ExifTool/Exif.pm PrintFNumber function (lines 5607-5615)
/// Uses 2 decimal places for values < 1.0, 1 decimal place for values >= 1.0
pub fn fnumber_print_conv(val: &TagValue) -> String {
    match val.as_f64() {
        Some(f_number) => {
            if f_number > 0.0 {
                // ExifTool logic: 2 decimal places for < 1.0, 1 decimal place for >= 1.0
                if f_number < 1.0 {
                    format!("{f_number:.2}")
                } else {
                    format!("{f_number:.1}")
                }
            } else {
                format!("Unknown ({val})")
            }
        }
        None => {
            // Handle rational format directly if ValueConv wasn't applied
            if let TagValue::Rational(num, denom) = val {
                if *denom != 0 {
                    let f_number = *num as f64 / *denom as f64;
                    if f_number > 0.0 {
                        if f_number < 1.0 {
                            return format!("{f_number:.2}");
                        } else {
                            return format!("{f_number:.1}");
                        }
                    }
                }
            }
            format!("Unknown ({val})")
        }
    }
}

/// ExposureTime PrintConv - formats shutter speed
/// ExifTool: lib/Image/ExifTool/Exif.pm ExposureTime PrintConv  
/// Converts decimal seconds to fractional notation (e.g., 0.0005 -> "1/2000")
pub fn exposuretime_print_conv(val: &TagValue) -> String {
    match val.as_f64() {
        Some(exposure_time) => {
            if exposure_time >= 1.0 {
                // For exposures >= 1 second, show as decimal
                format!("{exposure_time}")
            } else if exposure_time > 0.0 {
                // For fractional exposures, show as 1/x
                let denominator = (1.0 / exposure_time).round() as u32;
                format!("1/{denominator}")
            } else {
                "0".to_string()
            }
        }
        None => {
            // Handle rational format directly if ValueConv wasn't applied
            if let TagValue::Rational(num, denom) = val {
                if *denom != 0 && *num != 0 {
                    if *num >= *denom {
                        let exposure_time = *num as f64 / *denom as f64;
                        return format!("{exposure_time}");
                    } else {
                        return format!("{num}/{denom}");
                    }
                }
            }
            format!("Unknown ({val})")
        }
    }
}

/// FocalLength PrintConv - formats focal length with "mm" unit
/// ExifTool: lib/Image/ExifTool/Exif.pm lines 2387-2393
/// Note: We normalize ExifTool's inconsistent formatting to show integers without decimals
pub fn focallength_print_conv(val: &TagValue) -> String {
    match val.as_f64() {
        Some(focal_length) => {
            // Round to 1 decimal place like ExifTool, but remove .0 for integers
            let rounded = (focal_length * 10.0).round() / 10.0;
            if (rounded.fract()).abs() < 0.001 {
                format!("{} mm", rounded as i32)
            } else {
                format!("{rounded:.1} mm")
            }
        }
        None => {
            // Handle rational format directly if ValueConv wasn't applied
            if let TagValue::Rational(num, denom) = val {
                if *denom != 0 {
                    let focal_length = *num as f64 / *denom as f64;
                    let rounded = (focal_length * 10.0).round() / 10.0;
                    if (rounded.fract()).abs() < 0.001 {
                        return format!("{} mm", rounded as i32);
                    } else {
                        return format!("{rounded:.1} mm");
                    }
                }
            }
            format!("Unknown ({val})")
        }
    }
}

/// FocalLengthIn35mmFormat PrintConv - formats 35mm equivalent focal length
/// ExifTool: lib/Image/ExifTool/Exif.pm lines 2827-2834
/// PrintConv => '"$val mm"',
pub fn focallength_in_35mm_format_print_conv(val: &TagValue) -> String {
    match val.as_u16() {
        Some(focal_length) => {
            // Format as integer with no decimal places to match ExifTool
            format!("{focal_length} mm")
        }
        None => format!("Unknown ({val})"),
    }
}

/// Composite GPS Altitude PrintConv
/// ExifTool: lib/Image/ExifTool/GPS.pm:423-431
/// Formats GPS altitude with "Above/Below Sea Level" based on sign
pub fn composite_gps_gpsaltitude_print_conv(val: &TagValue) -> String {
    // Handle numeric value
    if let Some(v) = val.as_f64() {
        if v.is_infinite() {
            return "inf".to_string();
        }
        if v.is_nan() {
            return "undef".to_string();
        }

        // Round to 1 decimal place: int($val * 10) / 10
        let rounded = (v * 10.0).round() / 10.0;

        // Check if negative (below sea level) and make positive for display
        if rounded < 0.0 {
            return format!("{:.1} m Below Sea Level", -rounded);
        } else {
            return format!("{rounded:.1} m Above Sea Level");
        }
    }

    // Handle string value that might already be formatted (fallback for existing formatting)
    if let Some(s) = val.as_string() {
        if s == "inf" || s == "undef" {
            return s.to_string();
        }

        // Try to parse numeric value from string like "25.2 m"
        // Simple parsing without regex dependency
        let cleaned = s.trim().trim_end_matches(" m").trim_end_matches("m");
        if let Ok(v) = cleaned.parse::<f64>() {
            let rounded = (v * 10.0).round() / 10.0;
            if rounded < 0.0 {
                return format!("{:.1} m Below Sea Level", -rounded);
            } else {
                return format!("{rounded:.1} m Above Sea Level");
            }
        }
    }

    format!("Unknown ({val})")
}

/// EXIF LensInfo PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm PrintLensInfo function
/// Converts 4 rational values to form "12-20mm f/3.8-4.5" or "50mm f/1.4"
pub fn lensinfo_print_conv(val: &TagValue) -> String {
    // LensInfo should contain 4 rational values
    let vals = match val {
        TagValue::RationalArray(array) => {
            // Extract the 4 rational values
            if array.len() != 4 {
                return format!("Unknown ({val})");
            }

            let mut values = Vec::new();
            for (numerator, denominator) in array {
                if *denominator == 0 {
                    values.push(None); // undefined value (ExifTool shows as "undef")
                } else {
                    values.push(Some(*numerator as f64 / *denominator as f64));
                }
            }
            values
        }
        _ => return format!("Unknown ({val})"),
    };

    // Check we have exactly 4 values
    if vals.len() != 4 {
        return format!("Unknown ({val})");
    }

    // Build the lens info string
    // vals[0] = min focal length
    // vals[1] = max focal length
    // vals[2] = min aperture
    // vals[3] = max aperture

    let mut result = String::new();

    // Focal length range
    match (vals[0], vals[1]) {
        (Some(min_focal), Some(max_focal)) => {
            // Format focal length - use integer formatting unless fractional part exists
            // ExifTool: 3.99mm shows as "3.99mm", not "4mm"
            if min_focal.fract() == 0.0 {
                result.push_str(&format!("{min_focal:.0}"));
            } else {
                // Format with enough precision to match ExifTool
                // ExifTool uses Perl's default number stringification
                // For 299253/190607 = 1.570000052...
                let formatted = format!("{min_focal:.9}");
                // Remove trailing zeros but keep meaningful precision
                let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
                result.push_str(trimmed);
            }

            // Add max focal if different from min (not a prime lens)
            // Pentax Q writes zero for upper value of fixed-focal-length lenses (ExifTool comment)
            if max_focal != min_focal && max_focal > 0.0 {
                result.push('-');
                if max_focal.fract() == 0.0 {
                    result.push_str(&format!("{max_focal:.0}"));
                } else {
                    // Format with enough precision to match ExifTool
                    let formatted = format!("{max_focal:.9}");
                    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
                    result.push_str(trimmed);
                }
            }
            result.push_str("mm");
        }
        _ => return format!("Unknown ({val})"),
    }

    // Aperture range
    match (vals[2], vals[3]) {
        (Some(min_aperture), Some(max_aperture)) => {
            result.push_str(" f/");
            // Format aperture per ExifTool logic
            if min_aperture == 0.0 {
                result.push('0');
            } else if min_aperture < 1.0 {
                result.push_str(&format!("{min_aperture:.2}"));
            } else {
                // ExifTool: Use 1 decimal place, but remove trailing zeros
                let formatted = format!("{min_aperture:.1}");
                let trimmed = if formatted.ends_with(".0") {
                    &formatted[..formatted.len() - 2]
                } else {
                    &formatted
                };
                result.push_str(trimmed);
            }

            // Add max aperture if different from min (variable aperture zoom)
            if max_aperture != min_aperture && max_aperture > 0.0 {
                result.push('-');
                if max_aperture == 0.0 {
                    result.push('0');
                } else if max_aperture < 1.0 {
                    result.push_str(&format!("{max_aperture:.2}"));
                } else {
                    let formatted = format!("{max_aperture:.1}");
                    let trimmed = if formatted.ends_with(".0") {
                        &formatted[..formatted.len() - 2]
                    } else {
                        &formatted
                    };
                    result.push_str(trimmed);
                }
            }
        }
        (None, None) => {
            // Both aperture values are undefined - ExifTool shows as "f/?"
            result.push_str(" f/?");
        }
        _ => return format!("Unknown ({val})"),
    }

    result
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

    #[test]
    fn test_fnumber_print_conv() {
        // Values >= 1.0 get 1 decimal place
        assert_eq!(fnumber_print_conv(&TagValue::F64(4.0)), "4.0");
        assert_eq!(fnumber_print_conv(&TagValue::F64(2.8)), "2.8");
        assert_eq!(fnumber_print_conv(&TagValue::F64(1.4)), "1.4");
        assert_eq!(fnumber_print_conv(&TagValue::F64(11.0)), "11.0");
        assert_eq!(fnumber_print_conv(&TagValue::F64(0.640234375)), "0.64");

        // Values < 1.0 get 2 decimal places
        assert_eq!(fnumber_print_conv(&TagValue::F64(0.95)), "0.95");
        assert_eq!(fnumber_print_conv(&TagValue::F64(0.7)), "0.70");

        // Test with rational input
        assert_eq!(fnumber_print_conv(&TagValue::Rational(4, 1)), "4.0");
    }

    #[test]
    fn test_exposuretime_print_conv() {
        assert_eq!(exposuretime_print_conv(&TagValue::F64(0.0005)), "1/2000");
        assert_eq!(exposuretime_print_conv(&TagValue::F64(0.5)), "1/2");
        assert_eq!(exposuretime_print_conv(&TagValue::F64(2.0)), "2");
        assert_eq!(
            exposuretime_print_conv(&TagValue::Rational(1, 2000)),
            "1/2000"
        );
    }

    #[test]
    fn test_focallength_print_conv() {
        // Integers should not show decimal places
        assert_eq!(focallength_print_conv(&TagValue::F64(24.0)), "24 mm");
        assert_eq!(focallength_print_conv(&TagValue::F64(50.0)), "50 mm");
        assert_eq!(focallength_print_conv(&TagValue::F64(200.0)), "200 mm");
        assert_eq!(focallength_print_conv(&TagValue::F64(0.0)), "0 mm");

        // Decimals should be rounded to 1 decimal place like ExifTool
        assert_eq!(focallength_print_conv(&TagValue::F64(4.67)), "4.7 mm"); // 4.67 -> 4.7
        assert_eq!(focallength_print_conv(&TagValue::F64(42.3)), "42.3 mm");
        assert_eq!(focallength_print_conv(&TagValue::F64(105.5)), "105.5 mm");
        assert_eq!(focallength_print_conv(&TagValue::F64(5.7)), "5.7 mm");
        assert_eq!(focallength_print_conv(&TagValue::F64(1.56)), "1.6 mm"); // 1.56 -> 1.6 (round up)
        assert_eq!(focallength_print_conv(&TagValue::F64(1.54)), "1.5 mm"); // 1.54 -> 1.5 (round down)
        assert_eq!(focallength_print_conv(&TagValue::F64(1.57)), "1.6 mm"); // iPhone case: 1.57 -> 1.6

        // Test with rational input
        assert_eq!(focallength_print_conv(&TagValue::Rational(24, 1)), "24 mm");
        assert_eq!(
            focallength_print_conv(&TagValue::Rational(57, 10)),
            "5.7 mm"
        );
    }

    #[test]
    fn test_focallength_in_35mm_format_print_conv() {
        // Values should be formatted as integers with no decimal places
        assert_eq!(
            focallength_in_35mm_format_print_conv(&TagValue::U16(28)),
            "28 mm"
        );
        assert_eq!(
            focallength_in_35mm_format_print_conv(&TagValue::U16(50)),
            "50 mm"
        );
        assert_eq!(
            focallength_in_35mm_format_print_conv(&TagValue::U16(167)),
            "167 mm"
        );
        assert_eq!(
            focallength_in_35mm_format_print_conv(&TagValue::U16(400)),
            "400 mm"
        );

        // Test with non-U16 value
        assert_eq!(
            focallength_in_35mm_format_print_conv(&TagValue::String("invalid".to_string())),
            "Unknown (invalid)"
        );
    }
}
