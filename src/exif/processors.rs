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
                // For MakerNotes, we need manufacturer-specific processing
                // Try to detect the manufacturer from Make tag
                if let Some(processor) = self.detect_makernote_processor() {
                    debug!(
                        "Detected manufacturer-specific processor for MakerNotes: {}",
                        processor
                    );
                    Some(processor)
                } else {
                    // Return None to trigger fallback to existing processing
                    // This allows Canon and other manufacturers to use their direct processing functions
                    debug!(
                        "No processor registry match for MakerNotes, will use fallback processing"
                    );
                    None
                }
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

        // 3. Check processor registry for registered processors
        // This allows manufacturer-specific processors like Canon to be found
        let context = match self.create_processor_context(dir_name, &HashMap::new()) {
            Ok(ctx) => ctx,
            Err(_) => {
                debug!(
                    "Failed to create processor context, using default EXIF processor for {}",
                    dir_name
                );
                return ("Exif".to_string(), HashMap::new());
            }
        };

        let registry = get_global_registry();
        if let Some((processor_key, _)) = registry.find_best_processor(&context) {
            debug!(
                "Found registered processor {} for directory {}",
                processor_key, dir_name
            );
            return (processor_key.to_string(), HashMap::new());
        }

        // 4. Final fallback to EXIF
        debug!(
            "No registered processor found, using default EXIF processor for {}",
            dir_name
        );
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
                            let source_info = self.create_tag_source_info("ProcessedData");
                            self.store_tag_with_precedence(tag_id, tag_value.clone(), source_info);
                            debug!(
                                "Stored tag: {} (0x{:04X}) = {:?}",
                                tag_name, tag_id, tag_value
                            );
                        } else {
                            debug!("    → FAILED to resolve tag name");
                            // For unknown tag names, try to parse as hex if it looks like Tag_XXXX format
                            if let Some(tag_id) = self.parse_hex_tag_name(&tag_name) {
                                let source_info = self.create_tag_source_info("ProcessedData");
                                self.store_tag_with_precedence(
                                    tag_id,
                                    tag_value.clone(),
                                    source_info,
                                );
                                debug!(
                                    "Stored hex tag: {} (0x{:04X}) = {:?}",
                                    tag_name, tag_id, tag_value
                                );
                            } else {
                                // Store manufacturer-specific tags with synthetic IDs to preserve them
                                let synthetic_id = self.generate_synthetic_tag_id(&tag_name);
                                let source_info = self.create_tag_source_info("ProcessedData");
                                self.store_tag_with_precedence(
                                    synthetic_id,
                                    tag_value.clone(),
                                    source_info,
                                );
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
        let result = self.process_subdirectory(&dir_info);
        debug!(
            "Completed process_subdirectory for {} at offset {:#x}, result: {:?}",
            subdir_name,
            offset,
            result.is_ok()
        );
        result
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
    pub(crate) fn detect_makernote_processor(&self) -> Option<String> {
        // Extract Make and Model from current tags for detection
        let make = self
            .get_tag_across_namespaces(0x010F) // Make tag
            .and_then(|v| v.as_string())
            .unwrap_or("");

        let model = self
            .get_tag_across_namespaces(0x0110) // Model tag
            .and_then(|v| v.as_string())
            .unwrap_or("");

        debug!(
            "Detecting MakerNote processor for Make: '{}', Model: '{}'",
            make, model
        );

        // ExifTool: lib/Image/ExifTool/MakerNotes.pm:60-68 Canon detection
        if canon::detect_canon_signature(make) {
            debug!(
                "Detected Canon MakerNote signature - using fallback to direct Canon processing"
            );
            // Return None to force fallback to direct Canon processing
            // This ensures process_canon_makernotes() is called for Main table extraction
            return None;
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

        // ExifTool: lib/Image/ExifTool/MakerNotes.pm:486-494 Minolta detection
        // MakerNoteMinolta condition: $$self{Make}=~/^(Konica Minolta|Minolta)/i
        if make.to_lowercase().starts_with("minolta")
            || make.to_lowercase().starts_with("konica minolta")
        {
            debug!("Detected Minolta MakerNote (Make field: {})", make);
            return Some("Minolta::Main".to_string());
        }

        // ExifTool: lib/Image/ExifTool/MakerNotes.pm:515-533 Olympus detection
        // For Olympus MakerNotes, use standard IFD parsing to discover subdirectories like Equipment (0x2010)
        // ExifTool: Olympus MakerNotes are processed as standard IFD first to find subdirectory tags
        if olympus::is_olympus_makernote(make) {
            debug!("Detected Olympus MakerNote (Make field: {})", make);
            debug!("Using standard IFD parsing for Olympus MakerNotes to discover Equipment subdirectory");
            return Some("Exif".to_string()); // Use standard IFD parsing
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
        if let Some(make_tag) = self.get_tag_across_namespaces(0x010F) {
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
            .get_tag_across_namespaces(0x010F) // Make tag
            .and_then(|v| v.as_string());

        let model = self
            .get_tag_across_namespaces(0x0110) // Model tag
            .and_then(|v| v.as_string());

        let firmware = self
            .get_tag_across_namespaces(0x0131) // Software tag (often contains firmware)
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
        for (&(tag_id, ref namespace), tag_value) in &self.extracted_tags {
            // Use TAG_PREFIX mechanism for consistent naming
            let source_info = self.tag_sources.get(&(tag_id, namespace.clone()));
            let tag_name = Self::generate_tag_prefix_name(tag_id, source_info);
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
        debug!("=== FALLBACK PROCESSING CALLED ===");
        debug!("Using fallback processing for directory {}", dir_info.name);

        // Use the existing processing logic as fallback
        match dir_info.name.as_str() {
            "MakerNotes" => {
                // Try to detect manufacturer and route accordingly
                let make = self
                    .get_tag_across_namespaces(0x010F) // Make tag
                    .and_then(|v| v.as_string())
                    .unwrap_or("");

                if canon::detect_canon_signature(make) {
                    canon::process_canon_makernotes(self, dir_info.dir_start, dir_info.dir_len)
                } else if nikon::detect_nikon_signature(make) {
                    nikon::process_nikon_makernotes(self, dir_info.dir_start)
                } else if sony::is_sony_makernote(make, "") {
                    // Sony MakerNotes processing - call Sony subdirectory processing
                    debug!("Detected Sony MakerNotes for Make: '{}' - calling Sony subdirectory processing", make);
                    debug!(
                        "Sony MakerNotes directory start: {:#x}, length: {}",
                        dir_info.dir_start, dir_info.dir_len
                    );
                    let result = sony::process_sony_subdirectory_tags(self);
                    debug!("Sony subdirectory processing result: {:?}", result.is_ok());
                    if let Err(ref e) = result {
                        debug!("Sony subdirectory processing error: {}", e);
                    }
                    result
                } else if make.to_lowercase().starts_with("minolta")
                    || make.to_lowercase().starts_with("konica minolta")
                {
                    debug!("Processing Minolta MakerNotes using standard IFD parsing");
                    // ExifTool: Minolta MakerNotes are processed as standard IFD
                    // ExifTool: lib/Image/ExifTool/MakerNotes.pm:486-494 MakerNoteMinolta
                    self.process_exif_ifd(dir_info.dir_start, &dir_info.name)
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
    pub(crate) fn resolve_tag_name_to_id(&mut self, tag_name: &str) -> Option<u16> {
        use crate::generated::Exif_pm::main_tags::EXIF_MAIN_TAGS;
        use crate::generated::GPS_pm::main_tags::GPS_MAIN_TAGS;

        // 1. Direct lookup in generated tables
        // Since we don't have BY_NAME maps, search through the tag tables
        for (tag_id, tag_def) in EXIF_MAIN_TAGS.iter() {
            if tag_def.name == tag_name {
                return Some(*tag_id);
            }
        }
        for (tag_id, tag_def) in GPS_MAIN_TAGS.iter() {
            if tag_def.name == tag_name {
                return Some(*tag_id);
            }
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

        // 5. Handle manufacturer-specific binary data tag names
        // These come from ProcessBinaryData processors (Sony, Canon, etc.)
        if self.is_manufacturer_specific_tag(tag_name) {
            if let Some(synthetic_id) = self.assign_synthetic_tag_id(tag_name) {
                return Some(synthetic_id);
            }
        }

        // 6. Handle ExifTool-style group names (e.g., "MakerNotes:SelfTimer" -> "SelfTimer", "EXIF:Make" -> "Make")
        if tag_name.contains(':') {
            let simple_name = tag_name.split(':').next_back().unwrap_or(tag_name);
            for (tag_id, tag_def) in crate::generated::Exif_pm::main_tags::EXIF_MAIN_TAGS.iter() {
                if tag_def.name == simple_name {
                    return Some(*tag_id as u16);
                }
            }
            for (tag_id, tag_def) in crate::generated::GPS_pm::main_tags::GPS_MAIN_TAGS.iter() {
                if tag_def.name == simple_name {
                    return Some(*tag_id as u16);
                }
            }
        }

        debug!("Failed to resolve tag name: '{}'", tag_name);
        None
    }

    /// Check if a tag name is manufacturer-specific (comes from ProcessBinaryData)
    fn is_manufacturer_specific_tag(&self, tag_name: &str) -> bool {
        // Sony-specific binary data tag names
        let sony_tags = [
            "AFType",
            "AFAreaMode",
            "AFPointsInFocus",
            "AFPointSelected",
            "FocusMode",
            "FocusStatus",
            "CameraType",
            "CameraType2",
            "ISOSetting",
            "WhiteBalanceSetting",
        ];

        // Canon-specific binary data tag names (for completeness)
        let canon_tags = [
            "AFPointSelected",
            "AFAreaMode",
            "DriveMode",
            "WhiteBalance",
            "ImageQuality",
            "FlashMode",
            "ContinuousDrive",
            "FocusMode",
        ];

        sony_tags.contains(&tag_name) || canon_tags.contains(&tag_name)
    }

    /// Assign synthetic tag ID for manufacturer-specific tags
    fn assign_synthetic_tag_id(&mut self, tag_name: &str) -> Option<u16> {
        // Generate a synthetic tag ID in the 0xC000-0xEFFF range for binary data tags
        // This matches the range expected by the synthetic tag name resolution logic
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        tag_name.hash(&mut hasher);
        let synthetic_id = (hasher.finish() % 0x2FFF) as u16 + 0xC000;

        // Store the tag name mapping for output generation
        self.store_tag_name_mapping(synthetic_id, tag_name);

        // Store the tag source info with manufacturer namespace
        let namespace = if self.is_sony_tag(tag_name) {
            "Sony"
        } else if self.is_canon_tag(tag_name) {
            "Canon"
        } else {
            "MakerNotes"
        };

        let source_info = crate::types::TagSourceInfo::new(
            namespace.to_string(),
            "MakerNotes".to_string(),
            format!("{namespace}::BinaryData"),
        );
        self.tag_sources
            .insert((synthetic_id, namespace.to_string()), source_info);

        debug!(
            "Assigned synthetic tag ID 0x{:04X} to '{}' with namespace '{}'",
            synthetic_id, tag_name, namespace
        );
        Some(synthetic_id)
    }

    /// Check if tag name is Sony-specific
    fn is_sony_tag(&self, tag_name: &str) -> bool {
        let sony_tags = [
            "AFType",
            "AFAreaMode",
            "AFPointsInFocus",
            "AFPointSelected",
            "FocusMode",
            "FocusStatus",
            "CameraType",
            "CameraType2",
            "ISOSetting",
            "WhiteBalanceSetting",
        ];
        sony_tags.contains(&tag_name)
    }

    /// Check if tag name is Canon-specific
    fn is_canon_tag(&self, tag_name: &str) -> bool {
        let canon_tags = [
            "AFPointSelected",
            "AFAreaMode",
            "DriveMode",
            "WhiteBalance",
            "ImageQuality",
            "FlashMode",
            "ContinuousDrive",
            "FocusMode",
        ];
        canon_tags.contains(&tag_name)
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
        // Determine the namespace for this tag
        let namespace = if self.is_sony_tag(tag_name) {
            "Sony"
        } else if self.is_canon_tag(tag_name) {
            "Canon"
        } else {
            "MakerNotes"
        };

        // Store in the format "Group:TagName" expected by synthetic tag resolution
        // Use defensive grouping to avoid double nesting
        let full_tag_name = crate::utils::ensure_group_prefix(tag_name, namespace);
        self.synthetic_tag_names
            .insert(tag_id, full_tag_name.clone());

        debug!(
            "Mapping synthetic ID 0x{:04X} -> '{}'",
            tag_id, full_tag_name
        );
    }

    // Phase 5: Trait-based processor system integrated with fallback compatibility
}
