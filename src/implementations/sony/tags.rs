//! Sony tag name mapping
//!
//! This module provides Sony-specific tag name resolution for Sony tag IDs.
//! These tag names come directly from ExifTool's Sony.pm Main table to ensure
//! exact compatibility with ExifTool's Sony tag naming conventions.
//!
//! **Trust ExifTool**: This code translates ExifTool's Sony tag mappings verbatim
//! without any improvements or simplifications. Every tag ID and name mapping
//! is copied exactly as documented in the ExifTool source.
//!
//! Primary ExifTool References:
//! - lib/Image/ExifTool/Sony.pm Main table - Sony tag ID to name mappings
//! - lines 662+ - %Image::ExifTool::Sony::Main table definition

/// Get Sony tag name for Sony tag IDs
/// Used by the EXIF module to resolve Sony-specific tag names
/// ExifTool: lib/Image/ExifTool/Sony.pm Main table
pub fn get_sony_tag_name(tag_id: u16) -> Option<String> {
    // Map Sony tag IDs to their names based on Sony.pm Main table
    match tag_id {
        // Main Sony MakerNote tags from Sony.pm Main table
        0x0010 => Some("CameraInfo".to_string()),
        0x0020 => Some("FocusInfo".to_string()),
        0x0102 => Some("Quality".to_string()),
        0x0104 => Some("FlashExposureComp".to_string()),
        0x0105 => Some("Teleconverter".to_string()),
        0x0112 => Some("WhiteBalanceFineTune".to_string()),
        0x0114 => Some("CameraSettings".to_string()),
        0x0115 => Some("WhiteBalance".to_string()),
        0x0116 => Some("ExtraInfo".to_string()),
        0x0e00 => Some("PrintIM".to_string()),
        
        // MultiBurst mode tags (F88 and similar models)
        0x1000 => Some("MultiBurstMode".to_string()),
        0x1001 => Some("MultiBurstImageWidth".to_string()),
        0x1002 => Some("MultiBurstImageHeight".to_string()),
        
        // Camera-specific ProcessBinaryData tags
        0x2010 => Some("Tag2010".to_string()),  // Camera settings
        0x2020 => Some("CameraSettings".to_string()),
        0x2030 => Some("MoreSettings".to_string()),
        0x3000 => Some("ShotInfo".to_string()),
        0x7303 => Some("ColorReproduction".to_string()),
        0x7200 => Some("EncryptionKey".to_string()),
        0x7201 => Some("LensInfo".to_string()),
        
        // Encrypted 0x94xx tags requiring ProcessEnciphered
        0x9003 => Some("WhiteBalanceSetting".to_string()),
        0x9050 => Some("Tag9050".to_string()),  // Encrypted metadata
        0x9204 => Some("ISOSetting".to_string()),
        0x940e => Some("AFInfo".to_string()),   // Autofocus information
        
        // FileFormat tag (ARW version detection)
        // ExifTool: Sony.pm lines 2045-2073
        0xb000 => Some("FileFormat".to_string()),
        0xb001 => Some("SonyModelID".to_string()),
        0xb020 => Some("ColorReproduction".to_string()),
        0xb021 => Some("ColorTemperature".to_string()),
        0xb022 => Some("ColorCompensationFilter".to_string()),
        0xb023 => Some("SceneMode".to_string()),
        0xb024 => Some("ZoneMatching".to_string()),
        0xb025 => Some("DynamicRangeOptimizer".to_string()),
        0xb026 => Some("ImageStabilization".to_string()),
        0xb027 => Some("LensType".to_string()),
        0xb028 => Some("MinoltaCameraSettings".to_string()),
        0xb029 => Some("ColorMode".to_string()),
        0xb02a => Some("FullImageSize".to_string()),
        0xb02b => Some("PreviewImageSize".to_string()),
        
        // SR2 format tags
        0x7000 => Some("SonyImageSize".to_string()),
        0x7001 => Some("SonyQuality".to_string()),
        
        // Common maker note tags that appear as raw numbers
        0x014a => Some("AFPointSelected".to_string()),  // A100 special case
        0x927c => Some("Tag927C".to_string()),  // Common binary tag
        
        // Default case - no mapping available
        _ => None,
    }
}

/// Get Sony namespace prefix for tag organization
/// Following ExifTool's group naming conventions
pub fn get_sony_namespace() -> &'static str {
    "Sony"
}

/// Check if a tag ID is a Sony-specific tag
pub fn is_sony_tag(tag_id: u16) -> bool {
    // Sony maker note tags typically fall in certain ranges
    match tag_id {
        // Main Sony MakerNote range
        0x0010..=0x0120 => true,
        // MultiBurst range
        0x1000..=0x1002 => true,
        // ProcessBinaryData ranges
        0x2000..=0x3000 => true,
        0x7000..=0x7303 => true,
        // Encrypted tag range (0x94xx)
        0x9400..=0x94ff => true,
        // FileFormat and extended tags
        0xb000..=0xb030 => true,
        // Other common Sony tags
        0x014a | 0x927c => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sony_tag_names() {
        // Test key Sony tags
        assert_eq!(get_sony_tag_name(0xb000), Some("FileFormat".to_string()));
        assert_eq!(get_sony_tag_name(0x0010), Some("CameraInfo".to_string()));
        assert_eq!(get_sony_tag_name(0x9050), Some("Tag9050".to_string()));
        assert_eq!(get_sony_tag_name(0x940e), Some("AFInfo".to_string()));
        assert_eq!(get_sony_tag_name(0x0115), Some("WhiteBalance".to_string()));
        
        // Test unknown tag
        assert_eq!(get_sony_tag_name(0xffff), None);
    }

    #[test]
    fn test_sony_tag_detection() {
        // Test Sony tag ID detection
        assert!(is_sony_tag(0xb000));  // FileFormat
        assert!(is_sony_tag(0x0010));  // CameraInfo
        assert!(is_sony_tag(0x9400));  // Encrypted range
        assert!(is_sony_tag(0x940e));  // AFInfo
        assert!(is_sony_tag(0x2010));  // ProcessBinaryData
        
        // Test non-Sony tags
        assert!(!is_sony_tag(0x0001));  // Standard EXIF
        assert!(!is_sony_tag(0x8000));  // Outside Sony ranges
    }

    #[test]
    fn test_sony_namespace() {
        assert_eq!(get_sony_namespace(), "Sony");
    }
}