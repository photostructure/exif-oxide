//! IFD (Image File Directory) parsing logic
//!
//! This module contains the core IFD parsing functionality for processing
//! EXIF directory structures, including subdirectory recursion management.
//!
//! ExifTool Reference: lib/Image/ExifTool/Exif.pm IFD processing

use crate::generated::TAG_BY_ID;
use crate::tiff_types::{ByteOrder, IfdEntry, TiffFormat};
use crate::types::{DirectoryInfo, ExifError, Result, TagValue};
use crate::value_extraction;
use tracing::{debug, trace, warn};

use super::ExifReader;

impl ExifReader {
    /// Process a subdirectory with recursion prevention
    /// ExifTool: ProcessDirectory with PROCESSED tracking
    pub(crate) fn process_subdirectory(&mut self, dir_info: &DirectoryInfo) -> Result<()> {
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

            match self.parse_ifd_entry(entry_offset, byte_order, ifd_name, index) {
                Ok(()) => {
                    debug!("Successfully processed {} entry {}", ifd_name, index);
                } // Successfully parsed
                Err(e) => {
                    // Graceful degradation - log warning but continue
                    // ExifTool: lib/Image/ExifTool/Exif.pm:6360-6365 error handling
                    debug!("Error parsing {} entry {}: {}", ifd_name, index, e);
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
    ) -> Result<()> {
        // Parse 12-byte IFD entry structure
        let entry = IfdEntry::parse(&self.data, entry_offset, byte_order)?;

        // Look up tag definition in appropriate table based on IFD type
        // ExifTool: Different IFDs use different tag tables
        let tag_def = TAG_BY_ID.get(&(entry.tag_id as u32));

        // Milestone 3: Support for common numeric formats with PrintConv
        // ExifTool: lib/Image/ExifTool/Exif.pm:6390-6570 value extraction
        match entry.format {
            TiffFormat::Ascii => {
                let value = value_extraction::extract_ascii_value(&self.data, &entry, byte_order)?;
                if !value.is_empty() {
                    let tag_value = TagValue::String(value);
                    let (final_value, _print) =
                        self.apply_conversions(&tag_value, tag_def.copied());
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
                let value = value_extraction::extract_byte_value(&self.data, &entry)?;
                let tag_value = TagValue::U8(value);
                let (final_value, _print) = self.apply_conversions(&tag_value, tag_def.copied());
                trace!(
                    "Extracted BYTE tag {:#x} from {}: {:?}",
                    entry.tag_id,
                    ifd_name,
                    final_value
                );
                let source_info = self.create_tag_source_info(ifd_name);
                self.store_tag_with_precedence(entry.tag_id, final_value, source_info);
            }
            TiffFormat::Short => {
                let value = value_extraction::extract_short_value(&self.data, &entry, byte_order)?;
                let tag_value = TagValue::U16(value);
                let (final_value, _print) = self.apply_conversions(&tag_value, tag_def.copied());

                trace!(
                    "Extracted SHORT tag {:#x} from {}: {:?}",
                    entry.tag_id,
                    ifd_name,
                    final_value
                );

                let source_info = self.create_tag_source_info(ifd_name);
                self.store_tag_with_precedence(entry.tag_id, final_value, source_info);
            }
            TiffFormat::Long => {
                let value = value_extraction::extract_long_value(&self.data, &entry, byte_order)?;
                let tag_value = TagValue::U32(value);

                // Milestone 5: Check for SubDirectory tags (ExifIFD, GPS, etc.)
                // ExifTool: SubDirectory processing for nested IFDs
                if let Some(tag_def) = tag_def {
                    if self.is_subdirectory_tag(entry.tag_id) {
                        self.process_subdirectory_tag(entry.tag_id, value, tag_def.name, None)?;
                    }
                }

                let (final_value, _print) = self.apply_conversions(&tag_value, tag_def.copied());
                trace!(
                    "Extracted LONG tag {:#x} from {}: {:?}",
                    entry.tag_id,
                    ifd_name,
                    final_value
                );
                let source_info = self.create_tag_source_info(ifd_name);
                self.store_tag_with_precedence(entry.tag_id, final_value, source_info);
            }
            TiffFormat::Rational => {
                // Milestone 6: RATIONAL format support (format 5)
                // ExifTool: 2x uint32 values representing numerator/denominator
                let value =
                    value_extraction::extract_rational_value(&self.data, &entry, byte_order)?;
                let (final_value, _print) = self.apply_conversions(&value, tag_def.copied());
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
                let (final_value, _print) = self.apply_conversions(&value, tag_def.copied());
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
                    // For subdirectory UNDEFINED tags, the data starts at the offset
                    // ExifTool: MakerNotes and other subdirectories stored as UNDEFINED
                    let offset = entry.value_or_offset as usize;
                    let size = entry.count as usize;

                    // Get tag name from definition or use fallback for known subdirectory tags
                    let tag_name = if let Some(tag_def) = tag_def {
                        Some(tag_def.name)
                    } else {
                        // Fallback names for known subdirectory tags without definitions
                        match entry.tag_id {
                            0x927C => Some("MakerNotes"),
                            _ => {
                                debug!("UNDEFINED subdirectory tag {:#x} has no tag definition and no fallback", entry.tag_id);
                                None // Skip unknown subdirectory tags
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
                            name,
                            Some(size),
                        )?;
                    }
                } else {
                    // Regular UNDEFINED data - store as raw bytes for now
                    // TODO: Implement specific UNDEFINED tag processing as needed
                    if let Some(tag_def) = tag_def {
                        debug!(
                            "UNDEFINED tag {:#x} ({}) not yet implemented (format 7, {} bytes)",
                            entry.tag_id, tag_def.name, entry.count
                        );
                    }
                }
            }
            _ => {
                // For other formats, store raw value for now
                // Future milestones will implement additional formats
                if let Some(tag_def) = tag_def {
                    self.warnings.push(format!(
                        "Unimplemented format {:?} for tag {} ({})",
                        entry.format, entry.tag_id, tag_def.name
                    ));
                }
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
}
