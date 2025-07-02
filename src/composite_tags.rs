//! Composite tag processing module
//!
//! This module contains the logic for building composite tags through
//! multi-pass dependency resolution and computing individual composite tag
//! values. This code was extracted from `src/exif.rs`to reduce the monolithic
//! file size.
//!
//! The main entry point is the facade pattern in
//! `ExifReader::build_composite_tags()` which calls into this module's
//! functions.

use std::collections::{HashMap, HashSet};
use tracing::{debug, trace, warn};

use crate::generated::TAG_BY_ID;
use crate::generated::{CompositeTagDef, COMPOSITE_TAGS};
use crate::types::TagValue;

/// Build the initial available tags map from extracted tags with group prefixes
/// This replaces the inline logic from the original single-pass implementation
pub fn build_available_tags_map(
    extracted_tags: &HashMap<u16, TagValue>,
    tag_sources: &HashMap<u16, crate::types::TagSourceInfo>,
) -> HashMap<String, TagValue> {
    let mut available_tags = HashMap::new();

    // Add extracted tags with group prefixes
    for (&tag_id, value) in extracted_tags {
        let ifd_name = tag_sources
            .get(&tag_id)
            .map(|s| s.ifd_name.as_str())
            .unwrap_or("Root");

        let group_name = match ifd_name {
            "Root" | "IFD0" | "IFD1" => "EXIF",
            "GPS" => "GPS",
            "ExifIFD" => "EXIF",
            "InteropIFD" => "EXIF",
            "MakerNotes" => "MakerNotes",
            _ => "EXIF",
        };

        let base_tag_name = TAG_BY_ID
            .get(&(tag_id as u32))
            .map(|tag_def| tag_def.name.to_string())
            .unwrap_or_else(|| format!("Tag_{tag_id:04X}"));

        // Add with group prefix (e.g., "GPS:GPSLatitude")
        let prefixed_name = format!("{group_name}:{base_tag_name}");
        available_tags.insert(prefixed_name, value.clone());

        // Also add without group prefix for broader matching (e.g., "GPSLatitude")
        available_tags.insert(base_tag_name, value.clone());
    }

    available_tags
}

/// Check if a composite tag can be built (all required dependencies available)
/// This is the core dependency resolution logic for multi-pass building
pub fn can_build_composite(
    composite_def: &CompositeTagDef,
    available_tags: &HashMap<String, TagValue>,
    built_composites: &HashSet<&str>,
) -> bool {
    // Check all required dependencies
    for (_index, tag_name) in composite_def.require {
        if !is_dependency_available(tag_name, available_tags, built_composites) {
            trace!(
                "Missing required dependency for {}: {}",
                composite_def.name,
                tag_name
            );
            return false;
        }
    }

    // All required dependencies are available
    true
}

/// Check if a specific dependency (tag name) is available
/// Handles group prefixes and composite tag references
pub fn is_dependency_available(
    tag_name: &str,
    available_tags: &HashMap<String, TagValue>,
    built_composites: &HashSet<&str>,
) -> bool {
    // Direct lookup in available tags
    if available_tags.contains_key(tag_name) {
        return true;
    }

    // Check with various group prefixes
    for prefix in &["EXIF", "GPS", "MakerNotes", "Composite"] {
        let prefixed_name = format!("{prefix}:{tag_name}");
        if available_tags.contains_key(&prefixed_name) {
            return true;
        }
    }

    // Special handling for composite dependencies
    // Check if the tag is a composite that has already been built
    if built_composites.contains(tag_name) {
        return true;
    }

    false
}

/// Handle unresolved composite tags (circular dependencies or missing base tags)
/// This provides diagnostic information and graceful degradation
pub fn handle_unresolved_composites(unresolved_composites: &[&CompositeTagDef]) {
    warn!(
        "Unable to resolve {} composite tags - possible circular dependencies or missing base tags:",
        unresolved_composites.len()
    );

    for composite_def in unresolved_composites {
        let mut missing_deps = Vec::new();
        for (_index, tag_name) in composite_def.require {
            // Note: We could make this more detailed by checking available_tags/built_composites
            // but for now, just log the unresolved composite and its requirements
            missing_deps.push(*tag_name);
        }

        warn!("  - {} requires: {:?}", composite_def.name, missing_deps);
    }

    // Future enhancement: Could implement ExifTool's "final pass ignoring inhibits"
    // strategy here for additional fallback resolution
}

/// Compute a single composite tag value based on its dependencies
/// ExifTool: lib/Image/ExifTool.pm composite tag evaluation
pub fn compute_composite_tag(
    composite_def: &CompositeTagDef,
    available_tags: &HashMap<String, TagValue>,
) -> Option<TagValue> {
    // Check if all required dependencies are available
    for (_index, tag_name) in composite_def.require {
        if !available_tags.contains_key(*tag_name) {
            trace!(
                "Missing required dependency for {}: {}",
                composite_def.name,
                tag_name
            );
            return None;
        }
    }

    // Dispatch to specific composite tag implementations
    // Each implementation translates ExifTool's Perl ValueConv expression
    match composite_def.name {
        // Existing implementations
        "ImageSize" => compute_image_size(available_tags),
        "GPSAltitude" => compute_gps_altitude(available_tags),
        "PreviewImageSize" => compute_preview_image_size(available_tags),
        "ShutterSpeed" => compute_shutter_speed(available_tags),

        // New implementations for common composite tags
        "Aperture" => compute_aperture(available_tags),
        "DateTimeOriginal" => compute_datetime_original(available_tags),
        "FocalLength35efl" => compute_focal_length_35efl(available_tags),
        "ScaleFactor35efl" => compute_scale_factor_35efl(available_tags),
        "SubSecDateTimeOriginal" => compute_subsec_datetime_original(available_tags),
        "CircleOfConfusion" => compute_circle_of_confusion(available_tags),

        _ => {
            // For other composite tags, log what dependencies are available vs missing
            let mut available_deps = Vec::new();
            let mut missing_deps = Vec::new();

            for (_index, tag_name) in composite_def
                .require
                .iter()
                .chain(composite_def.desire.iter())
            {
                if available_tags.contains_key(*tag_name) {
                    available_deps.push(*tag_name);
                } else {
                    missing_deps.push(*tag_name);
                }
            }

            trace!(
                "Composite tag {} not yet implemented. Available deps: {:?}, Missing deps: {:?}",
                composite_def.name,
                available_deps,
                missing_deps
            );
            None
        }
    }
}

/// Apply ValueConv and PrintConv transformations to composite tag values
/// Returns tuple of (value, print) where:
/// - value: The computed value (composite tags don't have ValueConv)
/// - print: The result after PrintConv (or value.to_string() if no PrintConv)
pub fn apply_composite_conversions(
    computed_value: &TagValue,
    composite_def: &CompositeTagDef,
) -> (TagValue, String) {
    use crate::registry;

    let value = computed_value.clone();

    // Apply PrintConv if present to get human-readable string
    let print = if let Some(print_conv_ref) = composite_def.print_conv_ref {
        registry::apply_print_conv(print_conv_ref, &value)
    } else {
        value.to_string()
    };

    (value, print)
}

/// Multi-pass composite tag resolution and computation
/// This is the main logic extracted from ExifReader::build_composite_tags()
pub fn resolve_and_compute_composites(
    mut available_tags: HashMap<String, TagValue>,
) -> HashMap<String, TagValue> {
    const MAX_PASSES: usize = 10; // Reasonable limit to prevent infinite loops

    let mut composite_tags = HashMap::new();
    let mut built_composites = HashSet::new();
    let mut pending_composites: Vec<&CompositeTagDef> = COMPOSITE_TAGS.iter().collect();

    debug!(
        "Starting multi-pass composite building with {} pending composites",
        pending_composites.len()
    );

    // Multi-pass loop to handle composite-on-composite dependencies
    for pass in 1..=MAX_PASSES {
        let mut progress_made = false;
        let mut deferred_composites = Vec::new();
        let initial_pending_count = pending_composites.len();

        trace!(
            "Pass {}: Processing {} pending composites",
            pass,
            initial_pending_count
        );

        for composite_def in pending_composites {
            if can_build_composite(composite_def, &available_tags, &built_composites) {
                // All dependencies available - build the composite
                if let Some(computed_value) = compute_composite_tag(composite_def, &available_tags)
                {
                    // Apply PrintConv to the computed value
                    let (final_value, _print) =
                        apply_composite_conversions(&computed_value, composite_def);

                    let composite_name = format!("Composite:{}", composite_def.name);

                    // Add to available_tags for future composite dependencies
                    available_tags.insert(composite_name.clone(), final_value.clone());
                    available_tags.insert(composite_def.name.to_string(), final_value.clone());

                    // Store in composite_tags collection
                    composite_tags.insert(composite_name.clone(), final_value);
                    built_composites.insert(composite_def.name);

                    debug!("Built composite tag: {} (pass {})", composite_name, pass);
                    progress_made = true;
                }
            } else {
                // Dependencies not available - defer for next pass
                deferred_composites.push(composite_def);
            }
        }

        let built_this_pass = initial_pending_count - deferred_composites.len();
        trace!(
            "Pass {} complete: built {} composites, {} deferred",
            pass,
            built_this_pass,
            deferred_composites.len()
        );

        // Exit conditions
        if deferred_composites.is_empty() {
            debug!("All composite tags built successfully in {} passes", pass);
            break; // All composites built
        }

        if !progress_made {
            // No progress made - either circular dependency or unresolvable dependencies
            warn!(
                "No progress made in pass {} - {} composites remain unbuilt",
                pass,
                deferred_composites.len()
            );
            handle_unresolved_composites(&deferred_composites);
            break;
        }

        pending_composites = deferred_composites;
    }

    debug!(
        "Composite building complete: {} total composites built",
        built_composites.len()
    );

    composite_tags
}

// =============================================================================
// Individual Composite Tag Computation Methods
// =============================================================================

/// Compute ImageSize composite (ImageWidth + ImageHeight)
/// ExifTool: lib/Image/ExifTool/Composite.pm ImageSize definition
fn compute_image_size(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    // First try ImageWidth/ImageHeight from EXIF
    if let (Some(width), Some(height)) = (
        available_tags.get("ImageWidth"),
        available_tags.get("ImageHeight"),
    ) {
        if let (Some(w), Some(h)) = (width.as_u32(), height.as_u32()) {
            return Some(TagValue::String(format!("{w}x{h}")));
        }
    }

    // Try EXIF variants
    if let (Some(width), Some(height)) = (
        available_tags.get("ExifImageWidth"),
        available_tags.get("ExifImageHeight"),
    ) {
        if let (Some(w), Some(h)) = (width.as_u32(), height.as_u32()) {
            return Some(TagValue::String(format!("{w}x{h}")));
        }
    }

    None
}

/// Compute GPSAltitude composite (GPSAltitude + GPSAltitudeRef)
/// ExifTool: lib/Image/ExifTool/GPS.pm GPSAltitude composite
fn compute_gps_altitude(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
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

            return Some(TagValue::String(format!("{sign}{decimal_alt:.1} m")));
        }
    }
    None
}

/// Compute PreviewImageSize composite
fn compute_preview_image_size(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
    if let (Some(width), Some(height)) = (
        available_tags.get("PreviewImageWidth"),
        available_tags.get("PreviewImageHeight"),
    ) {
        if let (Some(w), Some(h)) = (width.as_u32(), height.as_u32()) {
            return Some(TagValue::String(format!("{w}x{h}")));
        }
    }
    None
}

/// Compute ShutterSpeed composite (ExposureTime formatted as '1/x' or 'x''')  
/// ExifTool: lib/Image/ExifTool/Composite.pm ShutterSpeed definition
/// ValueConv: ($val[2] and $val[2]>0) ? $val[2] : (defined($val[0]) ? $val[0] : $val[1])
/// Dependencies: ExposureTime(0), ShutterSpeedValue(1), BulbDuration(2)
fn compute_shutter_speed(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
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
fn format_shutter_speed(time_seconds: f64) -> TagValue {
    if time_seconds >= 1.0 {
        // Slow shutter speeds: format as decimal seconds
        TagValue::String(format!("{time_seconds:.1}"))
    } else if time_seconds > 0.0 {
        // Fast shutter speeds: format as 1/x
        let reciprocal = 1.0 / time_seconds;
        TagValue::String(format!("1/{:.0}", reciprocal.round()))
    } else {
        // Invalid time value
        TagValue::String("0".to_string())
    }
}

/// Compute Aperture composite tag
/// ExifTool: lib/Image/ExifTool/Composite.pm - "$val[0] || $val[1]"
/// Tries FNumber first, falls back to ApertureValue
fn compute_aperture(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
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
fn compute_datetime_original(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
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
            return Some(TagValue::String(format!("{date_str} {time_str}")));
        }
    }

    None
}

/// Compute FocalLength35efl composite tag
/// ExifTool: lib/Image/ExifTool/Composite.pm
/// ValueConv: "ToFloat(@val); ($val[0] || 0) * ($val[1] || 1)"
fn compute_focal_length_35efl(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
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
fn compute_scale_factor_35efl(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
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
fn compute_subsec_datetime_original(
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
fn compute_circle_of_confusion(available_tags: &HashMap<String, TagValue>) -> Option<TagValue> {
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
