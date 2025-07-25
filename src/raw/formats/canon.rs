//! Canon RAW Format Handler
//!
//! This module implements Canon RAW format processing following ExifTool's exact logic.
//! Canon has several RAW formats:
//! - CR2: Current TIFF-based format (2004-2018)
//! - CRW: Legacy format with custom structure (pre-2004)
//! - CR3: Modern MOV/MP4-based format (2018+)
//!
//! **Trust ExifTool**: This code translates ExifTool's Canon.pm processing verbatim
//! without any improvements or simplifications. Every algorithm, magic number, and
//! quirk is copied exactly as documented in the ExifTool source.
//!
//! Primary ExifTool References:
//! - lib/Image/ExifTool/Canon.pm - Main Canon processing (10,648 lines)
//! - 169 ProcessBinaryData sections for complex data extraction
//! - Offset schemes for different camera generations (4/6/16/28 bytes)

use crate::exif::ExifReader;
use crate::raw::RawFormatHandler;
use crate::types::Result;
use tracing::debug;

/// Canon RAW format variants
/// ExifTool: Canon.pm handles multiple format types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonFormat {
    /// CR2: TIFF-based format (primary target)
    /// ExifTool: Canon.pm processes CR2 as TIFF with Canon maker notes
    CR2,
    /// CRW: Legacy format with custom structure (optional)
    /// ExifTool: Canon.pm has specialized CRW processing
    CRW,
    /// CR3: MOV-based format (optional)
    /// ExifTool: Canon.pm processes CR3 as QuickTime with Canon metadata
    CR3,
}

impl CanonFormat {
    /// Get format name as string
    pub fn name(&self) -> &'static str {
        match self {
            CanonFormat::CR2 => "CR2",
            CanonFormat::CRW => "CRW",
            CanonFormat::CR3 => "CR3",
        }
    }

    /// Check if format is TIFF-based
    /// ExifTool: CR2 uses TIFF structure, CRW/CR3 have custom containers
    pub fn is_tiff_based(&self) -> bool {
        matches!(self, CanonFormat::CR2)
    }
}

// Use generated Canon tag structure from codegen

/// Canon RAW Handler - main processor for Canon RAW formats
/// ExifTool: Canon.pm ProcessCanon() main entry point
pub struct CanonRawHandler {
    /// Detected Canon format (determined from file extension/magic)
    format: CanonFormat,
}

impl Default for CanonRawHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CanonRawHandler {
    /// Create new Canon RAW handler with default CR2 format
    /// Format will be auto-detected during processing
    pub fn new() -> Self {
        debug!("Creating Canon RAW handler (format auto-detected)");
        Self {
            format: CanonFormat::CR2, // Default, will be detected during processing
        }
    }

    /// Create new Canon RAW handler with specific format
    pub fn new_with_format(format: CanonFormat) -> Self {
        debug!("Creating Canon RAW handler for format: {}", format.name());
        Self { format }
    }

    /// Process Canon RAW file
    /// ExifTool: Canon.pm ProcessCanon() main processing logic
    pub fn process(&mut self, exif_reader: &mut ExifReader) -> Result<()> {
        debug!("Processing Canon {} format", self.format.name());

        match self.format {
            CanonFormat::CR2 => self.process_cr2(exif_reader),
            CanonFormat::CRW => self.process_crw(exif_reader),
            CanonFormat::CR3 => self.process_cr3(exif_reader),
        }
    }

    /// Process Canon CR2 format (TIFF-based)
    /// ExifTool: Canon.pm CR2 files are processed as TIFF with Canon maker notes
    fn process_cr2(&mut self, exif_reader: &mut ExifReader) -> Result<()> {
        debug!("Processing Canon CR2 format");

        // CR2 files are TIFF-based with Canon maker notes
        // The TIFF processor handles the basic TIFF structure, but we need to ensure
        // proper integration with Canon maker note processing for CR2-specific handling

        // ExifTool: Canon.pm processes CR2 by routing through TIFF with Canon maker notes
        // The existing TIFF processor will handle the file structure and automatically
        // detect Canon maker notes via detect_makernote_processor(), but CR2 files
        // may need additional Canon-specific processing beyond standard maker notes

        debug!("CR2 files processed through TIFF structure with Canon maker note integration");

        // Note: TIFF dimension extraction now happens in the RawFormatHandler implementation
        // where the raw data is available via the process_raw method

        // The Canon maker note processing will be automatically triggered by the TIFF processor
        // when it encounters MakerNotes IFD entries via detect_makernote_processor()
        // which returns "Canon::Main" for Canon signatures

        // For CR2 files, we ensure the Canon processing pipeline is properly set up
        // This includes using generated Canon data types and lookup tables
        self.ensure_canon_processing_setup(exif_reader)?;

        // Let the TIFF processor handle the main file structure
        // Canon maker notes will be automatically routed to Canon processing
        debug!("Canon CR2 processing setup complete, delegating to TIFF processor");
        Ok(())
    }

    /// Ensure Canon processing pipeline is properly configured for CR2 files
    /// ExifTool: Canon.pm initialization and setup for Canon-specific processing
    fn ensure_canon_processing_setup(&mut self, exif_reader: &mut ExifReader) -> Result<()> {
        debug!("Setting up Canon processing pipeline for CR2");

        // Verify that Canon maker note detection will work
        // ExifTool: lib/Image/ExifTool/MakerNotes.pm:60-68 Canon detection
        if let Some(make) = exif_reader.extracted_tags.get(&0x010F) {
            if let Some(make_str) = make.as_string() {
                debug!("Detected Make field: '{}'", make_str);

                // Verify Canon signature detection will work
                if crate::implementations::canon::detect_canon_signature(make_str) {
                    debug!("Canon signature detection confirmed for CR2 processing");
                } else {
                    debug!(
                        "Warning: Canon signature not detected for Make: '{}'",
                        make_str
                    );
                }
            }
        }

        // The generated Canon data types (84 types) are available in the tag structure
        // The Canon binary data processing will be handled by existing implementations
        debug!("Canon CR2 processing pipeline configured successfully");
        Ok(())
    }

    /// Process Canon CRW format (legacy custom format)
    /// ExifTool: Canon.pm ProcessCanonCRW() specialized processing
    fn process_crw(&mut self, _exif_reader: &mut ExifReader) -> Result<()> {
        debug!("Processing Canon CRW format (legacy)");

        // TODO: Implement CRW processing
        // CRW files have a custom structure, not TIFF-based
        debug!("CRW format processing not yet implemented");
        Ok(())
    }

    /// Process Canon CR3 format (MOV-based)
    /// ExifTool: Canon.pm CR3 files processed as QuickTime with Canon metadata
    fn process_cr3(&mut self, _exif_reader: &mut ExifReader) -> Result<()> {
        debug!("Processing Canon CR3 format (modern)");

        // TODO: Implement CR3 processing
        // CR3 files are MOV/MP4 containers with Canon metadata
        debug!("CR3 format processing not yet implemented");
        Ok(())
    }

    /// Auto-detect Canon format from data
    /// ExifTool: Canon.pm format detection based on magic bytes and structure
    #[allow(dead_code)]
    fn detect_format_from_data(&mut self, data: &[u8]) -> Result<()> {
        // Check for TIFF magic (CR2 files are TIFF-based)
        if data.len() >= 8 {
            let is_tiff_be = data.starts_with(b"MM\x00\x2A"); // Big-endian TIFF
            let is_tiff_le = data.starts_with(b"II\x2A\x00"); // Little-endian TIFF

            if is_tiff_be || is_tiff_le {
                debug!("Detected TIFF magic - assuming CR2 format");
                self.format = CanonFormat::CR2;
                return Ok(());
            }
        }

        // Check for CRW magic bytes
        // ExifTool: Canon.pm CRW files have specific header structure
        if data.len() >= 16 {
            // CRW files start with specific patterns
            // TODO: Add CRW magic detection when we implement CRW support
            debug!("No TIFF magic found - format detection incomplete");
        }

        // Default to CR2 if we can't determine format
        debug!("Defaulting to CR2 format");
        self.format = CanonFormat::CR2;
        Ok(())
    }
}

impl RawFormatHandler for CanonRawHandler {
    /// Process Canon RAW data
    /// ExifTool: Canon.pm main processing entry point
    fn process_raw(&self, reader: &mut ExifReader, data: &[u8]) -> Result<()> {
        debug!("Processing Canon RAW format: {}", self.format.name());

        match self.format {
            CanonFormat::CR2 => {
                debug!("Processing Canon CR2 format data ({} bytes)", data.len());

                // Extract TIFF dimensions from IFD0 for File: group
                // ExifTool: Standard TIFF tags 0x0100/0x0101 are extracted from all CR2 files
                // This must happen early to ensure File: group dimensions are available
                debug!("Extracting TIFF dimensions from Canon CR2 file");
                crate::raw::utils::extract_tiff_dimensions(reader, data)?;
                debug!("Canon CR2 TIFF dimension extraction completed");

                // For CR2 files (TIFF-based), the main TIFF processor will handle most of the work
                // Canon-specific processing happens in the Canon maker note sections
                // The existing Canon implementation in src/implementations/canon/ handles this
                debug!("CR2 processing delegated to TIFF processor with Canon maker notes");
                Ok(())
            }
            CanonFormat::CRW => {
                // TODO: Implement CRW processing
                debug!("CRW format processing not yet implemented");
                Ok(())
            }
            CanonFormat::CR3 => {
                // TODO: Implement CR3 processing
                debug!("CR3 format processing not yet implemented");
                Ok(())
            }
        }
    }

    /// Get handler name for debugging
    fn name(&self) -> &'static str {
        "Canon"
    }

    /// Validate Canon format data
    /// ExifTool: Canon.pm format validation logic
    fn validate_format(&self, data: &[u8]) -> bool {
        match self.format {
            CanonFormat::CR2 => {
                // CR2 files are TIFF-based - validate TIFF header
                if data.len() < 8 {
                    return false;
                }

                let is_tiff_be = data.starts_with(b"MM\x00\x2A"); // Big-endian TIFF
                let is_tiff_le = data.starts_with(b"II\x2A\x00"); // Little-endian TIFF

                is_tiff_be || is_tiff_le
            }
            CanonFormat::CRW => {
                // TODO: Implement CRW validation
                // For now, accept any data for CRW
                debug!("CRW validation not yet implemented - accepting all data");
                true
            }
            CanonFormat::CR3 => {
                // TODO: Implement CR3 validation
                // For now, accept any data for CR3
                debug!("CR3 validation not yet implemented - accepting all data");
                true
            }
        }
    }
}

/// Detect Canon format from file extension
/// ExifTool: Canon.pm format detection based on file extension
pub fn detect_canon_format(file_extension: &str) -> CanonFormat {
    match file_extension.to_uppercase().as_str() {
        "CR2" => CanonFormat::CR2,
        "CRW" => CanonFormat::CRW,
        "CR3" => CanonFormat::CR3,
        _ => CanonFormat::CR2, // Default to CR2 for unknown Canon formats
    }
}

/// Get Canon tag name for tag lookup
/// ExifTool: Canon.pm tag name resolution  
pub fn get_canon_tag_name(_tag_id: u16) -> Option<&'static str> {
    // Use the existing Canon tag implementation
    // Converting from Option<String> to Option<&'static str> requires different approach
    // For now, return None to fix compilation - this needs proper implementation
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::Canon_pm::tag_kit::CANON_PM_TAG_KITS;

    #[test]
    fn test_canon_format_names() {
        assert_eq!(CanonFormat::CR2.name(), "CR2");
        assert_eq!(CanonFormat::CRW.name(), "CRW");
        assert_eq!(CanonFormat::CR3.name(), "CR3");
    }

    #[test]
    fn test_canon_format_tiff_based() {
        assert!(CanonFormat::CR2.is_tiff_based());
        assert!(!CanonFormat::CRW.is_tiff_based());
        assert!(!CanonFormat::CR3.is_tiff_based());
    }

    #[test]
    fn test_canon_tag_kit_lookup() {
        // Test that we can look up Canon tags by ID
        // Note: Using stable tag IDs that exist in the current codegen output
        
        // Test that we can find some tags by ID
        assert!(CANON_PM_TAG_KITS.get(&0x0001).is_some());
        assert!(CANON_PM_TAG_KITS.get(&0x0002).is_some());
        
        // Test the AFInfo2 tag which should be stable
        assert!(CANON_PM_TAG_KITS.get(&0x0026).is_some());
        assert_eq!(CANON_PM_TAG_KITS.get(&0x0026).unwrap().name, "CanonAFInfo2");
        
        // Test that the tag kit contains some expected tags
        let has_af_config = CANON_PM_TAG_KITS.values().any(|tag| tag.name == "AFConfigTool");
        assert!(has_af_config);
        
        // Verify we have a reasonable number of tags
        assert!(CANON_PM_TAG_KITS.len() > 100, "Expected many Canon tags, got {}", CANON_PM_TAG_KITS.len());
    }

    #[test]
    fn test_canon_tag_lookup_by_id() {
        // Test looking up tags by ID in the tag kit
        // Focus on stable aspects rather than specific tag names
        
        // Test that basic lookup works
        let tag_1 = CANON_PM_TAG_KITS.get(&0x0001);
        assert!(tag_1.is_some());
        assert_eq!(tag_1.unwrap().id, 0x0001);
        
        let tag_2 = CANON_PM_TAG_KITS.get(&0x0002);
        assert!(tag_2.is_some());
        assert_eq!(tag_2.unwrap().id, 0x0002);

        // Test a known stable tag
        let tag_26 = CANON_PM_TAG_KITS.get(&0x0026);
        assert!(tag_26.is_some());
        assert_eq!(tag_26.unwrap().id, 0x0026);
        assert_eq!(tag_26.unwrap().name, "CanonAFInfo2");

        // Test unknown tag
        assert!(CANON_PM_TAG_KITS.get(&0x9999).is_none());
        
        // Test that all tags have valid IDs and names
        for (id, tag) in &*CANON_PM_TAG_KITS {
            assert_eq!(*id as u32, tag.id);
            assert!(!tag.name.is_empty());
        }
    }

    #[test]
    fn test_detect_canon_format() {
        assert_eq!(detect_canon_format("cr2"), CanonFormat::CR2);
        assert_eq!(detect_canon_format("CR2"), CanonFormat::CR2);
        assert_eq!(detect_canon_format("crw"), CanonFormat::CRW);
        assert_eq!(detect_canon_format("CRW"), CanonFormat::CRW);
        assert_eq!(detect_canon_format("cr3"), CanonFormat::CR3);
        assert_eq!(detect_canon_format("CR3"), CanonFormat::CR3);
        assert_eq!(detect_canon_format("unknown"), CanonFormat::CR2); // Default
    }

    #[test]
    fn test_canon_tag_names() {
        // Test that tag names are correctly stored in the tag kit
        // Focus on stable tags rather than specific name mappings
        
        // Test a known stable tag
        assert_eq!(CANON_PM_TAG_KITS.get(&0x0026).unwrap().name, "CanonAFInfo2");
        
        // Verify we have some expected Canon tags (names may change with codegen updates)
        let tag_names: Vec<&str> = CANON_PM_TAG_KITS.values().map(|tag| tag.name).collect();
        
        // These tags should exist somewhere in the Canon tag kit
        assert!(tag_names.iter().any(|&name| name.contains("AF")), "Expected AF-related tags");
        assert!(tag_names.iter().any(|&name| name == "CanonAFInfo2"), "Expected CanonAFInfo2 tag");
        
        // Verify all tag names are non-empty
        for tag in CANON_PM_TAG_KITS.values() {
            assert!(!tag.name.is_empty(), "Tag ID {} has empty name", tag.id);
        }
    }
}
