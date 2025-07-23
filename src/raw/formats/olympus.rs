//! Olympus RAW format handler

#![allow(dead_code, unused_variables)]
//!
//! This module implements ExifTool's Olympus.pm processing logic exactly.
//! Olympus ORF files are TIFF-based formats with manufacturer-specific maker note
//! sections that support dual processing modes (binary data vs IFD).
//!
//! ExifTool Reference: lib/Image/ExifTool/Olympus.pm (4,235 lines total)
//! Processing: Standard TIFF IFD processing with specialized section handlers

use crate::exif::ExifReader;
use crate::raw::RawFormatHandler;
use crate::tiff_types::TiffHeader;
use crate::types::{DirectoryInfo, Result, TagValue};
use tracing;

// Use generated Olympus tag structure enum
use crate::generated::Olympus_pm::tag_structure::OlympusDataType;

/// Olympus RAW format handler
/// ExifTool: lib/Image/ExifTool/Olympus.pm - TIFF-based with dual processing modes
pub struct OlympusRawHandler {
    // Core sections we process
    // ExifTool: Olympus.pm sections 0x2010-0x5000 (9 core sections from generated enum)
    supported_sections: [OlympusDataType; 9], // Using generated Olympus data type enum
}

impl Default for OlympusRawHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl OlympusRawHandler {
    /// Create new Olympus RAW handler with section mappings from generated enum
    /// ExifTool: lib/Image/ExifTool/Olympus.pm section definitions (now generated)
    pub fn new() -> Self {
        // Core data sections from generated enum - matches ExifTool exactly
        // ExifTool: Olympus.pm Main table structure (84 total variants from generated enum)
        let supported_sections = [
            OlympusDataType::Equipment,       // 0x2010: Camera/lens hardware info
            OlympusDataType::CameraSettings,  // 0x2020: Core camera settings
            OlympusDataType::RawDevelopment,  // 0x2030: RAW processing parameters
            OlympusDataType::RawDev2,         // 0x2031: Additional RAW parameters
            OlympusDataType::ImageProcessing, // 0x2040: Image processing, art filters
            OlympusDataType::FocusInfo,       // 0x2050: Autofocus information
            OlympusDataType::RawInfo,         // 0x3000: RAW file specific info
            OlympusDataType::MainInfo,        // 0x4000: Main Olympus tag table
            OlympusDataType::UnknownInfo,     // 0x5000: Unknown/experimental data
        ];

        Self { supported_sections }
    }

    /// Process Olympus-specific sections from maker notes
    /// ExifTool: Olympus.pm dual processing modes (binary data vs IFD)
    fn process_olympus_sections(
        &self,
        reader: &mut ExifReader,
        data: &[u8],
        base_offset: u64,
        data_pos: u64,
    ) -> Result<()> {
        // Get the currently extracted tags from TIFF IFD processing
        // Clone the tags to avoid borrowing issues
        let extracted_tags: Vec<(u16, TagValue)> = reader
            .get_extracted_tags()
            .iter()
            .map(|(tag_id, tag_value)| (*tag_id, tag_value.clone()))
            .collect();

        // Process sections found in the maker notes using generated enum
        for &section_type in &self.supported_sections {
            let tag_id = section_type.tag_id();
            if let Some((_, tag_value)) = extracted_tags.iter().find(|(id, _)| *id == tag_id) {
                match section_type {
                    OlympusDataType::Equipment => self.process_equipment_section(
                        reader,
                        tag_value,
                        data,
                        base_offset,
                        data_pos,
                    )?,
                    OlympusDataType::CameraSettings => self.process_camera_settings_section(
                        reader,
                        tag_value,
                        data,
                        base_offset,
                        data_pos,
                    )?,
                    OlympusDataType::FocusInfo => self.process_focus_info_section(
                        reader,
                        tag_value,
                        data,
                        base_offset,
                        data_pos,
                    )?,
                    OlympusDataType::RawDevelopment => self.process_raw_development_section(
                        reader,
                        tag_value,
                        data,
                        base_offset,
                        data_pos,
                    )?,
                    OlympusDataType::ImageProcessing => self.process_image_processing_section(
                        reader,
                        tag_value,
                        data,
                        base_offset,
                        data_pos,
                    )?,
                    _ => {
                        // Basic section processing for unknown sections
                        tracing::debug!(
                            "Found Olympus section: {} at tag {:#x}",
                            section_type.name(),
                            tag_id
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Process Equipment section (0x2010)
    /// ExifTool: Olympus.pm Equipment section - camera/lens hardware info
    /// Equipment section is an IFD structure, not raw binary data
    fn process_equipment_section(
        &self,
        reader: &mut ExifReader,
        tag_value: &TagValue,
        data: &[u8],
        base_offset: u64,
        data_pos: u64,
    ) -> Result<()> {
        // ExifTool: Equipment section is processed as an IFD
        // lib/Image/ExifTool/Olympus.pm:1587-1686 Equipment table

        match tag_value {
            TagValue::U32(offset) => {
                // Equipment section offset is relative to MakerNotes data
                // ExifTool: SubDirectory => { Start => '$val' } means offset relative to current data context
                // The current reader context contains the correct base and data position for MakerNotes
                let dir_info = DirectoryInfo {
                    name: "Olympus:Equipment".to_string(),
                    dir_start: *offset as usize,
                    dir_len: 0,        // Will be calculated by IFD processing
                    base: base_offset, // Use MakerNotes base
                    data_pos,          // Use MakerNotes data position
                    allow_reprocess: false,
                };

                tracing::debug!(
                    "Processing Olympus Equipment IFD at offset {:#x} (base: {:#x}, data_pos: {:#x})", 
                    offset, base_offset, data_pos
                );

                // Process the Equipment IFD to extract camera and lens info
                reader.process_subdirectory(&dir_info)?;
            }
            TagValue::Binary(bytes) => {
                // In some cases, the Equipment data might be directly embedded
                // For now, log a warning as we need to handle this case
                tracing::warn!(
                    "Equipment section as binary data not yet implemented, {} bytes",
                    bytes.len()
                );
            }
            _ => {
                tracing::debug!("Unexpected Equipment section format: {:?}", tag_value);
            }
        }

        Ok(())
    }

    /// Process CameraSettings section (0x2020)
    /// ExifTool: Olympus.pm CameraSettings section - core camera settings
    fn process_camera_settings_section(
        &self,
        reader: &mut ExifReader,
        tag_value: &TagValue,
        data: &[u8],
        base_offset: u64,
        data_pos: u64,
    ) -> Result<()> {
        // ExifTool: CameraSettings section processing
        // For now, just log that we found this section
        tracing::debug!("Processing Olympus CameraSettings section");
        Ok(())
    }

    /// Process FocusInfo section (0x2050)
    /// ExifTool: Olympus.pm FocusInfo section - autofocus information
    fn process_focus_info_section(
        &self,
        reader: &mut ExifReader,
        tag_value: &TagValue,
        data: &[u8],
        base_offset: u64,
        data_pos: u64,
    ) -> Result<()> {
        // ExifTool: FocusInfo section processing
        // For now, just log that we found this section
        tracing::debug!("Processing Olympus FocusInfo section");
        Ok(())
    }

    /// Process RawDevelopment section (0x2030)
    /// ExifTool: Olympus.pm RawDevelopment section - RAW processing parameters
    fn process_raw_development_section(
        &self,
        reader: &mut ExifReader,
        tag_value: &TagValue,
        data: &[u8],
        base_offset: u64,
        data_pos: u64,
    ) -> Result<()> {
        // ExifTool: RawDevelopment section processing
        // For now, just log that we found this section
        tracing::debug!("Processing Olympus RawDevelopment section");
        Ok(())
    }

    /// Process ImageProcessing section (0x2040)
    /// ExifTool: Olympus.pm ImageProcessing section - image processing, art filters
    fn process_image_processing_section(
        &self,
        reader: &mut ExifReader,
        tag_value: &TagValue,
        data: &[u8],
        base_offset: u64,
        data_pos: u64,
    ) -> Result<()> {
        // ExifTool: ImageProcessing section processing
        // For now, just log that we found this section
        tracing::debug!("Processing Olympus ImageProcessing section");
        Ok(())
    }
}

impl RawFormatHandler for OlympusRawHandler {
    /// Process Olympus ORF data
    /// ExifTool: Standard TIFF processing with Olympus-specific section handling
    fn process_raw(&self, reader: &mut ExifReader, data: &[u8]) -> Result<()> {
        // ORF files are TIFF-based, so we parse the TIFF header first
        // ExifTool: lib/Image/ExifTool/Olympus.pm uses standard TIFF processing

        // Step 1: Parse TIFF header to get IFD structure
        let header = TiffHeader::parse(data)?;
        reader.set_test_header(header.clone());
        reader.set_test_data(data.to_vec());

        // Step 2: Process the main IFD using existing TIFF infrastructure
        // ExifTool: Olympus.pm processes TIFF IFDs first, then applies special handling
        let dir_info = DirectoryInfo {
            name: "IFD0".to_string(), // Use standard TIFF IFD name for proper processing
            dir_start: header.ifd0_offset as usize,
            dir_len: 0,  // Will be calculated by IFD processing
            base: 0,     // Standard TIFF base
            data_pos: 0, // No additional data position offset
            allow_reprocess: false,
        };

        // Process the TIFF IFD to extract standard EXIF tags and maker notes
        reader.process_subdirectory(&dir_info)?;

        // Step 3: Apply Olympus-specific section processing to extracted maker note entries
        // ExifTool: Olympus.pm applies additional processing for sections like Equipment (0x2010)
        // For ORF files, use standard TIFF base and data position
        self.process_olympus_sections(reader, data, reader.get_base(), 0)?;

        Ok(())
    }

    fn name(&self) -> &'static str {
        "Olympus"
    }

    fn validate_format(&self, data: &[u8]) -> bool {
        // ExifTool: Olympus.pm validation logic - TIFF-based format
        super::super::detector::validate_olympus_orf_magic(data)
    }
}

/// Get Olympus ORF tag name by ID
/// ExifTool: lib/Image/ExifTool/Olympus.pm tag definitions
pub fn get_olympus_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        0x2010 => Some("Equipment"),
        0x2020 => Some("CameraSettings"),
        0x2030 => Some("RawDevelopment"),
        0x2031 => Some("RawDev2"),
        0x2040 => Some("ImageProcessing"),
        0x2050 => Some("FocusInfo"),
        0x3000 => Some("RawInfo"),
        0x4000 => Some("MainInfo"),
        0x5000 => Some("UnknownInfo"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_olympus_handler_creation() {
        let handler = OlympusRawHandler::new();
        assert_eq!(handler.name(), "Olympus");
        assert_eq!(handler.supported_sections.len(), 9);
    }

    #[test]
    fn test_get_olympus_tag_name() {
        // Test known tag names
        assert_eq!(get_olympus_tag_name(0x2010), Some("Equipment"));
        assert_eq!(get_olympus_tag_name(0x2020), Some("CameraSettings"));
        assert_eq!(get_olympus_tag_name(0x2050), Some("FocusInfo"));
        assert_eq!(get_olympus_tag_name(0x4000), Some("MainInfo"));

        // Test unknown tag
        assert_eq!(get_olympus_tag_name(0x9999), None);
    }

    #[test]
    fn test_format_validation() {
        let handler = OlympusRawHandler::new();

        // Test valid TIFF magic (big-endian)
        let valid_be_data = b"MM\x00\x2A\x00\x00\x00\x08test_data";
        assert!(handler.validate_format(valid_be_data));

        // Test valid TIFF magic (little-endian)
        let valid_le_data = b"II\x2A\x00\x08\x00\x00\x00test_data";
        assert!(handler.validate_format(valid_le_data));

        // Test invalid magic
        let invalid_data = b"XX\x2A\x00\x08\x00\x00\x00test_data";
        assert!(!handler.validate_format(invalid_data));

        // Test insufficient data
        let short_data = b"MM\x00";
        assert!(!handler.validate_format(short_data));
    }

    #[test]
    fn test_section_mapping() {
        let handler = OlympusRawHandler::new();

        // Test that all expected sections are present using generated enum
        assert!(handler
            .supported_sections
            .contains(&OlympusDataType::Equipment));
        assert!(handler
            .supported_sections
            .contains(&OlympusDataType::CameraSettings));
        assert!(handler
            .supported_sections
            .contains(&OlympusDataType::RawDevelopment));
        assert!(handler
            .supported_sections
            .contains(&OlympusDataType::RawDev2));
        assert!(handler
            .supported_sections
            .contains(&OlympusDataType::ImageProcessing));
        assert!(handler
            .supported_sections
            .contains(&OlympusDataType::FocusInfo));
        assert!(handler
            .supported_sections
            .contains(&OlympusDataType::RawInfo));
        assert!(handler
            .supported_sections
            .contains(&OlympusDataType::MainInfo));
        assert!(handler
            .supported_sections
            .contains(&OlympusDataType::UnknownInfo));

        // Test that enum methods work correctly
        assert_eq!(OlympusDataType::Equipment.tag_id(), 0x2010);
        assert_eq!(OlympusDataType::CameraSettings.tag_id(), 0x2020);
        assert_eq!(OlympusDataType::RawDevelopment.tag_id(), 0x2030);
        assert_eq!(OlympusDataType::Equipment.name(), "Equipment");
        assert_eq!(OlympusDataType::CameraSettings.name(), "CameraSettings");

        // Test reverse lookup from tag ID
        assert_eq!(
            OlympusDataType::from_tag_id(0x2010),
            Some(OlympusDataType::Equipment)
        );
        assert_eq!(
            OlympusDataType::from_tag_id(0x2020),
            Some(OlympusDataType::CameraSettings)
        );

        // Test unmapped section
        assert_eq!(OlympusDataType::from_tag_id(0x9999), None);
    }
}
