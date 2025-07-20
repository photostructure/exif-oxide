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
        // Debug logging to understand selection logic
        debug!(
            "OlympusEquipmentProcessor::can_process - manufacturer: {:?}, table: {}",
            context.manufacturer, context.table_name
        );

        // ExifTool: Standard IFD directories must use standard IFD parsing, not manufacturer processors
        // Following Trust ExifTool: ExifIFD, GPS, InteropIFD use standard TIFF processing
        if matches!(
            context.table_name.as_str(),
            "ExifIFD" | "GPS" | "InteropIFD" | "IFD0" | "IFD1"
        ) {
            debug!("OlympusEquipmentProcessor::can_process - returning Incompatible for standard IFD: {}", context.table_name);
            return ProcessorCapability::Incompatible;
        }

        // ExifTool: More flexible Olympus detection - any Make starting with "OLYMPUS"
        let is_olympus = context
            .manufacturer
            .as_ref()
            .map(|m| m.starts_with("OLYMPUS") || m == "OM Digital Solutions")
            .unwrap_or(false);
        let is_equipment = context.table_name.contains("Equipment");

        debug!(
            "OlympusEquipmentProcessor::can_process - is_olympus: {}, is_equipment: {}",
            is_olympus, is_equipment
        );

        // Equipment has WRITE_PROC => WriteExif in ExifTool, indicating it's an IFD structure
        // This processor only handles binary data, so return Incompatible for Equipment
        // ExifTool: lib/Image/ExifTool/Olympus.pm line 1588
        if is_equipment {
            debug!("OlympusEquipmentProcessor::can_process - returning Incompatible for Equipment (should be IFD)");
            ProcessorCapability::Incompatible
        } else if is_olympus {
            // This processor can handle other Olympus binary data sections
            debug!("OlympusEquipmentProcessor::can_process - returning Good");
            ProcessorCapability::Good
        } else {
            debug!("OlympusEquipmentProcessor::can_process - returning Incompatible");
            ProcessorCapability::Incompatible
        }
    }

    fn process_data(&self, data: &[u8], _context: &ProcessorContext) -> Result<ProcessorResult> {
        let mut result = ProcessorResult::new();
        debug!(
            "Processing Olympus Equipment section with {} bytes",
            data.len()
        );

        // Add debug logging to see what's in the Equipment data
        // This will help us determine if it's IFD format or raw binary
        if data.len() >= 50 {
            debug!(
                "Equipment data preview (first 50 bytes): {:02x?}",
                &data[0..50]
            );
        } else {
            debug!("Equipment data preview: {:02x?}", data);
        }

        // Check if this looks like an IFD (starts with entry count)
        if data.len() >= 2 {
            let entry_count = u16::from_le_bytes([data[0], data[1]]);
            debug!("Possible IFD entry count (LE): {}", entry_count);
            let entry_count_be = u16::from_be_bytes([data[0], data[1]]);
            debug!("Possible IFD entry count (BE): {}", entry_count_be);
        }

        // ExifTool: lib/Image/ExifTool/Olympus.pm Equipment table processing
        // Equipment has WRITE_PROC => WriteExif, CHECK_PROC => CheckExif
        // This indicates it's an IFD structure, not raw binary data!

        // TODO: Equipment should be parsed as IFD, not raw binary
        // For now, let's check what's at the expected offsets

        // Check data at offset 0x100 (CameraType2)
        if data.len() > 0x106 {
            debug!(
                "Data at CameraType2 offset 0x100: {:02x?}",
                &data[0x100..std::cmp::min(0x106, data.len())]
            );
            debug!(
                "As string: {:?}",
                String::from_utf8_lossy(&data[0x100..std::cmp::min(0x106, data.len())])
            );
        }

        // Check data at offset 0x201 (LensType)
        if data.len() > 0x207 {
            debug!(
                "Data at LensType offset 0x201: {:02x?}",
                &data[0x201..std::cmp::min(0x207, data.len())]
            );
        }

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
        // ExifTool: Standard IFD directories must use standard IFD parsing, not manufacturer processors
        // Following Trust ExifTool: ExifIFD, GPS, InteropIFD use standard TIFF processing
        if matches!(
            context.table_name.as_str(),
            "ExifIFD" | "GPS" | "InteropIFD" | "IFD0" | "IFD1"
        ) {
            debug!("OlympusCameraSettingsProcessor::can_process - returning Incompatible for standard IFD: {}", context.table_name);
            return ProcessorCapability::Incompatible;
        }

        // Equipment has WRITE_PROC => WriteExif, indicating it's an IFD structure
        // This processor only handles binary data, so reject Equipment tables
        if context.table_name.contains("Equipment") {
            debug!("OlympusCameraSettingsProcessor::can_process - returning Incompatible for Equipment (should be IFD)");
            return ProcessorCapability::Incompatible;
        }

        if (context.is_manufacturer("OLYMPUS")
            || context.is_manufacturer("OLYMPUS IMAGING CORP.")
            || context.is_manufacturer("OLYMPUS CORPORATION"))
            && context.table_name.contains("CameraSettings")
        {
            ProcessorCapability::Perfect
        } else if context.is_manufacturer("OLYMPUS")
            || context.is_manufacturer("OLYMPUS IMAGING CORP.")
            || context.is_manufacturer("OLYMPUS CORPORATION")
        {
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
        // ExifTool: Standard IFD directories must use standard IFD parsing, not manufacturer processors
        // Following Trust ExifTool: ExifIFD, GPS, InteropIFD use standard TIFF processing
        if matches!(
            context.table_name.as_str(),
            "ExifIFD" | "GPS" | "InteropIFD" | "IFD0" | "IFD1"
        ) {
            debug!("OlympusFocusInfoProcessor::can_process - returning Incompatible for standard IFD: {}", context.table_name);
            return ProcessorCapability::Incompatible;
        }

        // Equipment has WRITE_PROC => WriteExif, indicating it's an IFD structure
        // This processor only handles binary data, so reject Equipment tables
        if context.table_name.contains("Equipment") {
            return ProcessorCapability::Incompatible;
        }

        if (context.is_manufacturer("OLYMPUS")
            || context.is_manufacturer("OLYMPUS IMAGING CORP.")
            || context.is_manufacturer("OLYMPUS CORPORATION"))
            && context.table_name.contains("FocusInfo")
        {
            ProcessorCapability::Perfect
        } else if context.is_manufacturer("OLYMPUS")
            || context.is_manufacturer("OLYMPUS IMAGING CORP.")
            || context.is_manufacturer("OLYMPUS CORPORATION")
        {
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
