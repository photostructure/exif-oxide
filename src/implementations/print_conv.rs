//! PrintConv implementations for exif-oxide
//!
//! This module contains manual implementations of ExifTool's PrintConv functions.
//! Each function converts a raw tag value to a human-readable string.
//!
//! All implementations are direct translations from ExifTool source code,
//! with comments pointing back to the original ExifTool references.

use crate::types::TagValue;

/// EXIF Orientation PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:281-290 (%orientation hash)
/// Generated table: src/generated/Exif_pm/mod.rs
pub fn orientation_print_conv(val: &TagValue) -> TagValue {
    use crate::generated::Exif_pm::lookup_orientation;

    // Handle both u8 and u16 types - orientation values are 1-8 so fit in u8
    let orientation_val = match val {
        TagValue::U8(v) => Some(*v),
        TagValue::U16(v) if *v <= 255 => Some(*v as u8),
        _ => None,
    };

    match orientation_val {
        Some(val) => {
            if let Some(description) = lookup_orientation(val) {
                TagValue::string(description)
            } else {
                TagValue::string(format!("Unknown ({val})"))
            }
        }
        None => TagValue::string(format!("Unknown ({val})")),
    }
}

/// EXIF ResolutionUnit PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:2778-2782
pub fn resolutionunit_print_conv(val: &TagValue) -> TagValue {
    match val.as_u16() {
        Some(1) => "None".into(),
        Some(2) => "inches".into(),
        Some(3) => "cm".into(),
        _ => TagValue::string(format!("Unknown ({val})")),
    }
}

/// EXIF YCbCrPositioning PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:2802-2805
pub fn ycbcrpositioning_print_conv(val: &TagValue) -> TagValue {
    match val.as_u16() {
        Some(1) => "Centered".into(),
        Some(2) => "Co-sited".into(),
        _ => TagValue::string(format!("Unknown ({val})")),
    }
}

/// GPS Altitude PrintConv
/// ExifTool: lib/Image/ExifTool/GPS.pm:124 - '$val =~ /^(inf|undef)$/ ? $val : "$val m"'
pub fn gpsaltitude_print_conv(val: &TagValue) -> TagValue {
    match val.as_f64() {
        Some(v) if v.is_infinite() => "inf".into(),
        Some(v) if v.is_nan() => "undef".into(),
        Some(v) => TagValue::string(format!("{v:.1} m")), // Round to 0.1m - GPS accuracy limit
        None => TagValue::string(format!("Unknown ({val})")),
    }
}

/// GPS AltitudeRef PrintConv
/// ExifTool: lib/Image/ExifTool/GPS.pm GPSAltitudeRef tag definition
pub fn gpsaltituderef_print_conv(val: &TagValue) -> TagValue {
    match val.as_u8() {
        Some(0) => "Above Sea Level".into(),
        Some(1) => "Below Sea Level".into(),
        _ => TagValue::string(format!("Unknown ({val})")),
    }
}

/// GPS LatitudeRef PrintConv
/// ExifTool: lib/Image/ExifTool/GPS.pm GPSLatitudeRef tag definition
pub fn gpslatituderef_print_conv(val: &TagValue) -> TagValue {
    match val.as_string() {
        Some("N") => "North".into(),
        Some("S") => "South".into(),
        _ => TagValue::string(format!("Unknown ({val})")),
    }
}

/// GPS LongitudeRef PrintConv  
/// ExifTool: lib/Image/ExifTool/GPS.pm GPSLongitudeRef tag definition
pub fn gpslongituderef_print_conv(val: &TagValue) -> TagValue {
    match val.as_string() {
        Some("E") => "East".into(),
        Some("W") => "West".into(),
        _ => TagValue::string(format!("Unknown ({val})")),
    }
}

/// GPS Latitude PrintConv - returns decimal degrees directly
/// For GPS coordinates, we want the decimal value (post-ValueConv), not formatted with degree symbols
pub fn gpslatitude_print_conv(val: &TagValue) -> TagValue {
    val.clone() // Return the decimal value as-is
}

/// GPS Longitude PrintConv - returns decimal degrees directly  
/// For GPS coordinates, we want the decimal value (post-ValueConv), not formatted with degree symbols
pub fn gpslongitude_print_conv(val: &TagValue) -> TagValue {
    val.clone() // Return the decimal value as-is
}

/// GPS DestLatitude PrintConv - returns decimal degrees directly
/// For GPS coordinates, we want the decimal value (post-ValueConv), not formatted with degree symbols
pub fn gpsdestlatitude_print_conv(val: &TagValue) -> TagValue {
    val.clone() // Return the decimal value as-is
}

/// GPS DestLongitude PrintConv - returns decimal degrees directly
/// For GPS coordinates, we want the decimal value (post-ValueConv), not formatted with degree symbols
pub fn gpsdestlongitude_print_conv(val: &TagValue) -> TagValue {
    val.clone() // Return the decimal value as-is
}

/// EXIF Flash PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:164-197, %flash hash
/// Generated lookup: src/generated/Exif_pm/flash.rs
/// NOTE: This is NOT a bitmask conversion - ExifTool uses direct hash lookup for specific combined values
pub fn flash_print_conv(val: &TagValue) -> TagValue {
    use crate::generated::Exif_pm::lookup_flash;

    match val.as_u16() {
        Some(flash_val) => {
            if let Some(description) = lookup_flash(flash_val) {
                TagValue::string(description.to_string())
            } else {
                TagValue::string(format!("Unknown ({flash_val})"))
            }
        }
        None => TagValue::string(format!("Unknown ({val})")),
    }
}

/// EXIF ColorSpace PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:2620-2638
pub fn colorspace_print_conv(val: &TagValue) -> TagValue {
    TagValue::string(match val.as_u16() {
        Some(1) => "sRGB".to_string(),
        Some(2) => "Adobe RGB".to_string(),
        Some(0xffff) => "Uncalibrated".to_string(),
        // Sony-specific non-standard values (ref JD)
        Some(0xfffe) => "ICC Profile".to_string(),
        Some(0xfffd) => "Wide Gamut RGB".to_string(),
        // Unknown values shown in parentheses (ExifTool format)
        _ => {
            if let Some(num) = val.as_u16() {
                format!("Unknown ({num})")
            } else {
                format!("Unknown ({val})")
            }
        }
    })
}

/// EXIF WhiteBalance PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:2809-2821
// TODO: Add manufacturer-specific handling - Canon uses "Evaluative" vs "Multi-segment" for MeteringMode
pub fn whitebalance_print_conv(val: &TagValue) -> TagValue {
    TagValue::string(match val.as_u16() {
        Some(0) => "Auto".to_string(),
        Some(1) => "Manual".to_string(),
        _ => format!("Unknown ({val})"),
    })
}

/// EXIF MeteringMode PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:2357-2371
// TODO: Add manufacturer-specific handling - Canon uses "Evaluative" instead of "Multi-segment"
pub fn meteringmode_print_conv(val: &TagValue) -> TagValue {
    TagValue::string(match val.as_u16() {
        Some(0) => "Unknown".to_string(),
        Some(1) => "Average".to_string(),
        Some(2) => "Center-weighted average".to_string(),
        Some(3) => "Spot".to_string(),
        Some(4) => "Multi-spot".to_string(),
        Some(5) => "Multi-segment".to_string(),
        Some(6) => "Partial".to_string(),
        Some(255) => "Other".to_string(),
        _ => format!("Unknown ({val})"),
    })
}

/// EXIF ExposureProgram PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:2082-2097
/// NOTE: Value 9 is not standard EXIF but used by some Canon models
pub fn exposureprogram_print_conv(val: &TagValue) -> TagValue {
    TagValue::string(match val.as_u16() {
        Some(0) => "Not Defined".to_string(),
        Some(1) => "Manual".to_string(),
        Some(2) => "Program AE".to_string(),
        Some(3) => "Aperture-priority AE".to_string(),
        Some(4) => "Shutter speed priority AE".to_string(),
        Some(5) => "Creative (Slow speed)".to_string(),
        Some(6) => "Action (High speed)".to_string(),
        Some(7) => "Portrait".to_string(),
        Some(8) => "Landscape".to_string(),
        Some(9) => "Bulb".to_string(), // Canon-specific non-standard value
        _ => format!("Unknown ({val})"),
    })
}

/// FNumber PrintConv - formats f-stop values
/// ExifTool: lib/Image/ExifTool/Exif.pm PrintFNumber function (lines 5607-5615)
/// Uses 2 decimal places for values < 1.0, 1 decimal place for values >= 1.0
/// NOTE: This returns a numeric TagValue to preserve JSON numeric serialization
pub fn fnumber_print_conv(val: &TagValue) -> TagValue {
    match val.as_f64() {
        Some(f_number) => {
            if f_number > 0.0 {
                // ExifTool logic: 2 decimal places for < 1.0, 1 decimal place for >= 1.0
                // Apply the same rounding as ExifTool but return as numeric value
                let rounded = if f_number < 1.0 {
                    (f_number * 100.0).round() / 100.0 // 2 decimal places
                } else {
                    (f_number * 10.0).round() / 10.0 // 1 decimal place
                };
                TagValue::F64(rounded)
            } else {
                TagValue::string(format!("Unknown ({val})"))
            }
        }
        None => {
            // Handle rational format directly if ValueConv wasn't applied
            if let TagValue::Rational(num, denom) = val {
                if *denom != 0 {
                    let f_number = *num as f64 / *denom as f64;
                    if f_number > 0.0 {
                        let rounded = if f_number < 1.0 {
                            (f_number * 100.0).round() / 100.0 // 2 decimal places
                        } else {
                            (f_number * 10.0).round() / 10.0 // 1 decimal place
                        };
                        return TagValue::F64(rounded);
                    }
                }
            }
            TagValue::string(format!("Unknown ({val})"))
        }
    }
}

/// ExposureTime PrintConv - formats shutter speed
/// ExifTool: lib/Image/ExifTool/Exif.pm:5595-5605 PrintExposureTime
/// Converts decimal seconds to fractional notation (e.g., 0.0005 -> "1/2000")
/// NOTE: Returns numeric TagValue for simple numbers to preserve JSON numeric serialization
pub fn exposuretime_print_conv(val: &TagValue) -> TagValue {
    match val.as_f64() {
        Some(secs) => {
            // ExifTool: return $secs unless Image::ExifTool::IsFloat($secs);
            // We always have floats from as_f64(), so continue with the logic

            // ExifTool: if ($secs < 0.25001 and $secs > 0) {
            if secs < 0.25001 && secs > 0.0 {
                // ExifTool: return sprintf("1/%d",int(0.5 + 1/$secs));
                let denominator = (0.5 + 1.0 / secs) as i32;
                TagValue::string(format!("1/{denominator}"))
            } else {
                // ExifTool: $_ = sprintf("%.1f",$secs);
                let mut result = format!("{secs:.1}");
                // ExifTool: s/\.0$//;
                if result.ends_with(".0") {
                    result.truncate(result.len() - 2);
                }

                // If the result is a simple integer (like "0", "1", "2"), return as numeric
                // to match ExifTool's JSON output behavior. Keep decimals as strings.
                if result.chars().all(|c| c.is_ascii_digit() || c == '-') {
                    if let Ok(int_val) = result.parse::<i32>() {
                        TagValue::I32(int_val)
                    } else {
                        TagValue::string(result)
                    }
                } else {
                    TagValue::string(result)
                }
            }
        }
        None => {
            // Handle rational format directly if ValueConv wasn't applied
            if let TagValue::Rational(num, denom) = val {
                if *denom != 0 && *num != 0 {
                    let secs = *num as f64 / *denom as f64;
                    // Apply the same logic as above
                    if secs < 0.25001 && secs > 0.0 {
                        let denominator = (0.5 + 1.0 / secs) as i32;
                        return TagValue::string(format!("1/{denominator}"));
                    } else {
                        let mut result = format!("{secs:.1}");
                        if result.ends_with(".0") {
                            result.truncate(result.len() - 2);
                        }

                        // If the result is a simple integer, return as numeric
                        if result.chars().all(|c| c.is_ascii_digit() || c == '-') {
                            if let Ok(int_val) = result.parse::<i32>() {
                                return TagValue::I32(int_val);
                            } else {
                                return TagValue::string(result);
                            }
                        } else {
                            return TagValue::string(result);
                        }
                    }
                }
            }
            TagValue::string(format!("Unknown ({val})"))
        }
    }
}

/// FocalLength PrintConv - formats focal length with "mm" unit
/// ExifTool: lib/Image/ExifTool/Exif.pm lines 2387-2393
/// Note: We normalize ExifTool's inconsistent formatting to show integers without decimals
pub fn focallength_print_conv(val: &TagValue) -> TagValue {
    TagValue::string(match val.as_f64() {
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
                        return TagValue::string(format!("{} mm", rounded as i32));
                    } else {
                        return TagValue::string(format!("{rounded:.1} mm"));
                    }
                }
            }
            format!("Unknown ({val})")
        }
    })
}

/// FocalLengthIn35mmFormat PrintConv - formats 35mm equivalent focal length
/// ExifTool: lib/Image/ExifTool/Exif.pm lines 2827-2834
/// PrintConv => '"$val mm"',
pub fn focallength_in_35mm_format_print_conv(val: &TagValue) -> TagValue {
    TagValue::string(match val.as_u16() {
        Some(focal_length) => {
            // Format as integer with no decimal places to match ExifTool
            format!("{focal_length} mm")
        }
        None => format!("Unknown ({val})"),
    })
}

/// Composite GPS Altitude PrintConv
/// ExifTool: lib/Image/ExifTool/GPS.pm:423-431
/// Formats GPS altitude with "Above/Below Sea Level" based on sign
pub fn composite_gps_gpsaltitude_print_conv(val: &TagValue) -> TagValue {
    TagValue::string(
        // Handle numeric value
        if let Some(v) = val.as_f64() {
            if v.is_infinite() {
                "inf".to_string()
            } else if v.is_nan() {
                "undef".to_string()
            } else {
                // Round to 1 decimal place: int($val * 10) / 10
                let rounded = (v * 10.0).round() / 10.0;

                // Check if negative (below sea level) and make positive for display
                if rounded < 0.0 {
                    format!("{:.1} m Below Sea Level", -rounded)
                } else {
                    format!("{rounded:.1} m Above Sea Level")
                }
            }
        } else if let Some(s) = val.as_string() {
            // Handle string value that might already be formatted (fallback for existing formatting)
            if s == "inf" || s == "undef" {
                s.to_string()
            } else {
                // Try to parse numeric value from string like "25.2 m"
                // Simple parsing without regex dependency
                let cleaned = s.trim().trim_end_matches(" m").trim_end_matches("m");
                if let Ok(v) = cleaned.parse::<f64>() {
                    let rounded = (v * 10.0).round() / 10.0;
                    if rounded < 0.0 {
                        format!("{:.1} m Below Sea Level", -rounded)
                    } else {
                        format!("{rounded:.1} m Above Sea Level")
                    }
                } else {
                    format!("Unknown ({val})")
                }
            }
        } else {
            format!("Unknown ({val})")
        },
    )
}

/// ISO PrintConv - passthrough numeric value
/// ExifTool outputs ISO as a JSON number, not a string
/// This is a simple passthrough that preserves the numeric type
pub fn iso_print_conv(val: &TagValue) -> TagValue {
    // ISO values should remain numeric in JSON output
    val.clone()
}

/// EXIF LensInfo PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm PrintLensInfo function
/// Converts 4 rational values to form "12-20mm f/3.8-4.5" or "50mm f/1.4"
pub fn lensinfo_print_conv(val: &TagValue) -> TagValue {
    // LensInfo should contain 4 rational values
    let vals = match val {
        TagValue::RationalArray(array) => {
            // Extract the 4 rational values
            if array.len() != 4 {
                return TagValue::string(format!("Unknown ({val})"));
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
        _ => return TagValue::string(format!("Unknown ({val})")),
    };

    // Check we have exactly 4 values
    if vals.len() != 4 {
        return TagValue::string(format!("Unknown ({val})"));
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
        _ => return TagValue::string(format!("Unknown ({val})")),
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
        _ => return TagValue::string(format!("Unknown ({val})")),
    }

    TagValue::string(result)
}

/// Generic decimal formatting functions for sprintf patterns
/// ExifTool: sprintf("%.1f",$val) - outputs numeric JSON when value looks like a number
pub fn decimal_1_print_conv(val: &TagValue) -> TagValue {
    let formatted = format!("{:.1}", val.as_f64().unwrap_or(0.0));
    // Use ExifTool's numeric detection to match JSON output behavior
    TagValue::string_with_numeric_detection(formatted)
}

pub fn decimal_2_print_conv(val: &TagValue) -> TagValue {
    let formatted = format!("{:.2}", val.as_f64().unwrap_or(0.0));
    // Use ExifTool's numeric detection to match JSON output behavior
    TagValue::string_with_numeric_detection(formatted)
}

pub fn signed_int_print_conv(val: &TagValue) -> TagValue {
    match val {
        TagValue::U8(v) => TagValue::String(format!("{:+}", *v as i32)),
        TagValue::U16(v) => TagValue::String(format!("{:+}", *v as i32)),
        TagValue::U32(v) => TagValue::String(format!("{:+}", *v as i32)),
        TagValue::SRational(num, den) if *den != 0 => TagValue::String(format!("{:+}", num / den)),
        _ => TagValue::String(format!("{:+.0}", val.as_f64().unwrap_or(0.0))),
    }
}

pub fn focal_length_3_decimals_print_conv(val: &TagValue) -> TagValue {
    TagValue::String(format!("{:.3} mm", val.as_f64().unwrap_or(0.0)))
}

/// Complex expression placeholder - delegates to appropriate function
/// This is used when tag_kit.pl can't determine the exact function
pub fn complex_expression_print_conv(val: &TagValue) -> TagValue {
    // For now, return the value unchanged
    // In the future, we could use context to determine the right function
    val.clone()
}

/// EXIF PrintFraction PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm PrintFraction sub
/// Converts a floating point value to a fractional representation with sign
/// Used for exposure compensation and similar values
pub fn print_fraction(val: &TagValue) -> TagValue {
    // Get floating point value
    let f_val = match val {
        TagValue::F64(v) => Some(*v),
        TagValue::Rational(num, den) => {
            if *den != 0 {
                Some(*num as f64 / *den as f64)
            } else {
                None
            }
        }
        TagValue::SRational(num, den) => {
            if *den != 0 {
                Some(*num as f64 / *den as f64)
            } else {
                None
            }
        }
        TagValue::I16(v) => Some(*v as f64),
        TagValue::I32(v) => Some(*v as f64),
        TagValue::U8(v) => Some(*v as f64),
        TagValue::U16(v) => Some(*v as f64),
        TagValue::U32(v) => Some(*v as f64),
        _ => None,
    };

    match f_val {
        Some(mut v) => {
            // ExifTool multiplies by 1.00001 to avoid round-off errors
            v *= 1.00001;

            if v == 0.0 {
                // ExifTool returns '0' string, but JSON output shows number 0
                // Match ExifTool's JSON behavior for zero values
                TagValue::F64(0.0)
            } else if (v.floor() / v).abs() > 0.999 {
                // Whole number
                TagValue::string(format!("{:+.0}", v.floor()))
            } else if ((v * 2.0).floor() / (v * 2.0)).abs() > 0.999 {
                // Half fraction (n/2)
                TagValue::string(format!("{:+.0}/2", (v * 2.0).floor()))
            } else if ((v * 3.0).floor() / (v * 3.0)).abs() > 0.999 {
                // Third fraction (n/3)
                TagValue::string(format!("{:+.0}/3", (v * 3.0).floor()))
            } else {
                // Use scientific notation with 3 significant digits
                TagValue::string(format!("{:+.3}", v))
            }
        }
        None => TagValue::string(format!("Unknown ({})", val)),
    }
}

/// Composite ImageSize PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:4641-4660 ImageSize PrintConv
/// PrintConv: '$val =~ tr/ /x/; $val' - replaces space with 'x' separator
pub fn imagesize_print_conv(val: &TagValue) -> TagValue {
    if let Some(size_str) = val.as_string() {
        // ExifTool: $val =~ tr/ /x/; $val
        // Replace space with 'x' to convert "1920 1080" to "1920x1080"
        TagValue::string(size_str.replace(' ', "x"))
    } else {
        TagValue::string(format!("Unknown ({val})"))
    }
}

/// EXIF ExifVersion PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:2203-2209
/// Converts undef bytes to ASCII string (e.g., [48, 50, 50, 48] -> "0220")
pub fn exifversion_print_conv(val: &TagValue) -> TagValue {
    match val {
        TagValue::U8Array(bytes) | TagValue::Binary(bytes) => {
            // Convert byte array to ASCII string
            let ascii_string: String = bytes
                .iter()
                .map(|&b| b as char)
                .collect::<String>()
                .trim_end_matches('\0') // Remove null terminators
                .to_string();
            TagValue::string(ascii_string)
        }
        _ => TagValue::string(format!("Unknown ({val})")),
    }
}

/// EXIF FlashpixVersion PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:2613-2619
/// Converts undef bytes to ASCII string (e.g., [48, 48, 49, 48] -> "0100")
pub fn flashpixversion_print_conv(val: &TagValue) -> TagValue {
    match val {
        TagValue::U8Array(bytes) | TagValue::Binary(bytes) => {
            // Convert byte array to ASCII string
            let ascii_string: String = bytes
                .iter()
                .map(|&b| b as char)
                .collect::<String>()
                .trim_end_matches('\0') // Remove null terminators
                .to_string();
            TagValue::string(ascii_string)
        }
        _ => TagValue::string(format!("Unknown ({val})")),
    }
}

/// EXIF InteropVersion PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:2850-2856 (InteropIFD definition)
/// Converts undef bytes to ASCII string (e.g., [48, 49, 48, 48] -> "0100")
pub fn interopversion_print_conv(val: &TagValue) -> TagValue {
    match val {
        TagValue::U8Array(bytes) | TagValue::Binary(bytes) => {
            // Convert byte array to ASCII string
            let ascii_string: String = bytes
                .iter()
                .map(|&b| b as char)
                .collect::<String>()
                .trim_end_matches('\0') // Remove null terminators
                .to_string();
            TagValue::string(ascii_string)
        }
        _ => TagValue::string(format!("Unknown ({val})")),
    }
}

/// EXIF ComponentsConfiguration PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:2262-2300
/// Converts component array to "Y, Cb, Cr, -" format
pub fn componentsconfiguration_print_conv(val: &TagValue) -> TagValue {
    match val {
        TagValue::U8Array(components) | TagValue::Binary(components) => {
            let component_names: Vec<&str> = components
                .iter()
                .map(|&comp| match comp {
                    0 => "-",
                    1 => "Y",
                    2 => "Cb",
                    3 => "Cr",
                    4 => "R",
                    5 => "G",
                    6 => "B",
                    _ => "Err",
                })
                .collect();

            TagValue::string(component_names.join(", "))
        }
        _ => TagValue::string(format!("Unknown ({val})")),
    }
}

/// EXIF FileSource PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:2313-2320
/// Converts FileSource byte value to descriptive string
pub fn filesource_print_conv(val: &TagValue) -> TagValue {
    match val {
        TagValue::U8Array(bytes) | TagValue::Binary(bytes) => {
            if let Some(&first_byte) = bytes.first() {
                match first_byte {
                    1 => TagValue::string("Film Scanner"),
                    2 => TagValue::string("Reflection Print Scanner"),
                    3 => TagValue::string("Digital Camera"),
                    _ => TagValue::string(format!("Unknown ({})", first_byte)),
                }
            } else {
                TagValue::string(format!("Unknown ({val})"))
            }
        }
        TagValue::U8(val) => match val {
            1 => TagValue::string("Film Scanner"),
            2 => TagValue::string("Reflection Print Scanner"),
            3 => TagValue::string("Digital Camera"),
            _ => TagValue::string(format!("Unknown ({val})")),
        },
        _ => TagValue::string(format!("Unknown ({val})")),
    }
}

/// EXIF CompressedBitsPerPixel PrintConv
/// ExifTool: lib/Image/ExifTool/Exif.pm:2301-2305
/// Standard rational64u conversion to decimal
pub fn compressedbitsperpixel_print_conv(val: &TagValue) -> TagValue {
    match val {
        TagValue::RationalArray(rationals) if rationals.len() == 1 => {
            let (num, den) = rationals[0];
            if den != 0 {
                TagValue::F64(num as f64 / den as f64)
            } else {
                TagValue::string(format!("Unknown ({val})"))
            }
        }
        TagValue::Rational(num, den) => {
            if *den != 0 {
                TagValue::F64(*num as f64 / *den as f64)
            } else {
                TagValue::string(format!("Unknown ({val})"))
            }
        }
        _ => TagValue::string(format!("Unknown ({val})")),
    }
}

/// Canon FileNumber PrintConv
/// ExifTool: lib/Image/ExifTool/Canon.pm:1229
/// PrintConv: '$_=$val,s/(\d+)(\d{4})/$1-$2/,$_'
/// Converts numeric FileNumber to "directory-file" format (e.g., 1181861 -> "118-1861")
pub fn canon_filenumber_print_conv(val: &TagValue) -> TagValue {
    // Convert value to string representation
    let val_str = match val {
        TagValue::U32(v) => v.to_string(),
        TagValue::U16(v) => v.to_string(),
        TagValue::I32(v) => v.to_string(),
        TagValue::String(s) => s.clone(),
        _ => return TagValue::string(format!("Unknown ({})", val)),
    };

    // ExifTool regex: s/(\d+)(\d{4})/$1-$2/
    // Match: digits followed by exactly 4 digits, replace with "digits-4digits"
    if val_str.len() >= 5 {
        let (directory, file) = val_str.split_at(val_str.len() - 4);
        TagValue::string(format!("{}-{}", directory, file))
    } else {
        // If less than 5 digits, can't split properly - return as-is
        TagValue::string(val_str)
    }
}

/// Canon SelfTimer PrintConv
/// ExifTool: lib/Image/ExifTool/Canon.pm:2182-2184 (SelfTimer Print)
pub fn canon_selftimer_print_conv(val: &TagValue) -> TagValue {
    if let Some(timer_val) = val.as_u16() {
        let timer_val = timer_val as i16; // Convert u16 to i16 for signed operations

        // return 'Off' unless $val;
        if timer_val == 0 {
            return TagValue::string("Off".to_string());
        }

        // return (($val&0xfff) / 10) . ' s' . ($val & 0x4000 ? ', Custom' : '');
        let seconds = ((timer_val & 0xfff) as f32) / 10.0;
        let custom = if timer_val & 0x4000 != 0 {
            ", Custom"
        } else {
            ""
        };
        TagValue::string(format!("{} s{}", seconds, custom))
    } else {
        TagValue::string(format!("Unknown ({})", val))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orientation_print_conv() {
        assert_eq!(
            orientation_print_conv(&TagValue::U16(1)),
            "Horizontal (normal)".into()
        );
        assert_eq!(
            orientation_print_conv(&TagValue::U16(8)),
            "Rotate 270 CW".into()
        );
        assert_eq!(
            orientation_print_conv(&TagValue::U16(99)),
            "Unknown (99)".into()
        );
    }

    #[test]
    fn test_resolutionunit_print_conv() {
        assert_eq!(resolutionunit_print_conv(&TagValue::U16(1)), "None".into());
        assert_eq!(
            resolutionunit_print_conv(&TagValue::U16(2)),
            "inches".into()
        );
        assert_eq!(resolutionunit_print_conv(&TagValue::U16(3)), "cm".into());
        assert_eq!(
            resolutionunit_print_conv(&TagValue::U16(99)),
            "Unknown (99)".into()
        );
    }

    #[test]
    fn test_ycbcrpositioning_print_conv() {
        assert_eq!(
            ycbcrpositioning_print_conv(&TagValue::U16(1)),
            "Centered".into()
        );
        assert_eq!(
            ycbcrpositioning_print_conv(&TagValue::U16(2)),
            "Co-sited".into()
        );
        assert_eq!(
            ycbcrpositioning_print_conv(&TagValue::U16(99)),
            "Unknown (99)".into()
        );
    }

    #[test]
    fn test_flash_print_conv() {
        // Test standard values
        assert_eq!(flash_print_conv(&TagValue::U16(0x00)), "No Flash".into());
        assert_eq!(flash_print_conv(&TagValue::U16(0x01)), "Fired".into());
        assert_eq!(flash_print_conv(&TagValue::U16(0x19)), "Auto, Fired".into());
        assert_eq!(
            flash_print_conv(&TagValue::U16(0x20)),
            "No flash function".into()
        );

        // Test red-eye reduction values
        assert_eq!(
            flash_print_conv(&TagValue::U16(0x41)),
            "Fired, Red-eye reduction".into()
        );
        assert_eq!(
            flash_print_conv(&TagValue::U16(0x59)),
            "Auto, Fired, Red-eye reduction".into()
        );

        // Test unknown value
        assert_eq!(
            flash_print_conv(&TagValue::U16(0x99)),
            "Unknown (153)".into()
        );
    }

    #[test]
    fn test_colorspace_print_conv() {
        assert_eq!(colorspace_print_conv(&TagValue::U16(1)), "sRGB".into());
        assert_eq!(colorspace_print_conv(&TagValue::U16(2)), "Adobe RGB".into());
        assert_eq!(
            colorspace_print_conv(&TagValue::U16(0xffff)),
            "Uncalibrated".into()
        );

        // Test Sony-specific values
        assert_eq!(
            colorspace_print_conv(&TagValue::U16(0xfffe)),
            "ICC Profile".into()
        );
        assert_eq!(
            colorspace_print_conv(&TagValue::U16(0xfffd)),
            "Wide Gamut RGB".into()
        );

        // Test unknown value
        assert_eq!(
            colorspace_print_conv(&TagValue::U16(0x99)),
            "Unknown (153)".into()
        );
    }

    #[test]
    fn test_whitebalance_print_conv() {
        assert_eq!(whitebalance_print_conv(&TagValue::U16(0)), "Auto".into());
        assert_eq!(whitebalance_print_conv(&TagValue::U16(1)), "Manual".into());
        assert_eq!(
            whitebalance_print_conv(&TagValue::U16(99)),
            "Unknown (99)".into()
        );
    }

    #[test]
    fn test_meteringmode_print_conv() {
        assert_eq!(meteringmode_print_conv(&TagValue::U16(0)), "Unknown".into());
        assert_eq!(meteringmode_print_conv(&TagValue::U16(1)), "Average".into());
        assert_eq!(
            meteringmode_print_conv(&TagValue::U16(2)),
            "Center-weighted average".into()
        );
        assert_eq!(meteringmode_print_conv(&TagValue::U16(3)), "Spot".into());
        assert_eq!(
            meteringmode_print_conv(&TagValue::U16(5)),
            "Multi-segment".into()
        );
        assert_eq!(meteringmode_print_conv(&TagValue::U16(255)), "Other".into());
        assert_eq!(
            meteringmode_print_conv(&TagValue::U16(99)),
            "Unknown (99)".into()
        );
    }

    #[test]
    fn test_exposureprogram_print_conv() {
        assert_eq!(
            exposureprogram_print_conv(&TagValue::U16(0)),
            "Not Defined".into()
        );
        assert_eq!(
            exposureprogram_print_conv(&TagValue::U16(1)),
            "Manual".into()
        );
        assert_eq!(
            exposureprogram_print_conv(&TagValue::U16(2)),
            "Program AE".into()
        );
        assert_eq!(
            exposureprogram_print_conv(&TagValue::U16(3)),
            "Aperture-priority AE".into()
        );
        assert_eq!(
            exposureprogram_print_conv(&TagValue::U16(7)),
            "Portrait".into()
        );

        // Test Canon-specific non-standard value
        assert_eq!(exposureprogram_print_conv(&TagValue::U16(9)), "Bulb".into());

        assert_eq!(
            exposureprogram_print_conv(&TagValue::U16(99)),
            "Unknown (99)".into()
        );
    }

    #[test]
    fn test_fnumber_print_conv() {
        // Values >= 1.0 get 1 decimal place
        assert_eq!(fnumber_print_conv(&TagValue::F64(4.0)), TagValue::F64(4.0));
        assert_eq!(fnumber_print_conv(&TagValue::F64(2.8)), TagValue::F64(2.8));
        assert_eq!(fnumber_print_conv(&TagValue::F64(1.4)), TagValue::F64(1.4));
        assert_eq!(
            fnumber_print_conv(&TagValue::F64(11.0)),
            TagValue::F64(11.0)
        );
        assert_eq!(
            fnumber_print_conv(&TagValue::F64(0.640234375)),
            TagValue::F64(0.64)
        );

        // Values < 1.0 get 2 decimal places
        assert_eq!(
            fnumber_print_conv(&TagValue::F64(0.95)),
            TagValue::F64(0.95)
        );
        assert_eq!(fnumber_print_conv(&TagValue::F64(0.7)), TagValue::F64(0.7));

        // Test with rational input
        assert_eq!(
            fnumber_print_conv(&TagValue::Rational(4, 1)),
            TagValue::F64(4.0)
        );
    }

    #[test]
    fn test_exposuretime_print_conv() {
        // Test fractional exposures < 0.25001
        assert_eq!(
            exposuretime_print_conv(&TagValue::F64(0.0005)),
            "1/2000".into()
        );
        assert_eq!(
            exposuretime_print_conv(&TagValue::F64(0.01)),
            "1/100".into()
        );
        assert_eq!(exposuretime_print_conv(&TagValue::F64(0.125)), "1/8".into());
        assert_eq!(exposuretime_print_conv(&TagValue::F64(0.25)), "1/4".into());

        // Test exposures >= 0.25001
        assert_eq!(
            exposuretime_print_conv(&TagValue::F64(0.0)),
            TagValue::I32(0)
        ); // Zero becomes number (like ExifTool)
        assert_eq!(exposuretime_print_conv(&TagValue::F64(0.5)), "0.5".into()); // Decimal stays string
        assert_eq!(
            exposuretime_print_conv(&TagValue::F64(1.0)),
            TagValue::I32(1)
        ); // Integer becomes number
        assert_eq!(
            exposuretime_print_conv(&TagValue::F64(2.0)),
            TagValue::I32(2)
        ); // Integer becomes number
        assert_eq!(exposuretime_print_conv(&TagValue::F64(2.5)), "2.5".into()); // Decimal stays string

        // Test with rational input
        assert_eq!(
            exposuretime_print_conv(&TagValue::Rational(1, 2000)),
            "1/2000".into()
        );
        assert_eq!(
            exposuretime_print_conv(&TagValue::Rational(1, 100)),
            "1/100".into()
        );
        assert_eq!(
            exposuretime_print_conv(&TagValue::Rational(1, 2)),
            "0.5".into()
        );
    }

    #[test]
    fn test_focallength_print_conv() {
        // Integers should not show decimal places
        assert_eq!(focallength_print_conv(&TagValue::F64(24.0)), "24 mm".into());
        assert_eq!(focallength_print_conv(&TagValue::F64(50.0)), "50 mm".into());
        assert_eq!(
            focallength_print_conv(&TagValue::F64(200.0)),
            "200 mm".into()
        );
        assert_eq!(focallength_print_conv(&TagValue::F64(0.0)), "0 mm".into());

        // Decimals should be rounded to 1 decimal place like ExifTool
        assert_eq!(
            focallength_print_conv(&TagValue::F64(4.67)),
            "4.7 mm".into()
        ); // 4.67 -> 4.7
        assert_eq!(
            focallength_print_conv(&TagValue::F64(42.3)),
            "42.3 mm".into()
        );
        assert_eq!(
            focallength_print_conv(&TagValue::F64(105.5)),
            "105.5 mm".into()
        );
        assert_eq!(focallength_print_conv(&TagValue::F64(5.7)), "5.7 mm".into());
        assert_eq!(
            focallength_print_conv(&TagValue::F64(1.56)),
            "1.6 mm".into()
        ); // 1.56 -> 1.6 (round up)
        assert_eq!(
            focallength_print_conv(&TagValue::F64(1.54)),
            "1.5 mm".into()
        ); // 1.54 -> 1.5 (round down)
        assert_eq!(
            focallength_print_conv(&TagValue::F64(1.57)),
            "1.6 mm".into()
        ); // iPhone case: 1.57 -> 1.6

        // Test with rational input
        assert_eq!(
            focallength_print_conv(&TagValue::Rational(24, 1)),
            "24 mm".into()
        );
        assert_eq!(
            focallength_print_conv(&TagValue::Rational(57, 10)),
            "5.7 mm".into()
        );
    }

    #[test]
    fn test_focallength_in_35mm_format_print_conv() {
        // Values should be formatted as integers with no decimal places
        assert_eq!(
            focallength_in_35mm_format_print_conv(&TagValue::U16(28)),
            "28 mm".into()
        );
        assert_eq!(
            focallength_in_35mm_format_print_conv(&TagValue::U16(50)),
            "50 mm".into()
        );
        assert_eq!(
            focallength_in_35mm_format_print_conv(&TagValue::U16(167)),
            "167 mm".into()
        );
        assert_eq!(
            focallength_in_35mm_format_print_conv(&TagValue::U16(400)),
            "400 mm".into()
        );

        // Test with non-U16 value
        assert_eq!(
            focallength_in_35mm_format_print_conv(&"invalid".into()),
            "Unknown (invalid)".into()
        );
    }

    #[test]
    fn test_gps_altitude_ref_print_conv() {
        // Test numeric values
        assert_eq!(
            gpsaltituderef_print_conv(&TagValue::U8(0)),
            "Above Sea Level".into()
        );
        assert_eq!(
            gpsaltituderef_print_conv(&TagValue::U8(1)),
            "Below Sea Level".into()
        );

        // Test unknown values
        assert_eq!(
            gpsaltituderef_print_conv(&TagValue::U8(99)),
            "Unknown (99)".into()
        );
    }

    #[test]
    fn test_gps_latitude_ref_print_conv() {
        // Test string values
        assert_eq!(
            gpslatituderef_print_conv(&TagValue::String("N".to_string())),
            "North".into()
        );
        assert_eq!(
            gpslatituderef_print_conv(&TagValue::String("S".to_string())),
            "South".into()
        );

        // Test unknown values
        assert_eq!(
            gpslatituderef_print_conv(&TagValue::String("X".to_string())),
            "Unknown (X)".into()
        );
    }

    #[test]
    fn test_gps_longitude_ref_print_conv() {
        // Test string values
        assert_eq!(
            gpslongituderef_print_conv(&TagValue::String("E".to_string())),
            "East".into()
        );
        assert_eq!(
            gpslongituderef_print_conv(&TagValue::String("W".to_string())),
            "West".into()
        );

        // Test unknown values
        assert_eq!(
            gpslongituderef_print_conv(&TagValue::String("Z".to_string())),
            "Unknown (Z)".into()
        );
    }

    #[test]
    fn test_print_fraction() {
        // Test zero value
        assert_eq!(print_fraction(&TagValue::F64(0.0)), TagValue::F64(0.0));

        // Test SRational zero (like ExposureCompensation 0/3)
        assert_eq!(
            print_fraction(&TagValue::SRational(0, 3)),
            TagValue::F64(0.0)
        );

        // Note: Other fraction formatting tests not critical for ExposureCompensation fix
    }

    #[test]
    fn test_canon_filenumber_print_conv() {
        // Test Canon FileNumber PrintConv
        // ExifTool: converts 1181861 -> "118-1861"
        assert_eq!(
            canon_filenumber_print_conv(&TagValue::U32(1181861)),
            TagValue::String("118-1861".to_string())
        );

        // Test other formats
        assert_eq!(
            canon_filenumber_print_conv(&TagValue::U16(12345)),
            TagValue::String("1-2345".to_string())
        );

        // Test edge cases
        assert_eq!(
            canon_filenumber_print_conv(&TagValue::U32(1000)),
            TagValue::String("1000".to_string()) // Less than 5 digits, return as-is
        );

        // Test string format
        assert_eq!(
            canon_filenumber_print_conv(&TagValue::String("1181861".to_string())),
            TagValue::String("118-1861".to_string())
        );
    }
}
