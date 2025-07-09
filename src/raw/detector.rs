//! RAW format detection and classification
//!
//! This module implements manufacturer-specific RAW format detection following
//! ExifTool's exact logic for determining which RAW handler to use.

use crate::file_detection::FileTypeDetectionResult;

/// RAW format types supported by exif-oxide
/// ExifTool: Each manufacturer module defines specific RAW format variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RawFormat {
    /// Kyocera Contax N Digital RAW format
    /// ExifTool: lib/Image/ExifTool/KyoceraRaw.pm - Simple ProcessBinaryData format
    Kyocera,

    /// Unknown or unsupported RAW format
    Unknown,
    // Future formats will be added here as we implement them:
    // Canon,     // CR2, CR3 formats
    // Nikon,     // NEF, NRW formats
    // Sony,      // ARW, SR2, SRF formats
    // Olympus,   // ORF format
    // Panasonic, // RW2 format
    // Fujifilm,  // RAF format
}

impl RawFormat {
    /// Get the format name as a string
    pub fn name(&self) -> &'static str {
        match self {
            RawFormat::Kyocera => "Kyocera",
            RawFormat::Unknown => "Unknown",
        }
    }
}

/// Detect RAW format from file type detection result
/// ExifTool: Each manufacturer module has specific detection logic
/// Based on file extension, magic bytes, and manufacturer detection
pub fn detect_raw_format(detection_result: &FileTypeDetectionResult) -> RawFormat {
    // Check for Kyocera RAW format first
    // ExifTool: KyoceraRaw.pm uses .raw extension + magic validation
    if detection_result.file_type == "RAW" && detection_result.format == "RAW" {
        // For now, assume .raw files are Kyocera format
        // In the future, we'd need magic byte validation here
        return RawFormat::Kyocera;
    }

    // Future format detection will be added here:
    // if detection_result.file_type == "CR2" { return RawFormat::Canon; }
    // if detection_result.file_type == "NEF" || detection_result.file_type == "NRW" { return RawFormat::Nikon; }
    // if detection_result.file_type == "ARW" { return RawFormat::Sony; }
    // if detection_result.file_type == "ORF" { return RawFormat::Olympus; }
    // if detection_result.file_type == "RW2" { return RawFormat::Panasonic; }
    // if detection_result.file_type == "RAF" { return RawFormat::Fujifilm; }

    RawFormat::Unknown
}

/// Validate Kyocera RAW magic bytes
/// ExifTool: KyoceraRaw.pm lines 60-65 - checks for 'ARECOYK' at offset 0x19
/// This is 'KYOCERA' reversed, which is how Kyocera stores the magic string
pub fn validate_kyocera_magic(data: &[u8]) -> bool {
    // Need at least 0x19 + 7 bytes to check magic
    if data.len() < 0x19 + 7 {
        return false;
    }

    // Check for reversed 'KYOCERA' string at offset 0x19
    // ExifTool: KyoceraRaw.pm line 65 - $val{19} eq 'ARECOYK'
    let magic_offset = 0x19;
    let expected_magic = b"ARECOYK";

    &data[magic_offset..magic_offset + 7] == expected_magic
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_format_names() {
        assert_eq!(RawFormat::Kyocera.name(), "Kyocera");
        assert_eq!(RawFormat::Unknown.name(), "Unknown");
    }

    #[test]
    fn test_kyocera_magic_validation() {
        // Create test data with Kyocera magic at offset 0x19
        let mut data = vec![0u8; 0x19 + 10];
        data[0x19..0x19 + 7].copy_from_slice(b"ARECOYK");

        assert!(validate_kyocera_magic(&data));

        // Test with wrong magic
        let mut bad_data = vec![0u8; 0x19 + 10];
        bad_data[0x19..0x19 + 7].copy_from_slice(b"WRONGXY");

        assert!(!validate_kyocera_magic(&bad_data));

        // Test with insufficient data
        let short_data = vec![0u8; 10];
        assert!(!validate_kyocera_magic(&short_data));
    }

    #[test]
    fn test_detect_raw_format() {
        // Test Kyocera detection
        let kyocera_result = FileTypeDetectionResult {
            file_type: "RAW".to_string(),
            format: "RAW".to_string(),
            mime_type: "application/octet-stream".to_string(),
            description: "RAW image".to_string(),
        };

        assert_eq!(detect_raw_format(&kyocera_result), RawFormat::Kyocera);

        // Test unknown format
        let unknown_result = FileTypeDetectionResult {
            file_type: "UNKNOWN".to_string(),
            format: "UNKNOWN".to_string(),
            mime_type: "application/octet-stream".to_string(),
            description: "Unknown format".to_string(),
        };

        assert_eq!(detect_raw_format(&unknown_result), RawFormat::Unknown);
    }
}
