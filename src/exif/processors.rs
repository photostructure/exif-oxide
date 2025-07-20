//! Processor dispatch and selection logic
//!
//! This module handles the dynamic processor selection and dispatch system
//! that routes different directory types to appropriate processing functions.
//!
//! ExifTool Reference: PROCESS_PROC system and ProcessDirectory dispatch

use crate::implementations::{canon, nikon, olympus, sony};
use crate::processor_registry::{get_global_registry, ProcessorContext};
use crate::types::{DirectoryInfo, Result};
use std::collections::HashMap;
use tracing::{debug, trace, warn};

use super::ExifReader;

impl ExifReader {
    /// Select appropriate processor for a directory
    /// ExifTool: $$subdir{ProcessProc} || $$tagTablePtr{PROCESS_PROC} || \&ProcessExif
    /// Phase 5: Simplified to return processor name string
    pub fn select_processor(&self, dir_name: &str, tag_id: Option<u16>) -> String {
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
    /// Phase 5: Simplified to return processor name strings
    pub(crate) fn select_processor_with_conditions(
        &self,
        dir_name: &str,
        tag_id: Option<u16>,
        _data: &[u8],
        _count: u32,
        _format: Option<&str>,
    ) -> (String, HashMap<String, String>) {
        // 1. Check for subdirectory-specific processor override
        if let Some(tag_id) = tag_id {
            if let Some(processor) = self.processor_dispatch.subdirectory_overrides.get(&tag_id) {
                debug!(
                    "Using SubDirectory ProcessProc override for tag {:#x}: {:?}",
                    tag_id, processor
                );
                return (processor.clone(), HashMap::new());
            }
        }

        // 2. Directory-specific defaults (before table-level processor)
        // ExifTool: Some directories have implicit processors
        let dir_specific = match dir_name {
            "GPS" => Some("GPS".to_string()),
            "ExifIFD" | "InteropIFD" => Some("Exif".to_string()),
            "MakerNotes" => {
                // Trust ExifTool: MakerNotes are parsed as standard TIFF IFDs first
                // to discover subdirectory tags like Equipment (0x2010), CameraSettings (0x2020), etc.
                Some("Exif".to_string())
            }
            // Manufacturer-specific subdirectories use manufacturer processors
            _ if dir_name.starts_with("Olympus:")
                || dir_name.starts_with("Canon:")
                || dir_name.starts_with("Nikon:") =>
            {
                None // Let manufacturer processors handle these
            }
            _ => None,
        };

        if let Some(processor) = dir_specific {
            debug!(
                "Using directory-specific processor for {}: {}",
                dir_name, processor
            );
            return (processor, HashMap::new());
        }

        // 3. Final fallback to EXIF
        // Phase 5: Simplified - no table-level processor lookup needed
        debug!("Using default EXIF processor for {}", dir_name);
        ("Exif".to_string(), HashMap::new())
    }

    /// Dispatch to the appropriate processor function
    /// ExifTool: Dynamic function dispatch with no strict 'refs'
    /// Phase 5: Simplified to use string-based processor names
    pub(crate) fn dispatch_processor(
        &mut self,
        processor_name: &str,
        dir_info: &DirectoryInfo,
    ) -> Result<()> {
        self.dispatch_processor_with_params(processor_name.to_string(), dir_info, &HashMap::new())
    }

    /// Dispatch processor with parameters support
    /// ExifTool: Processor dispatch with SubDirectory parameters
    /// Phase 5: Now uses trait-based processor registry
    pub(crate) fn dispatch_processor_with_params(
        &mut self,
        processor: String, // Trust ExifTool: "Exif" means standard IFD parsing
        dir_info: &DirectoryInfo,
        parameters: &HashMap<String, String>,
    ) -> Result<()> {
        debug!(
            "dispatch_processor_with_params called for directory: {} with processor: {}",
            dir_info.name, processor
        );

        // Trust ExifTool: "Exif" processor means standard IFD parsing for standard directories
        // But manufacturer subdirectories like "Olympus:Equipment" should use binary data processors
        if processor == "Exif" && !dir_info.name.contains(":") {
            debug!(
                "Using standard IFD parsing for {} (Trust ExifTool)",
                dir_info.name
            );
            return self.parse_ifd(dir_info.dir_start, &dir_info.name);
        }

        debug!(
            "Dispatching processor for directory {} using processor registry",
            dir_info.name,
        );
        debug!("=== PROCESSOR SELECTION ===");

        // Create ProcessorContext from current ExifReader state
        let context = self.create_processor_context(&dir_info.name, parameters)?;
        debug!(
            "Context: manufacturer={:?}, table={}",
            context.manufacturer, context.table_name
        );

        // Get the global processor registry
        let registry = get_global_registry();
        debug!("Available processors: {}", registry.processor_count());

        // Find the best processor for this context
        if let Some((processor_key, processor)) = registry.find_best_processor(&context) {
            debug!(
                "Selected processor {} for directory {}",
                processor_key, dir_info.name
            );

            // Extract the data for processing
            let data = self.extract_directory_data(dir_info)?;

            // Process the data using the selected processor
            match processor.process_data(&data, &context) {
                Ok(result) => {
                    // === PROCESSOR RESULT ANALYSIS ===
                    debug!("=== PROCESSOR RESULT ANALYSIS ===");
                    debug!("Processor returned {} tags", result.extracted_tags.len());

                    // Merge extracted tags into ExifReader state
                    for (tag_name, tag_value) in result.extracted_tags {
                        debug!("  Raw tag: '{}' = {:?}", tag_name, tag_value);

                        // Convert tag_name to tag_id and store in extracted_tags
                        if let Some(tag_id) = self.resolve_tag_name_to_id(&tag_name) {
                            debug!("    → Resolved to ID: 0x{:04X}", tag_id);
                            self.extracted_tags.insert(tag_id, tag_value.clone());
                            debug!(
                                "Stored tag: {} (0x{:04X}) = {:?}",
                                tag_name, tag_id, tag_value
                            );
                        } else {
                            debug!("    → FAILED to resolve tag name");
                            // For unknown tag names, try to parse as hex if it looks like Tag_XXXX format
                            if let Some(tag_id) = self.parse_hex_tag_name(&tag_name) {
                                self.extracted_tags.insert(tag_id, tag_value.clone());
                                debug!(
                                    "Stored hex tag: {} (0x{:04X}) = {:?}",
                                    tag_name, tag_id, tag_value
                                );
                            } else {
                                // Store manufacturer-specific tags with synthetic IDs to preserve them
                                let synthetic_id = self.generate_synthetic_tag_id(&tag_name);
                                self.extracted_tags.insert(synthetic_id, tag_value.clone());
                                debug!(
                                    "Stored unresolved tag with synthetic ID: {} (0x{:04X}) = {:?}",
                                    tag_name, synthetic_id, tag_value
                                );

                                // Store tag name mapping for output generation
                                self.store_tag_name_mapping(synthetic_id, &tag_name);
                            }
                        }
                    }

                    debug!(
                        "Current extracted_tags count: {}",
                        self.extracted_tags.len()
                    );

                    // Handle warnings
                    for warning in result.warnings {
                        self.warnings.push(warning);
                    }

                    // Process nested processors if any
                    for (next_key, _next_context) in result.next_processors {
                        debug!("Processing nested processor: {}", next_key);
                        // TODO: Recursive processing with new context
                        // This would be implemented when we have more complex processors
                    }

                    Ok(())
                }
                Err(e) => {
                    // TODO: Milestone 20 (Error Classification) - Add to error payload instead of warning
                    warn!(
                        "Processor {} failed for directory {}: {}",
                        processor_key, dir_info.name, e
                    );
                    self.warnings
                        .push(format!("Processor {processor_key} failed: {e}"));

                    // Fall back to existing processing for compatibility
                    self.fallback_to_existing_processing(dir_info)
                }
            }
        } else {
            // No suitable processor found - fall back to existing processing
            debug!(
                "No processor found for directory {}, using fallback",
                dir_info.name
            );
            // TODO: Milestone 20 (Error Classification) - Add to error payload
            self.warnings.push(format!(
                "No processor available for directory: {}",
                dir_info.name
            ));

            self.fallback_to_existing_processing(dir_info)
        }
    }

    /// Process a SubDirectory tag by following the pointer to nested IFD
    /// ExifTool: SubDirectory processing with Start => '$val'
    // TODO: Replace magic numbers with named constants (matches above is_subdirectory_tag function)
    pub(crate) fn process_subdirectory_tag(
        &mut self,
        tag_id: u16,
        offset: u32,
        tag_name: &str,
        size: Option<usize>,
    ) -> Result<()> {
        debug!(
            "process_subdirectory_tag called for tag_id: 0x{:04x}, offset: 0x{:x}, tag_name: {}",
            tag_id, offset, tag_name
        );

        let subdir_name = match tag_id {
            0x8769 => {
                debug!("Matched ExifIFD for tag 0x{:04x}", tag_id);
                "ExifIFD"
            }
            0x8825 => {
                debug!("Matched GPS for tag 0x{:04x}", tag_id);
                "GPS"
            }
            0xA005 => {
                debug!("Matched InteropIFD for tag 0x{:04x}", tag_id);
                "InteropIFD"
            }
            0x927C => {
                debug!("Matched MakerNotes for tag 0x{:04x}", tag_id);
                "MakerNotes"
            }

            // Olympus subdirectory tags - only when in Olympus context
            // ExifTool: lib/Image/ExifTool/Olympus.pm subdirectory definitions
            0x2010 => {
                debug!("Matched Olympus:Equipment for tag 0x{:04x}", tag_id);
                "Olympus:Equipment"
            }
            0x2020 => {
                debug!("Matched Olympus:CameraSettings for tag 0x{:04x}", tag_id);
                "Olympus:CameraSettings"
            }
            0x2030 => {
                debug!("Matched Olympus:RawDevelopment for tag 0x{:04x}", tag_id);
                "Olympus:RawDevelopment"
            }
            0x2031 => {
                debug!("Matched Olympus:RawDev2 for tag 0x{:04x}", tag_id);
                "Olympus:RawDev2"
            }
            0x2040 => {
                debug!("Matched Olympus:ImageProcessing for tag 0x{:04x}", tag_id);
                "Olympus:ImageProcessing"
            }
            0x2050 => {
                debug!("Matched Olympus:FocusInfo for tag 0x{:04x}", tag_id);
                "Olympus:FocusInfo"
            }
            0x3000 => {
                debug!("Matched Olympus:RawInfo for tag 0x{:04x}", tag_id);
                "Olympus:RawInfo"
            }
            0x4000 => {
                debug!("Matched Olympus:MainInfo for tag 0x{:04x}", tag_id);
                "Olympus:MainInfo"
            }
            0x5000 => {
                debug!("Matched Olympus:UnknownInfo for tag 0x{:04x}", tag_id);
                "Olympus:UnknownInfo"
            }

            _ => {
                debug!("Unknown subdirectory tag 0x{:04x}, returning early", tag_id);
                return Ok(());
            }
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
    /// Phase 5: Simplified to return processor name strings
    // TODO: Replace magic numbers with named constants (matches other subdirectory functions)
    pub(crate) fn get_subdirectory_processor_override(&self, tag_id: u16) -> Option<String> {
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
    pub fn configure_processor_dispatch(&mut self, dispatch: crate::types::ProcessorDispatch) {
        self.processor_dispatch = dispatch;
    }

    /// Add SubDirectory processor override
    /// ExifTool: SubDirectory ProcessProc configuration
    /// Phase 5: Simplified to use processor name strings
    pub fn add_subdirectory_override(&mut self, tag_id: u16, processor: String) {
        self.processor_dispatch
            .subdirectory_overrides
            .insert(tag_id, processor);
    }

    /// Detect manufacturer-specific MakerNote processor
    /// ExifTool: lib/Image/ExifTool/MakerNotes.pm conditional dispatch system
    /// Phase 5: Simplified to return processor name string
    fn detect_makernote_processor(&self) -> Option<String> {
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
            return Some("Canon::Main".to_string());
        }

        // ExifTool: lib/Image/ExifTool/MakerNotes.pm:152-163 Nikon detection
        if nikon::detect_nikon_signature(make) {
            debug!("Detected Nikon MakerNote signature: '{}'", make);
            return Some("Nikon::Main".to_string());
        }

        // ExifTool: lib/Image/ExifTool/MakerNotes.pm:1007-1075 Sony detection
        if sony::is_sony_makernote(make, model) {
            debug!("Detected Sony MakerNote (Make field: {})", make);
            return Some("Sony::Main".to_string());
        }

        // ExifTool: lib/Image/ExifTool/MakerNotes.pm:515-533 Olympus detection
        if olympus::is_olympus_makernote(make) {
            debug!("Detected Olympus MakerNote (Make field: {})", make);
            return Some("Olympus::Main".to_string());
        }

        // Return None to fall back to EXIF processor when no manufacturer detected
        debug!("No specific MakerNote processor detected, falling back to EXIF");
        None
    }

    /// Check if a tag ID represents a SubDirectory pointer
    /// ExifTool: SubDirectory tags like ExifIFD (0x8769), GPS (0x8825)
    // TODO: Replace magic numbers with named constants (e.g. EXIF_IFD_TAG = 0x8769) for better readability
    pub(crate) fn is_subdirectory_tag(&self, tag_id: u16) -> bool {
        let result = match tag_id {
            0x8769 => true, // ExifIFD - Camera settings subdirectory
            0x8825 => true, // GPS - GPS information subdirectory
            0xA005 => true, // InteropIFD - Interoperability subdirectory
            0x927C => true, // MakerNotes - Manufacturer-specific data

            // Olympus subdirectory tags (when in Olympus MakerNotes context)
            // ExifTool: lib/Image/ExifTool/Olympus.pm subdirectory definitions
            0x2010 | // Equipment - Camera/lens hardware info
            0x2020 | // CameraSettings - Core camera settings  
            0x2030 | // RawDevelopment - RAW processing parameters
            0x2031 | // RawDev2 - Additional RAW parameters
            0x2040 | // ImageProcessing - Image processing, art filters
            0x2050 | // FocusInfo - Autofocus information
            0x3000 | // RawInfo - RAW file specific info
            0x4000 | // MainInfo - Main Olympus tag table
            0x5000   // UnknownInfo - Unknown/experimental data
            => {
                // ExifTool: Olympus.pm lines 1169-1189 - these are always subdirectories
                // when found in MakerNotes, regardless of Make tag availability
                let in_makernotes = self.path.last() == Some(&"MakerNotes".to_string());
                if in_makernotes {
                    debug!(
                        "is_subdirectory_tag for Olympus tag 0x{:04x} in MakerNotes - always true", 
                        tag_id
                    );
                    true
                } else {
                    // Only treat as subdirectory if we're processing Olympus files
                    let is_olympus_context = self.is_olympus_subdirectory_context();
                    debug!(
                        "is_subdirectory_tag for Olympus tag 0x{:04x} - olympus_context: {}", 
                        tag_id, is_olympus_context
                    );
                    is_olympus_context
                }
            },

            _ => false,
        };

        if tag_id == 0x2010 {
            debug!(
                "is_subdirectory_tag for Equipment tag 0x2010 - returning: {}",
                result
            );
        }

        result
    }

    /// Check if we're currently in Olympus MakerNotes context for subdirectory processing
    /// Used to determine if Olympus-specific subdirectory tags should be processed
    fn is_olympus_subdirectory_context(&self) -> bool {
        // Check if the Make field indicates this is an Olympus camera
        if let Some(make_tag) = self.extracted_tags.get(&0x010F) {
            if let Some(make_str) = make_tag.as_string() {
                let is_olympus = olympus::is_olympus_makernote(make_str);
                debug!(
                    "is_olympus_subdirectory_context - Make: '{}', is_olympus: {}",
                    make_str, is_olympus
                );
                return is_olympus;
            }
        }
        debug!("is_olympus_subdirectory_context - No Make tag found, returning false");
        false
    }

    /// Create ProcessorContext from current ExifReader state
    /// This bridges the gap between ExifReader's internal state and the processor system
    fn create_processor_context(
        &self,
        table_name: &str,
        parameters: &HashMap<String, String>,
    ) -> Result<ProcessorContext> {
        // Extract manufacturer info from current tags
        let manufacturer = self
            .extracted_tags
            .get(&0x010F) // Make tag
            .and_then(|v| v.as_string());

        let model = self
            .extracted_tags
            .get(&0x0110) // Model tag
            .and_then(|v| v.as_string());

        let firmware = self
            .extracted_tags
            .get(&0x0131) // Software tag (often contains firmware)
            .and_then(|v| v.as_string());

        // Detect file format - simplified for now
        // TODO: Pass actual file format from parsing context
        let file_format = crate::formats::FileFormat::Jpeg; // Default assumption

        // Create the context
        let mut context = ProcessorContext::new(file_format, table_name.to_string());

        if let Some(manufacturer) = manufacturer {
            context = context.with_manufacturer(manufacturer.to_string());
        }

        if let Some(model) = model {
            context = context.with_model(model.to_string());
        }

        if let Some(firmware) = firmware {
            context = context.with_firmware(firmware.to_string());
        }

        // Add parameters
        context = context.with_parameters(parameters.clone());

        // Add byte order from TIFF header if available
        if let Some(header) = &self.header {
            context = context.with_byte_order(header.byte_order);
        }

        // Add current offset context
        context = context.with_data_offset(self.base as usize);

        // Add parent tags for context (simplified - just the extracted tags)
        let mut parent_tags = HashMap::new();
        for (&tag_id, tag_value) in &self.extracted_tags {
            let tag_name = format!("Tag_{tag_id:04X}");
            parent_tags.insert(tag_name, tag_value.clone());
        }
        context = context.with_parent_tags(parent_tags);

        Ok(context)
    }

    /// Extract directory data for processor input
    /// This gets the binary data that the processor will analyze
    fn extract_directory_data(&self, dir_info: &DirectoryInfo) -> Result<Vec<u8>> {
        let start = dir_info.dir_start;
        let end = if dir_info.dir_len > 0 {
            start + dir_info.dir_len
        } else {
            // If no explicit length, try to read a reasonable amount
            std::cmp::min(start + 1024, self.data.len()) // Read up to 1KB
        };

        if start >= self.data.len() {
            return Err(crate::types::ExifError::ParseError(format!(
                "Directory start {} beyond data bounds {}",
                start,
                self.data.len()
            )));
        }

        let end = std::cmp::min(end, self.data.len());
        Ok(self.data[start..end].to_vec())
    }

    /// Fall back to existing processing when processor registry fails
    /// This ensures compatibility during the transition period
    fn fallback_to_existing_processing(&mut self, dir_info: &DirectoryInfo) -> Result<()> {
        debug!("Using fallback processing for directory {}", dir_info.name);

        // Use the existing processing logic as fallback
        match dir_info.name.as_str() {
            "MakerNotes" => {
                // Try to detect manufacturer and route accordingly
                let make = self
                    .extracted_tags
                    .get(&0x010F) // Make tag
                    .and_then(|v| v.as_string())
                    .unwrap_or("");

                if canon::detect_canon_signature(make) {
                    canon::process_canon_makernotes(self, dir_info.dir_start, dir_info.dir_len)
                } else if nikon::detect_nikon_signature(make) {
                    nikon::process_nikon_makernotes(self, dir_info.dir_start)
                } else {
                    // Fall back to standard EXIF processing
                    self.process_exif_ifd(dir_info.dir_start, &dir_info.name)
                }
            }
            _ => {
                // Standard EXIF IFD processing for all other directories
                self.process_exif_ifd(dir_info.dir_start, &dir_info.name)
            }
        }
    }

    /// Resolve tag name to tag ID using generated tag tables
    /// This bridges the gap between processor string-based tag names and ExifReader's u16 tag IDs
    fn resolve_tag_name_to_id(&self, tag_name: &str) -> Option<u16> {
        use crate::generated::TAG_BY_NAME;

        // 1. Direct lookup in generated tables
        if let Some(tag_def) = TAG_BY_NAME.get(tag_name) {
            return Some(tag_def.id as u16);
        }

        // 2. Handle "Tag_XXXX" hex format
        if let Some(hex_part) = tag_name.strip_prefix("Tag_") {
            if let Ok(tag_id) = u16::from_str_radix(hex_part, 16) {
                return Some(tag_id);
            }
        }

        // 3. Handle "0xXXXX" hex format
        if let Some(hex_part) = tag_name.strip_prefix("0x") {
            if let Ok(tag_id) = u16::from_str_radix(hex_part, 16) {
                return Some(tag_id);
            }
        }

        // 4. Handle decimal format
        if let Ok(tag_id) = tag_name.parse::<u16>() {
            return Some(tag_id);
        }

        // 5. Handle ExifTool-style group names (e.g., "MakerNotes:SelfTimer" -> "SelfTimer", "EXIF:Make" -> "Make")
        if tag_name.contains(':') {
            let simple_name = tag_name.split(':').next_back().unwrap_or(tag_name);
            if let Some(tag_def) = TAG_BY_NAME.get(simple_name) {
                return Some(tag_def.id as u16);
            }
        }

        debug!("Failed to resolve tag name: '{}'", tag_name);
        None
    }

    /// Parse hex tag names in the format "Tag_XXXX" to tag IDs
    /// This handles cases where processors return generic hex tag names
    fn parse_hex_tag_name(&self, tag_name: &str) -> Option<u16> {
        if let Some(hex_part) = tag_name.strip_prefix("Tag_") {
            if let Ok(tag_id) = u16::from_str_radix(hex_part, 16) {
                return Some(tag_id);
            }
        }
        None
    }

    /// Generate a synthetic tag ID for unresolved tag names
    /// Uses high ID range (0xF000-0xFFFF) to avoid conflicts with standard EXIF tags
    fn generate_synthetic_tag_id(&self, tag_name: &str) -> u16 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        tag_name.hash(&mut hasher);
        let hash = hasher.finish();

        // Map to synthetic ID range 0xF000-0xFFFF
        0xF000 + ((hash as u16) & 0x0FFF)
    }

    /// Store tag name mapping for synthetic IDs
    /// This allows us to reconstruct the original tag names during output generation
    fn store_tag_name_mapping(&mut self, tag_id: u16, tag_name: &str) {
        // TODO: Implement tag name mapping storage
        // For now, just log it - this would be stored in ExifReader state
        debug!("Mapping synthetic ID 0x{:04X} -> '{}'", tag_id, tag_name);
    }

    // Phase 5: Trait-based processor system integrated with fallback compatibility
}
