//! Individual composite tag computation implementations
//!
//! This module contains the specific computation functions for each composite tag,
//! translating ExifTool's Perl ValueConv expressions to Rust.

use regex::Regex;
use std::collections::HashMap;
use tracing::trace;

use crate::types::TagValue;

/// Compute ImageSize composite (ImageWidth + ImageHeight)
/// ExifTool: lib/Image/ExifTool/Composite.pm ImageSize definition
/// ValueConv: return $val[4] if $val[4]; return "$val[2] $val[3]" if $val[2] and $val[3] and $$self{TIFF_TYPE} =~ /^(CR2|Canon 1D RAW|IIQ|EIP)$/; return "$val[0] $val[1]" if IsFloat($val[0]) and IsFloat($val[1]); return undef;
pub fn compute_image_size(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // Check RawImageCroppedSize first (index 4 in desire list)
    if let Some(raw_size) = available_tags.get("RawImageCroppedSize") {
        return Some(raw_size.clone());
    }

    // Try ExifImageWidth/ExifImageHeight (indexes 2,3 in desire list)
    // Note: ExifTool checks TIFF_TYPE here, but we'll use these for all files for now
    if let (Some(width), Some(height)) = (
        available_tags.get("ExifImageWidth"),
        available_tags.get("ExifImageHeight"),
    ) {
        if let (Some(w), Some(h)) = (width.as_u32(), height.as_u32()) {
            return Some(TagValue::string(format!("{w} {h}"))); // ExifTool uses space separator
        }
    }

    // Finally try ImageWidth/ImageHeight (indexes 0,1 in require list)
    if let (Some(width), Some(height)) = (
        available_tags.get("ImageWidth"),
        available_tags.get("ImageHeight"),
    ) {
        if let (Some(w), Some(h)) = (width.as_u32(), height.as_u32()) {
            return Some(TagValue::string(format!("{w} {h}"))); // ExifTool uses space separator
        }
    }

    None
}

/// Compute GPSAltitude composite (GPSAltitude + GPSAltitudeRef)
/// ExifTool: lib/Image/ExifTool/GPS.pm GPSAltitude composite
pub fn compute_gps_altitude(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    if let Some(altitude) = available_tags.get("GPSAltitude") {
        let alt_ref = available_tags.get("GPSAltitudeRef");

        // Convert rational to decimal
        if let Some(alt_value) = altitude.as_rational() {
            let decimal_alt = alt_value.0 as f64 / alt_value.1 as f64;

            // Check if below sea level (ref = 1)
            let sign = if let Some(ref_val) = alt_ref {
                if let Some(ref_str) = ref_val.as_string() {
                    if ref_str == "1" {
                        "-"
                    } else {
                        ""
                    }
                } else {
                    ""
                }
            } else {
                ""
            };

            return Some(TagValue::string(format!("{sign}{decimal_alt:.1} m")));
        }
    }
    None
}

/// Compute PreviewImageSize composite
pub fn compute_preview_image_size(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    if let (Some(width), Some(height)) = (
        available_tags.get("PreviewImageWidth"),
        available_tags.get("PreviewImageHeight"),
    ) {
        if let (Some(w), Some(h)) = (width.as_u32(), height.as_u32()) {
            return Some(TagValue::string(format!("{w}x{h}")));
        }
    }
    None
}

/// Compute ShutterSpeed composite (ExposureTime formatted as '1/x' or 'x''')  
/// ExifTool: lib/Image/ExifTool/Composite.pm ShutterSpeed definition
/// ValueConv: ($val[2] and $val[2]>0) ? $val[2] : (defined($val[0]) ? $val[0] : $val[1])
/// Dependencies: ExposureTime(0), ShutterSpeedValue(1), BulbDuration(2)
pub fn compute_shutter_speed(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // Check BulbDuration first (index 2) - if > 0, use it
    if let Some(bulb_duration) = available_tags.get("BulbDuration") {
        if let Some(duration) = bulb_duration.as_f64() {
            if duration > 0.0 {
                return Some(format_shutter_speed(duration));
            }
        }
    }

    // Try ExposureTime (index 0)
    if let Some(exposure_time) = available_tags.get("ExposureTime") {
        if let Some(time) = exposure_time.as_f64() {
            return Some(format_shutter_speed(time));
        }
        // Handle rational ExposureTime
        if let Some((num, den)) = exposure_time.as_rational() {
            if den != 0 {
                let time = num as f64 / den as f64;
                return Some(format_shutter_speed(time));
            }
        }
    }

    // Finally try ShutterSpeedValue (index 1)
    if let Some(shutter_speed_val) = available_tags.get("ShutterSpeedValue") {
        if let Some(speed_val) = shutter_speed_val.as_f64() {
            // ShutterSpeedValue is typically in APEX units: speed = 2^value
            let time = 2.0_f64.powf(-speed_val);
            return Some(format_shutter_speed(time));
        }
        // Handle rational ShutterSpeedValue
        if let Some((num, den)) = shutter_speed_val.as_rational() {
            if den != 0 {
                let speed_val = num as f64 / den as f64;
                let time = 2.0_f64.powf(-speed_val);
                return Some(format_shutter_speed(time));
            }
        }
    }

    None
}

/// Format shutter speed as '1/x' for fast speeds or 'x' for slow speeds
/// ExifTool: lib/Image/ExifTool/Exif.pm PrintConv for shutter speeds
pub fn format_shutter_speed(time_seconds: f64) -> TagValue {
    if time_seconds >= 1.0 {
        // Slow shutter speeds: format as decimal seconds
        TagValue::string(format!("{time_seconds:.1}"))
    } else if time_seconds > 0.0 {
        // Fast shutter speeds: format as 1/x
        let reciprocal = 1.0 / time_seconds;
        TagValue::string(format!("1/{:.0}", reciprocal.round()))
    } else {
        // Invalid time value
        "0".into()
    }
}

/// Compute Aperture composite tag
/// ExifTool: lib/Image/ExifTool/Composite.pm - "$val[0] || $val[1]"
/// Tries FNumber first, falls back to ApertureValue
pub fn compute_aperture(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // Try FNumber first (index 0 in desire list)
    if let Some(fnumber) = available_tags.get("FNumber") {
        return Some(fnumber.clone());
    }

    // Fall back to ApertureValue (index 1 in desire list)
    if let Some(aperture_value) = available_tags.get("ApertureValue") {
        return Some(aperture_value.clone());
    }

    None
}

/// Compute DateTimeOriginal composite tag
/// ExifTool: lib/Image/ExifTool/Composite.pm
/// Returns DateTimeCreated if it contains a space, otherwise combines DateCreated + TimeCreated
pub fn compute_datetime_original(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // Check DateTimeCreated first (index 0)
    if let Some(datetime_created) = available_tags.get("DateTimeCreated") {
        if let Some(dt_str) = datetime_created.as_string() {
            if dt_str.contains(' ') {
                return Some(datetime_created.clone());
            }
        }
    }

    // Combine DateCreated and TimeCreated
    if let (Some(date), Some(time)) = (
        available_tags.get("DateCreated"),
        available_tags.get("TimeCreated"),
    ) {
        if let (Some(date_str), Some(time_str)) = (date.as_string(), time.as_string()) {
            return Some(TagValue::string(format!("{date_str} {time_str}")));
        }
    }

    None
}

/// Compute FocalLength35efl composite tag
/// ExifTool: lib/Image/ExifTool/Composite.pm
/// ValueConv: "ToFloat(@val); ($val[0] || 0) * ($val[1] || 1)"
pub fn compute_focal_length_35efl(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    if let Some(focal_length) = available_tags.get("FocalLength") {
        let fl = focal_length.as_f64().unwrap_or(0.0);

        // Get ScaleFactor35efl if available (index 1 in desire list)
        let scale_factor = available_tags
            .get("ScaleFactor35efl")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);

        let result = fl * scale_factor;
        return Some(TagValue::F64(result));
    }

    None
}

/// Compute ScaleFactor35efl composite tag
/// ExifTool: lib/Image/ExifTool/Composite.pm
/// ValueConv: "Image::ExifTool::Exif::CalcScaleFactor35efl($self, @val)"
/// This is a complex calculation that depends on many factors
pub fn compute_scale_factor_35efl(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // This is a placeholder for the complex ScaleFactor35efl calculation
    // The full implementation requires porting CalcScaleFactor35efl from ExifTool

    // At minimum, we need FocalLength to compute scale factor
    available_tags.get("FocalLength")?;

    // For now, return a default scale factor of 1.0 when we have focal length
    // TODO: Implement full CalcScaleFactor35efl logic (Milestone 11.5 or later)
    trace!("ScaleFactor35efl computation not fully implemented - returning 1.0");
    Some(TagValue::F64(1.0))
}

/// Compute SubSecDateTimeOriginal composite tag
/// ExifTool: lib/Image/ExifTool/Composite.pm
/// Combines DateTimeOriginal with SubSecTimeOriginal and OffsetTimeOriginal
pub fn compute_subsec_datetime_original(
    available_tags: &HashMap<String, TagValue>,
) -> Option<TagValue> {
    // Require DateTimeOriginal
    let datetime_original = available_tags.get("DateTimeOriginal")?;
    let datetime_str = datetime_original.as_string()?;

    let mut result = datetime_str.to_string();

    // Add SubSecTimeOriginal if available
    if let Some(subsec) = available_tags.get("SubSecTimeOriginal") {
        if let Some(subsec_str) = subsec.as_string() {
            if !subsec_str.is_empty() {
                result.push('.');
                result.push_str(subsec_str);
            }
        }
    }

    // Add OffsetTimeOriginal if available
    if let Some(offset) = available_tags.get("OffsetTimeOriginal") {
        if let Some(offset_str) = offset.as_string() {
            if !offset_str.is_empty() {
                result.push_str(offset_str);
            }
        }
    }

    Some(TagValue::String(result))
}

/// Compute CircleOfConfusion composite tag
/// ExifTool: lib/Image/ExifTool/Composite.pm
/// ValueConv: "sqrt(24*24+36*36) / ($val * 1440)"
pub fn compute_circle_of_confusion(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // Require ScaleFactor35efl
    let scale_factor = available_tags.get("ScaleFactor35efl")?;
    let scale = scale_factor.as_f64()?;

    if scale == 0.0 {
        return None;
    }

    // Calculate diagonal of 35mm frame: sqrt(24^2 + 36^2) = 43.267...
    let diagonal_35mm = (24.0_f64 * 24.0 + 36.0 * 36.0).sqrt();

    // CoC = diagonal / (scale * 1440)
    let coc = diagonal_35mm / (scale * 1440.0);

    Some(TagValue::F64(coc))
}

/// Compute Megapixels composite tag from ImageSize
/// ExifTool: lib/Image/ExifTool/Composite.pm Megapixels definition
/// ValueConv: my @d = ($val =~ /\d+/g); $d[0] * $d[1] / 1000000
pub fn compute_megapixels(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // Require ImageSize
    let image_size = available_tags.get("ImageSize")?;
    let image_size_str = image_size.as_string()?;

    // Trust ExifTool: extract all digit sequences from the string
    // The Perl regex /\d+/g finds all sequences of digits
    let digit_regex = Regex::new(r"\d+").ok()?;
    let digits: Vec<u32> = digit_regex
        .find_iter(image_size_str)
        .filter_map(|m| m.as_str().parse::<u32>().ok())
        .collect();

    // Need at least 2 numbers (width and height)
    if digits.len() < 2 {
        return None;
    }

    // ExifTool: $d[0] * $d[1] / 1000000
    let width = digits[0] as f64;
    let height = digits[1] as f64;
    let megapixels = (width * height) / 1_000_000.0;

    Some(TagValue::F64(megapixels))
}

/// Compute GPSPosition composite tag from GPSLatitude and GPSLongitude
/// ExifTool: lib/Image/ExifTool/Composite.pm GPSPosition definition
/// ValueConv: (length($val[0]) or length($val[1])) ? "$val[0] $val[1]" : undef
pub fn compute_gps_position(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    let gps_latitude = available_tags.get("GPSLatitude");
    let gps_longitude = available_tags.get("GPSLongitude");

    // Trust ExifTool: if either latitude or longitude has content (length > 0), combine them
    let lat_str = gps_latitude.and_then(|v| v.as_string()).unwrap_or_default();
    let lon_str = gps_longitude
        .and_then(|v| v.as_string())
        .unwrap_or_default();

    // ExifTool: (length($val[0]) or length($val[1])) ? "$val[0] $val[1]" : undef
    if !lat_str.is_empty() || !lon_str.is_empty() {
        Some(TagValue::string(format!("{lat_str} {lon_str}")))
    } else {
        None
    }
}

/// Compute HyperfocalDistance composite tag
/// ExifTool: lib/Image/ExifTool/Composite.pm HyperfocalDistance definition  
/// ValueConv: ToFloat(@val); return 'inf' unless $val[1] and $val[2]; return $val[0] * $val[0] / ($val[1] * $val[2] * 1000);
pub fn compute_hyperfocal_distance(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // Require FocalLength (index 0)
    let focal_length = available_tags.get("FocalLength")?.as_f64()?;

    // Require Aperture (index 1)
    let aperture = available_tags.get("Aperture")?.as_f64()?;

    // Require CircleOfConfusion (index 2)
    let circle_of_confusion = available_tags.get("CircleOfConfusion")?.as_f64()?;

    // ExifTool: return 'inf' unless $val[1] and $val[2]
    if aperture == 0.0 || circle_of_confusion == 0.0 {
        return Some("inf".into());
    }

    // ExifTool: $val[0] * $val[0] / ($val[1] * $val[2] * 1000)
    let hyperfocal_distance =
        (focal_length * focal_length) / (aperture * circle_of_confusion * 1000.0);

    Some(TagValue::F64(hyperfocal_distance))
}

/// Compute FOV (Field of View) composite tag
/// ExifTool: lib/Image/ExifTool/Composite.pm FOV definition
/// Complex trigonometric calculation with focus distance correction
pub fn compute_fov(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // Require FocalLength (index 0) and ScaleFactor35efl (index 1)
    let focal_length = available_tags.get("FocalLength")?.as_f64()?;
    let scale_factor = available_tags.get("ScaleFactor35efl")?.as_f64()?;

    // ExifTool: return undef unless $val[0] and $val[1]
    if focal_length == 0.0 || scale_factor == 0.0 {
        return None;
    }

    // Focus distance correction (optional index 2)
    let focus_distance = available_tags.get("FocusDistance").and_then(|v| v.as_f64());

    // ExifTool: my $corr = 1;
    let mut corr = 1.0;

    // ExifTool focus distance correction logic
    if let Some(focus_dist) = focus_distance {
        if focus_dist > 0.0 {
            // ExifTool: my $d = 1000 * $val[2] - $val[0]; $corr += $val[0]/$d if $d > 0;
            let d = 1000.0 * focus_dist - focal_length;
            if d > 0.0 {
                corr += focal_length / d;
            }
        }
    }

    // ExifTool: my $fd2 = atan2(36, 2*$val[0]*$val[1]*$corr);
    let fd2 = (36.0 / (2.0 * focal_length * scale_factor * corr)).atan();

    // ExifTool: my @fov = ( $fd2 * 360 / 3.14159 );
    let fov_angle = fd2 * 360.0 / std::f64::consts::PI;
    let mut fov_values = vec![fov_angle];

    // ExifTool: if ($val[2] and $val[2] > 0 and $val[2] < 10000) { push @fov, 2 * $val[2] * sin($fd2) / cos($fd2); }
    if let Some(focus_dist) = focus_distance {
        if focus_dist > 0.0 && focus_dist < 10000.0 {
            let field_width = 2.0 * focus_dist * fd2.sin() / fd2.cos();
            fov_values.push(field_width);
        }
    }

    // ExifTool: return join(' ', @fov);
    let fov_str = fov_values
        .iter()
        .map(|v| format!("{v:.1}"))
        .collect::<Vec<_>>()
        .join(" ");

    Some(TagValue::String(fov_str))
}

/// Compute DOF (Depth of Field) composite tag
/// ExifTool: lib/Image/ExifTool/Composite.pm DOF definition
/// Complex photography calculation with multiple distance fallbacks
pub fn compute_dof(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // Required: FocalLength (index 0), Aperture (index 1), CircleOfConfusion (index 2)
    let focal_length = available_tags.get("FocalLength")?.as_f64()?;
    let aperture = available_tags.get("Aperture")?.as_f64()?;
    let circle_of_confusion = available_tags.get("CircleOfConfusion")?.as_f64()?;

    // ExifTool: return 0 unless $f and $val[2];
    if focal_length == 0.0 || circle_of_confusion == 0.0 {
        return Some("0".into());
    }

    // ExifTool distance fallback logic: try multiple distance sources
    let mut distance = None;

    // First try FocusDistance (index 3)
    if let Some(focus_dist) = available_tags.get("FocusDistance").and_then(|v| v.as_f64()) {
        if focus_dist > 0.0 {
            distance = Some(focus_dist);
        } else {
            // ExifTool: $d or $d = 1e10;    # (use large number for infinity)
            distance = Some(1e10);
        }
    }

    // Fall back to other distance sources
    if distance.is_none() {
        for tag_name in &[
            "SubjectDistance",
            "ObjectDistance",
            "ApproximateFocusDistance",
        ] {
            if let Some(dist) = available_tags.get(*tag_name).and_then(|v| v.as_f64()) {
                if dist > 0.0 {
                    distance = Some(dist);
                    break;
                }
            }
        }
    }

    // ExifTool: unless (defined $d) { return undef unless defined $val[7] and defined $val[8]; $d = ($val[7] + $val[8]) / 2; }
    if distance.is_none() {
        let lower = available_tags
            .get("FocusDistanceLower")
            .and_then(|v| v.as_f64());
        let upper = available_tags
            .get("FocusDistanceUpper")
            .and_then(|v| v.as_f64());

        if let (Some(lower_val), Some(upper_val)) = (lower, upper) {
            distance = Some((lower_val + upper_val) / 2.0);
        } else {
            return None;
        }
    }

    let d = distance?;

    // ExifTool DOF calculation: my $t = $val[1] * $val[2] * ($d * 1000 - $f) / ($f * $f);
    let t = aperture * circle_of_confusion * (d * 1000.0 - focal_length)
        / (focal_length * focal_length);

    // ExifTool: my @v = ($d / (1 + $t), $d / (1 - $t));
    let near = d / (1.0 + t);
    let mut far = d / (1.0 - t);

    // ExifTool: $v[1] < 0 and $v[1] = 0; # 0 means 'inf'
    if far < 0.0 {
        far = 0.0; // 0 means infinity in ExifTool
    }

    // ExifTool: return join(' ',@v);
    Some(TagValue::string(format!("{near:.3} {far:.3}")))
}
