//! Olympus-specific BinaryDataProcessor implementations
//!
//! These processors implement the BinaryDataProcessor trait for Olympus camera data
//! following ExifTool's Olympus.pm processing logic exactly.
//!
//! ## ExifTool Reference
//!
//! lib/Image/ExifTool/Olympus.pm - Equipment section, CameraSettings, FocusInfo, etc.

use crate::generated::Olympus_pm::{lookup_olympus_camera_types, lookup_olympus_lens_types};
use crate::processor_registry::{
    BinaryDataProcessor, ProcessorCapability, ProcessorContext, ProcessorMetadata, ProcessorResult,
};
use crate::types::{Result, TagValue};
use tracing::debug;

/// Olympus Equipment section processor
///
/// Processes Olympus Equipment section (tag 0x2010) following ExifTool's exact logic.
/// ExifTool: lib/Image/ExifTool/Olympus.pm Equipment table (lines 1587-1686)
pub struct OlympusEquipmentProcessor;

impl BinaryDataProcessor for OlympusEquipmentProcessor {
    fn can_process(&self, context: &ProcessorContext) -> ProcessorCapability {
        // Perfect match if Olympus manufacturer and Equipment table
        if context.is_manufacturer("OLYMPUS") && context.table_name.contains("Equipment") {
            ProcessorCapability::Perfect
        } else if context.is_manufacturer("OLYMPUS") {
            ProcessorCapability::Good
        } else {
            ProcessorCapability::Incompatible
        }
    }

    fn process_data(&self, data: &[u8], _context: &ProcessorContext) -> Result<ProcessorResult> {
        let mut result = ProcessorResult::new();
        debug!(
            "Processing Olympus Equipment section with {} bytes",
            data.len()
        );

        // ExifTool: lib/Image/ExifTool/Olympus.pm Equipment table processing
        // Tags are processed as IFD entries within the Equipment section

        // Equipment section is processed as an IFD, so we extract known offsets
        // ExifTool: Equipment table tag definitions

        // Extract CameraType2 (0x100) - 6-byte string
        // ExifTool: lib/Image/ExifTool/Olympus.pm line 1598-1604
        if data.len() >= 0x106 && data.len() > 0x100 {
            let camera_bytes = &data[0x100..std::cmp::min(0x106, data.len())];
            let camera_str = String::from_utf8_lossy(camera_bytes)
                .trim_end_matches('\0')
                .trim()
                .to_string();

            if !camera_str.is_empty() {
                // Look up camera name using generated table
                if let Some(camera_name) = lookup_olympus_camera_types(&camera_str) {
                    result.extracted_tags.insert(
                        "CameraType2".to_string(),
                        TagValue::String(camera_name.to_string()),
                    );
                    debug!("Extracted camera type: {} -> {}", camera_str, camera_name);
                }
            }
        }

        // Extract LensType (0x201) - 6 bytes: int8u[6]
        // ExifTool: lib/Image/ExifTool/Olympus.pm line 1631-1647
        // ValueConv: sprintf("%x %.2x %.2x",@a[0,2,3])
        if data.len() >= 0x207 && data.len() > 0x201 {
            let lens_bytes = &data[0x201..std::cmp::min(0x207, data.len())];
            if lens_bytes.len() >= 6 {
                // ExifTool logic: format bytes 0, 2, 3 as hex string
                let lens_code = format!(
                    "{:x} {:02x} {:02x}",
                    lens_bytes[0], lens_bytes[2], lens_bytes[3]
                );

                // Look up lens name using generated table
                if let Some(lens_name) = lookup_olympus_lens_types(&lens_code) {
                    result.extracted_tags.insert(
                        "LensType".to_string(),
                        TagValue::String(lens_name.to_string()),
                    );
                    debug!("Extracted lens type: {} -> {}", lens_code, lens_name);
                }
            }
        }

        if result.extracted_tags.is_empty() {
            result.add_warning("No Olympus Equipment tags extracted".to_string());
        } else {
            debug!(
                "Olympus Equipment processor extracted {} tags",
                result.extracted_tags.len()
            );
        }

        Ok(result)
    }

    fn get_metadata(&self) -> ProcessorMetadata {
        ProcessorMetadata::new(
            "Olympus Equipment Processor".to_string(),
            "Processes Olympus Equipment section using ExifTool logic".to_string(),
        )
        .with_manufacturer("OLYMPUS".to_string())
    }
}

/// Olympus CameraSettings section processor
///
/// Processes Olympus CameraSettings section (tag 0x2020) following ExifTool's logic.
/// ExifTool: lib/Image/ExifTool/Olympus.pm CameraSettings table
pub struct OlympusCameraSettingsProcessor;

impl BinaryDataProcessor for OlympusCameraSettingsProcessor {
    fn can_process(&self, context: &ProcessorContext) -> ProcessorCapability {
        if context.is_manufacturer("OLYMPUS") && context.table_name.contains("CameraSettings") {
            ProcessorCapability::Perfect
        } else if context.is_manufacturer("OLYMPUS") {
            ProcessorCapability::Good
        } else {
            ProcessorCapability::Incompatible
        }
    }

    fn process_data(&self, data: &[u8], _context: &ProcessorContext) -> Result<ProcessorResult> {
        let mut result = ProcessorResult::new();
        debug!(
            "Processing Olympus CameraSettings section with {} bytes",
            data.len()
        );

        // ExifTool: lib/Image/ExifTool/Olympus.pm CameraSettings table processing
        // This is a placeholder for now - can be expanded based on ExifTool's exact implementation

        debug!("Olympus CameraSettings processing (placeholder)");
        result.add_warning("Olympus CameraSettings processing not fully implemented".to_string());

        Ok(result)
    }

    fn get_metadata(&self) -> ProcessorMetadata {
        ProcessorMetadata::new(
            "Olympus CameraSettings Processor".to_string(),
            "Processes Olympus CameraSettings section using ExifTool logic".to_string(),
        )
        .with_manufacturer("OLYMPUS".to_string())
    }
}

/// Olympus FocusInfo section processor
///
/// Processes Olympus FocusInfo section (tag 0x2050) following ExifTool's logic.
/// ExifTool: lib/Image/ExifTool/Olympus.pm FocusInfo table
pub struct OlympusFocusInfoProcessor;

impl BinaryDataProcessor for OlympusFocusInfoProcessor {
    fn can_process(&self, context: &ProcessorContext) -> ProcessorCapability {
        if context.is_manufacturer("OLYMPUS") && context.table_name.contains("FocusInfo") {
            ProcessorCapability::Perfect
        } else if context.is_manufacturer("OLYMPUS") {
            ProcessorCapability::Good
        } else {
            ProcessorCapability::Incompatible
        }
    }

    fn process_data(&self, data: &[u8], _context: &ProcessorContext) -> Result<ProcessorResult> {
        let mut result = ProcessorResult::new();
        debug!(
            "Processing Olympus FocusInfo section with {} bytes",
            data.len()
        );

        // ExifTool: lib/Image/ExifTool/Olympus.pm FocusInfo table processing
        // This is a placeholder for now - can be expanded based on ExifTool's exact implementation

        debug!("Olympus FocusInfo processing (placeholder)");
        result.add_warning("Olympus FocusInfo processing not fully implemented".to_string());

        Ok(result)
    }

    fn get_metadata(&self) -> ProcessorMetadata {
        ProcessorMetadata::new(
            "Olympus FocusInfo Processor".to_string(),
            "Processes Olympus FocusInfo section using ExifTool logic".to_string(),
        )
        .with_manufacturer("OLYMPUS".to_string())
    }
}
