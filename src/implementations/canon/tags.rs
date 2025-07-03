//! Canon tag name mapping
//!
//! This module provides Canon-specific tag name resolution for synthetic Canon tag IDs.
//! These tag names come directly from ExifTool's Canon.pm Main table to ensure
//! exact compatibility with ExifTool's Canon tag naming conventions.
//!
//! **Trust ExifTool**: This code translates ExifTool's Canon tag mappings verbatim
//! without any improvements or simplifications. Every tag ID and name mapping
//! is copied exactly as documented in the ExifTool source.
//!
//! Primary ExifTool References:
//! - lib/Image/ExifTool/Canon.pm Main table - Canon tag ID to name mappings

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
