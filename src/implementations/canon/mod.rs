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
    extract_camera_settings, extract_focal_length, extract_my_colors, extract_panorama,
    extract_shot_info, find_canon_camera_settings_tag,
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

    // Apply Canon-specific PrintConv processing to main Canon table tags
    // ExifTool: Canon.pm Main table PrintConv entries need manual application
    apply_canon_main_table_print_conv(exif_reader)?;

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
                    // Note: tag_name already includes "MakerNotes:" prefix from extract_camera_settings()

                    debug!("Canon CameraSettings: {} = {:?}", tag_name, converted_value);

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
                    tags_to_store.push((synthetic_id, converted_value, tag_name));
                }
            }
            Err(e) => {
                debug!("Failed to extract Canon CameraSettings: {}", e);
            }
        }
    }

    // Get model for conditional processing
    let model = exif_reader
        .extracted_tags
        .get(&0x0110) // Model tag
        .and_then(|tag| tag.as_string())
        .unwrap_or("");

    // Process other Canon binary data tags using similar approach
    let mut other_tags = process_other_canon_binary_tags_with_reader(
        exif_reader,
        data,
        dir_start,
        byte_order,
        model,
    )?;
    tags_to_store.append(&mut other_tags);

    // Now store all collected tags (after all borrows are released)
    for (synthetic_id, converted_value, full_tag_name) in tags_to_store {
        exif_reader
            .extracted_tags
            .insert(synthetic_id, converted_value);
        // Store the mapping from synthetic ID to tag name
        exif_reader
            .synthetic_tag_names
            .insert(synthetic_id, full_tag_name.clone());
        
        // CRITICAL: Store TagSourceInfo for synthetic Canon tags so they get proper Group1 assignment
        // Without this, Canon MakerNote tags default to Group1="IFD0" instead of Group1="Canon"
        use crate::types::TagSourceInfo;
        let canon_source_info = TagSourceInfo::new(
            "Canon".to_string(),    // namespace: Canon for maker note tags
            "Canon".to_string(),    // ifd_name: Canon so get_group1() returns "Canon"
            "Canon::BinaryData".to_string(),  // processor: Canon binary data processor
        );
        exif_reader.tag_sources.insert(synthetic_id, canon_source_info);
        
        debug!(
            "Stored Canon tag {} with synthetic ID 0x{:04X} and TagSourceInfo with ifd_name='Canon'",
            full_tag_name, synthetic_id
        );
    }

    Ok(())
}

/// Find specific Canon tag data with proper offset handling
/// ExifTool: Canon.pm handles maker note offsets properly
fn find_canon_tag_data_with_full_access<'a>(
    full_data: &'a [u8],
    maker_note_data: &'a [u8],
    maker_note_offset: usize,
    tag_id: u16,
) -> Option<&'a [u8]> {
    use crate::tiff_types::ByteOrder;

    debug!(
        "Searching for Canon tag {:#x} with proper offset handling (maker note at {:#x})",
        tag_id, maker_note_offset
    );

    // Canon typically uses little-endian byte order
    let byte_order = ByteOrder::LittleEndian;

    // Parse Canon IFD structure in maker note data
    if maker_note_data.len() < 2 {
        debug!("Maker note data too short for IFD header");
        return None;
    }

    // Read number of directory entries
    let num_entries = match byte_order.read_u16(maker_note_data, 0) {
        Ok(count) => count,
        Err(e) => {
            debug!("Failed to read Canon IFD entry count: {}", e);
            return None;
        }
    };

    debug!("Canon IFD has {} entries", num_entries);

    // Search through IFD entries for the requested tag
    for i in 0..num_entries {
        let entry_offset = 2 + (i as usize * 12);

        if entry_offset + 12 > maker_note_data.len() {
            debug!("Entry {} beyond maker note data bounds", i);
            break;
        }

        let entry_tag = match byte_order.read_u16(maker_note_data, entry_offset) {
            Ok(tag) => tag,
            Err(e) => {
                debug!("Failed to read tag at entry {}: {}", i, e);
                continue;
            }
        };

        if entry_tag == tag_id {
            debug!("Found Canon tag {:#x} at entry {}", tag_id, i);

            let format = match byte_order.read_u16(maker_note_data, entry_offset + 2) {
                Ok(f) => f,
                Err(e) => {
                    debug!("Failed to read format for tag {:#x}: {}", tag_id, e);
                    return None;
                }
            };

            let count = match byte_order.read_u32(maker_note_data, entry_offset + 4) {
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

            let data_size = count as usize * component_size;

            // If data fits in 4 bytes, it's stored directly in the value field
            if data_size <= 4 {
                let value_start = entry_offset + 8;
                let value_end = value_start + data_size;
                if value_end <= maker_note_data.len() {
                    return Some(&maker_note_data[value_start..value_end]);
                }
            } else {
                // Otherwise, the value field contains an offset
                let data_offset = match byte_order.read_u32(maker_note_data, entry_offset + 8) {
                    Ok(offset) => offset as usize,
                    Err(e) => {
                        debug!("Failed to read data offset for tag {:#x}: {}", tag_id, e);
                        return None;
                    }
                };

                debug!("Tag {:#x} data at offset {:#x}", tag_id, data_offset);

                // CRITICAL FIX: Canon offsets are relative to TIFF header base
                // The offset is relative to the ExifReader's base, not the maker note
                let absolute_offset = data_offset;
                let data_end = absolute_offset + data_size;

                if data_end <= full_data.len() {
                    debug!(
                        "Reading Canon tag {:#x} data from absolute offset {:#x}",
                        tag_id, absolute_offset
                    );
                    return Some(&full_data[absolute_offset..data_end]);
                } else {
                    debug!("Tag {:#x} data extends beyond full file data (offset={:#x}, size={}, file_size={})", 
                           tag_id, absolute_offset, data_size, full_data.len());
                }
            }
        }
    }

    debug!("Canon tag {:#x} not found in IFD", tag_id);
    None
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
/// Returns Vec of (synthetic_id, tag_value, full_tag_name) tuples for storage
fn process_other_canon_binary_tags_with_reader(
    exif_reader: &crate::exif::ExifReader,
    maker_note_data: &[u8],
    maker_note_offset: usize,
    byte_order: ByteOrder,
    model: &str,
) -> Result<Vec<(u16, crate::TagValue, String)>> {
    debug!("Processing other Canon binary data tags with proper offset handling");

    let mut tags_to_store = Vec::new();
    let full_data = exif_reader.get_data();

    // Generate synthetic IDs using hash function
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Process Canon FocalLength (tag 0x0002) with proper offset handling
    // ExifTool: Canon.pm:2637 %Canon::FocalLength
    if let Some(focal_length_data) =
        find_canon_tag_data_with_full_access(full_data, maker_note_data, maker_note_offset, 0x0002)
    {
        debug!("Processing Canon FocalLength using existing Canon processor with proper offsets");

        match extract_focal_length(focal_length_data, 0, focal_length_data.len(), byte_order) {
            Ok(focal_info) => {
                debug!("Extracted {} Canon FocalLength tags", focal_info.len());
                for (tag_name, tag_value) in focal_info {
                    debug!("Canon FocalLength: {} = {:?}", tag_name, tag_value);

                    // Generate synthetic ID for this tag
                    let mut hasher = DefaultHasher::new();
                    tag_name.hash(&mut hasher);
                    let hash = hasher.finish();
                    let synthetic_id = 0xC000 + ((hash as u16) & 0x0FFF);

                    tags_to_store.push((synthetic_id, tag_value, tag_name));
                }
            }
            Err(e) => {
                debug!("Failed to extract Canon FocalLength: {}", e);
            }
        }
    }

    // Process Canon ShotInfo (tag 0x0004) with proper offset handling
    // ExifTool: Canon.pm:2715 %Canon::ShotInfo
    if let Some(shot_info_data) =
        find_canon_tag_data_with_full_access(full_data, maker_note_data, maker_note_offset, 0x0004)
    {
        debug!("Processing Canon ShotInfo using existing Canon processor with proper offsets");

        match extract_shot_info(shot_info_data, 0, shot_info_data.len(), byte_order) {
            Ok(shot_info) => {
                debug!("Extracted {} Canon ShotInfo tags", shot_info.len());
                for (tag_name, tag_value) in shot_info {
                    debug!("Canon ShotInfo: {} = {:?}", tag_name, tag_value);

                    // Generate synthetic ID for this tag
                    let mut hasher = DefaultHasher::new();
                    tag_name.hash(&mut hasher);
                    let hash = hasher.finish();
                    let synthetic_id = 0xC000 + ((hash as u16) & 0x0FFF);

                    tags_to_store.push((synthetic_id, tag_value, tag_name));
                }
            }
            Err(e) => {
                debug!("Failed to extract Canon ShotInfo: {}", e);
            }
        }
    }

    // Process Canon Panorama (tag 0x0005) with proper offset handling
    // ExifTool: Canon.pm:2999 %Canon::Panorama with ProcessBinaryData
    if let Some(panorama_data) =
        find_canon_tag_data_with_full_access(full_data, maker_note_data, maker_note_offset, 0x0005)
    {
        debug!("Processing Canon Panorama using existing Canon processor with proper offsets");

        match extract_panorama(panorama_data, 0, panorama_data.len(), byte_order) {
            Ok(panorama_info) => {
                debug!("Extracted {} Canon Panorama tags", panorama_info.len());
                for (tag_name, tag_value) in panorama_info {
                    debug!("Canon Panorama: {} = {:?}", tag_name, tag_value);

                    // Generate synthetic ID for this tag
                    let mut hasher = DefaultHasher::new();
                    tag_name.hash(&mut hasher);
                    let hash = hasher.finish();
                    let synthetic_id = 0xC000 + ((hash as u16) & 0x0FFF);

                    tags_to_store.push((synthetic_id, tag_value, tag_name));
                }
            }
            Err(e) => {
                debug!("Failed to extract Canon Panorama: {}", e);
            }
        }
    }

    // Process Canon AFInfo2 (tag 0x0026) with proper offset handling
    // ExifTool: Canon.pm:4477 %Canon::AFInfo2
    if let Some(af_info2_data) =
        find_canon_tag_data_with_full_access(full_data, maker_note_data, maker_note_offset, 0x0026)
    {
        debug!("Processing Canon AFInfo2 using serial data processor with proper offsets");

        match af_info::process_serial_data(
            af_info2_data,
            0,
            af_info2_data.len(),
            byte_order,
            &af_info::create_af_info2_table(),
            model,
        ) {
            Ok(af_info2) => {
                debug!("Extracted {} Canon AFInfo2 tags", af_info2.len());
                for (tag_name, tag_value) in af_info2 {
                    debug!("Canon AFInfo2: {} = {:?}", tag_name, tag_value);

                    // Generate synthetic ID for this tag
                    let mut hasher = DefaultHasher::new();
                    tag_name.hash(&mut hasher);
                    let hash = hasher.finish();
                    let synthetic_id = 0xC000 + ((hash as u16) & 0x0FFF);

                    tags_to_store.push((synthetic_id, tag_value, tag_name));
                }
            }
            Err(e) => {
                debug!("Failed to extract Canon AFInfo2: {}", e);
            }
        }
    }

    Ok(tags_to_store)
}

/// Legacy function for backward compatibility
/// Process other Canon binary data tags using existing Canon processors
/// ExifTool: Canon.pm processes various Canon subdirectories
/// Returns Vec of (synthetic_id, tag_value, full_tag_name) tuples for storage
#[allow(dead_code)]
fn process_other_canon_binary_tags(
    data: &[u8],
    byte_order: ByteOrder,
    model: &str,
) -> Result<Vec<(u16, crate::TagValue, String)>> {
    debug!("Processing other Canon binary data tags");

    let mut tags_to_store = Vec::new();

    // Generate synthetic IDs using hash function
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Process Canon FocalLength (tag 0x0002)
    // ExifTool: Canon.pm:2637 %Canon::FocalLength
    if let Some(focal_length_data) = find_canon_tag_data(data, 0x0002) {
        debug!("Processing Canon FocalLength using existing Canon processor");

        match extract_focal_length(focal_length_data, 0, focal_length_data.len(), byte_order) {
            Ok(focal_info) => {
                debug!("Extracted {} Canon FocalLength tags", focal_info.len());
                for (tag_name, tag_value) in focal_info {
                    debug!("Canon FocalLength: {} = {:?}", tag_name, tag_value);

                    // Generate synthetic ID for this tag
                    let mut hasher = DefaultHasher::new();
                    tag_name.hash(&mut hasher);
                    let hash = hasher.finish();
                    let synthetic_id = 0xC000 + ((hash as u16) & 0x0FFF);

                    tags_to_store.push((synthetic_id, tag_value, tag_name));
                }
            }
            Err(e) => {
                debug!("Failed to extract Canon FocalLength: {}", e);
            }
        }
    }

    // Process Canon ShotInfo (tag 0x0004)
    // ExifTool: Canon.pm:2715 %Canon::ShotInfo
    if let Some(shot_info_data) = find_canon_tag_data(data, 0x0004) {
        debug!("Processing Canon ShotInfo using existing Canon processor");

        match extract_shot_info(shot_info_data, 0, shot_info_data.len(), byte_order) {
            Ok(shot_info) => {
                debug!("Extracted {} Canon ShotInfo tags", shot_info.len());
                for (tag_name, tag_value) in shot_info {
                    debug!("Canon ShotInfo: {} = {:?}", tag_name, tag_value);

                    // Generate synthetic ID for this tag
                    let mut hasher = DefaultHasher::new();
                    tag_name.hash(&mut hasher);
                    let hash = hasher.finish();
                    let synthetic_id = 0xC000 + ((hash as u16) & 0x0FFF);

                    tags_to_store.push((synthetic_id, tag_value, tag_name));
                }
            }
            Err(e) => {
                debug!("Failed to extract Canon ShotInfo: {}", e);
            }
        }
    }

    // Process Canon AFInfo (tag 0x0012)
    // ExifTool: Canon.pm:4440 %Canon::AFInfo
    if let Some(af_info_data) = find_canon_tag_data(data, 0x0012) {
        debug!("Processing Canon AFInfo using serial data processor");

        // Use model passed from parent function

        match af_info::process_serial_data(
            af_info_data,
            0,
            af_info_data.len(),
            byte_order,
            &af_info::create_af_info_table(),
            model,
        ) {
            Ok(af_info) => {
                debug!("Extracted {} Canon AFInfo tags", af_info.len());
                for (tag_name, tag_value) in af_info {
                    debug!("Canon AFInfo: {} = {:?}", tag_name, tag_value);

                    // Generate synthetic ID for this tag
                    let mut hasher = DefaultHasher::new();
                    tag_name.hash(&mut hasher);
                    let hash = hasher.finish();
                    let synthetic_id = 0xC000 + ((hash as u16) & 0x0FFF);

                    tags_to_store.push((synthetic_id, tag_value, tag_name));
                }
            }
            Err(e) => {
                debug!("Failed to extract Canon AFInfo: {}", e);
            }
        }
    }

    // Process Canon AFInfo2 (tag 0x0026)
    // ExifTool: Canon.pm:4477 %Canon::AFInfo2
    if let Some(af_info2_data) = find_canon_tag_data(data, 0x0026) {
        debug!("Processing Canon AFInfo2 using serial data processor");

        // CRITICAL FIX: Test byte order to determine correct reading
        // AFInfo2 data might use different byte order than maker note IFD
        let afinfo2_byte_order = if af_info2_data.len() >= 2 {
            // Test both byte orders to see which gives reasonable values
            let le_val = ByteOrder::LittleEndian
                .read_u16(af_info2_data, 0)
                .unwrap_or(0);
            let be_val = ByteOrder::BigEndian.read_u16(af_info2_data, 0).unwrap_or(0);

            debug!("AFInfo2 first value: LE={}, BE={}", le_val, be_val);

            // AFInfoSize should be around 96 (0x60), not 31482 (0x7AFA)
            // Use big-endian if it gives a more reasonable value
            if be_val < 200 && le_val > 30000 {
                debug!("Using big-endian byte order for AFInfo2");
                ByteOrder::BigEndian
            } else {
                debug!("Using little-endian byte order for AFInfo2");
                byte_order
            }
        } else {
            byte_order
        };

        match af_info::process_serial_data(
            af_info2_data,
            0,
            af_info2_data.len(),
            afinfo2_byte_order,
            &af_info::create_af_info2_table(),
            model,
        ) {
            Ok(af_info2) => {
                debug!("Extracted {} Canon AFInfo2 tags", af_info2.len());
                for (tag_name, tag_value) in af_info2 {
                    debug!("Canon AFInfo2: {} = {:?}", tag_name, tag_value);

                    // Generate synthetic ID for this tag
                    let mut hasher = DefaultHasher::new();
                    tag_name.hash(&mut hasher);
                    let hash = hasher.finish();
                    let synthetic_id = 0xC000 + ((hash as u16) & 0x0FFF);

                    tags_to_store.push((synthetic_id, tag_value, tag_name));
                }
            }
            Err(e) => {
                debug!("Failed to extract Canon AFInfo2: {}", e);
            }
        }
    }

    // Process Canon Panorama (tag 0x0005)
    // ExifTool: Canon.pm:2999 %Canon::Panorama with ProcessBinaryData
    if let Some(panorama_data) = find_canon_tag_data(data, 0x0005) {
        debug!("Processing Canon Panorama using existing Canon processor");

        match extract_panorama(panorama_data, 0, panorama_data.len(), byte_order) {
            Ok(panorama_info) => {
                debug!("Extracted {} Canon Panorama tags", panorama_info.len());
                for (tag_name, tag_value) in panorama_info {
                    debug!("Canon Panorama: {} = {:?}", tag_name, tag_value);

                    // Generate synthetic ID for this tag
                    let mut hasher = DefaultHasher::new();
                    tag_name.hash(&mut hasher);
                    let hash = hasher.finish();
                    let synthetic_id = 0xC000 + ((hash as u16) & 0x0FFF);

                    tags_to_store.push((synthetic_id, tag_value, tag_name));
                }
            }
            Err(e) => {
                debug!("Failed to extract Canon Panorama: {}", e);
            }
        }
    }

    // Process Canon MyColors (tag 0x001d)
    // ExifTool: Canon.pm:3131 %Canon::MyColors with ProcessBinaryData and validation
    if let Some(my_colors_data) = find_canon_tag_data(data, 0x001d) {
        debug!("Processing Canon MyColors using existing Canon processor");

        match extract_my_colors(my_colors_data, 0, my_colors_data.len(), byte_order) {
            Ok(my_colors_info) => {
                debug!("Extracted {} Canon MyColors tags", my_colors_info.len());
                for (tag_name, tag_value) in my_colors_info {
                    debug!("Canon MyColors: {} = {:?}", tag_name, tag_value);

                    // Generate synthetic ID for this tag
                    let mut hasher = DefaultHasher::new();
                    tag_name.hash(&mut hasher);
                    let hash = hasher.finish();
                    let synthetic_id = 0xC000 + ((hash as u16) & 0x0FFF);

                    tags_to_store.push((synthetic_id, tag_value, tag_name));
                }
            }
            Err(e) => {
                debug!("Failed to extract Canon MyColors: {}", e);
            }
        }
    }

    Ok(tags_to_store)
}

/// Apply Canon-specific PrintConv processing to main Canon table tags
/// ExifTool: Canon.pm Main table PrintConv entries for human-readable output
fn apply_canon_main_table_print_conv(exif_reader: &mut crate::exif::ExifReader) -> Result<()> {
    use crate::generated::Canon_pm::canonmodelid::lookup_canon_model_id;
    use crate::types::TagValue;

    debug!("Applying Canon main table PrintConv processing");

    // Create a list of tag IDs that need Canon-specific PrintConv processing
    let mut tags_to_update = Vec::new();

    // Find Canon tags that need PrintConv processing
    for (&tag_id, tag_value) in &exif_reader.extracted_tags {
        // Only process tags that have Canon source info (MakerNotes namespace)
        if let Some(source_info) = exif_reader.tag_sources.get(&tag_id) {
            if source_info.namespace == "MakerNotes" && source_info.ifd_name.starts_with("Canon") {
                match tag_id {
                    0x10 => {
                        // CanonModelID - apply generated lookup table
                        // ExifTool: Canon.pm CanonModelID PrintConv
                        if let TagValue::U32(model_id) = tag_value {
                            if let Some(model_name) = lookup_canon_model_id(*model_id) {
                                debug!("Canon CanonModelID: {} -> {}", model_id, model_name);
                                tags_to_update
                                    .push((tag_id, TagValue::String(model_name.to_string())));
                            } else {
                                debug!("Canon CanonModelID: {} (unknown model)", model_id);
                                tags_to_update.push((
                                    tag_id,
                                    TagValue::String(format!("Unknown model ({model_id})")),
                                ));
                            }
                        }
                    }
                    _ => {
                        // Other Canon tags can be added here as needed
                    }
                }
            }
        }
    }

    // Apply the PrintConv updates
    for (tag_id, new_value) in tags_to_update {
        exif_reader.extracted_tags.insert(tag_id, new_value);
    }

    debug!("Canon main table PrintConv processing completed");
    Ok(())
}

/// Apply Canon CameraSettings PrintConv using generated lookup tables  
/// ExifTool: Canon.pm CameraSettings PrintConv with lookup tables
fn apply_camera_settings_print_conv(
    tag_name: &str,
    tag_value: &crate::types::TagValue,
) -> crate::types::TagValue {
    use crate::generated::Canon_pm::camerasettings_inline::*;
    use crate::generated::Canon_pm::canonquality::lookup_canon_quality;

    debug!(
        "Applying Canon CameraSettings PrintConv for tag: {}",
        tag_name
    );

    // Strip MakerNotes: prefix if present for matching
    let clean_tag_name = tag_name.strip_prefix("MakerNotes:").unwrap_or(tag_name);

    // Apply generated lookup tables based on the tag name
    // ExifTool: Canon.pm CameraSettings table PrintConv entries
    match clean_tag_name {
        "MacroMode" => {
            if let Some(value) = tag_value.as_u8() {
                if let Some(macro_mode) = lookup_camera_settings__macro_mode(value) {
                    return crate::types::TagValue::String(macro_mode.to_string());
                }
            }
        }
        "Quality" => {
            if let Some(value) = tag_value.as_u16() {
                // Convert u16 to i16 for lookup function
                let signed_value = value as i16;
                if let Some(quality) = lookup_canon_quality(signed_value) {
                    return crate::types::TagValue::String(quality.to_string());
                }
            }
        }
        "FocusMode" => {
            // TODO: Add FocusMode lookup when Canon-specific lookup is generated
            // For now, return the raw value as string
            return crate::types::TagValue::String(format!("FocusMode {tag_value}"));
        }
        "CanonFlashMode" => {
            // CanonFlashMode might use the same lookup as FlashMode, or need a different one
            if let Some(value) = tag_value.as_u8() {
                if let Some(flash_mode) = lookup_camera_settings__flash_mode(value) {
                    return crate::types::TagValue::String(flash_mode.to_string());
                }
            }
        }
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
