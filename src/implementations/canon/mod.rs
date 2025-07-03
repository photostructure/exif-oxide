//! Canon-specific EXIF processing coordinator
//!
//! This module coordinates Canon manufacturer-specific processing,
//! dispatching to specialized sub-modules for different aspects.
//!
//! **Trust ExifTool**: This code translates ExifTool's Canon processing verbatim
//! without any improvements or simplifications. Every algorithm, magic number, and
//! quirk is copied exactly as documented in the ExifTool source.
//!
//! Primary ExifTool References:
//! - lib/Image/ExifTool/Canon.pm - Canon tag tables and processing
//! - lib/Image/ExifTool/MakerNotes.pm - Canon MakerNote detection and offset fixing

pub mod binary_data;
pub mod offset_schemes;
pub mod tiff_footer;

// Re-export commonly used binary_data functions for easier access
pub use binary_data::{
    create_canon_camera_settings_table, extract_binary_data_tags, extract_binary_value,
    find_canon_camera_settings_tag,
};
// Re-export offset scheme functions
pub use offset_schemes::{detect_canon_signature, detect_offset_scheme, CanonOffsetScheme};

use crate::tiff_types::ByteOrder;
use crate::types::{Result, TagValue};
use std::collections::HashMap;
use tracing::debug;

// CameraSettings functions are provided by the binary_data module

// extract_camera_settings function is provided by the binary_data module

/// Canon AF Info tag definition for sequential data processing
/// ExifTool: Canon.pm:10224-10306 ProcessSerialData
#[derive(Debug, Clone)]
pub struct CanonAfInfoTag {
    /// Sequential index (0-based like ExifTool sequence processing)
    pub sequence: u32,
    /// Tag name
    pub name: String,
    /// Format type for this tag
    pub format: CanonAfFormat,
    /// Size calculation for variable arrays
    pub size_expr: CanonAfSizeExpr,
    /// Optional condition for tag extraction
    pub condition: Option<CanonAfCondition>,
    /// PrintConv lookup table
    pub print_conv: Option<HashMap<u16, String>>,
}

/// Canon AF data format types
/// ExifTool: Canon.pm AFInfo/AFInfo2 format specifications
#[derive(Debug, Clone)]
pub enum CanonAfFormat {
    /// Fixed 16-bit unsigned integer
    Int16u,
    /// Fixed 16-bit signed integer
    Int16s,
    /// Variable array of 16-bit signed integers with dynamic count
    Int16sArray(CanonAfSizeExpr),
}

/// Size expression for variable arrays in Canon AF data
/// ExifTool: Canon.pm expressions like $val{0}, int(($val{0}+15)/16)
#[derive(Debug, Clone)]
pub enum CanonAfSizeExpr {
    /// Fixed count
    Fixed(usize),
    /// Reference to previously extracted value: $val{N}
    ValueRef(u32),
    /// Ceiling division: int(($val{N}+15)/16) for bit packing
    CeilDiv(u32, u32), // (value_ref, divisor)
}

/// Conditional logic for Canon AF tag extraction
/// ExifTool: Canon.pm Condition expressions
#[derive(Debug, Clone)]
pub enum CanonAfCondition {
    /// Model-based condition: $$self{Model} !~ /EOS/
    ModelNotEos,
    /// Model-based condition: $$self{Model} =~ /EOS/
    ModelIsEos,
}

impl CanonAfSizeExpr {
    /// Calculate array size based on previously extracted values
    /// ExifTool: Canon.pm ProcessSerialData size calculation
    pub fn calculate_size(&self, extracted_values: &HashMap<u32, u16>) -> usize {
        match self {
            CanonAfSizeExpr::Fixed(count) => *count,
            CanonAfSizeExpr::ValueRef(value_ref) => {
                extracted_values.get(value_ref).copied().unwrap_or(0) as usize
            }
            CanonAfSizeExpr::CeilDiv(value_ref, divisor) => {
                let val = extracted_values.get(value_ref).copied().unwrap_or(0) as usize;
                let divisor = *divisor as usize;
                val.div_ceil(divisor) // Ceiling division
            }
        }
    }
}

/// Create Canon AFInfo tag table for sequential processing
/// ExifTool: Canon.pm:8916-9053 %Canon::AFInfo
pub fn create_af_info_table() -> Vec<CanonAfInfoTag> {
    vec![
        // Sequence 0: NumAFPoints
        CanonAfInfoTag {
            sequence: 0,
            name: "NumAFPoints".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 1: ValidAFPoints
        CanonAfInfoTag {
            sequence: 1,
            name: "ValidAFPoints".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 2: CanonImageWidth
        CanonAfInfoTag {
            sequence: 2,
            name: "CanonImageWidth".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 3: CanonImageHeight
        CanonAfInfoTag {
            sequence: 3,
            name: "CanonImageHeight".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 4: AFImageWidth
        CanonAfInfoTag {
            sequence: 4,
            name: "AFImageWidth".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 5: AFImageHeight
        CanonAfInfoTag {
            sequence: 5,
            name: "AFImageHeight".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 6: AFAreaWidth
        CanonAfInfoTag {
            sequence: 6,
            name: "AFAreaWidth".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 7: AFAreaHeight
        CanonAfInfoTag {
            sequence: 7,
            name: "AFAreaHeight".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 8: AFAreaXPositions - Variable array: int16s[$val{0}]
        CanonAfInfoTag {
            sequence: 8,
            name: "AFAreaXPositions".to_string(),
            format: CanonAfFormat::Int16sArray(CanonAfSizeExpr::ValueRef(0)),
            size_expr: CanonAfSizeExpr::ValueRef(0),
            condition: None,
            print_conv: None,
        },
        // Sequence 9: AFAreaYPositions - Variable array: int16s[$val{0}]
        CanonAfInfoTag {
            sequence: 9,
            name: "AFAreaYPositions".to_string(),
            format: CanonAfFormat::Int16sArray(CanonAfSizeExpr::ValueRef(0)),
            size_expr: CanonAfSizeExpr::ValueRef(0),
            condition: None,
            print_conv: None,
        },
        // Sequence 10: AFPointsInFocus - Variable array: int16s[int(($val{0}+15)/16)]
        CanonAfInfoTag {
            sequence: 10,
            name: "AFPointsInFocus".to_string(),
            format: CanonAfFormat::Int16sArray(CanonAfSizeExpr::CeilDiv(0, 16)),
            size_expr: CanonAfSizeExpr::CeilDiv(0, 16),
            condition: None,
            print_conv: None,
        },
        // Sequence 11: PrimaryAFPoint - conditional based on camera model
        CanonAfInfoTag {
            sequence: 11,
            name: "PrimaryAFPoint".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: Some(CanonAfCondition::ModelIsEos),
            print_conv: None,
        },
    ]
}

/// Create Canon AFInfo2 tag table for sequential processing  
/// ExifTool: Canon.pm:9055-9189 %Canon::AFInfo2
pub fn create_af_info2_table() -> Vec<CanonAfInfoTag> {
    vec![
        // Sequence 0: AFInfoSize
        CanonAfInfoTag {
            sequence: 0,
            name: "AFInfoSize".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 1: AFAreaMode
        CanonAfInfoTag {
            sequence: 1,
            name: "AFAreaMode".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: {
                let mut conv = HashMap::new();
                conv.insert(0, "Off (Manual Focus)".to_string());
                conv.insert(1, "AF Point Expansion (surround)".to_string());
                conv.insert(2, "Single-point AF".to_string());
                Some(conv)
            },
        },
        // Sequence 2: NumAFPoints
        CanonAfInfoTag {
            sequence: 2,
            name: "NumAFPoints".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 3: ValidAFPoints
        CanonAfInfoTag {
            sequence: 3,
            name: "ValidAFPoints".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 4: CanonImageWidth
        CanonAfInfoTag {
            sequence: 4,
            name: "CanonImageWidth".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 5: CanonImageHeight
        CanonAfInfoTag {
            sequence: 5,
            name: "CanonImageHeight".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 6: AFImageWidth
        CanonAfInfoTag {
            sequence: 6,
            name: "AFImageWidth".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 7: AFImageHeight
        CanonAfInfoTag {
            sequence: 7,
            name: "AFImageHeight".to_string(),
            format: CanonAfFormat::Int16u,
            size_expr: CanonAfSizeExpr::Fixed(1),
            condition: None,
            print_conv: None,
        },
        // Sequence 8: AFAreaWidths - Variable array: int16s[$val{2}]
        CanonAfInfoTag {
            sequence: 8,
            name: "AFAreaWidths".to_string(),
            format: CanonAfFormat::Int16sArray(CanonAfSizeExpr::ValueRef(2)),
            size_expr: CanonAfSizeExpr::ValueRef(2),
            condition: None,
            print_conv: None,
        },
        // Sequence 9: AFAreaHeights - Variable array: int16s[$val{2}]
        CanonAfInfoTag {
            sequence: 9,
            name: "AFAreaHeights".to_string(),
            format: CanonAfFormat::Int16sArray(CanonAfSizeExpr::ValueRef(2)),
            size_expr: CanonAfSizeExpr::ValueRef(2),
            condition: None,
            print_conv: None,
        },
        // Sequence 10: AFAreaXPositions - Variable array: int16s[$val{2}]
        CanonAfInfoTag {
            sequence: 10,
            name: "AFAreaXPositions".to_string(),
            format: CanonAfFormat::Int16sArray(CanonAfSizeExpr::ValueRef(2)),
            size_expr: CanonAfSizeExpr::ValueRef(2),
            condition: None,
            print_conv: None,
        },
        // Sequence 11: AFAreaYPositions - Variable array: int16s[$val{2}]
        CanonAfInfoTag {
            sequence: 11,
            name: "AFAreaYPositions".to_string(),
            format: CanonAfFormat::Int16sArray(CanonAfSizeExpr::ValueRef(2)),
            size_expr: CanonAfSizeExpr::ValueRef(2),
            condition: None,
            print_conv: None,
        },
        // Sequence 12: AFPointsInFocus - Variable array: int16s[int(($val{2}+15)/16)]
        CanonAfInfoTag {
            sequence: 12,
            name: "AFPointsInFocus".to_string(),
            format: CanonAfFormat::Int16sArray(CanonAfSizeExpr::CeilDiv(2, 16)),
            size_expr: CanonAfSizeExpr::CeilDiv(2, 16),
            condition: None,
            print_conv: None,
        },
        // Sequence 13: AFPointsSelected - Variable array: int16s[int(($val{2}+15)/16)]
        CanonAfInfoTag {
            sequence: 13,
            name: "AFPointsSelected".to_string(),
            format: CanonAfFormat::Int16sArray(CanonAfSizeExpr::CeilDiv(2, 16)),
            size_expr: CanonAfSizeExpr::CeilDiv(2, 16),
            condition: Some(CanonAfCondition::ModelIsEos),
            print_conv: None,
        },
    ]
}

/// Process Canon AFInfo/AFInfo2 serial data with variable-length arrays
/// ExifTool: Canon.pm:10224-10306 ProcessSerialData implementation
///
/// This implements ExifTool's sequential data processing where:
/// - Data is processed in sequence order (0, 1, 2, ...)
/// - Array sizes are calculated from previously extracted values
/// - Position advances based on each tag's calculated size
pub fn process_serial_data(
    data: &[u8],
    offset: usize,
    size: usize,
    byte_order: ByteOrder,
    table: &[CanonAfInfoTag],
    model: &str,
) -> Result<HashMap<String, TagValue>> {
    debug!(
        "Processing Canon AF serial data: offset={:#x}, size={}, entries={}",
        offset,
        size,
        table.len()
    );

    let mut results = HashMap::new();
    let mut extracted_values: HashMap<u32, u16> = HashMap::new();
    let mut current_pos = 0;

    // Process each tag in sequence order
    // ExifTool: Canon.pm:10245-10278 sequential processing loop
    for tag in table {
        if current_pos >= size {
            debug!(
                "Reached end of data at position {}, stopping processing",
                current_pos
            );
            break;
        }

        // Check conditions
        if let Some(condition) = &tag.condition {
            match condition {
                CanonAfCondition::ModelNotEos => {
                    if model.contains("EOS") {
                        debug!("Skipping tag {} due to ModelNotEos condition", tag.name);
                        continue;
                    }
                }
                CanonAfCondition::ModelIsEos => {
                    if !model.contains("EOS") {
                        debug!("Skipping tag {} due to ModelIsEos condition", tag.name);
                        continue;
                    }
                }
            }
        }

        // Calculate array size for this tag
        let array_size = tag.size_expr.calculate_size(&extracted_values);

        match &tag.format {
            CanonAfFormat::Int16u | CanonAfFormat::Int16s => {
                // Single value extraction
                if current_pos + 2 > size {
                    debug!(
                        "Not enough data for tag {} at position {}",
                        tag.name, current_pos
                    );
                    break;
                }

                let data_offset = offset + current_pos;
                if data_offset + 2 > data.len() {
                    debug!(
                        "Data offset {:#x} beyond buffer bounds for tag {}",
                        data_offset, tag.name
                    );
                    break;
                }

                let raw_value = byte_order.read_u16(data, data_offset)?;
                let value = match tag.format {
                    CanonAfFormat::Int16s => TagValue::I16(raw_value as i16),
                    _ => TagValue::U16(raw_value),
                };

                // Store extracted value for later reference
                extracted_values.insert(tag.sequence, raw_value);

                // Apply PrintConv if available
                let final_value = if let Some(print_conv) = &tag.print_conv {
                    if let Some(converted) = print_conv.get(&raw_value) {
                        TagValue::String(converted.clone())
                    } else {
                        value
                    }
                } else {
                    value
                };

                debug!(
                    "Extracted {} = {:?} (raw: {}) at sequence {} position {}",
                    tag.name, final_value, raw_value, tag.sequence, current_pos
                );

                results.insert(format!("MakerNotes:{}", tag.name), final_value);
                current_pos += 2;
            }
            CanonAfFormat::Int16sArray(_) => {
                // Variable array extraction
                let total_bytes = array_size * 2; // int16s = 2 bytes each
                if current_pos + total_bytes > size {
                    debug!(
                        "Not enough data for array tag {} (need {} bytes, have {})",
                        tag.name,
                        total_bytes,
                        size - current_pos
                    );
                    break;
                }

                let mut array_values = Vec::new();
                for i in 0..array_size {
                    let data_offset = offset + current_pos + (i * 2);
                    if data_offset + 2 > data.len() {
                        debug!(
                            "Array element {} beyond buffer bounds for tag {}",
                            i, tag.name
                        );
                        break;
                    }

                    let raw_value = byte_order.read_u16(data, data_offset)? as i16;
                    array_values.push(TagValue::I16(raw_value));
                }

                debug!(
                    "Extracted array {} with {} elements at sequence {} position {}",
                    tag.name,
                    array_values.len(),
                    tag.sequence,
                    current_pos
                );

                // Format array as space-separated string for compatibility with ExifTool
                let array_string = array_values
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(" ");

                results.insert(
                    format!("MakerNotes:{}", tag.name),
                    TagValue::String(array_string),
                );
                current_pos += total_bytes;
            }
        }
    }

    Ok(results)
}

/// Process Canon MakerNotes data
/// ExifTool: lib/Image/ExifTool/Canon.pm Canon MakerNote processing
/// This function processes Canon MakerNotes as an IFD structure to extract Canon-specific tags
pub fn process_canon_makernotes(
    exif_reader: &mut crate::exif::ExifReader,
    dir_start: usize,
    size: usize,
) -> Result<()> {
    use crate::types::DirectoryInfo;

    debug!(
        "Processing Canon MakerNotes: start={:#x}, size={}",
        dir_start, size
    );

    // Canon MakerNotes are structured as a standard IFD
    // ExifTool: Canon.pm Main table processes Canon tags as subdirectories
    let dir_info = DirectoryInfo {
        name: "Canon".to_string(),
        dir_start,
        dir_len: size,
        base: exif_reader.base,
        data_pos: 0,
        allow_reprocess: false,
    };

    // Process the Canon MakerNotes IFD to extract individual Canon tags
    // This will extract tags like CanonCameraSettings, CanonShotInfo, etc.
    exif_reader.process_subdirectory(&dir_info)?;

    debug!("Canon MakerNotes processing completed");
    Ok(())
}

/// Get Canon tag name for synthetic Canon tag IDs (>= 0xC000)
/// Used by the EXIF module to resolve Canon-specific tag names
/// ExifTool: lib/Image/ExifTool/Canon.pm Main table
pub fn get_canon_tag_name(tag_id: u16) -> Option<String> {
    // Map Canon tag IDs to their names based on Canon.pm Main table
    match tag_id {
        // Standard Canon MakerNote tags (0x1-0x30 range)
        0x1 => Some("CanonCameraSettings".to_string()),
        0x2 => Some("CanonFocalLength".to_string()),
        0x3 => Some("CanonFlashInfo".to_string()),
        0x4 => Some("CanonShotInfo".to_string()),
        0x5 => Some("CanonPanorama".to_string()),
        0x6 => Some("CanonImageType".to_string()),
        0x7 => Some("CanonFirmwareVersion".to_string()),
        0x8 => Some("FileNumber".to_string()),
        0x9 => Some("OwnerName".to_string()),
        0xa => Some("UnknownD30".to_string()),
        0xc => Some("SerialNumber".to_string()),
        0xd => Some("CanonCameraInfo".to_string()),
        0xe => Some("CanonFileLength".to_string()),
        0xf => Some("CustomFunctions".to_string()),
        0x10 => Some("CanonModelID".to_string()),
        0x11 => Some("MovieInfo".to_string()),
        0x12 => Some("CanonAFInfo".to_string()),
        0x13 => Some("ThumbnailImageValidArea".to_string()),
        0x15 => Some("SerialNumberFormat".to_string()),
        0x1a => Some("SuperMacro".to_string()),
        0x1c => Some("DateStampMode".to_string()),
        0x1d => Some("MyColors".to_string()),
        0x1e => Some("FirmwareRevision".to_string()),
        0x23 => Some("Categories".to_string()),
        0x24 => Some("FaceDetect1".to_string()),
        0x25 => Some("FaceDetect2".to_string()),
        0x26 => Some("CanonAFInfo2".to_string()),
        0x27 => Some("ContrastInfo".to_string()),
        0x28 => Some("ImageUniqueID".to_string()),
        0x2f => Some("FaceDetect3".to_string()),
        0x35 => Some("TimeInfo".to_string()),
        0x38 => Some("BatteryType".to_string()),
        0x3c => Some("AFInfoSize".to_string()),
        0x81 => Some("RawDataOffset".to_string()),
        0x83 => Some("OriginalDecisionDataOffset".to_string()),
        0x90 => Some("CustomFunctionsD30".to_string()),
        0x91 => Some("PersonalFunctions".to_string()),
        0x92 => Some("PersonalFunctionValues".to_string()),
        0x93 => Some("CanonFileInfo".to_string()),
        0x94 => Some("AFPointsInFocus1D".to_string()),
        0x95 => Some("LensModel".to_string()),
        0x96 => Some("SerialInfo".to_string()),
        0x97 => Some("DustRemovalData".to_string()),
        0x98 => Some("CropInfo".to_string()),
        0x99 => Some("CustomFunctions2".to_string()),
        0x9a => Some("AspectInfo".to_string()),
        0xa0 => Some("ProcessingInfo".to_string()),
        0xa1 => Some("ToneCurveTable".to_string()),
        0xa2 => Some("SharpnessTable".to_string()),
        0xa3 => Some("SharpnessFreqTable".to_string()),
        0xa4 => Some("WhiteBalanceTable".to_string()),
        0xa9 => Some("ColorBalance".to_string()),
        0xaa => Some("MeasuredColor".to_string()),
        0xae => Some("ColorTemperature".to_string()),
        0xb0 => Some("CanonFlags".to_string()),
        0xb1 => Some("ModifiedInfo".to_string()),
        0xb2 => Some("ToneCurveMatching".to_string()),
        0xb3 => Some("WhiteBalanceMatching".to_string()),
        0xb4 => Some("ColorSpace".to_string()),
        0xb6 => Some("PreviewImageInfo".to_string()),
        0xd0 => Some("VRDOffset".to_string()),
        0xe0 => Some("SensorInfo".to_string()),
        0x4001 => Some("ColorData1".to_string()),
        0x4002 => Some("CRWParam".to_string()),
        0x4003 => Some("ColorInfo".to_string()),
        0x4005 => Some("Flavor".to_string()),
        0x4008 => Some("PictureStyleUserDef".to_string()),
        0x4009 => Some("PictureStylePC".to_string()),
        0x4010 => Some("CustomPictureStyleFileName".to_string()),
        0x4013 => Some("AFMicroAdj".to_string()),
        0x4015 => Some("VignettingCorr".to_string()),
        0x4016 => Some("VignettingCorr2".to_string()),
        0x4018 => Some("LightingOpt".to_string()),
        0x4019 => Some("LensInfo".to_string()),
        0x4020 => Some("AmbienceInfo".to_string()),
        0x4021 => Some("MultiExp".to_string()),
        0x4024 => Some("FilterInfo".to_string()),
        0x4025 => Some("HDRInfo".to_string()),
        0x4028 => Some("AFConfig".to_string()),

        // Synthetic Canon tag IDs in the 0xC000+ range would be handled here
        // These are typically generated by Canon-specific processing
        // For now, return None for unknown tags to fall back to generic naming
        _ => None,
    }
}

// Unit tests are in a separate module
#[cfg(test)]
mod tests;
