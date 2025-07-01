//! File format detection and JPEG parsing
//!
//! This module handles file format detection using magic bytes and provides
//! JPEG segment scanning to locate EXIF data. Implements Milestone 1:
//! - Real file I/O with streaming support (Read + Seek traits)
//! - JPEG magic bytes detection (0xFFD8)
//! - JPEG segment scanner to find APP1 (EXIF) segments
//! - Graceful handling of non-JPEG files

use crate::exif::ExifReader;
use crate::generated::{EXIF_MAIN_TAGS, REQUIRED_PRINT_CONV, REQUIRED_VALUE_CONV};
use crate::types::{ExifData, ExifError, Result, TagValue};
use std::collections::HashMap;
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
}

/// JPEG segment types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JpegSegment {
    /// Start of Image (0xD8)
    Soi,
    /// Application segments 0-15 (APP0-APP15)
    App(u8),
    /// Start of Frame (0xC0)
    Sof,
    /// Define Huffman Table (0xC4)
    Dht,
    /// Start of Scan (0xDA)
    Sos,
    /// End of Image (0xD9)
    Eoi,
    /// Other segments
    Other(u8),
}

impl JpegSegment {
    fn from_marker(marker: u8) -> Self {
        match marker {
            0xD8 => Self::Soi,
            0xE0..=0xEF => Self::App(marker - 0xE0),
            0xC0 => Self::Sof,
            0xC4 => Self::Dht,
            0xDA => Self::Sos,
            0xD9 => Self::Eoi,
            _ => Self::Other(marker),
        }
    }

    /// Check if this is an APP1 segment (contains EXIF)
    #[allow(dead_code)]
    fn is_app1(&self) -> bool {
        matches!(self, Self::App(1))
    }

    /// Get the marker byte for this segment
    #[allow(dead_code)]
    fn marker_byte(&self) -> u8 {
        match self {
            Self::Soi => 0xD8,
            Self::App(app_num) => 0xE0 + app_num,
            Self::Sof => 0xC0,
            Self::Dht => 0xC4,
            Self::Sos => 0xDA,
            Self::Eoi => 0xD9,
            Self::Other(marker) => *marker,
        }
    }
}

/// JPEG segment scanner result
#[derive(Debug)]
pub struct JpegSegmentInfo {
    pub segment_type: JpegSegment,
    pub offset: u64,
    pub length: u16,
    pub has_exif: bool,
}

/// Scan JPEG file for segments and locate EXIF data
///
/// Returns information about APP1 segment containing EXIF data if found.
/// This implements the core of Milestone 1's JPEG segment scanning.
pub fn scan_jpeg_segments<R: Read + Seek>(mut reader: R) -> Result<Option<JpegSegmentInfo>> {
    // Verify JPEG magic bytes
    let mut magic = [0u8; 2];
    reader.read_exact(&mut magic)?;
    if magic != [0xFF, 0xD8] {
        return Err(ExifError::InvalidFormat(
            "Not a valid JPEG file (missing 0xFFD8 magic bytes)".to_string(),
        ));
    }

    let mut current_pos = 2u64; // After SOI marker

    loop {
        // Read segment marker
        let mut marker_bytes = [0u8; 2];
        if reader.read_exact(&mut marker_bytes).is_err() {
            // End of file reached without finding EXIF
            break;
        }

        if marker_bytes[0] != 0xFF {
            return Err(ExifError::ParseError(
                "Invalid JPEG segment marker".to_string(),
            ));
        }

        let segment = JpegSegment::from_marker(marker_bytes[1]);
        current_pos += 2;

        match segment {
            JpegSegment::Soi => {
                // Already processed
                continue;
            }
            JpegSegment::Eoi => {
                // End of image
                break;
            }
            JpegSegment::Sos => {
                // Start of scan - no more metadata segments
                break;
            }
            JpegSegment::App(app_num) => {
                // Read segment length
                let mut length_bytes = [0u8; 2];
                reader.read_exact(&mut length_bytes)?;
                let length = u16::from_be_bytes(length_bytes);
                current_pos += 2;

                if app_num == 1 {
                    // APP1 segment - check for EXIF
                    let mut exif_header = [0u8; 6]; // Read "Exif\0\0"
                    if reader.read_exact(&mut exif_header).is_ok() {
                        // Check for EXIF identifier
                        if &exif_header[0..4] == b"Exif"
                            && exif_header[4] == 0
                            && exif_header[5] == 0
                        {
                            // ExifTool: lib/Image/ExifTool/JPEG.pm:48 - "Exif\0" condition
                            // The TIFF data starts immediately after "Exif\0\0"
                            return Ok(Some(JpegSegmentInfo {
                                segment_type: segment,
                                offset: current_pos + 6, // After "Exif\0\0" (6 bytes)
                                length: length - 8, // Subtract segment length header (2 bytes) + "Exif\0\0" (6 bytes) = 8 total
                                has_exif: true,
                            }));
                        }
                    }

                    // Reset to start of segment data
                    reader.seek(SeekFrom::Start(current_pos + 2))?;
                }

                // Skip to next segment
                let segment_data_length = length.saturating_sub(2) as u64;
                reader.seek(SeekFrom::Current(segment_data_length as i64))?;
                current_pos += segment_data_length;
            }
            _ => {
                // Other segments - skip them
                let mut length_bytes = [0u8; 2];
                if reader.read_exact(&mut length_bytes).is_ok() {
                    let length = u16::from_be_bytes(length_bytes);
                    let segment_data_length = length.saturating_sub(2) as u64;
                    reader.seek(SeekFrom::Current(segment_data_length as i64))?;
                    current_pos += 2 + segment_data_length;
                } else {
                    break;
                }
            }
        }
    }

    Ok(None)
}

/// Extract metadata from a file (Milestone 1: real file I/O with JPEG detection)
///
/// This function now implements real file reading and JPEG segment scanning.
/// It detects JPEG files by magic bytes and locates EXIF data in APP1 segments.
pub fn extract_metadata(path: &Path, show_missing: bool) -> Result<ExifData> {
    // Open file with buffered reading for performance
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // Detect format using magic bytes
    let format = detect_file_format(&mut reader)?;

    // Get actual file metadata
    let file_metadata = std::fs::metadata(path)?;
    let file_size = file_metadata.len();

    let mut tags = HashMap::new();

    // Basic file information (now real data)
    tags.insert(
        "FileName".to_string(),
        TagValue::String(
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        ),
    );

    tags.insert(
        "Directory".to_string(),
        TagValue::String(
            path.parent()
                .unwrap_or_else(|| Path::new("."))
                .to_string_lossy()
                .to_string(),
        ),
    );

    tags.insert(
        "FileSize".to_string(),
        TagValue::String(format!("{file_size} bytes")),
    );

    // Format file modification time
    if let Ok(modified) = file_metadata.modified() {
        if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
            tags.insert(
                "FileModifyDate".to_string(),
                TagValue::String(format!("{} seconds since epoch", duration.as_secs())),
            );
        }
    }

    tags.insert(
        "FileType".to_string(),
        TagValue::String(format!("{format:?}")),
    );
    tags.insert(
        "FileTypeExtension".to_string(),
        TagValue::String(format.extension().to_string()),
    );
    tags.insert(
        "MIMEType".to_string(),
        TagValue::String(format.mime_type().to_string()),
    );

    // Format-specific processing
    match format {
        FileFormat::Jpeg => {
            // Scan for EXIF data in JPEG segments
            match scan_jpeg_segments(&mut reader)? {
                Some(segment_info) => {
                    let exif_status = format!(
                        "EXIF data found in APP1 segment at offset {:#x}, length {} bytes",
                        segment_info.offset, segment_info.length
                    );

                    // Add EXIF detection status
                    tags.insert(
                        "ExifDetectionStatus".to_string(),
                        TagValue::String(exif_status),
                    );

                    // Extract actual EXIF data using our new ExifReader
                    reader.seek(SeekFrom::Start(segment_info.offset))?;
                    let mut exif_data = vec![0u8; segment_info.length as usize];
                    reader.read_exact(&mut exif_data)?;

                    // Parse EXIF data
                    let mut exif_reader = ExifReader::new();
                    match exif_reader.parse_exif_data(&exif_data) {
                        Ok(()) => {
                            // Successfully parsed EXIF - extract all found tags
                            let exif_tags = exif_reader.get_all_tags();
                            for (tag_name, tag_value) in exif_tags {
                                tags.insert(tag_name, tag_value);
                            }

                            // Add EXIF parsing status
                            let header = exif_reader.get_header().unwrap();
                            tags.insert(
                                "ExifByteOrder".to_string(),
                                TagValue::String(
                                    match header.byte_order {
                                        crate::exif::ByteOrder::LittleEndian => {
                                            "Little-endian (Intel)"
                                        }
                                        crate::exif::ByteOrder::BigEndian => {
                                            "Big-endian (Motorola)"
                                        }
                                    }
                                    .to_string(),
                                ),
                            );

                            // Include any parsing warnings
                            let warnings = exif_reader.get_warnings();
                            if !warnings.is_empty() {
                                tags.insert(
                                    "ExifWarnings".to_string(),
                                    TagValue::String(format!("{} warnings", warnings.len())),
                                );
                            }
                        }
                        Err(e) => {
                            // EXIF parsing failed - include error but continue
                            tags.insert(
                                "ExifParseError".to_string(),
                                TagValue::String(format!("Failed to parse EXIF data: {e}")),
                            );
                        }
                    }
                }
                None => {
                    // No EXIF data available
                    tags.insert(
                        "ExifDetectionStatus".to_string(),
                        TagValue::String("No EXIF data found in JPEG file".to_string()),
                    );
                }
            }
        }
        FileFormat::Tiff => {
            tags.insert(
                "ExifDetectionStatus".to_string(),
                TagValue::String(
                    "TIFF format detected but parsing not implemented yet".to_string(),
                ),
            );
        }
        _ => {
            tags.insert(
                "ExifDetectionStatus".to_string(),
                TagValue::String(format!(
                    "{format:?} format detected but parsing not implemented yet"
                )),
            );
        }
    }

    let missing_implementations = if show_missing {
        let mut missing = vec![
            "✅ Real file I/O and size detection - IMPLEMENTED in Milestone 1".to_string(),
            "✅ Magic byte file type detection - IMPLEMENTED in Milestone 1".to_string(),
            "✅ JPEG segment parsing - IMPLEMENTED in Milestone 1".to_string(),
            "✅ EXIF header parsing with endianness detection - IMPLEMENTED in Milestone 2"
                .to_string(),
            "✅ IFD (Image File Directory) parsing - IMPLEMENTED in Milestone 2".to_string(),
            "✅ Basic tag value extraction (ASCII/SHORT/LONG) - IMPLEMENTED in Milestone 2"
                .to_string(),
            "✅ Make/Model/Software extraction - IMPLEMENTED in Milestone 2".to_string(),
            "🔄 Additional EXIF formats (RATIONAL, BYTE) - NEXT in Milestone 3".to_string(),
            "🔄 Offset handling for long values - NEXT in Milestone 3".to_string(),
            "⏳ MakerNote parsing - Future milestone".to_string(),
            "⏳ Subdirectory following - Future milestone".to_string(),
            "⏳ GPS coordinate conversion - Future milestone".to_string(),
            "⏳ Date/time parsing - Future milestone".to_string(),
        ];

        // Add generated tag information
        missing.push(format!(
            "Generated {} mainstream EXIF tags from ExifTool",
            EXIF_MAIN_TAGS.len()
        ));
        missing.push(format!(
            "PrintConv implementations needed: {} functions",
            REQUIRED_PRINT_CONV.len()
        ));
        missing.push(format!(
            "ValueConv implementations needed: {} functions",
            REQUIRED_VALUE_CONV.len()
        ));

        // Add some example missing conversions
        if !REQUIRED_PRINT_CONV.is_empty() {
            missing.push(format!(
                "Example missing PrintConv: {}",
                REQUIRED_PRINT_CONV[0]
            ));
        }
        if !REQUIRED_VALUE_CONV.is_empty() {
            missing.push(format!(
                "Example missing ValueConv: {}",
                REQUIRED_VALUE_CONV[0]
            ));
        }

        Some(missing)
    } else {
        None
    };

    Ok(ExifData {
        source_file: path.to_string_lossy().to_string(),
        exif_tool_version: "0.1.0-oxide".to_string(),
        tags,
        errors: vec![], // No errors in mock implementation
        missing_implementations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_format_detection() {
        // Test JPEG magic bytes
        let jpeg_data = [0xFF, 0xD8, 0xFF, 0xE0]; // JPEG magic
        let mut cursor = std::io::Cursor::new(jpeg_data);
        assert_eq!(detect_file_format(&mut cursor).unwrap(), FileFormat::Jpeg);

        // Test TIFF magic bytes (little-endian)
        let tiff_data = [0x49, 0x49, 0x2A, 0x00]; // TIFF LE magic
        let mut cursor = std::io::Cursor::new(tiff_data);
        assert_eq!(detect_file_format(&mut cursor).unwrap(), FileFormat::Tiff);

        // Test TIFF magic bytes (big-endian)
        let tiff_data = [0x4D, 0x4D, 0x00, 0x2A]; // TIFF BE magic
        let mut cursor = std::io::Cursor::new(tiff_data);
        assert_eq!(detect_file_format(&mut cursor).unwrap(), FileFormat::Tiff);

        // Test unsupported format
        let unknown_data = [0x12, 0x34, 0x56, 0x78];
        let mut cursor = std::io::Cursor::new(unknown_data);
        assert!(detect_file_format(&mut cursor).is_err());
    }

    #[test]
    fn test_format_properties() {
        assert_eq!(FileFormat::Jpeg.mime_type(), "image/jpeg");
        assert_eq!(FileFormat::Jpeg.extension(), "jpg");
        assert_eq!(FileFormat::CanonRaw.mime_type(), "image/x-canon-cr2");
    }

    #[test]
    fn test_extract_metadata() {
        // Use a real test image file
        let test_file = std::path::Path::new("test-images/canon/Canon_T3i.JPG");

        // Skip test if file doesn't exist (CI environments might not have test images)
        if !test_file.exists() {
            eprintln!(
                "Skipping test - test image not found: {}",
                test_file.display()
            );
            return;
        }

        let metadata = extract_metadata(test_file, false).unwrap();

        assert_eq!(metadata.source_file, test_file.to_string_lossy());
        assert_eq!(metadata.exif_tool_version, "0.1.0-oxide");
        assert!(metadata.tags.contains_key("FileName"));
        assert!(metadata.tags.contains_key("FileType"));
        assert!(metadata.tags.contains_key("ExifDetectionStatus"));
        assert!(metadata.missing_implementations.is_none());

        // Should extract real EXIF data
        assert!(metadata.tags.contains_key("Make"));
        assert!(metadata.tags.contains_key("Model"));

        // Verify the extracted values match what we expect from this Canon image
        if let Some(make) = metadata.tags.get("Make") {
            assert_eq!(make.as_string(), Some("Canon"));
        }
        if let Some(model) = metadata.tags.get("Model") {
            assert_eq!(model.as_string(), Some("Canon EOS REBEL T3i"));
        }

        // Test with --show-missing
        let metadata_with_missing = extract_metadata(test_file, true).unwrap();
        assert!(metadata_with_missing.missing_implementations.is_some());
    }
}
