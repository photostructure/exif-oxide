//! Manual fallback implementations for composite tags
//!
//! This module contains composite tag computations that cannot be auto-generated
//! by the PPI pipeline. These are used when:
//! 1. The Perl expression is too complex for automatic translation
//! 2. The expression uses ExifTool-specific features like `$$self{TIFF_TYPE}`
//! 3. The expression requires manufacturer-specific lookup tables
//!
//! ## Design
//!
//! Each fallback function has two variants:
//! - `compute_*` - HashMap-based lookup for orchestration.rs compatibility
//! - `composite_*` - Array-based signature matching `CompositeValueConvFn`
//!
//! The `COMPOSITE_FALLBACKS` registry maps composite names to array-based
//! functions for codegen to emit function pointers when PPI fails.
//!
//! ## References
//!
//! - TRUST-EXIFTOOL.md: Core principle - translate ExifTool EXACTLY
//! - P03c-composite-tags.md: Task 6 specification
//! - ExifTool source: lib/Image/ExifTool.pm, lib/Image/ExifTool/Exif.pm

use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;
use tracing::trace;

use crate::core::types::{ExifContext, ExifError, Result};
use crate::core::TagValue;

// =============================================================================
// COMPOSITE_FALLBACKS Registry
// =============================================================================
//
// This registry maps composite tag names to fallback functions with the
// CompositeValueConvFn signature. Codegen queries this registry when PPI
// fails to translate an expression.

/// Type alias for composite fallback functions
pub type CompositeFallbackFn =
    fn(&[TagValue], &[TagValue], &[TagValue], Option<&ExifContext>) -> Result<TagValue>;

/// Registry of manually-implemented composite calculations
///
/// Each entry replaces a PPI-untranslatable expression with a Rust function.
/// Codegen emits these as function pointers when PPI generation fails.
///
/// IMPORTANT: These are PLACEHOLDERS. As PPI capabilities expand,
/// move implementations to generated code.
pub static COMPOSITE_FALLBACKS: LazyLock<HashMap<&'static str, CompositeFallbackFn>> =
    LazyLock::new(|| {
        HashMap::from([
            // Complex composites that PPI cannot translate
            ("ImageSize", composite_image_size as CompositeFallbackFn),
            ("Megapixels", composite_megapixels as CompositeFallbackFn),
            ("LensID", composite_lens_id as CompositeFallbackFn),
            // GPS composites - need sign conversion from ref
            ("GPSLatitude", composite_gps_latitude as CompositeFallbackFn),
            (
                "GPSLongitude",
                composite_gps_longitude as CompositeFallbackFn,
            ),
            ("GPSAltitude", composite_gps_altitude as CompositeFallbackFn),
            ("GPSPosition", composite_gps_position as CompositeFallbackFn),
            ("GPSDateTime", composite_gps_datetime as CompositeFallbackFn),
            // DateTime composites - need subsec/offset handling
            (
                "SubSecDateTimeOriginal",
                composite_subsec_datetime_original as CompositeFallbackFn,
            ),
            (
                "SubSecCreateDate",
                composite_subsec_create_date as CompositeFallbackFn,
            ),
            (
                "SubSecModifyDate",
                composite_subsec_modify_date as CompositeFallbackFn,
            ),
            (
                "DateTimeCreated",
                composite_datetime_created as CompositeFallbackFn,
            ),
            // Lens system - complex lookup tables
            ("Lens", composite_lens as CompositeFallbackFn),
            ("LensSpec", composite_lens_spec as CompositeFallbackFn),
            ("LensType", composite_lens_type as CompositeFallbackFn),
            // Photography calculations
            (
                "ShutterSpeed",
                composite_shutter_speed as CompositeFallbackFn,
            ),
            ("Aperture", composite_aperture as CompositeFallbackFn),
            ("ISO", composite_iso as CompositeFallbackFn),
            (
                "ScaleFactor35efl",
                composite_scale_factor_35efl as CompositeFallbackFn,
            ),
            (
                "FocalLength35efl",
                composite_focal_length_35efl as CompositeFallbackFn,
            ),
            (
                "CircleOfConfusion",
                composite_circle_of_confusion as CompositeFallbackFn,
            ),
            (
                "HyperfocalDistance",
                composite_hyperfocal_distance as CompositeFallbackFn,
            ),
            ("DOF", composite_dof as CompositeFallbackFn),
            ("FOV", composite_fov as CompositeFallbackFn),
            ("LightValue", composite_light_value as CompositeFallbackFn),
            // Image dimensions
            ("ImageWidth", composite_image_width as CompositeFallbackFn),
            ("ImageHeight", composite_image_height as CompositeFallbackFn),
            ("Rotation", composite_rotation as CompositeFallbackFn),
            // Preview/Thumbnail
            (
                "PreviewImageSize",
                composite_preview_image_size as CompositeFallbackFn,
            ),
            (
                "ThumbnailImage",
                composite_thumbnail_image as CompositeFallbackFn,
            ),
            (
                "PreviewImage",
                composite_preview_image as CompositeFallbackFn,
            ),
            // Date/Time
            (
                "DateTimeOriginal",
                composite_datetime_original as CompositeFallbackFn,
            ),
            // Media
            ("Duration", composite_duration as CompositeFallbackFn),
        ])
    });

// =============================================================================
// Array-Based Fallback Functions (CompositeValueConvFn signature)
// =============================================================================
//
// These functions receive dependency arrays (vals, prts, raws) like generated
// functions do. They delegate to the HashMap-based implementations after
// building an available_tags map from the arrays.
//
// The dependency indices are based on the composite tag definitions in ExifTool.
// For example, ImageSize has:
//   Desire: [0]=ImageWidth, [1]=ImageHeight, [2]=ExifImageWidth, [3]=ExifImageHeight, [4]=RawImageCroppedSize

/// ImageSize composite - uses $$self{TIFF_TYPE} context
/// PLACEHOLDER: lib/Image/ExifTool/Exif.pm:4641-4660
///
/// Desire indices:
/// 0: ImageWidth, 1: ImageHeight, 2: ExifImageWidth, 3: ExifImageHeight, 4: RawImageCroppedSize
pub fn composite_image_size(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    // Check RawImageCroppedSize first (index 4)
    if let Some(raw_size) = vals.get(4) {
        if !matches!(raw_size, TagValue::Empty) {
            if let Some(size_str) = raw_size.as_string() {
                let formatted = size_str.replace(' ', "x");
                return Ok(TagValue::string(formatted));
            }
            return Ok(raw_size.clone());
        }
    }

    // Try ImageWidth/ImageHeight (indices 0, 1)
    if let (Some(width), Some(height)) = (vals.first(), vals.get(1)) {
        if let (Some(w), Some(h)) = (width.as_u32(), height.as_u32()) {
            return Ok(TagValue::string(format!("{w}x{h}")));
        }
    }

    // Try ExifImageWidth/ExifImageHeight (indices 2, 3)
    if let (Some(width), Some(height)) = (vals.get(2), vals.get(3)) {
        if let (Some(w), Some(h)) = (width.as_u32(), height.as_u32()) {
            return Ok(TagValue::string(format!("{w}x{h}")));
        }
    }

    Err(ExifError::ParseError(
        "Cannot compute ImageSize: missing dimensions".to_string(),
    ))
}

/// Megapixels composite - uses regex extraction
/// PLACEHOLDER: lib/Image/ExifTool/Composite.pm
/// ValueConv: my @d = ($val =~ /\d+/g); $d[0] * $d[1] / 1000000
///
/// Require indices: 0: ImageSize
pub fn composite_megapixels(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    let image_size = vals
        .first()
        .ok_or_else(|| ExifError::ParseError("Megapixels requires ImageSize".to_string()))?;

    let image_size_str = image_size
        .as_string()
        .ok_or_else(|| ExifError::ParseError("ImageSize must be a string".to_string()))?;

    // Trust ExifTool: extract all digit sequences from the string
    let digit_regex =
        Regex::new(r"\d+").map_err(|e| ExifError::ParseError(format!("Regex error: {}", e)))?;

    let digits: Vec<u32> = digit_regex
        .find_iter(image_size_str)
        .filter_map(|m| m.as_str().parse::<u32>().ok())
        .collect();

    if digits.len() < 2 {
        return Err(ExifError::ParseError(
            "ImageSize must contain at least 2 numbers".to_string(),
        ));
    }

    // ExifTool ValueConv: $d[0] * $d[1] / 1000000
    let width = digits[0] as f64;
    let height = digits[1] as f64;
    let megapixels = (width * height) / 1_000_000.0;

    // ExifTool PrintConv: sprintf("%.*f", ($val >= 1 ? 1 : ($val >= 0.001 ? 3 : 6)), $val)
    // Adaptive precision: 1 decimal for >=1MP, 3 for >=0.001MP, 6 for tiny sensors
    let precision = if megapixels >= 1.0 {
        1
    } else if megapixels >= 0.001 {
        3
    } else {
        6
    };
    let formatted = format!("{:.prec$}", megapixels, prec = precision);
    Ok(TagValue::string(formatted))
}

/// LensID composite - complex lookup tables
/// PLACEHOLDER: lib/Image/ExifTool/Exif.pm:5197-5255
///
/// Uses manufacturer-specific lookup tables for lens identification.
pub fn composite_lens_id(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    // Try LensType first (usually index 0)
    if let Some(lens_type) = vals.first() {
        if !matches!(lens_type, TagValue::Empty) {
            return Ok(lens_type.clone());
        }
    }

    // Fallback: try secondary sources
    for val in vals.iter().skip(1) {
        if !matches!(val, TagValue::Empty) {
            return Ok(val.clone());
        }
    }

    Err(ExifError::ParseError(
        "Cannot compute LensID: no lens type available".to_string(),
    ))
}

/// GPSLatitude composite - sign from reference
/// ExifTool: lib/Image/ExifTool/GPS.pm:368-384
/// ValueConv: '$val[1] =~ /^S/i ? -$val[0] : $val[0]'
///
/// Require indices: 0: GPSLatitude, 1: GPSLatitudeRef
pub fn composite_gps_latitude(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    let latitude = vals
        .first()
        .ok_or_else(|| ExifError::ParseError("GPSLatitude requires latitude value".to_string()))?;

    let lat_ref = vals.get(1);

    let lat_value = latitude
        .as_f64()
        .ok_or_else(|| ExifError::ParseError("GPSLatitude must be numeric".to_string()))?;

    // Check if reference is South (negate latitude)
    let signed_latitude = if let Some(ref_val) = lat_ref {
        if let Some(ref_str) = ref_val.as_string() {
            if ref_str.to_lowercase().starts_with('s') {
                -lat_value
            } else {
                lat_value
            }
        } else {
            lat_value
        }
    } else {
        lat_value
    };

    Ok(TagValue::F64(signed_latitude))
}

/// GPSLongitude composite - sign from reference
/// ExifTool: lib/Image/ExifTool/GPS.pm:385-405
/// ValueConv: '$val[1] =~ /^W/i ? -$val[0] : $val[0]'
///
/// Require indices: 0: GPSLongitude, 1: GPSLongitudeRef
pub fn composite_gps_longitude(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    let longitude = vals.first().ok_or_else(|| {
        ExifError::ParseError("GPSLongitude requires longitude value".to_string())
    })?;

    let lon_ref = vals.get(1);

    let lon_value = longitude
        .as_f64()
        .ok_or_else(|| ExifError::ParseError("GPSLongitude must be numeric".to_string()))?;

    // Check if reference is West (negate longitude)
    let signed_longitude = if let Some(ref_val) = lon_ref {
        if let Some(ref_str) = ref_val.as_string() {
            if ref_str.to_lowercase().starts_with('w') {
                -lon_value
            } else {
                lon_value
            }
        } else {
            lon_value
        }
    } else {
        lon_value
    };

    Ok(TagValue::F64(signed_longitude))
}

/// GPSAltitude composite - sign from reference
/// ExifTool: lib/Image/ExifTool/GPS.pm:406-432
pub fn composite_gps_altitude(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    let altitude = vals
        .first()
        .ok_or_else(|| ExifError::ParseError("GPSAltitude requires altitude value".to_string()))?;

    let alt_ref = vals.get(1);

    let alt_value = altitude
        .as_f64()
        .ok_or_else(|| ExifError::ParseError("GPSAltitude must be numeric".to_string()))?;

    // Check reference: 0 = Above Sea Level, 1 = Below Sea Level
    let signed_altitude = if let Some(ref_val) = alt_ref {
        if let Some(ref_str) = ref_val.as_string() {
            if ref_str == "1" || ref_str.to_lowercase().contains("below") {
                -alt_value.abs()
            } else {
                alt_value.abs()
            }
        } else {
            alt_value.abs()
        }
    } else {
        alt_value.abs()
    };

    Ok(TagValue::F64(signed_altitude))
}

/// GPSPosition composite
/// ExifTool: lib/Image/ExifTool/Exif.pm:5165-5196
pub fn composite_gps_position(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    let lat = vals.first().and_then(|v| v.as_f64());
    let lon = vals.get(1).and_then(|v| v.as_f64());

    match (lat, lon) {
        (Some(lat_val), Some(lon_val)) => Ok(TagValue::string(format!("{} {}", lat_val, lon_val))),
        (Some(lat_val), None) => Ok(TagValue::string(format!("{} 0", lat_val))),
        (None, Some(lon_val)) => Ok(TagValue::string(format!("0 {}", lon_val))),
        (None, None) => Err(ExifError::ParseError(
            "GPSPosition requires at least one coordinate".to_string(),
        )),
    }
}

/// GPSDateTime composite
/// ExifTool: lib/Image/ExifTool/GPS.pm:355-365
/// ValueConv: "$val[0] $val[1]Z"
pub fn composite_gps_datetime(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    let date_stamp = vals
        .first()
        .ok_or_else(|| ExifError::ParseError("GPSDateTime requires GPSDateStamp".to_string()))?;

    let time_stamp = vals
        .get(1)
        .ok_or_else(|| ExifError::ParseError("GPSDateTime requires GPSTimeStamp".to_string()))?;

    let date_str = date_stamp
        .as_string()
        .ok_or_else(|| ExifError::ParseError("GPSDateStamp must be a string".to_string()))?;

    let time_str = time_stamp
        .as_string()
        .ok_or_else(|| ExifError::ParseError("GPSTimeStamp must be a string".to_string()))?;

    Ok(TagValue::string(format!("{} {}Z", date_str, time_str)))
}

/// SubSecDateTimeOriginal composite
/// ExifTool: lib/Image/ExifTool/Exif.pm:4894-4912
pub fn composite_subsec_datetime_original(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    let datetime = vals.first().ok_or_else(|| {
        ExifError::ParseError("SubSecDateTimeOriginal requires DateTimeOriginal".to_string())
    })?;

    let datetime_str = datetime
        .as_string()
        .ok_or_else(|| ExifError::ParseError("DateTimeOriginal must be a string".to_string()))?;

    let mut result = datetime_str.to_string();

    // Add subseconds if available (index 1)
    if let Some(subsec) = vals.get(1) {
        if let Some(subsec_str) = subsec.as_string() {
            if !subsec_str.is_empty() {
                result.push('.');
                result.push_str(subsec_str);
            }
        }
    }

    // Add offset if available (index 2)
    if let Some(offset) = vals.get(2) {
        if let Some(offset_str) = offset.as_string() {
            if !offset_str.is_empty() {
                result.push_str(offset_str);
            }
        }
    }

    Ok(TagValue::String(result))
}

/// SubSecCreateDate composite
/// ExifTool: lib/Image/ExifTool/Exif.pm:5077-5095
pub fn composite_subsec_create_date(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    let datetime = vals
        .first()
        .ok_or_else(|| ExifError::ParseError("SubSecCreateDate requires CreateDate".to_string()))?;

    let datetime_str = datetime
        .as_string()
        .ok_or_else(|| ExifError::ParseError("CreateDate must be a string".to_string()))?;

    let mut result = datetime_str.to_string();

    // Add subseconds if available
    if let Some(subsec) = vals.get(1) {
        if let Some(subsec_str) = subsec.as_string() {
            if !subsec_str.is_empty() && !result.contains('.') {
                // Find the time pattern and append subseconds
                if result.contains(':') {
                    result.push('.');
                    result.push_str(subsec_str);
                }
            }
        }
    }

    // Add offset if available
    if let Some(offset) = vals.get(2) {
        if let Some(offset_str) = offset.as_string() {
            if !offset_str.is_empty() && !result.contains('+') && !result.contains('-') {
                result.push_str(offset_str);
            }
        }
    }

    Ok(TagValue::String(result))
}

/// SubSecModifyDate composite
/// ExifTool: lib/Image/ExifTool/Exif.pm:5096-5114
pub fn composite_subsec_modify_date(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    // Same logic as SubSecCreateDate
    composite_subsec_create_date(vals, _prts, _raws, _ctx)
}

/// DateTimeCreated composite
/// ExifTool: lib/Image/ExifTool/IPTC.pm:1388-1396
pub fn composite_datetime_created(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    let date = vals
        .first()
        .ok_or_else(|| ExifError::ParseError("DateTimeCreated requires DateCreated".to_string()))?;

    let time = vals
        .get(1)
        .ok_or_else(|| ExifError::ParseError("DateTimeCreated requires TimeCreated".to_string()))?;

    let date_str = date
        .as_string()
        .ok_or_else(|| ExifError::ParseError("DateCreated must be a string".to_string()))?;

    let time_str = time
        .as_string()
        .ok_or_else(|| ExifError::ParseError("TimeCreated must be a string".to_string()))?;

    // Format as standard datetime
    if date_str.len() == 8 && time_str.len() >= 6 {
        let year = &date_str[0..4];
        let month = &date_str[4..6];
        let day = &date_str[6..8];
        let hour = &time_str[0..2];
        let minute = &time_str[2..4];
        let second = &time_str[4..6];

        Ok(TagValue::string(format!(
            "{}:{}:{} {}:{}:{}",
            year, month, day, hour, minute, second
        )))
    } else {
        Ok(TagValue::string(format!("{} {}", date_str, time_str)))
    }
}

/// DateTimeOriginal composite
/// ExifTool: lib/Image/ExifTool/Composite.pm
pub fn composite_datetime_original(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    // Check DateTimeCreated first
    if let Some(datetime) = vals.first() {
        if let Some(dt_str) = datetime.as_string() {
            if dt_str.contains(' ') {
                return Ok(datetime.clone());
            }
        }
    }

    // Combine DateCreated and TimeCreated
    if let (Some(date), Some(time)) = (vals.first(), vals.get(1)) {
        if let (Some(date_str), Some(time_str)) = (date.as_string(), time.as_string()) {
            return Ok(TagValue::string(format!("{} {}", date_str, time_str)));
        }
    }

    Err(ExifError::ParseError(
        "DateTimeOriginal requires date and time".to_string(),
    ))
}

/// Lens composite
/// ExifTool: lib/Image/ExifTool/Canon.pm:9684-9691
pub fn composite_lens(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    // Try MinFocalLength and MaxFocalLength (indices 0, 1)
    if let (Some(min_focal), Some(max_focal)) = (vals.first(), vals.get(1)) {
        if let (Some(min_f), Some(max_f)) = (min_focal.as_f64(), max_focal.as_f64()) {
            let lens_desc = if (min_f - max_f).abs() < 0.1 {
                format!("{:.1} mm", min_f)
            } else {
                format!("{:.1} - {:.1} mm", min_f, max_f)
            };
            return Ok(TagValue::string(lens_desc));
        }
    }

    // Fallback to any available value
    for val in vals {
        if !matches!(val, TagValue::Empty) {
            return Ok(val.clone());
        }
    }

    Err(ExifError::ParseError("Cannot compute Lens".to_string()))
}

/// LensSpec composite
/// ExifTool: lib/Image/ExifTool/Nikon.pm:13165-13172
pub fn composite_lens_spec(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    // Combine Lens and LensType
    if let (Some(lens), Some(lens_type)) = (vals.first(), vals.get(1)) {
        let lens_str = lens.as_string().unwrap_or_default();
        let type_str = lens_type.as_string().unwrap_or_default();
        return Ok(TagValue::string(format!("{} {}", lens_str, type_str)));
    }

    // Fallback
    for val in vals {
        if !matches!(val, TagValue::Empty) {
            return Ok(val.clone());
        }
    }

    Err(ExifError::ParseError("Cannot compute LensSpec".to_string()))
}

/// LensType composite
/// ExifTool: lib/Image/ExifTool/Olympus.pm:4290-4299
pub fn composite_lens_type(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    // Combine LensTypeMake and LensTypeModel (indices 0, 1)
    if let (Some(make), Some(model)) = (vals.first(), vals.get(1)) {
        let make_str = make.as_string().unwrap_or_default();
        let model_str = model.as_string().unwrap_or_default();
        return Ok(TagValue::string(format!("{} {}", make_str, model_str)));
    }

    // Fallback
    for val in vals {
        if !matches!(val, TagValue::Empty) {
            return Ok(val.clone());
        }
    }

    Err(ExifError::ParseError("Cannot compute LensType".to_string()))
}

/// ShutterSpeed composite
/// ExifTool: lib/Image/ExifTool/Composite.pm
pub fn composite_shutter_speed(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    // Check BulbDuration first (index 2)
    if let Some(bulb) = vals.get(2) {
        if let Some(duration) = bulb.as_f64() {
            if duration > 0.0 {
                return Ok(format_shutter_speed(duration));
            }
        }
    }

    // Try ExposureTime (index 0)
    if let Some(exposure) = vals.first() {
        if let Some(time) = exposure.as_f64() {
            return Ok(format_shutter_speed(time));
        }
        if let Some((num, den)) = exposure.as_rational() {
            if den != 0 {
                return Ok(format_shutter_speed(num as f64 / den as f64));
            }
        }
    }

    // Try ShutterSpeedValue (index 1) - APEX units
    if let Some(ssv) = vals.get(1) {
        if let Some(apex_val) = ssv.as_f64() {
            let time = 2.0_f64.powf(-apex_val);
            return Ok(format_shutter_speed(time));
        }
    }

    Err(ExifError::ParseError(
        "Cannot compute ShutterSpeed".to_string(),
    ))
}

/// Format shutter speed as '1/x' or decimal seconds
fn format_shutter_speed(time_seconds: f64) -> TagValue {
    if time_seconds >= 1.0 {
        TagValue::string(format!("{:.1}", time_seconds))
    } else if time_seconds > 0.0 {
        let reciprocal = 1.0 / time_seconds;
        TagValue::string(format!("1/{:.0}", reciprocal.round()))
    } else {
        "0".into()
    }
}

/// Aperture composite
/// ExifTool: lib/Image/ExifTool/Composite.pm
pub fn composite_aperture(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    // Try FNumber first (index 0), then ApertureValue (index 1)
    for val in vals {
        if let Some(f) = val.as_f64() {
            let formatted = if f.fract() < 0.01 {
                format!("{:.0}", f)
            } else {
                format!("{:.1}", f)
            };
            return Ok(TagValue::string(formatted));
        }
    }

    Err(ExifError::ParseError("Cannot compute Aperture".to_string()))
}

/// ISO composite
/// ExifTool: lib/Image/ExifTool/Canon.pm:9792-9806
pub fn composite_iso(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    // Try each value in order
    for val in vals {
        if let Some(iso) = get_numeric_iso_value(val) {
            return Ok(TagValue::U32(iso));
        }
    }

    Err(ExifError::ParseError("Cannot compute ISO".to_string()))
}

/// Extract numeric ISO value
fn get_numeric_iso_value(tag_value: &TagValue) -> Option<u32> {
    match tag_value {
        TagValue::U32(val) if *val > 0 && *val <= 100_000 => Some(*val),
        TagValue::U16(val) if *val > 0 => Some(*val as u32),
        TagValue::U8(val) if *val > 0 => Some(*val as u32),
        TagValue::I32(val) if *val > 0 => Some(*val as u32),
        TagValue::F64(val) if *val > 0.0 => Some(val.round() as u32),
        TagValue::Rational(num, den) if *den != 0 && *num > 0 => {
            Some((*num as f64 / *den as f64).round() as u32)
        }
        TagValue::String(s) => {
            let first = s.split(',').next()?.trim();
            Some(first.parse::<f64>().ok()?.round() as u32)
        }
        _ => None,
    }
    .filter(|&val| val > 0 && val <= 100_000)
}

/// ScaleFactor35efl composite
/// ExifTool: lib/Image/ExifTool/Composite.pm:583-596
pub fn composite_scale_factor_35efl(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    // FocalLength is required (index 0)
    let focal_length = vals.first().ok_or_else(|| {
        ExifError::ParseError("ScaleFactor35efl requires FocalLength".to_string())
    })?;

    let _fl = focal_length
        .as_f64()
        .ok_or_else(|| ExifError::ParseError("FocalLength must be numeric".to_string()))?;

    // Try FocalLengthIn35mmFormat (index 1)
    if let Some(fl_35mm) = vals.get(1) {
        if let Some(fl_35mm_val) = fl_35mm.as_f64() {
            if let Some(fl_val) = vals.first().and_then(|v| v.as_f64()) {
                if fl_val > 0.0 && fl_35mm_val > 0.0 {
                    return Ok(TagValue::F64(fl_35mm_val / fl_val));
                }
            }
        }
    }

    // Default fallback: assume APS-C
    Ok(TagValue::F64(1.5))
}

/// FocalLength35efl composite
/// ExifTool: lib/Image/ExifTool/Composite.pm
pub fn composite_focal_length_35efl(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    let focal_length = vals.first().ok_or_else(|| {
        ExifError::ParseError("FocalLength35efl requires FocalLength".to_string())
    })?;

    let fl = focal_length
        .as_f64()
        .ok_or_else(|| ExifError::ParseError("FocalLength must be numeric".to_string()))?;

    // Get ScaleFactor35efl (index 1)
    let scale = vals.get(1).and_then(|v| v.as_f64()).unwrap_or(1.0);

    Ok(TagValue::F64(fl * scale))
}

/// CircleOfConfusion composite
/// ExifTool: lib/Image/ExifTool/Composite.pm
/// ValueConv: "sqrt(24*24+36*36) / ($val * 1440)"
pub fn composite_circle_of_confusion(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    let scale_factor = vals.first().ok_or_else(|| {
        ExifError::ParseError("CircleOfConfusion requires ScaleFactor35efl".to_string())
    })?;

    let scale = scale_factor
        .as_f64()
        .ok_or_else(|| ExifError::ParseError("ScaleFactor35efl must be numeric".to_string()))?;

    if scale == 0.0 {
        return Err(ExifError::ParseError(
            "ScaleFactor35efl cannot be zero".to_string(),
        ));
    }

    let diagonal_35mm = (24.0_f64 * 24.0 + 36.0 * 36.0).sqrt();
    let coc = diagonal_35mm / (scale * 1440.0);

    Ok(TagValue::F64(coc))
}

/// HyperfocalDistance composite
/// ExifTool: lib/Image/ExifTool/Composite.pm
pub fn composite_hyperfocal_distance(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    let focal_length = vals.first().and_then(|v| v.as_f64()).ok_or_else(|| {
        ExifError::ParseError("HyperfocalDistance requires FocalLength".to_string())
    })?;

    let aperture = vals
        .get(1)
        .and_then(|v| v.as_f64())
        .ok_or_else(|| ExifError::ParseError("HyperfocalDistance requires Aperture".to_string()))?;

    let coc = vals.get(2).and_then(|v| v.as_f64()).ok_or_else(|| {
        ExifError::ParseError("HyperfocalDistance requires CircleOfConfusion".to_string())
    })?;

    if aperture == 0.0 || coc == 0.0 {
        return Ok("inf".into());
    }

    let hyperfocal = (focal_length * focal_length) / (aperture * coc * 1000.0);
    Ok(TagValue::F64(hyperfocal))
}

/// DOF composite
/// ExifTool: lib/Image/ExifTool/Composite.pm
pub fn composite_dof(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    let focal_length = vals
        .first()
        .and_then(|v| v.as_f64())
        .ok_or_else(|| ExifError::ParseError("DOF requires FocalLength".to_string()))?;

    let aperture = vals
        .get(1)
        .and_then(|v| v.as_f64())
        .ok_or_else(|| ExifError::ParseError("DOF requires Aperture".to_string()))?;

    let coc = vals
        .get(2)
        .and_then(|v| v.as_f64())
        .ok_or_else(|| ExifError::ParseError("DOF requires CircleOfConfusion".to_string()))?;

    if focal_length == 0.0 || coc == 0.0 {
        return Ok("0".into());
    }

    // Get focus distance (indices 3-6)
    let mut distance = None;
    for i in 3..=6 {
        if let Some(d) = vals.get(i).and_then(|v| v.as_f64()) {
            if d > 0.0 {
                distance = Some(d);
                break;
            }
        }
    }

    let d = distance.unwrap_or(1e10); // Use large number for infinity

    #[allow(clippy::suspicious_operation_groupings)]
    let t = aperture * coc * (d * 1000.0 - focal_length) / (focal_length * focal_length);

    let near = d / (1.0 + t);
    let mut far = d / (1.0 - t);

    if far < 0.0 {
        far = 0.0; // 0 means infinity
    }

    Ok(TagValue::string(format!("{:.3} {:.3}", near, far)))
}

/// FOV composite
/// ExifTool: lib/Image/ExifTool/Composite.pm
pub fn composite_fov(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    let focal_length = vals
        .first()
        .and_then(|v| v.as_f64())
        .ok_or_else(|| ExifError::ParseError("FOV requires FocalLength".to_string()))?;

    let scale_factor = vals
        .get(1)
        .and_then(|v| v.as_f64())
        .ok_or_else(|| ExifError::ParseError("FOV requires ScaleFactor35efl".to_string()))?;

    if focal_length == 0.0 || scale_factor == 0.0 {
        return Err(ExifError::ParseError(
            "FOV requires non-zero FocalLength and ScaleFactor35efl".to_string(),
        ));
    }

    let focus_distance = vals.get(2).and_then(|v| v.as_f64());

    let mut corr = 1.0;
    if let Some(focus_dist) = focus_distance {
        if focus_dist > 0.0 {
            let d = 1000.0 * focus_dist - focal_length;
            if d > 0.0 {
                corr += focal_length / d;
            }
        }
    }

    let fd2 = (36.0 / (2.0 * focal_length * scale_factor * corr)).atan();
    let fov_angle = fd2 * 360.0 / std::f64::consts::PI;

    let mut fov_values = vec![fov_angle];
    if let Some(focus_dist) = focus_distance {
        if focus_dist > 0.0 && focus_dist < 10000.0 {
            let field_width = 2.0 * focus_dist * fd2.sin() / fd2.cos();
            fov_values.push(field_width);
        }
    }

    let fov_str = fov_values
        .iter()
        .map(|v| format!("{:.1}", v))
        .collect::<Vec<_>>()
        .join(" ");

    Ok(TagValue::String(fov_str))
}

/// LightValue composite
/// ExifTool: lib/Image/ExifTool/Exif.pm:4685-4697
pub fn composite_light_value(
    vals: &[TagValue],
    prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    // Aperture (index 0), ShutterSpeed (index 1), ISO (index 2 from prt)
    let aperture = vals
        .first()
        .and_then(|v| v.as_f64())
        .ok_or_else(|| ExifError::ParseError("LightValue requires Aperture".to_string()))?;

    let shutter_speed = vals
        .get(1)
        .and_then(|v| v.as_f64())
        .ok_or_else(|| ExifError::ParseError("LightValue requires ShutterSpeed".to_string()))?;

    // ExifTool uses $prt[2] for ISO
    let iso = prts
        .get(2)
        .and_then(|v| v.as_f64())
        .ok_or_else(|| ExifError::ParseError("LightValue requires ISO".to_string()))?;

    if aperture <= 0.0 || shutter_speed <= 0.0 || iso <= 0.0 {
        return Err(ExifError::ParseError(
            "LightValue requires positive values".to_string(),
        ));
    }

    let light_value = (aperture * aperture * 100.0 / (shutter_speed * iso)).log2();
    Ok(TagValue::F64(light_value))
}

/// ImageWidth composite
pub fn composite_image_width(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    // Panasonic sensor borders (indices 0=SensorLeftBorder, 1=SensorRightBorder)
    if let (Some(left), Some(right)) = (vals.first(), vals.get(1)) {
        if let (Some(l), Some(r)) = (left.as_u32(), right.as_u32()) {
            return Ok(TagValue::U32(r - l));
        }
    }

    // Try other indices
    for val in vals {
        if let Some(w) = val.as_u32() {
            return Ok(TagValue::U32(w));
        }
    }

    Err(ExifError::ParseError(
        "Cannot compute ImageWidth".to_string(),
    ))
}

/// ImageHeight composite
pub fn composite_image_height(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    // Panasonic sensor borders (indices 0=SensorTopBorder, 1=SensorBottomBorder)
    if let (Some(top), Some(bottom)) = (vals.first(), vals.get(1)) {
        if let (Some(t), Some(b)) = (top.as_u32(), bottom.as_u32()) {
            return Ok(TagValue::U32(b - t));
        }
    }

    // Try other indices
    for val in vals {
        if let Some(h) = val.as_u32() {
            return Ok(TagValue::U32(h));
        }
    }

    Err(ExifError::ParseError(
        "Cannot compute ImageHeight".to_string(),
    ))
}

/// Rotation composite
/// ExifTool: lib/Image/ExifTool/Composite.pm:435-443
pub fn composite_rotation(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    let orientation = vals
        .first()
        .ok_or_else(|| ExifError::ParseError("Rotation requires Orientation".to_string()))?;

    let orientation_val = orientation
        .as_u8()
        .ok_or_else(|| ExifError::ParseError("Orientation must be numeric".to_string()))?;

    let rotation = match orientation_val {
        1 | 2 => 0,
        3 | 4 => 180,
        5 | 6 => 90,
        7 | 8 => 270,
        _ => {
            return Err(ExifError::ParseError(
                "Invalid Orientation value".to_string(),
            ))
        }
    };

    Ok(TagValue::U16(rotation))
}

/// PreviewImageSize composite
pub fn composite_preview_image_size(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    if let (Some(width), Some(height)) = (vals.first(), vals.get(1)) {
        if let (Some(w), Some(h)) = (width.as_u32(), height.as_u32()) {
            return Ok(TagValue::string(format!("{}x{}", w, h)));
        }
    }

    Err(ExifError::ParseError(
        "Cannot compute PreviewImageSize".to_string(),
    ))
}

/// ThumbnailImage composite
pub fn composite_thumbnail_image(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    // Check offset and length
    if let (Some(offset), Some(length)) = (vals.first(), vals.get(1)) {
        if let (Some(_off), Some(len)) = (offset.as_u32(), length.as_u32()) {
            if len > 0 {
                return Ok(TagValue::string(format!(
                    "(Binary data {} bytes, use -b option to extract)",
                    len
                )));
            }
        }
    }

    Err(ExifError::ParseError(
        "Cannot compute ThumbnailImage".to_string(),
    ))
}

/// PreviewImage composite
pub fn composite_preview_image(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    // Same as ThumbnailImage
    composite_thumbnail_image(vals, _prts, _raws, _ctx)
}

/// Duration composite
pub fn composite_duration(
    vals: &[TagValue],
    _prts: &[TagValue],
    _raws: &[TagValue],
    _ctx: Option<&ExifContext>,
) -> Result<TagValue> {
    // Method 1: FrameRate and FrameCount
    if let (Some(rate), Some(count)) = (vals.first(), vals.get(1)) {
        if let (Some(r), Some(c)) = (rate.as_f64(), count.as_f64()) {
            if r > 0.0 {
                return Ok(TagValue::F64(c / r));
            }
        }
    }

    // Method 2: SampleRate and TotalSamples
    if let (Some(rate), Some(samples)) = (vals.get(2), vals.get(3)) {
        if let (Some(r), Some(s)) = (rate.as_f64(), samples.as_f64()) {
            if r > 0.0 {
                return Ok(TagValue::F64(s / r));
            }
        }
    }

    Err(ExifError::ParseError("Cannot compute Duration".to_string()))
}

// =============================================================================
// HashMap-Based Functions (for orchestration.rs compatibility)
// =============================================================================
//
// These functions use the original HashMap-based signature for backward
// compatibility with try_manual_composite_computation() in orchestration.rs.
// They will be removed once all callers use the array-based interface.

/// Compute ImageSize composite with proper formatting (HashMap interface)
/// ExifTool: lib/Image/ExifTool/Exif.pm:4641-4660 ImageSize definition
pub fn compute_image_size(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // Check RawImageCroppedSize first (index 4 in desire list)
    if let Some(raw_size) = available_tags.get("RawImageCroppedSize") {
        if let Some(size_str) = raw_size.as_string() {
            let formatted = size_str.replace(' ', "x");
            return Some(TagValue::string(formatted));
        }
        return Some(raw_size.clone());
    }

    // ExifTool logic: Only use ExifImageWidth/Height for Canon and Phase One RAW formats
    let use_exif_dimensions = is_canon_raw_tiff_type(available_tags);

    if use_exif_dimensions {
        if let (Some(width), Some(height)) = (
            available_tags.get("EXIF:ExifImageWidth"),
            available_tags.get("EXIF:ExifImageHeight"),
        ) {
            if let (Some(w), Some(h)) = (width.as_u32(), height.as_u32()) {
                return Some(TagValue::string(format!("{}x{}", w, h)));
            }
        }
    }

    // Use ImageWidth/ImageHeight
    if let (Some(width), Some(height)) = (
        available_tags
            .get("File:ImageWidth")
            .or_else(|| available_tags.get("ImageWidth")),
        available_tags
            .get("File:ImageHeight")
            .or_else(|| available_tags.get("ImageHeight")),
    ) {
        if let (Some(w), Some(h)) = (width.as_u32(), height.as_u32()) {
            return Some(TagValue::string(format!("{}x{}", w, h)));
        }
    }

    None
}

/// Check if TIFF_TYPE matches Canon/Phase One RAW formats
fn is_canon_raw_tiff_type(available_tags: &HashMap<String, TagValue>) -> bool {
    if let Some(file_type) = available_tags.get("File:FileType") {
        if let Some("CR2" | "Canon 1D RAW" | "IIQ" | "EIP") = file_type.as_string() {
            return true;
        }
    }

    if let Some(extension) = available_tags.get("File:FileTypeExtension") {
        if let Some(ext_str) = extension.as_string() {
            return ext_str.to_uppercase() == "CR2" || ext_str.to_uppercase() == "IIQ";
        }
    }

    false
}

/// Compute Megapixels composite (HashMap interface)
pub fn compute_megapixels(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    let image_size = available_tags.get("ImageSize")?;
    let image_size_str = image_size.as_string()?;

    let digit_regex = Regex::new(r"\d+").ok()?;
    let digits: Vec<u32> = digit_regex
        .find_iter(image_size_str)
        .filter_map(|m| m.as_str().parse::<u32>().ok())
        .collect();

    if digits.len() < 2 {
        return None;
    }

    let width = digits[0] as f64;
    let height = digits[1] as f64;
    let megapixels = (width * height) / 1_000_000.0;

    Some(TagValue::string(format!("{:.1}", megapixels)))
}

/// Compute GPSAltitude composite (HashMap interface)
pub fn compute_gps_altitude(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    let altitude = available_tags
        .get("GPS:GPSAltitude")
        .or_else(|| available_tags.get("EXIF:GPSAltitude"))
        .or_else(|| available_tags.get("XMP:GPSAltitude"))
        .or_else(|| available_tags.get("GPSAltitude"))?;

    let alt_ref = available_tags
        .get("GPS:GPSAltitudeRef")
        .or_else(|| available_tags.get("EXIF:GPSAltitudeRef"))
        .or_else(|| available_tags.get("XMP:GPSAltitudeRef"))
        .or_else(|| available_tags.get("GPSAltitudeRef"));

    let decimal_alt = if let Some(alt_value) = altitude.as_rational() {
        alt_value.0 as f64 / alt_value.1 as f64
    } else {
        altitude.as_f64()?
    };

    let signed_altitude = if let Some(ref_val) = alt_ref {
        if let Some(ref_str) = ref_val.as_string() {
            if ref_str == "1" || ref_str.to_lowercase().contains("below") {
                -decimal_alt.abs()
            } else {
                decimal_alt.abs()
            }
        } else {
            decimal_alt.abs()
        }
    } else {
        decimal_alt.abs()
    };

    Some(TagValue::F64(signed_altitude))
}

/// Compute GPSPosition composite (HashMap interface)
pub fn compute_gps_position(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    let lat_value = available_tags.get("GPSLatitude").and_then(|v| v.as_f64());
    let lon_value = available_tags.get("GPSLongitude").and_then(|v| v.as_f64());

    match (lat_value, lon_value) {
        (Some(lat), Some(lon)) => Some(TagValue::string(format!("{} {}", lat, lon))),
        (Some(lat), None) => Some(TagValue::string(format!("{} 0", lat))),
        (None, Some(lon)) => Some(TagValue::string(format!("0 {}", lon))),
        (None, None) => None,
    }
}

/// Compute GPSLatitude composite (HashMap interface)
pub fn compute_gps_latitude(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    let latitude = available_tags
        .get("GPS:GPSLatitude")
        .or_else(|| available_tags.get("EXIF:GPSLatitude"))
        .or_else(|| available_tags.get("GPSLatitude"))?;
    let latitude_ref = available_tags
        .get("GPS:GPSLatitudeRef")
        .or_else(|| available_tags.get("EXIF:GPSLatitudeRef"))
        .or_else(|| available_tags.get("GPSLatitudeRef"))?;

    let lat_value = latitude.as_f64()?;
    let ref_str = latitude_ref.as_string()?;

    let signed = if ref_str.to_lowercase().starts_with('s') {
        -lat_value
    } else {
        lat_value
    };

    Some(TagValue::F64(signed))
}

/// Compute GPSLongitude composite (HashMap interface)
pub fn compute_gps_longitude(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    let longitude = available_tags
        .get("GPS:GPSLongitude")
        .or_else(|| available_tags.get("EXIF:GPSLongitude"))
        .or_else(|| available_tags.get("GPSLongitude"))?;
    let longitude_ref = available_tags
        .get("GPS:GPSLongitudeRef")
        .or_else(|| available_tags.get("EXIF:GPSLongitudeRef"))
        .or_else(|| available_tags.get("GPSLongitudeRef"))?;

    let lon_value = longitude.as_f64()?;
    let ref_str = longitude_ref.as_string()?;

    let signed = if ref_str.to_lowercase().starts_with('w') {
        -lon_value
    } else {
        lon_value
    };

    Some(TagValue::F64(signed))
}

/// Compute GPSDateTime composite (HashMap interface)
pub fn compute_gps_datetime(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    let date_stamp = available_tags.get("GPSDateStamp")?;
    let time_stamp = available_tags.get("GPSTimeStamp")?;

    let date_str = date_stamp.as_string()?;
    let time_str = time_stamp.as_string()?;

    Some(TagValue::string(format!("{} {}Z", date_str, time_str)))
}

/// Compute ShutterSpeed composite (HashMap interface)
pub fn compute_shutter_speed(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    if let Some(bulb_duration) = available_tags.get("BulbDuration") {
        if let Some(duration) = bulb_duration.as_f64() {
            if duration > 0.0 {
                return Some(format_shutter_speed(duration));
            }
        }
    }

    if let Some(exposure_time) = available_tags.get("ExposureTime") {
        if let Some(time) = exposure_time.as_f64() {
            return Some(format_shutter_speed(time));
        }
        if let Some((num, den)) = exposure_time.as_rational() {
            if den != 0 {
                return Some(format_shutter_speed(num as f64 / den as f64));
            }
        }
    }

    if let Some(shutter_speed_val) = available_tags.get("ShutterSpeedValue") {
        if let Some(speed_val) = shutter_speed_val.as_f64() {
            return Some(format_shutter_speed(2.0_f64.powf(-speed_val)));
        }
    }

    None
}

/// Compute Aperture composite (HashMap interface)
pub fn compute_aperture(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    let raw_value = available_tags
        .get("FNumber")
        .or_else(|| available_tags.get("ApertureValue"))?;

    let aperture_f64 = raw_value.as_f64()?;

    let formatted = if aperture_f64.fract() < 0.01 {
        format!("{:.0}", aperture_f64)
    } else {
        format!("{:.1}", aperture_f64)
    };

    Some(TagValue::string(formatted))
}

/// Compute ISO composite (HashMap interface)
pub fn compute_iso(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    for tag_name in &[
        "ISO",
        "ISOSpeed",
        "ISOSpeedRatings",
        "PhotographicSensitivity",
        "CameraISO",
        "SonyISO",
    ] {
        if let Some(iso) = available_tags.get(*tag_name) {
            if let Some(iso_val) = get_numeric_iso_value(iso) {
                return Some(TagValue::U32(iso_val));
            }
        }
    }

    // Canon fallback
    if let (Some(base_iso), Some(auto_iso)) =
        (available_tags.get("BaseISO"), available_tags.get("AutoISO"))
    {
        if let (Some(base), Some(auto)) = (
            get_numeric_iso_value(base_iso),
            get_numeric_iso_value(auto_iso),
        ) {
            let calculated = (base as f64 * auto as f64 / 100.0).round() as u32;
            return Some(TagValue::U32(calculated));
        }
    }

    None
}

/// Compute PreviewImageSize composite (HashMap interface)
pub fn compute_preview_image_size(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    if let (Some(width), Some(height)) = (
        available_tags.get("PreviewImageWidth"),
        available_tags.get("PreviewImageHeight"),
    ) {
        if let (Some(w), Some(h)) = (width.as_u32(), height.as_u32()) {
            return Some(TagValue::string(format!("{}x{}", w, h)));
        }
    }
    None
}

/// Compute ThumbnailImage composite (HashMap interface)
pub fn compute_thumbnail_image(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    if let (Some(offset), Some(length)) = (
        available_tags.get("ThumbnailOffset"),
        available_tags.get("ThumbnailLength"),
    ) {
        if let (Some(_), Some(length_val)) = (offset.as_u32(), length.as_u32()) {
            if length_val > 0 {
                return Some(TagValue::string(format!(
                    "(Binary data {} bytes, use -b option to extract)",
                    length_val
                )));
            }
        }
    }

    if let (Some(offset), Some(length)) = (
        available_tags.get("OtherImageStart"),
        available_tags.get("OtherImageLength"),
    ) {
        if let (Some(_), Some(length_val)) = (offset.as_u32(), length.as_u32()) {
            if length_val > 0 {
                return Some(TagValue::string(format!(
                    "(Binary data {} bytes, use -b option to extract)",
                    length_val
                )));
            }
        }
    }

    None
}

/// Compute PreviewImage composite (HashMap interface)
pub fn compute_preview_image(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    if let (Some(start), Some(length)) = (
        available_tags.get("PreviewImageStart"),
        available_tags.get("PreviewImageLength"),
    ) {
        if let (Some(_), Some(length_val)) = (start.as_u32(), length.as_u32()) {
            if length_val > 0 {
                return Some(TagValue::string(format!(
                    "(Binary data {} bytes, use -b option to extract)",
                    length_val
                )));
            }
        }
    }

    if let (Some(start), Some(length)) = (
        available_tags.get("OtherImageStart"),
        available_tags.get("OtherImageLength"),
    ) {
        if let (Some(_), Some(length_val)) = (start.as_u32(), length.as_u32()) {
            if length_val > 0 {
                return Some(TagValue::string(format!(
                    "(Binary data {} bytes, use -b option to extract)",
                    length_val
                )));
            }
        }
    }

    None
}

/// Compute DateTimeOriginal composite (HashMap interface)
pub fn compute_datetime_original(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    if let Some(datetime_created) = available_tags.get("DateTimeCreated") {
        if let Some(dt_str) = datetime_created.as_string() {
            if dt_str.contains(' ') {
                return Some(datetime_created.clone());
            }
        }
    }

    if let (Some(date), Some(time)) = (
        available_tags.get("DateCreated"),
        available_tags.get("TimeCreated"),
    ) {
        if let (Some(date_str), Some(time_str)) = (date.as_string(), time.as_string()) {
            return Some(TagValue::string(format!("{} {}", date_str, time_str)));
        }
    }

    None
}

/// Compute DateTimeCreated composite (HashMap interface)
pub fn compute_datetime_created(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    let date_created = available_tags
        .get("IPTC:DateCreated")
        .or_else(|| available_tags.get("DateCreated"))?;
    let time_created = available_tags
        .get("IPTC:TimeCreated")
        .or_else(|| available_tags.get("TimeCreated"))?;

    let date_str = date_created.as_string()?;
    let time_str = time_created.as_string()?;

    if date_str.len() == 8 && time_str.len() >= 6 {
        let year = &date_str[0..4];
        let month = &date_str[4..6];
        let day = &date_str[6..8];
        let hour = &time_str[0..2];
        let minute = &time_str[2..4];
        let second = &time_str[4..6];

        Some(TagValue::string(format!(
            "{}:{}:{} {}:{}:{}",
            year, month, day, hour, minute, second
        )))
    } else {
        Some(TagValue::string(format!("{} {}", date_str, time_str)))
    }
}

/// Compute SubSecDateTimeOriginal composite (HashMap interface)
pub fn compute_subsec_datetime_original(
    available_tags: &HashMap<String, TagValue>,
) -> Option<TagValue> {
    let datetime_original = available_tags.get("EXIF:DateTimeOriginal")?;
    let datetime_str = datetime_original.as_string()?;

    let mut result = datetime_str.to_string();

    if let Some(subsec) = available_tags.get("SubSecTimeOriginal") {
        if let Some(subsec_str) = subsec.as_string() {
            if !subsec_str.is_empty() {
                result.push('.');
                result.push_str(subsec_str);
            }
        }
    }

    if let Some(offset) = available_tags.get("OffsetTimeOriginal") {
        if let Some(offset_str) = offset.as_string() {
            if !offset_str.is_empty() {
                result.push_str(offset_str);
            }
        }
    }

    Some(TagValue::String(result))
}

/// Compute SubSecCreateDate composite (HashMap interface)
pub fn compute_subsec_create_date(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    let create_date = available_tags.get("EXIF:CreateDate")?;
    let create_date_str = create_date.as_string()?;

    let mut result = create_date_str.to_string();

    if let Some(subsec) = available_tags.get("SubSecTimeDigitized") {
        if let Some(subsec_str) = subsec.as_string() {
            if !subsec_str.is_empty() && !result.contains('.') && result.contains(':') {
                result.push('.');
                result.push_str(subsec_str);
            }
        }
    }

    if let Some(offset) = available_tags.get("OffsetTimeDigitized") {
        if let Some(offset_str) = offset.as_string() {
            if !offset_str.is_empty() && !result.contains('+') && !result.contains('-') {
                result.push_str(offset_str);
            }
        }
    }

    Some(TagValue::String(result))
}

/// Compute SubSecModifyDate composite (HashMap interface)
pub fn compute_subsec_modify_date(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    let modify_date = available_tags.get("EXIF:ModifyDate")?;
    let modify_date_str = modify_date.as_string()?;

    let mut result = modify_date_str.to_string();

    if let Some(subsec) = available_tags.get("SubSecTime") {
        if let Some(subsec_str) = subsec.as_string() {
            if !subsec_str.is_empty() && !result.contains('.') && result.contains(':') {
                result.push('.');
                result.push_str(subsec_str);
            }
        }
    }

    if let Some(offset) = available_tags.get("OffsetTime") {
        if let Some(offset_str) = offset.as_string() {
            if !offset_str.is_empty() && !result.contains('+') && !result.contains('-') {
                result.push_str(offset_str);
            }
        }
    }

    Some(TagValue::String(result))
}

/// Compute FocalLength35efl composite (HashMap interface)
pub fn compute_focal_length_35efl(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    let focal_length = available_tags.get("FocalLength")?;
    let fl = focal_length.as_f64().unwrap_or(0.0);

    let scale_factor = available_tags
        .get("ScaleFactor35efl")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);

    Some(TagValue::F64(fl * scale_factor))
}

/// Compute ScaleFactor35efl composite (HashMap interface)
pub fn compute_scale_factor_35efl(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    trace!("ScaleFactor35efl function called");

    let focal_length_tag = available_tags.get("FocalLength")?;
    let _focal_length_val = extract_focal_length_value(focal_length_tag)?;

    // Check FocalLengthIn35mmFormat first
    if let Some(focal_35mm) = available_tags.get("FocalLengthIn35mmFormat") {
        if let Some(focal_35mm_val) = extract_focal_length_value(focal_35mm) {
            if let Some(focal_length) = available_tags.get("FocalLength") {
                if let Some(focal_length_val) = extract_focal_length_value(focal_length) {
                    if focal_length_val > 0.0 && focal_35mm_val > 0.0 {
                        return Some(TagValue::F64(focal_35mm_val / focal_length_val));
                    }
                }
            }
        }
    }

    // Canon-specific scale factor sources
    for canon_tag in &["CanonScaleFactor", "ScaleFactor35efl"] {
        if let Some(scale_val) = available_tags.get(*canon_tag) {
            if let Some(scale) = scale_val.as_f64() {
                if scale > 0.0 {
                    return Some(TagValue::F64(scale));
                }
            }
        }
    }

    // Default fallback: assume APS-C
    Some(TagValue::F64(1.5))
}

fn extract_focal_length_value(tag_value: &TagValue) -> Option<f64> {
    if let Some(focal_length) = tag_value.as_f64() {
        if focal_length > 0.0 {
            return Some(focal_length);
        }
    }

    if let Some((num, den)) = tag_value.as_rational() {
        if den != 0 {
            let focal_length = num as f64 / den as f64;
            if focal_length > 0.0 {
                return Some(focal_length);
            }
        }
    }

    None
}

/// Compute CircleOfConfusion composite (HashMap interface)
pub fn compute_circle_of_confusion(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    let scale_factor = available_tags.get("ScaleFactor35efl")?;
    let scale = scale_factor.as_f64()?;

    if scale == 0.0 {
        return None;
    }

    let diagonal_35mm = (24.0_f64 * 24.0 + 36.0 * 36.0).sqrt();
    let coc = diagonal_35mm / (scale * 1440.0);

    Some(TagValue::F64(coc))
}

/// Compute HyperfocalDistance composite (HashMap interface)
pub fn compute_hyperfocal_distance(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    let focal_length = available_tags.get("FocalLength")?.as_f64()?;
    let aperture = available_tags.get("Aperture")?.as_f64()?;
    let coc = available_tags.get("CircleOfConfusion")?.as_f64()?;

    if aperture == 0.0 || coc == 0.0 {
        return Some("inf".into());
    }

    let hyperfocal = (focal_length * focal_length) / (aperture * coc * 1000.0);
    Some(TagValue::F64(hyperfocal))
}

/// Compute FOV composite (HashMap interface)
pub fn compute_fov(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    let focal_length = available_tags.get("FocalLength")?.as_f64()?;
    let scale_factor = available_tags.get("ScaleFactor35efl")?.as_f64()?;

    if focal_length == 0.0 || scale_factor == 0.0 {
        return None;
    }

    let focus_distance = available_tags.get("FocusDistance").and_then(|v| v.as_f64());

    let mut corr = 1.0;
    if let Some(focus_dist) = focus_distance {
        if focus_dist > 0.0 {
            let d = 1000.0 * focus_dist - focal_length;
            if d > 0.0 {
                corr += focal_length / d;
            }
        }
    }

    let fd2 = (36.0 / (2.0 * focal_length * scale_factor * corr)).atan();
    let fov_angle = fd2 * 360.0 / std::f64::consts::PI;

    let mut fov_values = vec![fov_angle];
    if let Some(focus_dist) = focus_distance {
        if focus_dist > 0.0 && focus_dist < 10000.0 {
            let field_width = 2.0 * focus_dist * fd2.sin() / fd2.cos();
            fov_values.push(field_width);
        }
    }

    let fov_str = fov_values
        .iter()
        .map(|v| format!("{:.1}", v))
        .collect::<Vec<_>>()
        .join(" ");

    Some(TagValue::String(fov_str))
}

/// Compute DOF composite (HashMap interface)
pub fn compute_dof(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    let focal_length = available_tags.get("FocalLength")?.as_f64()?;
    let aperture = available_tags.get("Aperture")?.as_f64()?;
    let coc = available_tags.get("CircleOfConfusion")?.as_f64()?;

    if focal_length == 0.0 || coc == 0.0 {
        return Some("0".into());
    }

    let mut distance = None;

    if let Some(focus_dist) = available_tags.get("FocusDistance").and_then(|v| v.as_f64()) {
        if focus_dist > 0.0 {
            distance = Some(focus_dist);
        } else {
            distance = Some(1e10);
        }
    }

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

    if distance.is_none() {
        let lower = available_tags
            .get("FocusDistanceLower")
            .and_then(|v| v.as_f64());
        let upper = available_tags
            .get("FocusDistanceUpper")
            .and_then(|v| v.as_f64());

        if let (Some(lower_val), Some(upper_val)) = (lower, upper) {
            distance = Some((lower_val + upper_val) / 2.0);
        }
    }

    let d = distance?;

    #[allow(clippy::suspicious_operation_groupings)]
    let t = aperture * coc * (d * 1000.0 - focal_length) / (focal_length * focal_length);

    let near = d / (1.0 + t);
    let mut far = d / (1.0 - t);

    if far < 0.0 {
        far = 0.0;
    }

    Some(TagValue::string(format!("{:.3} {:.3}", near, far)))
}

/// Compute ImageWidth composite (HashMap interface)
pub fn compute_image_width(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // Panasonic sensor border calculation
    if let (Some(left), Some(right)) = (
        available_tags
            .get("EXIF:SensorLeftBorder")
            .or_else(|| available_tags.get("SensorLeftBorder")),
        available_tags
            .get("EXIF:SensorRightBorder")
            .or_else(|| available_tags.get("SensorRightBorder")),
    ) {
        if let (Some(left_val), Some(right_val)) = (left.as_u32(), right.as_u32()) {
            return Some(TagValue::U32(right_val - left_val));
        }
    }

    // Priority: SubIFD3 > IFD0 > ExifIFD
    for tag_name in &[
        "SubIFD3:ImageWidth",
        "ImageWidth",
        "ExifImageWidth",
        "File:ImageWidth",
    ] {
        if let Some(width) = available_tags.get(*tag_name) {
            if let Some(w) = width.as_u32() {
                return Some(TagValue::U32(w));
            }
        }
    }

    None
}

/// Compute ImageHeight composite (HashMap interface)
pub fn compute_image_height(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // Panasonic sensor border calculation
    if let (Some(top), Some(bottom)) = (
        available_tags
            .get("EXIF:SensorTopBorder")
            .or_else(|| available_tags.get("SensorTopBorder")),
        available_tags
            .get("EXIF:SensorBottomBorder")
            .or_else(|| available_tags.get("SensorBottomBorder")),
    ) {
        if let (Some(top_val), Some(bottom_val)) = (top.as_u32(), bottom.as_u32()) {
            return Some(TagValue::U32(bottom_val - top_val));
        }
    }

    // Priority: SubIFD3 > IFD0 > ExifIFD
    for tag_name in &[
        "SubIFD3:ImageHeight",
        "ImageHeight",
        "ExifImageHeight",
        "File:ImageHeight",
    ] {
        if let Some(height) = available_tags.get(*tag_name) {
            if let Some(h) = height.as_u32() {
                return Some(TagValue::U32(h));
            }
        }
    }

    None
}

/// Compute Rotation composite (HashMap interface)
pub fn compute_rotation(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    let orientation = available_tags.get("Orientation")?;
    let orientation_val = orientation.as_u8()?;

    let rotation = match orientation_val {
        1 | 2 => 0,
        3 | 4 => 180,
        5 | 6 => 90,
        7 | 8 => 270,
        _ => return None,
    };

    Some(TagValue::U16(rotation))
}

/// Compute LightValue composite (HashMap interface)
pub fn compute_light_value(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    let aperture = available_tags.get("Aperture")?;
    let shutter_speed = available_tags.get("ShutterSpeed")?;
    let iso = available_tags.get("ISO")?;

    let aperture_val = extract_positive_float(aperture)?;
    let shutter_speed_val = extract_positive_float(shutter_speed)?;
    let iso_val = extract_positive_float(iso)?;

    let light_value = (aperture_val * aperture_val * 100.0 / (shutter_speed_val * iso_val)).log2();

    Some(TagValue::F64(light_value))
}

fn extract_positive_float(tag_value: &TagValue) -> Option<f64> {
    match tag_value {
        TagValue::F64(val) if *val > 0.0 => Some(*val),
        TagValue::U32(val) if *val > 0 => Some(*val as f64),
        TagValue::U16(val) if *val > 0 => Some(*val as f64),
        TagValue::U8(val) if *val > 0 => Some(*val as f64),
        TagValue::I32(val) if *val > 0 => Some(*val as f64),
        TagValue::I16(val) if *val > 0 => Some(*val as f64),
        TagValue::Rational(num, den) if *den != 0 && *num > 0 => Some(*num as f64 / *den as f64),
        TagValue::String(s) => parse_float_from_string(s).filter(|&val| val > 0.0),
        _ => None,
    }
}

fn parse_float_from_string(s: &str) -> Option<f64> {
    if let Some(slash_pos) = s.find('/') {
        let numerator = s[..slash_pos].parse::<f64>().ok()?;
        let denominator = s[slash_pos + 1..].parse::<f64>().ok()?;
        if denominator != 0.0 {
            return Some(numerator / denominator);
        }
    }
    s.parse::<f64>().ok()
}

/// Compute Lens composite (HashMap interface)
pub fn compute_lens(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    if let (Some(min_focal), Some(max_focal)) = (
        available_tags.get("MinFocalLength"),
        available_tags.get("MaxFocalLength"),
    ) {
        if let (Some(min_f), Some(max_f)) = (min_focal.as_f64(), max_focal.as_f64()) {
            let lens_desc = if (min_f - max_f).abs() < 0.1 {
                format!("{:.1} mm", min_f)
            } else {
                format!("{:.1} - {:.1} mm", min_f, max_f)
            };
            return Some(TagValue::string(lens_desc));
        }
    }

    if let Some(lens_model) = available_tags.get("LensModel") {
        return Some(lens_model.clone());
    }

    if let Some(lens) = available_tags.get("Lens") {
        return Some(lens.clone());
    }

    None
}

/// Compute LensID composite (HashMap interface)
pub fn compute_lens_id(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // Check manufacturer for Olympus-specific handling
    let make = available_tags
        .get("Make")
        .or_else(|| available_tags.get("EXIF:Make"))
        .and_then(|v| v.as_string())
        .unwrap_or_default();

    if make.contains("OLYMPUS") {
        if let Some(lens_model) = available_tags.get("EXIF:LensModel") {
            if let Some(lens_string) = lens_model.as_string() {
                if lens_string.contains("mm") || lens_string.contains("F") {
                    return Some(lens_model.clone());
                }
            }
        }
    }

    if let Some(lens_type) = available_tags.get("LensType") {
        return Some(lens_type.clone());
    }

    if let Some(xmp_lens_id) = available_tags.get("LensID") {
        return Some(xmp_lens_id.clone());
    }

    None
}

/// Compute LensSpec composite (HashMap interface)
pub fn compute_lens_spec(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    if let (Some(lens), Some(lens_type)) =
        (available_tags.get("Lens"), available_tags.get("LensType"))
    {
        let lens_str = lens.as_string().unwrap_or_default();
        let type_str = lens_type.as_string().unwrap_or_default();
        return Some(TagValue::string(format!("{} {}", lens_str, type_str)));
    }

    if let Some(lens_info) = available_tags.get("LensInfo") {
        return Some(lens_info.clone());
    }

    None
}

/// Compute LensType composite (HashMap interface)
pub fn compute_lens_type(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    if let (Some(make), Some(model)) = (
        available_tags.get("LensTypeMake"),
        available_tags.get("LensTypeModel"),
    ) {
        let make_str = make.as_string().unwrap_or_default();
        let model_str = model.as_string().unwrap_or_default();
        return Some(TagValue::string(format!("{} {}", make_str, model_str)));
    }

    if let Some(lens_type) = available_tags.get("LensType") {
        return Some(lens_type.clone());
    }

    for fallback_tag in &["LensType2", "LensType3", "RFLensType"] {
        if let Some(lens_type) = available_tags.get(*fallback_tag) {
            return Some(lens_type.clone());
        }
    }

    None
}

/// Compute Duration composite (HashMap interface)
pub fn compute_duration(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // Method 1: FrameRate and FrameCount
    if let (Some(frame_rate), Some(frame_count)) = (
        available_tags.get("FrameRate"),
        available_tags.get("FrameCount"),
    ) {
        if let (Some(rate), Some(count)) = (frame_rate.as_f64(), frame_count.as_f64()) {
            if rate > 0.0 {
                return Some(TagValue::F64(count / rate));
            }
        }
    }

    // Method 2: SampleRate and TotalSamples
    if let (Some(sample_rate), Some(total_samples)) = (
        available_tags.get("SampleRate"),
        available_tags.get("TotalSamples"),
    ) {
        if let (Some(rate), Some(samples)) = (sample_rate.as_f64(), total_samples.as_f64()) {
            if rate > 0.0 {
                return Some(TagValue::F64(samples / rate));
            }
        }
    }

    None
}
