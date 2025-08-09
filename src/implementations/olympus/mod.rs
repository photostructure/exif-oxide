//! Olympus-specific MakerNote processing
//!
//! This module implements Olympus MakerNote detection following ExifTool's Olympus processing
//! from MakerNotes.pm, focusing on proper namespace handling and binary data processing.
//!
//! **Trust ExifTool**: This code translates ExifTool's Olympus detection patterns verbatim
//! without any improvements or simplifications. Every detection pattern and signature
//! is copied exactly as documented in the ExifTool source.
//!
//! Primary ExifTool References:
//! - lib/Image/ExifTool/MakerNotes.pm:515-533 - Olympus MakerNote detection patterns
//! - lib/Image/ExifTool/Olympus.pm - Olympus tag tables and processing

// Equipment tag lookup now handled by generated code

use tracing::trace;

/// Olympus MakerNote signature patterns from ExifTool
/// ExifTool: lib/Image/ExifTool/MakerNotes.pm:515-533 Olympus detection conditions
#[derive(Debug, Clone, PartialEq)]
pub enum OlympusSignature {
    /// Older Olympus/Epson format starting with "OLYMP\0" or "EPSON\0"
    /// ExifTool: MakerNoteOlympus Condition '$$valPt =~ /^(OLYMP|EPSON)\0/'
    OlympusOld,
    /// Newer Olympus format starting with "OLYMPUS\0"
    /// ExifTool: MakerNoteOlympus2 Condition '$$valPt =~ /^OLYMPUS\0/'
    OlympusNew,
    /// Newest OM System format starting with "OM SYSTEM\0"
    /// ExifTool: MakerNoteOlympus3 Condition '$$valPt =~ /^OM SYSTEM\0/'
    OmSystem,
}

impl OlympusSignature {
    /// Get the byte offset to the actual maker note data
    /// ExifTool: Start parameter in SubDirectory definitions
    pub fn data_offset(&self) -> usize {
        match self {
            OlympusSignature::OlympusOld => 8,  // Start => '$valuePtr + 8'
            OlympusSignature::OlympusNew => 12, // Start => '$valuePtr + 12'
            OlympusSignature::OmSystem => 16,   // Start => '$valuePtr + 16'
        }
    }

    /// Get the base offset adjustment
    /// ExifTool: Base parameter in SubDirectory definitions  
    pub fn base_offset(&self) -> i32 {
        match self {
            OlympusSignature::OlympusOld => 0,   // No Base adjustment
            OlympusSignature::OlympusNew => -12, // Base => '$start - 12'
            OlympusSignature::OmSystem => -16,   // Base => '$start - 16'
        }
    }
}

/// Detect Olympus MakerNote signature from binary data and Make field
/// ExifTool: lib/Image/ExifTool/MakerNotes.pm:515-533 Olympus detection logic
pub fn detect_olympus_signature(_make: &str, maker_note_data: &[u8]) -> Option<OlympusSignature> {
    if maker_note_data.is_empty() {
        return None;
    }

    // Priority order matches ExifTool's table order in MakerNotes.pm

    // 1. MakerNoteOlympus3: OM SYSTEM (newest format)
    // ExifTool: MakerNotes.pm:530 '$$valPt =~ /^OM SYSTEM\0/'
    if maker_note_data.starts_with(b"OM SYSTEM\0") {
        trace!("Detected OM System signature");
        return Some(OlympusSignature::OmSystem);
    }

    // 2. MakerNoteOlympus2: OLYMPUS\0 (newer format)
    // ExifTool: MakerNotes.pm:523 '$$valPt =~ /^OLYMPUS\0/'
    if maker_note_data.starts_with(b"OLYMPUS\0") {
        trace!("Detected Olympus new signature");
        return Some(OlympusSignature::OlympusNew);
    }

    // 3. MakerNoteOlympus: OLYMP\0 or EPSON\0 (older format)
    // ExifTool: MakerNotes.pm:516 '$$valPt =~ /^(OLYMP|EPSON)\0/'
    if maker_note_data.starts_with(b"OLYMP\0") || maker_note_data.starts_with(b"EPSON\0") {
        trace!("Detected Olympus old signature (OLYMP/EPSON)");
        return Some(OlympusSignature::OlympusOld);
    }

    // No Olympus signature detected
    None
}

/// Detect if this is an Olympus MakerNote based on Make field
/// This is used as a fallback when signature detection fails
pub fn is_olympus_makernote(make: &str) -> bool {
    // ExifTool: Check if Make field indicates Olympus
    make.starts_with("OLYMPUS") || make == "OM Digital Solutions"
}

/// Find Olympus tag ID by name from the tag kit system
/// Used for applying PrintConv to subdirectory-extracted tags
fn find_olympus_tag_id_by_name(tag_name: &str) -> Option<u32> {
    use crate::generated::olympus::OLYMPUS_MAIN_TAGS;

    // Search through all Olympus main tags entries to find matching name
    for (&tag_id, tag_def) in OLYMPUS_MAIN_TAGS.iter() {
        if tag_def.name == tag_name {
            return Some(tag_id.into());
        }
    }
    None
}

/// Process Equipment subdirectory data using ExifTool's dual format logic
/// ExifTool: Olympus.pm lines 1170-1190 - Equipment format condition processing
///
/// Equipment data can be in two formats:
/// - Binary format (old cameras like E-1, E-300): malformed undef/string data
/// - IFD format (newer cameras): standard TIFF IFD structure
pub fn process_equipment_subdirectory(
    data: &[u8],
    byte_order: crate::tiff_types::ByteOrder,
) -> crate::types::Result<Vec<(String, crate::types::TagValue)>> {
    use crate::tiff_types::{IfdEntry, TiffFormat};
    use crate::types::TagValue;
    use crate::value_extraction::{extract_ascii_value, extract_long_value, extract_short_value};
    use tracing::{debug, warn};

    debug!("Processing Equipment subdirectory: {} bytes", data.len());

    // Detect format based on data structure (ExifTool's format condition logic)
    let tiff_format = detect_equipment_format(data);
    debug!("Detected Equipment format: {}", tiff_format);

    // ExifTool format condition: $format ne "ifd" and $format ne "int32u"
    // If format is NOT ifd/int32u, use binary Equipment processing
    // If format IS ifd/int32u, use EquipmentIFD (standard IFD) processing
    let use_binary_format = tiff_format != "ifd" && tiff_format != "int32u";

    if use_binary_format {
        debug!(
            "Using binary Equipment processing for format: {}",
            tiff_format
        );
        // TODO: Implement binary Equipment parsing for old cameras (E-1, E-300)
        // This requires handling the malformed subdirectory format that ExifTool mentions
        warn!("Binary Equipment format not yet implemented - this affects older Olympus cameras");
        return Ok(vec![]);
    }

    debug!("Using IFD Equipment processing for format: {}", tiff_format);

    // Process as standard TIFF IFD structure
    let mut equipment_tags = Vec::new();

    if data.len() < 2 {
        warn!("Equipment IFD data too short: {} bytes", data.len());
        return Ok(equipment_tags);
    }

    // Parse IFD entry count
    let entry_count = byte_order.read_u16(data, 0)?;
    debug!("Equipment IFD contains {} entries", entry_count);

    if data.len() < (2 + entry_count as usize * 12) {
        warn!(
            "Equipment IFD truncated: need {} bytes, have {}",
            2 + entry_count as usize * 12,
            data.len()
        );
        return Ok(equipment_tags);
    }

    // Process each IFD entry
    for i in 0..entry_count {
        let entry_offset = 2 + (i as usize * 12);

        match IfdEntry::parse_with_context(data, entry_offset, byte_order, true) {
            Ok(entry) => {
                let tag_name = get_equipment_tag_name(entry.tag_id);
                debug!(
                    "Equipment tag 0x{:04x} ({}): format={:?}, count={}, offset=0x{:08x}",
                    entry.tag_id, tag_name, entry.format, entry.count, entry.value_or_offset
                );

                // Extract tag value based on TIFF format
                let tag_value = match entry.format {
                    TiffFormat::Ascii => {
                        match extract_ascii_value(data, &entry, byte_order, entry.tag_id) {
                            Ok(s) => TagValue::String(s),
                            Err(e) => {
                                warn!(
                                    "Failed to extract ASCII value for Equipment tag 0x{:04x}: {}",
                                    entry.tag_id, e
                                );
                                continue;
                            }
                        }
                    }
                    TiffFormat::Short => match extract_short_value(data, &entry, byte_order) {
                        Ok(val) => TagValue::U16(val),
                        Err(e) => {
                            warn!(
                                "Failed to extract short value for Equipment tag 0x{:04x}: {}",
                                entry.tag_id, e
                            );
                            continue;
                        }
                    },
                    TiffFormat::Long => match extract_long_value(data, &entry, byte_order) {
                        Ok(val) => TagValue::U32(val),
                        Err(e) => {
                            warn!(
                                "Failed to extract long value for Equipment tag 0x{:04x}: {}",
                                entry.tag_id, e
                            );
                            continue;
                        }
                    },
                    TiffFormat::Undefined => {
                        // Handle undefined format as byte array (common for Equipment data)
                        if entry.count <= 4 {
                            // Data is inline in the offset field
                            let bytes = entry.value_or_offset.to_le_bytes();
                            let actual_bytes = &bytes[..entry.count.min(4) as usize];
                            TagValue::U8Array(actual_bytes.to_vec())
                        } else {
                            // Data is at offset location
                            if (entry.value_or_offset as usize + entry.count as usize) <= data.len()
                            {
                                let start = entry.value_or_offset as usize;
                                let end = start + entry.count as usize;
                                TagValue::U8Array(data[start..end].to_vec())
                            } else {
                                warn!("Equipment undefined data out of bounds: offset={}, count={}, data_len={}", 
                                     entry.value_or_offset, entry.count, data.len());
                                continue;
                            }
                        }
                    }
                    _ => {
                        debug!(
                            "Unsupported Equipment tag format: {:?} for tag 0x{:04x}",
                            entry.format, entry.tag_id
                        );
                        continue;
                    }
                };

                debug!("Extracted Equipment tag '{}': {:?}", tag_name, tag_value);
                equipment_tags.push((tag_name, tag_value));
            }
            Err(e) => {
                warn!("Failed to parse Equipment IFD entry {}: {}", i, e);
                continue;
            }
        }
    }

    debug!(
        "Equipment IFD processing complete: {} tags extracted",
        equipment_tags.len()
    );
    Ok(equipment_tags)
}

/// Detect Equipment format based on data structure
/// ExifTool: Implements format condition evaluation
fn detect_equipment_format(data: &[u8]) -> String {
    if data.len() < 2 {
        return "unknown".to_string();
    }

    // Check if it looks like a valid IFD structure
    // IFD starts with entry count (typically 1-50 for Equipment)
    let entry_count = u16::from_le_bytes([data[0], data[1]]);

    // Equipment IFDs typically have 5-25 entries
    if entry_count > 0 && entry_count <= 50 {
        let expected_size = 2 + (entry_count as usize * 12) + 4; // entries + next IFD offset
        if data.len() >= expected_size - 4 {
            // Allow missing next IFD offset
            // Additional validation: check if IFD entries look valid
            if validate_ifd_structure(data, entry_count) {
                return "ifd".to_string();
            }
        }
    }

    // If it doesn't look like IFD, assume binary format (old cameras)
    "undef".to_string()
}

/// Generate synthetic ID for Equipment tags
/// Creates unique IDs in the synthetic range (0x8000-0xFFFF) for Equipment tags
fn generate_equipment_synthetic_id(tag_name: &str) -> u16 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Create a hash from the tag name
    let mut hasher = DefaultHasher::new();
    "Olympus::Equipment".hash(&mut hasher);
    tag_name.hash(&mut hasher);
    let hash = hasher.finish();

    // Map to synthetic ID range (0x8000-0xFFFF)
    0x8000 | ((hash & 0x7FFF) as u16)
}

/// Validate IFD structure for Equipment data
fn validate_ifd_structure(data: &[u8], entry_count: u16) -> bool {
    if data.len() < 2 + (entry_count as usize * 12) {
        return false;
    }

    // Check first few IFD entries for valid tag IDs and formats
    for i in 0..entry_count.min(3) {
        let entry_offset = 2 + (i as usize * 12);
        if entry_offset + 12 > data.len() {
            return false;
        }

        let tag_id = u16::from_le_bytes([data[entry_offset], data[entry_offset + 1]]);
        let format = u16::from_le_bytes([data[entry_offset + 2], data[entry_offset + 3]]);

        // Equipment tags are typically in ranges 0x000-0x1003
        if tag_id > 0x1003 {
            return false;
        }

        // Check for valid TIFF format
        if format == 0 || format > 12 {
            return false;
        }
    }

    true
}

/// Get Equipment tag name from tag ID
/// ExifTool: Olympus.pm Equipment table lines 1588-1769
fn get_equipment_tag_name(tag_id: u16) -> String {
    match tag_id {
        0x000 => "EquipmentVersion".to_string(),
        0x100 => "CameraType2".to_string(),
        0x101 => "SerialNumber".to_string(),
        0x102 => "InternalSerialNumber".to_string(),
        0x103 => "FocalPlaneDiagonal".to_string(),
        0x104 => "BodyFirmwareVersion".to_string(),
        0x201 => "LensType".to_string(),
        0x202 => "LensSerialNumber".to_string(),
        0x203 => "LensModel".to_string(),
        0x204 => "LensFirmwareVersion".to_string(),
        0x205 => "MaxApertureAtMinFocal".to_string(),
        0x206 => "MaxApertureAtMaxFocal".to_string(),
        0x207 => "MinFocalLength".to_string(),
        0x208 => "MaxFocalLength".to_string(),
        0x20a => "MaxAperture".to_string(),
        0x20b => "LensProperties".to_string(),
        0x301 => "Extender".to_string(),
        0x302 => "ExtenderSerialNumber".to_string(),
        0x303 => "ExtenderModel".to_string(),
        0x304 => "ExtenderFirmwareVersion".to_string(),
        0x1000 => "FlashType".to_string(),
        0x1001 => "FlashModel".to_string(),
        0x1002 => "FlashFirmwareVersion".to_string(),
        0x1003 => "FlashSerialNumber".to_string(),
        _ => format!("Olympus_0x{:04X}", tag_id), // Unknown Equipment tag
    }
}

/// Get Olympus tag name from synthetic tag ID used in MakerNotes namespace
/// This maps the synthetic Equipment tag IDs back to their proper names
pub fn get_olympus_tag_name(tag_id: u16) -> Option<String> {
    match tag_id {
        // Synthetic Equipment tag IDs (defined in src/exif/ifd.rs:167-169)
        0xF100 => Some("CameraType2".to_string()), // Equipment 0x100
        0xF101 => Some("SerialNumber".to_string()), // Equipment 0x101
        0xF201 => Some("LensType".to_string()),    // Equipment 0x201
        _ => None,                                 // Not an Olympus synthetic tag
    }
}

/// Process Olympus subdirectory tags with custom Equipment processing
/// ExifTool: Olympus.pm SubDirectory processing for binary data expansion
///
/// This function handles Equipment (0x2010) subdirectory processing directly
/// to bypass the stubbed generated code and provide proper tag name resolution.
pub fn process_olympus_subdirectory_tags(
    exif_reader: &mut crate::exif::ExifReader,
) -> crate::types::Result<()> {
    use crate::exif::subdirectory_processing::process_subdirectories_with_printconv;
    // TODO: Task E - Replace tag_kit functions with manufacturer-specific implementations
    // use crate::generated::olympus::tag_kit;
    use crate::tiff_types::ByteOrder;
    use crate::types::TagValue;
    use tracing::debug;

    debug!("Processing Olympus subdirectory tags with custom Equipment handling");

    // First, handle Equipment subdirectory (0x2010) directly to bypass generated stub
    if let Some(equipment_tag) = exif_reader.get_tag_across_namespaces(0x2010) {
        debug!("Found Equipment tag (0x2010), processing with custom handler");

        if let TagValue::Binary(equipment_data) | TagValue::U8Array(equipment_data) = equipment_tag
        {
            debug!(
                "Equipment tag contains {} bytes of binary data",
                equipment_data.len()
            );

            // Get byte order from TIFF header
            let byte_order = exif_reader
                .header
                .as_ref()
                .map(|h| h.byte_order)
                .unwrap_or(ByteOrder::LittleEndian);

            // Process Equipment subdirectory with our custom processor
            match process_equipment_subdirectory(equipment_data, byte_order) {
                Ok(equipment_tags) => {
                    debug!(
                        "Custom Equipment processor extracted {} tags",
                        equipment_tags.len()
                    );

                    // Store Equipment tags with proper namespace and synthetic IDs
                    for (tag_name, tag_value) in equipment_tags {
                        // Generate synthetic ID for Equipment tag
                        let synthetic_id = generate_equipment_synthetic_id(&tag_name);

                        debug!(
                            "Storing Equipment tag '{}' with synthetic ID 0x{:04x}: {:?}",
                            tag_name, synthetic_id, tag_value
                        );

                        // Store with Olympus namespace for proper grouping
                        let source_info = crate::types::TagSourceInfo::new(
                            "Olympus".to_string(),
                            "Olympus".to_string(),
                            "Olympus::Equipment".to_string(),
                        );

                        exif_reader.store_tag_with_precedence(synthetic_id, tag_value, source_info);

                        // Also store the tag name mapping for proper output
                        let full_tag_name = format!("Olympus:{}", tag_name);
                        exif_reader
                            .synthetic_tag_names
                            .insert(synthetic_id, full_tag_name);
                    }
                }
                Err(e) => {
                    debug!("Custom Equipment processor failed: {}", e);
                }
            }
        } else {
            debug!(
                "Equipment tag exists but is not binary data: {:?}",
                equipment_tag
            );
        }
    } else {
        debug!("No Equipment tag (0x2010) found");
    }

    // Then continue with regular subdirectory processing for other tags
    debug!("Processing remaining Olympus subdirectory tags using generic system");

    // Use the generic subdirectory processing with Olympus-specific functions
    // Fix Group1 assignment: Use "Olympus" as namespace for group1="Olympus" instead of "MakerNotes"
    // TODO: Task E - Replace tag_kit functions with manufacturer-specific implementations
    // process_subdirectories_with_printconv(
    //     exif_reader,
    //     "Olympus",
    //     "Olympus",
    //     tag_kit::has_subdirectory,
    //     tag_kit::process_subdirectory,
    //     tag_kit::apply_print_conv,
    //     find_olympus_tag_id_by_name,
    // )?;

    debug!("Olympus subdirectory processing completed");
    Ok(())
}
