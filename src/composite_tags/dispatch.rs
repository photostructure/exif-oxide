//! Composite tag computation dispatcher
//!
//! This module provides the central dispatcher that routes composite tag
//! computation requests to the appropriate implementation functions.

use std::collections::HashMap;
use tracing::trace;

use crate::generated::CompositeTagDef;
use crate::types::TagValue;

use super::implementations::*;

/// Compute a single composite tag value based on its dependencies
/// ExifTool: lib/Image/ExifTool.pm composite tag evaluation
pub fn compute_composite_tag(
    composite_def: &CompositeTagDef,
    available_tags: &HashMap<String, TagValue>,
) -> Option<TagValue> {
    // Special handling for flexible composite tags like ImageSize
    // Most composite implementations have their own dependency logic in the compute functions
    if composite_def.name == "ImageSize" {
        // ImageSize handles its own dependency logic internally
    } else {
        // Check if all required dependencies are available for other composites
        for tag_name in composite_def.require {
            if !available_tags.contains_key(*tag_name) {
                trace!(
                    "Missing required dependency for {}: {}",
                    composite_def.name,
                    tag_name
                );
                return None;
            }
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
        "Megapixels" => compute_megapixels(available_tags),
        "GPSPosition" => compute_gps_position(available_tags),
        "HyperfocalDistance" => compute_hyperfocal_distance(available_tags),
        "FOV" => compute_fov(available_tags),
        "DOF" => compute_dof(available_tags),

        // Phase 1: Core Essential Tags
        "ISO" => compute_iso(available_tags),
        "ImageWidth" => compute_image_width(available_tags),
        "ImageHeight" => compute_image_height(available_tags),
        "Rotation" => compute_rotation(available_tags),

        // Phase 2: GPS Consolidation
        "GPSDateTime" => compute_gps_datetime(available_tags),
        "GPSLatitude" => compute_gps_latitude(available_tags),
        "GPSLongitude" => compute_gps_longitude(available_tags),

        // Phase 3: SubSec Timestamps
        "SubSecCreateDate" => compute_subsec_create_date(available_tags),
        "SubSecModifyDate" => compute_subsec_modify_date(available_tags),
        "SubSecMediaCreateDate" => compute_subsec_media_create_date(available_tags),

        // Phase 4: Lens System
        "Lens" => compute_lens(available_tags),
        "LensID" => compute_lens_id(available_tags),
        "LensSpec" => compute_lens_spec(available_tags),
        "LensType" => compute_lens_type(available_tags),

        // Phase 5: Media Tags & Advanced Features
        "Duration" => compute_duration(available_tags),
        // Enhanced ScaleFactor35efl (keep existing simple version for compatibility)
        // "ScaleFactor35efl" => compute_scale_factor_35efl_enhanced(available_tags),
        _ => {
            // For other composite tags, log what dependencies are available vs missing
            let mut available_deps = Vec::new();
            let mut missing_deps = Vec::new();

            for tag_name in composite_def
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
