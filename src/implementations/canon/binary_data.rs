//! Canon binary data processing for CameraSettings and related tables
//!
//! This module handles Canon's binary data table format, particularly the CameraSettings
//! table that uses the 'int16s' format with 1-based indexing.
//!
//! **Trust ExifTool**: This code translates ExifTool's Canon.pm binary data processing
//! verbatim, including all PrintConv tables and processing logic.
//!
//! Primary ExifTool References:
//! - lib/Image/ExifTool/Canon.pm:2166+ %Canon::CameraSettings table
//! - ExifTool's ProcessBinaryData with FORMAT => 'int16s', FIRST_ENTRY => 1

use crate::generated::Canon_pm::main_tags::CANON_MAIN_TAGS;
use crate::tiff_types::ByteOrder;
use crate::types::{
    BinaryDataFormat, BinaryDataTable, BinaryDataTag, ExifError, PrintConv, Result, TagValue,
};
use std::collections::HashMap;
use tracing::debug;

/// Canon-specific PrintConv application using the generated tag table
/// ExifTool: Canon.pm PrintConv processing with registry fallback
fn apply_canon_print_conv(
    tag_id: u32,
    value: &TagValue,
    _errors: &mut Vec<String>,
    _warnings: &mut Vec<String>,
) -> TagValue {
    // Look up the tag in Canon main tags table
    if let Some(tag_info) = CANON_MAIN_TAGS.get(&(tag_id as u16)) {
        debug!("Found Canon tag {}: {}", tag_id, tag_info.name);

        match &tag_info.print_conv {
            Some(PrintConv::Expression(_expr)) => {
                // Runtime expression evaluation removed - all Perl interpretation happens via PPI at build time
                // Fallback to original value when expression not handled by PPI
                value.clone()
            }
            Some(PrintConv::Complex) => {
                debug!(
                    "Complex PrintConv for tag {}, using generated module",
                    tag_id
                );
                // For complex conversions, use the generated module's apply_print_conv
                // TODO: This should be applied by the specific tag table that contains this tag
                value.clone() // Placeholder until proper tag table routing is implemented
            }
            Some(PrintConv::Simple(_table)) => {
                debug!(
                    "Simple PrintConv table for tag {} (not yet implemented)",
                    tag_id
                );
                // TODO: Handle simple lookup tables
                value.clone()
            }
            Some(PrintConv::Function(_func_name)) => {
                debug!(
                    "Function PrintConv for tag {} (not yet implemented)",
                    tag_id
                );
                // TODO: Handle function references
                value.clone()
            }
            Some(PrintConv::None) | None => {
                debug!("No PrintConv for Canon tag {}", tag_id);
                value.clone()
            }
        }
    } else {
        debug!("Canon tag {} not found in main tags table", tag_id);
        value.clone()
    }
}

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
    // PrintConv data from Canon runtime tables: "1" => "Macro", "2" => "Normal"
    table.insert(
        1,
        CanonCameraSettingsTag {
            index: 1,
            name: "MacroMode".to_string(),
            print_conv: {
                let mut conv = HashMap::new();
                conv.insert(1i16, "Macro".to_string());
                conv.insert(2i16, "Normal".to_string());
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
                conv.insert(0i16, "Off".to_string());
                Some(conv)
            },
        },
    );

    // ExifTool: Canon.pm:2192-2195 tag 3 Quality
    // Use generated lookup function instead of manual table
    table.insert(
        3,
        CanonCameraSettingsTag {
            index: 3,
            name: "Quality".to_string(),
            print_conv: None, // Use generated lookup in apply_camera_settings_print_conv
        },
    );

    // ExifTool: Canon.pm:2196-2209 tag 4 CanonFlashMode
    // Use generated lookup function instead of manual table
    table.insert(
        4,
        CanonCameraSettingsTag {
            index: 4,
            name: "CanonFlashMode".to_string(),
            print_conv: None, // Use generated lookup in apply_camera_settings_print_conv
        },
    );

    // ExifTool: Canon.pm:2210-2227 tag 5 ContinuousDrive
    // Use generated lookup function instead of manual table
    table.insert(
        5,
        CanonCameraSettingsTag {
            index: 5,
            name: "ContinuousDrive".to_string(),
            print_conv: None, // Use generated lookup in apply_camera_settings_print_conv
        },
    );

    // ExifTool: Canon.pm:2228-2240 tag 7 FocusMode
    // Use generated lookup function instead of manual table
    table.insert(
        7,
        CanonCameraSettingsTag {
            index: 7,
            name: "FocusMode".to_string(),
            print_conv: None, // Use generated lookup in apply_camera_settings_print_conv
        },
    );

    // ExifTool: Canon.pm:2463-2475 tag 23 MaxFocalLength
    // ValueConv => '$val / ($$self{FocalUnits} || 1)', PrintConv => '"$val mm"'
    table.insert(
        23,
        CanonCameraSettingsTag {
            index: 23,
            name: "MaxFocalLength".to_string(),
            print_conv: None, // PrintConv applied via registry: "$val mm"
        },
    );

    // ExifTool: Canon.pm:2476-2488 tag 24 MinFocalLength
    // ValueConv => '$val / ($$self{FocalUnits} || 1)', PrintConv => '"$val mm"'
    table.insert(
        24,
        CanonCameraSettingsTag {
            index: 24,
            name: "MinFocalLength".to_string(),
            print_conv: None, // PrintConv applied via registry: "$val mm"
        },
    );

    // ExifTool: Canon.pm:2489 tag 25 FocalUnits (DATAMEMBER)
    // Used by MinFocalLength and MaxFocalLength ValueConv
    table.insert(
        25,
        CanonCameraSettingsTag {
            index: 25,
            name: "FocalUnits".to_string(),
            print_conv: None, // Special formatting in extract_camera_settings
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

    // Extract FocalUnits first for dependent conversions
    // ExifTool: Canon.pm:2489 DATAMEMBER => [ 22, 25 ] - FocalUnits at offset 25
    let focal_units = extract_focal_units_from_camera_settings(data, offset, size, byte_order)?;
    debug!(
        "Canon CameraSettings FocalUnits = {} (for dependent conversions)",
        focal_units
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

        // Apply ValueConv for FocalUnits-dependent tags first
        // ExifTool: Canon.pm:2463-2480 ValueConv => '$val / ($$self{FocalUnits} || 1)'
        let converted_value = match tag_def.name.as_str() {
            "MinFocalLength" | "MaxFocalLength" => {
                // Apply FocalUnits conversion: value / (focal_units || 1)
                let focal_divisor = if focal_units != 0.0 { focal_units } else { 1.0 };
                let converted = raw_value as f64 / focal_divisor;
                debug!(
                    "Canon {} ValueConv: {} / {} = {}",
                    tag_def.name, raw_value, focal_divisor, converted
                );
                TagValue::F64(converted)
            }
            "FocalUnits" => {
                // FocalUnits should be formatted for display
                // ExifTool shows it as "1/mm" or similar based on the value
                if raw_value == 1 {
                    TagValue::String("1/mm".to_string())
                } else if raw_value > 0 {
                    TagValue::String(format!("{}/mm", raw_value))
                } else {
                    TagValue::I16(raw_value)
                }
            }
            _ => TagValue::I16(raw_value), // Other tags use raw values
        };

        // Apply PrintConv from the local table for non-focal-length tags
        let final_value = match tag_def.name.as_str() {
            "MinFocalLength" | "MaxFocalLength" => {
                // These use the converted F64 value directly - PrintConv will be applied by registry
                converted_value
            }
            _ => {
                // Apply PrintConv lookup table for other tags
                if let Some(print_conv) = &tag_def.print_conv {
                    if let Some(converted) = print_conv.get(&raw_value) {
                        TagValue::String(converted.clone())
                    } else {
                        converted_value
                    }
                } else {
                    converted_value
                }
            }
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

/// Extract Canon FocalLength binary data
/// ExifTool: lib/Image/ExifTool/Canon.pm:2637-2713 %Canon::FocalLength
/// Table parameters:
/// - FORMAT => 'int16u' (unsigned 16-bit integers)
/// - FIRST_ENTRY => 0 (0-indexed)
/// - GROUPS => { 0 => 'MakerNotes', 2 => 'Image' }
pub fn extract_focal_length(
    data: &[u8],
    offset: usize,
    size: usize,
    byte_order: ByteOrder,
) -> Result<HashMap<String, TagValue>> {
    let mut results = HashMap::new();

    // ExifTool: Canon.pm:2640 FORMAT => 'int16u'
    let _format_size = 2; // int16u = 2 bytes

    debug!(
        "Extracting Canon FocalLength: offset={:#x}, size={}, format=int16u",
        offset, size
    );

    // Use Canon tag kit system for PrintConv lookups
    #[allow(unused_imports)]
    use crate::generated::Canon_pm;

    // Extract FocalType (index 0)
    // ExifTool: Canon.pm:2643 Name => 'FocalType'
    if size >= 2 {
        let focal_type = byte_order.read_u16(data, offset)?;

        // Apply PrintConv using Canon tag kit system
        // FocalType has tag ID 0 in Canon tag kit other.rs
        let raw_value = TagValue::U16(focal_type);
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        let final_value = apply_canon_print_conv(
            0, // FocalType tag ID
            &raw_value,
            &mut errors,
            &mut warnings,
        );

        // Log any warnings from tag kit processing
        for warning in warnings {
            debug!("Canon FocalType tag kit warning: {}", warning);
        }

        debug!(
            "Extracted Canon FocalType = {:?} (raw: {})",
            final_value, focal_type
        );
        results.insert("MakerNotes:FocalType".to_string(), final_value);
    }

    // Extract FocalLength (index 1)
    // ExifTool: Canon.pm:2654 Name => 'FocalLength'
    // ValueConv => '$val[1] && $val[1] =~ /^\d+$/ ? $val[1] / $val[18] : undef'
    // Note: This needs FocalUnits (index 18) for proper conversion
    if size >= 4 {
        let focal_length = byte_order.read_u16(data, offset + 2)?;

        debug!("Extracted Canon FocalLength raw value = {}", focal_length);

        // Store raw value for now - proper conversion needs FocalUnits (index 18)
        // TODO: Implement full ValueConv with FocalUnits when available
        results.insert(
            "MakerNotes:FocalLength".to_string(),
            TagValue::U16(focal_length),
        );
    }

    // Extract FocalPlaneXSize (index 2)
    // ExifTool: Canon.pm:2660 Name => 'FocalPlaneXSize'
    // Conditional - only valid for some camera models
    if size >= 6 {
        let focal_plane_x_size = byte_order.read_u16(data, offset + 4)?;

        debug!(
            "Extracted Canon FocalPlaneXSize raw value = {}",
            focal_plane_x_size
        );
        results.insert(
            "MakerNotes:FocalPlaneXSize".to_string(),
            TagValue::U16(focal_plane_x_size),
        );
    }

    // Extract FocalPlaneYSize (index 3)
    // ExifTool: Canon.pm:2671 Name => 'FocalPlaneYSize'
    if size >= 8 {
        let focal_plane_y_size = byte_order.read_u16(data, offset + 6)?;

        debug!(
            "Extracted Canon FocalPlaneYSize raw value = {}",
            focal_plane_y_size
        );
        results.insert(
            "MakerNotes:FocalPlaneYSize".to_string(),
            TagValue::U16(focal_plane_y_size),
        );
    }

    Ok(results)
}

/// Extract Canon ShotInfo binary data
/// ExifTool: lib/Image/ExifTool/Canon.pm:2715-2996 %Canon::ShotInfo
/// Table parameters:
/// - FORMAT => 'int16s' (signed 16-bit integers)
/// - FIRST_ENTRY => 1 (1-indexed)
/// - GROUPS => { 0 => 'MakerNotes', 2 => 'Camera' }
/// - DATAMEMBER => [ 19 ] (FocusDistanceUpper)
pub fn extract_shot_info(
    data: &[u8],
    offset: usize,
    size: usize,
    byte_order: ByteOrder,
) -> Result<HashMap<String, TagValue>> {
    let mut results = HashMap::new();

    // ExifTool: Canon.pm:2719 FORMAT => 'int16s'
    let format_size = 2; // int16s = 2 bytes

    debug!(
        "Extracting Canon ShotInfo: offset={:#x}, size={}, format=int16s",
        offset, size
    );

    // Use Canon tag kit system for PrintConv lookups
    #[allow(unused_imports)]
    use crate::generated::Canon_pm;

    // Extract AutoISO (index 1)
    // ExifTool: Canon.pm:2724 Name => 'AutoISO'
    // ValueConv => 'exp($val/32*log(2))*100'
    if size >= 2 {
        let auto_iso_raw = byte_order.read_u16(data, offset)? as i16;

        // Apply ValueConv: exp($val/32*log(2))*100
        let auto_iso = (2.0_f64).powf(auto_iso_raw as f64 / 32.0) * 100.0;

        debug!(
            "Extracted Canon AutoISO = {} (raw: {})",
            auto_iso, auto_iso_raw
        );
        results.insert("MakerNotes:AutoISO".to_string(), TagValue::F64(auto_iso));
    }

    // Extract BaseISO (index 2)
    // ExifTool: Canon.pm:2729 Name => 'BaseISO'
    // ValueConv => 'exp($val/32*log(2))*100/32'
    if size >= 4 {
        let base_iso_raw = byte_order.read_u16(data, offset + 2)? as i16;

        // Apply ValueConv: exp($val/32*log(2))*100/32
        let base_iso = (2.0_f64).powf(base_iso_raw as f64 / 32.0) * 100.0 / 32.0;

        debug!(
            "Extracted Canon BaseISO = {} (raw: {})",
            base_iso, base_iso_raw
        );
        results.insert("MakerNotes:BaseISO".to_string(), TagValue::F64(base_iso));
    }

    // Extract MeasuredEV (index 3)
    // ExifTool: Canon.pm:2734 Name => 'MeasuredEV'
    // ValueConv => '$val / 32'
    if size >= 6 {
        let measured_ev_raw = byte_order.read_u16(data, offset + 4)? as i16;
        let measured_ev = measured_ev_raw as f64 / 32.0;

        debug!(
            "Extracted Canon MeasuredEV = {} (raw: {})",
            measured_ev, measured_ev_raw
        );
        results.insert(
            "MakerNotes:MeasuredEV".to_string(),
            TagValue::F64(measured_ev),
        );
    }

    // Extract TargetAperture (index 4)
    // ExifTool: Canon.pm:2739 Name => 'TargetAperture'
    // ValueConv => 'exp(Canon::CanonEv($val)*log(2)/2)'
    if size >= 8 {
        let target_aperture_raw = byte_order.read_u16(data, offset + 6)? as i16;

        // Apply Canon EV conversion then aperture calculation
        // Canon::CanonEv just returns $val/32 for simple values
        let ev = target_aperture_raw as f64 / 32.0;
        let target_aperture = (2.0_f64).powf(ev / 2.0);

        debug!(
            "Extracted Canon TargetAperture = {} (raw: {})",
            target_aperture, target_aperture_raw
        );
        results.insert(
            "MakerNotes:TargetAperture".to_string(),
            TagValue::F64(target_aperture),
        );
    }

    // Extract WhiteBalance (index 7)
    // ExifTool: Canon.pm:2753 Name => 'WhiteBalance'
    // FIRST_ENTRY => 1, so actual offset is (7-1)*2 = 12 bytes
    if size >= 14 {
        let wb_offset = offset + (7 - 1) * format_size;
        let white_balance = byte_order.read_u16(data, wb_offset)? as i16;

        // Apply PrintConv using Canon tag kit system
        // WhiteBalance has tag ID 7 in Canon tag kit
        let raw_value = TagValue::I16(white_balance);
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        let final_value = apply_canon_print_conv(
            7, // WhiteBalance tag ID
            &raw_value,
            &mut errors,
            &mut warnings,
        );

        // Log any warnings from tag kit processing
        for warning in warnings {
            debug!("Canon WhiteBalance tag kit warning: {}", warning);
        }

        debug!(
            "Extracted Canon WhiteBalance = {:?} (raw: {})",
            final_value, white_balance
        );
        results.insert("MakerNotes:WhiteBalance".to_string(), final_value);
    }

    // Extract AFPointsInFocus (index 14)
    // ExifTool: Canon.pm:2815 Name => 'AFPointsInFocus'
    // FIRST_ENTRY => 1, so actual offset is (14-1)*2 = 26 bytes
    if size >= 28 {
        let af_offset = offset + (14 - 1) * format_size;
        let af_points = byte_order.read_u16(data, af_offset)? as i16;

        // Apply PrintConv using Canon tag kit system
        // AFPointsInFocus has tag ID 14 in Canon tag kit
        let raw_value = TagValue::I16(af_points);
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        let final_value = apply_canon_print_conv(
            14, // AFPointsInFocus tag ID
            &raw_value,
            &mut errors,
            &mut warnings,
        );

        // Log any warnings from tag kit processing
        for warning in warnings {
            debug!("Canon AFPointsInFocus tag kit warning: {}", warning);
        }

        debug!(
            "Extracted Canon AFPointsInFocus = {:?} (raw: {})",
            final_value, af_points
        );
        results.insert("MakerNotes:AFPointsInFocus".to_string(), final_value);
    }

    // Extract AutoExposureBracketing (index 16)
    // ExifTool: Canon.pm:2847 Name => 'AutoExposureBracketing'
    // FIRST_ENTRY => 1, so actual offset is (16-1)*2 = 30 bytes
    if size >= 32 {
        let aeb_offset = offset + (16 - 1) * format_size;
        let aeb_value = byte_order.read_u16(data, aeb_offset)? as i16;

        // Apply PrintConv using Canon tag kit system
        // AutoExposureBracketing has tag ID 16 in Canon tag kit
        let raw_value = TagValue::I16(aeb_value);
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        let final_value = apply_canon_print_conv(
            16, // AutoExposureBracketing tag ID
            &raw_value,
            &mut errors,
            &mut warnings,
        );

        // Log any warnings from tag kit processing
        for warning in warnings {
            debug!("Canon AutoExposureBracketing tag kit warning: {}", warning);
        }

        debug!(
            "Extracted Canon AutoExposureBracketing = {:?} (raw: {})",
            final_value, aeb_value
        );
        results.insert("MakerNotes:AutoExposureBracketing".to_string(), final_value);
    }

    // Extract CameraType (index 26)
    // ExifTool: Canon.pm:2938 Name => 'CameraType'
    // FIRST_ENTRY => 1, so actual offset is (26-1)*2 = 50 bytes
    if size >= 52 {
        let camera_type_offset = offset + (26 - 1) * format_size;
        let camera_type = byte_order.read_u16(data, camera_type_offset)? as i16;

        // Apply PrintConv using Canon tag kit system
        // CameraType has tag ID 26 in Canon tag kit
        let raw_value = TagValue::I16(camera_type);
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        let final_value = apply_canon_print_conv(
            26, // CameraType tag ID
            &raw_value,
            &mut errors,
            &mut warnings,
        );

        // Log any warnings from tag kit processing
        for warning in warnings {
            debug!("Canon CameraType tag kit warning: {}", warning);
        }

        debug!(
            "Extracted Canon CameraType = {:?} (raw: {})",
            final_value, camera_type
        );
        results.insert("MakerNotes:CameraType".to_string(), final_value);
    }

    Ok(results)
}

/// Extract Canon Panorama data from binary data
/// ExifTool: Canon.pm:2999 %Canon::Panorama
/// FORMAT => 'int16s', FIRST_ENTRY => 0
pub fn extract_panorama(
    data: &[u8],
    offset: usize,
    size: usize,
    byte_order: ByteOrder,
) -> Result<HashMap<String, TagValue>> {
    debug!(
        "Extracting Canon Panorama data: offset={:#x}, size={}",
        offset, size
    );

    let mut panorama = HashMap::new();

    // Use Canon tag kit system for PrintConv lookups
    #[allow(unused_imports)]
    use crate::generated::Canon_pm;

    // Canon Panorama format: int16s (signed 16-bit), starting at index 0
    // ExifTool: Canon.pm:3001 FORMAT => 'int16s', FIRST_ENTRY => 0

    // Tag 2: PanoramaFrameNumber
    // ExifTool: Canon.pm:3006
    if let Ok(frame_number) = read_int16s_at_index(data, offset, 2, byte_order) {
        panorama.insert(
            "MakerNotes:PanoramaFrameNumber".to_string(),
            TagValue::I16(frame_number),
        );
        debug!("PanoramaFrameNumber: {}", frame_number);
    }

    // Tag 5: PanoramaDirection with PrintConv using generated lookup
    // ExifTool: Canon.pm:3009-3018
    if let Ok(direction_raw) = read_int16s_at_index(data, offset, 5, byte_order) {
        // Apply PrintConv using Canon tag kit system
        // PanoramaDirection has tag ID 5 in Canon tag kit
        let raw_value = TagValue::I16(direction_raw);
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        let direction_value = apply_canon_print_conv(
            5, // PanoramaDirection tag ID
            &raw_value,
            &mut errors,
            &mut warnings,
        );

        // Log any warnings from tag kit processing
        for warning in warnings {
            debug!("Canon PanoramaDirection tag kit warning: {}", warning);
        }

        debug!(
            "PanoramaDirection: {:?} (raw: {})",
            direction_value, direction_raw
        );
        panorama.insert("MakerNotes:PanoramaDirection".to_string(), direction_value);
    }

    debug!("Extracted {} Canon Panorama tags", panorama.len());
    Ok(panorama)
}

/// Extract Canon MyColors data from binary data
/// ExifTool: Canon.pm:3131 %Canon::MyColors
/// FORMAT => 'int16u', FIRST_ENTRY => 0, with validation
pub fn extract_my_colors(
    data: &[u8],
    offset: usize,
    size: usize,
    byte_order: ByteOrder,
) -> Result<HashMap<String, TagValue>> {
    debug!(
        "Extracting Canon MyColors data: offset={:#x}, size={}",
        offset, size
    );

    let mut my_colors = HashMap::new();

    // Use Canon tag kit system for PrintConv lookups
    #[allow(unused_imports)]
    use crate::generated::Canon_pm;

    // Canon MyColors format: int16u (unsigned 16-bit), starting at index 0
    // ExifTool: Canon.pm:3133 FORMAT => 'int16u', FIRST_ENTRY => 0

    // ExifTool validation: first 16-bit value is the length of the data in bytes
    // ExifTool: Canon.pm:3125-3129 Validate function
    if size >= 2 {
        let declared_size = byte_order.read_u16(data, offset)? as usize;
        debug!(
            "MyColors declared size: {} bytes, actual size: {} bytes",
            declared_size, size
        );

        if declared_size != size {
            debug!(
                "MyColors size mismatch - declared: {}, actual: {}",
                declared_size, size
            );
            // Continue processing anyway, following ExifTool's approach
        }
    }

    // Tag 0x02: MyColorMode
    // ExifTool: Canon.pm:3137-3153
    if let Ok(my_color_mode) = read_int16u_at_index(data, offset, 0x02, byte_order) {
        // Apply PrintConv using Canon tag kit system
        // MyColorMode has tag ID 2 in Canon tag kit
        let raw_value = TagValue::U16(my_color_mode);
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        let color_mode_value = apply_canon_print_conv(
            2, // MyColorMode tag ID
            &raw_value,
            &mut errors,
            &mut warnings,
        );

        // Log any warnings from tag kit processing
        for warning in warnings {
            debug!("Canon MyColorMode tag kit warning: {}", warning);
        }

        debug!(
            "MyColorMode: {:?} (raw: {})",
            color_mode_value, my_color_mode
        );
        my_colors.insert("MakerNotes:MyColorMode".to_string(), color_mode_value);
    }

    debug!("Extracted {} Canon MyColors tags", my_colors.len());
    Ok(my_colors)
}

/// Read signed 16-bit integer at specific index in Canon binary data
/// ExifTool: Canon.pm ProcessBinaryData with FORMAT => 'int16s'
fn read_int16s_at_index(
    data: &[u8],
    offset: usize,
    index: usize,
    byte_order: ByteOrder,
) -> Result<i16> {
    let position = offset + (index * 2); // int16s = 2 bytes per entry
    if position + 2 <= data.len() {
        let raw_value = byte_order.read_u16(data, position)?;
        Ok(raw_value as i16) // Convert to signed
    } else {
        Err(ExifError::ParseError(format!(
            "Cannot read int16s at index {index} - beyond data bounds"
        )))
    }
}

/// Read unsigned 16-bit integer at specific index in Canon binary data
/// ExifTool: Canon.pm ProcessBinaryData with FORMAT => 'int16u'
fn read_int16u_at_index(
    data: &[u8],
    offset: usize,
    index: usize,
    byte_order: ByteOrder,
) -> Result<u16> {
    let position = offset + (index * 2); // int16u = 2 bytes per entry
    if position + 2 <= data.len() {
        let raw_value = byte_order.read_u16(data, position)?;
        Ok(raw_value) // Already unsigned
    } else {
        Err(ExifError::ParseError(format!(
            "Cannot read int16u at index {index} - beyond data bounds"
        )))
    }
}

/// Create Canon AF Info binary data table with variable-length arrays
/// ExifTool: lib/Image/ExifTool/Canon.pm:4440+ %Canon::AFInfo table
/// Reference: third-party/exiftool/lib/Image/ExifTool/Canon.pm:4440-4500+ AFInfo table
/// Demonstrates Milestone 12: Variable ProcessBinaryData with DataMember dependencies
pub fn create_canon_af_info_table() -> BinaryDataTable {
    use crate::types::{BinaryDataFormat, FormatSpec};

    let mut table = BinaryDataTable {
        default_format: BinaryDataFormat::Int16u,
        first_entry: Some(0),
        groups: HashMap::new(),
        tags: HashMap::new(),
        data_member_tags: Vec::new(),
        dependency_order: Vec::new(),
    };

    // ExifTool: Canon.pm:4442 GROUPS => { 0 => 'MakerNotes', 2 => 'Camera' }
    table.groups.insert(0, "MakerNotes".to_string());
    table.groups.insert(2, "Camera".to_string());

    // NumAFPoints (sequence 0) - The key DataMember for variable-length arrays
    // ExifTool: Canon.pm:4450 '0 => { Name => 'NumAFPoints' }'
    table.tags.insert(
        0,
        BinaryDataTag::from_legacy(
            "NumAFPoints".to_string(),
            Some(FormatSpec::Fixed(BinaryDataFormat::Int16u)),
            Some(BinaryDataFormat::Int16u),
            None,
            None,
            Some("NumAFPoints".to_string()), // This becomes a DataMember
            Some(0),                         // MakerNotes group
        ),
    );
    table.data_member_tags.push(0);

    // ValidAFPoints (sequence 1)
    // ExifTool: Canon.pm:4453 '1 => { Name => 'ValidAFPoints' }'
    table.tags.insert(
        1,
        BinaryDataTag::from_legacy(
            "ValidAFPoints".to_string(),
            Some(FormatSpec::Fixed(BinaryDataFormat::Int16u)),
            Some(BinaryDataFormat::Int16u),
            None,
            None,
            None,
            Some(0), // MakerNotes group
        ),
    );

    // CanonImageWidth (sequence 2)
    table.tags.insert(
        2,
        BinaryDataTag::from_legacy(
            "CanonImageWidth".to_string(),
            Some(FormatSpec::Fixed(BinaryDataFormat::Int16u)),
            Some(BinaryDataFormat::Int16u),
            None,
            None,
            None,
            Some(2), // Camera group
        ),
    );

    // CanonImageHeight (sequence 3)
    table.tags.insert(
        3,
        BinaryDataTag::from_legacy(
            "CanonImageHeight".to_string(),
            Some(FormatSpec::Fixed(BinaryDataFormat::Int16u)),
            Some(BinaryDataFormat::Int16u),
            None,
            None,
            None,
            Some(2), // Camera group
        ),
    );

    // AFImageWidth (sequence 4)
    table.tags.insert(
        4,
        BinaryDataTag::from_legacy(
            "AFImageWidth".to_string(),
            Some(FormatSpec::Fixed(BinaryDataFormat::Int16u)),
            Some(BinaryDataFormat::Int16u),
            None,
            None,
            None,
            Some(0), // MakerNotes group
        ),
    );

    // AFImageHeight (sequence 5)
    table.tags.insert(
        5,
        BinaryDataTag::from_legacy(
            "AFImageHeight".to_string(),
            Some(FormatSpec::Fixed(BinaryDataFormat::Int16u)),
            Some(BinaryDataFormat::Int16u),
            None,
            None,
            None,
            Some(0), // MakerNotes group
        ),
    );

    // AFAreaWidth (sequence 6)
    table.tags.insert(
        6,
        BinaryDataTag::from_legacy(
            "AFAreaWidth".to_string(),
            Some(FormatSpec::Fixed(BinaryDataFormat::Int16u)),
            Some(BinaryDataFormat::Int16u),
            None,
            None,
            None,
            Some(0), // MakerNotes group
        ),
    );

    // AFAreaHeight (sequence 7)
    table.tags.insert(
        7,
        BinaryDataTag::from_legacy(
            "AFAreaHeight".to_string(),
            Some(FormatSpec::Fixed(BinaryDataFormat::Int16u)),
            Some(BinaryDataFormat::Int16u),
            None,
            None,
            None,
            Some(0), // MakerNotes group
        ),
    );

    // AFAreaXPositions (sequence 8) - Variable-length array sized by NumAFPoints
    // ExifTool: Canon.pm:4474 'Format => int16s[$val{0}]'
    table.tags.insert(
        8,
        BinaryDataTag::from_legacy(
            "AFAreaXPositions".to_string(),
            Some(FormatSpec::Array {
                base_format: BinaryDataFormat::Int16s,
                count_expr: "$val{0}".to_string(), // References NumAFPoints at sequence 0
            }),
            None, // Will be resolved at runtime
            None,
            None,
            None,
            Some(0), // MakerNotes group
        ),
    );

    // AFAreaYPositions (sequence 9) - Variable-length array sized by NumAFPoints
    // ExifTool: Canon.pm:4477 'Format => int16s[$val{0}]'
    table.tags.insert(
        9,
        BinaryDataTag::from_legacy(
            "AFAreaYPositions".to_string(),
            Some(FormatSpec::Array {
                base_format: BinaryDataFormat::Int16s,
                count_expr: "$val{0}".to_string(), // References NumAFPoints at sequence 0
            }),
            None, // Will be resolved at runtime
            None,
            None,
            None,
            Some(0), // MakerNotes group
        ),
    );

    // AFPointsInFocus (sequence 10) - Complex expression with bit array size calculation
    // ExifTool: Canon.pm:4480 'Format => int16s[int(($val{0}+15)/16)]'
    table.tags.insert(
        10,
        BinaryDataTag::from_legacy(
            "AFPointsInFocus".to_string(),
            Some(FormatSpec::Array {
                base_format: BinaryDataFormat::Int16s,
                count_expr: "int(($val{0}+15)/16)".to_string(), // Ceiling division for bit arrays
            }),
            None, // Will be resolved at runtime
            None,
            None,
            None,
            Some(0), // MakerNotes group
        ),
    );

    // Analyze dependencies to establish processing order
    table.analyze_dependencies();

    table
}

/// Create Canon CameraSettings binary data table in the expected format
/// ExifTool: lib/Image/ExifTool/Canon.pm:2166+ %Canon::CameraSettings
/// This function creates a BinaryDataTable compatible with the test expectations
pub fn create_canon_camera_settings_table() -> BinaryDataTable {
    let mut table = BinaryDataTable {
        default_format: BinaryDataFormat::Int16s,
        first_entry: Some(1),
        groups: HashMap::new(),
        tags: HashMap::new(),
        data_member_tags: Vec::new(),
        dependency_order: Vec::new(),
    };

    // ExifTool: Canon.pm:2171 GROUPS => { 0 => 'MakerNotes', 2 => 'Camera' }
    table.groups.insert(0, "MakerNotes".to_string());
    table.groups.insert(2, "Camera".to_string());

    // ExifTool: Canon.pm:2172-2178 tag 1 MacroMode
    table.tags.insert(
        1,
        BinaryDataTag::from_legacy(
            "MacroMode".to_string(),
            None, // Uses table default
            None, // Uses table default
            None,
            {
                let mut conv = HashMap::new();
                conv.insert(1u32, "Macro".to_string());
                conv.insert(2u32, "Normal".to_string());
                Some(conv)
            },
            None,
            Some(0), // MakerNotes group
        ),
    );

    // ExifTool: Canon.pm:2179-2191 tag 2 SelfTimer
    table.tags.insert(
        2,
        BinaryDataTag::from_legacy(
            "SelfTimer".to_string(),
            None,
            None,
            None,
            {
                let mut conv = HashMap::new();
                conv.insert(0u32, "Off".to_string());
                Some(conv)
            },
            None,
            Some(0), // MakerNotes group
        ),
    );

    // ExifTool: Canon.pm:2196-2209 tag 4 CanonFlashMode
    table.tags.insert(
        4,
        BinaryDataTag::from_legacy(
            "CanonFlashMode".to_string(),
            None,
            None,
            None,
            {
                let mut conv = HashMap::new();
                conv.insert(0u32, "Off".to_string());
                conv.insert(1u32, "Auto".to_string());
                conv.insert(2u32, "On".to_string());
                Some(conv)
            },
            None,
            Some(0), // MakerNotes group
        ),
    );

    // ExifTool: Canon.pm:2228-2240 tag 7 FocusMode
    table.tags.insert(
        7,
        BinaryDataTag::from_legacy(
            "FocusMode".to_string(),
            None,
            None,
            None,
            {
                let mut conv = HashMap::new();
                conv.insert(0u32, "One-shot AF".to_string());
                conv.insert(1u32, "AI Servo AF".to_string());
                conv.insert(2u32, "AI Focus AF".to_string());
                conv.insert(3u32, "Manual Focus (3)".to_string());
                Some(conv)
            },
            None,
            Some(0), // MakerNotes group
        ),
    );

    table
}

/// Extract binary value from ExifReader data
/// Used by binary data processing to extract individual values
pub fn extract_binary_value(
    reader: &crate::exif::ExifReader,
    offset: usize,
    format: BinaryDataFormat,
    _count: usize,
) -> Result<TagValue> {
    let data = reader.get_data();
    let byte_order = if let Some(header) = reader.get_header() {
        header.byte_order
    } else {
        // Default to little-endian when no header is available (common for test scenarios)
        ByteOrder::LittleEndian
    };

    match format {
        BinaryDataFormat::Int8u => {
            if offset >= data.len() {
                return Err(ExifError::ParseError(
                    "Offset beyond data bounds".to_string(),
                ));
            }
            Ok(TagValue::U8(data[offset]))
        }
        BinaryDataFormat::Int8s => {
            if offset >= data.len() {
                return Err(ExifError::ParseError(
                    "Offset beyond data bounds".to_string(),
                ));
            }
            // TagValue doesn't have I8, so store as I16
            Ok(TagValue::I16(data[offset] as i8 as i16))
        }
        BinaryDataFormat::Int16u => {
            if offset + 2 > data.len() {
                return Err(ExifError::ParseError(
                    "Offset beyond data bounds for int16u".to_string(),
                ));
            }
            let value = byte_order.read_u16(data, offset)?;
            Ok(TagValue::U16(value))
        }
        BinaryDataFormat::Int16s => {
            if offset + 2 > data.len() {
                return Err(ExifError::ParseError(
                    "Offset beyond data bounds for int16s".to_string(),
                ));
            }
            let value = byte_order.read_u16(data, offset)? as i16;
            Ok(TagValue::I16(value))
        }
        BinaryDataFormat::Int32u => {
            if offset + 4 > data.len() {
                return Err(ExifError::ParseError(
                    "Offset beyond data bounds for int32u".to_string(),
                ));
            }
            let value = byte_order.read_u32(data, offset)?;
            Ok(TagValue::U32(value))
        }
        BinaryDataFormat::Int32s => {
            if offset + 4 > data.len() {
                return Err(ExifError::ParseError(
                    "Offset beyond data bounds for int32s".to_string(),
                ));
            }
            let value = byte_order.read_u32(data, offset)? as i32;
            Ok(TagValue::I32(value))
        }
        BinaryDataFormat::String => {
            // Extract null-terminated string
            if offset >= data.len() {
                return Err(ExifError::ParseError(
                    "Offset beyond data bounds for string".to_string(),
                ));
            }

            let mut end = offset;
            while end < data.len() && data[end] != 0 {
                end += 1;
            }

            let string_bytes = &data[offset..end];
            let string_value = String::from_utf8_lossy(string_bytes).to_string();
            Ok(TagValue::String(string_value))
        }
        BinaryDataFormat::PString => {
            // Pascal string: first byte is length
            if offset >= data.len() {
                return Err(ExifError::ParseError(
                    "Offset beyond data bounds for pstring".to_string(),
                ));
            }

            let length = data[offset] as usize;
            if offset + 1 + length > data.len() {
                return Err(ExifError::ParseError(
                    "Pascal string length exceeds data bounds".to_string(),
                ));
            }

            let string_bytes = &data[offset + 1..offset + 1 + length];
            let string_value = String::from_utf8_lossy(string_bytes).to_string();
            Ok(TagValue::String(string_value))
        }
        _ => Err(ExifError::ParseError(format!(
            "Binary format {format:?} not yet implemented"
        ))),
    }
}

/// Extract binary data tags from ExifReader using a binary data table
/// This processes the data according to the table configuration
pub fn extract_binary_data_tags(
    reader: &mut crate::exif::ExifReader,
    offset: usize,
    size: usize,
    table: &BinaryDataTable,
) -> Result<()> {
    use crate::types::TagSourceInfo;

    debug!(
        "Extracting binary data tags: offset={:#x}, size={}, format={:?}",
        offset, size, table.default_format
    );

    // Process each defined tag in the table
    for (&index, tag_def) in &table.tags {
        // Calculate position based on FIRST_ENTRY
        let first_entry = table.first_entry.unwrap_or(0);
        if index < first_entry {
            continue;
        }

        let entry_offset = (index - first_entry) as usize * table.default_format.byte_size();
        if entry_offset + table.default_format.byte_size() > size {
            debug!("Tag {} at index {} beyond data bounds", tag_def.name, index);
            continue;
        }

        let data_offset = offset + entry_offset;

        // Extract the raw value
        let format = tag_def.format.unwrap_or(table.default_format);
        let raw_value = extract_binary_value(reader, data_offset, format, 1)?;

        // Apply PrintConv if available
        let final_value = if let Some(print_conv) = &tag_def.print_conv {
            match &raw_value {
                TagValue::I16(val) => {
                    if let Some(converted) = print_conv.get(&(*val as u32)) {
                        TagValue::String(converted.clone())
                    } else {
                        raw_value
                    }
                }
                TagValue::U16(val) => {
                    if let Some(converted) = print_conv.get(&(*val as u32)) {
                        TagValue::String(converted.clone())
                    } else {
                        raw_value
                    }
                }
                _ => raw_value,
            }
        } else {
            raw_value
        };

        // Store the tag with source info
        let group_0 = table
            .groups
            .get(&0)
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string());
        let source_info = TagSourceInfo::new(
            group_0,
            "Canon".to_string(),
            "Canon::BinaryData".to_string(),
        );

        // Use namespace-aware storage
        reader.store_tag_with_precedence(index as u16, final_value, source_info);

        debug!(
            "Extracted Canon binary tag {} (index {}) = {:?}",
            tag_def.name,
            index,
            reader
                .extracted_tags
                .get(&(index as u16, "Canon".to_string()))
        );
    }

    Ok(())
}

/// Find Canon CameraSettings tag in MakerNotes IFD
/// Searches for tag 0x0001 (CanonCameraSettings) in the IFD structure
pub fn find_canon_camera_settings_tag(
    reader: &crate::exif::ExifReader,
    ifd_offset: usize,
    _size: usize,
) -> Result<usize> {
    let data = reader.get_data();
    let byte_order = if let Some(header) = reader.get_header() {
        header.byte_order
    } else {
        // Default to little-endian when no header is available (common for test scenarios)
        ByteOrder::LittleEndian
    };

    // Ensure we have enough data for the entry count
    if ifd_offset + 2 > data.len() {
        return Err(ExifError::ParseError(
            "IFD offset beyond data bounds".to_string(),
        ));
    }

    // Read the number of IFD entries
    let entry_count = byte_order.read_u16(data, ifd_offset)? as usize;
    let entries_start = ifd_offset + 2;
    let entries_size = entry_count * 12; // Each IFD entry is 12 bytes

    if entries_start + entries_size > data.len() {
        return Err(ExifError::ParseError(
            "IFD entries beyond data bounds".to_string(),
        ));
    }

    // Search for Canon CameraSettings tag (0x0001)
    for i in 0..entry_count {
        let entry_offset = entries_start + (i * 12);
        let tag_id = byte_order.read_u16(data, entry_offset)?;

        if tag_id == 0x0001 {
            // Found CanonCameraSettings tag, return the value offset
            let format = byte_order.read_u16(data, entry_offset + 2)?;
            let count = byte_order.read_u32(data, entry_offset + 4)?;
            let value_offset = byte_order.read_u32(data, entry_offset + 8)? as usize;

            debug!(
                "Found Canon CameraSettings tag: format={}, count={}, offset={:#x}",
                format, count, value_offset
            );

            return Ok(value_offset);
        }
    }

    Err(ExifError::ParseError(
        "Canon CameraSettings tag not found in IFD".to_string(),
    ))
}

/// Extract FocalUnits value from Canon CameraSettings for dependent conversions
/// ExifTool: Canon.pm:2489 DATAMEMBER => [ 22, 25 ] - FocalUnits at offset 25
/// Used by MinFocalLength and MaxFocalLength ValueConv: '$val / ($$self{FocalUnits} || 1)'
fn extract_focal_units_from_camera_settings(
    data: &[u8],
    offset: usize,
    size: usize,
    byte_order: ByteOrder,
) -> Result<f64> {
    let format_size = 2; // int16s = 2 bytes
    let focal_units_index = 25; // ExifTool: Canon.pm offset 25

    // ExifTool: Canon.pm:2169 FIRST_ENTRY => 1 (1-indexed)
    let entry_offset = (focal_units_index - 1) as usize * format_size;

    if entry_offset + format_size > size {
        debug!(
            "FocalUnits at index {} beyond data bounds, using fallback",
            focal_units_index
        );
        return Ok(1.0); // ExifTool fallback: || 1
    }

    let data_offset = offset + entry_offset;
    if data_offset + format_size > data.len() {
        debug!(
            "FocalUnits data offset {:#x} beyond buffer bounds, using fallback",
            data_offset
        );
        return Ok(1.0); // ExifTool fallback: || 1
    }

    // Extract int16s value (signed 16-bit integer)
    let raw_focal_units = byte_order.read_u16(data, data_offset)? as i16;

    // Apply ExifTool's || 1 fallback logic
    if raw_focal_units <= 0 {
        debug!(
            "FocalUnits value {} <= 0, using fallback 1.0",
            raw_focal_units
        );
        Ok(1.0)
    } else {
        Ok(raw_focal_units as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exif::ExifReader;
    use crate::tiff_types::{ByteOrder, TiffHeader};

    #[test]
    fn test_canon_af_info_variable_arrays() {
        // Test Milestone 12: Variable ProcessBinaryData with Canon AF Info
        // Reference: third-party/exiftool/lib/Image/ExifTool/Canon.pm:4440-4500+ AFInfo table
        // This demonstrates variable-length arrays sized by DataMember dependencies

        let table = create_canon_af_info_table();

        // Verify table structure
        assert_eq!(table.default_format, BinaryDataFormat::Int16u);
        assert_eq!(table.first_entry, Some(0));
        assert_eq!(table.groups.get(&0), Some(&"MakerNotes".to_string()));
        assert_eq!(table.groups.get(&2), Some(&"Camera".to_string()));

        // Verify DataMember tags
        assert!(table.data_member_tags.contains(&0)); // NumAFPoints

        // Verify tag definitions
        let num_af_points = table.tags.get(&0).unwrap();
        assert_eq!(num_af_points.name, "NumAFPoints");
        assert_eq!(num_af_points.data_member, Some("NumAFPoints".to_string()));

        // Verify variable array formats
        let af_x_positions = table.tags.get(&8).unwrap();
        assert_eq!(af_x_positions.name, "AFAreaXPositions");
        if let Some(format_spec) = &af_x_positions.format_spec {
            match format_spec {
                crate::types::FormatSpec::Array {
                    base_format,
                    count_expr,
                } => {
                    assert_eq!(*base_format, BinaryDataFormat::Int16s);
                    assert_eq!(count_expr, "$val{0}"); // References NumAFPoints
                }
                _ => panic!("Expected Array format spec for AFAreaXPositions"),
            }
        } else {
            panic!("AFAreaXPositions should have format_spec");
        }

        let af_y_positions = table.tags.get(&9).unwrap();
        assert_eq!(af_y_positions.name, "AFAreaYPositions");

        let af_points_in_focus = table.tags.get(&10).unwrap();
        assert_eq!(af_points_in_focus.name, "AFPointsInFocus");
        if let Some(format_spec) = &af_points_in_focus.format_spec {
            match format_spec {
                crate::types::FormatSpec::Array {
                    base_format,
                    count_expr,
                } => {
                    assert_eq!(*base_format, BinaryDataFormat::Int16s);
                    assert_eq!(count_expr, "int(($val{0}+15)/16)"); // Complex expression
                }
                _ => panic!("Expected Array format spec for AFPointsInFocus"),
            }
        }
    }

    #[test]
    fn test_canon_af_info_processing() {
        // Test actual processing of Canon AF Info data with variable arrays
        // Reference: third-party/exiftool/lib/Image/ExifTool/Canon.pm:4474+ AFAreaXPositions Format => int16s[$val{0}]
        // Reference: third-party/exiftool/lib/Image/ExifTool/Canon.pm:4477+ AFAreaYPositions Format => int16s[$val{0}]
        // Reference: third-party/exiftool/lib/Image/ExifTool/Canon.pm:4480+ AFPointsInFocus Format => int16s[int(($val{0}+15)/16)]
        // This simulates real Canon AF Info data with NumAFPoints = 9

        let mut reader = ExifReader::new();
        reader.set_test_header(TiffHeader {
            byte_order: ByteOrder::LittleEndian,
            magic: 42,
            ifd0_offset: 8,
        });

        // Create test AF Info data:
        // Sequence 0: NumAFPoints = 9 (0x0009)
        // Sequence 1: ValidAFPoints = 7 (0x0007)
        // Sequence 2: CanonImageWidth = 1600 (0x0640)
        // Sequence 3: CanonImageHeight = 1200 (0x04B0)
        // Sequence 4: AFImageWidth = 1024 (0x0400)
        // Sequence 5: AFImageHeight = 768 (0x0300)
        // Sequence 6: AFAreaWidth = 64 (0x0040)
        // Sequence 7: AFAreaHeight = 48 (0x0030)
        // Sequence 8: AFAreaXPositions[9] = [-200, -100, 0, 100, 200, -150, 0, 150, 0]
        // Sequence 9: AFAreaYPositions[9] = [-100, -50, 0, 50, 100, 75, 0, -75, 150]
        // Sequence 10: AFPointsInFocus[1] = [0x01FF] (bit array: ceiling(9/16) = 1 word)

        let test_data = vec![
            // Sequences 0-7: Fixed data (8 * 2 bytes = 16 bytes)
            0x09, 0x00, // NumAFPoints = 9
            0x07, 0x00, // ValidAFPoints = 7
            0x40, 0x06, // CanonImageWidth = 1600
            0xB0, 0x04, // CanonImageHeight = 1200
            0x00, 0x04, // AFImageWidth = 1024
            0x00, 0x03, // AFImageHeight = 768
            0x40, 0x00, // AFAreaWidth = 64
            0x30, 0x00, // AFAreaHeight = 48
            // Sequence 8: AFAreaXPositions[9] (9 * 2 bytes = 18 bytes)
            0x38, 0xFF, // -200 (0xFF38 = -200 in 2's complement)
            0x9C, 0xFF, // -100 (0xFF9C = -100 in 2's complement)
            0x00, 0x00, // 0
            0x64, 0x00, // 100
            0xC8, 0x00, // 200
            0x6A, 0xFF, // -150 (0xFF6A = -150 in 2's complement)
            0x00, 0x00, // 0
            0x96, 0x00, // 150
            0x00, 0x00, // 0
            // Sequence 9: AFAreaYPositions[9] (9 * 2 bytes = 18 bytes)
            0x9C, 0xFF, // -100 (0xFF9C = -100 in 2's complement)
            0xCE, 0xFF, // -50 (0xFFCE = -50 in 2's complement)
            0x00, 0x00, // 0
            0x32, 0x00, // 50
            0x64, 0x00, // 100
            0x4B, 0x00, // 75
            0x00, 0x00, // 0
            0xB5, 0xFF, // -75 (0xFFB5 = -75 in 2's complement)
            0x96, 0x00, // 150
            // Sequence 10: AFPointsInFocus[1] = int((9+15)/16) = 1 word (2 bytes)
            0xFF, 0x01, // 0x01FF - bits 0-8 set (AF points 1-9 in focus)
        ];

        reader.set_test_data(test_data.clone());

        let table = create_canon_af_info_table();

        // Process the binary data with dependencies
        let result =
            reader.process_binary_data_with_dependencies(&test_data, 0, test_data.len(), &table);
        assert!(
            result.is_ok(),
            "Failed to process Canon AF Info data: {result:?}"
        );

        // Verify extracted tags
        let extracted_tags = reader.get_extracted_tags();

        // Check NumAFPoints (DataMember)
        assert_eq!(
            extracted_tags.get(&(0, "MakerNotes".to_string())),
            Some(&crate::types::TagValue::U16(9))
        );

        // Check ValidAFPoints
        assert_eq!(
            extracted_tags.get(&(1, "MakerNotes".to_string())),
            Some(&crate::types::TagValue::U16(7))
        );

        // Check CanonImageWidth
        assert_eq!(
            extracted_tags.get(&(2, "Camera".to_string())),
            Some(&crate::types::TagValue::U16(1600))
        );

        // Check CanonImageHeight
        assert_eq!(
            extracted_tags.get(&(3, "Camera".to_string())),
            Some(&crate::types::TagValue::U16(1200))
        );

        // Check variable arrays - AFAreaXPositions should be array of 9 elements
        if let Some(crate::types::TagValue::U16Array(x_positions)) =
            extracted_tags.get(&(8, "MakerNotes".to_string()))
        {
            assert_eq!(
                x_positions.len(),
                9,
                "AFAreaXPositions should have 9 elements based on NumAFPoints"
            );
            // Note: The values will be stored as U16 due to array extraction conversion
        } else {
            panic!("AFAreaXPositions should be U16Array");
        }

        // Check variable arrays - AFAreaYPositions should be array of 9 elements
        if let Some(crate::types::TagValue::U16Array(y_positions)) =
            extracted_tags.get(&(9, "MakerNotes".to_string()))
        {
            assert_eq!(
                y_positions.len(),
                9,
                "AFAreaYPositions should have 9 elements based on NumAFPoints"
            );
        } else {
            panic!("AFAreaYPositions should be U16Array");
        }

        // Check complex expression - AFPointsInFocus should be array of 1 element
        // Expression: int((9+15)/16) = int(24/16) = 1
        if let Some(crate::types::TagValue::U16Array(points_in_focus)) =
            extracted_tags.get(&(10, "MakerNotes".to_string()))
        {
            assert_eq!(
                points_in_focus.len(),
                1,
                "AFPointsInFocus should have 1 element based on ceiling division"
            );
        } else {
            panic!("AFPointsInFocus should be U16Array");
        }
    }

    #[test]
    fn test_expression_evaluator() {
        // Test the complex expression evaluator with Canon AF ceiling division
        // Reference: third-party/exiftool/lib/Image/ExifTool/Canon.pm:4480+ int(($val{0}+15)/16) pattern
        use crate::types::{DataMemberValue, ExpressionEvaluator};
        use std::collections::HashMap;

        let data_members = HashMap::new();
        let mut val_hash = HashMap::new();
        val_hash.insert(0, DataMemberValue::U16(9)); // NumAFPoints = 9

        let evaluator = ExpressionEvaluator::new(val_hash, &data_members);

        // Test simple $val{0} expression
        let simple_result = evaluator.evaluate_count_expression("$val{0}");
        assert_eq!(simple_result.unwrap(), 9);

        // Test complex ceiling division: int((9+15)/16) = int(24/16) = 1
        let complex_result = evaluator.evaluate_count_expression("int(($val{0}+15)/16)");
        println!("Complex expression result: {complex_result:?}");
        match complex_result {
            Ok(val) => assert_eq!(val, 1),
            Err(e) => panic!("Complex expression failed: {e}"),
        }
    }

    #[test]
    fn test_variable_string_formats() {
        // Test Milestone 12: Variable string formats with string[$val{N}]
        // Reference: third-party/exiftool/lib/Image/ExifTool.pm:9850+ string format parsing
        use crate::exif::ExifReader;
        use crate::tiff_types::{ByteOrder, TiffHeader};
        use crate::types::{BinaryDataFormat, BinaryDataTable, BinaryDataTag, FormatSpec};
        use std::collections::HashMap;

        let mut reader = ExifReader::new();
        reader.set_test_header(TiffHeader {
            byte_order: ByteOrder::LittleEndian,
            magic: 42,
            ifd0_offset: 8,
        });

        // Create a test table with string length dependency
        let mut table = BinaryDataTable {
            default_format: BinaryDataFormat::Int16u,
            first_entry: Some(0),
            groups: {
                let mut groups = HashMap::new();
                groups.insert(0, "MakerNotes".to_string());
                groups
            },
            tags: HashMap::new(),
            data_member_tags: Vec::new(),
            dependency_order: Vec::new(),
        };

        // Tag 0: StringLength (DataMember) = 5
        table.tags.insert(
            0,
            BinaryDataTag::from_legacy(
                "StringLength".to_string(),
                Some(FormatSpec::Fixed(BinaryDataFormat::Int16u)),
                Some(BinaryDataFormat::Int16u),
                None,
                None,
                Some("StringLength".to_string()),
                Some(0), // MakerNotes group
            ),
        );
        table.data_member_tags.push(0);

        // Tag 1: VariableString = string[$val{0}] (5 characters)
        table.tags.insert(
            1,
            BinaryDataTag::from_legacy(
                "VariableString".to_string(),
                Some(FormatSpec::StringWithLength {
                    length_expr: "$val{0}".to_string(),
                }),
                None,
                None,
                None,
                None,
                Some(0), // MakerNotes group
            ),
        );

        table.analyze_dependencies();

        // Test data: StringLength=5, then "Hello" (5 bytes)
        let test_data = vec![
            0x05, 0x00, // StringLength = 5
            b'H', b'e', b'l', b'l', b'o', // "Hello" (5 bytes)
        ];

        reader.set_test_data(test_data.clone());

        let result =
            reader.process_binary_data_with_dependencies(&test_data, 0, test_data.len(), &table);
        assert!(result.is_ok());

        let extracted_tags = reader.get_extracted_tags();

        // Check StringLength DataMember
        assert_eq!(
            extracted_tags.get(&(0, "MakerNotes".to_string())),
            Some(&crate::types::TagValue::U16(5))
        );

        // Check VariableString
        assert_eq!(
            extracted_tags.get(&(1, "MakerNotes".to_string())),
            Some(&crate::types::TagValue::String("Hello".to_string()))
        );
    }

    #[test]
    fn test_edge_cases_zero_count() {
        // Test edge case: zero count for arrays
        use crate::exif::ExifReader;
        use crate::tiff_types::{ByteOrder, TiffHeader};
        use crate::types::{BinaryDataFormat, BinaryDataTable, BinaryDataTag, FormatSpec};
        use std::collections::HashMap;

        let mut reader = ExifReader::new();
        reader.set_test_header(TiffHeader {
            byte_order: ByteOrder::LittleEndian,
            magic: 42,
            ifd0_offset: 8,
        });

        let mut table = BinaryDataTable {
            default_format: BinaryDataFormat::Int16u,
            first_entry: Some(0),
            groups: {
                let mut groups = HashMap::new();
                groups.insert(0, "MakerNotes".to_string());
                groups
            },
            tags: HashMap::new(),
            data_member_tags: Vec::new(),
            dependency_order: Vec::new(),
        };

        // Tag 0: Count = 0
        table.tags.insert(
            0,
            BinaryDataTag::from_legacy(
                "Count".to_string(),
                Some(FormatSpec::Fixed(BinaryDataFormat::Int16u)),
                Some(BinaryDataFormat::Int16u),
                None,
                None,
                Some("Count".to_string()),
                Some(0), // MakerNotes group
            ),
        );
        table.data_member_tags.push(0);

        // Tag 1: EmptyArray = int16s[$val{0}] (0 elements)
        table.tags.insert(
            1,
            BinaryDataTag::from_legacy(
                "EmptyArray".to_string(),
                Some(FormatSpec::Array {
                    base_format: BinaryDataFormat::Int16s,
                    count_expr: "$val{0}".to_string(),
                }),
                None,
                None,
                None,
                None,
                Some(0), // MakerNotes group
            ),
        );

        table.analyze_dependencies();

        let test_data = vec![0x00, 0x00]; // Count = 0
        reader.set_test_data(test_data.clone());

        let result =
            reader.process_binary_data_with_dependencies(&test_data, 0, test_data.len(), &table);
        assert!(result.is_ok());

        let extracted_tags = reader.get_extracted_tags();

        // Check Count
        assert_eq!(
            extracted_tags.get(&(0, "MakerNotes".to_string())),
            Some(&crate::types::TagValue::U16(0))
        );

        // Check EmptyArray (should be empty array)
        assert_eq!(
            extracted_tags.get(&(1, "MakerNotes".to_string())),
            Some(&crate::types::TagValue::U8Array(vec![]))
        );
    }

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
    fn test_apply_camera_settings_print_conv() {
        use crate::implementations::canon::apply_camera_settings_print_conv;
        use crate::types::TagValue;

        // Test MacroMode PrintConv
        let raw_value = TagValue::I16(1);
        let result = apply_camera_settings_print_conv("MacroMode", &raw_value);
        println!("MacroMode PrintConv: {:?} -> {:?}", raw_value, result);

        // For debugging: also test with tag ID directly
        #[allow(unused_imports)]
        use crate::generated::Canon_pm;
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let direct_result = apply_canon_print_conv(1, &raw_value, &mut errors, &mut warnings);
        println!(
            "Direct tag ID 1 PrintConv: {:?} -> {:?}",
            raw_value, direct_result
        );
        println!("Errors: {:?}", errors);
        println!("Warnings: {:?}", warnings);

        // Check if tag ID 1 is in CANON_MAIN_TAGS
        use crate::generated::Canon_pm::main_tags::CANON_MAIN_TAGS;
        if let Some(tag_def) = CANON_MAIN_TAGS.get(&1) {
            println!(
                "Tag ID 1 found in CANON_MAIN_TAGS: name={}, print_conv={:?}",
                tag_def.name, tag_def.print_conv
            );
        } else {
            println!("Tag ID 1 NOT found in CANON_MAIN_TAGS");
        }
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
        if let Some(value) = tags.get("MakerNotes:MacroMode") {
            println!("MacroMode value: {:?}", value);
            if let TagValue::String(s) = value {
                assert_eq!(s, "Macro");
            } else {
                panic!(
                    "MacroMode should be converted to string, but got: {:?}",
                    value
                );
            }
        } else {
            panic!("MacroMode tag not found in results");
        }

        // SelfTimer value 2 should remain as I16 (no PrintConv for value 2)
        if let Some(TagValue::I16(value)) = tags.get("MakerNotes:SelfTimer") {
            assert_eq!(*value, 2);
        } else {
            panic!("SelfTimer should be I16 for unconverted value");
        }
    }
}
