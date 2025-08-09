//! Composite tag computation dispatcher
//!
//! This module provides the central dispatcher that routes composite tag
//! computation requests to the appropriate implementation functions.

use std::collections::HashMap;
use tracing::trace;

use crate::generated::composite_tags::CompositeTagDef;
use crate::types::TagValue;

use super::implementations::*;
use super::resolution::resolve_tag_dependency;
use super::value_conv_evaluator::ValueConvEvaluator;

/// Compute a single composite tag value based on its dependencies using ExifTool's resolution
/// ExifTool: lib/Image/ExifTool.pm composite tag evaluation with dynamic dependency resolution
pub fn compute_composite_tag(
    composite_def: &CompositeTagDef,
    available_tags: &HashMap<String, TagValue>,
    built_composites: &std::collections::HashSet<&str>,
) -> Option<TagValue> {
    // Create a resolved dependency map for the composite computation
    // This maps dependency names to their resolved values using ExifTool's dynamic lookup
    let mut resolved_dependencies = HashMap::new();

    // Resolve all require dependencies
    for tag_name in composite_def.require {
        if let Some(resolved_value) =
            resolve_tag_dependency(tag_name, available_tags, built_composites)
        {
            resolved_dependencies.insert(tag_name.to_string(), resolved_value);
        } else {
            trace!(
                "Missing required dependency for {}: {}",
                composite_def.name,
                tag_name
            );
            return None;
        }
    }

    // Resolve all desire dependencies (optional)
    for tag_name in composite_def.desire {
        if let Some(resolved_value) =
            resolve_tag_dependency(tag_name, available_tags, built_composites)
        {
            resolved_dependencies.insert(tag_name.to_string(), resolved_value);
        }
    }

    // For backward compatibility, also include original available_tags
    // This allows compute functions to access tags not explicitly in require/desire
    for (key, value) in available_tags {
        resolved_dependencies
            .entry(key.clone())
            .or_insert_with(|| value.clone());
    }

    // Dispatch to specific composite tag implementations
    // Each implementation translates ExifTool's Perl ValueConv expression
    match composite_def.name {
        // Existing implementations
        "ImageSize" => compute_image_size(&resolved_dependencies),
        "GPSAltitude" => compute_gps_altitude(&resolved_dependencies),
        "PreviewImageSize" => compute_preview_image_size(&resolved_dependencies),
        "ThumbnailImage" => compute_thumbnail_image(&resolved_dependencies),
        "PreviewImage" => compute_preview_image(&resolved_dependencies),
        "ShutterSpeed" => compute_shutter_speed(&resolved_dependencies),

        // New implementations for common composite tags
        "Aperture" => compute_aperture(&resolved_dependencies),
        "DateTimeCreated" => compute_datetime_created(&resolved_dependencies),
        "DateTimeOriginal" => compute_datetime_original(&resolved_dependencies),
        "FocalLength35efl" => compute_focal_length_35efl(&resolved_dependencies),
        "ScaleFactor35efl" => compute_scale_factor_35efl(&resolved_dependencies),
        "SubSecDateTimeOriginal" => compute_subsec_datetime_original(&resolved_dependencies),
        "CircleOfConfusion" => compute_circle_of_confusion(&resolved_dependencies),
        "Megapixels" => compute_megapixels(&resolved_dependencies),
        "GPSPosition" => compute_gps_position(&resolved_dependencies),
        "HyperfocalDistance" => compute_hyperfocal_distance(&resolved_dependencies),
        "FOV" => compute_fov(&resolved_dependencies),
        "DOF" => compute_dof(&resolved_dependencies),

        // Phase 1: Core Essential Tags
        "ISO" => compute_iso(&resolved_dependencies),
        "ImageWidth" => compute_image_width(&resolved_dependencies),
        "ImageHeight" => compute_image_height(&resolved_dependencies),
        "Rotation" => compute_rotation(&resolved_dependencies),

        // Phase 2: GPS Consolidation
        "GPSDateTime" => compute_gps_datetime(&resolved_dependencies),
        "GPSLatitude" => compute_gps_latitude(&resolved_dependencies),
        "GPSLongitude" => compute_gps_longitude(&resolved_dependencies),

        // Phase 3: SubSec Timestamps
        "SubSecCreateDate" => compute_subsec_create_date(&resolved_dependencies),
        "SubSecModifyDate" => compute_subsec_modify_date(&resolved_dependencies),
        "SubSecMediaCreateDate" => compute_subsec_media_create_date(&resolved_dependencies),

        // Phase 4: Lens System
        "Lens" => compute_lens(&resolved_dependencies),
        "LensID" => compute_lens_id(&resolved_dependencies),
        "LensSpec" => compute_lens_spec(&resolved_dependencies),
        "LensType" => compute_lens_type(&resolved_dependencies),

        // Phase 5: Media Tags & Advanced Features
        "Duration" => compute_duration(&resolved_dependencies),

        // Advanced composite calculations
        "LightValue" => compute_light_value(&resolved_dependencies),

        // Enhanced ScaleFactor35efl (keep existing simple version for compatibility)
        // "ScaleFactor35efl" => compute_scale_factor_35efl_enhanced(available_tags),
        _ => {
            // Try dynamic ValueConv evaluation for generated composite definitions
            if composite_def.value_conv.is_some() {
                trace!(
                    "Attempting dynamic evaluation for composite: {}",
                    composite_def.name
                );

                let mut evaluator = ValueConvEvaluator::new();
                if let Some(result) =
                    evaluator.evaluate_composite(composite_def, &resolved_dependencies)
                {
                    trace!("Dynamic evaluation succeeded for: {}", composite_def.name);
                    return Some(result);
                }

                trace!("Dynamic evaluation failed for: {}", composite_def.name);
            }

            // Log available vs missing dependencies for debugging
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
                "Composite tag {} not implemented (no ValueConv or evaluation failed). Available deps: {:?}, Missing deps: {:?}",
                composite_def.name,
                available_deps,
                missing_deps
            );
            None
        }
    }
}
