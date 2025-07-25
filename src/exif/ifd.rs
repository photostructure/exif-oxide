//! IFD (Image File Directory) parsing logic
//!
//! This module contains the core IFD parsing functionality for processing
//! EXIF directory structures, including subdirectory recursion management.
//!
//! ExifTool Reference: lib/Image/ExifTool/Exif.pm IFD processing

use crate::implementations::olympus;
use crate::tiff_types::{ByteOrder, IfdEntry, TiffFormat};
use crate::types::{DirectoryInfo, ExifError, Result, TagValue};
use crate::value_extraction;
use tracing::{debug, trace, warn};

use super::ExifReader;

impl ExifReader {
    /// Process MakerNotes with manufacturer signature detection and offset adjustment
    /// ExifTool: MakerNotes.pm manufacturer-specific processing
    fn process_maker_notes_with_signature_detection(
        &mut self,
        entry: &IfdEntry,
        _byte_order: ByteOrder,
        ifd_name: &str,
    ) -> Result<()> {
        use crate::implementations::olympus::{detect_olympus_signature, is_olympus_makernote};

        let offset = entry.value_or_offset as usize;
        let size = entry.count as usize;

        // Store the original MakerNotes offset for subdirectory calculations
        // ExifTool: Subdirectory offsets are relative to this position
        self.maker_notes_original_offset = Some(offset);

        // Extract manufacturer from Make tag for signature detection
        let make = self
            .extracted_tags
            .get(&0x010F) // Make tag
            .and_then(|v| v.as_string())
            .unwrap_or_default();

        debug!(
            "Processing UNDEFINED subdirectory tag {:#x} (MakerNotes) from {}: offset={:#x}, size={}, make='{}'",
            entry.tag_id, ifd_name, offset, size, make
        );

        // Get MakerNotes data for signature detection
        if offset + size > self.data.len() {
            return Err(ExifError::ParseError(format!(
                "MakerNotes data at offset {:#x} + size {} exceeds file size {}",
                offset,
                size,
                self.data.len()
            )));
        }

        let maker_notes_data = self.data[offset..offset + size].to_vec();
        let mut adjusted_offset = offset;
        let mut _adjusted_base = 0i64;

        // Detect Olympus signature and apply offset adjustments
        if let Some(signature) = detect_olympus_signature(make, &maker_notes_data) {
            let data_offset = signature.data_offset();
            let base_offset = signature.base_offset();

            adjusted_offset = offset + data_offset;
            _adjusted_base = base_offset as i64;

            debug!(
                "Detected Olympus signature: {:?}, data_offset: {}, base_offset: {}, adjusted_offset: {:#x}",
                signature, data_offset, base_offset, adjusted_offset
            );
        } else if is_olympus_makernote(make) {
            // Fallback for Olympus cameras without proper signature
            debug!("Olympus camera detected via Make field but no signature found, using default offset");
        }

        // Validate adjusted offset
        if adjusted_offset >= self.data.len() {
            return Err(ExifError::ParseError(format!(
                "Adjusted MakerNotes offset {:#x} exceeds file size {}",
                adjusted_offset,
                self.data.len()
            )));
        }

        // Process MakerNotes as subdirectory with adjusted offset
        let tag_name = "MakerNotes";
        debug!(
            "Processing SubDirectory: {} -> {} at offset {:#x}",
            format!("Tag_{:x}", entry.tag_id),
            tag_name,
            adjusted_offset
        );

        debug!(
            "About to call process_subdirectory_tag for MakerNotes with make='{}'",
            make
        );
        self.process_subdirectory_tag(entry.tag_id, adjusted_offset as u32, tag_name, Some(size))?;
        debug!("Completed process_subdirectory_tag for MakerNotes");

        // Don't store raw binary MakerNotes data - the subdirectory processing should have extracted the manufacturer-specific tags
        // ExifTool: MakerNotes are processed as subdirectories, not stored as raw binary data

        Ok(())
    }
    /// Process a subdirectory with recursion prevention
    /// ExifTool: ProcessDirectory with PROCESSED tracking
    pub(crate) fn process_subdirectory(&mut self, dir_info: &DirectoryInfo) -> Result<()> {
        debug!(
            "process_subdirectory called for directory: {}",
            dir_info.name
        );

        // Calculate unique address for recursion prevention
        // ExifTool: $addr = $$dirInfo{DirStart} + $$dirInfo{DataPos} + ($$dirInfo{Base}||0) + $$self{BASE}
        let addr = dir_info.dir_start as u64 + dir_info.data_pos + dir_info.base + self.base;

        // Check for infinite loops (ExifTool PROCESSED tracking)
        if let Some(prev_dir) = self.processed.get(&addr) {
            if !dir_info.allow_reprocess {
                warn!(
                    "Circular reference detected: {} already processed at address {:#x} (was {})",
                    dir_info.name, addr, prev_dir
                );
                self.warnings.push(format!(
                    "Circular reference detected: {} already processed at address {:#x} (was {})",
                    dir_info.name, addr, prev_dir
                ));
                return Ok(()); // Graceful degradation, not fatal
            }
        }

        trace!(
            "Entering directory {} at address {:#x}, path: {}",
            dir_info.name,
            addr,
            self.path.join("/")
        );

        // Enter subdirectory context
        self.path.push(dir_info.name.clone());
        self.processed.insert(addr, dir_info.name.clone());

        // Select and dispatch to appropriate processor
        // ExifTool: ProcessDirectory with PROCESS_PROC dispatch
        let processor = self.select_processor(&dir_info.name, None);
        let result = self.dispatch_processor(&processor, dir_info);

        // Exit subdirectory context
        self.path.pop();

        result
    }

    /// Parse a single IFD (Image File Directory)
    /// ExifTool: lib/Image/ExifTool/Exif.pm:6232-6342 IFD processing
    pub(crate) fn parse_ifd(&mut self, ifd_offset: usize, ifd_name: &str) -> Result<()> {
        if ifd_offset + 2 > self.data.len() {
            return Err(ExifError::ParseError(format!(
                "IFD offset {ifd_offset:#x} beyond data bounds"
            )));
        }

        let byte_order = self.header.as_ref().unwrap().byte_order;

        // Read number of entries (first 2 bytes of IFD)
        // ExifTool: lib/Image/ExifTool/Exif.pm:6235 numEntries
        let num_entries = byte_order.read_u16(&self.data, ifd_offset)? as usize;

        debug!(
            "IFD {} at offset {:#x} has {} entries",
            ifd_name, ifd_offset, num_entries
        );

        // Calculate directory size: 2 bytes (count) + 12 bytes per entry + 4 bytes (next IFD)
        // ExifTool: lib/Image/ExifTool/Exif.pm:6236-6237 dirSize calculation
        let dir_size = 2 + 12 * num_entries + 4;
        let dir_end = ifd_offset + dir_size;

        if dir_end > self.data.len() {
            // Graceful degradation - ExifTool continues parsing what it can
            // ExifTool: lib/Image/ExifTool/Exif.pm:6238-6247 short directory handling
            self.warnings.push(format!(
                "Short directory size for {ifd_name} (missing {} bytes)",
                dir_end - self.data.len()
            ));
        }

        // Process each IFD entry
        // ExifTool: lib/Image/ExifTool/Exif.pm:6342-6349 entry loop
        for index in 0..num_entries {
            let entry_offset = ifd_offset + 2 + 12 * index;

            if entry_offset + 12 > self.data.len() {
                debug!(
                    "IFD {} entry {} at offset {:#x} beyond data bounds (data len: {})",
                    ifd_name,
                    index,
                    entry_offset,
                    self.data.len()
                );
                self.warnings
                    .push(format!("IFD entry {index} beyond data bounds"));
                break; // Graceful degradation
            }

            match self.parse_ifd_entry(entry_offset, byte_order, ifd_name, index, ifd_offset) {
                Ok(()) => {
                    debug!("Successfully processed {} entry {}", ifd_name, index);
                } // Successfully parsed
                Err(e) => {
                    // Graceful degradation - log warning but continue
                    // ExifTool: lib/Image/ExifTool/Exif.pm:6360-6365 error handling
                    debug!("Error parsing {} entry {}: {}", ifd_name, index, e);
                    warn!(
                        ifd_name = %ifd_name,
                        entry_index = index,
                        error = %e,
                        "Failed to parse IFD entry, continuing with graceful degradation"
                    );
                    self.warnings
                        .push(format!("Error parsing {ifd_name} entry {index}: {e}"));
                }
            }
        }

        Ok(())
    }

    /// Parse a single IFD entry and extract tag value
    /// ExifTool: lib/Image/ExifTool/Exif.pm:6347-6570 entry processing
    fn parse_ifd_entry(
        &mut self,
        entry_offset: usize,
        byte_order: ByteOrder,
        ifd_name: &str,
        _index: usize,
        ifd_offset: usize,
    ) -> Result<()> {
        // Check if we're processing Olympus MakerNotes for FixFormat support
        // ExifTool: lib/Image/ExifTool/Olympus.pm dual-path processing
        let is_olympus_makernotes = self.is_olympus_makernotes_context(ifd_name);
        debug!(
            "IFD {} olympus context: {}",
            ifd_name, is_olympus_makernotes
        );

        // Parse 12-byte IFD entry structure with Olympus context
        let entry = if is_olympus_makernotes {
            IfdEntry::parse_with_context(&self.data, entry_offset, byte_order, true)?
        } else {
            IfdEntry::parse(&self.data, entry_offset, byte_order)?
        };

        // Debug: Log all tag IDs being processed
        // debug!(
        //     "Processing tag {:#x} ({}) from {} (format: {:?}, count: {})",
        //     entry.tag_id, entry.tag_id, ifd_name, entry.format, entry.count
        // );

        // Tag definitions are now looked up directly in tag kits during conversion
        // No need to pass around tag_def anymore

        // Milestone 3: Support for common numeric formats with PrintConv
        // ExifTool: lib/Image/ExifTool/Exif.pm:6390-6570 value extraction
        match entry.format {
            TiffFormat::Ascii => {
                let value = value_extraction::extract_ascii_value(&self.data, &entry, byte_order)?;
                // debug!("ASCII tag {:#x} extracted value: {:?} (length: {})", entry.tag_id, value, value.len());
                if !value.is_empty() {
                    let tag_value = TagValue::String(value);
                    let (final_value, _print) =
                        self.apply_conversions(&tag_value, entry.tag_id, ifd_name);
                    trace!(
                        "Extracted ASCII tag {:#x} from {}: {:?}",
                        entry.tag_id,
                        ifd_name,
                        final_value
                    );
                    let source_info = self.create_tag_source_info(ifd_name);
                    self.store_tag_with_precedence(entry.tag_id, final_value, source_info);
                }
            }
            TiffFormat::Byte => {
                // Handle both single byte and byte arrays
                let tag_value = if entry.count == 1 {
                    let value = value_extraction::extract_byte_value(&self.data, &entry)?;
                    TagValue::U8(value)
                } else {
                    // Handle byte arrays (count > 1)
                    let values = value_extraction::extract_byte_array_value(&self.data, &entry)?;
                    TagValue::U8Array(values)
                };

                let (final_value, _print) =
                    self.apply_conversions(&tag_value, entry.tag_id, ifd_name);
                trace!(
                    "Extracted BYTE tag {:#x} from {} (count: {}): {:?}",
                    entry.tag_id,
                    ifd_name,
                    entry.count,
                    final_value
                );
                let source_info = self.create_tag_source_info(ifd_name);
                self.store_tag_with_precedence(entry.tag_id, final_value, source_info);
            }
            TiffFormat::Short => {
                // Handle both single and array SHORT values
                // Critical for Canon binary data (CameraSettings, ShotInfo, etc.)
                if entry.count == 1 {
                    let value =
                        value_extraction::extract_short_value(&self.data, &entry, byte_order)?;
                    let tag_value = TagValue::U16(value);
                    let (final_value, _print) =
                        self.apply_conversions(&tag_value, entry.tag_id, ifd_name);

                    trace!(
                        "Extracted SHORT tag {:#x} from {}: {:?}",
                        entry.tag_id,
                        ifd_name,
                        final_value
                    );

                    let source_info = self.create_tag_source_info(ifd_name);
                    self.store_tag_with_precedence(entry.tag_id, final_value.clone(), source_info);

                    // Check for NEF -> NRW conversion
                    // ExifTool Exif.pm:1139-1140 - recognize NRW from JPEG-compressed thumbnail in IFD0
                    if entry.tag_id == 0x0103 && ifd_name == "IFD0" {
                        if let TagValue::U16(compression) = final_value {
                            if compression == 6 && self.original_file_type.as_deref() == Some("NEF")
                            {
                                // Override file type from NEF to NRW
                                debug!("Detected NRW format: IFD0 Compression=6 in NEF file");
                                self.overridden_file_type = Some("NRW".to_string());
                            }
                        }
                    }
                } else {
                    // Multiple SHORT values - use array extraction
                    let values = value_extraction::extract_short_array_value(
                        &self.data, &entry, byte_order,
                    )?;
                    let tag_value = TagValue::U16Array(values);
                    let (final_value, _print) =
                        self.apply_conversions(&tag_value, entry.tag_id, ifd_name);

                    trace!(
                        "Extracted SHORT array tag {:#x} from {} with {} values",
                        entry.tag_id,
                        ifd_name,
                        entry.count
                    );

                    let source_info = self.create_tag_source_info(ifd_name);
                    self.store_tag_with_precedence(entry.tag_id, final_value, source_info);
                }
            }
            TiffFormat::Long => {
                if entry.count == 1 {
                    // Single LONG value - use existing logic
                    let value =
                        value_extraction::extract_long_value(&self.data, &entry, byte_order)?;
                    let tag_value = TagValue::U32(value);

                    // Milestone 5: Check for SubDirectory tags (ExifIFD, GPS, etc.)
                    // ExifTool: SubDirectory processing for nested IFDs
                    if self.is_subdirectory_tag(entry.tag_id) {
                        let tag_name = self.get_tag_name(entry.tag_id, ifd_name);
                        self.process_subdirectory_tag(entry.tag_id, value, &tag_name, None)?;
                    }

                    let (final_value, _print) =
                        self.apply_conversions(&tag_value, entry.tag_id, ifd_name);
                    trace!(
                        "Extracted LONG tag {:#x} from {}: {:?}",
                        entry.tag_id,
                        ifd_name,
                        final_value
                    );
                    let source_info = self.create_tag_source_info(ifd_name);
                    self.store_tag_with_precedence(entry.tag_id, final_value, source_info);
                } else {
                    // Multiple LONG values - extract as array
                    debug!(
                        "Extracting LONG array tag {:#x} from {} with count {}",
                        entry.tag_id, ifd_name, entry.count
                    );

                    let values =
                        value_extraction::extract_long_array(&self.data, &entry, byte_order)?;
                    let tag_value = TagValue::U32Array(values);

                    // Check for SubDirectory tags that might need array processing
                    // ExifTool: Some subdirectory tags contain arrays of data (like ColorData)
                    if self.is_subdirectory_tag(entry.tag_id) {
                        let tag_name = self.get_tag_name(entry.tag_id, ifd_name);
                        // For subdirectory processing, pass the raw data offset and size
                        // ExifTool: ColorData arrays are processed as byte sequences at their data location
                        debug!(
                            "Processing LONG array subdirectory tag {:#x} ({}): {} values at offset {:#x}",
                            entry.tag_id, tag_name, entry.count, entry.value_or_offset
                        );
                        let size = (entry.count as usize) * 4; // 4 bytes per LONG
                        self.process_subdirectory_tag(
                            entry.tag_id,
                            entry.value_or_offset,
                            &tag_name,
                            Some(size),
                        )?;
                    }

                    let (final_value, _print) =
                        self.apply_conversions(&tag_value, entry.tag_id, ifd_name);
                    trace!(
                        "Extracted LONG array tag {:#x} from {} with {} values: {:?}",
                        entry.tag_id,
                        ifd_name,
                        entry.count,
                        final_value
                    );
                    let source_info = self.create_tag_source_info(ifd_name);
                    self.store_tag_with_precedence(entry.tag_id, final_value, source_info);
                }
            }
            TiffFormat::Rational => {
                // Milestone 6: RATIONAL format support (format 5)
                // ExifTool: 2x uint32 values representing numerator/denominator
                let value =
                    value_extraction::extract_rational_value(&self.data, &entry, byte_order)?;
                let (final_value, _print) = self.apply_conversions(&value, entry.tag_id, ifd_name);
                trace!(
                    "Extracted RATIONAL tag {:#x} from {}: {:?}",
                    entry.tag_id,
                    ifd_name,
                    final_value
                );
                let source_info = self.create_tag_source_info(ifd_name);
                self.store_tag_with_precedence(entry.tag_id, final_value, source_info);
            }
            TiffFormat::SRational => {
                // Milestone 6: SRATIONAL format support (format 10)
                // ExifTool: 2x int32 values representing numerator/denominator
                let value =
                    value_extraction::extract_srational_value(&self.data, &entry, byte_order)?;
                let (final_value, _print) = self.apply_conversions(&value, entry.tag_id, ifd_name);
                trace!(
                    "Extracted SRATIONAL tag {:#x} from {}: {:?}",
                    entry.tag_id,
                    ifd_name,
                    final_value
                );
                let source_info = self.create_tag_source_info(ifd_name);
                self.store_tag_with_precedence(entry.tag_id, final_value, source_info);
            }
            TiffFormat::Undefined => {
                // UNDEFINED format - can contain various data types including subdirectories
                // ExifTool: lib/Image/ExifTool/Exif.pm undefined data handling
                debug!(
                    "Processing UNDEFINED tag {:#x} from {}",
                    entry.tag_id, ifd_name
                );

                // Check if this is a subdirectory tag (like MakerNotes)
                debug!(
                    "Checking if UNDEFINED tag {:#x} is a subdirectory tag",
                    entry.tag_id
                );
                if self.is_subdirectory_tag(entry.tag_id) {
                    debug!("UNDEFINED tag {:#x} is a subdirectory tag", entry.tag_id);

                    // Special handling for MakerNotes - detect manufacturer signature and adjust offset
                    if entry.tag_id == 0x927C {
                        // MakerNotes
                        return self.process_maker_notes_with_signature_detection(
                            &entry, byte_order, ifd_name,
                        );
                    }

                    // For other subdirectory UNDEFINED tags, the data starts at the offset
                    // ExifTool: MakerNotes and other subdirectories stored as UNDEFINED
                    let offset = entry.value_or_offset as usize;
                    let size = entry.count as usize;

                    // Get tag name from definition or use fallback for known subdirectory tags
                    let tag_name = match entry.tag_id {
                        0x927C => Some("MakerNotes".to_string()),
                        _ => {
                            // Try to get name from tag kit
                            let name = self.get_tag_name(entry.tag_id, ifd_name);
                            if name.starts_with("Tag_") {
                                debug!(
                                    "UNDEFINED subdirectory tag {:#x} has no tag definition",
                                    entry.tag_id
                                );
                                None // Skip unknown subdirectory tags
                            } else {
                                Some(name)
                            }
                        }
                    };

                    if let Some(name) = tag_name {
                        debug!(
                            "Processing UNDEFINED subdirectory tag {:#x} ({}) from {}: offset={:#x}, size={}",
                            entry.tag_id,
                            name,
                            ifd_name,
                            offset,
                            size
                        );
                        self.process_subdirectory_tag(
                            entry.tag_id,
                            offset as u32,
                            &name,
                            Some(size),
                        )?;
                    }
                } else {
                    // Regular UNDEFINED data - extract the binary data
                    let tag_name = self.get_tag_name(entry.tag_id, ifd_name);

                    // Special handling for JpgFromRaw tags (Panasonic RW2 embedded JPEG preview)
                    if entry.tag_id == 0x002e || entry.tag_id == 0x0127 {
                        // JpgFromRaw and JpgFromRaw2 - extract the embedded JPEG data
                        // ExifTool: PanasonicRaw.pm tags 0x002e and 0x0127 contain JPEG data
                        if let Ok(binary_data) =
                            value_extraction::extract_byte_array_value(&self.data, &entry)
                        {
                            let tag_value = TagValue::Binary(binary_data);
                            let source_info = self.create_tag_source_info(ifd_name);
                            self.store_tag_with_precedence(entry.tag_id, tag_value, source_info);
                            debug!(
                                "Extracted UNDEFINED tag {:#x} ({}) as binary data ({} bytes)",
                                entry.tag_id, tag_name, entry.count
                            );
                        } else {
                            debug!(
                                "Failed to extract binary data for UNDEFINED tag {:#x} ({})",
                                entry.tag_id, tag_name
                            );
                        }
                    } else {
                        // Other UNDEFINED data - store as raw bytes for now
                        // TODO: Implement specific UNDEFINED tag processing as needed
                        debug!(
                            "UNDEFINED tag {:#x} ({}) not yet implemented (format 7, {} bytes)",
                            entry.tag_id, tag_name, entry.count
                        );
                    }
                }
            }
            TiffFormat::Ifd => {
                // IFD format - subdirectory pointer (typically from FixFormat conversion)
                // ExifTool: Olympus FixFormat converts invalid formats to IFD for subdirectory processing
                debug!(
                    "Processing IFD tag {:#x} from {} (format: {:?}, count: {})",
                    entry.tag_id, ifd_name, entry.format, entry.count
                );

                // For IFD format tags, extract the offset value like LONG format
                let value = value_extraction::extract_long_value(&self.data, &entry, byte_order)?;
                let tag_value = TagValue::U32(value);

                // Check if this is a subdirectory tag
                if self.is_subdirectory_tag(entry.tag_id) {
                    debug!(
                        "IFD tag {:#x} is a subdirectory tag, processing as subdirectory",
                        entry.tag_id
                    );
                    let tag_name = self.get_tag_name(entry.tag_id, ifd_name);

                    // CRITICAL: Olympus SubDirectory Offset Calculation
                    // ================================================
                    // When processing subdirectories within MakerNotes that have manufacturer signatures,
                    // the subdirectory offsets are relative to the ORIGINAL MakerNotes position in the file,
                    // NOT the adjusted position after the signature header.
                    //
                    // Example for Olympus ORF:
                    // - MakerNotes tag at file offset: 0xdf4
                    // - Olympus signature ("OLYMPUS\0"): 12 bytes
                    // - MakerNotes TIFF data starts at: 0xdf4 + 12 = 0xe00
                    // - Equipment subdirectory offset in IFD: 0x72
                    // - WRONG: 0xe00 + 0x72 = 0xe72 (points to middle of data)
                    // - RIGHT: 0xdf4 + 0x72 = 0xe66 (points to IFD start)
                    //
                    // ExifTool: lib/Image/ExifTool/Olympus.pm lines 1157-1168
                    // "Olympus really screwed up the format... the count is 2 bytes short"
                    let subdirectory_offset = if ifd_name == "MakerNotes"
                        && self.maker_notes_original_offset.is_some()
                    {
                        // Use the original MakerNotes position for offset calculation
                        let original_offset = self.maker_notes_original_offset.unwrap();
                        debug!(
                            "Adjusting subdirectory offset using original MakerNotes position: {:#x} + {:#x} = {:#x}",
                            original_offset, value, original_offset + value as usize
                        );
                        (original_offset + value as usize) as u32
                    } else if ifd_name == "MakerNotes" {
                        // Fallback if we don't have the original offset
                        debug!(
                            "Adjusting subdirectory offset for MakerNotes context: {:#x} + {:#x} = {:#x}",
                            ifd_offset, value, ifd_offset + value as usize
                        );
                        (ifd_offset + value as usize) as u32
                    } else {
                        value
                    };

                    self.process_subdirectory_tag(
                        entry.tag_id,
                        subdirectory_offset,
                        &tag_name,
                        None,
                    )?;
                } else {
                    debug!(
                        "IFD tag {:#x} is not a subdirectory tag, storing as regular tag",
                        entry.tag_id
                    );
                }

                // Also store the tag value for completeness
                let (final_value, _print) =
                    self.apply_conversions(&tag_value, entry.tag_id, ifd_name);
                let source_info = self.create_tag_source_info(ifd_name);
                self.store_tag_with_precedence(entry.tag_id, final_value, source_info);
            }
            _ => {
                // For other formats, store raw value for now
                // Future milestones will implement additional formats
                let tag_name = self.get_tag_name(entry.tag_id, ifd_name);
                self.warnings.push(format!(
                    "Unimplemented format {:?} for tag {} ({})",
                    entry.format, entry.tag_id, tag_name
                ));
            }
        }

        Ok(())
    }

    /// Process standard EXIF IFD (renamed from parse_ifd)
    /// ExifTool: ProcessExif function for standard IFD processing
    pub fn process_exif_ifd(&mut self, ifd_offset: usize, ifd_name: &str) -> Result<()> {
        // This is the existing parse_ifd logic, renamed for clarity
        self.parse_ifd(ifd_offset, ifd_name)
    }

    /// Get tag name from tag kits
    /// ExifTool: Uses format-specific tag tables with conditional logic
    fn get_tag_name(&self, tag_id: u16, ifd_name: &str) -> String {
        use crate::generated::Exif_pm::tag_kit;
        use crate::generated::GPS_pm::tag_kit as gps_tag_kit;

        // For GPS IFD, check GPS tag kit first to avoid conflicts
        // (e.g., tag 0x0002 is GPSLatitude in GPS IFD, InteropVersion in InteropIFD)
        if ifd_name == "GPS" {
            if let Some(tag_def) = gps_tag_kit::GPS_PM_TAG_KITS.get(&(tag_id as u32)) {
                return tag_def.name.to_string();
            }
        }

        // Check EXIF tag kit for other IFDs
        if let Some(tag_def) = tag_kit::EXIF_PM_TAG_KITS.get(&(tag_id as u32)) {
            return tag_def.name.to_string();
        }

        // Default name if not found
        format!("Tag_{:04X}", tag_id)
    }

    /// Try resolving tag using conditional tag resolution with full entry context
    /// TODO: Re-enable when conditional tags are generated
    #[allow(dead_code)]
    fn try_conditional_tag_resolution_with_entry(
        &self,
        _entry: &crate::tiff_types::IfdEntry,
    ) -> Option<crate::generated::Canon_pm::main_conditional_tags::ResolvedTag> {
        // TODO: Generate conditional tags first
        // let make = self.extracted_tags.get(&0x010F)?.as_string()?;
        // if !make.to_lowercase().contains("canon") {
        //     return None;
        // }
        // let context = self.build_conditional_context_with_entry(entry)?;
        // let conditional_tags = crate::stubs::CanonConditionalTags::new();
        // conditional_tags.resolve_tag(&entry.tag_id.to_string(), &context)
        None
    }

    /// Build ConditionalContext from current EXIF parsing state with full entry context
    /// TODO: Re-enable when conditional tags are generated
    #[allow(dead_code)]
    fn build_conditional_context_with_entry(
        &self,
        _entry: &crate::tiff_types::IfdEntry,
    ) -> Option<()> {
        // Changed return type temporarily
        // TODO: Generate conditional context first
        None
    }

    /// Check if we're currently processing Olympus MakerNotes
    /// ExifTool: lib/Image/ExifTool/Olympus.pm FixFormat processing context
    fn is_olympus_makernotes_context(&self, ifd_name: &str) -> bool {
        // Check if the IFD name indicates Olympus MakerNotes
        if ifd_name.contains("MakerNotes") || ifd_name.starts_with("Olympus") {
            // Check if the Make field indicates this is an Olympus camera
            if let Some(make_tag) = self.extracted_tags.get(&0x010F) {
                if let Some(make_str) = make_tag.as_string() {
                    return olympus::is_olympus_makernote(make_str);
                }
            }
        }
        false
    }
}
