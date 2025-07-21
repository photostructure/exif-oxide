//! Olympus Equipment tag definitions
//!
//! This module provides tag name lookups for the Olympus Equipment subdirectory.
//! Based on ExifTool: lib/Image/ExifTool/Olympus.pm %Image::ExifTool::Olympus::Equipment
//!
//! Generated from ExifTool source - DO NOT EDIT MANUALLY

/// Get the tag name for an Olympus Equipment tag
/// ExifTool: lib/Image/ExifTool/Olympus.pm Equipment table (lines 1587-1768)
/// This mapping is extracted from ExifTool during codegen
pub fn get_equipment_tag_name(tag_id: u16) -> Option<&'static str> {
    // Tag name mapping extracted from ExifTool Olympus.pm Equipment table
    match tag_id {
        0x0000 => Some("EquipmentVersion"),
        0x0100 => Some("CameraType2"),
        0x0101 => Some("SerialNumber"),
        0x0102 => Some("InternalSerialNumber"),
        0x0103 => Some("FocalPlaneDiagonal"),
        0x0104 => Some("BodyFirmwareVersion"),
        0x0201 => Some("LensType"),
        0x0202 => Some("LensSerialNumber"),
        0x0203 => Some("LensModel"),
        0x0204 => Some("LensFirmwareVersion"),
        0x0205 => Some("MaxApertureAtMinFocal"),
        0x0206 => Some("MaxApertureAtMaxFocal"),
        0x0207 => Some("MinFocalLength"),
        0x0208 => Some("MaxFocalLength"),
        0x020a => Some("MaxAperture"),
        0x020b => Some("LensProperties"),
        0x0301 => Some("Extender"),
        0x0302 => Some("ExtenderSerialNumber"),
        0x0303 => Some("ExtenderModel"),
        0x0304 => Some("ExtenderFirmwareVersion"),
        0x0403 => Some("ConversionLens"),
        0x1000 => Some("FlashType"),
        0x1001 => Some("FlashModel"),
        0x1002 => Some("FlashFirmwareVersion"),
        0x1003 => Some("FlashSerialNumber"),
        _ => None,
    }
}
