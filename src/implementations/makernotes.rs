// MakerNotes conditional dispatch system
// Implements ExifTool's signature-based manufacturer detection
// Based on third-party/exiftool/lib/Image/ExifTool/MakerNotes.pm

use crate::generated::fuji_film::main_tags::FUJIFILM_MAIN_TAGS;
use crate::tiff_types::ByteOrder;
use crate::types::{Result, TagValue};
use tracing::{debug, trace};

/// Override for the generated process_tag_0x927c_subdirectory function
/// This replaces the stub implementation with proper conditional dispatch
pub fn process_tag_0x927c_subdirectory(
    data: &[u8],
    byte_order: ByteOrder,
) -> Result<Vec<(String, TagValue)>> {
    debug!(
        "MakerNotes override: processing {} bytes with conditional dispatch",
        data.len()
    );

    // Delegate to the conditional dispatch system
    process_makernotes_conditional_dispatch(data, byte_order)
}

/// MakerNotes conditional dispatch processor
/// Implements ExifTool's pattern matching system from MakerNotes.pm
///
/// Reference: third-party/exiftool/lib/Image/ExifTool/MakerNotes.pm:34-1102
/// The @Image::ExifTool::MakerNotes::Main array contains conditional entries
/// that are evaluated sequentially until first match
pub fn process_makernotes_conditional_dispatch(
    data: &[u8],
    byte_order: ByteOrder,
) -> Result<Vec<(String, TagValue)>> {
    debug!(
        "MakerNotes conditional dispatch: processing {} bytes",
        data.len()
    );

    // Get the first 32 bytes for signature matching (ExifTool pattern)
    let signature_bytes = if data.len() >= 32 { &data[0..32] } else { data };

    trace!("MakerNotes signature bytes: {:02X?}", signature_bytes);

    // Convert to string for pattern matching (safe subset for ASCII patterns)
    let signature_str = String::from_utf8_lossy(signature_bytes);
    debug!("MakerNotes signature string: {:?}", signature_str);

    // Evaluate conditions sequentially (first match wins)
    // Following the order from MakerNotes.pm:557-586 for Olympus variants

    // Check for Olympus signature patterns
    if let Some(olympus_tags) = check_olympus_patterns(&signature_str, data, byte_order)? {
        debug!("MakerNotes: Matched Olympus signature, delegating to Olympus processor");
        return Ok(olympus_tags);
    }

    // Check for other manufacturer patterns (stub for now)
    if let Some(other_tags) = check_other_manufacturer_patterns(&signature_str, data, byte_order)? {
        debug!("MakerNotes: Matched other manufacturer signature");
        return Ok(other_tags);
    }

    // No manufacturer-specific pattern matched, return generic processing
    debug!("MakerNotes: No manufacturer signature matched, using generic IFD processing");
    Ok(vec![])
}

/// Check for Olympus maker note signature patterns
/// Based on MakerNotes.pm:557-586
fn check_olympus_patterns(
    signature: &str,
    data: &[u8],
    byte_order: ByteOrder,
) -> Result<Option<Vec<(String, TagValue)>>> {
    // MakerNoteOlympus - Legacy format
    // Condition: $$valPt =~ /^(OLYMP|EPSON)\0/
    if signature.starts_with("OLYMP\0") || signature.starts_with("EPSON\0") {
        debug!("MakerNotes: Matched MakerNoteOlympus pattern (legacy OLYMP/EPSON)");
        return process_olympus_makernotes(data, byte_order, 8).map(Some);
    }

    // MakerNoteOlympus2 - Modern format
    // Condition: $$valPt =~ /^OLYMPUS\0/
    if signature.starts_with("OLYMPUS\0") {
        debug!("MakerNotes: Matched MakerNoteOlympus2 pattern (modern OLYMPUS)");
        return process_olympus_makernotes(data, byte_order, 12).map(Some);
    }

    // MakerNoteOlympus3 - OM System rebranding
    // Condition: $$valPt =~ /^OM SYSTEM\0/
    if signature.starts_with("OM SYSTEM\0") {
        debug!("MakerNotes: Matched MakerNoteOlympus3 pattern (OM SYSTEM)");
        return process_olympus_makernotes(data, byte_order, 16).map(Some);
    }

    Ok(None)
}

/// Process Olympus maker notes with calculated start offset
/// Delegates to the Olympus implementation
fn process_olympus_makernotes(
    data: &[u8],
    byte_order: ByteOrder,
    start_offset: usize,
) -> Result<Vec<(String, TagValue)>> {
    debug!(
        "Processing Olympus maker notes: {} bytes, start_offset={}",
        data.len(),
        start_offset
    );

    // Validate start offset
    if start_offset >= data.len() {
        debug!(
            "Start offset {} exceeds data length {}, returning empty",
            start_offset,
            data.len()
        );
        return Ok(vec![]);
    }

    // Extract the maker note data after the header
    let makernote_data = &data[start_offset..];

    // Delegate to the Olympus Equipment processor directly
    // This bypasses the full Olympus IFD processing and focuses on Equipment extraction
    debug!(
        "Processing Olympus maker notes data with {} bytes after header",
        makernote_data.len()
    );

    // The makernote_data should contain the Olympus IFD structure
    // We need to parse this as an IFD and extract the Equipment tag (0x2010)
    process_olympus_ifd_for_equipment(makernote_data, byte_order)
}

/// Process Olympus IFD specifically to extract Equipment tag (0x2010)
///
/// This parses the Equipment IFD structure to extract real Equipment tag data.
/// Follows ExifTool's dual format approach: TIFF IFD parsing for newer cameras.
///
/// Reference: third-party/exiftool/lib/Image/ExifTool/Olympus.pm:1620-1640 (Equipment dual format)
/// Reference: third-party/exiftool/lib/Image/ExifTool/Olympus.pm:4270-4350 (Equipment tag table)
fn process_olympus_ifd_for_equipment(
    data: &[u8],
    byte_order: ByteOrder,
) -> Result<Vec<(String, TagValue)>> {
    debug!(
        "Processing Olympus IFD for Equipment extraction: {} bytes",
        data.len()
    );

    if data.len() < 14 {
        debug!(
            "Equipment data too small for IFD structure (need at least 14 bytes for IFD header)"
        );
        return Ok(vec![]);
    }

    // Parse the Equipment data as a TIFF IFD structure
    // The data should start with the IFD entry count (2 bytes) followed by IFD entries
    let entry_count = match byte_order {
        ByteOrder::LittleEndian => u16::from_le_bytes([data[0], data[1]]) as usize,
        ByteOrder::BigEndian => u16::from_be_bytes([data[0], data[1]]) as usize,
    };

    debug!("Equipment IFD has {} entries", entry_count);

    if entry_count == 0 || entry_count > 100 {
        debug!("Invalid Equipment IFD entry count: {}", entry_count);
        return Ok(vec![]);
    }

    let expected_size = 2 + (entry_count * 12) + 4; // 2 bytes for count + 12 bytes per entry + 4 bytes for next IFD offset
    if data.len() < expected_size {
        debug!(
            "Equipment data too small: {} bytes, need {} bytes",
            data.len(),
            expected_size
        );
        return Ok(vec![]);
    }

    let mut equipment_tags = Vec::new();
    let mut offset = 2; // Skip the entry count

    // Parse each IFD entry (12 bytes each)
    for i in 0..entry_count {
        if offset + 12 > data.len() {
            debug!("Not enough data for Equipment IFD entry {}", i);
            break;
        }

        // Parse IFD entry: tag (2) + type (2) + count (4) + value/offset (4)
        let tag_id = match byte_order {
            ByteOrder::LittleEndian => u16::from_le_bytes([data[offset], data[offset + 1]]),
            ByteOrder::BigEndian => u16::from_be_bytes([data[offset], data[offset + 1]]),
        };

        let format_id = match byte_order {
            ByteOrder::LittleEndian => u16::from_le_bytes([data[offset + 2], data[offset + 3]]),
            ByteOrder::BigEndian => u16::from_be_bytes([data[offset + 2], data[offset + 3]]),
        };

        let count = match byte_order {
            ByteOrder::LittleEndian => u32::from_le_bytes([
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]),
            ByteOrder::BigEndian => u32::from_be_bytes([
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]),
        };

        let value_offset = match byte_order {
            ByteOrder::LittleEndian => u32::from_le_bytes([
                data[offset + 8],
                data[offset + 9],
                data[offset + 10],
                data[offset + 11],
            ]),
            ByteOrder::BigEndian => u32::from_be_bytes([
                data[offset + 8],
                data[offset + 9],
                data[offset + 10],
                data[offset + 11],
            ]),
        } as usize;

        debug!(
            "Equipment entry {}: tag=0x{:x}, format={}, count={}, value_offset={}",
            i, tag_id, format_id, count, value_offset
        );

        // Extract value based on tag ID and format
        if let Some(tag_value) =
            extract_equipment_tag_value(tag_id, format_id, count, value_offset, data, byte_order)
        {
            let tag_name = get_equipment_tag_name(tag_id);
            debug!("Extracted Equipment tag: {} = {:?}", tag_name, tag_value);
            equipment_tags.push((tag_name, tag_value));
        } else {
            debug!("Could not extract value for Equipment tag 0x{:x}", tag_id);
        }

        offset += 12;
    }

    debug!(
        "Equipment extraction complete, returning {} real tags",
        equipment_tags.len()
    );
    Ok(equipment_tags)
}

/// Extract Equipment tag value based on tag ID, format, and data location
///
/// Reference: third-party/exiftool/lib/Image/ExifTool/Olympus.pm:4270-4350
fn extract_equipment_tag_value(
    tag_id: u16,
    format_id: u16,
    count: u32,
    value_offset: usize,
    data: &[u8],
    _byte_order: ByteOrder,
) -> Option<TagValue> {
    match tag_id {
        // 0x100 - CameraType2 (string, count: 6)
        0x100 => {
            if format_id == 2 && count <= 32 {
                // ASCII format
                let value_data = if count <= 4 {
                    // Value fits in the value field itself
                    &data[value_offset.saturating_sub(8)
                        ..value_offset.saturating_sub(8) + count as usize]
                } else if value_offset < data.len() && value_offset + count as usize <= data.len() {
                    // Value is at offset location
                    &data[value_offset..value_offset + count as usize]
                } else {
                    return None;
                };

                if let Ok(camera_type) = std::str::from_utf8(value_data) {
                    let cleaned = camera_type.trim_end_matches('\0').trim();
                    return Some(TagValue::String(cleaned.to_string()));
                }
            }
        }

        // 0x101 - SerialNumber (string, count: 32)
        0x101 => {
            if format_id == 2 && count <= 32 {
                // ASCII format
                let value_data =
                    if value_offset < data.len() && value_offset + count as usize <= data.len() {
                        &data[value_offset..value_offset + count as usize]
                    } else {
                        return None;
                    };

                if let Ok(serial) = std::str::from_utf8(value_data) {
                    let cleaned = serial.trim_end_matches('\0').trim();
                    return Some(TagValue::String(cleaned.to_string()));
                }
            }
        }

        // 0x201 - LensType (6 bytes: Make, Unknown, Model, Sub-model, Unknown, Unknown)
        0x201 => {
            if format_id == 1 && count == 6 {
                // BYTE format, 6 bytes
                let lens_data = if value_offset < data.len() && value_offset + 6 <= data.len() {
                    &data[value_offset..value_offset + 6]
                } else {
                    return None;
                };

                // Convert 6-byte array to hex string format (only use bytes 0, 2, 3)
                // Reference: sprintf("%x %.2x %.2x", @a[0,2,3]) from Olympus.pm
                let hex_key = format!(
                    "{:x} {:02x} {:02x}",
                    lens_data[0], lens_data[2], lens_data[3]
                );
                debug!(
                    "LensType raw bytes: {:?}, hex key: '{}'",
                    lens_data, hex_key
                );

                // For now, return the raw hex key - the LensID composite will handle lookup
                return Some(TagValue::String(hex_key));
            }
        }

        _ => {
            debug!("Unknown Equipment tag 0x{:x}, skipping", tag_id);
        }
    }

    None
}

/// Get Equipment tag name from tag ID
///
/// Maps Equipment tag IDs to their proper names for namespace storage
/// Reference: third-party/exiftool/lib/Image/ExifTool/Olympus.pm:4270-4350
fn get_equipment_tag_name(tag_id: u16) -> String {
    match tag_id {
        0x100 => "CameraType2".to_string(),
        0x101 => "SerialNumber".to_string(),
        0x201 => "LensType".to_string(),
        0x202 => "LensSerialNumber".to_string(),
        0x203 => "LensModel".to_string(),
        0x204 => "LensFirmwareVersion".to_string(),
        0x205 => "MaxAperture".to_string(),
        0x206 => "MinAperture".to_string(),
        0x207 => "FocalLength".to_string(),
        0x208 => "MaxFocalLength".to_string(),
        _ => format!("Olympus_0x{:04X}", tag_id),
    }
}

/// Check for other manufacturer signature patterns
/// TODO: Implement Canon, Nikon, Sony, etc. when needed
fn check_other_manufacturer_patterns(
    signature: &str,
    data: &[u8],
    byte_order: ByteOrder,
) -> Result<Option<Vec<(String, TagValue)>>> {
    // Check for FujiFilm signature patterns
    // Reference: MakerNotes.pm - Condition => '$valPt =~ /^(FUJIFILM|GENERALE)/'
    if signature.starts_with("FUJIFILM") || signature.starts_with("GENERALE") {
        debug!("MakerNotes: Matched FujiFilm signature pattern");
        return process_fujifilm_makernotes(data, byte_order).map(Some);
    }

    // Placeholder for other manufacturers
    // Canon: $$self{Make} =~ /^Canon/
    // Nikon: $$valPt=~/^Nikon\x00\x02/
    // Sony: Complex conditions for DSC/CAM/MOBILE variants
    // etc.

    Ok(None)
}

/// Process FujiFilm maker notes with ExifTool-exact specifications
/// Reference: MakerNotes.pm - MakerNoteFujiFilm SubDirectory settings
fn process_fujifilm_makernotes(
    data: &[u8],
    _byte_order: ByteOrder,
) -> Result<Vec<(String, TagValue)>> {
    debug!(
        "Processing FujiFilm maker notes: {} bytes total",
        data.len()
    );

    // ExifTool: OffsetPt => '$valuePtr+8' - Skip 8-byte header
    if data.len() < 8 {
        debug!(
            "FujiFilm MakerNotes data too small: {} bytes, need at least 8 for header",
            data.len()
        );
        return Ok(vec![]);
    }

    let makernote_data = &data[8..];
    debug!(
        "FujiFilm MakerNotes: skipped 8-byte header, processing {} bytes with LittleEndian",
        makernote_data.len()
    );

    // ExifTool: ByteOrder => 'LittleEndian' - Force LittleEndian regardless of input
    let byte_order = ByteOrder::LittleEndian;

    // Parse as TIFF IFD with FujiFilm tag kit for proper tag name resolution
    parse_fujifilm_ifd(makernote_data, byte_order)
}

/// Parse FujiFilm IFD and resolve tag names using the generated tag kit
fn parse_fujifilm_ifd(data: &[u8], byte_order: ByteOrder) -> Result<Vec<(String, TagValue)>> {
    debug!("Parsing FujiFilm IFD: {} bytes", data.len());

    if data.len() < 14 {
        debug!("FujiFilm IFD data too small for IFD structure (need at least 14 bytes)");
        return Ok(vec![]);
    }

    // Parse the IFD entry count (2 bytes)
    let entry_count = match byte_order {
        ByteOrder::LittleEndian => u16::from_le_bytes([data[0], data[1]]) as usize,
        ByteOrder::BigEndian => u16::from_be_bytes([data[0], data[1]]) as usize,
    };

    debug!("FujiFilm IFD has {} entries", entry_count);

    if entry_count == 0 || entry_count > 1000 {
        debug!("Invalid FujiFilm IFD entry count: {}", entry_count);
        return Ok(vec![]);
    }

    let expected_size = 2 + (entry_count * 12) + 4; // 2 bytes for count + 12 bytes per entry + 4 bytes for next IFD offset
    if data.len() < expected_size {
        debug!(
            "FujiFilm IFD data too small: {} bytes, need {} bytes",
            data.len(),
            expected_size
        );
        return Ok(vec![]);
    }

    let mut fujifilm_tags = Vec::new();
    let mut offset = 2; // Skip the entry count

    // Parse each IFD entry (12 bytes each)
    for i in 0..entry_count {
        if offset + 12 > data.len() {
            debug!("Not enough data for FujiFilm IFD entry {}", i);
            break;
        }

        // Parse IFD entry: tag (2) + type (2) + count (4) + value/offset (4)
        let tag_id = match byte_order {
            ByteOrder::LittleEndian => u16::from_le_bytes([data[offset], data[offset + 1]]) as u32,
            ByteOrder::BigEndian => u16::from_be_bytes([data[offset], data[offset + 1]]) as u32,
        };

        let format_id = match byte_order {
            ByteOrder::LittleEndian => u16::from_le_bytes([data[offset + 2], data[offset + 3]]),
            ByteOrder::BigEndian => u16::from_be_bytes([data[offset + 2], data[offset + 3]]),
        };

        let count = match byte_order {
            ByteOrder::LittleEndian => u32::from_le_bytes([
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]),
            ByteOrder::BigEndian => u32::from_be_bytes([
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]),
        };

        let value_offset = match byte_order {
            ByteOrder::LittleEndian => u32::from_le_bytes([
                data[offset + 8],
                data[offset + 9],
                data[offset + 10],
                data[offset + 11],
            ]),
            ByteOrder::BigEndian => u32::from_be_bytes([
                data[offset + 8],
                data[offset + 9],
                data[offset + 10],
                data[offset + 11],
            ]),
        } as usize;

        debug!(
            "FujiFilm entry {}: tag=0x{:x}, format={}, count={}, value_offset={}",
            i, tag_id, format_id, count, value_offset
        );

        // Extract the raw value (simplified for now)
        if let Some(raw_value) =
            extract_ifd_tag_value(format_id, count, value_offset, data, byte_order)
        {
            // Resolve tag name using FujiFilm tag kit
            let tag_name = get_fujifilm_tag_name(tag_id);
            debug!(
                "FujiFilm tag: {} (0x{:x}) = {:?}",
                tag_name, tag_id, raw_value
            );
            fujifilm_tags.push((tag_name, raw_value));
        } else {
            debug!("Could not extract value for FujiFilm tag 0x{:x}", tag_id);
        }

        offset += 12;
    }

    debug!(
        "FujiFilm IFD parsing complete, returning {} tags",
        fujifilm_tags.len()
    );
    Ok(fujifilm_tags)
}

/// Get FujiFilm tag name from tag ID using the generated tag kit
fn get_fujifilm_tag_name(tag_id: u32) -> String {
    if let Some(tag_kit) = FUJIFILM_MAIN_TAGS.get(&tag_id) {
        tag_kit.name.to_string()
    } else {
        format!("Fujifilm_0x{:04X}", tag_id)
    }
}

/// Extract IFD tag value based on format and data location (simplified)
fn extract_ifd_tag_value(
    format_id: u16,
    count: u32,
    value_offset: usize,
    data: &[u8],
    byte_order: ByteOrder,
) -> Option<TagValue> {
    match format_id {
        1 => {
            // BYTE format
            if count == 1 {
                // Single byte value - stored directly in value field
                if value_offset < data.len() {
                    Some(TagValue::U8(data[value_offset]))
                } else {
                    None
                }
            } else {
                // Multiple bytes - stored at offset
                if value_offset < data.len() && value_offset + count as usize <= data.len() {
                    let bytes = data[value_offset..value_offset + count as usize].to_vec();
                    Some(TagValue::U8Array(bytes))
                } else {
                    None
                }
            }
        }
        2 => {
            // ASCII format
            if value_offset < data.len() && value_offset + count as usize <= data.len() {
                let value_data = &data[value_offset..value_offset + count as usize];
                if let Ok(string_val) = std::str::from_utf8(value_data) {
                    let cleaned = string_val.trim_end_matches('\0').trim();
                    Some(TagValue::String(cleaned.to_string()))
                } else {
                    None
                }
            } else {
                None
            }
        }
        3 => {
            // SHORT format (16-bit unsigned)
            if count == 1 {
                // Single value - get from value field
                let value = match byte_order {
                    ByteOrder::LittleEndian => {
                        u16::from_le_bytes([data[value_offset], data[value_offset + 1]])
                    }
                    ByteOrder::BigEndian => {
                        u16::from_be_bytes([data[value_offset], data[value_offset + 1]])
                    }
                };
                Some(TagValue::U16(value))
            } else {
                // Multiple values - stored at offset
                if value_offset < data.len() && value_offset + (count as usize * 2) <= data.len() {
                    let mut values = Vec::new();
                    for i in 0..count as usize {
                        let offset = value_offset + (i * 2);
                        let value = match byte_order {
                            ByteOrder::LittleEndian => {
                                u16::from_le_bytes([data[offset], data[offset + 1]])
                            }
                            ByteOrder::BigEndian => {
                                u16::from_be_bytes([data[offset], data[offset + 1]])
                            }
                        };
                        values.push(value);
                    }
                    Some(TagValue::U16Array(values))
                } else {
                    None
                }
            }
        }
        4 => {
            // LONG format (32-bit unsigned)
            if count == 1 {
                let value = match byte_order {
                    ByteOrder::LittleEndian => u32::from_le_bytes([
                        data[value_offset],
                        data[value_offset + 1],
                        data[value_offset + 2],
                        data[value_offset + 3],
                    ]),
                    ByteOrder::BigEndian => u32::from_be_bytes([
                        data[value_offset],
                        data[value_offset + 1],
                        data[value_offset + 2],
                        data[value_offset + 3],
                    ]),
                };
                Some(TagValue::U32(value))
            } else {
                // Multiple values - stored at offset
                if value_offset < data.len() && value_offset + (count as usize * 4) <= data.len() {
                    let mut values = Vec::new();
                    for i in 0..count as usize {
                        let offset = value_offset + (i * 4);
                        let value = match byte_order {
                            ByteOrder::LittleEndian => u32::from_le_bytes([
                                data[offset],
                                data[offset + 1],
                                data[offset + 2],
                                data[offset + 3],
                            ]),
                            ByteOrder::BigEndian => u32::from_be_bytes([
                                data[offset],
                                data[offset + 1],
                                data[offset + 2],
                                data[offset + 3],
                            ]),
                        };
                        values.push(value);
                    }
                    Some(TagValue::U32Array(values))
                } else {
                    None
                }
            }
        }
        _ => {
            debug!("Unsupported FujiFilm format type: {}", format_id);
            None
        }
    }
}
