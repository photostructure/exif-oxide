//! Generic subdirectory processing with PrintConv application
//!
//! This module provides universal subdirectory processing that works with any manufacturer's
//! tag kit system. It connects subdirectory binary data extraction with PrintConv formatting
//! to produce human-readable tag values instead of raw arrays.
//!
//! **Trust ExifTool**: This implementation follows ExifTool's ProcessBinaryData pattern
//! of automatically applying PrintConv after extracting raw binary values.

use crate::exif::ExifReader;
use crate::tiff_types::ByteOrder;
use crate::types::{Result, TagValue};
use tracing::{debug, warn};

/// Generic subdirectory processing function that works with any manufacturer's tag kit
///
/// This function:
/// 1. Finds tags with subdirectory processing using the manufacturer's `has_subdirectory()` function
/// 2. Extracts raw binary data using the manufacturer's `process_subdirectory()` function  
/// 3. Applies PrintConv using the manufacturer's `apply_print_conv()` function
/// 4. Stores formatted results with proper synthetic IDs
///
/// # Arguments
/// * `exif_reader` - The ExifReader containing extracted tags
/// * `manufacturer` - Manufacturer name (e.g., "Canon", "Nikon", "Sony")
/// * `namespace` - Tag namespace (e.g., "MakerNotes", "EXIF")
/// * `has_subdirectory_fn` - Function to check if tag has subdirectory processing
/// * `process_subdirectory_fn` - Function to extract subdirectory binary data
/// * `apply_print_conv_fn` - Function to apply PrintConv formatting
/// * `find_tag_id_by_name_fn` - Function to look up tag ID by name for PrintConv
pub fn process_subdirectories_with_printconv<H, P, A, F>(
    exif_reader: &mut ExifReader,
    manufacturer: &str,
    namespace: &str,
    has_subdirectory_fn: H,
    process_subdirectory_fn: P,
    apply_print_conv_fn: A,
    find_tag_id_by_name_fn: F,
) -> Result<()>
where
    H: Fn(u32) -> bool,
    P: Fn(u32, &TagValue, ByteOrder) -> Result<std::collections::HashMap<String, TagValue>>,
    A: Fn(
        u32,
        &TagValue,
        &mut crate::expressions::ExpressionEvaluator,
        &mut Vec<String>,
        &mut Vec<String>,
    ) -> TagValue,
    F: Fn(&str) -> Option<u32>,
{
    debug!(
        "Processing {} subdirectory tags with PrintConv",
        manufacturer
    );

    // Collect tags that have subdirectory processing
    let mut subdirectory_tags = Vec::new();

    for (&(tag_id, ref tag_namespace), tag_value) in &exif_reader.extracted_tags {
        // Check if this is a tag from the specified manufacturer and namespace
        if let Some(source_info) = exif_reader
            .tag_sources
            .get(&(tag_id, tag_namespace.clone()))
        {
            if source_info.namespace == namespace && source_info.ifd_name.starts_with(manufacturer)
            {
                // Check if this tag has subdirectory processing
                let has_subdir = has_subdirectory_fn(tag_id as u32);
                debug!(
                    "{} tag 0x{:04x} has_subdirectory: {}",
                    manufacturer, tag_id, has_subdir
                );
                if has_subdir {
                    debug!(
                        "Found {} tag 0x{:04x} with subdirectory processing",
                        manufacturer, tag_id
                    );
                    subdirectory_tags.push((tag_id, tag_value.clone(), source_info.clone()));
                }
            }
        }
    }

    // Process each subdirectory tag
    for (tag_id, tag_value, _source_info) in subdirectory_tags {
        debug!(
            "Processing subdirectory for {} tag 0x{:04x}",
            manufacturer, tag_id
        );

        // Get byte order from TIFF header
        let byte_order = exif_reader
            .header
            .as_ref()
            .map(|h| h.byte_order)
            .unwrap_or(ByteOrder::LittleEndian);

        match process_subdirectory_fn(tag_id as u32, &tag_value, byte_order) {
            Ok(extracted_tags) => {
                debug!(
                    "Extracted {} tags from {} subdirectory 0x{:04x}",
                    extracted_tags.len(),
                    manufacturer,
                    tag_id
                );

                // Initialize counter for deterministic synthetic ID generation
                let mut synthetic_counter: u16 = 0;

                // Store each extracted tag with PrintConv applied
                for (tag_name, value) in extracted_tags {
                    // Skip tags marked as Unknown (matching ExifTool's default behavior)
                    if tag_name.contains("Unknown") {
                        debug!("Skipping unknown {} tag: {}", manufacturer, tag_name);
                        continue;
                    }

                    // Generate a deterministic synthetic tag ID for the extracted tag
                    // Uses parent tag ID's upper bits and counter for lower bits
                    // This ensures unique IDs for each tag in the subdirectory
                    let synthetic_id = 0x8000 | (tag_id & 0x7F00) | (synthetic_counter & 0xFF);
                    synthetic_counter += 1;

                    // Check bounds to prevent overflow into other ID ranges
                    if synthetic_counter > 255 {
                        warn!(
                            "Too many synthetic tags extracted from {} subdirectory 0x{:04x}, some may be lost",
                            manufacturer, tag_id
                        );
                        break;
                    }

                    debug!(
                        "Storing extracted {} tag '{}' from subdirectory 0x{:04x} with synthetic ID 0x{:04x}",
                        manufacturer, tag_name, tag_id, synthetic_id
                    );

                    // Add debug assertion to catch any future collision bugs
                    debug_assert!(
                        !exif_reader.synthetic_tag_names.contains_key(&synthetic_id),
                        "Synthetic tag ID collision detected: 0x{:04x} for {} tag '{}'",
                        synthetic_id,
                        manufacturer,
                        tag_name
                    );

                    // Apply PrintConv to format the extracted value
                    let formatted_value =
                        if let Some(kit_tag_id) = find_tag_id_by_name_fn(&tag_name) {
                            debug!(
                                "Found {} tag kit ID {} for tag '{}'",
                                manufacturer, kit_tag_id, tag_name
                            );

                            // Create ExpressionEvaluator and error tracking
                            let mut evaluator = crate::expressions::ExpressionEvaluator::new();
                            let mut errors = Vec::new();
                            let mut warnings = Vec::new();

                            // Apply PrintConv using the manufacturer's tag kit system
                            let result = apply_print_conv_fn(
                                kit_tag_id,
                                &value,
                                &mut evaluator,
                                &mut errors,
                                &mut warnings,
                            );

                            // Log any warnings or errors from PrintConv processing
                            for warning in warnings {
                                debug!(
                                    "{} subdirectory PrintConv warning for '{}': {}",
                                    manufacturer, tag_name, warning
                                );
                            }
                            for error in errors {
                                debug!(
                                    "{} subdirectory PrintConv error for '{}': {}",
                                    manufacturer, tag_name, error
                                );
                            }

                            debug!(
                                "Applied {} PrintConv to '{}': {:?} -> {:?}",
                                manufacturer, tag_name, value, result
                            );
                            result
                        } else {
                            debug!(
                                "No {} tag kit mapping found for '{}', using raw value",
                                manufacturer, tag_name
                            );
                            value
                        };

                    // Store the extracted tag with proper namespace
                    let full_tag_name = ensure_group_prefix(&tag_name, namespace);
                    exif_reader
                        .synthetic_tag_names
                        .insert(synthetic_id, full_tag_name.clone());

                    debug!(
                        "Storing {} tag 0x{:04x} with synthetic name mapping: '{}'",
                        manufacturer, synthetic_id, full_tag_name
                    );

                    exif_reader.store_tag_with_precedence(
                        synthetic_id,
                        formatted_value,
                        crate::types::TagSourceInfo::new(
                            namespace.to_string(),
                            manufacturer.to_string(),
                            format!("{}::SubDirectory", manufacturer),
                        ),
                    );
                }

                // Remove the original array tag since we've expanded it
                let tag_key = (tag_id, namespace.to_string());
                exif_reader.extracted_tags.remove(&tag_key);
                exif_reader.tag_sources.remove(&tag_key);
            }
            Err(e) => {
                debug!(
                    "Failed to process subdirectory for {} tag 0x{:04x}: {}",
                    manufacturer, tag_id, e
                );
                // Keep the original array data if subdirectory processing fails
            }
        }
    }

    debug!(
        "{} subdirectory processing with PrintConv completed",
        manufacturer
    );
    Ok(())
}

/// Helper function to ensure tag names have proper group prefix
/// This matches ExifTool's group prefixing behavior
fn ensure_group_prefix(tag_name: &str, group: &str) -> String {
    if tag_name.starts_with(&format!("{}:", group)) {
        tag_name.to_string()
    } else {
        format!("{}:{}", group, tag_name)
    }
}
