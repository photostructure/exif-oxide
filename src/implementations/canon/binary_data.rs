//! Canon binary data processing for CameraSettings and related tables
//!
//! This module handles Canon's binary data table format, particularly the CameraSettings
//! table that uses the 'int16s' format with 1-based indexing.
//!
//! **ExifTool is Gospel**: This code translates ExifTool's Canon.pm binary data processing
//! verbatim, including all PrintConv tables and processing logic.
//!
//! Primary ExifTool References:
//! - lib/Image/ExifTool/Canon.pm:2166+ %Canon::CameraSettings table
//! - ExifTool's ProcessBinaryData with FORMAT => 'int16s', FIRST_ENTRY => 1

use crate::tiff_types::ByteOrder;
use crate::types::{Result, TagValue};
use std::collections::HashMap;
use tracing::debug;

/// Canon CameraSettings binary data tag definition
/// ExifTool: lib/Image/ExifTool/Canon.pm:2166-2240+ %Canon::CameraSettings
#[derive(Debug, Clone)]
pub struct CanonCameraSettingsTag {
    /// Tag index (1-based like ExifTool FIRST_ENTRY => 1)
    pub index: u32,
    /// Tag name
    pub name: String,
    /// PrintConv lookup table for human-readable values
    pub print_conv: Option<HashMap<i16, String>>,
}

/// Create Canon CameraSettings binary data table
/// ExifTool: lib/Image/ExifTool/Canon.pm:2166+ %Canon::CameraSettings
pub fn create_camera_settings_table() -> HashMap<u32, CanonCameraSettingsTag> {
    let mut table = HashMap::new();

    // ExifTool: Canon.pm:2172-2178 tag 1 MacroMode
    table.insert(
        1,
        CanonCameraSettingsTag {
            index: 1,
            name: "MacroMode".to_string(),
            print_conv: {
                let mut conv = HashMap::new();
                conv.insert(1, "Macro".to_string());
                conv.insert(2, "Normal".to_string());
                Some(conv)
            },
        },
    );

    // ExifTool: Canon.pm:2179-2191 tag 2 SelfTimer
    table.insert(
        2,
        CanonCameraSettingsTag {
            index: 2,
            name: "SelfTimer".to_string(),
            print_conv: {
                // Note: SelfTimer has complex Perl PrintConv logic
                // For now, implementing basic Off detection
                // TODO: Implement full PrintConv logic from Canon.pm:2182-2185
                let mut conv = HashMap::new();
                conv.insert(0, "Off".to_string());
                Some(conv)
            },
        },
    );

    // ExifTool: Canon.pm:2192-2195 tag 3 Quality
    table.insert(
        3,
        CanonCameraSettingsTag {
            index: 3,
            name: "Quality".to_string(),
            print_conv: {
                // Note: Quality uses %canonQuality hash reference
                // TODO: Implement canonQuality lookup table
                None // Placeholder for now
            },
        },
    );

    // ExifTool: Canon.pm:2196-2209 tag 4 CanonFlashMode
    table.insert(
        4,
        CanonCameraSettingsTag {
            index: 4,
            name: "CanonFlashMode".to_string(),
            print_conv: {
                let mut conv = HashMap::new();
                conv.insert(-1, "n/a".to_string()); // PH, EOS M MOV video
                conv.insert(0, "Off".to_string());
                conv.insert(1, "Auto".to_string());
                conv.insert(2, "On".to_string());
                conv.insert(3, "Red-eye reduction".to_string());
                conv.insert(4, "Slow-sync".to_string());
                conv.insert(5, "Red-eye reduction (Auto)".to_string());
                conv.insert(6, "Red-eye reduction (On)".to_string());
                conv.insert(16, "External flash".to_string()); // not set in D30 or 300D
                Some(conv)
            },
        },
    );

    // ExifTool: Canon.pm:2210-2227 tag 5 ContinuousDrive
    table.insert(
        5,
        CanonCameraSettingsTag {
            index: 5,
            name: "ContinuousDrive".to_string(),
            print_conv: {
                let mut conv = HashMap::new();
                conv.insert(0, "Single".to_string());
                conv.insert(1, "Continuous".to_string());
                conv.insert(2, "Movie".to_string()); // PH
                conv.insert(3, "Continuous, Speed Priority".to_string()); // PH
                Some(conv)
            },
        },
    );

    // ExifTool: Canon.pm:2228-2240 tag 7 FocusMode
    table.insert(
        7,
        CanonCameraSettingsTag {
            index: 7,
            name: "FocusMode".to_string(),
            print_conv: {
                let mut conv = HashMap::new();
                conv.insert(0, "One-shot AF".to_string());
                conv.insert(1, "AI Servo AF".to_string());
                conv.insert(2, "AI Focus AF".to_string());
                conv.insert(3, "Manual Focus (3)".to_string());
                conv.insert(4, "Single".to_string());
                conv.insert(5, "Continuous".to_string());
                conv.insert(6, "Manual Focus (6)".to_string());
                conv.insert(16, "Pan Focus".to_string()); // PH
                Some(conv)
            },
        },
    );

    table
}

/// Extract Canon CameraSettings binary data
/// ExifTool: ProcessBinaryData with Canon CameraSettings table parameters
///
/// Table parameters from Canon.pm:2166-2171:
/// - FORMAT => 'int16s' (signed 16-bit integers)
/// - FIRST_ENTRY => 1 (1-indexed)
/// - GROUPS => { 0 => 'MakerNotes', 2 => 'Camera' }
pub fn extract_camera_settings(
    data: &[u8],
    offset: usize,
    size: usize,
    byte_order: ByteOrder,
) -> Result<HashMap<String, TagValue>> {
    let table = create_camera_settings_table();
    let mut results = HashMap::new();

    // ExifTool: Canon.pm:2168 FORMAT => 'int16s'
    let format_size = 2; // int16s = 2 bytes

    debug!(
        "Extracting Canon CameraSettings: offset={:#x}, size={}, format=int16s",
        offset, size
    );

    // Process defined tags
    for (&index, tag_def) in &table {
        // ExifTool: Canon.pm:2169 FIRST_ENTRY => 1 (1-indexed)
        let entry_offset = (index - 1) as usize * format_size;

        if entry_offset + format_size > size {
            debug!("Tag {} at index {} beyond data bounds", tag_def.name, index);
            continue;
        }

        let data_offset = offset + entry_offset;

        if data_offset + format_size > data.len() {
            debug!(
                "Tag {} data offset {:#x} beyond buffer bounds",
                tag_def.name, data_offset
            );
            continue;
        }

        // Extract int16s value (signed 16-bit integer)
        let raw_value = byte_order.read_u16(data, data_offset)? as i16;

        // Apply PrintConv if available
        let final_value = if let Some(print_conv) = &tag_def.print_conv {
            if let Some(converted) = print_conv.get(&raw_value) {
                TagValue::String(converted.clone())
            } else {
                TagValue::I16(raw_value)
            }
        } else {
            TagValue::I16(raw_value)
        };

        debug!(
            "Extracted Canon {} = {:?} (raw: {}) at index {}",
            tag_def.name, final_value, raw_value, index
        );

        // Store with MakerNotes group prefix like ExifTool
        // ExifTool: Canon.pm:2171 GROUPS => { 0 => 'MakerNotes', 2 => 'Camera' }
        let tag_name = format!("MakerNotes:{}", tag_def.name);
        results.insert(tag_name, final_value);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_camera_settings_table() {
        let table = create_camera_settings_table();

        // Test that expected tags are present
        assert!(table.contains_key(&1)); // MacroMode
        assert!(table.contains_key(&2)); // SelfTimer
        assert!(table.contains_key(&3)); // Quality
        assert!(table.contains_key(&4)); // CanonFlashMode
        assert!(table.contains_key(&5)); // ContinuousDrive
        assert!(table.contains_key(&7)); // FocusMode

        // Test tag structure
        let macro_mode = table.get(&1).unwrap();
        assert_eq!(macro_mode.name, "MacroMode");
        assert!(macro_mode.print_conv.is_some());

        let print_conv = macro_mode.print_conv.as_ref().unwrap();
        assert_eq!(print_conv.get(&1), Some(&"Macro".to_string()));
        assert_eq!(print_conv.get(&2), Some(&"Normal".to_string()));
    }

    #[test]
    fn test_extract_camera_settings_basic() {
        // Create test data: two int16s values
        let test_data = vec![0x00, 0x01, 0x00, 0x02]; // [1, 2] in big-endian

        let result = extract_camera_settings(&test_data, 0, 4, ByteOrder::BigEndian);
        assert!(result.is_ok());

        let tags = result.unwrap();

        // Should have extracted MacroMode (index 1) and SelfTimer (index 2)
        assert!(tags.contains_key("MakerNotes:MacroMode"));
        assert!(tags.contains_key("MakerNotes:SelfTimer"));

        // MacroMode value 1 should be converted to "Macro"
        if let Some(TagValue::String(value)) = tags.get("MakerNotes:MacroMode") {
            assert_eq!(value, "Macro");
        } else {
            panic!("MacroMode should be converted to string");
        }

        // SelfTimer value 2 should remain as I16 (no PrintConv for value 2)
        if let Some(TagValue::I16(value)) = tags.get("MakerNotes:SelfTimer") {
            assert_eq!(*value, 2);
        } else {
            panic!("SelfTimer should be I16 for unconverted value");
        }
    }
}
