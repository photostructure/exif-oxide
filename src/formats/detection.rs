//! File format detection using magic bytes
//!
//! This module handles file format detection using magic bytes and provides
//! utilities for determining file types reliably. Implements magic byte
//! detection for JPEG and TIFF formats following ExifTool patterns.

use crate::types::{ExifError, Result};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

/// Detect file format using magic bytes (primary) with extension fallback
///
/// Reads the first few bytes of the file to identify format by magic signature.
/// This is more reliable than extension-based detection.
pub fn detect_file_format<R: Read + Seek>(mut reader: R) -> Result<FileFormat> {
    let mut magic_bytes = [0u8; 4];
    reader.read_exact(&mut magic_bytes)?;

    // Reset to beginning for subsequent reading
    reader.seek(SeekFrom::Start(0))?;

    match &magic_bytes[0..2] {
        // JPEG magic bytes: 0xFFD8
        [0xFF, 0xD8] => Ok(FileFormat::Jpeg),
        // TIFF magic bytes: "II" (little-endian) or "MM" (big-endian)
        [0x49, 0x49] | [0x4D, 0x4D] => Ok(FileFormat::Tiff),
        _ => {
            // Check for other formats by examining more bytes
            Err(ExifError::Unsupported(
                "Unsupported file format - not a JPEG or TIFF".to_string(),
            ))
        }
    }
}

/// Convenience function to detect format from file path
pub fn detect_file_format_from_path(path: &Path) -> Result<FileFormat> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    detect_file_format(reader)
}

/// Supported file formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    Jpeg,
    Tiff,
    CanonRaw,
    NikonRaw,
    SonyRaw,
    Dng,
}

impl FileFormat {
    /// Get the MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            FileFormat::Jpeg => "image/jpeg",
            FileFormat::Tiff => "image/tiff",
            FileFormat::CanonRaw => "image/x-canon-cr2",
            FileFormat::NikonRaw => "image/x-nikon-nef",
            FileFormat::SonyRaw => "image/x-sony-arw",
            FileFormat::Dng => "image/x-adobe-dng",
        }
    }

    /// Get the typical file extension
    pub fn extension(&self) -> &'static str {
        match self {
            FileFormat::Jpeg => "jpg",
            FileFormat::Tiff => "tif",
            FileFormat::CanonRaw => "cr2",
            FileFormat::NikonRaw => "nef",
            FileFormat::SonyRaw => "arw",
            FileFormat::Dng => "dng",
        }
    }

    /// Get ExifTool-compatible FileType name
    /// ExifTool.pm %fileTypeLookup hash maps extensions to these type names
    /// See: https://github.com/exiftool/exiftool/blob/master/lib/Image/ExifTool.pm#L229-L580
    pub fn file_type(&self) -> &'static str {
        match self {
            FileFormat::Jpeg => "JPEG",
            FileFormat::Tiff => "TIFF",
            FileFormat::CanonRaw => "CR2",
            FileFormat::NikonRaw => "NEF",
            FileFormat::SonyRaw => "ARW",
            FileFormat::Dng => "DNG",
        }
    }

    /// Get ExifTool-compatible FileTypeExtension
    /// Based on ExifTool.pm %fileTypeExt hash overrides
    /// ExifTool.pm:582-592 - Special cases where extension differs from FileType
    /// See: https://github.com/exiftool/exiftool/blob/master/lib/Image/ExifTool.pm#L582-L592
    /// Note: ExifTool does NOT uppercase the extension despite the uc() call in FoundTag
    pub fn file_type_extension(&self) -> &'static str {
        match self {
            // ExifTool %fileTypeExt overrides: 'JPEG' => 'jpg', 'TIFF' => 'tif'
            FileFormat::Jpeg => "jpg", // ExifTool: FileType "JPEG" → Extension "jpg"
            FileFormat::Tiff => "tif", // ExifTool: FileType "TIFF" → Extension "tif"
            // For other formats, FileTypeExtension = lowercase(FileType)
            FileFormat::CanonRaw => "cr2",
            FileFormat::NikonRaw => "nef",
            FileFormat::SonyRaw => "arw",
            FileFormat::Dng => "dng",
        }
    }
}

/// Get format properties for validation and processing
pub fn get_format_properties(format: FileFormat) -> FormatProperties {
    FormatProperties {
        mime_type: format.mime_type(),
        extension: format.extension(),
        supports_exif: matches!(format, FileFormat::Jpeg | FileFormat::Tiff),
        supports_makernotes: matches!(format, FileFormat::Jpeg | FileFormat::Tiff),
    }
}

/// Properties for a detected file format
#[derive(Debug, Clone)]
pub struct FormatProperties {
    pub mime_type: &'static str,
    pub extension: &'static str,
    pub supports_exif: bool,
    pub supports_makernotes: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_jpeg_magic_bytes() {
        let jpeg_magic = [0xFF, 0xD8, 0xFF, 0xE0]; // JPEG magic bytes
        let cursor = Cursor::new(jpeg_magic);
        let format = detect_file_format(cursor).unwrap();
        assert_eq!(format, FileFormat::Jpeg);
    }

    #[test]
    fn test_tiff_magic_bytes_little_endian() {
        let tiff_magic = [0x49, 0x49, 0x2A, 0x00]; // TIFF LE magic bytes
        let cursor = Cursor::new(tiff_magic);
        let format = detect_file_format(cursor).unwrap();
        assert_eq!(format, FileFormat::Tiff);
    }

    #[test]
    fn test_tiff_magic_bytes_big_endian() {
        let tiff_magic = [0x4D, 0x4D, 0x00, 0x2A]; // TIFF BE magic bytes
        let cursor = Cursor::new(tiff_magic);
        let format = detect_file_format(cursor).unwrap();
        assert_eq!(format, FileFormat::Tiff);
    }

    #[test]
    fn test_unsupported_format() {
        let unknown_magic = [0x12, 0x34, 0x56, 0x78];
        let cursor = Cursor::new(unknown_magic);
        let result = detect_file_format(cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_properties() {
        let jpeg_props = get_format_properties(FileFormat::Jpeg);
        assert_eq!(jpeg_props.mime_type, "image/jpeg");
        assert_eq!(jpeg_props.extension, "jpg");
        assert!(jpeg_props.supports_exif);
        assert!(jpeg_props.supports_makernotes);

        let tiff_props = get_format_properties(FileFormat::Tiff);
        assert_eq!(tiff_props.mime_type, "image/tiff");
        assert_eq!(tiff_props.extension, "tif");
        assert!(tiff_props.supports_exif);
        assert!(tiff_props.supports_makernotes);
    }
}
