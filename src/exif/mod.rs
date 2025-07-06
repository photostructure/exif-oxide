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

mod binary_data;
mod ifd;
mod processors;
mod tags;

// Only re-export what needs to be public - most functionality is internal

use crate::tiff_types::TiffHeader;
use crate::types::{
    DataMemberValue, DirectoryInfo, ExifError, ProcessorDispatch, Result, TagSourceInfo, TagValue,
};
use std::collections::HashMap;
use tracing::debug;

/// Stateful EXIF reader for processing JPEG-embedded EXIF data
/// ExifTool: lib/Image/ExifTool/Exif.pm ProcessExif function architecture
#[derive(Debug)]
pub struct ExifReader {
    /// Extracted tag values by tag ID
    pub(crate) extracted_tags: HashMap<u16, TagValue>,
    /// Enhanced tag source tracking for conflict resolution
    /// Maps tag_id -> TagSourceInfo with namespace, priority, and processor context
    pub(crate) tag_sources: HashMap<u16, TagSourceInfo>,
    /// TIFF header information
    pub(crate) header: Option<TiffHeader>,
    /// Raw EXIF data buffer
    pub(crate) data: Vec<u8>,
    /// Parse errors (non-fatal, for graceful degradation)
    pub(crate) warnings: Vec<String>,

    // Milestone 5: Stateful processing features
    /// PROCESSED hash for recursion prevention
    /// ExifTool: $$self{PROCESSED} prevents infinite loops
    pub(crate) processed: HashMap<u64, String>,
    /// PATH stack for directory hierarchy tracking
    /// ExifTool: $$self{PATH} tracks current directory path
    pub(crate) path: Vec<String>,
    /// DataMember storage for tag dependencies
    /// ExifTool: DataMember tags needed by other tags
    pub(crate) data_members: HashMap<String, DataMemberValue>,
    /// Current base offset for pointer calculations
    /// ExifTool: $$dirInfo{Base} + $$self{BASE}
    pub(crate) base: u64,
    /// Processor dispatch configuration
    /// ExifTool: PROCESS_PROC system for different directory types
    pub(crate) processor_dispatch: ProcessorDispatch,
    /// Computed composite tag values
    /// Milestone 8f: Infrastructure for composite tag computation
    pub(crate) composite_tags: HashMap<String, TagValue>,
    /// Original file type from detection (e.g., "NEF")
    pub(crate) original_file_type: Option<String>,
    /// Overridden file type based on content (e.g., "NRW")
    pub(crate) overridden_file_type: Option<String>,
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
            original_file_type: None,
            overridden_file_type: None,
        }
    }

    /// Set the original file type from detection
    pub fn set_file_type(&mut self, file_type: String) {
        self.original_file_type = Some(file_type);
    }

    /// Get the overridden file type if any
    pub fn get_overridden_file_type(&self) -> Option<String> {
        self.overridden_file_type.clone()
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

    /// Get extracted tag by ID
    pub fn get_tag_by_id(&self, tag_id: u16) -> Option<&TagValue> {
        self.extracted_tags.get(&tag_id)
    }

    /// Get all extracted tags with their names (conversions already applied during extraction)
    /// Returns tags with group prefixes (e.g., "EXIF:Make", "GPS:GPSLatitude", "Composite:ImageSize")
    /// matching ExifTool's -G mode behavior
    /// Milestone 8f: Now includes composite tags with "Composite:" prefix
    pub fn get_all_tags(&self) -> HashMap<String, TagValue> {
        use crate::generated::TAG_BY_ID;
        use crate::implementations::canon;

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
            let base_tag_name = if tag_id >= 0xC000 {
                // Only check Canon names for synthetic Canon tag IDs
                canon::get_canon_tag_name(tag_id).unwrap_or_else(|| format!("Tag_{tag_id:04X}"))
            } else {
                // Regular EXIF tags - use main tag table
                TAG_BY_ID
                    .get(&(tag_id as u32))
                    .map(|tag_def| tag_def.name.to_string())
                    .unwrap_or_else(|| format!("Tag_{tag_id:04X}"))
            };

            // Format with group prefix
            let tag_name = format!("{group_name}:{base_tag_name}");

            debug!(
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
            debug!("Composite tag: {} -> {:?}", tag_name, value);
            result.insert(tag_name.clone(), value.clone());
        }

        result
    }

    /// Get all tags as TagEntry objects with both value and print representations
    /// This is the new API that returns both ValueConv and PrintConv results
    /// Milestone 8b: TagEntry API implementation
    pub fn get_all_tag_entries(&mut self) -> Vec<crate::types::TagEntry> {
        use crate::generated::{COMPOSITE_TAGS, TAG_BY_ID};
        use crate::implementations::canon;
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
            let (base_tag_name, tag_def) = if tag_id >= 0xC000 {
                // Canon-specific synthetic tag IDs - no definition in main table
                let canon_tag_name = canon::get_canon_tag_name(tag_id)
                    .unwrap_or_else(|| format!("Tag_{tag_id:04X}"));
                (canon_tag_name, None)
            } else {
                // Check if this tag should be looked up in the global table based on source context
                // ExifTool: lib/Image/ExifTool/Exif.pm:6375 uses context-specific tag tables
                // to prevent maker note tags from being interpreted as GPS/EXIF tags
                let should_lookup_global = source_info.is_none_or(|info| {
                    // Only lookup in global table if the source IFD matches the tag's expected context
                    // ExifTool: Different IFDs use different tag tables (GPS.pm:119-126 vs Canon.pm:1216-1220)
                    // This prevents tag ID conflicts like 0x6 = GPSAltitude vs CanonImageType
                    match info.ifd_name.as_str() {
                        name if name.starts_with("Canon") => false, // Canon maker notes - don't lookup GPS/EXIF tags
                        name if name.starts_with("Nikon") => false, // Nikon maker notes - don't lookup GPS/EXIF tags
                        _ => true, // Standard IFDs (IFD0, ExifIFD, GPS, etc.) - lookup in global table
                    }
                });

                if should_lookup_global {
                    // Regular EXIF tags - look up in unified table
                    // ExifTool: lib/Image/ExifTool/ExifTool.pm:9026 $$tagTablePtr{$tagID}
                    let tag_def = TAG_BY_ID.get(&(tag_id as u32)).copied();
                    let name = tag_def
                        .map(|def| def.name.to_string())
                        .unwrap_or_else(|| format!("Tag_{tag_id:04X}"));
                    (name, tag_def)
                } else {
                    // Maker note tags - don't lookup in global table to avoid conflicts
                    // ExifTool: lib/Image/ExifTool/Exif.pm:6190-6191 inMakerNotes context detection
                    (format!("Tag_{tag_id:04X}"), None)
                }
            };

            // Apply conversions to get both value and print
            let (value, print) = self.apply_conversions(raw_value, tag_def);

            // Get group1 value using TagSourceInfo
            let group1_name = if let Some(source_info) = source_info {
                source_info.get_group1()
            } else {
                "IFD0".to_string() // Default fallback
            };

            let entry = TagEntry {
                group: group_name.to_string(),
                group1: group1_name,
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
                    group1: "Composite".to_string(),
                    name: name.to_string(),
                    value,
                    print,
                };

                entries.push(entry);
            } else {
                // Fallback if definition not found
                let entry = TagEntry {
                    group: "Composite".to_string(),
                    group1: "Composite".to_string(),
                    name: name.to_string(),
                    value: raw_value.clone(),
                    print: raw_value.clone(),
                };

                entries.push(entry);
            }
        }

        entries
    }

    /// Get parsing warnings
    pub fn get_warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Get TIFF header information
    pub fn get_header(&self) -> Option<&TiffHeader> {
        self.header.as_ref()
    }

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

    /// Test helper: Get data length (public for integration tests)
    pub fn get_data_len(&self) -> usize {
        self.data.len()
    }

    /// Get access to the raw EXIF data (public for Canon module)
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    /// Test helper: Add extracted tag with source info for testing
    /// Only available when the 'test-helpers' feature is enabled
    /// DO NOT USE in production code - only for tests
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn add_test_tag(&mut self, tag_id: u16, value: TagValue, namespace: &str, ifd_name: &str) {
        use crate::types::TagSourceInfo;
        self.extracted_tags.insert(tag_id, value);
        self.tag_sources.insert(
            tag_id,
            TagSourceInfo::new(
                namespace.to_string(),
                ifd_name.to_string(),
                "Exif".to_string(),
            ),
        );
    }
}

impl Default for ExifReader {
    fn default() -> Self {
        Self::new()
    }
}
