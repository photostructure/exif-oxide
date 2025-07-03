//! Processor dispatch and selection logic
//!
//! This module handles the dynamic processor selection and dispatch system
//! that routes different directory types to appropriate processing functions.
//!
//! ExifTool Reference: PROCESS_PROC system and ProcessDirectory dispatch

use crate::implementations::{canon, sony};
use crate::tiff_types::IfdEntry;
use crate::types::{DirectoryInfo, ExifError, ProcessorType, Result, SonyProcessor, TagSourceInfo};
use std::collections::HashMap;
use tracing::{debug, trace, warn};

use super::ExifReader;

impl ExifReader {
    /// Select appropriate processor for a directory
    /// ExifTool: $$subdir{ProcessProc} || $$tagTablePtr{PROCESS_PROC} || \&ProcessExif
    pub fn select_processor(&self, dir_name: &str, tag_id: Option<u16>) -> ProcessorType {
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
    pub(crate) fn select_processor_with_conditions(
        &self,
        dir_name: &str,
        tag_id: Option<u16>,
        data: &[u8],
        count: u32,
        format: Option<&str>,
    ) -> (ProcessorType, HashMap<String, String>) {
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
                return (processor.clone(), HashMap::new());
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
            return (processor, HashMap::new());
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
        (ProcessorType::Exif, HashMap::new())
    }

    /// Dispatch to the appropriate processor function
    /// ExifTool: Dynamic function dispatch with no strict 'refs'
    pub(crate) fn dispatch_processor(
        &mut self,
        processor: ProcessorType,
        dir_info: &DirectoryInfo,
    ) -> Result<()> {
        self.dispatch_processor_with_params(processor, dir_info, &HashMap::new())
    }

    /// Dispatch processor with parameters support
    /// ExifTool: Processor dispatch with SubDirectory parameters
    pub(crate) fn dispatch_processor_with_params(
        &mut self,
        processor: ProcessorType,
        dir_info: &DirectoryInfo,
        parameters: &HashMap<String, String>,
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
                            canon::process_canon_makernotes(
                                self,
                                dir_info.dir_start,
                                dir_info.dir_len,
                            )
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
    pub(crate) fn process_subdirectory_tag(
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
    pub(crate) fn get_subdirectory_processor_override(&self, tag_id: u16) -> Option<ProcessorType> {
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
    pub fn add_subdirectory_override(&mut self, tag_id: u16, processor: ProcessorType) {
        self.processor_dispatch
            .subdirectory_overrides
            .insert(tag_id, processor);
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
            canon::process_canon_makernotes(self, dir_info.dir_start, size)?;
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
    pub(crate) fn process_exif_ifd_with_namespace(
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

    /// Check if a tag ID represents a SubDirectory pointer
    /// ExifTool: SubDirectory tags like ExifIFD (0x8769), GPS (0x8825)
    // TODO: Replace magic numbers with named constants (e.g. EXIF_IFD_TAG = 0x8769) for better readability
    pub(crate) fn is_subdirectory_tag(&self, tag_id: u16) -> bool {
        match tag_id {
            0x8769 => true, // ExifIFD - Camera settings subdirectory
            0x8825 => true, // GPS - GPS information subdirectory
            0xA005 => true, // InteropIFD - Interoperability subdirectory
            0x927C => true, // MakerNotes - Manufacturer-specific data
            _ => false,
        }
    }
}
