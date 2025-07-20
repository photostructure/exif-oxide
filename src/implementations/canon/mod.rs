//! Canon-specific EXIF processing coordinator
//!
//! This module coordinates Canon manufacturer-specific processing,
//! dispatching to specialized sub-modules for different aspects.
//!
//! **Trust ExifTool**: This code translates ExifTool's Canon processing verbatim
//! without any improvements or simplifications. Every algorithm, magic number, and
//! quirk is copied exactly as documented in the ExifTool source.
//!
//! Primary ExifTool References:
//! - lib/Image/ExifTool/Canon.pm - Canon tag tables and processing
//! - lib/Image/ExifTool/MakerNotes.pm - Canon MakerNote detection and offset fixing

pub mod af_info;
pub mod binary_data;
pub mod offset_schemes;
pub mod tags;
pub mod tiff_footer;

// Re-export commonly used binary_data functions for easier access
pub use binary_data::{
    create_canon_camera_settings_table, extract_binary_data_tags, extract_binary_value,
    extract_camera_settings, find_canon_camera_settings_tag,
};
// Re-export offset scheme functions
pub use offset_schemes::{detect_canon_signature, detect_offset_scheme, CanonOffsetScheme};
// Re-export tag name functions
pub use tags::get_canon_tag_name;

use crate::tiff_types::ByteOrder;
use crate::types::Result;
use tracing::debug;

// CameraSettings functions are provided by the binary_data module

// extract_camera_settings function is provided by the binary_data module

/// Process Canon MakerNotes data
/// ExifTool: lib/Image/ExifTool/Canon.pm Canon MakerNote processing
/// This function processes Canon MakerNotes as an IFD structure to extract Canon-specific tags
pub fn process_canon_makernotes(
    exif_reader: &mut crate::exif::ExifReader,
    dir_start: usize,
    size: usize,
) -> Result<()> {
    use crate::types::DirectoryInfo;

    debug!(
        "Processing Canon MakerNotes: start={:#x}, size={}",
        dir_start, size
    );

    // Canon MakerNotes are structured as a standard IFD with Canon-specific tag processing
    // ExifTool: Canon.pm Main table processes Canon tags as subdirectories
    // Key insight: Canon tags need Canon-specific processors, not generic TIFF processing

    // First, process as standard IFD to extract the raw Canon tag structure
    let dir_info = DirectoryInfo {
        name: "Canon".to_string(),
        dir_start,
        dir_len: size,
        base: exif_reader.base,
        data_pos: 0,
        allow_reprocess: true,
    };

    // Process the Canon MakerNotes IFD to extract individual Canon tags
    // This extracts the basic Canon tag structure (tag IDs and data)
    exif_reader.process_subdirectory(&dir_info)?;

    // CRITICAL: Now process specific Canon binary data tags using existing Canon processors
    // ExifTool: Canon.pm processes each Canon tag through specific SubDirectory processors

    // Process Canon binary data tags directly using existing implementations
    process_canon_binary_data_with_existing_processors(exif_reader, dir_start, size)?;

    debug!("Canon MakerNotes processing completed");
    Ok(())
}

/// Process Canon binary data using existing Canon processors and generated lookup tables
/// ExifTool: Canon.pm processes Canon maker notes through specific binary data processors
fn process_canon_binary_data_with_existing_processors(
    exif_reader: &mut crate::exif::ExifReader,
    dir_start: usize,
    size: usize,
) -> Result<()> {
    debug!("Processing Canon binary data using existing Canon processors");

    // Collect tags to store after processing (to avoid borrow issues)
    let mut tags_to_store = Vec::new();

    // Get the raw maker note data to process with Canon-specific processors
    let full_data = exif_reader.get_data();
    let data = &full_data[dir_start..dir_start + size];
    let byte_order = ByteOrder::LittleEndian; // Canon typically uses little-endian

    // Process Canon CameraSettings (tag 0x0001) using existing Canon processor
    // ExifTool: Canon.pm CanonCameraSettings SubDirectory processing
    if let Some(camera_settings_data) = find_canon_tag_data(data, 0x0001) {
        debug!("Processing Canon CameraSettings using existing Canon processor");

        // Use existing extract_camera_settings function with generated lookup tables
        match extract_camera_settings(
            camera_settings_data,
            0, // offset within the camera settings data
            camera_settings_data.len(),
            byte_order,
        ) {
            Ok(camera_settings) => {
                debug!(
                    "Extracted {} Canon CameraSettings tags",
                    camera_settings.len()
                );

                // Apply generated PrintConv lookup tables to camera settings
                for (tag_name, tag_value) in camera_settings {
                    let converted_value = apply_camera_settings_print_conv(&tag_name, &tag_value);
                    let full_tag_name = format!("MakerNotes:{tag_name}");

                    debug!(
                        "Canon CameraSettings: {} = {:?}",
                        full_tag_name, converted_value
                    );

                    // Generate a synthetic tag ID for Canon CameraSettings tags
                    // Using high range (0xC000+) to avoid conflicts with standard tags
                    // Add hash of tag name to ensure uniqueness
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};

                    let mut hasher = DefaultHasher::new();
                    tag_name.hash(&mut hasher);
                    let hash = hasher.finish();
                    let synthetic_id = 0xC000 + ((hash as u16) & 0x0FFF);

                    // Collect tag to store later
                    tags_to_store.push((synthetic_id, converted_value, full_tag_name));
                }
            }
            Err(e) => {
                debug!("Failed to extract Canon CameraSettings: {}", e);
            }
        }
    }

    // Process other Canon binary data tags using similar approach
    process_other_canon_binary_tags(data, byte_order)?;

    // Now store all collected tags (after all borrows are released)
    for (synthetic_id, converted_value, full_tag_name) in tags_to_store {
        exif_reader
            .extracted_tags
            .insert(synthetic_id, converted_value);
        debug!(
            "Stored Canon tag {} with synthetic ID 0x{:04X}",
            full_tag_name, synthetic_id
        );
    }

    Ok(())
}

/// Find specific Canon tag data within the maker note IFD
/// ExifTool: Canon.pm searches for specific tag IDs within the Canon IFD
fn find_canon_tag_data(data: &[u8], tag_id: u16) -> Option<&[u8]> {
    use crate::tiff_types::ByteOrder;

    debug!(
        "Searching for Canon tag {:#x} in {} bytes of data",
        tag_id,
        data.len()
    );

    // Canon typically uses little-endian byte order
    let byte_order = ByteOrder::LittleEndian;

    // Parse Canon IFD structure
    // IFD format: 2-byte count + n*12-byte entries + 4-byte next offset
    if data.len() < 2 {
        debug!("Data too short for IFD header");
        return None;
    }

    // Read number of directory entries
    let num_entries = match byte_order.read_u16(data, 0) {
        Ok(count) => count,
        Err(e) => {
            debug!("Failed to read Canon IFD entry count: {}", e);
            return None;
        }
    };
    debug!("Canon IFD has {} entries", num_entries);

    // Calculate required size
    let ifd_size = 2 + (num_entries as usize * 12) + 4;
    if data.len() < ifd_size {
        debug!(
            "Data too short for complete IFD: need {}, have {}",
            ifd_size,
            data.len()
        );
        return None;
    }

    // Search through IFD entries for the requested tag
    for i in 0..num_entries {
        // Each IFD entry is 12 bytes:
        // 0-2: tag ID
        // 2-4: format
        // 4-8: count
        // 8-12: value/offset
        let entry_offset = 2 + (i as usize * 12);

        let entry_tag = match byte_order.read_u16(data, entry_offset) {
            Ok(tag) => tag,
            Err(e) => {
                debug!("Failed to read tag at entry {}: {}", i, e);
                continue;
            }
        };

        if entry_tag == tag_id {
            debug!("Found Canon tag {:#x} at entry {}", tag_id, i);

            // Read format and count
            let format = match byte_order.read_u16(data, entry_offset + 2) {
                Ok(fmt) => fmt,
                Err(e) => {
                    debug!("Failed to read format for tag {:#x}: {}", tag_id, e);
                    return None;
                }
            };

            let count = match byte_order.read_u32(data, entry_offset + 4) {
                Ok(c) => c,
                Err(e) => {
                    debug!("Failed to read count for tag {:#x}: {}", tag_id, e);
                    return None;
                }
            };

            debug!("Tag {:#x}: format={}, count={}", tag_id, format, count);

            // Calculate data size based on format
            let component_size = match format {
                1 => 1,  // BYTE
                2 => 1,  // ASCII
                3 => 2,  // SHORT
                4 => 4,  // LONG
                5 => 8,  // RATIONAL
                6 => 1,  // SBYTE
                7 => 1,  // UNDEFINED
                8 => 2,  // SSHORT
                9 => 4,  // SLONG
                10 => 8, // SRATIONAL
                11 => 4, // FLOAT
                12 => 8, // DOUBLE
                _ => {
                    debug!("Unknown format {} for tag {:#x}", format, tag_id);
                    return None;
                }
            };

            let data_size = (count as usize) * component_size;
            debug!("Tag {:#x} data size: {} bytes", tag_id, data_size);

            // If data fits in 4 bytes, it's stored directly in the value field
            if data_size <= 4 {
                let value_start = entry_offset + 8;
                let value_end = value_start + data_size;
                if value_end <= data.len() {
                    return Some(&data[value_start..value_end]);
                }
            } else {
                // Otherwise, the value field contains an offset
                let data_offset = match byte_order.read_u32(data, entry_offset + 8) {
                    Ok(offset) => offset as usize,
                    Err(e) => {
                        debug!("Failed to read data offset for tag {:#x}: {}", tag_id, e);
                        return None;
                    }
                };
                debug!("Tag {:#x} data at offset {:#x}", tag_id, data_offset);

                // The offset is relative to the start of the Canon maker note data
                let data_start = data_offset;
                let data_end = data_start + data_size;

                if data_end <= data.len() {
                    return Some(&data[data_start..data_end]);
                } else {
                    debug!("Tag {:#x} data extends beyond available data", tag_id);
                }
            }
        }
    }

    debug!("Canon tag {:#x} not found in IFD", tag_id);
    None
}

/// Process other Canon binary data tags using existing Canon processors
/// ExifTool: Canon.pm processes various Canon subdirectories
fn process_other_canon_binary_tags(_data: &[u8], _byte_order: ByteOrder) -> Result<()> {
    debug!("Processing other Canon binary data tags");

    // TODO: Process Canon ShotInfo, FocalLength, AFInfo, etc.
    // using existing Canon binary data processors and generated lookup tables

    Ok(())
}

/// Apply Canon CameraSettings PrintConv using generated lookup tables  
/// ExifTool: Canon.pm CameraSettings PrintConv with lookup tables
fn apply_camera_settings_print_conv(
    tag_name: &str,
    tag_value: &crate::types::TagValue,
) -> crate::types::TagValue {
    use crate::generated::Canon_pm::camerasettings_inline::*;

    debug!(
        "Applying Canon CameraSettings PrintConv for tag: {}",
        tag_name
    );

    // Apply generated lookup tables based on the tag name
    // ExifTool: Canon.pm CameraSettings table PrintConv entries
    match tag_name {
        "FlashMode" => {
            if let Some(value) = tag_value.as_u8() {
                if let Some(flash_mode) = lookup_camera_settings__flash_mode(value) {
                    return crate::types::TagValue::String(flash_mode.to_string());
                }
            }
        }
        "WhiteBalance2" => {
            if let Some(value) = tag_value.as_u16() {
                if let Some(white_balance) = lookup_camera_settings__white_balance2(value) {
                    return crate::types::TagValue::String(white_balance.to_string());
                }
            }
        }
        "ColorSpace" => {
            if let Some(value) = tag_value.as_u8() {
                if let Some(color_space) = lookup_camera_settings__color_space(value) {
                    return crate::types::TagValue::String(color_space.to_string());
                }
            }
        }
        "SceneMode" => {
            if let Some(value) = tag_value.as_u8() {
                if let Some(scene_mode) = lookup_camera_settings__scene_mode(value) {
                    return crate::types::TagValue::String(scene_mode.to_string());
                }
            }
        }
        "ExposureMode" => {
            if let Some(value) = tag_value.as_u8() {
                if let Some(exposure_mode) = lookup_camera_settings__exposure_mode(value) {
                    return crate::types::TagValue::String(exposure_mode.to_string());
                }
            }
        }
        "MeteringMode" => {
            if let Some(value) = tag_value.as_u16() {
                if let Some(metering_mode) = lookup_camera_settings__metering_mode(value) {
                    return crate::types::TagValue::String(metering_mode.to_string());
                }
            }
        }
        _ => {
            debug!(
                "No PrintConv available for Canon CameraSettings tag: {}",
                tag_name
            );
        }
    }

    // Return original value if no lookup table matches
    tag_value.clone()
}
