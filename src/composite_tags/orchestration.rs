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

/// Compute a composite tag value using generated function or fallback registry
///
/// Priority order:
/// 1. Generated ValueConv function pointer (from PPI translation)
/// 2. COMPOSITE_FALLBACKS registry lookup (for complex expressions PPI can't translate)
///
/// ExifTool: lib/Image/ExifTool.pm:4056-4080 - composite tag evaluation
fn compute_composite_value(
    composite_def: &CompositeTagDef,
    available_tags: &HashMap<String, TagDependencyValues>,
    built_composites: &HashSet<String>,
) -> Option<TagValue> {
    use crate::core::COMPOSITE_FALLBACKS;

    // Get the dependency arrays - now properly separated into raw/val/prt
    // ExifTool: lib/Image/ExifTool.pm:3553-3560
    let (vals, prts, raws) =
        resolve_dependency_arrays(composite_def, available_tags, built_composites);

    // Priority 1: Call the generated ValueConv function if available
    if let Some(value_conv_fn) = composite_def.value_conv {
        match value_conv_fn(&vals, &prts, &raws, None) {
            Ok(value) => {
                trace!(
                    "Computed composite {} via generated function: {:?}",
                    composite_def.name,
                    value
                );
                return Some(value);
            }
            Err(e) => {
                trace!(
                    "ValueConv function failed for {}: {:?}",
                    composite_def.name,
                    e
                );
                // Fall through to try fallback
            }
        }
    }

    // Priority 2: Check COMPOSITE_FALLBACKS registry
    if let Some(fallback_fn) = COMPOSITE_FALLBACKS.get(composite_def.name) {
        match fallback_fn(&vals, &prts, &raws, None) {
            Ok(value) => {
                trace!(
                    "Computed composite {} via fallback registry: {:?}",
                    composite_def.name,
                    value
                );
                return Some(value);
            }
            Err(e) => {
                trace!(
                    "Fallback function failed for {}: {:?}",
                    composite_def.name,
                    e
                );
            }
        }
    }

    trace!(
        "No implementation for composite {}, value_conv_expr: {:?}",
        composite_def.name,
        composite_def.value_conv_expr
    );
    None
}

/// Apply PrintConv transformation to a computed composite value
/// Returns the value suitable for display
///
/// ExifTool: lib/Image/ExifTool.pm:4081-4095 - PrintConv application
///
/// Note: This takes pre-computed dependency arrays to avoid duplicate resolution.
/// The caller should compute these once via `resolve_dependency_arrays()`.
///
/// IMPORTANT: For tags computed via COMPOSITE_FALLBACKS (value_conv: None),
/// the fallback functions already apply appropriate formatting. The generated
/// PrintConv functions have a semantic mismatch - they use $val as vals[0]
/// (the dependency) when ExifTool's PrintConv expects $val to be the ValueConv
/// result. Until codegen is fixed, skip PrintConv for fallback-computed tags.
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

    // Skip PrintConv for tags computed via COMPOSITE_FALLBACKS
    // These fallback functions already apply appropriate formatting, and the
    // generated PrintConv has a bug: it uses vals[0] for $val instead of the
    // computed ValueConv result. See docs/todo/P03c-composite-tags.md.
    if composite_def.value_conv.is_none() {
        return computed_value.clone();
    }

    // Try generated PrintConv function (only for generated ValueConv tags)
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
