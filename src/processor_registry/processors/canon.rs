//! Canon-specific BinaryDataProcessor implementations
//!
//! These processors implement the BinaryDataProcessor trait for Canon camera data
//! while delegating to the existing Canon implementation modules. This maintains
//! the "Trust ExifTool" principle by reusing proven processing logic.
//!
//! ## ExifTool Reference
//!
//! Canon.pm ProcessBinaryData, ProcessSerialData, and related processing functions

use super::super::{
    BinaryDataProcessor, ProcessorCapability, ProcessorContext, ProcessorMetadata, ProcessorResult,
};
use crate::implementations::canon;
use crate::implementations::canon::tags::get_canon_tag_name;
use crate::types::{Result, TagValue};
use std::collections::HashMap;
use tracing::debug;

/// Type alias for tag kit processor functions
type TagKitProcessor = fn(
    u32,
    &TagValue,
    crate::tiff_types::ByteOrder,
    Option<&str>,
) -> Result<HashMap<String, TagValue>>;

/// Canon Serial Data processor using existing implementation
///
/// Processes Canon camera serial data using the existing `canon::binary_data`
/// implementation. This processor is selected for Canon cameras with table
/// names containing "SerialData".
///
/// ## ExifTool Reference
///
/// Canon.pm ProcessSerialData function and related binary data processing
pub struct CanonSerialDataProcessor;

impl BinaryDataProcessor for CanonSerialDataProcessor {
    fn can_process(&self, context: &ProcessorContext) -> ProcessorCapability {
        // Perfect match if Canon manufacturer and SerialData table
        if context.is_manufacturer("Canon") && context.table_name.contains("SerialData") {
            return ProcessorCapability::Perfect;
        }

        // Good match for Canon with any binary data table
        if context.is_manufacturer("Canon") && context.table_name.contains("Binary") {
            return ProcessorCapability::Good;
        }

        // Only compatible with Canon-specific tables
        ProcessorCapability::Incompatible
    }

    fn process_data(&self, data: &[u8], context: &ProcessorContext) -> Result<ProcessorResult> {
        debug!(
            "Processing Canon SerialData with {} bytes for table: {}",
            data.len(),
            context.table_name
        );

        let mut result = ProcessorResult::new();

        // Delegate to existing Canon binary data processing
        // This extracts camera settings and serial data using proven ExifTool logic
        let byte_order = context
            .byte_order
            .unwrap_or(crate::tiff_types::ByteOrder::LittleEndian);
        let camera_settings = canon::binary_data::extract_camera_settings(
            data,
            context.data_offset,
            data.len(),
            byte_order,
        );

        match camera_settings {
            Ok(tags) => {
                // Convert HashMap<String, TagValue> to ProcessorResult format
                for (tag_name, tag_value) in tags {
                    result.add_tag(tag_name, tag_value);
                }

                debug!(
                    "Canon SerialData processor extracted {} tags",
                    result.extracted_tags.len()
                );
            }
            Err(e) => {
                let warning = format!("Canon SerialData processing failed: {e}");
                result.add_warning(warning);
                debug!("Canon SerialData processing error: {}", e);
            }
        }

        Ok(result)
    }

    fn get_metadata(&self) -> ProcessorMetadata {
        ProcessorMetadata::new(
            "Canon Serial Data Processor".to_string(),
            "Processes Canon camera serial data using existing implementation".to_string(),
        )
        .with_manufacturer("Canon".to_string())
        .with_required_context("manufacturer".to_string())
        .with_example_condition(
            "manufacturer == 'Canon' && table.contains('SerialData')".to_string(),
        )
    }
}

/// Canon Camera Settings processor for binary data tables
///
/// Processes Canon CameraSettings binary data using the existing implementation.
/// This handles the complex binary data formats used by Canon cameras.
///
/// ## ExifTool Reference
///
/// Canon.pm CameraSettings table processing with ProcessBinaryData
pub struct CanonCameraSettingsProcessor;

impl BinaryDataProcessor for CanonCameraSettingsProcessor {
    fn can_process(&self, context: &ProcessorContext) -> ProcessorCapability {
        // Only process Canon-specific tables, not standard EXIF directories
        // ExifTool Canon.pm only processes Canon:: prefixed tables
        if !context.is_manufacturer("Canon") {
            return ProcessorCapability::Incompatible;
        }

        // Perfect match for Canon CameraSettings table
        if context.table_name == "Canon::CameraSettings" {
            return ProcessorCapability::Perfect;
        }

        // Good match for Canon binary data tables that start with Canon::
        if context.table_name.starts_with("Canon::") && context.table_name.contains("Settings") {
            return ProcessorCapability::Good;
        }

        // Incompatible with non-Canon tables (like ExifIFD, GPS, etc.)
        ProcessorCapability::Incompatible
    }

    fn process_data(&self, data: &[u8], context: &ProcessorContext) -> Result<ProcessorResult> {
        debug!(
            "Processing Canon CameraSettings with {} bytes for table: {}",
            data.len(),
            context.table_name
        );

        let mut result = ProcessorResult::new();

        // Use the specialized Canon CameraSettings extraction function that handles FocalUnits dependencies
        // This calls our extract_camera_settings function with proper ValueConv and PrintConv
        let byte_order = context
            .byte_order
            .unwrap_or(crate::tiff_types::ByteOrder::LittleEndian);
        let extracted_tags = canon::binary_data::extract_camera_settings(
            data,
            0, // offset
            data.len(),
            byte_order,
        )?;

        for (tag_name, tag_value) in extracted_tags {
            result.add_tag(tag_name, tag_value);
        }

        if result.extracted_tags.is_empty() {
            result.add_warning("No Canon CameraSettings tags extracted".to_string());
        } else {
            debug!(
                "Canon CameraSettings processor extracted {} tags",
                result.extracted_tags.len()
            );
        }

        Ok(result)
    }

    fn get_metadata(&self) -> ProcessorMetadata {
        ProcessorMetadata::new(
            "Canon Camera Settings Processor".to_string(),
            "Processes Canon CameraSettings binary data table".to_string(),
        )
        .with_manufacturer("Canon".to_string())
        .with_required_context("manufacturer".to_string())
        .with_example_condition(
            "manufacturer == 'Canon' && table.contains('CameraSettings')".to_string(),
        )
    }
}

/// Enhanced Canon processor for newer models with additional data
///
/// Handles Canon R5, R6, and other newer models that have enhanced
/// binary data formats and additional metadata fields.
///
/// ## ExifTool Reference
///
/// Canon.pm model-specific processing for newer camera bodies
pub struct CanonSerialDataMkIIProcessor;

impl BinaryDataProcessor for CanonSerialDataMkIIProcessor {
    fn can_process(&self, context: &ProcessorContext) -> ProcessorCapability {
        // Perfect match for newer Canon models with enhanced data
        if context.is_manufacturer("Canon") {
            if let Some(model) = &context.model {
                if (model.contains("EOS R5")
                    || model.contains("EOS R6")
                    || model.contains("EOS R3"))
                    && context.table_name.contains("SerialData")
                {
                    return ProcessorCapability::Perfect;
                }
            }
        }

        ProcessorCapability::Incompatible
    }

    fn process_data(&self, data: &[u8], context: &ProcessorContext) -> Result<ProcessorResult> {
        debug!(
            "Processing Canon MkII SerialData with {} bytes for model: {:?}",
            data.len(),
            context.model
        );

        let mut result = ProcessorResult::new();

        // Enhanced serial data processing for newer Canon models
        if data.len() < 12 {
            result.add_warning("Canon MkII serial data too short".to_string());
            return Ok(result);
        }

        // Extract extended serial information (following ExifTool patterns)
        // ExifTool: Enhanced data extraction for newer models

        // Serial number (first 8 bytes)
        let serial_bytes = &data[0..8];
        let serial_number = String::from_utf8_lossy(serial_bytes)
            .trim_end_matches('\0')
            .to_string();
        if !serial_number.is_empty() {
            result.add_tag("SerialNumber".to_string(), TagValue::String(serial_number));
        }

        // Firmware version (bytes 8-11) - enhanced format for newer models
        if data.len() >= 12 {
            let firmware_major = data[8];
            let firmware_minor = data[9];
            let firmware_patch = u16::from_le_bytes([data[10], data[11]]);

            let firmware_version = format!("{firmware_major}.{firmware_minor}.{firmware_patch}");
            result.add_tag(
                "FirmwareVersion".to_string(),
                TagValue::String(firmware_version),
            );
        }

        // Additional model-specific data extraction could go here
        if let Some(model) = &context.model {
            result.add_tag("CameraModel".to_string(), TagValue::String(model.clone()));
        }

        debug!(
            "Canon MkII SerialData processor extracted {} tags",
            result.extracted_tags.len()
        );

        Ok(result)
    }

    fn get_metadata(&self) -> ProcessorMetadata {
        ProcessorMetadata::new(
            "Canon Serial Data MkII".to_string(),
            "Enhanced serial data processing for newer Canon models (R5, R6, R3)".to_string(),
        )
        .with_manufacturer("Canon".to_string())
        .with_required_context("manufacturer".to_string())
        .with_required_context("model".to_string())
        .with_example_condition(
            "manufacturer == 'Canon' && (model.contains('EOS R5') || model.contains('EOS R6'))"
                .to_string(),
        )
    }
}

/// Canon Main processor that delegates to tag kit system
///
/// This processor handles Canon MakerNotes processing by using the tag kit system
/// for binary data integration. It replaces the direct Canon MakerNotes processing
/// to enable proper binary data parsing for tags like ProcessingInfo.
///
/// ## ExifTool Reference
///
/// Canon.pm MakerNotes processing with tag kit integration for binary data tables
pub struct CanonMainProcessor;

impl BinaryDataProcessor for CanonMainProcessor {
    fn can_process(&self, context: &ProcessorContext) -> ProcessorCapability {
        // Perfect match for Canon MakerNotes
        if context.is_manufacturer("Canon") && context.table_name == "MakerNotes" {
            return ProcessorCapability::Perfect;
        }

        // Good match for any Canon table starting with Canon::
        if context.is_manufacturer("Canon") && context.table_name.starts_with("Canon::") {
            return ProcessorCapability::Good;
        }

        ProcessorCapability::Incompatible
    }

    fn process_data(&self, data: &[u8], context: &ProcessorContext) -> Result<ProcessorResult> {
        debug!(
            "Canon Main processor processing {} bytes for table: {}",
            data.len(),
            context.table_name
        );

        let mut result = ProcessorResult::new();

        // Use generic tag kit integration for Canon MakerNotes
        let byte_order = context
            .byte_order
            .unwrap_or(crate::tiff_types::ByteOrder::LittleEndian);
        let extracted_tags = extract_makernotes_via_tag_kit(
            data,
            context.data_offset,
            byte_order,
            "Canon",
            crate::generated::Canon_pm::tag_kit::process_subdirectory,
        );

        match extracted_tags {
            Ok(tags) => {
                debug!("Canon tag kit extracted {} tags", tags.len());
                for (tag_name, tag_value) in tags {
                    result.add_tag(tag_name, tag_value);
                }
            }
            Err(e) => {
                let warning = format!("Canon tag kit processing failed: {e}");
                result.add_warning(warning);
                debug!("Canon tag kit processing error: {}", e);
            }
        }

        if result.extracted_tags.is_empty() {
            result.add_warning("No Canon tags extracted from MakerNotes".to_string());
        } else {
            debug!(
                "Canon Main processor extracted {} tags",
                result.extracted_tags.len()
            );
        }

        Ok(result)
    }

    fn get_metadata(&self) -> ProcessorMetadata {
        ProcessorMetadata::new(
            "Canon Main Processor".to_string(),
            "Processes Canon MakerNotes using tag kit system with binary data integration"
                .to_string(),
        )
        .with_manufacturer("Canon".to_string())
        .with_required_context("manufacturer".to_string())
        .with_example_condition("manufacturer == 'Canon' && table == 'MakerNotes'".to_string())
    }
}

/// Generic MakerNotes extraction via tag kit system
///
/// This function provides a DRY solution for all manufacturers to integrate their tag kits
/// with the processor registry system. It parses MakerNotes as an IFD structure and calls
/// the manufacturer-specific tag kit processor for each subdirectory tag.
///
/// ## Pattern for all manufacturers:
/// 1. Parse MakerNotes as IFD to extract individual tag entries  
/// 2. For each tag with binary data, call manufacturer tag kit's process_subdirectory
/// 3. Collect and return all extracted tags with proper namespacing
///
/// ## Usage:
/// ```rust,ignore
/// let tags = extract_makernotes_via_tag_kit(
///     data, offset, byte_order, "Canon",
///     crate::generated::Canon_pm::tag_kit::process_subdirectory
/// )?;
/// ```
fn extract_makernotes_via_tag_kit(
    data: &[u8],
    data_offset: usize,
    byte_order: crate::tiff_types::ByteOrder,
    manufacturer: &str,
    tag_kit_processor: TagKitProcessor,
) -> Result<HashMap<String, TagValue>> {
    use crate::implementations::nikon::ifd::extract_tag_value;
    use crate::tiff_types::IfdEntry;

    let mut extracted_tags = HashMap::new();

    debug!(
        "Processing {} MakerNotes via tag kit: {} bytes at offset {:#x}",
        manufacturer,
        data.len(),
        data_offset
    );

    // Parse MakerNotes as IFD structure to extract individual tag entries
    // Canon MakerNotes have NO TIFF header - they start directly with IFD entry count
    // ExifTool: lib/Image/ExifTool/MakerNotes.pm Canon "(starts with an IFD)"
    if data.len() < 2 {
        return Ok(extracted_tags);
    }

    // Canon MakerNotes start directly with IFD entry count (no header)
    let ifd_offset = 0;

    // Read number of IFD entries at the beginning of the data
    let entry_count = match byte_order {
        crate::tiff_types::ByteOrder::LittleEndian => {
            u16::from_le_bytes([data[ifd_offset], data[ifd_offset + 1]])
        }
        crate::tiff_types::ByteOrder::BigEndian => {
            u16::from_be_bytes([data[ifd_offset], data[ifd_offset + 1]])
        }
    } as usize;

    debug!(
        "Found {} IFD entries in {} MakerNotes",
        entry_count, manufacturer
    );

    // Process each IFD entry through the tag kit
    let mut offset = ifd_offset + 2;
    for _i in 0..entry_count {
        if offset + 12 > data.len() {
            break;
        }

        let entry = match IfdEntry::parse(data, offset, byte_order) {
            Ok(entry) => entry,
            Err(e) => {
                debug!("Failed to parse IFD entry at offset {:#x}: {}", offset, e);
                break;
            }
        };

        // Extract tag data as TagValue for tag kit processing
        let tag_value = match extract_tag_value(data, &entry, byte_order) {
            Ok(value) => value,
            Err(e) => {
                debug!(
                    "Failed to extract tag value for tag 0x{:04x}: {}",
                    entry.tag_id, e
                );
                offset += 12;
                continue;
            }
        };

        // First try tag kit for binary data tags (subdirectories)
        // TODO: Pass actual model information when available at this processing level
        let tag_kit_result = tag_kit_processor(entry.tag_id as u32, &tag_value, byte_order, None);

        match tag_kit_result {
            Ok(processed_tags) if !processed_tags.is_empty() => {
                // Successfully processed as binary data tag
                debug!(
                    "Tag 0x{:04x} extracted {} sub-tags via {} tag kit",
                    entry.tag_id,
                    processed_tags.len(),
                    manufacturer
                );
                for (name, value) in processed_tags {
                    // Tag storage system will add MakerNotes namespace automatically
                    // based on IFD name, so don't add prefix here to avoid double namespacing
                    extracted_tags.insert(name, value);
                }
            }
            Ok(_) | Err(_) => {
                // Not a binary data tag or tag kit failed - treat as regular MakerNotes tag
                let tag_name = get_canon_tag_name(entry.tag_id)
                    .unwrap_or_else(|| format!("CanonTag_{:04x}", entry.tag_id));

                debug!(
                    "Tag 0x{:04x} processed as regular MakerNotes tag: {}",
                    entry.tag_id, tag_name
                );

                // Apply PrintConv for regular Canon tags like FileNumber
                // ExifTool: lib/Image/ExifTool/Canon.pm PrintConv handling for scalar tags
                let final_value = if manufacturer == "Canon" {
                    // Apply Canon-specific PrintConv using the tag kit system
                    use crate::expressions::ExpressionEvaluator;
                    let mut evaluator = ExpressionEvaluator::new();
                    let mut errors = Vec::new();
                    let mut warnings = Vec::new();

                    let print_value = crate::generated::Canon_pm::tag_kit::apply_print_conv(
                        entry.tag_id as u32,
                        &tag_value,
                        &mut evaluator,
                        &mut errors,
                        &mut warnings,
                    );

                    // Log any warnings
                    for warning in warnings {
                        debug!(
                            "Canon PrintConv warning for tag 0x{:04x}: {}",
                            entry.tag_id, warning
                        );
                    }

                    // Use print value if different from raw, otherwise use raw
                    if print_value != tag_value {
                        debug!(
                            "Applied Canon PrintConv to tag 0x{:04x}: {:?} -> {:?}",
                            entry.tag_id, tag_value, print_value
                        );
                        print_value
                    } else {
                        tag_value
                    }
                } else {
                    tag_value
                };

                // Store as regular tag - tag storage system will add MakerNotes namespace
                extracted_tags.insert(tag_name, final_value);
            }
        }

        offset += 12;
    }

    debug!(
        "{} tag kit extraction completed with {} tags",
        manufacturer,
        extracted_tags.len()
    );
    Ok(extracted_tags)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::FileFormat;
    use crate::processor_registry::ProcessorContext;

    #[test]
    fn test_canon_serial_data_processor_capability() {
        let processor = CanonSerialDataProcessor;

        // Perfect match for Canon SerialData
        let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::SerialData".to_string())
            .with_manufacturer("Canon".to_string());

        assert_eq!(
            processor.can_process(&context),
            ProcessorCapability::Perfect
        );

        // Incompatible for non-Canon
        let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::SerialData".to_string())
            .with_manufacturer("Nikon".to_string());

        assert_eq!(
            processor.can_process(&context),
            ProcessorCapability::Incompatible
        );
    }

    #[test]
    fn test_canon_mk2_processor_capability() {
        let processor = CanonSerialDataMkIIProcessor;

        // Perfect match for Canon R5
        let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::SerialData".to_string())
            .with_manufacturer("Canon".to_string())
            .with_model("Canon EOS R5".to_string());

        assert_eq!(
            processor.can_process(&context),
            ProcessorCapability::Perfect
        );

        // Incompatible for older Canon models
        let context = ProcessorContext::new(FileFormat::Jpeg, "Canon::SerialData".to_string())
            .with_manufacturer("Canon".to_string())
            .with_model("Canon EOS 5D Mark IV".to_string());

        assert_eq!(
            processor.can_process(&context),
            ProcessorCapability::Incompatible
        );
    }

    #[test]
    fn test_canon_processor_metadata() {
        let processor = CanonSerialDataProcessor;
        let metadata = processor.get_metadata();

        assert_eq!(metadata.name, "Canon Serial Data Processor");
        assert!(metadata
            .supported_manufacturers
            .contains(&"Canon".to_string()));
        assert!(metadata
            .required_context
            .contains(&"manufacturer".to_string()));
    }
}
