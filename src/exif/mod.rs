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

// use crate::generated::Canon_pm::main_conditional_tags::{CanonConditionalTags, ConditionalContext}; // TODO: Generate conditional tags
// use crate::generated::FujiFilm_pm::main_model_detection::{
//     ConditionalContext as FujiFilmConditionalContext, FujiFilmModelDetection,
// }; // TODO: Generate FujiFilm model detection
use crate::tiff_types::TiffHeader;
use crate::types::{
    DataMemberValue, DirectoryInfo, ExifError, ProcessorDispatch, Result, TagSourceInfo, TagValue,
};
use std::collections::HashMap;
use tracing::{debug, trace};

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
    /// Track the original MakerNotes offset for subdirectory calculations
    /// ExifTool: Subdirectory offsets within MakerNotes are relative to the original file position
    pub(crate) maker_notes_original_offset: Option<usize>,
    /// Computed composite tag values
    /// Milestone 8f: Infrastructure for composite tag computation
    pub(crate) composite_tags: HashMap<String, TagValue>,
    /// Original file type from detection (e.g., "NEF")
    pub(crate) original_file_type: Option<String>,
    /// Overridden file type based on content (e.g., "NRW")
    pub(crate) overridden_file_type: Option<String>,
    /// Mapping from synthetic tag IDs to their original tag names
    /// Used for Canon binary data tags that use synthetic IDs in the 0xC000 range
    pub(crate) synthetic_tag_names: HashMap<u16, String>,
}

impl ExifReader {
    /// Get current base offset for pointer calculations
    /// ExifTool: $$dirInfo{Base} + $$self{BASE}  
    pub(crate) fn get_base(&self) -> u64 {
        self.base
    }
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
            maker_notes_original_offset: None,
            composite_tags: HashMap::new(),
            original_file_type: None,
            overridden_file_type: None,
            synthetic_tag_names: HashMap::new(),
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

    /// Get the original file type from detection
    pub fn get_original_file_type(&self) -> Option<String> {
        self.original_file_type.clone()
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

    /// Lookup tag name by source information to resolve tag ID conflicts
    /// (e.g., tag 0x0002 is GPSLatitude in GPS IFD, InteropVersion in InteropIFD)
    fn lookup_tag_name_by_source(
        &self,
        tag_id: u16,
        source_info: Option<&TagSourceInfo>,
    ) -> String {
        use crate::generated::Exif_pm::tag_kit::EXIF_PM_TAG_KITS;
        use crate::generated::GPS_pm::tag_kit::GPS_PM_TAG_KITS;

        // Check if this tag originated from GPS IFD
        if let Some(source) = source_info {
            if source.ifd_name == "GPS" {
                // For GPS IFD tags, check GPS tag kit first
                if let Some(tag_def) = GPS_PM_TAG_KITS.get(&(tag_id as u32)) {
                    return tag_def.name.to_string();
                }
            }
        }

        // For all other IFDs, check EXIF tag kit first, then GPS as fallback
        EXIF_PM_TAG_KITS
            .get(&(tag_id as u32))
            .map(|tag_def| tag_def.name.to_string())
            .or_else(|| {
                GPS_PM_TAG_KITS
                    .get(&(tag_id as u32))
                    .map(|tag_def| tag_def.name.to_string())
            })
            .unwrap_or_else(|| format!("Tag_{tag_id:04X}"))
    }

    /// Get all extracted tags with their names (conversions already applied during extraction)
    /// Returns tags with group prefixes (e.g., "EXIF:Make", "GPS:GPSLatitude", "Composite:ImageSize")
    /// matching ExifTool's -G mode behavior
    /// Milestone 8f: Now includes composite tags with "Composite:" prefix
    pub fn get_all_tags(&self) -> HashMap<String, TagValue> {
        use crate::generated::Exif_pm::tag_kit::EXIF_PM_TAG_KITS;
        use crate::generated::GPS_pm::tag_kit::GPS_PM_TAG_KITS;
        use crate::implementations::canon;
        use crate::implementations::sony;

        let mut result = HashMap::new();

        // Add extracted tags
        for (&tag_id, value) in &self.extracted_tags {
            // Get the enhanced source info for this tag
            let source_info = self.tag_sources.get(&tag_id);

            // Look up tag name using format-specific tables when appropriate
            let (group_name, base_tag_name) = if tag_id >= 0xC000 {
                // Check synthetic tag names mapping first for Canon binary data tags
                if let Some(synthetic_name) = self.synthetic_tag_names.get(&tag_id) {
                    // Parse the full "Group:TagName" format from synthetic_tag_names
                    if let Some((group_part, name_part)) = synthetic_name.split_once(':') {
                        (group_part.to_string(), name_part.to_string())
                    } else {
                        // No group prefix, use source info or default
                        let group = if let Some(source_info) = source_info {
                            source_info.namespace.clone()
                        } else {
                            "EXIF".to_string()
                        };
                        (group, synthetic_name.clone())
                    }
                } else {
                    // Fall back to manufacturer-specific names for other synthetic IDs
                    let group = if let Some(source_info) = source_info {
                        source_info.namespace.clone()
                    } else {
                        "EXIF".to_string()
                    };

                    let tag_name = if let Some(source_info) = source_info {
                        match source_info.namespace.as_str() {
                            "Canon" => canon::get_canon_tag_name(tag_id)
                                .unwrap_or_else(|| format!("Tag_{tag_id:04X}")),
                            "Sony" => sony::get_sony_tag_name(tag_id)
                                .unwrap_or_else(|| format!("Tag_{tag_id:04X}")),
                            _ => format!("Tag_{tag_id:04X}"),
                        }
                    } else {
                        // No source info, try Canon as fallback (historical behavior)
                        canon::get_canon_tag_name(tag_id)
                            .unwrap_or_else(|| format!("Tag_{tag_id:04X}"))
                    };

                    (group, tag_name)
                }
            } else {
                // Use namespace from TagSourceInfo or default to EXIF for non-synthetic tags
                let group = if let Some(source_info) = source_info {
                    source_info.namespace.clone()
                } else {
                    "EXIF".to_string()
                };

                // Check for RAW format-specific tag names
                // ExifTool: Uses format-specific tag tables (e.g., PanasonicRaw::Main for RW2 files)
                let tag_name = if let Some(file_type) = &self.original_file_type {
                    if file_type == "RW2" && group == "EXIF" {
                        // Use Panasonic-specific tag definitions for RW2 files
                        // ExifTool: PanasonicRaw.pm %Image::ExifTool::PanasonicRaw::Main hash
                        if let Some(panasonic_name) =
                            crate::raw::formats::panasonic::get_panasonic_tag_name(tag_id)
                        {
                            panasonic_name.to_string()
                        } else {
                            // Fall through to standard lookup if not a known Panasonic tag
                            EXIF_PM_TAG_KITS
                                .get(&(tag_id as u32))
                                .map(|tag_def| tag_def.name.to_string())
                                .or_else(|| {
                                    GPS_PM_TAG_KITS
                                        .get(&(tag_id as u32))
                                        .map(|tag_def| tag_def.name.to_string())
                                })
                                .unwrap_or_else(|| format!("Tag_{tag_id:04X}"))
                        }
                    } else {
                        // Regular EXIF tags for other formats - use source-aware tag table lookup
                        self.lookup_tag_name_by_source(tag_id, source_info)
                    }
                } else {
                    // No file type information - use source-aware lookup
                    self.lookup_tag_name_by_source(tag_id, source_info)
                };

                (group, tag_name)
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

            // Debug: Check for tag ID conflicts specifically for 0x0002
            if tag_id == 0x0002 {
                debug!("DEBUG: Tag 0x0002 final output - IFD: {}, Tag Name: {}, Group: {}, Value: {:?}", 
                    source_info.map(|s| s.ifd_name.as_str()).unwrap_or("unknown"),
                    base_tag_name, group_name, value);
            }

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
        use crate::generated::Exif_pm::tag_kit::EXIF_PM_TAG_KITS;
        use crate::generated::GPS_pm::tag_kit::GPS_PM_TAG_KITS;
        use crate::generated::COMPOSITE_TAGS;
        use crate::implementations::canon;
        use crate::implementations::sony;
        use crate::types::TagEntry;

        let mut entries = Vec::new();

        // Process extracted tags
        for (&tag_id, raw_value) in &self.extracted_tags {
            // Get the enhanced source info for this tag
            let source_info = self.tag_sources.get(&tag_id);

            // Look up tag name and group, handling synthetic tags properly
            let (group_name, base_tag_name, _tag_def) = if tag_id >= 0xC000 {
                // Check synthetic tag names mapping first for Canon binary data tags
                if let Some(synthetic_name) = self.synthetic_tag_names.get(&tag_id) {
                    // Parse the full "Group:TagName" format from synthetic_tag_names
                    if let Some((group_part, name_part)) = synthetic_name.split_once(':') {
                        (group_part, name_part.to_string(), None::<()>)
                    } else {
                        // No group prefix, use source info or default
                        let group = if let Some(source_info) = source_info {
                            source_info.namespace.as_str()
                        } else {
                            "EXIF"
                        };
                        (group, synthetic_name.clone(), None)
                    }
                } else {
                    // Canon-specific synthetic tag IDs - try conditional resolution first
                    // For binary data, pass count as data length for count-based conditions
                    let count = match raw_value {
                        TagValue::Binary(data) => Some(data.len() as u32),
                        _ => None,
                    };
                    if let Some(conditional_name) =
                        self.resolve_conditional_tag_name(tag_id, count, None, None)
                    {
                        let group = if let Some(source_info) = source_info {
                            source_info.namespace.as_str()
                        } else {
                            "EXIF"
                        };
                        (group, conditional_name, None)
                    } else {
                        // Fall back to manufacturer-specific tag names
                        let group = if let Some(source_info) = source_info {
                            source_info.namespace.as_str()
                        } else {
                            "EXIF"
                        };

                        let tag_name = if let Some(source_info) = source_info {
                            match source_info.namespace.as_str() {
                                "Canon" => canon::get_canon_tag_name(tag_id)
                                    .unwrap_or_else(|| format!("Tag_{tag_id:04X}")),
                                "Sony" => sony::get_sony_tag_name(tag_id)
                                    .unwrap_or_else(|| format!("Tag_{tag_id:04X}")),
                                _ => format!("Tag_{tag_id:04X}"),
                            }
                        } else {
                            // No source info, try Canon as fallback (historical behavior)
                            canon::get_canon_tag_name(tag_id)
                                .unwrap_or_else(|| format!("Tag_{tag_id:04X}"))
                        };

                        (group, tag_name, None)
                    }
                }
            } else {
                // Use namespace from TagSourceInfo or default to EXIF for regular tags
                let group = if let Some(source_info) = source_info {
                    source_info.namespace.as_str()
                } else {
                    "EXIF" // Default fallback
                };
                let (name, def) = {
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
                            name if name.starts_with("Olympus") => false, // Olympus maker notes - don't lookup GPS/EXIF tags
                            "MakerNotes" => false, // Generic maker notes - don't lookup GPS/EXIF tags
                            "KyoceraRaw" => false, // Kyocera RAW - don't lookup GPS/EXIF tags
                            "IFD0" if self.original_file_type.as_deref() == Some("RW2") => {
                                // TODO: HANDOFF TASK - Complete tag precedence fix
                                // See: docs/milestones/HANDOFF-panasonic-rw2-tag-mapping-completion.md
                                // For Panasonic RW2, only exclude Panasonic-specific tag ranges
                                // ExifTool: PanasonicRaw.pm Main table covers 0x01-0x2F range
                                // Allow standard EXIF tags (Make=0x10F, Model=0x110, ColorSpace=0xA001, etc.)
                                !(0x01..=0x2F).contains(&tag_id) // Allow standard EXIF tags, exclude Panasonic-specific
                            }
                            _ => true, // Standard IFDs (IFD0, ExifIFD, GPS, etc.) - lookup in global table
                        }
                    });

                    if should_lookup_global {
                        // Regular EXIF tags - try conditional resolution first for Canon tags
                        // For binary data, pass count as data length for count-based conditions
                        let count = match raw_value {
                            TagValue::Binary(data) => Some(data.len() as u32),
                            _ => None,
                        };
                        if let Some(conditional_name) =
                            self.resolve_conditional_tag_name(tag_id, count, None, None)
                        {
                            (conditional_name, None)
                        } else {
                            // Context-aware tag lookup to handle IFD-specific tag collisions
                            // ExifTool: lib/Image/ExifTool/Exif.pm:8968 GPS vs InteropIFD conflict handling
                            let (name, _tag_def) = if let Some(source_info) = source_info {
                                match (source_info.ifd_name.as_str(), tag_id) {
                                    // InteropIFD-specific tags
                                    // ExifTool: lib/Image/ExifTool/Exif.pm InteropIFD table
                                    ("InteropIFD", 0x0001) => {
                                        ("InteroperabilityIndex".to_string(), None::<()>)
                                    }
                                    ("InteropIFD", 0x0002) => {
                                        ("InteroperabilityVersion".to_string(), None::<()>)
                                    }

                                    // GPS IFD tags - check GPS tag kit first to avoid conflicts
                                    // (e.g., tag 0x0002 is GPSLatitude in GPS IFD, InteropVersion in InteropIFD)
                                    ("GPS", _) => {
                                        let name = GPS_PM_TAG_KITS
                                            .get(&(tag_id as u32))
                                            .map(|def| def.name.to_string())
                                            .or_else(|| {
                                                EXIF_PM_TAG_KITS
                                                    .get(&(tag_id as u32))
                                                    .map(|def| def.name.to_string())
                                            })
                                            .unwrap_or_else(|| format!("Tag_{tag_id:04X}"));
                                        (name, None)
                                    }

                                    // All other contexts use global lookup
                                    _ => {
                                        let name = EXIF_PM_TAG_KITS
                                            .get(&(tag_id as u32))
                                            .map(|def| def.name.to_string())
                                            .or_else(|| {
                                                GPS_PM_TAG_KITS
                                                    .get(&(tag_id as u32))
                                                    .map(|def| def.name.to_string())
                                            })
                                            .unwrap_or_else(|| format!("Tag_{tag_id:04X}"));
                                        (name, None)
                                    }
                                }
                            } else {
                                // No context info - fall back to global lookup
                                let name = EXIF_PM_TAG_KITS
                                    .get(&(tag_id as u32))
                                    .map(|def| def.name.to_string())
                                    .or_else(|| {
                                        GPS_PM_TAG_KITS
                                            .get(&(tag_id as u32))
                                            .map(|def| def.name.to_string())
                                    })
                                    .unwrap_or_else(|| format!("Tag_{tag_id:04X}"));
                                (name, None)
                            };

                            // Debug logging for ColorSpace and WhiteBalance
                            if tag_id == 0xa001 || tag_id == 0xa403 {
                                debug!(
                                "Processing tag 0x{:04x} ({}) with value {:?}, tag_def found: {}",
                                tag_id,
                                name,
                                raw_value,
                                false
                            );
                            }

                            (name, None)
                        }
                    } else {
                        // Maker note tags - don't lookup in global table to avoid conflicts
                        // ExifTool: lib/Image/ExifTool/Exif.pm:6190-6191 inMakerNotes context detection

                        // Check for RAW format-specific tags
                        if let Some(source_info) = source_info {
                            if source_info.ifd_name == "KyoceraRaw" {
                                // Use Kyocera-specific tag name lookup
                                let kyocera_tag_name = crate::raw::get_kyocera_tag_name(tag_id)
                                    .map(|name| name.to_string())
                                    .unwrap_or_else(|| format!("Tag_{tag_id:04X}"));
                                (kyocera_tag_name, None)
                            } else if source_info.ifd_name == "IFD0"
                                && self.original_file_type.as_deref() == Some("RW2")
                            {
                                // Use Panasonic RW2-specific tag name lookup
                                // ExifTool: PanasonicRaw.pm Main table for RW2 IFD0
                                let panasonic_tag_name =
                                    crate::raw::formats::panasonic::get_panasonic_tag_name(tag_id)
                                        .map(|name| name.to_string())
                                        .unwrap_or_else(|| format!("Tag_{tag_id:04X}"));
                                (panasonic_tag_name, None)
                            } else {
                                // Check for manufacturer-specific maker note tags
                                if source_info.ifd_name.starts_with("Canon")
                                    || source_info.ifd_name == "MakerNotes"
                                {
                                    // Use Canon-specific tag name lookup for Canon maker note tags
                                    let canon_tag_name = canon::get_canon_tag_name(tag_id)
                                        .unwrap_or_else(|| format!("Tag_{tag_id:04X}"));
                                    (canon_tag_name, None)
                                } else if source_info.ifd_name.starts_with("Sony") {
                                    // Use Sony-specific tag name lookup for Sony maker note tags
                                    let sony_tag_name = sony::get_sony_tag_name(tag_id)
                                        .unwrap_or_else(|| format!("Tag_{tag_id:04X}"));
                                    (sony_tag_name, None)
                                } else {
                                    // Other maker note tags
                                    (format!("Tag_{tag_id:04X}"), None)
                                }
                            }
                        } else {
                            (format!("Tag_{tag_id:04X}"), None)
                        }
                    }
                };
                (group, name, def)
            };

            // Apply conversions to get both value and print
            let ifd_name = source_info
                .as_ref()
                .map(|s| s.ifd_name.as_str())
                .unwrap_or("");
            let (value, print) = self.apply_conversions(raw_value, tag_id, ifd_name);

            // Get group1 value using TagSourceInfo with special handling for IFD pointer tags
            let group1_name = if let Some(source_info) = source_info {
                // Special case: GPSInfo tag (0x8825) should have group1='GPS' even when in IFD0
                // ExifTool: GPS IFD pointer tags belong to GPS group logically
                if tag_id == 0x8825 {
                    "GPS".to_string()
                } else {
                    source_info.get_group1()
                }
            } else {
                "IFD0".to_string() // Default fallback
            };

            // Debug logging for ColorSpace and WhiteBalance
            if tag_id == 0xa001 || tag_id == 0xa403 {
                debug!("Creating TagEntry for 0x{:04x}: group={}, group1={}, name={}, value={:?}, print={:?}", 
                    tag_id, group_name, group1_name, base_tag_name, value, print);
            }

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

    /// Create ConditionalContext for conditional tag resolution
    fn create_conditional_context(
        &self,
        count: Option<u32>,
        format: Option<String>,
        binary_data: Option<Vec<u8>>,
    ) -> crate::generated::Canon_pm::main_conditional_tags::ConditionalContext {
        let make = self
            .extracted_tags
            .get(&0x010F)
            .and_then(|v| v.as_string())
            .map(|s| s.to_string());
        let model = self
            .extracted_tags
            .get(&0x0110)
            .and_then(|v| v.as_string())
            .map(|s| s.to_string());
        crate::generated::Canon_pm::main_conditional_tags::ConditionalContext {
            make,
            model,
            count,
            format,
            binary_data,
        }
    }

    /// Create ConditionalContext for FujiFilm conditional tag resolution
    fn create_fujifilm_conditional_context(
        &self,
        count: Option<u32>,
        format: Option<String>,
    ) -> crate::generated::FujiFilm_pm::main_model_detection::ConditionalContext {
        let make = self
            .extracted_tags
            .get(&0x010F)
            .and_then(|v| v.as_string())
            .map(|s| s.to_string());
        let model = self
            .extracted_tags
            .get(&0x0110)
            .and_then(|v| v.as_string())
            .map(|s| s.to_string());
        crate::generated::FujiFilm_pm::main_model_detection::ConditionalContext {
            make,
            model,
            count,
            format,
        }
    }

    /// Resolve conditional tag name using manufacturer-specific logic
    /// Returns the resolved tag name if conditional resolution succeeds, None otherwise
    fn resolve_conditional_tag_name(
        &self,
        tag_id: u16,
        count: Option<u32>,
        format: Option<String>,
        binary_data: Option<Vec<u8>>,
    ) -> Option<String> {
        // Only perform conditional resolution for Canon tags when we have Canon context
        if let Some(make) = self.extracted_tags.get(&0x010F).and_then(|v| v.as_string()) {
            if make.contains("Canon") {
                let context = self.create_conditional_context(count, format, binary_data);
                let canon_resolver =
                    crate::generated::Canon_pm::main_conditional_tags::CanonConditionalTags::new();

                if let Some(resolved) = canon_resolver.resolve_tag(&tag_id.to_string(), &context) {
                    trace!(
                        "Conditional tag resolution: 0x{:04x} -> {} (subdirectory: {}, writable: {})",
                        tag_id, resolved.name, resolved.subdirectory, resolved.writable
                    );
                    return Some(resolved.name);
                }
            } else if make.contains("FUJIFILM") {
                let context = self.create_fujifilm_conditional_context(count, format);
                let model = self
                    .extracted_tags
                    .get(&0x0110) // Model tag
                    .and_then(|v| v.as_string())
                    .unwrap_or("")
                    .to_string();
                let fujifilm_resolver = crate::generated::FujiFilm_pm::main_model_detection::FujiFilmModelDetection::new(model);

                if let Some(resolved_name) =
                    fujifilm_resolver.resolve_conditional_tag(&tag_id.to_string(), &context)
                {
                    trace!(
                        "FujiFilm conditional tag resolution: 0x{:04x} -> {}",
                        tag_id,
                        resolved_name
                    );
                    return Some(resolved_name.to_string());
                }
            }
        }

        None
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
