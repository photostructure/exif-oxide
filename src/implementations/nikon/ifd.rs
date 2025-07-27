//! Nikon IFD processing and tag extraction
//!
//! **Trust ExifTool**: This code translates ExifTool's Nikon IFD processing verbatim.
//!
//! ExifTool Reference: lib/Image/ExifTool/Nikon.pm IFD processing and tag extraction
//!
//! This module handles:
//! - IFD entry parsing and value extraction
//! - Encryption key pre-scanning (SerialNumber, ShutterCount)
//! - Standard Nikon tag processing with PrintConv application
//! - Model-specific tag table selection

use crate::exif::ExifReader;
use crate::implementations::nikon::{af_processing, encryption, tags};
use crate::tiff_types::{ByteOrder, IfdEntry, TiffFormat};
use crate::types::{ExifError, Result, TagValue};
use crate::value_extraction;
use tracing::{debug, trace};

/// Extract string value from IFD entry for encryption keys
/// ExifTool: String extraction for SerialNumber tag
pub fn extract_string_value(
    data: &[u8],
    entry: &IfdEntry,
    byte_order: ByteOrder,
) -> Result<String> {
    match entry.format {
        TiffFormat::Ascii => value_extraction::extract_ascii_value(data, entry, byte_order),
        _ => {
            // For non-ASCII formats, convert to string representation
            match entry.format {
                TiffFormat::Byte => {
                    let val = value_extraction::extract_byte_value(data, entry)?;
                    Ok(val.to_string())
                }
                TiffFormat::Short => {
                    let val = value_extraction::extract_short_value(data, entry, byte_order)?;
                    Ok(val.to_string())
                }
                TiffFormat::Long => {
                    let val = value_extraction::extract_long_value(data, entry, byte_order)?;
                    Ok(val.to_string())
                }
                _ => Err(ExifError::ParseError(format!(
                    "Unsupported format for string extraction: {:?}",
                    entry.format
                ))),
            }
        }
    }
}

/// Extract numeric value from IFD entry for encryption keys
/// ExifTool: Numeric extraction for ShutterCount tag
pub fn extract_numeric_value(data: &[u8], entry: &IfdEntry, byte_order: ByteOrder) -> Result<u32> {
    match entry.format {
        TiffFormat::Byte => {
            let val = value_extraction::extract_byte_value(data, entry)?;
            Ok(val as u32)
        }
        TiffFormat::Short => {
            let val = value_extraction::extract_short_value(data, entry, byte_order)?;
            Ok(val as u32)
        }
        TiffFormat::Long => value_extraction::extract_long_value(data, entry, byte_order),
        _ => Err(ExifError::ParseError(format!(
            "Unsupported format for numeric extraction: {:?}",
            entry.format
        ))),
    }
}

/// Pre-scan Nikon data for encryption keys before main processing
/// ExifTool: Nikon.pm pre-scan logic for tags 0x001d and 0x00a7
pub fn prescan_for_encryption_keys(
    reader: &ExifReader,
    base_offset: usize,
    keys: &mut encryption::NikonEncryptionKeys,
) -> Result<()> {
    trace!("Pre-scanning Nikon data for encryption keys");

    let data = reader.get_data();

    // Validate we have enough data for an IFD
    if base_offset + 2 > data.len() {
        debug!(
            "Insufficient data for Nikon key pre-scan at offset {:#x}",
            base_offset
        );
        return Ok(());
    }

    // Get byte order from the reader's header
    let byte_order = match &reader.header {
        Some(header) => header.byte_order,
        None => {
            debug!("No TIFF header available for byte order, using little endian");
            ByteOrder::LittleEndian
        }
    };

    // Read number of IFD entries
    let num_entries = match byte_order.read_u16(data, base_offset) {
        Ok(count) => count as usize,
        Err(_) => {
            debug!(
                "Failed to read IFD entry count at offset {:#x}",
                base_offset
            );
            return Ok(());
        }
    };

    debug!(
        "Pre-scanning {} IFD entries for Nikon encryption keys",
        num_entries
    );

    // Process each IFD entry looking for encryption key tags
    // ExifTool: Nikon.pm pre-scan for SerialNumber (0x001d) and ShutterCount (0x00a7)
    for index in 0..num_entries {
        let entry_offset = base_offset + 2 + 12 * index;

        if entry_offset + 12 > data.len() {
            debug!(
                "Entry {} at offset {:#x} beyond data bounds",
                index, entry_offset
            );
            break;
        }

        // Parse IFD entry
        let entry = match IfdEntry::parse(data, entry_offset, byte_order) {
            Ok(entry) => entry,
            Err(e) => {
                trace!("Failed to parse IFD entry {}: {:?}", index, e);
                continue;
            }
        };

        // Check for encryption key tags
        match entry.tag_id {
            0x001d => {
                // SerialNumber encryption key
                trace!("Found SerialNumber tag (0x001d) at entry {}", index);

                if let Ok(serial) = extract_string_value(data, &entry, byte_order) {
                    debug!("Extracted Nikon serial number for encryption: {}", serial);
                    keys.store_serial_key(serial);
                } else {
                    debug!("Failed to extract SerialNumber value");
                }
            }
            0x00a7 => {
                // ShutterCount encryption key
                trace!("Found ShutterCount tag (0x00a7) at entry {}", index);

                if let Ok(count) = extract_numeric_value(data, &entry, byte_order) {
                    debug!("Extracted Nikon shutter count for encryption: {}", count);
                    keys.store_count_key(count);
                } else {
                    debug!("Failed to extract ShutterCount value");
                }
            }
            _ => {
                // Not an encryption key tag, skip
                continue;
            }
        }
    }

    if keys.has_required_keys() {
        debug!("Nikon encryption key pre-scan completed successfully - both keys available");
    } else {
        debug!("Nikon encryption key pre-scan completed - some keys missing");
    }

    Ok(())
}

/// Extract tag value from IFD entry using appropriate format
/// ExifTool: Generic tag value extraction for different TIFF formats
pub fn extract_tag_value(data: &[u8], entry: &IfdEntry, byte_order: ByteOrder) -> Result<TagValue> {
    match entry.format {
        TiffFormat::Byte => {
            let val = value_extraction::extract_byte_value(data, entry)?;
            Ok(TagValue::U8(val))
        }
        TiffFormat::Ascii => {
            let val = value_extraction::extract_ascii_value(data, entry, byte_order)?;
            Ok(TagValue::String(val))
        }
        TiffFormat::Short => {
            let val = value_extraction::extract_short_value(data, entry, byte_order)?;
            Ok(TagValue::U16(val))
        }
        TiffFormat::Long => {
            let val = value_extraction::extract_long_value(data, entry, byte_order)?;
            Ok(TagValue::U32(val))
        }
        TiffFormat::Rational => {
            let val = value_extraction::extract_rational_value(data, entry, byte_order)?;
            Ok(val) // TagValue::Rational is already returned from extract_rational_value
        }
        _ => {
            // For unsupported formats, extract raw bytes
            trace!(
                "Unsupported format {:?} for tag {:#x}, extracting as raw bytes",
                entry.format,
                entry.tag_id
            );
            let raw_bytes = if entry.is_inline() {
                // Value stored inline in the 4-byte value field
                entry.value_or_offset.to_le_bytes().to_vec()
            } else {
                // Value stored at offset
                let offset = entry.value_or_offset as usize;
                let size = entry.count as usize;

                if offset + size > data.len() {
                    return Err(ExifError::ParseError(format!(
                        "Binary value offset {offset:#x} + size {size} beyond data bounds"
                    )));
                }

                data[offset..offset + size].to_vec()
            };
            Ok(TagValue::Binary(raw_bytes))
        }
    }
}

/// Apply Nikon-specific PrintConv function if available
/// ExifTool: Nikon.pm PrintConv function application
pub fn apply_nikon_print_conv(
    tag_id: u16,
    tag_value: TagValue,
    tag_table: &tags::NikonTagTable,
) -> TagValue {
    // Look for PrintConv function in the tag table
    for (id, _name, print_conv_fn) in tag_table.tags {
        if *id == tag_id {
            if let Some(conv_fn) = print_conv_fn {
                // Apply the PrintConv function
                match conv_fn(&tag_value) {
                    Ok(converted) => {
                        trace!(
                            "Applied PrintConv to tag {:#x}: {} -> {}",
                            tag_id,
                            tag_value,
                            converted
                        );
                        return TagValue::String(converted);
                    }
                    Err(e) => {
                        trace!("PrintConv failed for tag {:#x}: {}", tag_id, e);
                        return tag_value; // Return original value on conversion failure
                    }
                }
            }
            break;
        }
    }

    // No PrintConv function or not found, return original value
    tag_value
}

/// Extract raw binary data from IFD entry
/// For special processing of binary tags like AFInfo
fn extract_raw_data(data: &[u8], entry: &IfdEntry) -> Result<Vec<u8>> {
    if entry.is_inline() {
        // Value stored inline in the 4-byte value field
        let bytes = entry.value_or_offset.to_le_bytes();
        Ok(bytes[..entry.count.min(4) as usize].to_vec())
    } else {
        // Value stored at offset
        let offset = entry.value_or_offset as usize;
        let size = entry.count as usize;

        if offset + size > data.len() {
            return Err(ExifError::ParseError(format!(
                "Binary data offset {offset:#x} + size {size} beyond data bounds"
            )));
        }

        Ok(data[offset..offset + size].to_vec())
    }
}

/// Process standard (non-encrypted) Nikon tags
/// ExifTool: Nikon.pm standard tag processing
pub fn process_standard_nikon_tags(
    reader: &mut ExifReader,
    base_offset: usize,
    keys: &encryption::NikonEncryptionKeys,
) -> Result<()> {
    trace!(
        "Processing standard Nikon tags at offset {:#x}",
        base_offset
    );

    // Copy the data to avoid borrow checker issues with mutable reader access later
    let data = reader.get_data().to_vec();

    // Validate we have enough data for an IFD
    if base_offset + 2 > data.len() {
        debug!(
            "Insufficient data for Nikon tag processing at offset {:#x}",
            base_offset
        );
        return Ok(());
    }

    // Get byte order from the reader's header
    let byte_order = match &reader.header {
        Some(header) => header.byte_order,
        None => {
            debug!("No TIFF header available for byte order, using little endian");
            ByteOrder::LittleEndian
        }
    };

    // Read number of IFD entries
    let num_entries = match byte_order.read_u16(&data, base_offset) {
        Ok(count) => count as usize,
        Err(_) => {
            debug!(
                "Failed to read IFD entry count at offset {:#x}",
                base_offset
            );
            return Ok(());
        }
    };

    debug!("Processing {} Nikon IFD entries", num_entries);

    // Get the camera model from existing tags for table selection
    // Clone to avoid borrowing issues later when mutably borrowing reader
    let model = reader
        .get_tag_across_namespaces(0x0110) // Model tag
        .and_then(|v| v.as_string())
        .unwrap_or("Unknown Nikon")
        .to_string();

    // Select appropriate tag table based on camera model
    let tag_table = tags::select_nikon_tag_table(&model);
    debug!(
        "Using Nikon tag table: {} for model: {}",
        tag_table.name, model
    );

    // Process each IFD entry
    // ExifTool: Nikon.pm tag processing with PrintConv functions
    for index in 0..num_entries {
        let entry_offset = base_offset + 2 + 12 * index;

        if entry_offset + 12 > data.len() {
            debug!(
                "Entry {} at offset {:#x} beyond data bounds",
                index, entry_offset
            );
            break;
        }

        // Parse IFD entry
        let entry = match IfdEntry::parse(&data, entry_offset, byte_order) {
            Ok(entry) => entry,
            Err(e) => {
                trace!("Failed to parse IFD entry {}: {:?}", index, e);
                continue;
            }
        };

        // Look up tag in Nikon tag table
        if let Some(tag_name) = tags::get_nikon_tag_name(entry.tag_id, &model) {
            trace!("Processing Nikon tag {:#x}: {}", entry.tag_id, tag_name);

            // Special processing for AF Info tag
            if entry.tag_id == 0x0088 && tag_name == "AFInfo" {
                // Extract AF Info binary data and process it specially
                match extract_raw_data(&data, &entry) {
                    Ok(af_data) => {
                        debug!("Processing AF Info data ({} bytes)", af_data.len());

                        // Process AF Info using dedicated AF processing module
                        if let Err(e) =
                            af_processing::process_nikon_af_info(reader, &af_data, &model)
                        {
                            debug!("AF Info processing failed: {:?}", e);
                        }

                        // Also store the raw AF Info tag for compatibility
                        let tag_source = reader.create_tag_source_info("Nikon");
                        reader.store_tag_with_precedence(
                            entry.tag_id,
                            TagValue::Binary(af_data),
                            tag_source,
                        );
                    }
                    Err(e) => {
                        debug!("Failed to extract AF Info binary data: {:?}", e);
                    }
                }
            } else {
                // Standard tag processing
                match extract_tag_value(&data, &entry, byte_order) {
                    Ok(tag_value) => {
                        // Apply Nikon-specific PrintConv if available
                        let final_value =
                            apply_nikon_print_conv(entry.tag_id, tag_value, tag_table);

                        // Store the tag with proper namespace
                        let tag_source = reader.create_tag_source_info("Nikon");
                        reader.store_tag_with_precedence(entry.tag_id, final_value, tag_source);

                        debug!("Stored Nikon tag {:#x}: {}", entry.tag_id, tag_name);
                    }
                    Err(e) => {
                        trace!(
                            "Failed to extract tag {:#x} ({}): {:?}",
                            entry.tag_id,
                            tag_name,
                            e
                        );
                    }
                }
            }
        } else {
            trace!("Unknown Nikon tag {:#x}, skipping", entry.tag_id);
        }
    }

    // Store encryption status information
    let tag_source = reader.create_tag_source_info("Nikon");
    let encryption_status = if keys.has_required_keys() {
        format!(
            "Keys available (serial: {}, count: {})",
            keys.get_serial_key().unwrap_or("none"),
            keys.get_count_key()
                .map(|c| c.to_string())
                .unwrap_or("none".to_string())
        )
    } else {
        "Keys incomplete for decryption".to_string()
    };

    reader.store_tag_with_precedence(
        0x00FD, // Custom tag for encryption status
        TagValue::String(encryption_status),
        tag_source,
    );

    debug!(
        "Standard Nikon tag processing completed - {} entries processed",
        num_entries
    );
    Ok(())
}
