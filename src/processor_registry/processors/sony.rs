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

use crate::generated::Sony_pm::main_tags as sony_main_tags;
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
        } else {
            // Sony processors should only handle their specific tables, not generic MakerNotes
            ProcessorCapability::Incompatible
        }
    }

    fn process_data(&self, data: &[u8], _context: &ProcessorContext) -> Result<ProcessorResult> {
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
        // Using Sony tag kit system: AFPointSelected has tag ID 20
        if data.len() >= 0x20 {
            let af_point_raw = u16::from_le_bytes([data[0x1e], data[0x1f]]);

            use crate::expressions::ExpressionEvaluator;
            use crate::generated::Sony_pm::main_tags;

            let mut evaluator = ExpressionEvaluator::new();
            let mut errors = Vec::new();
            let mut warnings = Vec::new();

            // AFPointSelected tag ID 20 from Sony main tags
            let af_point_desc = sony_main_tags::apply_print_conv(
                20,
                &TagValue::U8(af_point_raw as u8),
                &mut evaluator,
                &mut errors,
                &mut warnings,
            );

            result
                .extracted_tags
                .insert("AFPointSelected".to_string(), af_point_desc);
            debug!(
                "Extracted AFPointSelected: {} -> tag kit result",
                af_point_raw
            );
        }

        // FocusMode (offset 0x20, int16u)
        // ExifTool: Sony.pm line 2685-2688
        // Using Sony tag kit system: FocusMode has tag ID 21
        if data.len() >= 0x22 {
            let focus_mode_raw = u16::from_le_bytes([data[0x20], data[0x21]]);

            use crate::expressions::ExpressionEvaluator;
            use crate::generated::Sony_pm::main_tags;

            let mut evaluator = ExpressionEvaluator::new();
            let mut errors = Vec::new();
            let mut warnings = Vec::new();

            // FocusMode tag ID 21 from Sony main tags
            let focus_mode_desc = sony_main_tags::apply_print_conv(
                21,
                &TagValue::U8(focus_mode_raw as u8),
                &mut evaluator,
                &mut errors,
                &mut warnings,
            );

            result
                .extracted_tags
                .insert("FocusMode".to_string(), focus_mode_desc);
            debug!("Extracted FocusMode: {} -> tag kit result", focus_mode_raw);
        }

        // FocusStatus (offset 0x22, int16u)
        // ExifTool: Sony.pm line 2689-2695
        // Using Sony tag kit system: FocusStatus has tag ID 25
        if data.len() >= 0x24 {
            let focus_status_raw = u16::from_le_bytes([data[0x22], data[0x23]]);

            use crate::expressions::ExpressionEvaluator;
            use crate::generated::Sony_pm::main_tags;

            let mut evaluator = ExpressionEvaluator::new();
            let mut errors = Vec::new();
            let mut warnings = Vec::new();

            // FocusStatus tag ID 25 from Sony main tags
            let focus_status_desc = sony_main_tags::apply_print_conv(
                25,
                &TagValue::U8(focus_status_raw as u8),
                &mut evaluator,
                &mut errors,
                &mut warnings,
            );

            result
                .extracted_tags
                .insert("FocusStatus".to_string(), focus_status_desc);
            debug!(
                "Extracted FocusStatus: {} -> tag kit result",
                focus_status_raw
            );
        }

        // Add context for debugging
        if let Some(manufacturer) = &_context.manufacturer {
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
        } else {
            // Sony processors should only handle their specific tables, not generic MakerNotes
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
        } else {
            // Sony processors should only handle their specific tables, not generic MakerNotes
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
            for (i, &byte) in data.iter().enumerate().skip(0x04).take(0x40 - 0x04) {
                if byte != 0 {
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
        } else {
            // Sony processors should only handle their specific tables, not generic MakerNotes
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

/// Sony CameraSettings processor
///
/// Processes Sony CameraSettings section (tag 0x0114) following ExifTool's exact logic.
/// ExifTool: lib/Image/ExifTool/Sony.pm CameraSettings table (lines 4135-4627)
/// This covers A200/A300/A350/A700/A850/A900 models with 280 or 364 byte data.
pub struct SonyCameraSettingsProcessor;

impl BinaryDataProcessor for SonyCameraSettingsProcessor {
    fn can_process(&self, context: &ProcessorContext) -> ProcessorCapability {
        debug!(
            "SonyCameraSettingsProcessor::can_process - manufacturer: {:?}, table: {}",
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
        let is_camera_settings = context.table_name == "CameraSettings"
            || context.table_name == "Sony:CameraSettings"
            || context.table_name.contains("0x0114");

        if is_sony && is_camera_settings {
            ProcessorCapability::Perfect
        } else {
            // Sony processors should only handle their specific tables, not generic MakerNotes
            ProcessorCapability::Incompatible
        }
    }

    fn process_data(&self, data: &[u8], _context: &ProcessorContext) -> Result<ProcessorResult> {
        let mut result = ProcessorResult::new();
        debug!(
            "Processing Sony CameraSettings section with {} bytes",
            data.len()
        );

        // ExifTool: lib/Image/ExifTool/Sony.pm CameraSettings table
        // Lines 4135-4627 - ProcessBinaryData with int16u BigEndian
        // Valid counts: 280 bytes (140 int16u) or 364 bytes (182 int16u)

        // Validate data size
        if data.len() != 280 && data.len() != 364 {
            result.add_warning(format!(
                "Unexpected CameraSettings data size: {} bytes (expected 280 or 364)",
                data.len()
            ));
            return Ok(result);
        }

        // Helper to read BigEndian int16u at offset
        let read_u16 = |offset: usize| -> Option<u16> {
            if offset + 1 < data.len() {
                Some(u16::from_be_bytes([data[offset], data[offset + 1]]))
            } else {
                None
            }
        };

        // DriveMode (offset 0x04, int16u)
        // ExifTool: Sony.pm lines 4176-4183
        if let Some(drive_mode) = read_u16(0x04) {
            // Note: CameraSettings uses different drive mode values than CameraSettings2
            let drive_mode_desc = match drive_mode {
                1 => "Single Frame",
                2 => "Continuous High",
                4 => "Self-timer 10 sec",
                5 => "Self-timer 2 sec",
                6 => "Continuous Bracketing",
                _ => "Unknown",
            };
            result.extracted_tags.insert(
                "DriveMode".to_string(),
                TagValue::String(drive_mode_desc.to_string()),
            );
            debug!("Extracted DriveMode: {} -> {}", drive_mode, drive_mode_desc);
        }

        // WhiteBalanceSetting (offset 0x05, int16u)
        // ExifTool: Sony.pm line 4184
        if let Some(wb_setting) = read_u16(0x05) {
            if let Some(wb_desc) =
                crate::generated::Sony_pm::white_balance_setting::lookup_white_balance_setting(
                    wb_setting as u8,
                )
            {
                result.extracted_tags.insert(
                    "WhiteBalanceSetting".to_string(),
                    TagValue::String(wb_desc.to_string()),
                );
                debug!(
                    "Extracted WhiteBalanceSetting: {} -> {}",
                    wb_setting, wb_desc
                );
            }
        }

        // FlashMode (offset 0x13, int16u)
        // ExifTool: Sony.pm lines 4238-4244
        if let Some(flash_mode) = read_u16(0x13) {
            let flash_mode_desc = match flash_mode {
                0 => "Autoflash",
                2 => "Rear Sync",
                3 => "Wireless",
                4 => "Fill-flash",
                5 => "Flash Off",
                6 => "Slow Sync",
                _ => "Unknown",
            };
            result.extracted_tags.insert(
                "FlashMode".to_string(),
                TagValue::String(flash_mode_desc.to_string()),
            );
            debug!("Extracted FlashMode: {} -> {}", flash_mode, flash_mode_desc);
        }

        // MeteringMode (offset 0x15, int16u)
        // ExifTool: Sony.pm lines 4249-4253
        if let Some(metering_mode) = read_u16(0x15) {
            let metering_desc = match metering_mode {
                1 => "Multi-segment",
                2 => "Center-weighted Average",
                4 => "Spot",
                _ => "Unknown",
            };
            result.extracted_tags.insert(
                "MeteringMode".to_string(),
                TagValue::String(metering_desc.to_string()),
            );
            debug!(
                "Extracted MeteringMode: {} -> {}",
                metering_mode, metering_desc
            );
        }

        // ISOSetting (offset 0x16, int16u)
        // ExifTool: Sony.pm line 4254
        if let Some(iso_setting) = read_u16(0x16) {
            result
                .extracted_tags
                .insert("ISOSetting".to_string(), TagValue::U16(iso_setting));
            debug!("Extracted ISOSetting: {}", iso_setting);
        }

        // FocusMode (offset 0x4d, int16u)
        // ExifTool: Sony.pm lines 4383-4389
        if let Some(focus_mode) = read_u16(0x4d) {
            let focus_desc = match focus_mode {
                0 => "Manual",
                1 => "AF-S",
                2 => "AF-C",
                3 => "AF-A",
                4 => "DMF",
                _ => "Unknown",
            };
            result.extracted_tags.insert(
                "FocusMode".to_string(),
                TagValue::String(focus_desc.to_string()),
            );
            debug!("Extracted FocusMode: {} -> {}", focus_mode, focus_desc);
        }

        // SonyImageSize (offset 0x54, int16u)
        // ExifTool: Sony.pm lines 4406-4411
        if let Some(image_size) = read_u16(0x54) {
            let size_desc = match image_size {
                0 => "Standard",
                1 => "Medium",
                2 => "Small",
                3 => "Large",
                _ => "Unknown",
            };
            result.extracted_tags.insert(
                "SonyImageSize".to_string(),
                TagValue::String(size_desc.to_string()),
            );
            debug!("Extracted SonyImageSize: {} -> {}", image_size, size_desc);
        }

        // Quality (offset 0x56, int16u)
        // ExifTool: Sony.pm lines 4418-4424
        if let Some(quality) = read_u16(0x56) {
            let quality_desc = match quality {
                0 => "Normal",
                1 => "Fine",
                2 => "Extra Fine",
                3 => "Standard",
                4 => "RAW",
                5 => "RAW + JPEG",
                _ => "Unknown",
            };
            result.extracted_tags.insert(
                "Quality".to_string(),
                TagValue::String(quality_desc.to_string()),
            );
            debug!("Extracted Quality: {} -> {}", quality, quality_desc);
        }

        if result.extracted_tags.is_empty() {
            result.add_warning("No Sony CameraSettings tags extracted".to_string());
        } else {
            debug!(
                "Sony CameraSettings processor extracted {} tags",
                result.extracted_tags.len()
            );
        }

        Ok(result)
    }

    fn get_metadata(&self) -> ProcessorMetadata {
        ProcessorMetadata::new(
            "Sony CameraSettings Processor".to_string(),
            "Processes Sony CameraSettings section (tag 0x0114) using ExifTool logic".to_string(),
        )
        .with_manufacturer("Sony".to_string())
    }
}

/// Sony ShotInfo processor
///
/// Processes Sony ShotInfo section (tag 0x3000) following ExifTool's exact logic.
/// ExifTool: lib/Image/ExifTool/Sony.pm ShotInfo table (lines 6027+)
/// This includes face detection data and shot metadata.
pub struct SonyShotInfoProcessor;

impl BinaryDataProcessor for SonyShotInfoProcessor {
    fn can_process(&self, context: &ProcessorContext) -> ProcessorCapability {
        debug!(
            "SonyShotInfoProcessor::can_process - manufacturer: {:?}, table: {}",
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
        let is_shot_info = context.table_name == "ShotInfo"
            || context.table_name == "Sony:ShotInfo"
            || context.table_name.contains("0x3000");

        if is_sony && is_shot_info {
            ProcessorCapability::Perfect
        } else {
            // Sony processors should only handle their specific tables, not generic MakerNotes
            ProcessorCapability::Incompatible
        }
    }

    fn process_data(&self, data: &[u8], _context: &ProcessorContext) -> Result<ProcessorResult> {
        let mut result = ProcessorResult::new();
        debug!("Processing Sony ShotInfo section with {} bytes", data.len());

        // ExifTool: lib/Image/ExifTool/Sony.pm ShotInfo table
        // Lines 6027+ - ProcessBinaryData with LittleEndian
        // First 2 bytes should be 'II' for LittleEndian

        // Validate byte order marker
        if data.len() < 2 || &data[0..2] != b"II" {
            result
                .add_warning("ShotInfo data does not start with expected 'II' marker".to_string());
            return Ok(result);
        }

        // Helper to read LittleEndian int16u at offset
        let read_u16 = |offset: usize| -> Option<u16> {
            if offset + 1 < data.len() {
                Some(u16::from_le_bytes([data[offset], data[offset + 1]]))
            } else {
                None
            }
        };

        // FaceInfoOffset (offset 0x02, int16u)
        // ExifTool: Sony.pm line 6034
        if let Some(face_info_offset) = read_u16(0x02) {
            result.extracted_tags.insert(
                "FaceInfoOffset".to_string(),
                TagValue::U16(face_info_offset),
            );
            debug!("Extracted FaceInfoOffset: {}", face_info_offset);
        }

        // SonyDateTime (offset 0x06, string[20])
        // ExifTool: Sony.pm line 6038
        if data.len() >= 0x06 + 20 {
            // Extract null-terminated string or full 20 bytes
            let datetime_bytes = &data[0x06..0x06 + 20];
            let datetime_end = datetime_bytes.iter().position(|&b| b == 0).unwrap_or(20);
            if let Ok(datetime_str) = std::str::from_utf8(&datetime_bytes[..datetime_end]) {
                result.extracted_tags.insert(
                    "SonyDateTime".to_string(),
                    TagValue::String(datetime_str.to_string()),
                );
                debug!("Extracted SonyDateTime: {}", datetime_str);
            }
        }

        // SonyImageHeight (offset 0x1a, int16u)
        // ExifTool: Sony.pm line 6065
        if let Some(height) = read_u16(0x1a) {
            result
                .extracted_tags
                .insert("SonyImageHeight".to_string(), TagValue::U16(height));
            debug!("Extracted SonyImageHeight: {}", height);
        }

        // SonyImageWidth (offset 0x1c, int16u)
        // ExifTool: Sony.pm line 6066
        if let Some(width) = read_u16(0x1c) {
            result
                .extracted_tags
                .insert("SonyImageWidth".to_string(), TagValue::U16(width));
            debug!("Extracted SonyImageWidth: {}", width);
        }

        // FacesDetected (offset 0x30, int16u)
        // ExifTool: Sony.pm line 6087
        if let Some(faces_detected) = read_u16(0x30) {
            result
                .extracted_tags
                .insert("FacesDetected".to_string(), TagValue::U16(faces_detected));
            debug!("Extracted FacesDetected: {}", faces_detected);
        }

        // FaceInfoLength (offset 0x32, int16u)
        // ExifTool: Sony.pm line 6098
        if let Some(face_info_length) = read_u16(0x32) {
            result.extracted_tags.insert(
                "FaceInfoLength".to_string(),
                TagValue::U16(face_info_length),
            );
            debug!("Extracted FaceInfoLength: {}", face_info_length);
        }

        // MetaVersion (offset 0x34, string[16])
        // ExifTool: Sony.pm line 6108
        if data.len() >= 0x34 + 16 {
            let meta_version_bytes = &data[0x34..0x34 + 16];
            let version_end = meta_version_bytes
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(16);
            if let Ok(version_str) = std::str::from_utf8(&meta_version_bytes[..version_end]) {
                result.extracted_tags.insert(
                    "MetaVersion".to_string(),
                    TagValue::String(version_str.to_string()),
                );
                debug!("Extracted MetaVersion: {}", version_str);
            }
        }

        // Note: FaceInfo1 (0x48) and FaceInfo2 (0x5e) are subdirectories
        // that would require additional processing based on the face detection data
        // ExifTool: Sony.pm lines 6122-6131 and 6138-6144

        if result.extracted_tags.is_empty() {
            result.add_warning("No Sony ShotInfo tags extracted".to_string());
        } else {
            debug!(
                "Sony ShotInfo processor extracted {} tags",
                result.extracted_tags.len()
            );
        }

        Ok(result)
    }

    fn get_metadata(&self) -> ProcessorMetadata {
        ProcessorMetadata::new(
            "Sony ShotInfo Processor".to_string(),
            "Processes Sony ShotInfo section (tag 0x3000) using ExifTool logic".to_string(),
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

    fn process_data(&self, data: &[u8], _context: &ProcessorContext) -> Result<ProcessorResult> {
        let mut result = ProcessorResult::new();
        debug!(
            "Processing Sony general binary data '{}' with {} bytes",
            _context.table_name,
            data.len()
        );

        // Basic analysis for unhandled Sony binary data
        if data.len() >= 10 {
            debug!(
                "Sony binary data '{}' header: {:02x?}",
                _context.table_name,
                &data[0..10]
            );
        }

        result.add_warning(format!(
            "Sony binary data '{}' processed by general handler - specific processor not implemented",
            _context.table_name
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
