//! Multi-pass orchestration logic for composite tag building
//!
//! This module handles the multi-pass building of composite tags, resolving
//! dependencies between composite tags and applying conversions.

use std::collections::{HashMap, HashSet};
use tracing::{debug, trace, warn};

use crate::generated::{CompositeTagDef, COMPOSITE_TAGS};
use crate::types::TagValue;

use super::dispatch::compute_composite_tag;
use super::resolution::can_build_composite;

/// Handle unresolved composite tags (circular dependencies or missing base tags)
/// This provides diagnostic information and graceful degradation
pub fn handle_unresolved_composites(unresolved_composites: &[&CompositeTagDef]) {
    warn!(
        "Unable to resolve {} composite tags - possible circular dependencies or missing base tags:",
        unresolved_composites.len()
    );

    for composite_def in unresolved_composites {
        let mut missing_deps = Vec::new();
        for tag_name in composite_def.require {
            // Note: We could make this more detailed by checking available_tags/built_composites
            // but for now, just log the unresolved composite and its requirements
            missing_deps.push(*tag_name);
        }

        warn!("  - {} requires: {:?}", composite_def.name, missing_deps);
    }

    // Future enhancement: Could implement ExifTool's "final pass ignoring inhibits"
    // strategy here for additional fallback resolution
}

/// Apply ValueConv and PrintConv transformations to composite tag values
/// Returns tuple of (value, print) where:
/// - value: The computed value (composite tags don't have ValueConv)
/// - print: The result after PrintConv (or value if no PrintConv)
pub fn apply_composite_conversions(
    computed_value: &TagValue,
    composite_def: &CompositeTagDef,
) -> (TagValue, TagValue) {
    use crate::registry;

    let value = computed_value.clone();

    // Apply PrintConv if present to get human-readable string
    let print = if let Some(print_conv_ref) = composite_def.print_conv_ref {
        registry::apply_print_conv(print_conv_ref, &value)
    } else {
        value.clone()
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
                    let (_final_value, print_value) =
                        apply_composite_conversions(&computed_value, composite_def);

                    let composite_name = format!("Composite:{}", composite_def.name);

                    // Add to available_tags for future composite dependencies
                    // Use the PrintConv result for final output
                    available_tags.insert(composite_name.clone(), print_value.clone());
                    available_tags.insert(composite_def.name.to_string(), print_value.clone());

                    // Store in composite_tags collection - use PrintConv result
                    composite_tags.insert(composite_name.clone(), print_value);
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
