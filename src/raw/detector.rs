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

    /// Minolta RAW format (MRW)
    /// ExifTool: lib/Image/ExifTool/MinoltaRaw.pm - Multi-block format with TTW, PRD, WBG blocks
    Minolta,

    /// Panasonic RAW format (RW2, RWL)
    /// ExifTool: lib/Image/ExifTool/PanasonicRaw.pm - TIFF-based with entry-based offsets
    Panasonic,

    /// Olympus RAW format (ORF)
    /// ExifTool: lib/Image/ExifTool/Olympus.pm - TIFF-based with dual processing modes
    Olympus,

    /// Canon RAW format (CR2, CRW, CR3)
    /// ExifTool: lib/Image/ExifTool/Canon.pm - Complex format with 169 ProcessBinaryData sections
    Canon,

    /// Unknown or unsupported RAW format
    Unknown,
    // Future formats will be added here as we implement them:
    // Nikon,     // NEF, NRW formats
    // Sony,      // ARW, SR2, SRF formats
    // Fujifilm,  // RAF format
}

impl RawFormat {
    /// Get the format name as a string
    pub fn name(&self) -> &'static str {
        match self {
            RawFormat::Kyocera => "Kyocera",
            RawFormat::Minolta => "Minolta",
            RawFormat::Panasonic => "Panasonic",
            RawFormat::Olympus => "Olympus",
            RawFormat::Canon => "Canon",
            RawFormat::Unknown => "Unknown",
        }
    }
}

/// Detect RAW format from file type detection result
/// ExifTool: Each manufacturer module has specific detection logic
/// Based on file extension, magic bytes, and manufacturer detection
pub fn detect_raw_format(detection_result: &FileTypeDetectionResult) -> RawFormat {
    // Check for Minolta MRW format
    // ExifTool: MinoltaRaw.pm lines 407-410 - checks for '\0MR[MI]' magic
    if detection_result.file_type == "MRW" {
        return RawFormat::Minolta;
    }

    // Check for Panasonic RW2/RWL formats
    // ExifTool: PanasonicRaw.pm - TIFF-based format
    if detection_result.file_type == "RW2" || detection_result.file_type == "RWL" {
        return RawFormat::Panasonic;
    }

    // Check for Olympus ORF format
    // ExifTool: Olympus.pm - TIFF-based format with dual processing modes
    if detection_result.file_type == "ORF" {
        return RawFormat::Olympus;
    }

    // Check for Canon RAW formats
    // ExifTool: Canon.pm - Multiple RAW format support
    if detection_result.file_type == "CR2"
        || detection_result.file_type == "CRW"
        || detection_result.file_type == "CR3"
    {
        return RawFormat::Canon;
    }

    // Check for Kyocera RAW format
    // ExifTool: KyoceraRaw.pm uses .raw extension + magic validation
    if detection_result.file_type == "RAW" && detection_result.format == "RAW" {
        // For now, assume .raw files are Kyocera format
        // In the future, we'd need magic byte validation here
        return RawFormat::Kyocera;
    }

    // Future format detection will be added here:
    // if detection_result.file_type == "NEF" || detection_result.file_type == "NRW" { return RawFormat::Nikon; }
    // if detection_result.file_type == "ARW" { return RawFormat::Sony; }
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

/// Validate Minolta MRW magic bytes
/// ExifTool: MinoltaRaw.pm lines 407-410 - checks for '\0MR[MI]' header
/// MRW files start with "\0MRM" (big-endian) or "\0MRI" (little-endian from ARW)
pub fn validate_minolta_mrw_magic(data: &[u8]) -> bool {
    // Need at least 8 bytes for MRW header
    if data.len() < 8 {
        return false;
    }

    // Check for MRW magic bytes at start of file
    // ExifTool: MinoltaRaw.pm line 410 - $data =~ /^\0MR([MI])/
    data.starts_with(b"\0MRM") || data.starts_with(b"\0MRI")
}

/// Validate Panasonic RW2/RWL magic bytes
/// ExifTool: PanasonicRaw.pm - TIFF-based format, so validate as TIFF
/// RW2/RWL files are essentially TIFF files with Panasonic-specific tags
pub fn validate_panasonic_rw2_magic(data: &[u8]) -> bool {
    // Need at least 8 bytes for TIFF header
    if data.len() < 8 {
        return false;
    }

    // Check for TIFF magic bytes (big-endian or little-endian)
    // ExifTool: Uses standard TIFF processing for RW2/RWL
    let is_tiff_be = data.starts_with(b"MM\x00\x2A"); // Big-endian TIFF
    let is_tiff_le = data.starts_with(b"II\x2A\x00"); // Little-endian TIFF

    is_tiff_be || is_tiff_le
}

/// Validate Olympus ORF magic bytes
/// ExifTool: Olympus.pm - TIFF-based format with dual processing modes
/// ORF files are essentially TIFF files with Olympus-specific maker note sections
pub fn validate_olympus_orf_magic(data: &[u8]) -> bool {
    // Need at least 8 bytes for TIFF header
    if data.len() < 8 {
        return false;
    }

    // Check for TIFF magic bytes (big-endian or little-endian)
    // ExifTool: Uses standard TIFF processing for ORF files
    let is_tiff_be = data.starts_with(b"MM\x00\x2A"); // Big-endian TIFF
    let is_tiff_le = data.starts_with(b"II\x2A\x00"); // Little-endian TIFF

    is_tiff_be || is_tiff_le
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_format_names() {
        assert_eq!(RawFormat::Kyocera.name(), "Kyocera");
        assert_eq!(RawFormat::Minolta.name(), "Minolta");
        assert_eq!(RawFormat::Panasonic.name(), "Panasonic");
        assert_eq!(RawFormat::Olympus.name(), "Olympus");
        assert_eq!(RawFormat::Canon.name(), "Canon");
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
    fn test_minolta_mrw_magic_validation() {
        // Test valid MRW big-endian magic
        let mrw_be_data = b"\0MRM\x00\x00\x00\x08test_data";
        assert!(validate_minolta_mrw_magic(mrw_be_data));

        // Test valid MRW little-endian magic (from ARW)
        let mrw_le_data = b"\0MRI\x00\x00\x00\x08test_data";
        assert!(validate_minolta_mrw_magic(mrw_le_data));

        // Test invalid magic
        let invalid_data = b"\0MRX\x00\x00\x00\x08test_data";
        assert!(!validate_minolta_mrw_magic(invalid_data));

        // Test insufficient data
        let short_data = b"\0MR";
        assert!(!validate_minolta_mrw_magic(short_data));
    }

    #[test]
    fn test_panasonic_rw2_magic_validation() {
        // Test valid TIFF big-endian magic
        let tiff_be_data = b"MM\x00\x2A\x00\x00\x00\x08";
        assert!(validate_panasonic_rw2_magic(tiff_be_data));

        // Test valid TIFF little-endian magic
        let tiff_le_data = b"II\x2A\x00\x08\x00\x00\x00";
        assert!(validate_panasonic_rw2_magic(tiff_le_data));

        // Test invalid magic
        let invalid_data = b"XX\x2A\x00\x08\x00\x00\x00";
        assert!(!validate_panasonic_rw2_magic(invalid_data));

        // Test insufficient data
        let short_data = b"MM\x00";
        assert!(!validate_panasonic_rw2_magic(short_data));
    }

    #[test]
    fn test_olympus_orf_magic_validation() {
        // Test valid TIFF big-endian magic
        let tiff_be_data = b"MM\x00\x2A\x00\x00\x00\x08";
        assert!(validate_olympus_orf_magic(tiff_be_data));

        // Test valid TIFF little-endian magic
        let tiff_le_data = b"II\x2A\x00\x08\x00\x00\x00";
        assert!(validate_olympus_orf_magic(tiff_le_data));

        // Test invalid magic
        let invalid_data = b"XX\x2A\x00\x08\x00\x00\x00";
        assert!(!validate_olympus_orf_magic(invalid_data));

        // Test insufficient data
        let short_data = b"MM\x00";
        assert!(!validate_olympus_orf_magic(short_data));
    }

    #[test]
    fn test_detect_raw_format() {
        // Test Minolta MRW detection
        let mrw_result = FileTypeDetectionResult {
            file_type: "MRW".to_string(),
            format: "MRW".to_string(),
            mime_type: "image/x-minolta-mrw".to_string(),
            description: "Minolta RAW image".to_string(),
        };
        assert_eq!(detect_raw_format(&mrw_result), RawFormat::Minolta);

        // Test Panasonic RW2 detection
        let rw2_result = FileTypeDetectionResult {
            file_type: "RW2".to_string(),
            format: "RW2".to_string(),
            mime_type: "image/x-panasonic-rw2".to_string(),
            description: "Panasonic RAW image".to_string(),
        };
        assert_eq!(detect_raw_format(&rw2_result), RawFormat::Panasonic);

        // Test Panasonic RWL detection
        let rwl_result = FileTypeDetectionResult {
            file_type: "RWL".to_string(),
            format: "RWL".to_string(),
            mime_type: "image/x-panasonic-rwl".to_string(),
            description: "Panasonic RAW image".to_string(),
        };
        assert_eq!(detect_raw_format(&rwl_result), RawFormat::Panasonic);

        // Test Olympus ORF detection
        let orf_result = FileTypeDetectionResult {
            file_type: "ORF".to_string(),
            format: "ORF".to_string(),
            mime_type: "image/x-olympus-orf".to_string(),
            description: "Olympus RAW image".to_string(),
        };
        assert_eq!(detect_raw_format(&orf_result), RawFormat::Olympus);

        // Test Canon CR2 detection
        let cr2_result = FileTypeDetectionResult {
            file_type: "CR2".to_string(),
            format: "CR2".to_string(),
            mime_type: "image/x-canon-cr2".to_string(),
            description: "Canon RAW image".to_string(),
        };
        assert_eq!(detect_raw_format(&cr2_result), RawFormat::Canon);

        // Test Canon CRW detection
        let crw_result = FileTypeDetectionResult {
            file_type: "CRW".to_string(),
            format: "CRW".to_string(),
            mime_type: "image/x-canon-crw".to_string(),
            description: "Canon RAW image".to_string(),
        };
        assert_eq!(detect_raw_format(&crw_result), RawFormat::Canon);

        // Test Canon CR3 detection
        let cr3_result = FileTypeDetectionResult {
            file_type: "CR3".to_string(),
            format: "CR3".to_string(),
            mime_type: "image/x-canon-cr3".to_string(),
            description: "Canon RAW image".to_string(),
        };
        assert_eq!(detect_raw_format(&cr3_result), RawFormat::Canon);

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
