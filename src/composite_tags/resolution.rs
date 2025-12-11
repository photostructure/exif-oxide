//! Dependency resolution functions for composite tags
//!
//! This module handles the logic for determining whether composite tags can be built
//! based on available dependencies, including group prefix handling and composite
//! tag references.
//!
//! ## Three-Array System (ExifTool Compatibility)
//!
//! ExifTool populates three parallel arrays for composite tag evaluation
//! (lib/Image/ExifTool.pm:3553-3560):
//!
//! - `@raw` - Unconverted raw values directly from storage
//! - `@val` - Values after ValueConv applied
//! - `@prt` - Values after PrintConv applied (human-readable)
//!
//! We mirror this with [`TagDependencyValues`] to properly support expressions
//! that reference different conversion stages (e.g., `$prt[n]` for printed values).

use std::collections::{HashMap, HashSet};
use tracing::{debug, trace};

use crate::generated::composite_tags::CompositeTagDef;
use crate::generated::Exif_pm::main_tags::EXIF_MAIN_TAGS;
use crate::generated::GPS_pm::main_tags::GPS_MAIN_TAGS;
use crate::types::TagValue;

/// Holds the three conversion stages of a tag value for composite resolution.
///
/// This mirrors ExifTool's `@raw`, `@val`, `@prt` arrays used in composite tag
/// evaluation (lib/Image/ExifTool.pm:3553-3560).
///
/// # ExifTool Reference
///
/// ```perl
/// $raw[$_] = $$rawValue{$$val{$_}};  # Raw from storage
/// ($val[$_], $prt[$_]) = $self->GetValue($$val{$_}, 'Both');  # Converted values
/// ```
#[derive(Debug, Clone)]
pub struct TagDependencyValues {
    /// Raw unconverted value from storage (Perl's `$raw[n]`)
    pub raw: TagValue,
    /// Value after ValueConv applied (Perl's `$val[n]`)
    pub val: TagValue,
    /// Value after PrintConv applied (Perl's `$prt[n]`)
    pub prt: TagValue,
}

/// Build the initial available tags map from extracted tags with group prefixes.
///
/// This version uses the same value for raw/val/prt (simple case where conversions
/// have already been applied). For proper ExifTool compatibility with separate
/// raw/val/prt values, use [`build_available_tags_map_with_conversions`].
///
/// This is still used by code that only needs simple tag lookups.
pub fn build_available_tags_map(
    extracted_tags: &HashMap<u16, TagValue>,
    tag_sources: &HashMap<u16, crate::types::TagSourceInfo>,
) -> HashMap<String, TagDependencyValues> {
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

        let base_tag_name = if group_name == "EXIF" {
            EXIF_MAIN_TAGS
                .get(&tag_id)
                .map(|tag_def| tag_def.name.to_string())
                .or_else(|| {
                    GPS_MAIN_TAGS
                        .get(&tag_id)
                        .map(|tag_def| tag_def.name.to_string())
                })
                .unwrap_or_else(|| {
                    // Use TAG_PREFIX mechanism for unknown tags
                    let source_info = tag_sources.get(&tag_id);
                    crate::exif::ExifReader::generate_tag_prefix_name(tag_id, source_info)
                })
        } else {
            // Use TAG_PREFIX mechanism for non-EXIF tags
            let source_info = tag_sources.get(&tag_id);
            crate::exif::ExifReader::generate_tag_prefix_name(tag_id, source_info)
        };

        // Create TagDependencyValues with same value for all three (fallback behavior)
        // ExifTool: lib/Image/ExifTool.pm:3553-3560
        // In proper implementation, raw should be the unconverted value
        let dep_values = TagDependencyValues {
            raw: value.clone(),
            val: value.clone(),
            prt: value.clone(),
        };

        // Add with group prefix (e.g., "GPS:GPSLatitude")
        let prefixed_name = format!("{group_name}:{base_tag_name}");
        available_tags.insert(prefixed_name, dep_values.clone());

        // Also add without group prefix for broader matching (e.g., "GPSLatitude")
        available_tags.insert(base_tag_name, dep_values);
    }

    available_tags
}

/// Build available tags map with proper raw/val/prt separation.
///
/// This is the ExifTool-compatible version that properly populates all three
/// value stages for each tag. Use this when you need accurate `$raw[n]`,
/// `$val[n]`, `$prt[n]` array values.
///
/// # Arguments
///
/// * `raw_tags` - Raw values before any conversion (used for `@raw`)
/// * `val_tags` - Values after ValueConv (used for `@val`)
/// * `prt_tags` - Values after PrintConv (used for `@prt`)
///
/// # ExifTool Reference
///
/// lib/Image/ExifTool.pm:3553-3560:
/// ```perl
/// $raw[$_] = $$rawValue{$$val{$_}};  # Raw from storage
/// ($val[$_], $prt[$_]) = $self->GetValue($$val{$_}, 'Both');
/// ```
pub fn build_available_tags_map_with_conversions(
    raw_tags: &HashMap<String, TagValue>,
    val_tags: &HashMap<String, TagValue>,
    prt_tags: &HashMap<String, TagValue>,
) -> HashMap<String, TagDependencyValues> {
    let mut available_tags = HashMap::new();

    // Use val_tags as the canonical set of tag names
    for (tag_name, val) in val_tags {
        let raw = raw_tags.get(tag_name).cloned().unwrap_or(val.clone());
        let prt = prt_tags.get(tag_name).cloned().unwrap_or(val.clone());

        available_tags.insert(
            tag_name.clone(),
            TagDependencyValues {
                raw,
                val: val.clone(),
                prt,
            },
        );
    }

    available_tags
}

/// Check if a composite tag can be built (all required dependencies available)
/// This is the core dependency resolution logic for multi-pass building
/// ExifTool: lib/Image/ExifTool.pm:3929-4115 BuildCompositeTags
pub fn can_build_composite(
    composite_def: &CompositeTagDef,
    available_tags: &HashMap<String, TagDependencyValues>,
    built_composites: &HashSet<String>,
) -> bool {
    // Check inhibit tags first - if any inhibit tag exists, we can't build
    // ExifTool: lib/Image/ExifTool.pm:4034-4036
    for inhibit_tag in composite_def.inhibit {
        if is_dependency_available(inhibit_tag, available_tags, built_composites) {
            trace!(
                "Composite {} inhibited by presence of {}",
                composite_def.name,
                inhibit_tag
            );
            return false;
        }
    }

    // Special case for ImageSize: ExifTool can build it with just desired dependencies
    // because the ValueConv has fallback logic
    if composite_def.name == "ImageSize" {
        debug!("Checking ImageSize special case dependencies");
        // Can build if we have any of: RawImageCroppedSize, ExifImageWidth+Height, or ImageWidth+Height
        if is_dependency_available("RawImageCroppedSize", available_tags, built_composites) {
            debug!("ImageSize: Found RawImageCroppedSize");
            return true;
        }
        if is_dependency_available("ExifImageWidth", available_tags, built_composites)
            && is_dependency_available("ExifImageHeight", available_tags, built_composites)
        {
            debug!("ImageSize: Found ExifImageWidth + ExifImageHeight");
            return true;
        }
        if is_dependency_available("ImageWidth", available_tags, built_composites)
            && is_dependency_available("ImageHeight", available_tags, built_composites)
        {
            debug!("ImageSize: Found ImageWidth + ImageHeight");
            return true;
        }
        debug!("ImageSize: No suitable dependencies found");
        return false;
    }

    // Special case: if there are no required dependencies, check if we have any desired ones
    if composite_def.require.is_empty() && !composite_def.desire.is_empty() {
        // Need at least one desired dependency
        for tag_name in composite_def.desire {
            if is_dependency_available(tag_name, available_tags, built_composites) {
                return true; // At least one desired dependency available
            }
        }
        trace!(
            "No desired dependencies available for {}: {:?}",
            composite_def.name,
            composite_def.desire
        );
        return false;
    }

    // Check all required dependencies
    for tag_name in composite_def.require {
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

/// Resolve dependency arrays for composite tag function calls
/// ExifTool: lib/Image/ExifTool.pm:3553-3560 - populates @raw, @val, @prt arrays
///
/// Returns tuple of (vals, prts, raws) where:
/// - vals: Values after ValueConv applied (Perl's @val)
/// - prts: Values after PrintConv applied (Perl's @prt)
/// - raws: Raw unconverted values (Perl's @raw)
///
/// The arrays are ordered by dependency index: require deps first, then desire deps
///
/// # ExifTool Reference
///
/// lib/Image/ExifTool.pm:3553-3560:
/// ```perl
/// foreach (keys %$val) {
///     next unless defined $$val{$_};
///     $raw[$_] = $$rawValue{$$val{$_}};
///     ($val[$_], $prt[$_]) = $self->GetValue($$val{$_}, 'Both');
///     # ...
/// }
/// ```
pub fn resolve_dependency_arrays(
    composite_def: &CompositeTagDef,
    available_tags: &HashMap<String, TagDependencyValues>,
    built_composites: &HashSet<String>,
) -> (Vec<TagValue>, Vec<TagValue>, Vec<TagValue>) {
    let mut vals = Vec::new();
    let mut prts = Vec::new();
    let mut raws = Vec::new();

    // Process required dependencies first (these map to $val[0], $val[1], etc.)
    // ExifTool: Require'd tags are indexed 0, 1, 2, ... in order
    for tag_name in composite_def.require {
        let dep_values = resolve_tag_dependency(tag_name, available_tags, built_composites);
        match dep_values {
            Some(dv) => {
                raws.push(dv.raw);
                vals.push(dv.val);
                prts.push(dv.prt);
            }
            None => {
                // Tag not found - use Empty for all three
                raws.push(TagValue::Empty);
                vals.push(TagValue::Empty);
                prts.push(TagValue::Empty);
            }
        }
    }

    // Then process desired dependencies (these continue the indexing)
    // ExifTool: Desire'd tags continue the indexing after Require'd tags
    for tag_name in composite_def.desire {
        let dep_values = resolve_tag_dependency(tag_name, available_tags, built_composites);
        match dep_values {
            Some(dv) => {
                raws.push(dv.raw);
                vals.push(dv.val);
                prts.push(dv.prt);
            }
            None => {
                raws.push(TagValue::Empty);
                vals.push(TagValue::Empty);
                prts.push(TagValue::Empty);
            }
        }
    }

    (vals, prts, raws)
}

/// Check if a specific dependency (tag name) is available using ExifTool's dynamic resolution
/// ExifTool: lib/Image/ExifTool.pm:3977-4005 BuildCompositeTags dependency resolution
/// ExifTool: lib/Image/ExifTool.pm:4006-4026 tag value lookup from $$rawValue{$reqTag}
pub fn is_dependency_available(
    tag_name: &str,
    available_tags: &HashMap<String, TagDependencyValues>,
    built_composites: &HashSet<String>,
) -> bool {
    resolve_tag_dependency(tag_name, available_tags, built_composites).is_some()
}

/// Resolve a tag dependency using ExifTool's dynamic lookup algorithm
/// ExifTool: lib/Image/ExifTool.pm:4006-4026 - dependency resolution from rawValue hash
/// ExifTool: lib/Image/ExifTool.pm:3977-3983 - composite-to-composite dependency handling
/// ExifTool: lib/Image/ExifTool.pm:4027-4055 - group matching and priority resolution
///
/// Returns the full [`TagDependencyValues`] with raw/val/prt values for the tag.
pub fn resolve_tag_dependency(
    tag_name: &str,
    available_tags: &HashMap<String, TagDependencyValues>,
    built_composites: &HashSet<String>,
) -> Option<TagDependencyValues> {
    // Step 1: Check if the tag is a composite that has already been built
    // ExifTool: lib/Image/ExifTool.pm:3977-3983 composite dependency handling
    if built_composites.contains(tag_name) {
        // Look for the built composite in available_tags
        let composite_name = format!("Composite:{tag_name}");
        if let Some(value) = available_tags.get(&composite_name) {
            return Some(value.clone());
        }
        // Also check without prefix (we store both formats)
        if let Some(value) = available_tags.get(tag_name) {
            return Some(value.clone());
        }
    }

    // Step 2: Direct lookup in available tags (highest priority)
    // ExifTool: lib/Image/ExifTool.pm:4006-4026 $$rawValue{$reqTag} lookup
    if let Some(value) = available_tags.get(tag_name) {
        return Some(value.clone());
    }

    // Step 3: Check with group prefixes following ExifTool's precedence
    // ExifTool: lib/Image/ExifTool.pm:4027-4055 group matching logic
    // Priority order matches ExifTool's typical group precedence
    for prefix in &["EXIF", "GPS", "MakerNotes", "Composite"] {
        let prefixed_name = format!("{prefix}:{tag_name}");
        if let Some(value) = available_tags.get(&prefixed_name) {
            return Some(value.clone());
        }
    }

    // Step 3.5: Handle GPS/EXIF group equivalence
    // ExifTool: GPS tags have Group0=EXIF, Group1=GPS, making them accessible as both
    // "GPS:GPSTagName" and "EXIF:GPSTagName" - lib/Image/ExifTool/GPS.pm:52
    if tag_name.starts_with("GPS:") {
        // Try EXIF: prefix for GPS: requests (GPS:GPSLatitude -> EXIF:GPSLatitude)
        let exif_tag_name = tag_name.replace("GPS:", "EXIF:");
        trace!(
            "GPS/EXIF equivalence: Looking for {} as {}",
            tag_name,
            exif_tag_name
        );
        if let Some(value) = available_tags.get(&exif_tag_name) {
            trace!(
                "GPS/EXIF equivalence: Found {} -> {}",
                tag_name,
                exif_tag_name
            );
            return Some(value.clone());
        }
    } else if tag_name.starts_with("EXIF:GPS") {
        // Try GPS: prefix for EXIF: requests (EXIF:GPSLatitude -> GPS:GPSLatitude)
        let gps_tag_name = tag_name.replace("EXIF:", "GPS:");
        trace!(
            "EXIF/GPS equivalence: Looking for {} as {}",
            tag_name,
            gps_tag_name
        );
        if let Some(value) = available_tags.get(&gps_tag_name) {
            trace!(
                "EXIF/GPS equivalence: Found {} -> {}",
                tag_name,
                gps_tag_name
            );
            return Some(value.clone());
        }
    }

    // Step 4: Try manual computation for special cases
    // This handles cases like "ISO" where ExifTool would find EXIF:ISO tag 0x8827
    // but we want to provide consolidated ISO from multiple sources
    if let Some(computed_value) = try_manual_tag_computation(tag_name, available_tags) {
        // For computed values, we use the same value for all three stages
        return Some(TagDependencyValues {
            raw: computed_value.clone(),
            val: computed_value.clone(),
            prt: computed_value,
        });
    }

    None
}

/// Attempt manual computation for tags that need consolidation logic
/// This handles cases where ExifTool uses direct tag lookup but we provide enhanced computation
/// ExifTool equivalent: direct tag access, but we add consolidation for user convenience
fn try_manual_tag_computation(
    tag_name: &str,
    available_tags: &HashMap<String, TagDependencyValues>,
) -> Option<TagValue> {
    // Convert to simple TagValue map using `.val` (ValueConv'd values).
    // See orchestration.rs::try_manual_composite_computation for rationale.
    let simple_map: HashMap<String, TagValue> = available_tags
        .iter()
        .map(|(k, v)| (k.clone(), v.val.clone()))
        .collect();

    match tag_name {
        // ISO consolidation - provides unified ISO value from multiple sources
        // ExifTool: searches for any tag named "ISO" in rawValue hash
        // We enhance this by consolidating from multiple ISO sources
        "ISO" => {
            // Import the compute_iso function from composite_fallbacks
            use crate::core::composite_fallbacks::compute_iso;
            compute_iso(&simple_map)
        }
        _ => None,
    }
}
