//! Dependency resolution functions for composite tags
//!
//! This module handles the logic for determining whether composite tags can be built
//! based on available dependencies, including group prefix handling and composite
//! tag references.

use std::collections::{HashMap, HashSet};
use tracing::{debug, trace};

use crate::generated::CompositeTagDef;
use crate::generated::TAG_BY_ID;
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
