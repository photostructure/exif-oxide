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

    // GPSPosition special case.
    // ExifTool (Exif.pm:5290) builds GPSPosition from the SIGNED Composite
    // GPSLatitude/GPSLongitude ValueConv results ('"$val[0] $val[1]"'). Its Require list is only
    // [GPSLatitude, GPSLongitude] (no *Ref tags), so the dependency arrays cannot carry the
    // west/south sign, and the signed GPS-module composites lose the name-keyed registry
    // collision with the Sony/QuickTime GPSLatitude defs. Compute directly from the full tag map
    // so the sign is applied and each coordinate is stringified with Perl's %.15g formatting.
    if composite_def.name == "GPSPosition" {
        let simple_map: HashMap<String, TagValue> = available_tags
            .iter()
            .map(|(k, v)| (k.clone(), v.val.clone()))
            .collect();
        return crate::core::composite_fallbacks::compute_gps_position(&simple_map);
    }

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

/// Return true when this composite must be deferred to a later pass because it
/// references another Composite tag that has not been built (or dropped) yet.
///
/// ExifTool: lib/Image/ExifTool.pm:4044-4053 — a "Composite:Name" prefixed
/// dependency defers while `$notBuilt{$name}`, except an Inhibit dependency
/// once `$allBuilt` is set. lib/Image/ExifTool.pm:4074-4078 — an unprefixed
/// non-Inhibit dependency defers while its name matches a not-yet-built
/// Composite tag.
fn must_defer_composite(
    composite_def: &CompositeTagDef,
    not_built: &HashSet<&str>,
    all_built: bool,
) -> bool {
    let deps = composite_def
        .require
        .iter()
        .map(|dep| (dep, false))
        .chain(composite_def.desire.iter().map(|dep| (dep, false)))
        .chain(composite_def.inhibit.iter().map(|dep| (dep, true)));

    for (dep, is_inhibit) in deps {
        // ExifTool matches `/^(.*):(.+)/` (greedy), i.e. splits at the LAST colon
        if let Some((group, name)) = dep.rsplit_once(':') {
            // Only an explicit Composite group prefix defers
            // ExifTool: lib/Image/ExifTool.pm:4046
            if group == "Composite" && not_built.contains(name) {
                // ExifTool: lib/Image/ExifTool.pm:4049 - once allBuilt is set,
                // stop deferring for Inhibit dependencies
                if !(is_inhibit && all_built) {
                    return true;
                }
            }
        } else if !is_inhibit && not_built.contains(dep) {
            // ExifTool: lib/Image/ExifTool.pm:4074-4078
            return true;
        }
    }
    false
}

/// Mirror ExifTool's undefined `$found` state: a composite with no Require'd
/// tags and none of its Desire'd/Inhibit'd tags present "can't be built anyway"
/// and is removed from `%notBuilt` so later tags stop deferring on it.
///
/// ExifTool: lib/Image/ExifTool.pm:4010-4092 ($found bookkeeping) and
/// lib/Image/ExifTool.pm:4124-4126 (`elsif (not defined $found)`).
fn composite_cannot_be_built_anyway(
    composite_def: &CompositeTagDef,
    available_tags: &HashMap<String, TagDependencyValues>,
    built_composites: &HashSet<String>,
) -> bool {
    if !composite_def.require.is_empty() {
        // A Require'd dependency always resolves $found to 0 or 1
        return false;
    }
    if composite_def.desire.is_empty() && composite_def.inhibit.is_empty() {
        // No dependencies at all => $found = 1 (ExifTool.pm:4013-4014)
        return false;
    }
    for dep in composite_def
        .desire
        .iter()
        .chain(composite_def.inhibit.iter())
    {
        if super::resolution::is_dependency_available(dep, available_tags, built_composites) {
            return false;
        }
    }
    true
}

/// Multi-pass composite tag resolution and computation
/// This is the main entry point for building all composite tags
///
/// ExifTool: lib/Image/ExifTool.pm:3969-4162 BuildCompositeTags
///
/// Takes a map of available tags with their raw/val/prt values and returns
/// computed composite tags as simple TagValue (the print value).
///
/// Composites are attempted in ExifTool's alphabetical table-key order, and a
/// composite that depends on another Composite tag is deferred until that tag
/// has been built (or dropped as unbuildable). Both points are required for
/// deterministic values: without them, whether e.g. FocalLength35efl sees
/// ScaleFactor35efl depends on HashMap iteration order, which is randomized
/// per process.
pub fn resolve_and_compute_composites(
    mut available_tags: HashMap<String, TagDependencyValues>,
) -> HashMap<String, TagValue> {
    let mut composite_tags = HashMap::new();
    let mut built_composites: HashSet<String> = HashSet::new();

    // ExifTool: lib/Image/ExifTool.pm:3984 `my @tagList = sort keys %$compTable;`
    // The accumulated Composite table is keyed "<Module>-<TagID>"
    // (AddCompositeTags, lib/Image/ExifTool.pm:5769-5793), so build order is
    // alphabetical on that string.
    let mut tag_list: Vec<&CompositeTagDef> = COMPOSITE_TAGS.values().copied().collect();
    tag_list.sort_by_key(|def| format!("{}-{}", def.module, def.name));

    debug!(
        "Starting multi-pass composite building with {} composites",
        tag_list.len()
    );

    // ExifTool: lib/Image/ExifTool.pm:3987 `my (%cache, $allBuilt);`
    let mut all_built = false;
    let mut pass = 0usize;

    loop {
        pass += 1;
        // ExifTool: lib/Image/ExifTool.pm:3990-3993 - %notBuilt is rebuilt from
        // the current tag list at the start of every pass
        let mut not_built: HashSet<&str> = tag_list.iter().map(|def| def.name).collect();
        let mut deferred_composites: Vec<&CompositeTagDef> = Vec::new();
        let tag_count = tag_list.len();
        if tag_count == 0 {
            break;
        }

        trace!("Pass {}: Processing {} composites", pass, tag_count);

        for composite_def in tag_list {
            // Our registry is keyed by name, so a name can only be built once
            if built_composites.contains(composite_def.name) {
                trace!("Skipping {} - already built", composite_def.name);
                continue;
            }

            // Defer while this composite depends on a Composite tag that is
            // still pending in this pass
            // ExifTool: lib/Image/ExifTool.pm:4044-4053, 4074-4078
            if must_defer_composite(composite_def, &not_built, all_built) {
                trace!(
                    "Deferring {} - depends on a Composite tag not yet built",
                    composite_def.name
                );
                deferred_composites.push(composite_def);
                continue;
            }

            let can_build = can_build_composite(composite_def, &available_tags, &built_composites);

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

                    // ExifTool: lib/Image/ExifTool.pm:4109 - later tags in this
                    // pass no longer defer on this name
                    not_built.remove(composite_def.name);

                    debug!("Built composite tag: {} (pass {})", composite_name, pass);
                } else {
                    // Computation failed. ExifTool has no failure mode at this
                    // point (its ValueConv is evaluated lazily after FoundTag),
                    // so treat the tag as handled: drop it from the list and
                    // stop dependents from waiting on it.
                    trace!("Failed to compute {} - dropping", composite_def.name);
                    not_built.remove(composite_def.name);
                }
            } else {
                // Not buildable. ExifTool drops the tag from the list here
                // (only tags waiting on other Composite tags are carried to the
                // next pass). A tag whose Desire'd/Inhibit'd dependencies are
                // all absent additionally leaves %notBuilt so later tags in
                // this pass stop deferring on it.
                // ExifTool: lib/Image/ExifTool.pm:4108-4126
                if composite_cannot_be_built_anyway(
                    composite_def,
                    &available_tags,
                    &built_composites,
                ) {
                    trace!(
                        "Composite {} can't be built anyway - removed from notBuilt",
                        composite_def.name
                    );
                    not_built.remove(composite_def.name);
                } else {
                    trace!(
                        "Missing required dependency for {} - dropping",
                        composite_def.name
                    );
                }
            }
        }

        trace!(
            "Pass {} complete: {} deferred of {}",
            pass,
            deferred_composites.len(),
            tag_count
        );

        // ExifTool: lib/Image/ExifTool.pm:4149 `last unless @deferredTags;`
        if deferred_composites.is_empty() {
            break;
        }

        // ExifTool: lib/Image/ExifTool.pm:4150-4158 - when EVERY tag was
        // deferred, try once more ignoring Composite Inhibit deferrals; if that
        // also defers everything, it is a circular dependency
        if deferred_composites.len() == tag_count {
            if all_built {
                warn!("Circular dependency in Composite tags");
                handle_unresolved_composites(&deferred_composites);
                break;
            }
            all_built = true;
        }

        tag_list = deferred_composites;
    }

    debug!(
        "Composite building complete: {} total composites built",
        built_composites.len()
    );

    composite_tags
}
