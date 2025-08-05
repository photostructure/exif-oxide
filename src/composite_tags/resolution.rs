//! Dependency resolution functions for composite tags
//!
//! This module handles the logic for determining whether composite tags can be built
//! based on available dependencies, including group prefix handling and composite
//! tag references.

use std::collections::{HashMap, HashSet};
use tracing::{debug, trace};

use crate::generated::exif_pm::main_tags::MAIN_TAGS as EXIF_MAIN_TAGS;
use crate::generated::gps_pm::main_tags::MAIN_TAGS as GPS_MAIN_TAGS;
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

/// Check if a specific dependency (tag name) is available using ExifTool's dynamic resolution
/// ExifTool: lib/Image/ExifTool.pm:3977-4005 BuildCompositeTags dependency resolution
/// ExifTool: lib/Image/ExifTool.pm:4006-4026 tag value lookup from $$rawValue{$reqTag}
pub fn is_dependency_available(
    tag_name: &str,
    available_tags: &HashMap<String, TagValue>,
    built_composites: &HashSet<&str>,
) -> bool {
    resolve_tag_dependency(tag_name, available_tags, built_composites).is_some()
}

/// Resolve a tag dependency using ExifTool's dynamic lookup algorithm
/// ExifTool: lib/Image/ExifTool.pm:4006-4026 - dependency resolution from rawValue hash
/// ExifTool: lib/Image/ExifTool.pm:3977-3983 - composite-to-composite dependency handling
/// ExifTool: lib/Image/ExifTool.pm:4027-4055 - group matching and priority resolution
pub fn resolve_tag_dependency(
    tag_name: &str,
    available_tags: &HashMap<String, TagValue>,
    built_composites: &HashSet<&str>,
) -> Option<TagValue> {
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
        return Some(computed_value);
    }

    None
}

/// Attempt manual computation for tags that need consolidation logic
/// This handles cases where ExifTool uses direct tag lookup but we provide enhanced computation
/// ExifTool equivalent: direct tag access, but we add consolidation for user convenience
fn try_manual_tag_computation(
    tag_name: &str,
    available_tags: &HashMap<String, TagValue>,
) -> Option<TagValue> {
    match tag_name {
        // ISO consolidation - provides unified ISO value from multiple sources
        // ExifTool: searches for any tag named "ISO" in rawValue hash
        // We enhance this by consolidating from multiple ISO sources
        "ISO" => {
            // Import the compute_iso function from implementations
            use crate::composite_tags::implementations::compute_iso;
            compute_iso(available_tags)
        }
        _ => None,
    }
}
