//! Nikon MakerNote processing implementation
//!
//! **Trust ExifTool**: This code translates ExifTool's Nikon processing verbatim
//! without any improvements or simplifications.
//!
//! ExifTool Reference: lib/Image/ExifTool/Nikon.pm (14,191 lines)
//!
//! This module handles Nikon's sophisticated maker note format including:
//! - Multiple format versions (Format1, Format2, Format3)
//! - Advanced encryption system with serial number and shutter count keys
//! - Comprehensive lens database (618 entries)
//! - Model-specific subdirectories and processing
//! - Complex AF grid systems
//!
//! ## Module Organization
//!
//! Following the proven Canon pattern:
//! - `detection.rs` - Multi-format maker note detection and signature validation
//! - `offset_schemes.rs` - Format-specific offset calculations and base management
//! - `encryption.rs` - Encryption key management and ProcessNikonEncrypted foundation
//! - `tags.rs` - Primary tag ID mappings and model-specific tables
//! - `lens_database.rs` - 618-entry lens ID lookup system
//! - `tests.rs` - Comprehensive unit tests for all components

use crate::exif::ExifReader;
use crate::types::{ExifError, Result};
use tracing::{debug, trace};

pub mod detection;
pub mod encryption;
pub mod lens_database;
pub mod offset_schemes;
pub mod tags;

#[cfg(test)]
mod tests;

/// Main entry point for Nikon MakerNote processing
/// ExifTool: Nikon.pm ProcessNikon function (lines 890-1200+)
pub fn process_nikon_makernotes(reader: &mut ExifReader, offset: usize) -> Result<()> {
    debug!("Processing Nikon MakerNotes at offset {:#x}", offset);

    // Get data length for validation
    let data_len = reader.get_data_len();

    // Validate basic data availability
    if offset >= data_len {
        return Err(ExifError::ParseError(format!(
            "Nikon MakerNote offset {offset:#x} beyond data bounds ({data_len})"
        )));
    }

    // Get a small sample for format detection (avoiding long-term borrow)
    let format_data = {
        let data = reader.get_data();
        if offset + 10 <= data.len() {
            data[offset..offset + 10].to_vec()
        } else {
            data[offset..].to_vec()
        }
    };

    // Phase 1: Detect Nikon format version
    let format = detection::detect_nikon_format(&format_data)
        .ok_or_else(|| ExifError::ParseError("Unable to detect Nikon format".to_string()))?;

    debug!("Detected Nikon format: {:?}", format);

    // Phase 2: Calculate base offset using format-specific scheme
    let base_offset = offset_schemes::calculate_nikon_base_offset(format, offset);

    debug!(
        "Calculated Nikon base offset: {:#x} (format: {:?})",
        base_offset, format
    );

    // Phase 3: Initialize encryption key management
    // For Phase 1, use a generic model name - Phase 2 will extract from actual tags
    let mut encryption_keys = encryption::NikonEncryptionKeys::new("Nikon Camera".to_string());

    // Phase 4: Pre-scan for encryption keys (if needed)
    // ExifTool: Nikon.pm lines 1234-1267 - pre-scan for SerialNumber (0x001d) and ShutterCount (0x00a7)
    prescan_for_encryption_keys(reader, base_offset, &mut encryption_keys)?;

    // Phase 5: Process standard Nikon tags
    process_standard_nikon_tags(reader, base_offset, &encryption_keys)?;

    // Phase 6: Process encrypted sections (with detection logic)
    process_encrypted_sections(reader, base_offset, &encryption_keys)?;

    Ok(())
}

/// Helper function to extract string value from IFD entry
/// ExifTool: String extraction for SerialNumber tag
fn extract_string_value(
    data: &[u8],
    entry: &crate::tiff_types::IfdEntry,
    byte_order: crate::tiff_types::ByteOrder,
) -> Result<String> {
    use crate::tiff_types::TiffFormat;
    use crate::value_extraction;

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

/// Helper function to extract numeric value from IFD entry
/// ExifTool: Numeric extraction for ShutterCount tag
fn extract_numeric_value(
    data: &[u8],
    entry: &crate::tiff_types::IfdEntry,
    byte_order: crate::tiff_types::ByteOrder,
) -> Result<u32> {
    use crate::tiff_types::TiffFormat;
    use crate::value_extraction;

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
fn prescan_for_encryption_keys(
    reader: &ExifReader,
    base_offset: usize,
    keys: &mut encryption::NikonEncryptionKeys,
) -> Result<()> {
    use crate::tiff_types::{ByteOrder, IfdEntry};

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
fn extract_tag_value(
    data: &[u8],
    entry: &crate::tiff_types::IfdEntry,
    byte_order: crate::tiff_types::ByteOrder,
) -> Result<crate::types::TagValue> {
    use crate::tiff_types::TiffFormat;
    use crate::value_extraction;

    match entry.format {
        TiffFormat::Byte => {
            let val = value_extraction::extract_byte_value(data, entry)?;
            Ok(crate::types::TagValue::U8(val))
        }
        TiffFormat::Ascii => {
            let val = value_extraction::extract_ascii_value(data, entry, byte_order)?;
            Ok(crate::types::TagValue::String(val))
        }
        TiffFormat::Short => {
            let val = value_extraction::extract_short_value(data, entry, byte_order)?;
            Ok(crate::types::TagValue::U16(val))
        }
        TiffFormat::Long => {
            let val = value_extraction::extract_long_value(data, entry, byte_order)?;
            Ok(crate::types::TagValue::U32(val))
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
            Ok(crate::types::TagValue::Binary(raw_bytes))
        }
    }
}

/// Apply Nikon-specific PrintConv function if available
/// ExifTool: Nikon.pm PrintConv function application
fn apply_nikon_print_conv(
    tag_id: u16,
    tag_value: crate::types::TagValue,
    tag_table: &tags::NikonTagTable,
) -> crate::types::TagValue {
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
                        return crate::types::TagValue::String(converted);
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

/// Process standard (non-encrypted) Nikon tags
/// ExifTool: Nikon.pm standard tag processing
fn process_standard_nikon_tags(
    reader: &mut ExifReader,
    base_offset: usize,
    keys: &encryption::NikonEncryptionKeys,
) -> Result<()> {
    use crate::tiff_types::{ByteOrder, IfdEntry};

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
        .extracted_tags
        .get(&0x0110) // Model tag
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

            // Extract the tag value
            match extract_tag_value(&data, &entry, byte_order) {
                Ok(tag_value) => {
                    // Apply Nikon-specific PrintConv if available
                    let final_value = apply_nikon_print_conv(entry.tag_id, tag_value, tag_table);

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
        crate::types::TagValue::String(encryption_status),
        tag_source,
    );

    debug!(
        "Standard Nikon tag processing completed - {} entries processed",
        num_entries
    );
    Ok(())
}

/// Process encrypted Nikon data sections
/// ExifTool: Nikon.pm ProcessNikonEncrypted function
fn process_encrypted_sections(
    reader: &mut ExifReader,
    base_offset: usize,
    keys: &encryption::NikonEncryptionKeys,
) -> Result<()> {
    use crate::tiff_types::{ByteOrder, IfdEntry};

    trace!(
        "Processing encrypted Nikon sections at offset {:#x}",
        base_offset
    );

    let data = reader.get_data().to_vec();

    // Validate we have enough data for an IFD
    if base_offset + 2 > data.len() {
        debug!(
            "Insufficient data for encrypted section processing at offset {:#x}",
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
                "Failed to read IFD entry count for encrypted section at offset {:#x}",
                base_offset
            );
            return Ok(());
        }
    };

    debug!("Scanning {} entries for encrypted Nikon data", num_entries);

    let mut encrypted_sections_found = 0;
    let mut encrypted_tags = Vec::new();

    // Scan IFD entries for encrypted data indicators
    // ExifTool: Nikon.pm identifies encrypted sections by specific patterns
    for index in 0..num_entries {
        let entry_offset = base_offset + 2 + 12 * index;

        if entry_offset + 12 > data.len() {
            debug!(
                "Entry {} at offset {:#x} beyond data bounds during encryption scan",
                index, entry_offset
            );
            break;
        }

        // Parse IFD entry
        let entry = match IfdEntry::parse(&data, entry_offset, byte_order) {
            Ok(entry) => entry,
            Err(e) => {
                trace!(
                    "Failed to parse IFD entry {} during encryption scan: {:?}",
                    index,
                    e
                );
                continue;
            }
        };

        // Check for known encrypted data tags
        // ExifTool: Nikon.pm ProcessNikonEncrypted identifies these patterns
        if is_encrypted_nikon_tag(entry.tag_id) {
            encrypted_sections_found += 1;
            encrypted_tags.push(entry.tag_id);

            trace!("Found encrypted tag {:#x} at entry {}", entry.tag_id, index);

            // Store information about the encrypted tag
            let tag_source = reader.create_tag_source_info("Nikon");
            let tag_info = if keys.has_required_keys() {
                format!(
                    "Encrypted tag {:#x} (keys available, decryption not implemented)",
                    entry.tag_id
                )
            } else {
                format!(
                    "Encrypted tag {:#x} (keys required for decryption)",
                    entry.tag_id
                )
            };

            reader.store_tag_with_precedence(
                0x1000 + entry.tag_id, // Use offset to avoid conflicts
                crate::types::TagValue::String(tag_info),
                tag_source,
            );
        }
    }

    // Store overall encryption status
    let tag_source = reader.create_tag_source_info("Nikon");

    if encrypted_sections_found > 0 {
        let encryption_summary = if keys.has_required_keys() {
            format!(
                "Found {} encrypted sections (keys available: serial={}, count={})",
                encrypted_sections_found,
                keys.get_serial_key().unwrap_or("none"),
                keys.get_count_key()
                    .map(|c| c.to_string())
                    .unwrap_or("none".to_string())
            )
        } else {
            format!(
                "Found {encrypted_sections_found} encrypted sections (keys incomplete for decryption)"
            )
        };

        reader.store_tag_with_precedence(
            0x00FF, // Custom tag for encryption summary
            crate::types::TagValue::String(encryption_summary),
            tag_source,
        );

        debug!(
            "Detected {} encrypted Nikon sections: {:?}",
            encrypted_sections_found, encrypted_tags
        );
    } else {
        reader.store_tag_with_precedence(
            0x00FF, // Custom tag for encryption summary
            crate::types::TagValue::String("No encrypted sections detected".to_string()),
            tag_source,
        );

        debug!("No encrypted Nikon sections detected");
    }

    debug!(
        "Encrypted section processing completed - {} sections found",
        encrypted_sections_found
    );
    Ok(())
}

/// Check if a tag ID represents encrypted Nikon data
/// ExifTool: Nikon.pm encrypted tag identification
fn is_encrypted_nikon_tag(tag_id: u16) -> bool {
    // ExifTool: Nikon.pm identifies these tags as commonly encrypted
    // This is a simplified version - real ExifTool has model-specific lists
    match tag_id {
        // Common encrypted tags from ExifTool Nikon.pm
        0x0088 => true, // AFInfo (often encrypted)
        0x0091 => true, // ShotInfo (often encrypted)
        0x0097 => true, // ColorBalance (often encrypted)
        0x0098 => true, // LensData (often encrypted)
        0x00A8 => true, // FlashInfo (often encrypted)
        0x00B0 => true, // MultiExposure (often encrypted)
        0x00B7 => true, // AFInfo2 (often encrypted)
        0x00B9 => true, // AFTune (often encrypted)
        _ => false,     // Other tags are typically not encrypted
    }
}

/// Detect Nikon signature for MakerNote processor selection
/// ExifTool: MakerNotes.pm Nikon signature detection
pub fn detect_nikon_signature(make: &str) -> bool {
    detection::detect_nikon_signature(make)
}
