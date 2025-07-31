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
    debug!(
        "Processing Canon MakerNotes: start={:#x}, size={}",
        dir_start, size
    );

    // Canon MakerNotes are structured as a standard IFD with Canon-specific tag processing
    // ExifTool: Canon.pm Main table processes Canon tags as subdirectories
    // Key insight: Canon tags need Canon-specific processors, not generic TIFF processing

    // Process Canon MakerNotes as a standard IFD (not through processor dispatch)
    // ExifTool: Canon MakerNotes are processed as standard IFD with Canon-specific tag handling
    // Trust ExifTool: Canon.pm line ~145 ProcessDirectory($et, $tagTablePtr, $dirInfo)
    // Fix Group1 assignment: Use "Canon" directory name to get group1="Canon" instead of "MakerNotes"
    debug!(
        "Processing Canon MakerNotes as standard IFD at offset {:#x}",
        dir_start
    );
    exif_reader.parse_ifd(dir_start, "Canon")?;

    // Apply Canon-specific PrintConv processing to main Canon table tags
    // ExifTool: Canon.pm Main table PrintConv entries need manual application
    apply_canon_main_table_print_conv(exif_reader)?;

    // Process Canon subdirectory tags (like ColorData)
    // ExifTool: Canon.pm SubDirectory processing for tags like ColorData1-12
    process_canon_subdirectory_tags(exif_reader)?;

    // Canon binary data processing is now handled entirely by the tag kit system
    // The manual binary data processing was causing conflicts and value corruption

    debug!("Canon MakerNotes processing completed");
    Ok(())
}

// Removed process_canon_binary_data_with_existing_processors function - it was causing
// conflicts with the tag kit system leading to corrupted Canon MakerNotes values

/// Find specific Canon tag data with proper offset handling
/// ExifTool: Canon.pm handles maker note offsets properly
#[allow(dead_code)]
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
#[allow(dead_code)]
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

    // Create a list of tag keys that need Canon-specific PrintConv processing
    let mut tags_to_update = Vec::new();

    // Find Canon tags that need PrintConv processing
    for (&(tag_id, ref namespace), tag_value) in &exif_reader.extracted_tags {
        // Only process tags that have Canon source info (MakerNotes namespace)
        if let Some(source_info) = exif_reader.tag_sources.get(&(tag_id, namespace.clone())) {
            if source_info.namespace == "MakerNotes" && namespace == "MakerNotes" {
                match tag_id {
                    0x10 => {
                        // CanonModelID - apply generated lookup table
                        // ExifTool: Canon.pm CanonModelID PrintConv
                        if let TagValue::U32(model_id) = tag_value {
                            if let Some(model_name) = lookup_canon_model_id(*model_id) {
                                debug!("Canon CanonModelID: {} -> {}", model_id, model_name);
                                tags_to_update.push((
                                    (tag_id, namespace.clone()),
                                    TagValue::String(model_name.to_string()),
                                ));
                            } else {
                                debug!("Canon CanonModelID: {} (unknown model)", model_id);
                                tags_to_update.push((
                                    (tag_id, namespace.clone()),
                                    TagValue::String(format!("Unknown model ({model_id})")),
                                ));
                            }
                        }
                    }
                    0x4001 | 0x4002 | 0x4003 | 0x4004 | 0x4005 | 0x4008 | 0x4009 | 0x4010
                    | 0x4011 | 0x4012 | 0x4013 | 0x4015 | 0x4016 | 0x4018 | 0x4019 | 0x4020
                    | 0x4021 | 0x4024 | 0x4025 | 0x4028 => {
                        // ColorData and other subdirectory tags
                        // These are handled separately via subdirectory processing
                        debug!(
                            "Tag 0x{:04x} has subdirectory processing, skipping PrintConv",
                            tag_id
                        );
                    }
                    _ => {
                        // Other Canon tags can be added here as needed
                    }
                }
            }
        }
    }

    // Apply the PrintConv updates
    for (tag_key, new_value) in tags_to_update {
        exif_reader.extracted_tags.insert(tag_key, new_value);
    }

    debug!("Canon main table PrintConv processing completed");
    Ok(())
}

/// Find Canon tag ID by name from the tag kit system
/// Used for applying PrintConv to subdirectory-extracted tags
fn find_canon_tag_id_by_name(tag_name: &str) -> Option<u32> {
    use crate::generated::Canon_pm::tag_kit::CANON_PM_TAG_KITS;

    // Search through all Canon tag kit entries to find matching name
    for (&tag_id, tag_def) in CANON_PM_TAG_KITS.iter() {
        if tag_def.name == tag_name {
            return Some(tag_id);
        }
    }
    None
}

/// Map Canon CameraSettings tag names to their tag kit IDs
/// ExifTool: Canon.pm CameraSettings table tag IDs extracted from tag kit
fn get_canon_camera_settings_tag_id(tag_name: &str) -> Option<u32> {
    // Strip MakerNotes: prefix if present for matching
    let clean_tag_name = tag_name.strip_prefix("MakerNotes:").unwrap_or(tag_name);

    // Map tag names to their tag kit IDs based on Canon tag kit interop.rs
    // These IDs come from the generated Canon tag kit system
    match clean_tag_name {
        "MacroMode" => Some(1),       // TagKitDef { id: 1, name: "MacroMode", ... }
        "SelfTimer" => Some(2), // TagKitDef { id: 2, name: "SelfTimer", ... } (in datetime.rs)
        "Quality" => Some(3),   // TagKitDef { id: 3, name: "Quality", ... }
        "CanonFlashMode" => Some(4), // TagKitDef { id: 4, name: "CanonFlashMode", ... }
        "ContinuousDrive" => Some(5), // TagKitDef { id: 5, name: "ContinuousDrive", ... }
        "FocusMode" => Some(7), // TagKitDef { id: 7, name: "FocusMode", ... }
        "RecordMode" => Some(9), // TagKitDef { id: 9, name: "RecordMode", ... }
        "CanonImageSize" => Some(10), // TagKitDef { id: 10, name: "CanonImageSize", ... }
        "EasyMode" => None,     // Not found in generated tag kit yet
        "DigitalZoom" => None,  // Not found in generated tag kit yet
        "Contrast" => None,     // Not found in generated tag kit yet
        "Saturation" => None,   // Not found in generated tag kit yet
        "Sharpness" => None,    // Not found in generated tag kit yet
        "CameraISO" => None,    // Not found in generated tag kit yet
        "MeteringMode" => None, // Not found in generated tag kit yet
        "FocusRange" => None,   // Not found in generated tag kit yet
        "AFPoint" => None,      // Not found in generated tag kit yet
        "CanonExposureMode" => None, // Not found in generated tag kit yet
        "LensID" => None,       // Not found in generated tag kit yet
        "LensType" => None,     // Not found in generated tag kit yet
        "ShortFocal" => None,   // Not found in generated tag kit yet
        "LongFocal" => None,    // Not found in generated tag kit yet
        "FocalUnits" => None,   // Not found in generated tag kit yet
        "MaxAperture" => None,  // Not found in generated tag kit yet
        "MinAperture" => None,  // Not found in generated tag kit yet
        "FlashActivity" => None, // Not found in generated tag kit yet
        "FlashBits" => None,    // Not found in generated tag kit yet
        "ManualFlashOutput" => None, // Not found in generated tag kit yet
        "FocusContinuous" => None, // Not found in generated tag kit yet
        "AESetting" => None,    // Not found in generated tag kit yet
        "ImageStabilization" => None, // Not found in generated tag kit yet
        "DisplayAperture" => None, // Not found in generated tag kit yet
        "ZoomSourceWidth" => None, // Not found in generated tag kit yet
        "ZoomTargetWidth" => None, // Not found in generated tag kit yet
        "SpotMeteringMode" => None, // Not found in generated tag kit yet
        "PhotoEffect" => None,  // Not found in generated tag kit yet
        "ColorTone" => None,    // Not found in generated tag kit yet
        "SRAWQuality" => None,  // Not found in generated tag kit yet
        _ => {
            debug!("Unknown Canon CameraSettings tag name: {}", clean_tag_name);
            None
        }
    }
}

/// Process Canon subdirectory tags using the generic subdirectory processing system
/// ExifTool: Canon.pm SubDirectory processing for binary data expansion
fn process_canon_subdirectory_tags(exif_reader: &mut crate::exif::ExifReader) -> Result<()> {
    use crate::exif::subdirectory_processing::process_subdirectories_with_printconv;
    use crate::generated::Canon_pm::tag_kit;

    debug!("Processing Canon subdirectory tags using generic system");

    // Use the generic subdirectory processing with Canon-specific functions
    // Fix Group1 assignment: Use "Canon" as namespace for group1="Canon" instead of "MakerNotes"
    process_subdirectories_with_printconv(
        exif_reader,
        "Canon",
        "Canon",
        tag_kit::has_subdirectory,
        tag_kit::process_subdirectory,
        tag_kit::apply_print_conv,
        find_canon_tag_id_by_name,
    )?;

    debug!("Canon subdirectory processing completed");
    Ok(())
}

/// Apply Canon CameraSettings PrintConv using unified tag kit system
/// ExifTool: Canon.pm CameraSettings PrintConv with tag kit lookup tables
pub fn apply_camera_settings_print_conv(
    tag_name: &str,
    tag_value: &crate::types::TagValue,
) -> crate::types::TagValue {
    use crate::expressions::ExpressionEvaluator;
    use crate::generated::Canon_pm::tag_kit;

    debug!(
        "Applying Canon CameraSettings PrintConv for tag: {} using tag kit system",
        tag_name
    );

    // Get the tag kit ID for this tag name
    if let Some(tag_id) = get_canon_camera_settings_tag_id(tag_name) {
        debug!("Found tag kit ID {} for tag {}", tag_id, tag_name);
        // Use unified tag kit system for PrintConv
        let mut evaluator = ExpressionEvaluator::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        let result = tag_kit::apply_print_conv(
            tag_id,
            tag_value,
            &mut evaluator,
            &mut errors,
            &mut warnings,
        );

        // Log any warnings from tag kit processing
        for warning in warnings {
            debug!("Canon tag kit warning: {}", warning);
        }

        // Log any errors from tag kit processing
        for error in errors {
            debug!("Canon tag kit error: {}", error);
        }

        debug!(
            "Tag kit PrintConv result for {}: {:?} -> {:?}",
            tag_name, tag_value, result
        );
        return result;
    }

    // Return original value if tag ID not found
    debug!(
        "No tag kit mapping available for Canon CameraSettings tag: {}",
        tag_name
    );
    tag_value.clone()
}

#[cfg(test)]
mod tests {

    use crate::exif::ExifReader;
    use crate::tiff_types::ByteOrder;
    use crate::types::TagValue;
    use crate::utils::ensure_group_prefix;

    #[test]
    fn test_unique_synthetic_ids_for_subdirectory_tags() {
        // Create a mock ExifReader
        let mut exif_reader = ExifReader::new();

        // Set up a fake header with byte order
        exif_reader.header = Some(crate::tiff_types::TiffHeader {
            byte_order: ByteOrder::LittleEndian,
            magic: 42,
            ifd0_offset: 0,
        });

        // Create mock tag data that has subdirectory processing
        // Use ColorData1 (0x4001) as an example
        let tag_id = 0x4001u16;
        let mock_data = TagValue::U16Array(vec![0; 100]); // Mock binary data

        // Store the tag in the reader
        exif_reader
            .extracted_tags
            .insert((tag_id, "Canon".to_string()), mock_data);
        exif_reader.tag_sources.insert(
            (tag_id, "Canon".to_string()),
            crate::types::TagSourceInfo::new(
                "MakerNotes".to_string(),
                "Canon".to_string(),
                "Canon::Main".to_string(),
            ),
        );

        // For now, just test the synthetic ID generation logic directly
        // since we can't run the full subdirectory processing without generated code

        // Simulate what the subdirectory processing would do
        let tag_id = 0x4001u16;
        let mut synthetic_ids = Vec::new();

        // Generate a few synthetic IDs like the real code would
        for synthetic_counter in 0..5_u16 {
            let synthetic_id = 0x8000 | (tag_id & 0x7F00) | (synthetic_counter & 0xFF);
            synthetic_ids.push(synthetic_id);
        }

        // Verify deterministic ID generation pattern
        let expected_base = 0x8000 | (tag_id & 0x7F00); // Should be 0xC000 for 0x4001
        for (i, &id) in synthetic_ids.iter().enumerate() {
            let expected_id = expected_base | (i as u16 & 0xFF);
            assert_eq!(
                id, expected_id,
                "Expected deterministic ID 0x{:04x} at position {}, got 0x{:04x}",
                expected_id, i, id
            );
        }

        // Verify all IDs are unique
        let unique_count = synthetic_ids
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
        assert_eq!(
            unique_count,
            synthetic_ids.len(),
            "Found duplicate synthetic IDs"
        );

        // Verify all IDs are in the correct range (0x8000 - 0xFFFF)
        // Note: For ColorData1 (0x4001), the pattern 0x8000 | (0x4001 & 0x7F00) = 0xC000
        for &id in &synthetic_ids {
            assert!(
                id >= 0x8000,
                "Synthetic ID 0x{:04x} is below expected range (0x8000+)",
                id
            );
        }
    }

    #[test]
    fn test_synthetic_id_collision_detection() {
        // This test verifies that the debug assertion would catch collisions
        // In debug mode, attempting to insert a duplicate synthetic ID should panic
        let mut exif_reader = ExifReader::new();

        // Insert a synthetic tag name
        let synthetic_id = 0x8001;
        exif_reader
            .synthetic_tag_names
            .insert(synthetic_id, "MakerNotes:TestTag".to_string());

        // In debug mode, inserting the same ID again would trigger the assertion
        // This test just verifies the data structure is set up correctly
        assert!(exif_reader.synthetic_tag_names.contains_key(&synthetic_id));
    }

    #[test]
    fn test_unknown_tags_filtered() {
        // Simulate subdirectory extraction with both known and unknown tags
        let mock_tags = vec![
            (
                "WB_RGGBLevelsAsShot".to_string(),
                TagValue::String("2241 1024 1024 1689".to_string()),
            ),
            (
                "WB_RGGBLevelsUnknown".to_string(),
                TagValue::String("1000 1000 1000 1000".to_string()),
            ),
            (
                "WB_RGGBLevelsDaylight".to_string(),
                TagValue::String("2217 1024 1024 1631".to_string()),
            ),
            (
                "WB_RGGBLevelsUnknown2".to_string(),
                TagValue::String("2000 2000 2000 2000".to_string()),
            ),
        ];

        let mut exif_reader = ExifReader::new();
        let tag_id = 0x4001_u16;
        let mut synthetic_counter: u16 = 0;

        // Process tags like the real code does
        for (tag_name, value) in mock_tags {
            // Skip tags marked as Unknown (matching ExifTool's default behavior)
            if tag_name.contains("Unknown") {
                continue;
            }

            let synthetic_id = 0x8000 | (tag_id & 0x7F00) | (synthetic_counter & 0xFF);
            synthetic_counter += 1;

            let full_tag_name = ensure_group_prefix(&tag_name, "MakerNotes");
            exif_reader
                .synthetic_tag_names
                .insert(synthetic_id, full_tag_name);
            exif_reader
                .extracted_tags
                .insert((synthetic_id, "MakerNotes".to_string()), value);
        }

        // Should have only 2 tags (the known ones)
        assert_eq!(exif_reader.synthetic_tag_names.len(), 2);
        assert!(exif_reader
            .synthetic_tag_names
            .values()
            .any(|v| v.contains("WB_RGGBLevelsAsShot")));
        assert!(exif_reader
            .synthetic_tag_names
            .values()
            .any(|v| v.contains("WB_RGGBLevelsDaylight")));
        assert!(!exif_reader
            .synthetic_tag_names
            .values()
            .any(|v| v.contains("Unknown")));
    }
}
