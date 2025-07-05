//! Tag management and source tracking
//!
//! This module handles tag storage, conflict resolution, and source tracking
//! for proper namespace handling and priority-based tag resolution.
//!
//! ExifTool Reference: Tag storage and conflict resolution logic

use crate::types::{TagSourceInfo, TagValue};
use tracing::debug;

use super::ExifReader;

impl ExifReader {
    /// Store tag with conflict resolution and proper namespace handling
    /// ExifTool behavior: Main EXIF tags take precedence over MakerNote tags with same ID
    pub fn store_tag_with_precedence(
        &mut self,
        tag_id: u16,
        value: TagValue,
        source_info: TagSourceInfo,
    ) {
        // Check if tag already exists
        if let Some(existing_source) = self.tag_sources.get(&tag_id) {
            // Compare priorities - higher priority wins
            if source_info.priority > existing_source.priority {
                debug!(
                    "Tag 0x{:04x}: Replacing lower priority {} with higher priority {}",
                    tag_id, existing_source.namespace, source_info.namespace
                );
                self.extracted_tags.insert(tag_id, value);
                self.tag_sources.insert(tag_id, source_info);
            } else if source_info.priority == existing_source.priority {
                // Same priority - keep first encountered (ExifTool behavior)
                debug!(
                    "Tag 0x{:04x}: Keeping first encountered {} over {}",
                    tag_id, existing_source.namespace, source_info.namespace
                );
                // Do not overwrite - keep existing
            } else {
                // Lower priority - ignore
                debug!(
                    "Tag 0x{:04x}: Ignoring lower priority {} (existing: {})",
                    tag_id, source_info.namespace, existing_source.namespace
                );
            }
        } else {
            // New tag - store it
            debug!(
                "Tag 0x{:04x}: Storing new {} tag",
                tag_id, source_info.namespace
            );
            self.extracted_tags.insert(tag_id, value);
            self.tag_sources.insert(tag_id, source_info);
        }
    }

    /// Create TagSourceInfo from IFD name with proper namespace mapping
    /// Maps legacy IFD names to proper ExifTool group names
    pub(crate) fn create_tag_source_info(&self, ifd_name: &str) -> TagSourceInfo {
        // Map IFD names to ExifTool group names
        // ExifTool: lib/Image/ExifTool/Exif.pm group mappings
        let namespace = match ifd_name {
            "Root" | "IFD0" | "IFD1" => "EXIF",
            "GPS" => "EXIF",     // GPS tags belong to EXIF group in ExifTool
            "ExifIFD" => "EXIF", // ExifIFD tags belong to EXIF group (Group0) in ExifTool
            "InteropIFD" => "EXIF",
            "MakerNotes" => "MakerNotes",
            _ => "EXIF", // Default to EXIF for unknown IFDs
        };

        let processor_name = if namespace == "MakerNotes" {
            // For MakerNotes, use a generic MakerNotes processor name
            "MakerNotes".to_string()
        } else {
            "Exif".to_string()
        };

        TagSourceInfo::new(namespace.to_string(), ifd_name.to_string(), processor_name)
    }

    /// Apply ValueConv and PrintConv conversions to a raw tag value
    /// ExifTool: lib/Image/ExifTool.pm conversion pipeline
    /// Returns tuple of (value, print) where:
    /// - value: The result after ValueConv (or raw if no ValueConv)
    /// - print: The result after PrintConv (or value.to_string() if no PrintConv)
    pub(crate) fn apply_conversions(
        &self,
        raw_value: &TagValue,
        tag_def: Option<&'static crate::generated::tags::TagDef>,
    ) -> (TagValue, String) {
        use crate::registry;

        let mut value = raw_value.clone();

        // Apply ValueConv first (if present)
        if let Some(tag_def) = tag_def {
            if let Some(value_conv_ref) = tag_def.value_conv_ref {
                value = registry::apply_value_conv(value_conv_ref, &value);
            }
        }

        // Apply PrintConv second (if present) to get human-readable string
        let print = if let Some(tag_def) = tag_def {
            if let Some(print_conv_ref) = tag_def.print_conv_ref {
                registry::apply_print_conv(print_conv_ref, &value)
            } else {
                value.to_string()
            }
        } else {
            value.to_string()
        };

        (value, print)
    }
}
