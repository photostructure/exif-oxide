//! Nikon ProcessBinaryData tables
//!
//! Port of ExifTool's Nikon binary data table definitions for processing
//! Nikon-specific binary data structures.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Nikon.pm (VRInfo, ShotInfo, etc.)"]

use crate::core::binary_data::{BinaryDataTable, BinaryDataTableBuilder};
use crate::core::types::ExifFormat;
use crate::core::{Endian, ExifValue};
use crate::error::Result;
use std::collections::HashMap;

/// Create VRInfo binary data table (Vibration Reduction Info)
pub fn create_vrinfo_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("VRInfo", ExifFormat::U8)
        .add_field(0, "VRInfoVersion", ExifFormat::U8, 4) // 4-byte version string
        .add_field(4, "VibrationReduction", ExifFormat::U8, 1)
        .add_field(6, "VRMode", ExifFormat::U8, 1)
        .build()
}

/// Create ShotInfoD80 binary data table (for Nikon D80)
pub fn create_shotinfo_d80_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("ShotInfoD80", ExifFormat::U8)
        .add_field(0, "ShotInfoVersion", ExifFormat::U8, 4) // Version string
        .add_field(586, "ShutterCount", ExifFormat::U32, 1)
        .add_masked_field(590, "Rotation", ExifFormat::U8, 0x07, 0)
        .add_masked_field(590, "VibrationReduction", ExifFormat::U8, 0x18, 3)
        .add_masked_field(590, "FlashFired", ExifFormat::U8, 0x20, 5)
        .build()
}

/// Create ShotInfoD40 binary data table (for Nikon D40/D40x/D60)
pub fn create_shotinfo_d40_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("ShotInfoD40", ExifFormat::U8)
        .add_field(0, "ShotInfoVersion", ExifFormat::U8, 4)
        .add_field(254, "ShutterCount", ExifFormat::U32, 1)
        .add_masked_field(289, "Rotation", ExifFormat::U8, 0x07, 0)
        .add_masked_field(289, "VibrationReduction", ExifFormat::U8, 0x18, 3)
        .build()
}

/// Create ShotInfoD90 binary data table (for Nikon D90)
pub fn create_shotinfo_d90_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("ShotInfoD90", ExifFormat::U8)
        .add_field(0, "ShotInfoVersion", ExifFormat::U8, 4)
        .add_field(693, "ShutterCount", ExifFormat::U32, 1)
        .add_masked_field(734, "Rotation", ExifFormat::U8, 0x07, 0)
        .add_masked_field(734, "VibrationReduction", ExifFormat::U8, 0x18, 3)
        .add_field(748, "CustomSettingsD90", ExifFormat::U8, 12) // SubDirectory
        .build()
}

/// Create ShotInfoD300 binary data table (for Nikon D300/D300S/D700)
pub fn create_shotinfo_d300_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("ShotInfoD300", ExifFormat::U8)
        .add_field(0, "ShotInfoVersion", ExifFormat::U8, 4)
        .add_field(693, "ShutterCount", ExifFormat::U32, 1)
        .add_masked_field(734, "Rotation", ExifFormat::U8, 0x07, 0)
        .add_masked_field(734, "VibrationReduction", ExifFormat::U8, 0x18, 3)
        .add_masked_field(734, "FlashFired", ExifFormat::U8, 0x20, 5)
        .add_field(748, "CustomSettingsD300", ExifFormat::U8, 12) // SubDirectory
        .build()
}

/// Create ShotInfoD3 binary data table (for Nikon D3/D3S/D3X)
pub fn create_shotinfo_d3_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("ShotInfoD3", ExifFormat::U8)
        .add_field(0, "ShotInfoVersion", ExifFormat::U8, 4)
        .add_field(693, "ShutterCount", ExifFormat::U32, 1)
        .add_masked_field(734, "Rotation", ExifFormat::U8, 0x07, 0)
        .add_masked_field(734, "VibrationReduction", ExifFormat::U8, 0x18, 3)
        .add_masked_field(734, "FlashFired", ExifFormat::U8, 0x20, 5)
        .add_field(748, "CustomSettingsD3", ExifFormat::U8, 12) // SubDirectory
        .build()
}

/// Create ColorBalance1 binary data table
pub fn create_colorbalance1_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("ColorBalance1", ExifFormat::U16)
        .add_field(0, "WB_RBLevels", ExifFormat::U16, 2)
        .build()
}

/// Create ColorBalance2 binary data table
pub fn create_colorbalance2_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("ColorBalance2", ExifFormat::U16)
        .add_field(0, "WB_RBLevels", ExifFormat::U16, 2)
        .add_field(4, "WB_GLevel", ExifFormat::U16, 1)
        .build()
}

/// Create ColorBalance2b binary data table (encrypted)
pub fn create_colorbalance2b_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("ColorBalance2b", ExifFormat::U16)
        .add_field(0, "WB_RBLevelsKelvin", ExifFormat::U16, 2)
        .add_field(4, "WB_GLevel", ExifFormat::U16, 1)
        .build()
}

/// Create ColorBalance3 binary data table
pub fn create_colorbalance3_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("ColorBalance3", ExifFormat::U16)
        .add_field(0, "WB_RBLevels", ExifFormat::U16, 2)
        .build()
}

/// Create ColorBalance4 binary data table
pub fn create_colorbalance4_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("ColorBalance4", ExifFormat::U16)
        .add_field(0, "WB_GRBGLevels", ExifFormat::U16, 4)
        .build()
}

/// Create LensData0100 binary data table
pub fn create_lensdata0100_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("LensData0100", ExifFormat::U8)
        .add_field(0, "LensDataVersion", ExifFormat::U8, 4)
        .add_field(6, "ExitPupilPosition", ExifFormat::U8, 1)
        .add_field(7, "AFAperture", ExifFormat::U8, 1)
        .add_field(9, "FocusPosition", ExifFormat::U8, 1)
        .add_field(10, "FocusDistance", ExifFormat::U8, 1)
        .add_field(11, "FocalLength", ExifFormat::U8, 1)
        .add_field(12, "LensIDNumber", ExifFormat::U8, 1)
        .add_field(13, "LensFStops", ExifFormat::U8, 1)
        .add_field(14, "MinFocalLength", ExifFormat::U8, 1)
        .add_field(15, "MaxFocalLength", ExifFormat::U8, 1)
        .add_field(16, "MaxApertureAtMinFocal", ExifFormat::U8, 1)
        .add_field(17, "MaxApertureAtMaxFocal", ExifFormat::U8, 1)
        .add_field(18, "MCUVersion", ExifFormat::U8, 1)
        .build()
}

/// Create LensData0101 binary data table
pub fn create_lensdata0101_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("LensData0101", ExifFormat::U8)
        .add_field(0, "LensDataVersion", ExifFormat::U8, 4)
        .add_field(6, "ExitPupilPosition", ExifFormat::U8, 1)
        .add_field(7, "AFAperture", ExifFormat::U8, 1)
        .add_field(9, "FocusPosition", ExifFormat::U8, 1)
        .add_field(10, "FocusDistance", ExifFormat::U8, 1)
        .add_field(11, "FocalLength", ExifFormat::U8, 1)
        .add_field(12, "LensIDNumber", ExifFormat::U8, 1)
        .add_field(13, "LensFStops", ExifFormat::U8, 1)
        .add_field(14, "MinFocalLength", ExifFormat::U8, 1)
        .add_field(15, "MaxFocalLength", ExifFormat::U8, 1)
        .add_field(16, "MaxApertureAtMinFocal", ExifFormat::U8, 1)
        .add_field(17, "MaxApertureAtMaxFocal", ExifFormat::U8, 1)
        .add_field(18, "MCUVersion", ExifFormat::U8, 1)
        .add_field(19, "EffectiveMaxAperture", ExifFormat::U8, 1)
        .build()
}

/// Create FlashInfo0100 binary data table
pub fn create_flashinfo0100_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("FlashInfo0100", ExifFormat::U8)
        .add_field(0, "FlashInfoVersion", ExifFormat::U8, 4)
        .add_field(4, "FlashSource", ExifFormat::U8, 1)
        .add_field(6, "ExternalFlashFirmware", ExifFormat::U16, 2)
        .add_field(8, "ExternalFlashFlags", ExifFormat::U8, 1)
        .add_masked_field(8, "FlashCommanderMode", ExifFormat::U8, 0x80, 7)
        .add_masked_field(8, "FlashControlMode", ExifFormat::U8, 0x7f, 0)
        .add_field(9, "FlashOutput", ExifFormat::U8, 1)
        .add_field(10, "FlashCompensation", ExifFormat::U8, 1)
        .add_field(11, "FlashGNDistance", ExifFormat::U8, 1)
        .add_field(12, "FlashGroupControlMode", ExifFormat::U8, 4)
        .add_field(16, "FlashGroupOutput", ExifFormat::U8, 4)
        .add_field(20, "FlashGroupCompensation", ExifFormat::U8, 4)
        .build()
}

/// Create FlashInfo0102 binary data table
pub fn create_flashinfo0102_table() -> BinaryDataTable {
    BinaryDataTableBuilder::new("FlashInfo0102", ExifFormat::U8)
        .add_field(0, "FlashInfoVersion", ExifFormat::U8, 4)
        .add_field(4, "FlashSource", ExifFormat::U8, 1)
        .add_field(6, "ExternalFlashFirmware", ExifFormat::U16, 2)
        .add_field(8, "ExternalFlashFlags", ExifFormat::U8, 1)
        .add_masked_field(8, "FlashCommanderMode", ExifFormat::U8, 0x80, 7)
        .add_masked_field(8, "FlashControlMode", ExifFormat::U8, 0x7f, 0)
        .add_field(9, "FlashOutput", ExifFormat::U8, 1)
        .add_field(10, "FlashCompensation", ExifFormat::U8, 1)
        .add_field(11, "FlashGNDistance", ExifFormat::U8, 1)
        .add_field(12, "FlashGroupControlMode", ExifFormat::U8, 4)
        .add_field(16, "FlashGroupOutput", ExifFormat::U8, 4)
        .add_field(20, "FlashGroupCompensation", ExifFormat::U8, 4)
        .add_field(24, "FlashColorFilter", ExifFormat::U8, 1)
        .build()
}

/// ProcessBinaryData implementation for Nikon binary tables
pub struct NikonBinaryDataProcessor;

impl NikonBinaryDataProcessor {
    /// Process Nikon binary data based on tag type and version
    pub fn process_binary_data(
        tag: u16,
        data: &[u8],
        byte_order: Endian,
        version: Option<&str>,
        model: Option<&str>,
    ) -> Result<HashMap<u16, ExifValue>> {
        match tag {
            // VRInfo (Vibration Reduction Info)
            0x001f => {
                let table = create_vrinfo_table();
                table.parse(data, byte_order)
            }

            // ShotInfo - varies by camera model and version
            0x0010 => Self::process_shotinfo(data, byte_order, version, model),

            // ColorBalance series
            0x0097 => Self::process_colorbalance(data, byte_order, version),

            // LensData
            0x0098 => Self::process_lensdata(data, byte_order, version),

            // FlashInfo
            0x00a8 => Self::process_flashinfo(data, byte_order, version),

            _ => {
                // Unknown binary data tag - return empty for now
                Ok(HashMap::new())
            }
        }
    }

    /// Process ShotInfo based on camera model and version
    fn process_shotinfo(
        data: &[u8],
        byte_order: Endian,
        _version: Option<&str>,
        model: Option<&str>,
    ) -> Result<HashMap<u16, ExifValue>> {
        let table = if let Some(model_name) = model {
            if model_name.contains("D40") || model_name.contains("D60") {
                create_shotinfo_d40_table()
            } else if model_name.contains("D80") {
                create_shotinfo_d80_table()
            } else if model_name.contains("D90") {
                create_shotinfo_d90_table()
            } else if model_name.contains("D300") || model_name.contains("D700") {
                create_shotinfo_d300_table()
            } else if model_name.contains("D3") {
                create_shotinfo_d3_table()
            } else {
                // Default to D80 format for unknown models
                create_shotinfo_d80_table()
            }
        } else {
            // No model info - use default
            create_shotinfo_d80_table()
        };

        table.parse(data, byte_order)
    }

    /// Process ColorBalance based on version
    fn process_colorbalance(
        data: &[u8],
        byte_order: Endian,
        version: Option<&str>,
    ) -> Result<HashMap<u16, ExifValue>> {
        let table = match version {
            Some("0100") => create_colorbalance1_table(),
            Some("0102") | Some("0103") => create_colorbalance2_table(),
            Some("0208") => create_colorbalance2b_table(),
            Some("0209") => create_colorbalance3_table(),
            Some("0400") => create_colorbalance4_table(),
            _ => create_colorbalance1_table(), // Default
        };

        table.parse(data, byte_order)
    }

    /// Process LensData based on version
    fn process_lensdata(
        data: &[u8],
        byte_order: Endian,
        version: Option<&str>,
    ) -> Result<HashMap<u16, ExifValue>> {
        let table = match version {
            Some("0100") => create_lensdata0100_table(),
            Some("0101") => create_lensdata0101_table(),
            _ => create_lensdata0100_table(), // Default
        };

        table.parse(data, byte_order)
    }

    /// Process FlashInfo based on version
    fn process_flashinfo(
        data: &[u8],
        byte_order: Endian,
        version: Option<&str>,
    ) -> Result<HashMap<u16, ExifValue>> {
        let table = match version {
            Some("0100") => create_flashinfo0100_table(),
            Some("0102") => create_flashinfo0102_table(),
            _ => create_flashinfo0100_table(), // Default
        };

        table.parse(data, byte_order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vrinfo_table_creation() {
        let table = create_vrinfo_table();
        assert_eq!(table.name, "VRInfo");
        assert_eq!(table.default_format, ExifFormat::U8);
        assert!(!table.fields.is_empty());
    }

    #[test]
    fn test_shotinfo_d80_table() {
        let table = create_shotinfo_d80_table();
        assert_eq!(table.name, "ShotInfoD80");
        assert!(table.fields.iter().any(|(pos, _)| *pos == 0)); // ShotInfoVersion
        assert!(table.fields.iter().any(|(pos, _)| *pos == 586)); // ShutterCount
        assert!(table.fields.iter().any(|(pos, _)| *pos == 590)); // Bit-masked fields
    }

    #[test]
    fn test_binary_data_processor() {
        // Test VRInfo processing with minimal data
        let data = vec![0x30, 0x31, 0x30, 0x30, 0x01, 0x00, 0x02]; // "0100" + VR data
        let result = NikonBinaryDataProcessor::process_binary_data(
            0x001f, // VRInfo tag
            &data,
            Endian::Little,
            None,
            None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_model_based_shotinfo_selection() {
        let data = vec![0x30; 800]; // Dummy data of sufficient size

        // Test D40 model
        let result = NikonBinaryDataProcessor::process_binary_data(
            0x0010,
            &data,
            Endian::Little,
            None,
            Some("NIKON D40"),
        );
        assert!(result.is_ok());

        // Test D300 model
        let result = NikonBinaryDataProcessor::process_binary_data(
            0x0010,
            &data,
            Endian::Little,
            None,
            Some("NIKON D300"),
        );
        assert!(result.is_ok());
    }
}
