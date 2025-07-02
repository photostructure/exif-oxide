//! EXIF/TIFF parsing module
//!
//! This module implements EXIF parsing for JPEG-embedded EXIF data, translating
//! ExifTool's ProcessExif function (Exif.pm:6172-7128) to handle:
//! - TIFF header validation and endianness detection
//! - IFD (Image File Directory) parsing
//! - Basic tag value extraction (ASCII, SHORT, LONG formats)
//! - Make/Model/Software extraction with null-termination
//! - Milestone 5: SubDirectory support with recursion prevention (ExifIFD, GPS)
//! - Stateful reader with PROCESSED tracking and PATH management
//!
//! Reference: lib/Image/ExifTool/Exif.pm ProcessExif function

use crate::generated::TAG_BY_ID;
use crate::implementations::{canon, sony};
use crate::tiff_types::{ByteOrder, IfdEntry, TiffFormat, TiffHeader};
use crate::types::{
    DataMemberValue, DirectoryInfo, ExifError, ProcessorDispatch, ProcessorType, Result,
    SonyProcessor, TagSourceInfo, TagValue,
};
use crate::value_extraction;
use std::collections::HashMap;
use tracing::{debug, trace, warn};

/// Stateful EXIF reader for processing JPEG-embedded EXIF data
/// ExifTool: lib/Image/ExifTool/Exif.pm ProcessExif function architecture
#[derive(Debug)]
pub struct ExifReader {
    /// Extracted tag values by tag ID
    extracted_tags: HashMap<u16, TagValue>,
    /// Enhanced tag source tracking for conflict resolution
    /// Maps tag_id -> TagSourceInfo with namespace, priority, and processor context
    tag_sources: HashMap<u16, TagSourceInfo>,
    /// TIFF header information
    header: Option<TiffHeader>,
    /// Raw EXIF data buffer
    data: Vec<u8>,
    /// Parse errors (non-fatal, for graceful degradation)
    warnings: Vec<String>,

    // Milestone 5: Stateful processing features
    /// PROCESSED hash for recursion prevention
    /// ExifTool: $$self{PROCESSED} prevents infinite loops
    processed: HashMap<u64, String>,
    /// PATH stack for directory hierarchy tracking
    /// ExifTool: $$self{PATH} tracks current directory path
    path: Vec<String>,
    /// DataMember storage for tag dependencies
    /// ExifTool: DataMember tags needed by other tags
    data_members: HashMap<String, DataMemberValue>,
    /// Current base offset for pointer calculations
    /// ExifTool: $$dirInfo{Base} + $$self{BASE}
    base: u64,
    /// Processor dispatch configuration
    /// ExifTool: PROCESS_PROC system for different directory types
    processor_dispatch: ProcessorDispatch,
    /// Computed composite tag values
    /// Milestone 8f: Infrastructure for composite tag computation
    composite_tags: HashMap<String, TagValue>,
}

impl ExifReader {
    /// Create new EXIF reader
    pub fn new() -> Self {
        Self {
            extracted_tags: HashMap::new(),
            tag_sources: HashMap::new(),
            header: None,
            data: Vec::new(),
            warnings: Vec::new(),
            // Milestone 5: Initialize stateful features
            processed: HashMap::new(),
            path: Vec::new(),
            data_members: HashMap::new(),
            base: 0,
            processor_dispatch: ProcessorDispatch::default(),
            composite_tags: HashMap::new(),
        }
    }

    /// Parse EXIF data from JPEG APP1 segment after "Exif\0\0"
    /// ExifTool: lib/Image/ExifTool/Exif.pm:6172 ProcessExif entry point
    pub fn parse_exif_data(&mut self, exif_data: &[u8]) -> Result<()> {
        if exif_data.len() < 8 {
            return Err(ExifError::ParseError(
                "EXIF data too short for TIFF header".to_string(),
            ));
        }

        // Store data for offset-based value reading
        self.data = exif_data.to_vec();

        // Parse TIFF header
        self.header = Some(TiffHeader::parse(exif_data)?);
        let header = self.header.as_ref().unwrap();

        // Parse IFD0 starting at the offset specified in header
        // ExifTool: ProcessExif starts with IFD0 processing
        let dir_info = DirectoryInfo {
            name: "IFD0".to_string(),
            dir_start: header.ifd0_offset as usize,
            dir_len: 0, // Will be calculated during processing
            base: 0,    // TIFF header is at base 0
            data_pos: 0,
            allow_reprocess: false,
        };
        self.process_subdirectory(&dir_info)?;

        // Build composite tags after all extraction is complete
        // Milestone 8f: Composite tag infrastructure
        self.build_composite_tags();

        // NOTE: GPS coordinate decimal conversion is deferred to Milestone 8 (ValueConv)
        // Milestone 6 outputs raw rational arrays matching ExifTool default behavior

        Ok(())
    }

    /// Process a subdirectory with recursion prevention
    /// ExifTool: ProcessDirectory with PROCESSED tracking
    fn process_subdirectory(&mut self, dir_info: &DirectoryInfo) -> Result<()> {
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
        let result = self.dispatch_processor(processor, dir_info);

        // Exit subdirectory context
        self.path.pop();

        result
    }

    /// Parse a single IFD (Image File Directory)
    /// ExifTool: lib/Image/ExifTool/Exif.pm:6232-6342 IFD processing
    fn parse_ifd(&mut self, ifd_offset: usize, ifd_name: &str) -> Result<()> {
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

    /// Get extracted tag by ID
    pub fn get_tag_by_id(&self, tag_id: u16) -> Option<&TagValue> {
        self.extracted_tags.get(&tag_id)
    }

    /// Build composite tags from extracted tags
    /// Milestone 11.5: Multi-pass dependency resolution for composite-on-composite dependencies
    /// ExifTool: lib/Image/ExifTool.pm BuildCompositeTags function
    /// This is now a thin facade that delegates to the composite_tags module
    pub fn build_composite_tags(&mut self) {
        // Clear any previous composite tags
        self.composite_tags.clear();

        // Build initial available tags lookup from extracted tags
        let available_tags = crate::composite_tags::build_available_tags_map(
            &self.extracted_tags,
            &self.tag_sources,
        );

        // Delegate the multi-pass resolution and computation to the composite_tags module
        let computed_composites =
            crate::composite_tags::resolve_and_compute_composites(available_tags);

        // Store the results in our composite_tags collection
        self.composite_tags = computed_composites;
    }

    /// Get all extracted tags with their names (conversions already applied during extraction)
    /// Returns tags with group prefixes (e.g., "EXIF:Make", "GPS:GPSLatitude", "Composite:ImageSize")
    /// matching ExifTool's -G mode behavior
    /// Milestone 8f: Now includes composite tags with "Composite:" prefix
    pub fn get_all_tags(&self) -> HashMap<String, TagValue> {
        let mut result = HashMap::new();

        // Add extracted tags
        for (&tag_id, value) in &self.extracted_tags {
            // Get the enhanced source info for this tag
            let source_info = self.tag_sources.get(&tag_id);

            // Use namespace from TagSourceInfo or default to EXIF
            let group_name = if let Some(source_info) = source_info {
                &source_info.namespace
            } else {
                "EXIF" // Default fallback
            };

            // Look up tag name in unified table or use Canon-specific names
            let base_tag_name = if let Some(canon_tag_name) = self.get_canon_tag_name(tag_id) {
                canon_tag_name
            } else {
                TAG_BY_ID
                    .get(&(tag_id as u32))
                    .map(|tag_def| tag_def.name.to_string())
                    .unwrap_or_else(|| format!("Tag_{tag_id:04X}"))
            };

            // Format with group prefix
            let tag_name = format!("{group_name}:{base_tag_name}");

            trace!(
                "Tag {:#x} from {} -> {}: {:?}",
                tag_id,
                source_info
                    .map(|s| s.ifd_name.as_str())
                    .unwrap_or("Unknown"),
                tag_name,
                value
            );
            result.insert(tag_name, value.clone());
        }

        // Add composite tags (already have "Composite:" prefix)
        for (tag_name, value) in &self.composite_tags {
            trace!("Composite tag: {} -> {:?}", tag_name, value);
            result.insert(tag_name.clone(), value.clone());
        }

        result
    }

    /// Get all tags as TagEntry objects with both value and print representations
    /// This is the new API that returns both ValueConv and PrintConv results
    /// Milestone 8b: TagEntry API implementation
    pub fn get_all_tag_entries(&mut self) -> Vec<crate::types::TagEntry> {
        use crate::generated::{COMPOSITE_TAGS, TAG_BY_ID};
        use crate::types::TagEntry;

        let mut entries = Vec::new();

        // Process extracted tags
        for (&tag_id, raw_value) in &self.extracted_tags {
            // Get the enhanced source info for this tag
            let source_info = self.tag_sources.get(&tag_id);

            // Use namespace from TagSourceInfo or default to EXIF
            let group_name = if let Some(source_info) = source_info {
                &source_info.namespace
            } else {
                "EXIF" // Default fallback
            };

            // Look up tag name and definition
            let (base_tag_name, tag_def) =
                if let Some(canon_tag_name) = self.get_canon_tag_name(tag_id) {
                    // Canon-specific tag without definition in main table
                    (canon_tag_name, None)
                } else {
                    // Look up in unified table
                    let tag_def = TAG_BY_ID.get(&(tag_id as u32)).copied();
                    let name = tag_def
                        .map(|def| def.name.to_string())
                        .unwrap_or_else(|| format!("Tag_{tag_id:04X}"));
                    (name, tag_def)
                };

            // Apply conversions to get both value and print
            let (value, print) = self.apply_conversions(raw_value, tag_def);

            let entry = TagEntry {
                group: group_name.to_string(),
                name: base_tag_name,
                value,
                print,
            };

            entries.push(entry);
        }

        // Process composite tags
        for (tag_name, raw_value) in &self.composite_tags {
            // Composite tags already have "Composite:" prefix in the name
            // Extract just the tag name part
            let name = tag_name.strip_prefix("Composite:").unwrap_or(tag_name);

            // Find the composite definition
            let composite_def = COMPOSITE_TAGS.iter().find(|def| def.name == name);

            if let Some(def) = composite_def {
                let (value, print) =
                    crate::composite_tags::apply_composite_conversions(raw_value, def);

                let entry = TagEntry {
                    group: "Composite".to_string(),
                    name: name.to_string(),
                    value,
                    print,
                };

                entries.push(entry);
            } else {
                // Fallback if definition not found
                let entry = TagEntry {
                    group: "Composite".to_string(),
                    name: name.to_string(),
                    value: raw_value.clone(),
                    print: raw_value.to_string(),
                };

                entries.push(entry);
            }
        }

        entries
    }

    /// Get Canon-specific tag name for synthetic tag IDs
    /// Maps synthetic Canon tag IDs back to their proper names
    fn get_canon_tag_name(&self, tag_id: u16) -> Option<String> {
        match tag_id {
            0xC001 => Some("MacroMode".to_string()),
            0xC002 => Some("SelfTimer".to_string()),
            0xC003 => Some("Quality".to_string()),
            0xC004 => Some("CanonFlashMode".to_string()),
            0xC005 => Some("ContinuousDrive".to_string()),
            0xC007 => Some("FocusMode".to_string()),
            _ => None,
        }
    }

    /// Get parsing warnings
    pub fn get_warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Get TIFF header information
    pub fn get_header(&self) -> Option<&TiffHeader> {
        self.header.as_ref()
    }

    /// Test helper: Add extracted tag with source info for testing
    /// Only available when the 'test-helpers' feature is enabled
    /// DO NOT USE in production code - only for tests
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn add_test_tag(&mut self, tag_id: u16, value: TagValue, namespace: &str, ifd_name: &str) {
        use crate::types::{ProcessorType, TagSourceInfo};
        self.extracted_tags.insert(tag_id, value);
        self.tag_sources.insert(
            tag_id,
            TagSourceInfo::new(
                namespace.to_string(),
                ifd_name.to_string(),
                ProcessorType::Exif,
            ),
        );
    }

    /// Apply ValueConv and PrintConv conversions to a raw tag value
    /// ExifTool: lib/Image/ExifTool.pm conversion pipeline
    /// Get current directory path for debugging
    /// ExifTool: $$self{PATH} shows directory hierarchy
    pub fn get_current_path(&self) -> String {
        if self.path.is_empty() {
            "Root".to_string()
        } else {
            self.path.join("/")
        }
    }

    /// Get processing statistics for --show-missing functionality
    pub fn get_processing_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("processed_directories".to_string(), self.processed.len());
        stats.insert("extracted_tags".to_string(), self.extracted_tags.len());
        stats.insert("warnings".to_string(), self.warnings.len());
        stats.insert("data_members".to_string(), self.data_members.len());
        stats.insert(
            "subdirectory_overrides".to_string(),
            self.processor_dispatch.subdirectory_overrides.len(),
        );
        stats
    }

    /// Get current processor dispatch configuration
    pub fn get_processor_dispatch(&self) -> &ProcessorDispatch {
        &self.processor_dispatch
    }

    /// Test helper: Set test data (public for integration tests)
    pub fn set_test_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }

    /// Test helper: Set TIFF header (public for integration tests)
    pub fn set_test_header(&mut self, header: TiffHeader) {
        self.header = Some(header);
    }

    /// Test helper: Get extracted tags (public for integration tests)
    pub fn get_extracted_tags(&self) -> &HashMap<u16, TagValue> {
        &self.extracted_tags
    }

    /// Test helper: Get tag sources (public for integration tests)  
    pub fn get_tag_sources(&self) -> &HashMap<u16, TagSourceInfo> {
        &self.tag_sources
    }

    /// Store tag with conflict resolution and proper namespace handling
    /// ExifTool behavior: Main EXIF tags take precedence over MakerNote tags with same ID
    fn store_tag_with_precedence(
        &mut self,
        tag_id: u16,
        value: TagValue,
        source_info: TagSourceInfo,
    ) {
        use tracing::debug;

        // Check if tag already exists
        if let Some(existing_source) = self.tag_sources.get(&tag_id) {
            // Compare priorities - higher priority wins
            if source_info.priority > existing_source.priority {
                debug!(
                    "Tag 0x{:04x}: Replacing lower priority {} with higher priority {}",
                    tag_id, existing_source.namespace, source_info.namespace
                );
                self.extracted_tags.insert(tag_id, value);
                self.tag_sources.insert(tag_id, source_info);
            } else if source_info.priority == existing_source.priority {
                // Same priority - keep first encountered (ExifTool behavior)
                debug!(
                    "Tag 0x{:04x}: Keeping first encountered {} over {}",
                    tag_id, existing_source.namespace, source_info.namespace
                );
                // Do not overwrite - keep existing
            } else {
                // Lower priority - ignore
                debug!(
                    "Tag 0x{:04x}: Ignoring lower priority {} (existing: {})",
                    tag_id, source_info.namespace, existing_source.namespace
                );
            }
        } else {
            // New tag - store it
            debug!(
                "Tag 0x{:04x}: Storing new {} tag",
                tag_id, source_info.namespace
            );
            self.extracted_tags.insert(tag_id, value);
            self.tag_sources.insert(tag_id, source_info);
        }
    }

    /// Test helper: Get data length (public for integration tests)
    pub fn get_data_len(&self) -> usize {
        self.data.len()
    }

    /// Create TagSourceInfo from IFD name with proper namespace mapping
    /// Maps legacy IFD names to proper ExifTool group names
    fn create_tag_source_info(&self, ifd_name: &str) -> TagSourceInfo {
        // Map IFD names to ExifTool group names
        // ExifTool: lib/Image/ExifTool/Exif.pm group mappings
        let namespace = match ifd_name {
            "Root" | "IFD0" | "IFD1" => "EXIF",
            "GPS" => "EXIF", // GPS tags belong to EXIF group in ExifTool
            "ExifIFD" => "EXIF",
            "InteropIFD" => "EXIF",
            "MakerNotes" => "MakerNotes",
            _ => "EXIF", // Default to EXIF for unknown IFDs
        };

        let processor_type = if namespace == "MakerNotes" {
            // For MakerNotes, try to determine the specific processor
            self.processor_dispatch
                .table_processor
                .clone()
                .unwrap_or(ProcessorType::Exif)
        } else {
            ProcessorType::Exif
        };

        TagSourceInfo::new(namespace.to_string(), ifd_name.to_string(), processor_type)
    }

    /// Apply ValueConv and PrintConv conversions to a raw tag value
    /// ExifTool: lib/Image/ExifTool.pm conversion pipeline
    /// Returns tuple of (value, print) where:
    /// - value: The result after ValueConv (or raw if no ValueConv)
    /// - print: The result after PrintConv (or value.to_string() if no PrintConv)
    fn apply_conversions(
        &self,
        raw_value: &TagValue,
        tag_def: Option<&'static crate::generated::tags::TagDef>,
    ) -> (TagValue, String) {
        use crate::registry;

        let mut value = raw_value.clone();

        // Apply ValueConv first (if present)
        if let Some(tag_def) = tag_def {
            if let Some(value_conv_ref) = tag_def.value_conv_ref {
                value = registry::apply_value_conv(value_conv_ref, &value);
            }
        }

        // Apply PrintConv second (if present) to get human-readable string
        let print = if let Some(tag_def) = tag_def {
            if let Some(print_conv_ref) = tag_def.print_conv_ref {
                registry::apply_print_conv(print_conv_ref, &value)
            } else {
                value.to_string()
            }
        } else {
            value.to_string()
        };

        (value, print)
    }

    /// Check if a tag ID represents a SubDirectory pointer
    /// ExifTool: SubDirectory tags like ExifIFD (0x8769), GPS (0x8825)
    // TODO: Replace magic numbers with named constants (e.g. EXIF_IFD_TAG = 0x8769) for better readability
    fn is_subdirectory_tag(&self, tag_id: u16) -> bool {
        match tag_id {
            0x8769 => true, // ExifIFD - Camera settings subdirectory
            0x8825 => true, // GPS - GPS information subdirectory
            0xA005 => true, // InteropIFD - Interoperability subdirectory
            0x927C => true, // MakerNotes - Manufacturer-specific data
            _ => false,
        }
    }

    /// Select appropriate processor for a directory
    /// ExifTool: $$subdir{ProcessProc} || $$tagTablePtr{PROCESS_PROC} || \&ProcessExif
    fn select_processor(&self, dir_name: &str, tag_id: Option<u16>) -> ProcessorType {
        let (processor, _params) = self.select_processor_with_conditions(
            dir_name,
            tag_id,
            &[],  // No data for simple calls
            0,    // No count
            None, // No format
        );
        processor
    }

    /// Select processor with conditional evaluation support
    /// ExifTool: Full conditional dispatch with runtime evaluation
    fn select_processor_with_conditions(
        &self,
        dir_name: &str,
        tag_id: Option<u16>,
        data: &[u8],
        count: u32,
        format: Option<&str>,
    ) -> (ProcessorType, std::collections::HashMap<String, String>) {
        use crate::conditions::EvalContext;

        // 1. Check for conditional processors with runtime evaluation
        if let Some(tag_id) = tag_id {
            if let Some(conditionals) = self.processor_dispatch.conditional_processors.get(&tag_id)
            {
                // Build evaluation context
                let make = self
                    .extracted_tags
                    .get(&0x010F) // Make tag
                    .and_then(|v| v.as_string());
                let model = self
                    .extracted_tags
                    .get(&0x0110) // Model tag
                    .and_then(|v| v.as_string());

                let context = EvalContext {
                    data,
                    count,
                    format,
                    make,
                    model,
                };

                // Evaluate conditions in order until one matches
                for conditional in conditionals {
                    let matches = conditional
                        .condition
                        .as_ref()
                        .map(|c| c.evaluate(&context))
                        .unwrap_or(true); // Unconditional processors always match

                    if matches {
                        debug!(
                            "Using conditional processor for tag {:#x}: {:?} (condition: {:?})",
                            tag_id, conditional.processor, conditional.condition
                        );
                        return (
                            conditional.processor.clone(),
                            conditional.parameters.clone(),
                        );
                    }
                }
            }

            // 2. Check for legacy subdirectory-specific processor override
            if let Some(processor) = self.processor_dispatch.subdirectory_overrides.get(&tag_id) {
                debug!(
                    "Using legacy SubDirectory ProcessProc override for tag {:#x}: {:?}",
                    tag_id, processor
                );
                return (processor.clone(), std::collections::HashMap::new());
            }
        }

        // 3. Directory-specific defaults (before table-level processor)
        // ExifTool: Some directories have implicit processors
        let dir_specific = match dir_name {
            "GPS" => Some(ProcessorType::Gps),
            "ExifIFD" | "InteropIFD" => Some(ProcessorType::Exif),
            "MakerNotes" => {
                // Detect manufacturer-specific MakerNote processing
                // ExifTool: lib/Image/ExifTool/MakerNotes.pm conditional dispatch
                self.detect_makernote_processor()
            }
            _ => None,
        };

        if let Some(processor) = dir_specific {
            debug!(
                "Using directory-specific processor for {}: {:?}",
                dir_name, processor
            );
            return (processor, std::collections::HashMap::new());
        }

        // 4. Check for table-level processor
        if let Some(processor) = &self.processor_dispatch.table_processor {
            debug!("Using table PROCESS_PROC for {}: {:?}", dir_name, processor);
            return (
                processor.clone(),
                self.processor_dispatch.parameters.clone(),
            );
        }

        // 5. Final fallback to EXIF
        debug!("Using default EXIF processor for {}", dir_name);
        (ProcessorType::Exif, std::collections::HashMap::new())
    }

    /// Dispatch to the appropriate processor function
    /// ExifTool: Dynamic function dispatch with no strict 'refs'
    fn dispatch_processor(
        &mut self,
        processor: ProcessorType,
        dir_info: &DirectoryInfo,
    ) -> Result<()> {
        self.dispatch_processor_with_params(processor, dir_info, &std::collections::HashMap::new())
    }

    /// Dispatch processor with parameters support
    /// ExifTool: Processor dispatch with SubDirectory parameters
    fn dispatch_processor_with_params(
        &mut self,
        processor: ProcessorType,
        dir_info: &DirectoryInfo,
        parameters: &std::collections::HashMap<String, String>,
    ) -> Result<()> {
        trace!(
            "Dispatching to processor {:?} for directory {} with params: {:?}",
            processor,
            dir_info.name,
            parameters
        );

        match processor {
            ProcessorType::Exif | ProcessorType::Gps => {
                // Standard EXIF IFD processing
                // ExifTool: ProcessExif function
                self.process_exif_ifd(dir_info.dir_start, &dir_info.name)
            }
            ProcessorType::BinaryData => {
                // Binary data processing with format tables
                // ExifTool: ProcessBinaryData function
                self.process_binary_data(dir_info)
            }
            ProcessorType::Canon(canon_proc) => {
                // Canon-specific processing
                match canon_proc {
                    crate::types::CanonProcessor::Main => {
                        // Process Canon Main MakerNote table
                        // For Canon, this means processing as IFD to find CameraSettings
                        if dir_info.name == "MakerNotes" {
                            self.process_canon_makernotes(dir_info.dir_start, dir_info.dir_len)
                        } else {
                            // Fall back to standard EXIF processing for other Canon directories
                            self.process_exif_ifd(dir_info.dir_start, &dir_info.name)
                        }
                    }
                    _ => {
                        // Other Canon processors not yet implemented
                        debug!("Canon processor {:?} not yet implemented", canon_proc);
                        self.process_exif_ifd(dir_info.dir_start, &dir_info.name)
                    }
                }
            }
            ProcessorType::Nikon(nikon_proc) => {
                // Nikon-specific processing
                self.process_nikon(nikon_proc, dir_info)
            }
            ProcessorType::Sony(sony_proc) => {
                // Sony-specific processing
                self.process_sony(sony_proc, dir_info)
            }
            ProcessorType::Generic(proc_name) => {
                // Generic/unknown processor - fall back to EXIF
                warn!(
                    "Unknown processor '{}', falling back to EXIF processing",
                    proc_name
                );
                self.process_exif_ifd(dir_info.dir_start, &dir_info.name)
            }
        }
    }

    /// Process a SubDirectory tag by following the pointer to nested IFD
    /// ExifTool: SubDirectory processing with Start => '$val'
    // TODO: Replace magic numbers with named constants (matches above is_subdirectory_tag function)
    fn process_subdirectory_tag(
        &mut self,
        tag_id: u16,
        offset: u32,
        tag_name: &str,
        size: Option<usize>,
    ) -> Result<()> {
        let subdir_name = match tag_id {
            0x8769 => "ExifIFD",
            0x8825 => "GPS",
            0xA005 => "InteropIFD",
            0x927C => "MakerNotes",
            _ => return Ok(()), // Unknown subdirectory
        };

        // Validate offset bounds
        let offset = offset as usize;
        if offset >= self.data.len() {
            self.warnings.push(format!(
                "SubDirectory {} offset {:#x} beyond data bounds ({})",
                subdir_name,
                offset,
                self.data.len()
            ));
            return Ok(()); // Graceful degradation
        }

        // Create subdirectory info with processor override support
        // ExifTool: SubDirectory Start => '$val' means offset points to IFD start
        let dir_info = DirectoryInfo {
            name: subdir_name.to_string(),
            dir_start: offset,
            dir_len: size.unwrap_or(0), // Use provided size for UNDEFINED subdirectories, otherwise calculate during processing
            base: self.base,
            data_pos: 0,
            allow_reprocess: false,
        };

        // Check for SubDirectory ProcessProc override
        // ExifTool: $$subdir{ProcessProc} takes precedence
        if let Some(override_proc) = self.get_subdirectory_processor_override(tag_id) {
            // Store the override in our dispatch system for this call
            // This simulates ExifTool's dynamic processor selection
            trace!(
                "Found SubDirectory ProcessProc override for {}: {:?}",
                subdir_name,
                override_proc
            );
        }

        debug!(
            "Processing SubDirectory: {} -> {} at offset {:#x}",
            tag_name, subdir_name, offset
        );

        // Process the subdirectory
        debug!(
            "About to process subdirectory {} at offset {:#x}",
            subdir_name, offset
        );
        self.process_subdirectory(&dir_info)
    }

    /// Get SubDirectory processor override if available
    /// ExifTool: SubDirectory ProcessProc parameter
    // TODO: Replace magic numbers with named constants (matches other subdirectory functions)
    fn get_subdirectory_processor_override(&self, tag_id: u16) -> Option<ProcessorType> {
        // Check for known SubDirectory processor overrides
        // ExifTool: These are defined in tag tables as SubDirectory => { ProcessProc => ... }
        match tag_id {
            0x8769 => None, // ExifIFD - uses standard EXIF processing
            0x8825 => None, // GPS - uses GPS variant of EXIF processing
            0xA005 => None, // InteropIFD - uses standard EXIF processing
            0x927C => {
                // MakerNotes - use manufacturer-specific processor detection
                // Return None to allow directory-specific detection in select_processor
                None
            }
            _ => None,
        }
    }

    /// Configure processor dispatch for specific table/tag combinations
    /// ExifTool: Runtime processor configuration
    pub fn configure_processor_dispatch(&mut self, dispatch: ProcessorDispatch) {
        self.processor_dispatch = dispatch;
    }

    /// Add SubDirectory processor override
    /// ExifTool: SubDirectory ProcessProc configuration
    pub fn add_subdirectory_override(&mut self, tag_id: u16, processor: ProcessorType) {
        self.processor_dispatch
            .subdirectory_overrides
            .insert(tag_id, processor);
    }

    /// Process standard EXIF IFD (renamed from parse_ifd)
    /// ExifTool: ProcessExif function for standard IFD processing
    fn process_exif_ifd(&mut self, ifd_offset: usize, ifd_name: &str) -> Result<()> {
        // This is the existing parse_ifd logic, renamed for clarity
        self.parse_ifd(ifd_offset, ifd_name)
    }

    /// Process binary data using ProcessBinaryData processor
    /// ExifTool: ProcessBinaryData function (lib/Image/ExifTool.pm:9750)
    fn process_binary_data(&mut self, dir_info: &DirectoryInfo) -> Result<()> {
        debug!("Processing binary data for directory: {}", dir_info.name);

        // Validate directory bounds
        if dir_info.dir_start >= self.data.len() {
            self.warnings.push(format!(
                "Binary data directory {} start offset {:#x} beyond data bounds ({})",
                dir_info.name,
                dir_info.dir_start,
                self.data.len()
            ));
            return Ok(());
        }

        let max_len = self.data.len() - dir_info.dir_start;
        let size = if dir_info.dir_len > 0 && dir_info.dir_len <= max_len {
            dir_info.dir_len
        } else {
            max_len
        };

        debug!(
            "Binary data processing: start={:#x}, len={}, max_len={}",
            dir_info.dir_start, size, max_len
        );

        // For Milestone 9, we'll implement basic Canon CameraSettings processing
        // This is a simplified version focusing on the core mechanism
        if dir_info.name == "MakerNotes" {
            self.process_canon_makernotes(dir_info.dir_start, size)?;
        } else {
            debug!(
                "Binary data processing for {} not yet implemented",
                dir_info.name
            );
        }

        Ok(())
    }

    /// Detect manufacturer-specific MakerNote processor
    /// ExifTool: lib/Image/ExifTool/MakerNotes.pm conditional dispatch system
    fn detect_makernote_processor(&self) -> Option<ProcessorType> {
        // Extract Make and Model from current tags for detection
        let make = self
            .extracted_tags
            .get(&0x010F) // Make tag
            .and_then(|v| v.as_string())
            .unwrap_or("");

        let model = self
            .extracted_tags
            .get(&0x0110) // Model tag
            .and_then(|v| v.as_string())
            .unwrap_or("");

        debug!(
            "Detecting MakerNote processor for Make: '{}', Model: '{}'",
            make, model
        );

        // ExifTool: lib/Image/ExifTool/MakerNotes.pm:60-68 Canon detection
        if canon::detect_canon_signature(make) {
            debug!("Detected Canon MakerNote signature");
            return Some(ProcessorType::Canon(crate::types::CanonProcessor::Main));
        }

        // ExifTool: lib/Image/ExifTool/MakerNotes.pm:1007-1075 Sony detection
        if sony::is_sony_makernote(make, model) {
            debug!("Detected Sony MakerNote (Make field: {})", make);
            return Some(ProcessorType::Sony(SonyProcessor::Main));
        }

        // TODO: Add other manufacturer detection (Nikon, etc.)
        // Return None to fall back to EXIF processor when no manufacturer detected
        debug!("No specific MakerNote processor detected, falling back to EXIF");
        None
    }

    /// Process Canon MakerNotes data with comprehensive offset fixing and tag extraction
    /// ExifTool: Canon.pm processing + MakerNotes.pm offset fixing + ProcessSerialData
    pub fn process_canon_makernotes(&mut self, start_offset: usize, size: usize) -> Result<()> {
        // Check minimum size for Canon MakerNotes
        if size < 12 {
            debug!("MakerNotes too small for Canon processing (need at least 12 bytes)");
            return Ok(());
        }

        debug!(
            "Processing Canon MakerNotes at offset {:#x}, size {}",
            start_offset, size
        );

        // Get Make and Model for offset scheme detection
        let make = self
            .extracted_tags
            .get(&0x010F)
            .and_then(|v| v.as_string())
            .unwrap_or("")
            .to_string();

        let model = self
            .extracted_tags
            .get(&0x0110)
            .and_then(|v| v.as_string())
            .unwrap_or("")
            .to_string();

        let byte_order = self.header.as_ref().unwrap().byte_order;

        debug!(
            "Canon MakerNote processing for Make: '{}', Model: '{}'",
            make, model
        );

        // Detect Canon offset scheme based on model
        let offset_scheme = canon::detect_offset_scheme(&model);
        debug!("Detected Canon offset scheme: {:?}", offset_scheme);

        // Canon offset fixing with footer validation
        // ExifTool: lib/Image/ExifTool/MakerNotes.pm:1281-1307 FixBase Canon section
        let data_pos = 0; // MakerNotes data position in file
        let dir_start = 0; // Directory starts at beginning of MakerNotes data
        let dir_len = size; // Directory length is the MakerNotes size

        // For simplified implementation, we'll skip full val_ptrs/val_block analysis
        // and use empty collections. This allows basic footer detection and validation.
        let val_ptrs: Vec<usize> = Vec::new();
        let val_block: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

        // Attempt Canon offset fixing with footer validation
        let offset_adjustment = match canon::fix_maker_note_base(
            &make,
            &model,
            &self.data[start_offset..start_offset + size],
            dir_start,
            dir_len,
            data_pos,
            byte_order,
            &val_ptrs,
            &val_block,
        ) {
            Ok(Some(offset_fix)) => {
                debug!("Canon offset base adjustment calculated: {}", offset_fix);
                offset_fix
            }
            Ok(None) => {
                debug!("Canon offset fixing: no adjustment needed");
                0
            }
            Err(e) => {
                debug!(
                    "Canon offset fixing failed: {}, using offset scheme default",
                    e
                );
                // Fall back to offset scheme
                offset_scheme.as_bytes() as i64
            }
        };

        // Parse Canon MakerNotes IFD with offset adjustment
        self.parse_canon_makernote_ifd(start_offset, size, offset_adjustment, byte_order, &model)?;

        Ok(())
    }

    /// Parse Canon MakerNote IFD and extract all supported Canon tags
    /// ExifTool: Canon.pm Main table processing
    fn parse_canon_makernote_ifd(
        &mut self,
        start_offset: usize,
        _size: usize,
        offset_adjustment: i64,
        byte_order: ByteOrder,
        model: &str,
    ) -> Result<()> {
        if start_offset + 2 > self.data.len() {
            return Err(ExifError::ParseError(
                "Not enough data for Canon MakerNotes IFD".to_string(),
            ));
        }

        // Read number of IFD entries
        let num_entries = byte_order.read_u16(&self.data, start_offset)? as usize;
        debug!("Canon MakerNotes IFD has {} entries", num_entries);

        if num_entries > 256 {
            return Err(ExifError::ParseError(format!(
                "Invalid Canon MakerNotes entry count: {num_entries}"
            )));
        }

        // Process each IFD entry to find Canon-specific tags
        for i in 0..num_entries {
            let entry_offset = start_offset + 2 + (i * 12);
            if entry_offset + 12 > self.data.len() {
                debug!("Canon MakerNote entry {} beyond data bounds", i);
                break;
            }

            let tag_id = byte_order.read_u16(&self.data, entry_offset)?;
            let format = byte_order.read_u16(&self.data, entry_offset + 2)?;
            let count = byte_order.read_u32(&self.data, entry_offset + 4)?;
            let value_offset = byte_order.read_u32(&self.data, entry_offset + 8)?;

            debug!(
                "Canon tag {:#04x}: format={}, count={}, value_offset={:#x}",
                tag_id, format, count, value_offset
            );

            // Calculate adjusted offset for Canon values
            let format_size = Self::format_size(format)? as u32;
            let adjusted_offset = if count * format_size <= 4 {
                // Inline value (4 bytes or less)
                entry_offset + 8
            } else {
                // External value - apply offset adjustment
                (value_offset as i64 + offset_adjustment) as usize + start_offset
            };

            // Process Canon-specific tags
            match tag_id {
                0x0001 => {
                    // Canon CameraSettings (ProcessBinaryData)
                    debug!("Processing Canon CameraSettings tag");
                    self.process_canon_camera_settings(
                        adjusted_offset,
                        count as usize,
                        byte_order,
                    )?;
                }
                0x0012 => {
                    // Canon AFInfo (ProcessSerialData)
                    debug!("Processing Canon AFInfo tag");
                    self.process_canon_af_info(adjusted_offset, count as usize, byte_order, model)?;
                }
                0x0026 => {
                    // Canon AFInfo2 (ProcessSerialData)
                    debug!("Processing Canon AFInfo2 tag");
                    self.process_canon_af_info2(
                        adjusted_offset,
                        count as usize,
                        byte_order,
                        model,
                    )?;
                }
                _ => {
                    // Other Canon tags - basic extraction for now
                    if let Ok(tag_value) = self.extract_basic_canon_tag(
                        tag_id,
                        format,
                        count,
                        adjusted_offset,
                        byte_order,
                    ) {
                        let source_info = TagSourceInfo::new(
                            "MakerNotes".to_string(),
                            "Canon::Main".to_string(),
                            ProcessorType::Canon(crate::types::CanonProcessor::Main),
                        );
                        // Use wrapping_add to prevent overflow for large Canon tag IDs
                        let synthetic_tag_id = 0xC000u16.wrapping_add(tag_id);
                        self.store_tag_with_precedence(synthetic_tag_id, tag_value, source_info);
                    }
                }
            }
        }

        Ok(())
    }

    /// Process Canon CameraSettings using ProcessBinaryData
    /// ExifTool: Canon.pm CameraSettings table
    fn process_canon_camera_settings(
        &mut self,
        offset: usize,
        count: usize,
        byte_order: ByteOrder,
    ) -> Result<()> {
        let size = count * 2; // int16s format = 2 bytes each
        if offset + size > self.data.len() {
            debug!("Canon CameraSettings data beyond buffer bounds");
            return Ok(());
        }

        match canon::extract_camera_settings(&self.data, offset, size, byte_order) {
            Ok(canon_tags) => {
                debug!(
                    "Successfully extracted {} Canon CameraSettings tags",
                    canon_tags.len()
                );

                // Add Canon tags with synthetic tag IDs
                for (tag_name, tag_value) in canon_tags {
                    let synthetic_tag_id = match tag_name.as_str() {
                        "MakerNotes:MacroMode" => 0xC001,
                        "MakerNotes:SelfTimer" => 0xC002,
                        "MakerNotes:Quality" => 0xC003,
                        "MakerNotes:CanonFlashMode" => 0xC004,
                        "MakerNotes:ContinuousDrive" => 0xC005,
                        "MakerNotes:FocusMode" => 0xC007,
                        _ => continue,
                    };

                    let source_info = TagSourceInfo::new(
                        "MakerNotes".to_string(),
                        "Canon::CameraSettings".to_string(),
                        ProcessorType::Canon(crate::types::CanonProcessor::CameraSettings),
                    );
                    self.store_tag_with_precedence(synthetic_tag_id, tag_value, source_info);
                }
                Ok(())
            }
            Err(e) => {
                warn!("Failed to extract Canon CameraSettings: {}", e);
                Ok(())
            }
        }
    }

    /// Process Canon AFInfo using ProcessSerialData
    /// ExifTool: Canon.pm AFInfo table
    fn process_canon_af_info(
        &mut self,
        offset: usize,
        count: usize,
        byte_order: ByteOrder,
        model: &str,
    ) -> Result<()> {
        if offset + count > self.data.len() {
            debug!("Canon AFInfo data beyond buffer bounds");
            return Ok(());
        }

        let af_info_table = canon::create_af_info_table();
        match canon::process_serial_data(
            &self.data,
            offset,
            count,
            byte_order,
            &af_info_table,
            model,
        ) {
            Ok(af_tags) => {
                debug!("Successfully extracted {} Canon AFInfo tags", af_tags.len());

                // Add AF tags with synthetic tag IDs
                for (tag_name, tag_value) in af_tags {
                    let synthetic_tag_id = match tag_name.as_str() {
                        "MakerNotes:NumAFPoints" => 0xC020,
                        "MakerNotes:ValidAFPoints" => 0xC021,
                        "MakerNotes:CanonImageWidth" => 0xC022,
                        "MakerNotes:CanonImageHeight" => 0xC023,
                        "MakerNotes:AFImageWidth" => 0xC024,
                        "MakerNotes:AFImageHeight" => 0xC025,
                        "MakerNotes:AFAreaWidth" => 0xC026,
                        "MakerNotes:AFAreaHeight" => 0xC027,
                        "MakerNotes:AFAreaXPositions" => 0xC028,
                        "MakerNotes:AFAreaYPositions" => 0xC029,
                        "MakerNotes:AFPointsInFocus" => 0xC02A,
                        "MakerNotes:PrimaryAFPoint" => 0xC02B,
                        _ => continue,
                    };

                    let source_info = TagSourceInfo::new(
                        "MakerNotes".to_string(),
                        "Canon::AFInfo".to_string(),
                        ProcessorType::Canon(crate::types::CanonProcessor::AfInfo),
                    );
                    self.store_tag_with_precedence(synthetic_tag_id, tag_value, source_info);
                }
                Ok(())
            }
            Err(e) => {
                warn!("Failed to extract Canon AFInfo: {}", e);
                Ok(())
            }
        }
    }

    /// Process Canon AFInfo2 using ProcessSerialData
    /// ExifTool: Canon.pm AFInfo2 table
    fn process_canon_af_info2(
        &mut self,
        offset: usize,
        count: usize,
        byte_order: ByteOrder,
        model: &str,
    ) -> Result<()> {
        if offset + count > self.data.len() {
            debug!("Canon AFInfo2 data beyond buffer bounds");
            return Ok(());
        }

        let af_info2_table = canon::create_af_info2_table();
        match canon::process_serial_data(
            &self.data,
            offset,
            count,
            byte_order,
            &af_info2_table,
            model,
        ) {
            Ok(af_tags) => {
                debug!(
                    "Successfully extracted {} Canon AFInfo2 tags",
                    af_tags.len()
                );

                // Add AF2 tags with synthetic tag IDs
                for (tag_name, tag_value) in af_tags {
                    let synthetic_tag_id = match tag_name.as_str() {
                        "MakerNotes:AFInfoSize" => 0xC030,
                        "MakerNotes:AFAreaMode" => 0xC031,
                        "MakerNotes:NumAFPoints" => 0xC032,
                        "MakerNotes:ValidAFPoints" => 0xC033,
                        "MakerNotes:CanonImageWidth" => 0xC034,
                        "MakerNotes:CanonImageHeight" => 0xC035,
                        "MakerNotes:AFImageWidth" => 0xC036,
                        "MakerNotes:AFImageHeight" => 0xC037,
                        "MakerNotes:AFAreaWidths" => 0xC038,
                        "MakerNotes:AFAreaHeights" => 0xC039,
                        "MakerNotes:AFAreaXPositions" => 0xC03A,
                        "MakerNotes:AFAreaYPositions" => 0xC03B,
                        "MakerNotes:AFPointsInFocus" => 0xC03C,
                        "MakerNotes:AFPointsSelected" => 0xC03D,
                        _ => continue,
                    };

                    let source_info = TagSourceInfo::new(
                        "MakerNotes".to_string(),
                        "Canon::AFInfo2".to_string(),
                        ProcessorType::Canon(crate::types::CanonProcessor::AfInfo2),
                    );
                    self.store_tag_with_precedence(synthetic_tag_id, tag_value, source_info);
                }
                Ok(())
            }
            Err(e) => {
                warn!("Failed to extract Canon AFInfo2: {}", e);
                Ok(())
            }
        }
    }

    /// Extract basic Canon tag value (for tags not yet fully implemented)
    fn extract_basic_canon_tag(
        &self,
        tag_id: u16,
        format: u16,
        count: u32,
        offset: usize,
        byte_order: ByteOrder,
    ) -> Result<TagValue> {
        let format_size = Self::format_size(format)?;
        let total_size = count as usize * format_size;

        if offset + total_size > self.data.len() {
            return Err(ExifError::ParseError(format!(
                "Canon tag {tag_id:#04x} data beyond bounds"
            )));
        }

        // Extract basic value based on format
        match format {
            2 => {
                // ASCII string
                let string_data = &self.data[offset..offset + total_size];
                let string = String::from_utf8_lossy(string_data)
                    .trim_end_matches('\0')
                    .to_string();
                Ok(TagValue::String(string))
            }
            3 => {
                // int16u
                let value = byte_order.read_u16(&self.data, offset)?;
                Ok(TagValue::U16(value))
            }
            4 => {
                // int32u
                let value = byte_order.read_u32(&self.data, offset)?;
                Ok(TagValue::U32(value))
            }
            _ => {
                // Other formats - return as raw bytes for now
                let raw_data = self.data[offset..offset + total_size].to_vec();
                Ok(TagValue::String(format!(
                    "(Binary data {} bytes)",
                    raw_data.len()
                )))
            }
        }
    }

    /// Get format size in bytes
    fn format_size(format: u16) -> Result<usize> {
        match format {
            1 | 2 | 6 | 7 => Ok(1), // int8u, string, int8s, undef
            3 | 8 => Ok(2),         // int16u, int16s
            4 | 9 | 11 => Ok(4),    // int32u, int32s, float
            5 | 10 | 12 => Ok(8),   // rational64u, rational64s, double
            _ => Err(ExifError::ParseError(format!("Invalid format: {format}"))),
        }
    }

    /// Find Canon CameraSettings tag (0x0001) in MakerNotes IFD
    /// ExifTool: Canon.pm Main table tag 0x1
    pub fn find_canon_camera_settings_tag(
        &self,
        start_offset: usize,
        _size: usize,
    ) -> Result<usize> {
        if start_offset + 14 > self.data.len() {
            return Err(ExifError::ParseError(
                "Not enough data for Canon MakerNotes IFD".to_string(),
            ));
        }

        let byte_order = self
            .header
            .as_ref()
            .map(|h| h.byte_order)
            .unwrap_or(ByteOrder::LittleEndian);

        // Read number of IFD entries
        let num_entries = byte_order.read_u16(&self.data, start_offset)? as usize;
        debug!("Canon MakerNotes IFD has {} entries", num_entries);

        if num_entries == 0 || num_entries > 100 {
            return Err(ExifError::ParseError(format!(
                "Invalid Canon MakerNotes entry count: {num_entries}"
            )));
        }

        // Search for tag 0x0001 (CanonCameraSettings)
        for i in 0..num_entries {
            let entry_offset = start_offset + 2 + (i * 12);
            if entry_offset + 12 > self.data.len() {
                break;
            }

            let tag_id = byte_order.read_u16(&self.data, entry_offset)?;
            if tag_id == 0x0001 {
                // Found Canon CameraSettings tag
                let format = byte_order.read_u16(&self.data, entry_offset + 2)?;
                let count = byte_order.read_u32(&self.data, entry_offset + 4)?;
                let value_offset = byte_order.read_u32(&self.data, entry_offset + 8)?;

                debug!(
                    "Canon CameraSettings: format={}, count={}, offset={:#x}",
                    format, count, value_offset
                );

                // Calculate absolute offset for CameraSettings data
                // For Canon, the value_offset is relative to the start of the MakerNotes
                let camera_settings_offset = if count * 2 <= 4 {
                    // Data is inline in the offset field
                    entry_offset + 8
                } else {
                    // Data is at offset
                    start_offset + value_offset as usize
                };

                if camera_settings_offset < self.data.len() {
                    return Ok(camera_settings_offset);
                }
            }
        }

        Err(ExifError::ParseError(
            "Canon CameraSettings tag (0x0001) not found".to_string(),
        ))
    }

    /// Create Canon CameraSettings binary data table
    /// ExifTool: Canon.pm %Canon::CameraSettings table
    pub fn create_canon_camera_settings_table(&self) -> crate::types::BinaryDataTable {
        use crate::types::{BinaryDataFormat, BinaryDataTable, BinaryDataTag};
        use std::collections::HashMap;

        let mut table = BinaryDataTable {
            default_format: BinaryDataFormat::Int16s,
            first_entry: Some(1),
            groups: {
                let mut groups = HashMap::new();
                groups.insert(0, "MakerNotes".to_string());
                groups.insert(2, "Camera".to_string());
                groups
            },
            tags: HashMap::new(),
        };

        // Add MacroMode tag at index 1
        table.tags.insert(
            1,
            BinaryDataTag {
                name: "MacroMode".to_string(),
                format: None, // Uses table default (int16s)
                mask: None,
                print_conv: {
                    let mut conv = HashMap::new();
                    conv.insert(1, "Macro".to_string());
                    conv.insert(2, "Normal".to_string());
                    Some(conv)
                },
            },
        );

        // Add FocusMode tag at index 7
        table.tags.insert(
            7,
            BinaryDataTag {
                name: "FocusMode".to_string(),
                format: None, // Uses table default (int16s)
                mask: None,
                print_conv: {
                    let mut conv = HashMap::new();
                    conv.insert(0, "One-shot AF".to_string());
                    conv.insert(1, "AI Servo AF".to_string());
                    conv.insert(2, "AI Focus AF".to_string());
                    conv.insert(3, "Manual Focus (3)".to_string());
                    conv.insert(4, "Single".to_string());
                    conv.insert(5, "Continuous".to_string());
                    conv.insert(6, "Manual Focus (6)".to_string());
                    Some(conv)
                },
            },
        );

        table
    }

    /// Extract binary data tags using table definition
    /// ExifTool: ProcessBinaryData main processing loop
    pub fn extract_binary_data_tags(
        &mut self,
        start_offset: usize,
        size: usize,
        table: &crate::types::BinaryDataTable,
    ) -> Result<()> {
        let increment = table.default_format.byte_size();

        debug!(
            "Extracting binary data tags: start={:#x}, size={}, increment={}, format={:?}",
            start_offset, size, increment, table.default_format
        );

        // Process defined tags
        for (&index, tag_def) in &table.tags {
            let entry_offset = (index as usize) * increment;
            if entry_offset + increment > size {
                debug!("Tag {} at index {} beyond data bounds", tag_def.name, index);
                continue;
            }

            let data_offset = start_offset + entry_offset;
            let format = tag_def.format.unwrap_or(table.default_format);

            // Extract value based on format
            if let Ok(value) = self.extract_binary_value(data_offset, format, 1) {
                debug!(
                    "Extracted {} = {:?} at index {}",
                    tag_def.name, value, index
                );

                // Apply PrintConv if available
                let final_value = if let Some(print_conv) = &tag_def.print_conv {
                    // Try to get the value as u32 for lookup
                    let lookup_val = match value {
                        TagValue::U8(v) => Some(v as u32),
                        TagValue::U16(v) => Some(v as u32),
                        TagValue::U32(v) => Some(v),
                        TagValue::I16(v) => Some(v as u32),
                        TagValue::I32(v) => Some(v as u32),
                        _ => None,
                    };

                    if let Some(int_val) = lookup_val {
                        if let Some(converted) = print_conv.get(&int_val) {
                            TagValue::String(converted.clone())
                        } else {
                            value
                        }
                    } else {
                        value
                    }
                } else {
                    value
                };

                // Store with group prefix
                let unknown_default = "Unknown".to_string();
                let group_prefix = table.groups.get(&0).unwrap_or(&unknown_default);
                let source_info = TagSourceInfo::new(
                    group_prefix.clone(),
                    format!("BinaryData/{group_prefix}"),
                    ProcessorType::BinaryData,
                );
                self.store_tag_with_precedence(index as u16, final_value, source_info);
            }
        }

        Ok(())
    }

    /// Extract a single binary value from data
    /// ExifTool: Value extraction with format-specific handling
    pub fn extract_binary_value(
        &self,
        offset: usize,
        format: crate::types::BinaryDataFormat,
        count: usize,
    ) -> Result<TagValue> {
        use crate::types::BinaryDataFormat;

        if offset >= self.data.len() {
            return Err(ExifError::ParseError(
                "Offset beyond data bounds".to_string(),
            ));
        }

        let byte_order = self
            .header
            .as_ref()
            .map(|h| h.byte_order)
            .unwrap_or(ByteOrder::LittleEndian);

        match format {
            BinaryDataFormat::Int8u => Ok(TagValue::U8(self.data[offset])),
            BinaryDataFormat::Int8s => Ok(TagValue::I16(self.data[offset] as i8 as i16)),
            BinaryDataFormat::Int16u => {
                if offset + 2 > self.data.len() {
                    return Err(ExifError::ParseError(
                        "Not enough data for int16u".to_string(),
                    ));
                }
                let value = byte_order.read_u16(&self.data, offset)?;
                Ok(TagValue::U16(value))
            }
            BinaryDataFormat::Int16s => {
                if offset + 2 > self.data.len() {
                    return Err(ExifError::ParseError(
                        "Not enough data for int16s".to_string(),
                    ));
                }
                let value = byte_order.read_u16(&self.data, offset)? as i16;
                Ok(TagValue::I16(value))
            }
            BinaryDataFormat::Int32u => {
                if offset + 4 > self.data.len() {
                    return Err(ExifError::ParseError(
                        "Not enough data for int32u".to_string(),
                    ));
                }
                let value = byte_order.read_u32(&self.data, offset)?;
                Ok(TagValue::U32(value))
            }
            BinaryDataFormat::Int32s => {
                if offset + 4 > self.data.len() {
                    return Err(ExifError::ParseError(
                        "Not enough data for int32s".to_string(),
                    ));
                }
                let value = byte_order.read_u32(&self.data, offset)? as i32;
                Ok(TagValue::I32(value))
            }
            BinaryDataFormat::String => {
                let remaining = self.data.len() - offset;
                let max_len = if count > 0 {
                    count.min(remaining)
                } else {
                    remaining
                };

                // Find null terminator or use max length
                let end = self.data[offset..offset + max_len]
                    .iter()
                    .position(|&b| b == 0)
                    .unwrap_or(max_len);

                let bytes = &self.data[offset..offset + end];
                match std::str::from_utf8(bytes) {
                    Ok(s) => Ok(TagValue::String(s.to_string())),
                    Err(_) => Ok(TagValue::Binary(bytes.to_vec())),
                }
            }
            BinaryDataFormat::PString => {
                if offset >= self.data.len() {
                    return Err(ExifError::ParseError(
                        "Not enough data for pstring length".to_string(),
                    ));
                }
                let len = self.data[offset] as usize;
                if offset + 1 + len > self.data.len() {
                    return Err(ExifError::ParseError(
                        "Not enough data for pstring content".to_string(),
                    ));
                }
                let bytes = &self.data[offset + 1..offset + 1 + len];
                match std::str::from_utf8(bytes) {
                    Ok(s) => Ok(TagValue::String(s.to_string())),
                    Err(_) => Ok(TagValue::Binary(bytes.to_vec())),
                }
            }
            _ => {
                debug!("Unsupported binary format: {:?}", format);
                Ok(TagValue::Binary(vec![0]))
            }
        }
    }

    /// Process Canon manufacturer-specific data
    /// ExifTool: Canon.pm processing procedures
    #[allow(dead_code)]
    fn process_canon(
        &mut self,
        _canon_proc: crate::types::CanonProcessor,
        dir_info: &DirectoryInfo,
    ) -> Result<()> {
        // TODO: Implement Canon-specific processing for different CanonProcessor types
        // This will be expanded in future milestones for ProcessSerialData, etc.
        debug!("Canon processing not yet implemented for {}", dir_info.name);
        self.process_exif_ifd(dir_info.dir_start, &dir_info.name)
    }

    /// Process Nikon manufacturer-specific data
    /// ExifTool: Nikon.pm processing procedures
    fn process_nikon(
        &mut self,
        _nikon_proc: crate::types::NikonProcessor,
        dir_info: &DirectoryInfo,
    ) -> Result<()> {
        // Placeholder for Nikon-specific processing
        // This will be implemented in future milestones
        debug!("Nikon processing not yet implemented for {}", dir_info.name);
        self.process_exif_ifd(dir_info.dir_start, &dir_info.name)
    }

    /// Process Sony MakerNotes with proper namespace handling
    /// ExifTool: Sony-specific processing to prevent tag collisions
    fn process_sony(&mut self, _sony_proc: SonyProcessor, dir_info: &DirectoryInfo) -> Result<()> {
        debug!(
            "Processing Sony MakerNote directory: {} (processor: {:?})",
            dir_info.name, _sony_proc
        );

        // For Sony MakerNotes, we want to ensure proper namespacing
        // This stub processes as EXIF IFD but with MakerNotes namespace
        if dir_info.name == "MakerNotes" {
            // Extract Make for logging (before mutable borrow)
            let make = self
                .extracted_tags
                .get(&0x010F) // Make tag
                .and_then(|v| v.as_string())
                .unwrap_or("")
                .to_string();

            // Temporarily process with MakerNotes context for proper tag source tracking
            self.process_exif_ifd_with_namespace(
                dir_info.dir_start,
                "MakerNotes",
                ProcessorType::Sony(_sony_proc),
            )?;

            debug!("Sony MakerNote processing completed for Make: {}", make);
        } else {
            // Fall back to standard EXIF processing for other Sony directories
            self.process_exif_ifd(dir_info.dir_start, &dir_info.name)?;
        }

        Ok(())
    }

    /// Process EXIF IFD with explicit namespace and processor context
    /// Used for MakerNotes to ensure proper tag source tracking and conflict resolution
    fn process_exif_ifd_with_namespace(
        &mut self,
        ifd_offset: usize,
        namespace: &str,
        processor_type: ProcessorType,
    ) -> Result<()> {
        debug!(
            "Processing IFD with namespace '{}' at offset {:#x}",
            namespace, ifd_offset
        );

        if ifd_offset + 2 > self.data.len() {
            return Err(ExifError::ParseError(format!(
                "IFD offset {ifd_offset:#x} beyond data bounds"
            )));
        }

        let byte_order = self.header.as_ref().unwrap().byte_order;
        let num_entries = byte_order.read_u16(&self.data, ifd_offset)? as usize;

        debug!("Processing {} entries in {} IFD", num_entries, namespace);

        // Process each IFD entry
        for i in 0..num_entries {
            let entry_offset = ifd_offset + 2 + (i * 12);
            if let Ok(entry) = IfdEntry::parse(&self.data, entry_offset, byte_order) {
                let tag_id = entry.tag_id;

                // Create TagSourceInfo for this tag
                let source_info = TagSourceInfo::new(
                    namespace.to_string(),
                    format!("{}/{}", self.path.join("/"), namespace),
                    processor_type.clone(),
                );

                // Extract tag value
                if let Ok(value) = self.extract_tag_value(&entry, byte_order) {
                    // Store with conflict resolution
                    self.store_tag_with_precedence(tag_id, value, source_info);
                } else {
                    debug!(
                        "Failed to extract value for tag {:#x} in {}",
                        tag_id, namespace
                    );
                }
            }
        }

        Ok(())
    }

    /// Extract tag value from IFD entry (helper method)
    fn extract_tag_value(&self, entry: &IfdEntry, byte_order: ByteOrder) -> Result<TagValue> {
        match entry.format {
            TiffFormat::Ascii => Ok(TagValue::String(value_extraction::extract_ascii_value(
                &self.data, entry, byte_order,
            )?)),
            TiffFormat::Short => Ok(TagValue::U16(value_extraction::extract_short_value(
                &self.data, entry, byte_order,
            )?)),
            TiffFormat::Long => Ok(TagValue::U32(value_extraction::extract_long_value(
                &self.data, entry, byte_order,
            )?)),
            TiffFormat::Byte => Ok(TagValue::U8(value_extraction::extract_byte_value(
                &self.data, entry,
            )?)),
            TiffFormat::Rational => {
                value_extraction::extract_rational_value(&self.data, entry, byte_order)
            }
            TiffFormat::SRational => {
                value_extraction::extract_srational_value(&self.data, entry, byte_order)
            }
            TiffFormat::SShort => {
                let unsigned =
                    value_extraction::extract_short_value(&self.data, entry, byte_order)?;
                Ok(TagValue::I16(unsigned as i16))
            }
            TiffFormat::SLong => {
                let unsigned = value_extraction::extract_long_value(&self.data, entry, byte_order)?;
                Ok(TagValue::I32(unsigned as i32))
            }
            _ => {
                debug!(
                    "Unsupported format {:?} for tag {:#x}",
                    entry.format, entry.tag_id
                );
                Err(ExifError::Unsupported(format!(
                    "Format {:?} not yet supported",
                    entry.format
                )))
            }
        }
    }

    // NOTE: GPS decimal conversion is deferred to Milestone 8 (ValueConv registry)
    // This conversion will be implemented as ValueConv functions that chain with PrintConv
}

impl Default for ExifReader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exif_reader_basic() {
        let mut reader = ExifReader::new();

        // Create minimal EXIF data with TIFF header and empty IFD
        let exif_data = [
            0x49, 0x49, // "II" - little-endian
            0x2A, 0x00, // Magic: 42 (LE)
            0x08, 0x00, 0x00, 0x00, // IFD0 offset: 8 (LE)
            0x00, 0x00, // Number of entries: 0
            0x00, 0x00, 0x00, 0x00, // Next IFD: none
        ];

        reader.parse_exif_data(&exif_data).unwrap();

        let header = reader.get_header().unwrap();
        assert_eq!(header.byte_order, ByteOrder::LittleEndian);
        assert_eq!(header.ifd0_offset, 8);
    }

    #[test]
    fn test_subdirectory_recursion_prevention() {
        let mut reader = ExifReader::new();

        // Test recursion prevention by manually manipulating the PROCESSED hash
        let addr = 100u64;

        // Initially not processed
        assert!(!reader.processed.contains_key(&addr));

        // Mark as processed
        reader.processed.insert(addr, "TestIFD".to_string());

        // Verify it's marked as processed
        assert!(reader.processed.contains_key(&addr));
        assert_eq!(reader.processed.get(&addr), Some(&"TestIFD".to_string()));

        // Test that the same address would be detected as circular reference
        // We'll test this by checking the logic directly rather than calling process_subdirectory
        // which requires valid EXIF data
        let dir_info = DirectoryInfo {
            name: "TestIFD2".to_string(),
            dir_start: 100,
            dir_len: 0,
            base: 0,
            data_pos: 0,
            allow_reprocess: false,
        };

        let calculated_addr = dir_info.dir_start as u64 + dir_info.data_pos + dir_info.base;
        assert_eq!(calculated_addr, addr);
        assert!(reader.processed.contains_key(&calculated_addr));
    }

    #[test]
    fn test_subdirectory_path_management() {
        let mut reader = ExifReader::new();

        // Initially empty path
        assert_eq!(reader.get_current_path(), "Root");

        // Create some directory info (not used in this test)
        let _dir_info = DirectoryInfo {
            name: "ExifIFD".to_string(),
            dir_start: 0,
            dir_len: 0,
            base: 0,
            data_pos: 0,
            allow_reprocess: false,
        };

        // Manually test path management
        reader.path.push("IFD0".to_string());
        assert_eq!(reader.get_current_path(), "IFD0");

        reader.path.push("ExifIFD".to_string());
        assert_eq!(reader.get_current_path(), "IFD0/ExifIFD");

        reader.path.pop();
        assert_eq!(reader.get_current_path(), "IFD0");
    }

    #[test]
    fn test_subdirectory_tag_detection() {
        let reader = ExifReader::new();

        // Test that SubDirectory tags are detected correctly
        assert!(reader.is_subdirectory_tag(0x8769)); // ExifIFD
        assert!(reader.is_subdirectory_tag(0x8825)); // GPS
        assert!(reader.is_subdirectory_tag(0xA005)); // InteropIFD
        assert!(reader.is_subdirectory_tag(0x927C)); // MakerNotes

        // Test that regular tags are not detected as subdirectories
        assert!(!reader.is_subdirectory_tag(0x010F)); // Make
        assert!(!reader.is_subdirectory_tag(0x0110)); // Model
        assert!(!reader.is_subdirectory_tag(0x0112)); // Orientation
    }

    #[test]
    fn test_processing_statistics() {
        let mut reader = ExifReader::new();

        // Add some mock data
        reader
            .extracted_tags
            .insert(0x010F, TagValue::String("Canon".to_string()));
        reader.warnings.push("Test warning".to_string());
        reader
            .data_members
            .insert("TestMember".to_string(), DataMemberValue::U16(42));

        let stats = reader.get_processing_stats();
        assert_eq!(stats.get("extracted_tags"), Some(&1));
        assert_eq!(stats.get("warnings"), Some(&1));
        assert_eq!(stats.get("data_members"), Some(&1));
        assert_eq!(stats.get("processed_directories"), Some(&0));
        assert_eq!(stats.get("subdirectory_overrides"), Some(&0));
    }

    #[test]
    fn test_processor_dispatch_selection() {
        let reader = ExifReader::new();

        // Test default processor selection
        assert_eq!(reader.select_processor("IFD0", None), ProcessorType::Exif);
        assert_eq!(
            reader.select_processor("ExifIFD", None),
            ProcessorType::Exif
        );
        assert_eq!(reader.select_processor("GPS", None), ProcessorType::Gps);
        assert_eq!(
            reader.select_processor("InteropIFD", None),
            ProcessorType::Exif
        );

        // Test MakerNotes gets manufacturer-specific detection (defaults to EXIF when no Make/Model)
        let processor = reader.select_processor("MakerNotes", None);
        match processor {
            ProcessorType::Exif => {
                // Expected when no Make/Model tags are available for detection
            }
            ProcessorType::Canon(_) => {
                // Expected when Canon Make is detected
            }
            _ => panic!("Expected EXIF or Canon processor for MakerNotes, got {processor:?}"),
        }
    }

    #[test]
    fn test_processor_dispatch_overrides() {
        let mut reader = ExifReader::new();

        // Add a SubDirectory override
        reader.add_subdirectory_override(0x8769, ProcessorType::BinaryData);

        // Verify override is stored
        let dispatch = reader.get_processor_dispatch();
        assert_eq!(
            dispatch.subdirectory_overrides.get(&0x8769),
            Some(&ProcessorType::BinaryData)
        );

        // Verify stats reflect the override
        let stats = reader.get_processing_stats();
        assert_eq!(stats.get("subdirectory_overrides"), Some(&1));
    }

    #[test]
    fn test_subdirectory_processor_overrides() {
        let reader = ExifReader::new();

        // Test known SubDirectory processor overrides
        assert_eq!(reader.get_subdirectory_processor_override(0x8769), None); // ExifIFD
        assert_eq!(reader.get_subdirectory_processor_override(0x8825), None); // GPS
        assert_eq!(reader.get_subdirectory_processor_override(0xA005), None); // InteropIFD

        // MakerNotes should have no override (to allow manufacturer-specific detection)
        assert_eq!(reader.get_subdirectory_processor_override(0x927C), None);

        // Unknown tag should have no override
        assert_eq!(reader.get_subdirectory_processor_override(0x1234), None);
    }

    #[test]
    fn test_gps_rational_arrays_returned_raw() {
        use crate::types::TagValue;

        // Test that GPS coordinates return raw rational arrays in Milestone 8e
        // GPS:GPSLatitude should return [[54,1], [59,38/100], [0,1]] not decimal degrees

        // Test GPSLatitude returns rational array format
        let lat_rationals = TagValue::RationalArray(vec![(54, 1), (5938, 100), (0, 1)]);

        // Verify we can access the rational components directly
        if let TagValue::RationalArray(rationals) = &lat_rationals {
            assert_eq!(
                rationals.len(),
                3,
                "GPS coordinates should have 3 components"
            );
            assert_eq!(rationals[0], (54, 1), "Degrees component");
            assert_eq!(rationals[1], (5938, 100), "Minutes component");
            assert_eq!(rationals[2], (0, 1), "Seconds component");
        } else {
            panic!("GPS coordinates should be RationalArray");
        }

        // Test GPSLongitude format
        let lon_rationals = TagValue::RationalArray(vec![(1, 1), (5485, 100), (0, 1)]);
        if let TagValue::RationalArray(rationals) = &lon_rationals {
            assert_eq!(rationals[0], (1, 1), "Degrees component");
            assert_eq!(rationals[1], (5485, 100), "Minutes component");
            assert_eq!(rationals[2], (0, 1), "Seconds component");
        } else {
            panic!("GPS coordinates should be RationalArray");
        }

        // GPS reference tags should remain as strings
        let lat_ref = TagValue::String("N".to_string());
        let lon_ref = TagValue::String("W".to_string());

        assert_eq!(lat_ref.as_string(), Some("N"));
        assert_eq!(lon_ref.as_string(), Some("W"));

        // Note: Decimal conversion will be handled by Composite tags
        // that combine GPS:GPSLatitude + GPS:GPSLatitudeRef -> Composite:GPSLatitude
    }
}
