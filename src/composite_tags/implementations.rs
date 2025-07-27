//! Individual composite tag computation implementations
//!
//! This module contains the specific computation functions for each composite tag,
//! translating ExifTool's Perl ValueConv expressions to Rust.

use regex::Regex;
use std::collections::HashMap;
use tracing::trace;

use crate::types::TagValue;

/// Compute ImageSize composite (ImageWidth + ImageHeight) - ValueConv only
/// ExifTool: lib/Image/ExifTool/Exif.pm:4641-4660 ImageSize definition
/// ValueConv: return $val[4] if $val[4]; return "$val[2] $val[3]" if $val[2] and $val[3] and $$self{TIFF_TYPE} =~ /^(CR2|Canon 1D RAW|IIQ|EIP)$/; return "$val[0] $val[1]" if IsFloat($val[0]) and IsFloat($val[1]); return undef;
/// PrintConv: '$val =~ tr/ /x/; $val' (handled separately in imagesize_print_conv)
pub fn compute_image_size(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // Check RawImageCroppedSize first (index 4 in desire list)
    if let Some(raw_size) = available_tags.get("RawImageCroppedSize") {
        return Some(raw_size.clone());
    }

    // ExifTool logic: Only use ExifImageWidth/Height for Canon and Phase One RAW formats
    // TIFF_TYPE =~ /^(CR2|Canon 1D RAW|IIQ|EIP)$/
    let use_exif_dimensions = is_canon_raw_tiff_type(available_tags);

    if use_exif_dimensions {
        // Try ExifImageWidth/ExifImageHeight for Canon/Phase One RAW (indexes 2,3)
        if let (Some(width), Some(height)) = (
            available_tags.get("EXIF:ExifImageWidth"),
            available_tags.get("EXIF:ExifImageHeight"),
        ) {
            if let (Some(w), Some(h)) = (width.as_u32(), height.as_u32()) {
                return Some(TagValue::string(format!("{w} {h}"))); // ValueConv: space separator
            }
        }
    }

    // Priority 3: Use the best available ImageWidth/ImageHeight values
    // This calls our composite computation functions directly to get dimensions
    // including Panasonic sensor border calculations, EXIF values, etc.
    if let (Some(width), Some(height)) = (
        compute_image_width(available_tags),
        compute_image_height(available_tags),
    ) {
        if let (Some(w), Some(h)) = (width.as_u32(), height.as_u32()) {
            return Some(TagValue::string(format!("{w} {h}"))); // ValueConv: space separator
        }
    }

    None
}

/// Check if TIFF_TYPE matches Canon/Phase One RAW formats that prefer ExifImageWidth/Height
/// ExifTool: lib/Image/ExifTool/Exif.pm:4655 TIFF_TYPE =~ /^(CR2|Canon 1D RAW|IIQ|EIP)$/
/// For now, we detect this by looking at file type indicators in available tags
fn is_canon_raw_tiff_type(available_tags: &HashMap<String, TagValue>) -> bool {
    // Check for File:FileType tag which should indicate RAW format
    if let Some(file_type) = available_tags.get("File:FileType") {
        // Match ExifTool's TIFF_TYPE regex: /^(CR2|Canon 1D RAW|IIQ|EIP)$/
        if let Some("CR2" | "Canon 1D RAW" | "IIQ" | "EIP") = file_type.as_string() {
            return true;
        }
    }

    // Fallback: Check File:FileTypeExtension
    if let Some(extension) = available_tags.get("File:FileTypeExtension") {
        if let Some(ext_str) = extension.as_string() {
            return ext_str.to_uppercase() == "CR2" || ext_str.to_uppercase() == "IIQ";
        }
    }

    false
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
    // Note: focal_length squared is correct per ExifTool's formula
    #[allow(clippy::suspicious_operation_groupings)]
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

/// Compute ISO composite tag by consolidating multiple ISO sources
/// This creates a unified ISO value from various manufacturer-specific and standard sources
/// Priority: ISO > ISOSpeed > ISOSpeedRatings[0] > PhotographicSensitivity > Manufacturer-specific
/// ExifTool: lib/Image/ExifTool/Canon.pm:9792-9806 (Canon ISO composite algorithm)
/// ExifTool: lib/Image/ExifTool/Exif.pm:2116-2124 (standard EXIF ISO tag)
/// exif-oxide specific: Unlike ExifTool, we provide consolidated ISO for user convenience
pub fn compute_iso(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // Priority 1: Standard EXIF ISO tag
    if let Some(iso) = available_tags.get("ISO") {
        if let Some(iso_val) = get_numeric_iso_value(iso) {
            return Some(TagValue::U32(iso_val));
        }
    }

    // Priority 2: ISOSpeed (EXIF 2.3 standard)
    if let Some(iso_speed) = available_tags.get("ISOSpeed") {
        if let Some(iso_val) = get_numeric_iso_value(iso_speed) {
            return Some(TagValue::U32(iso_val));
        }
    }

    // Priority 3: ISOSpeedRatings (older EXIF 2.2 name, array format)
    if let Some(iso_ratings) = available_tags.get("ISOSpeedRatings") {
        if let Some(iso_val) = get_numeric_iso_value(iso_ratings) {
            return Some(TagValue::U32(iso_val));
        }
    }

    // Priority 4: PhotographicSensitivity (EXIF 2.3 name)
    if let Some(photo_sens) = available_tags.get("PhotographicSensitivity") {
        if let Some(iso_val) = get_numeric_iso_value(photo_sens) {
            return Some(TagValue::U32(iso_val));
        }
    }

    // Priority 5: Manufacturer-specific ISO tags
    // Canon: Check CameraISO, BaseISO, AutoISO (following Canon.pm algorithm)
    if let Some(camera_iso) = available_tags.get("CameraISO") {
        if let Some(iso_val) = get_numeric_iso_value(camera_iso) {
            return Some(TagValue::U32(iso_val));
        }
    }

    // Canon fallback: BaseISO * AutoISO / 100
    // ExifTool: lib/Image/ExifTool/Canon.pm:9792-9806 ValueConv logic
    if let (Some(base_iso), Some(auto_iso)) =
        (available_tags.get("BaseISO"), available_tags.get("AutoISO"))
    {
        if let (Some(base), Some(auto)) = (
            get_numeric_iso_value(base_iso),
            get_numeric_iso_value(auto_iso),
        ) {
            let calculated_iso = (base as f64 * auto as f64 / 100.0).round() as u32;
            return Some(TagValue::U32(calculated_iso));
        }
    }

    // Sony: SonyISO (multiple possible sources)
    if let Some(sony_iso) = available_tags.get("SonyISO") {
        if let Some(iso_val) = get_numeric_iso_value(sony_iso) {
            return Some(TagValue::U32(iso_val));
        }
    }

    // Nikon: Various Nikon ISO sources (may require decryption in future)
    for nikon_tag in &["NikonISO", "ISOInfo"] {
        if let Some(nikon_iso) = available_tags.get(*nikon_tag) {
            if let Some(iso_val) = get_numeric_iso_value(nikon_iso) {
                return Some(TagValue::U32(iso_val));
            }
        }
    }

    None
}

/// Extract numeric ISO value from various TagValue formats
/// Handles arrays (taking first element), rationals, and different numeric types
fn get_numeric_iso_value(tag_value: &TagValue) -> Option<u32> {
    match tag_value {
        TagValue::U32(val) => Some(*val),
        TagValue::U16(val) => Some(*val as u32),
        TagValue::U8(val) => Some(*val as u32),
        TagValue::I32(val) if *val > 0 => Some(*val as u32),
        TagValue::I16(val) if *val > 0 => Some(*val as u32),
        TagValue::F64(val) if *val > 0.0 => Some(val.round() as u32),
        TagValue::Rational(num, den) if *den != 0 && *num > 0 => {
            Some((*num as f64 / *den as f64).round() as u32)
        }
        TagValue::String(s) => {
            // Handle string representations like "100" or arrays like "100, 200"
            let first_number = s.split(',').next()?.trim();
            Some(first_number.parse::<f64>().ok()?.round() as u32)
        }
        _ => None,
    }
    .filter(|&val| val > 0 && val <= 100_000) // Reasonable ISO range validation
}

/// Compute ImageWidth composite tag from various image dimension sources
/// Priority: SubIFD3:ImageWidth > IFD0:ImageWidth > ExifIFD:ExifImageWidth
/// Compute ImageWidth composite with Panasonic sensor border support
/// ExifTool: lib/Image/ExifTool/PanasonicRaw.pm:676-681 for Panasonic sensor borders
/// Plus general dimension source consolidation per P12 requirements
pub fn compute_image_width(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // Priority 1: Panasonic sensor border calculation (when available)
    // ExifTool: SensorRightBorder - SensorLeftBorder
    if let Some(panasonic_width) = compute_panasonic_image_width(available_tags) {
        return Some(panasonic_width);
    }

    // Priority 2: SubIFD3:ImageWidth (full resolution)
    if let Some(width) = available_tags.get("SubIFD3:ImageWidth") {
        if let Some(w) = width.as_u32() {
            return Some(TagValue::U32(w));
        }
    }

    // Priority 3: IFD0:ImageWidth
    if let Some(width) = available_tags.get("ImageWidth") {
        if let Some(w) = width.as_u32() {
            return Some(TagValue::U32(w));
        }
    }

    // Priority 4: ExifIFD:ExifImageWidth
    if let Some(width) = available_tags.get("ExifImageWidth") {
        if let Some(w) = width.as_u32() {
            return Some(TagValue::U32(w));
        }
    }

    // Try without group prefixes as fallback
    for tag_name in &["ImageWidth", "ExifImageWidth"] {
        if let Some(width) = available_tags.get(*tag_name) {
            if let Some(w) = width.as_u32() {
                return Some(TagValue::U32(w));
            }
        }
    }

    None
}

/// Compute ImageHeight composite tag from various image dimension sources
/// Priority: SubIFD3:ImageHeight > IFD0:ImageHeight > ExifIFD:ExifImageHeight
/// Compute ImageHeight composite with Panasonic sensor border support
/// ExifTool: lib/Image/ExifTool/PanasonicRaw.pm:682-687 for Panasonic sensor borders  
/// Plus general dimension source consolidation per P12 requirements
pub fn compute_image_height(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // Priority 1: Panasonic sensor border calculation (when available)
    // ExifTool: SensorBottomBorder - SensorTopBorder
    if let Some(panasonic_height) = compute_panasonic_image_height(available_tags) {
        return Some(panasonic_height);
    }

    // Priority 2: SubIFD3:ImageHeight (full resolution)
    if let Some(height) = available_tags.get("SubIFD3:ImageHeight") {
        if let Some(h) = height.as_u32() {
            return Some(TagValue::U32(h));
        }
    }

    // Priority 3: IFD0:ImageHeight
    if let Some(height) = available_tags.get("ImageHeight") {
        if let Some(h) = height.as_u32() {
            return Some(TagValue::U32(h));
        }
    }

    // Priority 4: ExifIFD:ExifImageHeight
    if let Some(height) = available_tags.get("ExifImageHeight") {
        if let Some(h) = height.as_u32() {
            return Some(TagValue::U32(h));
        }
    }

    // Try without group prefixes as fallback
    for tag_name in &["ImageHeight", "ExifImageHeight"] {
        if let Some(height) = available_tags.get(*tag_name) {
            if let Some(h) = height.as_u32() {
                return Some(TagValue::U32(h));
            }
        }
    }

    None
}

/// Compute Rotation composite tag from Orientation tag
/// ExifTool: lib/Image/ExifTool/Composite.pm:435-443 Rotation definition
/// Converts EXIF Orientation values (1-8) to rotation angles in degrees
pub fn compute_rotation(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    let orientation = available_tags.get("Orientation")?;
    let orientation_val = orientation.as_u8()?;

    // ExifTool: lib/Image/ExifTool/Composite.pm Rotation ValueConv
    // Orientation values 1-8 correspond to different rotations and flips
    let rotation = match orientation_val {
        1 => 0,           // Normal
        2 => 0,           // Mirror horizontal
        3 => 180,         // Rotate 180
        4 => 180,         // Mirror vertical
        5 => 90,          // Mirror horizontal and rotate 270 CW
        6 => 90,          // Rotate 90 CW
        7 => 270,         // Mirror horizontal and rotate 90 CW
        8 => 270,         // Rotate 270 CW
        _ => return None, // Invalid orientation value
    };

    Some(TagValue::U16(rotation))
}

/// Compute GPSDateTime composite tag from GPSDateStamp and GPSTimeStamp
/// ExifTool: lib/Image/ExifTool/GPS.pm:355-365 GPSDateTime definition
/// ValueConv: "$val[0] $val[1]Z" - concatenates date + space + time + Z for UTC
pub fn compute_gps_datetime(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // ExifTool requires both GPS:GPSDateStamp and GPS:GPSTimeStamp
    let date_stamp = available_tags.get("GPSDateStamp")?;
    let time_stamp = available_tags.get("GPSTimeStamp")?;

    let date_str = date_stamp.as_string()?;
    let time_str = time_stamp.as_string()?;

    // ExifTool ValueConv: "$val[0] $val[1]Z"
    // Concatenate date, space, time, and "Z" (UTC timezone indicator)
    Some(TagValue::string(format!("{date_str} {time_str}Z")))
}

/// Compute GPSLatitude composite tag with signed decimal degrees
/// ExifTool: lib/Image/ExifTool/GPS.pm:368-384 GPSLatitude definition
/// ValueConv: '$val[1] =~ /^S/i ? -$val[0] : $val[0]' - negates if South reference
pub fn compute_gps_latitude(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // ExifTool requires both GPS:GPSLatitude and GPS:GPSLatitudeRef
    let latitude = available_tags.get("GPSLatitude")?;
    let latitude_ref = available_tags.get("GPSLatitudeRef")?;

    let lat_value = latitude.as_f64()?;
    let ref_str = latitude_ref.as_string()?;

    // ExifTool ValueConv: '$val[1] =~ /^S/i ? -$val[0] : $val[0]'
    // If reference starts with 'S' (case-insensitive), negate the latitude
    let signed_latitude = if ref_str.to_lowercase().starts_with('s') {
        -lat_value
    } else {
        lat_value
    };

    Some(TagValue::F64(signed_latitude))
}

/// Compute GPSLongitude composite tag with signed decimal degrees  
/// ExifTool: lib/Image/ExifTool/GPS.pm:385-405 GPSLongitude definition
/// ValueConv: '$val[1] =~ /^W/i ? -$val[0] : $val[0]' - negates if West reference
pub fn compute_gps_longitude(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // ExifTool requires both GPS:GPSLongitude and GPS:GPSLongitudeRef
    let longitude = available_tags.get("GPSLongitude")?;
    let longitude_ref = available_tags.get("GPSLongitudeRef")?;

    let lon_value = longitude.as_f64()?;
    let ref_str = longitude_ref.as_string()?;

    // ExifTool ValueConv: '$val[1] =~ /^W/i ? -$val[0] : $val[0]'
    // If reference starts with 'W' (case-insensitive), negate the longitude
    let signed_longitude = if ref_str.to_lowercase().starts_with('w') {
        -lon_value
    } else {
        lon_value
    };

    Some(TagValue::F64(signed_longitude))
}

/// Compute SubSecCreateDate composite tag from EXIF:CreateDate with subseconds and timezone
/// ExifTool: lib/Image/ExifTool/Exif.pm:5077-5095 SubSecCreateDate definition
/// Uses %subSecConv logic from lib/Image/ExifTool/Exif.pm:4620-4636
pub fn compute_subsec_create_date(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // ExifTool requires EXIF:CreateDate
    let create_date = available_tags.get("CreateDate")?;
    let create_date_str = create_date.as_string()?;

    // Apply SubSec conversion logic from ExifTool
    let result = apply_subsec_conversion(
        &create_date_str,
        available_tags.get("SubSecTimeDigitized"),
        available_tags.get("OffsetTimeDigitized"),
    );

    Some(TagValue::string(result))
}

/// Compute SubSecModifyDate composite tag from EXIF:ModifyDate with subseconds and timezone
/// ExifTool: lib/Image/ExifTool/Exif.pm:5096-5114 SubSecModifyDate definition  
/// Uses %subSecConv logic from lib/Image/ExifTool/Exif.pm:4620-4636
pub fn compute_subsec_modify_date(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // ExifTool requires EXIF:ModifyDate
    let modify_date = available_tags.get("ModifyDate")?;
    let modify_date_str = modify_date.as_string()?;

    // Apply SubSec conversion logic from ExifTool
    let result = apply_subsec_conversion(
        &modify_date_str,
        available_tags.get("SubSecTime"),
        available_tags.get("OffsetTime"),
    );

    Some(TagValue::string(result))
}

/// Compute SubSecMediaCreateDate composite tag for media files
/// Note: ExifTool doesn't have this as a standard composite - media uses integer timestamps
/// This provides compatibility for media creation dates with subsecond precision
pub fn compute_subsec_media_create_date(
    available_tags: &HashMap<String, TagValue>,
) -> Option<TagValue> {
    // Try QuickTime MediaCreateDate first
    if let Some(media_create_date) = available_tags.get("MediaCreateDate") {
        let media_date_str = media_create_date.as_string()?;

        // Apply subsec conversion if subsecond data is available
        let result = apply_subsec_conversion(
            &media_date_str,
            available_tags.get("SubSecTime"),
            available_tags.get("OffsetTime"),
        );

        return Some(TagValue::string(result));
    }

    // Fallback to CreateDate for other media types
    if let Some(create_date) = available_tags.get("CreateDate") {
        let create_date_str = create_date.as_string()?;

        let result = apply_subsec_conversion(
            &create_date_str,
            available_tags.get("SubSecTimeDigitized"),
            available_tags.get("OffsetTimeDigitized"),
        );

        return Some(TagValue::string(result));
    }

    None
}

/// Apply ExifTool's SubSec conversion logic to combine datetime with subseconds and timezone
/// ExifTool: lib/Image/ExifTool/Exif.pm:4620-4636 %subSecConv hash RawConv logic
fn apply_subsec_conversion(
    base_datetime: &str,
    subsec_time: Option<&TagValue>,
    offset_time: Option<&TagValue>,
) -> String {
    let mut result = base_datetime.to_string();

    // Phase 1: Add subseconds if available and not already present
    // ExifTool: undef $v unless ($v = $val[0]) =~ s/( \d{2}:\d{2}:\d{2})(?!\.\d+)/$1\.$subSec/;
    if let Some(subsec) = subsec_time {
        if let Some(subsec_str) = subsec.as_string() {
            // Extract numeric digits from subsec field (ExifTool: $val[1]=~/^(\d+)/)
            if let Some(digits) = extract_numeric_digits(&subsec_str) {
                // Use negative lookahead equivalent: only add if time doesn't already have subseconds
                if !result.contains('.') && result.contains(':') {
                    // Find the time pattern " HH:MM:SS" and append subseconds
                    if let Some(time_pos) = result.rfind(' ') {
                        let (date_part, time_part) = result.split_at(time_pos + 1);
                        if time_part.len() == 8 && time_part.matches(':').count() == 2 {
                            result = format!("{}{}.{}", date_part, time_part, digits);
                        }
                    }
                }
            }
        }
    }

    // Phase 2: Add timezone offset if available and not already present
    // ExifTool: if (defined $val[2] and $val[0]!~/[-+]/ and $val[2]=~/^([-+])(\d{1,2}):(\d{2})/)
    if let Some(offset) = offset_time {
        if let Some(offset_str) = offset.as_string() {
            // Only add timezone if result doesn't already contain +/-
            if !result.contains('+') && !result.contains('-') {
                // Parse timezone format: [-+]HH:MM
                if let Some(formatted_offset) = format_timezone_offset(&offset_str) {
                    result.push_str(&formatted_offset);
                }
            }
        }
    }

    result
}

/// Extract numeric digits from subsecond time field
/// ExifTool: $val[1]=~/^(\d+)/
fn extract_numeric_digits(subsec_str: &str) -> Option<String> {
    // Find the first sequence of digits
    let mut digits = String::new();
    for ch in subsec_str.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
        } else if !digits.is_empty() {
            break; // Stop at first non-digit after finding digits
        }
    }

    if digits.is_empty() {
        None
    } else {
        Some(digits)
    }
}

/// Format timezone offset according to ExifTool's sprintf pattern
/// ExifTool: sprintf('%s%.2d:%.2d', $1, $2, $3) for pattern ^([-+])(\d{1,2}):(\d{2})
fn format_timezone_offset(offset_str: &str) -> Option<String> {
    // Handle "Z" timezone (UTC)
    if offset_str.trim() == "Z" {
        return Some("+00:00".to_string());
    }

    // Parse [-+]HH:MM format (allowing 1-2 digit hours)
    if let Some(captures) = regex::Regex::new(r"^([-+])(\d{1,2}):(\d{2})")
        .ok()?
        .captures(offset_str.trim())
    {
        let sign = &captures[1];
        let hours: i32 = captures[2].parse().ok()?;
        let minutes: i32 = captures[3].parse().ok()?;

        // Format with proper zero-padding (ExifTool sprintf format)
        Some(format!("{}{:02}:{:02}", sign, hours, minutes))
    } else {
        None
    }
}

/// Compute Lens composite tag - full lens description from manufacturer databases
/// ExifTool: lib/Image/ExifTool/Canon.pm:9684-9691 Canon Lens composite
/// ValueConv: $val[0] (returns first value from MinFocalLength)
/// PrintConv: Image::ExifTool::Canon::PrintFocalRange(@val)
pub fn compute_lens(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // Canon-specific implementation: uses MinFocalLength, MaxFocalLength
    // ExifTool: lib/Image/ExifTool/Canon.pm:9684-9691
    if let (Some(min_focal), Some(max_focal)) = (
        available_tags.get("MinFocalLength"),
        available_tags.get("MaxFocalLength"),
    ) {
        if let (Some(min_f), Some(max_f)) = (min_focal.as_f64(), max_focal.as_f64()) {
            // ExifTool PrintFocalRange function logic (Canon.pm:10212-10222)
            let lens_desc = if (min_f - max_f).abs() < 0.1 {
                // Prime lens: single focal length
                format!("{:.1} mm", min_f)
            } else {
                // Zoom lens: focal length range
                format!("{:.1} - {:.1} mm", min_f, max_f)
            };
            return Some(TagValue::string(lens_desc));
        }
    }

    // Fallback: try LensModel directly
    if let Some(lens_model) = available_tags.get("LensModel") {
        return Some(lens_model.clone());
    }

    // Fallback: try generic Lens tag
    if let Some(lens) = available_tags.get("Lens") {
        return Some(lens.clone());
    }

    None
}

/// Compute LensID composite tag - sophisticated lens identification from manufacturer data
/// ExifTool: lib/Image/ExifTool/Exif.pm:5197-5255 primary LensID implementation
/// Complex algorithm using PrintLensID function (lines 5775-5954)
pub fn compute_lens_id(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // Note: Full ExifTool LensID implementation is extremely complex (180+ lines)
    // involving manufacturer lens databases, adapter detection, teleconverter handling
    // This is a simplified version focusing on the most common cases

    // Primary: try LensType (required dependency)
    if let Some(lens_type) = available_tags.get("LensType") {
        // In a full implementation, this would look up lens_type in manufacturer tables
        // For now, return the LensType value directly as identifier
        return Some(lens_type.clone());
    }

    // Secondary: try XMP-aux:LensID (for XMP sources)
    // ExifTool: lib/Image/ExifTool/XMP.pm:2761-2779
    if let Some(xmp_lens_id) = available_tags.get("LensID") {
        return Some(xmp_lens_id.clone());
    }

    // Nikon-specific: construct 8-byte hex string from LensType components
    // ExifTool: lib/Image/ExifTool/Nikon.pm:13173-13189
    if let (Some(lens_type), Some(lens_info)) = (
        available_tags.get("LensType"),
        available_tags.get("LensInfo"),
    ) {
        // This would construct Nikon's 8-byte lens identifier
        // Format example: "0x123456789ABCDEF0"
        return Some(TagValue::string(format!(
            "{}:{}",
            lens_type.as_string().unwrap_or_default(),
            lens_info.as_string().unwrap_or_default()
        )));
    }

    None
}

/// Compute LensSpec composite tag - formatted lens specification
/// ExifTool: lib/Image/ExifTool/Nikon.pm:13165-13172 Nikon LensSpec
/// ValueConv: "$val[0] $val[1]" (concatenates Lens and LensType)
pub fn compute_lens_spec(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // Nikon implementation: combines Lens + LensType
    // ExifTool: lib/Image/ExifTool/Nikon.pm:13165-13172
    if let (Some(lens), Some(lens_type)) =
        (available_tags.get("Lens"), available_tags.get("LensType"))
    {
        let lens_str = lens.as_string().unwrap_or_default();
        let type_str = lens_type.as_string().unwrap_or_default();
        return Some(TagValue::string(format!("{} {}", lens_str, type_str)));
    }

    // Fallback: construct from focal length and aperture information
    if let Some(lens_info) = available_tags.get("LensInfo") {
        return Some(lens_info.clone());
    }

    // Generic construction from available lens parameters
    let mut spec_parts = Vec::new();

    // Add focal length information
    match (
        available_tags.get("MinFocalLength"),
        available_tags.get("MaxFocalLength"),
    ) {
        (Some(min_f), Some(max_f)) => {
            if let (Some(min_fl), Some(max_fl)) = (min_f.as_f64(), max_f.as_f64()) {
                if (min_fl - max_fl).abs() < 0.1 {
                    spec_parts.push(format!("{}mm", min_fl as u32));
                } else {
                    spec_parts.push(format!("{}-{}mm", min_fl as u32, max_fl as u32));
                }
            }
        }
        _ => {
            if let Some(focal_length) = available_tags.get("FocalLength") {
                if let Some(fl) = focal_length.as_f64() {
                    spec_parts.push(format!("{}mm", fl as u32));
                }
            }
        }
    }

    // Add aperture information
    if let Some(max_aperture) = available_tags.get("MaxAperture") {
        if let Some(ap) = max_aperture.as_f64() {
            spec_parts.push(format!("f/{:.1}", ap));
        }
    }

    if !spec_parts.is_empty() {
        Some(TagValue::string(spec_parts.join(" ")))
    } else {
        None
    }
}

/// Compute LensType composite tag - lens type from manufacturer-specific data
/// ExifTool: lib/Image/ExifTool/Olympus.pm:4290-4299 Olympus LensType
/// ValueConv: "$val[0] $val[1]" (concatenates LensTypeMake and LensTypeModel)
pub fn compute_lens_type(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // Olympus implementation: combines LensTypeMake + LensTypeModel
    // ExifTool: lib/Image/ExifTool/Olympus.pm:4290-4299
    if let (Some(make), Some(model)) = (
        available_tags.get("LensTypeMake"),
        available_tags.get("LensTypeModel"),
    ) {
        let make_str = make.as_string().unwrap_or_default();
        let model_str = model.as_string().unwrap_or_default();
        // Note: ExifTool also looks this up in %olympusLensTypes table for PrintConv
        return Some(TagValue::string(format!("{} {}", make_str, model_str)));
    }

    // Direct LensType tag (most common case)
    if let Some(lens_type) = available_tags.get("LensType") {
        return Some(lens_type.clone());
    }

    // Canon/Nikon fallbacks
    for fallback_tag in &["LensType2", "LensType3", "RFLensType"] {
        if let Some(lens_type) = available_tags.get(*fallback_tag) {
            return Some(lens_type.clone());
        }
    }

    None
}

/// Compute Duration composite tag for audio/video files
/// ExifTool: Multiple format-specific implementations in FLAC.pm, APE.pm, AIFF.pm, RIFF.pm, MPEG.pm, Vorbis.pm
/// This implements a consolidated version supporting common video formats
pub fn compute_duration(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // Method 1: RIFF-style calculation using FrameRate and FrameCount
    // ExifTool: lib/Image/ExifTool/RIFF.pm:1547-1580
    if let (Some(frame_rate), Some(frame_count)) = (
        available_tags.get("FrameRate"),
        available_tags.get("FrameCount"),
    ) {
        if let (Some(rate), Some(count)) = (frame_rate.as_f64(), frame_count.as_f64()) {
            if rate > 0.0 {
                let duration_seconds = count / rate;
                return Some(TagValue::F64(duration_seconds));
            }
        }
    }

    // Method 2: Video-specific frame calculations
    if let (Some(video_rate), Some(video_count)) = (
        available_tags.get("VideoFrameRate"),
        available_tags.get("VideoFrameCount"),
    ) {
        if let (Some(rate), Some(count)) = (video_rate.as_f64(), video_count.as_f64()) {
            if rate > 0.0 {
                let duration_seconds = count / rate;
                return Some(TagValue::F64(duration_seconds));
            }
        }
    }

    // Method 3: Audio-style calculation using SampleRate and TotalSamples
    // ExifTool: lib/Image/ExifTool/FLAC.pm:137-146, AIFF.pm:136-145
    if let (Some(sample_rate), Some(total_samples)) = (
        available_tags.get("SampleRate"),
        available_tags.get("TotalSamples"),
    ) {
        if let (Some(rate), Some(samples)) = (sample_rate.as_f64(), total_samples.as_f64()) {
            if rate > 0.0 {
                let duration_seconds = samples / rate;
                return Some(TagValue::F64(duration_seconds));
            }
        }
    }

    // Method 4: Bitrate-based approximation (less accurate)
    // ExifTool: lib/Image/ExifTool/MPEG.pm:385-415, Vorbis.pm:138-147
    if let (Some(file_size), Some(bitrate)) = (
        available_tags.get("FileSize"),
        available_tags
            .get("AudioBitrate")
            .or_else(|| available_tags.get("VideoBitrate")),
    ) {
        if let (Some(size), Some(rate)) = (file_size.as_f64(), bitrate.as_f64()) {
            if rate > 0.0 {
                // Duration = (FileSize in bits) / bitrate
                let duration_seconds = (size * 8.0) / rate;
                return Some(TagValue::F64(duration_seconds));
            }
        }
    }

    None
}

/// Compute Panasonic ImageWidth composite from sensor borders
/// ExifTool: lib/Image/ExifTool/PanasonicRaw.pm:676-681
/// ValueConv: '$val[1] - $val[0]' where val[0]=SensorLeftBorder, val[1]=SensorRightBorder
pub fn compute_panasonic_image_width(
    available_tags: &HashMap<String, TagValue>,
) -> Option<TagValue> {
    if let (Some(left), Some(right)) = (
        available_tags
            .get("EXIF:SensorLeftBorder")
            .or_else(|| available_tags.get("SensorLeftBorder")),
        available_tags
            .get("EXIF:SensorRightBorder")
            .or_else(|| available_tags.get("SensorRightBorder")),
    ) {
        if let (Some(left_val), Some(right_val)) = (left.as_u32(), right.as_u32()) {
            let width = right_val - left_val;
            return Some(TagValue::U32(width));
        }
    }
    None
}

/// Compute Panasonic ImageHeight composite from sensor borders  
/// ExifTool: lib/Image/ExifTool/PanasonicRaw.pm:682-687
/// ValueConv: '$val[1] - $val[0]' where val[0]=SensorTopBorder, val[1]=SensorBottomBorder
pub fn compute_panasonic_image_height(
    available_tags: &HashMap<String, TagValue>,
) -> Option<TagValue> {
    if let (Some(top), Some(bottom)) = (
        available_tags
            .get("EXIF:SensorTopBorder")
            .or_else(|| available_tags.get("SensorTopBorder")),
        available_tags
            .get("EXIF:SensorBottomBorder")
            .or_else(|| available_tags.get("SensorBottomBorder")),
    ) {
        if let (Some(top_val), Some(bottom_val)) = (top.as_u32(), bottom.as_u32()) {
            let height = bottom_val - top_val;
            return Some(TagValue::U32(height));
        }
    }
    None
}
