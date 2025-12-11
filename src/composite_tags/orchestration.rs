//! Multi-pass orchestration logic for composite tag building
//!
//! This module handles the multi-pass building of composite tags, resolving
//! dependencies between composite tags and applying conversions.
//!
//! ExifTool reference: lib/Image/ExifTool.pm:3929-4115 BuildCompositeTags

use std::collections::{HashMap, HashSet};
use tracing::{debug, trace, warn};

use crate::generated::composite_tags::{CompositeTagDef, COMPOSITE_TAGS};
use crate::types::TagValue;

use super::resolution::{can_build_composite, resolve_dependency_arrays, TagDependencyValues};

/// Handle unresolved composite tags (circular dependencies or missing base tags)
/// This provides diagnostic information and graceful degradation
/// ExifTool: lib/Image/ExifTool.pm:4103-4110 - final pass ignoring inhibits
pub fn handle_unresolved_composites(unresolved_composites: &[&CompositeTagDef]) {
    if unresolved_composites.is_empty() {
        return;
    }

    warn!(
        "Unable to resolve {} composite tags - possible circular dependencies or missing base tags",
        unresolved_composites.len()
    );

    for composite_def in unresolved_composites {
        let mut missing_deps = Vec::new();
        for tag_name in composite_def.require {
            missing_deps.push(*tag_name);
        }

        trace!("  - {} requires: {:?}", composite_def.name, missing_deps);
    }

    // Future enhancement: Could implement ExifTool's "final pass ignoring inhibits"
    // strategy here for additional fallback resolution
}

/// Compute a composite tag value using the generated function pointer
/// This replaces the old dispatch.rs match statement with direct function calls
///
/// ExifTool: lib/Image/ExifTool.pm:4056-4080 - composite tag evaluation
fn compute_composite_value(
    composite_def: &CompositeTagDef,
    available_tags: &HashMap<String, TagDependencyValues>,
    built_composites: &HashSet<String>,
) -> Option<TagValue> {
    // Get the dependency arrays - now properly separated into raw/val/prt
    // ExifTool: lib/Image/ExifTool.pm:3553-3560
    let (vals, prts, raws) =
        resolve_dependency_arrays(composite_def, available_tags, built_composites);

    // Call the generated ValueConv function if available
    if let Some(value_conv_fn) = composite_def.value_conv {
        match value_conv_fn(&vals, &prts, &raws, None) {
            Ok(value) => {
                trace!(
                    "Computed composite {} via generated function: {:?}",
                    composite_def.name,
                    value
                );
                Some(value)
            }
            Err(e) => {
                trace!(
                    "ValueConv function failed for {}: {:?}",
                    composite_def.name,
                    e
                );
                None
            }
        }
    } else {
        // No generated function - try fallback to manual implementations
        // This handles complex expressions that PPI couldn't translate
        try_manual_composite_computation(composite_def, available_tags, built_composites)
    }
}

/// Try manual computation for composites without generated functions
/// This handles complex expressions like ImageSize, Megapixels, LensID
fn try_manual_composite_computation(
    composite_def: &CompositeTagDef,
    available_tags: &HashMap<String, TagDependencyValues>,
    _built_composites: &HashSet<String>,
) -> Option<TagValue> {
    use crate::composite_tags::implementations::*;

    // Convert to simple TagValue map for legacy implementations.
    // We extract the `.val` (ValueConv'd) value, which matches ExifTool's `$val[n]`.
    //
    // Note: This means manual implementations cannot access `@raw` values directly.
    // Currently only Nikon LensID uses `@raw` in ExifTool (for hex formatting), and
    // our compute_lens_id has its own implementation that doesn't need raw bytes.
    // If future implementations need `@raw`, update them to take TagDependencyValues.
    let simple_map: HashMap<String, TagValue> = available_tags
        .iter()
        .map(|(k, v)| (k.clone(), v.val.clone()))
        .collect();

    // Route to manual implementations for complex composites
    match composite_def.name {
        // Core composites with complex logic
        "ImageSize" => compute_image_size(&simple_map),
        "Megapixels" => compute_megapixels(&simple_map),
        "GPSAltitude" => compute_gps_altitude(&simple_map),
        "GPSPosition" => compute_gps_position(&simple_map),
        "GPSLatitude" => compute_gps_latitude(&simple_map),
        "GPSLongitude" => compute_gps_longitude(&simple_map),
        "GPSDateTime" => compute_gps_datetime(&simple_map),
        "ShutterSpeed" => compute_shutter_speed(&simple_map),
        "Aperture" => compute_aperture(&simple_map),
        "ISO" => compute_iso(&simple_map),

        // Thumbnail/Preview
        "ThumbnailImage" => compute_thumbnail_image(&simple_map),
        "PreviewImage" => compute_preview_image(&simple_map),
        "PreviewImageSize" => compute_preview_image_size(&simple_map),

        // Date/Time composites
        "DateTimeOriginal" => compute_datetime_original(&simple_map),
        "DateTimeCreated" => compute_datetime_created(&simple_map),
        "SubSecDateTimeOriginal" => compute_subsec_datetime_original(&simple_map),
        "SubSecCreateDate" => compute_subsec_create_date(&simple_map),
        "SubSecModifyDate" => compute_subsec_modify_date(&simple_map),

        // Lens system
        "Lens" => compute_lens(&simple_map),
        "LensID" => compute_lens_id(&simple_map),
        "LensSpec" => compute_lens_spec(&simple_map),
        "LensType" => compute_lens_type(&simple_map),
        "FocalLength35efl" => compute_focal_length_35efl(&simple_map),
        "ScaleFactor35efl" => compute_scale_factor_35efl(&simple_map),

        // Image dimensions
        "ImageWidth" => compute_image_width(&simple_map),
        "ImageHeight" => compute_image_height(&simple_map),
        "Rotation" => compute_rotation(&simple_map),

        // Advanced calculations
        "CircleOfConfusion" => compute_circle_of_confusion(&simple_map),
        "HyperfocalDistance" => compute_hyperfocal_distance(&simple_map),
        "DOF" => compute_dof(&simple_map),
        "FOV" => compute_fov(&simple_map),
        "LightValue" => compute_light_value(&simple_map),
        "Duration" => compute_duration(&simple_map),

        _ => {
            trace!(
                "No implementation for composite {}, value_conv_expr: {:?}",
                composite_def.name,
                composite_def.value_conv_expr
            );
            None
        }
    }
}

/// Apply PrintConv transformation to a computed composite value
/// Returns the value suitable for display
///
/// ExifTool: lib/Image/ExifTool.pm:4081-4095 - PrintConv application
///
/// Note: This takes pre-computed dependency arrays to avoid duplicate resolution.
/// The caller should compute these once via `resolve_dependency_arrays()`.
fn apply_composite_print_conv(
    computed_value: &TagValue,
    composite_def: &CompositeTagDef,
    vals: &[TagValue],
    prts: &[TagValue],
    raws: &[TagValue],
) -> TagValue {
    // Per TRUST-EXIFTOOL.md: GPS coordinates should always be in decimal format
    // Skip PrintConv for GPS coordinate composite tags to return decimal values
    let is_gps_coordinate = matches!(
        composite_def.name,
        "GPSLatitude" | "GPSLongitude" | "GPSPosition" | "GPSAltitude"
    );

    if is_gps_coordinate {
        return computed_value.clone();
    }

    // Try generated PrintConv function first
    if let Some(print_conv_fn) = composite_def.print_conv {
        match print_conv_fn(vals, prts, raws, None) {
            Ok(print_value) => return print_value,
            Err(e) => {
                trace!(
                    "PrintConv function failed for {}: {:?}, using raw value",
                    composite_def.name,
                    e
                );
            }
        }
    }

    // Fallback: return the computed value as-is
    computed_value.clone()
}

/// Multi-pass composite tag resolution and computation
/// This is the main entry point for building all composite tags
///
/// ExifTool: lib/Image/ExifTool.pm:3929-4115 BuildCompositeTags
///
/// Takes a map of available tags with their raw/val/prt values and returns
/// computed composite tags as simple TagValue (the print value).
pub fn resolve_and_compute_composites(
    mut available_tags: HashMap<String, TagDependencyValues>,
) -> HashMap<String, TagValue> {
    const MAX_PASSES: usize = 10; // Reasonable limit to prevent infinite loops

    let mut composite_tags = HashMap::new();
    let mut built_composites: HashSet<String> = HashSet::new();

    // Collect all composite definitions from the registry
    // Note: COMPOSITE_TAGS is a HashMap which loses duplicates for same-named tags
    // ExifTool uses first-successful-match semantics
    let mut pending_composites: Vec<&CompositeTagDef> = COMPOSITE_TAGS.values().copied().collect();

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
            // Skip if this composite has already been successfully built
            // ExifTool: Only the first successful definition is used per tag name
            if built_composites.contains(composite_def.name) {
                trace!("Skipping {} - already built", composite_def.name);
                continue;
            }

            // Debug GPS composite tags specifically
            if composite_def.name.starts_with("GPS") {
                trace!(
                    "Processing GPS composite {} with dependencies: require={:?}, desire={:?}",
                    composite_def.name,
                    composite_def.require,
                    composite_def.desire
                );
            }

            let can_build = can_build_composite(composite_def, &available_tags, &built_composites);
            if composite_def.name.starts_with("GPS") {
                trace!(
                    "GPS composite {} can_build_composite result: {}",
                    composite_def.name,
                    can_build
                );
            }

            if can_build {
                // Resolve dependency arrays ONCE - used for both ValueConv and PrintConv
                // This avoids the duplicate resolution issue
                let (vals, prts, raws) =
                    resolve_dependency_arrays(composite_def, &available_tags, &built_composites);

                // All dependencies available - build the composite
                if let Some(computed_value) =
                    compute_composite_value(composite_def, &available_tags, &built_composites)
                {
                    // Apply PrintConv to the computed value, reusing pre-computed arrays
                    let print_value = apply_composite_print_conv(
                        &computed_value,
                        composite_def,
                        &vals,
                        &prts,
                        &raws,
                    );

                    let composite_name = format!("Composite:{}", composite_def.name);

                    // Add to available_tags for future composite dependencies
                    // Create TagDependencyValues with raw=val for computed composites
                    // (composites don't have a separate "raw" value)
                    let dep_values = TagDependencyValues {
                        raw: computed_value.clone(),
                        val: computed_value,
                        prt: print_value.clone(),
                    };
                    available_tags.insert(composite_name.clone(), dep_values.clone());
                    available_tags.insert(composite_def.name.to_string(), dep_values);

                    // Store in composite_tags collection - use PrintConv result
                    composite_tags.insert(composite_name.clone(), print_value);
                    built_composites.insert(composite_def.name.to_string());

                    debug!("Built composite tag: {} (pass {})", composite_name, pass);
                    progress_made = true;
                } else {
                    // Dependencies available but computation failed - try next definition with same name
                    trace!(
                        "Failed to compute {} - will try next definition if available",
                        composite_def.name
                    );
                    deferred_composites.push(composite_def);
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
            break;
        }

        if !progress_made {
            // No progress made - either circular dependency or unresolvable dependencies
            trace!(
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
