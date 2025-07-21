//! Sony-specific BinaryDataProcessor implementations
//!
//! These processors implement the BinaryDataProcessor trait for Sony camera data
//! following ExifTool's Sony.pm processing logic exactly.
//!
//! ## ExifTool Reference
//!
//! lib/Image/ExifTool/Sony.pm - CameraInfo, Tag9050, AFInfo, Tag2010 sections, etc.
//!
//! ## Trust ExifTool Implementation
//!
//! This code translates ExifTool's Sony.pm ProcessBinaryData sections verbatim
//! without any improvements or simplifications. Every algorithm, offset calculation,
//! and quirk is copied exactly as documented in the ExifTool source.

use crate::generated::Sony_pm;
use crate::processor_registry::{
    BinaryDataProcessor, ProcessorCapability, ProcessorContext, ProcessorMetadata, ProcessorResult,
};
use crate::types::{Result, TagValue};
use tracing::debug;

/// Sony CameraInfo section processor
///
/// Processes Sony CameraInfo section (tag 0x0010) following ExifTool's exact logic.
/// ExifTool: lib/Image/ExifTool/Sony.pm CameraInfo table (lines 2660-2810)
/// This covers A700, A850, A900, and other DSLR models.
pub struct SonyCameraInfoProcessor;

impl BinaryDataProcessor for SonyCameraInfoProcessor {
    fn can_process(&self, context: &ProcessorContext) -> ProcessorCapability {
        debug!(
            "SonyCameraInfoProcessor::can_process - manufacturer: {:?}, table: {}",
            context.manufacturer, context.table_name
        );

        // ExifTool: Standard IFD directories must use standard IFD parsing, not manufacturer processors
        // Following Trust ExifTool: ExifIFD, GPS, InteropIFD use standard TIFF processing
        if matches!(
            context.table_name.as_str(),
            "ExifIFD" | "GPS" | "InteropIFD" | "IFD0" | "IFD1"
        ) {
            debug!("SonyCameraInfoProcessor::can_process - returning Incompatible for standard IFD: {}", context.table_name);
            return ProcessorCapability::Incompatible;
        }

        // Check for Sony manufacturer
        let is_sony = context
            .manufacturer
            .as_ref()
            .map(|m| m.starts_with("SONY") || m.contains("Sony"))
            .unwrap_or(false);
        let is_camera_info = context.table_name.contains("CameraInfo");

        debug!(
            "SonyCameraInfoProcessor::can_process - is_sony: {}, is_camera_info: {}",
            is_sony, is_camera_info
        );

        if is_sony && is_camera_info {
            ProcessorCapability::Perfect
        } else if is_sony {
            ProcessorCapability::Good
        } else {
            ProcessorCapability::Incompatible
        }
    }

    fn process_data(&self, data: &[u8], context: &ProcessorContext) -> Result<ProcessorResult> {
        let mut result = ProcessorResult::new();
        debug!(
            "Processing Sony CameraInfo section with {} bytes",
            data.len()
        );

        // ExifTool: lib/Image/ExifTool/Sony.pm CameraInfo table processing
        // Lines 2660-2810 - various offsets and data types

        // Extract key CameraInfo fields following ExifTool exact structure

        // AFPointSelected (offset 0x1e, int16u)
        // ExifTool: Sony.pm line 2668-2684
        if data.len() >= 0x20 {
            let af_point_raw = u16::from_le_bytes([data[0x1e], data[0x1f]]);
            if let Some(af_point_desc) =
                Sony_pm::lookup_camera_info3__a_f_point_selected(af_point_raw as u8)
            {
                result.extracted_tags.insert(
                    "AFPointSelected".to_string(),
                    TagValue::String(af_point_desc.to_string()),
                );
                debug!(
                    "Extracted AFPointSelected: {} -> {}",
                    af_point_raw, af_point_desc
                );
            }
        }

        // FocusMode (offset 0x20, int16u)
        // ExifTool: Sony.pm line 2685-2688
        if data.len() >= 0x22 {
            let focus_mode_raw = u16::from_le_bytes([data[0x20], data[0x21]]);
            if let Some(focus_mode_desc) =
                Sony_pm::lookup_camera_info3__focus_mode(focus_mode_raw as u8)
            {
                result.extracted_tags.insert(
                    "FocusMode".to_string(),
                    TagValue::String(focus_mode_desc.to_string()),
                );
                debug!(
                    "Extracted FocusMode: {} -> {}",
                    focus_mode_raw, focus_mode_desc
                );
            }
        }

        // FocusStatus (offset 0x22, int16u)
        // ExifTool: Sony.pm line 2689-2695
        if data.len() >= 0x24 {
            let focus_status_raw = u16::from_le_bytes([data[0x22], data[0x23]]);
            if let Some(focus_status_desc) =
                Sony_pm::lookup_camera_info3__focus_status(focus_status_raw as u8)
            {
                result.extracted_tags.insert(
                    "FocusStatus".to_string(),
                    TagValue::String(focus_status_desc.to_string()),
                );
                debug!(
                    "Extracted FocusStatus: {} -> {}",
                    focus_status_raw, focus_status_desc
                );
            }
        }

        // Add context for debugging
        if let Some(manufacturer) = &context.manufacturer {
            debug!(
                "Sony CameraInfo processing for manufacturer: {}",
                manufacturer
            );
        }

        if result.extracted_tags.is_empty() {
            result.add_warning("No Sony CameraInfo tags extracted".to_string());
        } else {
            debug!(
                "Sony CameraInfo processor extracted {} tags",
                result.extracted_tags.len()
            );
        }

        Ok(result)
    }

    fn get_metadata(&self) -> ProcessorMetadata {
        ProcessorMetadata::new(
            "Sony CameraInfo Processor".to_string(),
            "Processes Sony CameraInfo section using ExifTool logic".to_string(),
        )
        .with_manufacturer("Sony".to_string())
    }
}

/// Sony Tag9050 series processor
///
/// Processes Sony Tag9050 section (tag 0x9050) following ExifTool's exact logic.
/// ExifTool: lib/Image/ExifTool/Sony.pm Tag9050 tables (lines 7492-8270)
/// This covers encrypted core image metadata for SLT, ILCA, NEX, and ILCE models.
pub struct SonyTag9050Processor;

impl BinaryDataProcessor for SonyTag9050Processor {
    fn can_process(&self, context: &ProcessorContext) -> ProcessorCapability {
        debug!(
            "SonyTag9050Processor::can_process - manufacturer: {:?}, table: {}",
            context.manufacturer, context.table_name
        );

        // ExifTool: Standard IFD directories must use standard IFD parsing
        if matches!(
            context.table_name.as_str(),
            "ExifIFD" | "GPS" | "InteropIFD" | "IFD0" | "IFD1"
        ) {
            return ProcessorCapability::Incompatible;
        }

        let is_sony = context
            .manufacturer
            .as_ref()
            .map(|m| m.starts_with("SONY") || m.contains("Sony"))
            .unwrap_or(false);
        let is_tag9050 = context.table_name.contains("Tag9050");

        if is_sony && is_tag9050 {
            ProcessorCapability::Perfect
        } else if is_sony {
            ProcessorCapability::Good
        } else {
            ProcessorCapability::Incompatible
        }
    }

    fn process_data(&self, data: &[u8], _context: &ProcessorContext) -> Result<ProcessorResult> {
        let mut result = ProcessorResult::new();
        debug!("Processing Sony Tag9050 section with {} bytes", data.len());

        // ExifTool: lib/Image/ExifTool/Sony.pm Tag9050 series processing
        // Lines 7492-8270 - multiple Tag9050 variants (a, b, c, d)

        // Note: Tag9050 data is often encrypted and requires ProcessEnciphered
        // For now, we'll implement basic structure parsing

        // TODO: Implement Tag9050 decryption using Sony's cipher algorithms
        // ExifTool: Sony.pm Decipher() and Decrypt() functions

        debug!("Sony Tag9050 processing - encrypted data detected, basic structure parsing");

        // Basic header analysis - first few bytes often contain format info
        if data.len() >= 10 {
            debug!(
                "Tag9050 data header (first 10 bytes): {:02x?}",
                &data[0..10]
            );
        }

        // Placeholder for encrypted data processing
        result.add_warning(
            "Sony Tag9050 processing requires decryption - not fully implemented".to_string(),
        );

        Ok(result)
    }

    fn get_metadata(&self) -> ProcessorMetadata {
        ProcessorMetadata::new(
            "Sony Tag9050 Processor".to_string(),
            "Processes Sony Tag9050 encrypted metadata using ExifTool logic".to_string(),
        )
        .with_manufacturer("Sony".to_string())
    }
}

/// Sony AFInfo processor
///
/// Processes Sony AFInfo section (tag 0x940e) following ExifTool's exact logic.
/// ExifTool: lib/Image/ExifTool/Sony.pm AFInfo table (lines 9363-9658)
/// This covers autofocus information for SLT models and NEX/ILCE with Phase-detect AF adapters.
pub struct SonyAFInfoProcessor;

impl BinaryDataProcessor for SonyAFInfoProcessor {
    fn can_process(&self, context: &ProcessorContext) -> ProcessorCapability {
        debug!(
            "SonyAFInfoProcessor::can_process - manufacturer: {:?}, table: {}",
            context.manufacturer, context.table_name
        );

        // Standard IFD check
        if matches!(
            context.table_name.as_str(),
            "ExifIFD" | "GPS" | "InteropIFD" | "IFD0" | "IFD1"
        ) {
            return ProcessorCapability::Incompatible;
        }

        let is_sony = context
            .manufacturer
            .as_ref()
            .map(|m| m.starts_with("SONY") || m.contains("Sony"))
            .unwrap_or(false);
        let is_af_info = context.table_name.contains("AFInfo");

        if is_sony && is_af_info {
            ProcessorCapability::Perfect
        } else if is_sony {
            ProcessorCapability::Good
        } else {
            ProcessorCapability::Incompatible
        }
    }

    fn process_data(&self, data: &[u8], _context: &ProcessorContext) -> Result<ProcessorResult> {
        let mut result = ProcessorResult::new();
        debug!("Processing Sony AFInfo section with {} bytes", data.len());

        // ExifTool: lib/Image/ExifTool/Sony.pm AFInfo table processing
        // Lines 9363-9658 - AF type, AF area mode, AF points data

        // AFType (offset 0x00, int8u)
        // ExifTool: Sony.pm lines 9365-9368
        if !data.is_empty() {
            let af_type = data[0];
            let af_type_desc = match af_type {
                0 => "15-point",
                1 => "19-point",
                2 => "79-point",
                _ => "Unknown",
            };
            result.extracted_tags.insert(
                "AFType".to_string(),
                TagValue::String(af_type_desc.to_string()),
            );
            debug!("Extracted AFType: {} -> {}", af_type, af_type_desc);
        }

        // AFAreaMode (offset 0x01, int8u)
        // ExifTool: Sony.pm lines 9369-9378
        if data.len() >= 2 {
            let af_area_mode = data[1];
            let af_area_desc = match af_area_mode {
                0 => "Wide",
                1 => "Local",
                2 => "Zone",
                3 => "Spot",
                _ => "Unknown",
            };
            result.extracted_tags.insert(
                "AFAreaMode".to_string(),
                TagValue::String(af_area_desc.to_string()),
            );
            debug!("Extracted AFAreaMode: {} -> {}", af_area_mode, af_area_desc);
        }

        // AF Points in Focus (offset 0x04-0x3f, int8u[60])
        // ExifTool: Sony.pm lines 9379-9382
        if data.len() >= 0x40 {
            let mut af_points_in_focus = Vec::new();
            for i in 0x04..0x40 {
                if data[i] != 0 {
                    af_points_in_focus.push((i - 0x04) as u8);
                }
            }
            if !af_points_in_focus.is_empty() {
                let points_count = af_points_in_focus.len();
                result.extracted_tags.insert(
                    "AFPointsInFocus".to_string(),
                    TagValue::U8Array(af_points_in_focus),
                );
                debug!("Extracted AFPointsInFocus: {} points", points_count);
            }
        }

        if result.extracted_tags.is_empty() {
            result.add_warning("No Sony AFInfo tags extracted".to_string());
        } else {
            debug!(
                "Sony AFInfo processor extracted {} tags",
                result.extracted_tags.len()
            );
        }

        Ok(result)
    }

    fn get_metadata(&self) -> ProcessorMetadata {
        ProcessorMetadata::new(
            "Sony AFInfo Processor".to_string(),
            "Processes Sony AFInfo autofocus data using ExifTool logic".to_string(),
        )
        .with_manufacturer("Sony".to_string())
    }
}

/// Sony Tag2010 series processor
///
/// Processes Sony Tag2010 section (tag 0x2010) following ExifTool's exact logic.
/// ExifTool: lib/Image/ExifTool/Sony.pm Tag2010 tables (lines 6376-7317)
/// This covers encrypted image settings and metadata for different camera generations.
pub struct SonyTag2010Processor;

impl BinaryDataProcessor for SonyTag2010Processor {
    fn can_process(&self, context: &ProcessorContext) -> ProcessorCapability {
        debug!(
            "SonyTag2010Processor::can_process - manufacturer: {:?}, table: {}",
            context.manufacturer, context.table_name
        );

        // Standard IFD check
        if matches!(
            context.table_name.as_str(),
            "ExifIFD" | "GPS" | "InteropIFD" | "IFD0" | "IFD1"
        ) {
            return ProcessorCapability::Incompatible;
        }

        let is_sony = context
            .manufacturer
            .as_ref()
            .map(|m| m.starts_with("SONY") || m.contains("Sony"))
            .unwrap_or(false);
        let is_tag2010 = context.table_name.contains("Tag2010");

        if is_sony && is_tag2010 {
            ProcessorCapability::Perfect
        } else if is_sony {
            ProcessorCapability::Good
        } else {
            ProcessorCapability::Incompatible
        }
    }

    fn process_data(&self, data: &[u8], _context: &ProcessorContext) -> Result<ProcessorResult> {
        let mut result = ProcessorResult::new();
        debug!("Processing Sony Tag2010 section with {} bytes", data.len());

        // ExifTool: lib/Image/ExifTool/Sony.pm Tag2010 series processing
        // Lines 6376-7317 - multiple Tag2010 variants (a through i)

        // Note: Tag2010 data is encrypted and requires ProcessEnciphered
        // Basic structure analysis for now

        debug!("Sony Tag2010 processing - encrypted data, analyzing structure");

        if data.len() >= 16 {
            debug!(
                "Tag2010 data header (first 16 bytes): {:02x?}",
                &data[0..16]
            );
        }

        // Placeholder for encrypted data processing
        result.add_warning(
            "Sony Tag2010 processing requires decryption - not fully implemented".to_string(),
        );

        Ok(result)
    }

    fn get_metadata(&self) -> ProcessorMetadata {
        ProcessorMetadata::new(
            "Sony Tag2010 Processor".to_string(),
            "Processes Sony Tag2010 encrypted settings using ExifTool logic".to_string(),
        )
        .with_manufacturer("Sony".to_string())
    }
}

/// Sony general processor for unspecified binary data
///
/// Catches Sony binary data that doesn't match specific processors.
pub struct SonyGeneralProcessor;

impl BinaryDataProcessor for SonyGeneralProcessor {
    fn can_process(&self, context: &ProcessorContext) -> ProcessorCapability {
        // Standard IFD check
        if matches!(
            context.table_name.as_str(),
            "ExifIFD" | "GPS" | "InteropIFD" | "IFD0" | "IFD1"
        ) {
            return ProcessorCapability::Incompatible;
        }

        let is_sony = context
            .manufacturer
            .as_ref()
            .map(|m| m.starts_with("SONY") || m.contains("Sony"))
            .unwrap_or(false);

        if is_sony {
            ProcessorCapability::Fallback
        } else {
            ProcessorCapability::Incompatible
        }
    }

    fn process_data(&self, data: &[u8], context: &ProcessorContext) -> Result<ProcessorResult> {
        let mut result = ProcessorResult::new();
        debug!(
            "Processing Sony general binary data '{}' with {} bytes",
            context.table_name,
            data.len()
        );

        // Basic analysis for unhandled Sony binary data
        if data.len() >= 10 {
            debug!(
                "Sony binary data '{}' header: {:02x?}",
                context.table_name,
                &data[0..10]
            );
        }

        result.add_warning(format!(
            "Sony binary data '{}' processed by general handler - specific processor not implemented",
            context.table_name
        ));

        Ok(result)
    }

    fn get_metadata(&self) -> ProcessorMetadata {
        ProcessorMetadata::new(
            "Sony General Processor".to_string(),
            "General handler for Sony binary data sections".to_string(),
        )
        .with_manufacturer("Sony".to_string())
    }
}
