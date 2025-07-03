//! JPEG-specific processing and segment scanning
//!
//! This module implements JPEG segment scanning to locate EXIF data,
//! following ExifTool's JPEG.pm implementation for segment parsing
//! and EXIF data extraction.

use crate::types::{ExifError, Result};
use std::io::{Read, Seek, SeekFrom};

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

/// Extract EXIF data from JPEG file
///
/// This function scans the JPEG for APP1 segments containing EXIF data
/// and returns the raw EXIF/TIFF data for further processing.
pub fn extract_jpeg_exif<R: Read + Seek>(mut reader: R) -> Result<Vec<u8>> {
    // Scan for EXIF segment
    reader.seek(SeekFrom::Start(0))?;
    let segment_info = scan_jpeg_segments(&mut reader)?;

    match segment_info {
        Some(info) if info.has_exif => {
            // Read EXIF data
            reader.seek(SeekFrom::Start(info.offset))?;
            let mut exif_data = vec![0u8; info.length as usize];
            reader.read_exact(&mut exif_data)?;
            Ok(exif_data)
        }
        _ => Err(ExifError::InvalidFormat(
            "No EXIF data found in JPEG file".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_jpeg_segment_from_marker() {
        assert_eq!(JpegSegment::from_marker(0xD8), JpegSegment::Soi);
        assert_eq!(JpegSegment::from_marker(0xE1), JpegSegment::App(1));
        assert_eq!(JpegSegment::from_marker(0xC0), JpegSegment::Sof);
        assert_eq!(JpegSegment::from_marker(0xDA), JpegSegment::Sos);
        assert_eq!(JpegSegment::from_marker(0xD9), JpegSegment::Eoi);
    }

    #[test]
    fn test_jpeg_segment_is_app1() {
        assert!(JpegSegment::App(1).is_app1());
        assert!(!JpegSegment::App(0).is_app1());
        assert!(!JpegSegment::Soi.is_app1());
    }

    #[test]
    fn test_jpeg_segment_marker_byte() {
        assert_eq!(JpegSegment::Soi.marker_byte(), 0xD8);
        assert_eq!(JpegSegment::App(1).marker_byte(), 0xE1);
        assert_eq!(JpegSegment::Eoi.marker_byte(), 0xD9);
    }

    #[test]
    fn test_scan_jpeg_segments_invalid_magic() {
        let invalid_jpeg = [0x12, 0x34, 0x56, 0x78];
        let cursor = Cursor::new(invalid_jpeg);
        let result = scan_jpeg_segments(cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_jpeg_segments_minimal() {
        // Minimal JPEG: SOI + EOI
        let minimal_jpeg = [0xFF, 0xD8, 0xFF, 0xD9];
        let cursor = Cursor::new(minimal_jpeg);
        let result = scan_jpeg_segments(cursor).unwrap();
        assert!(result.is_none()); // No EXIF data
    }

    #[test]
    fn test_scan_jpeg_segments_with_app1_exif() {
        // JPEG with APP1 segment containing EXIF
        let jpeg_data = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xE1, // APP1 marker
            0x00, 0x10, // Segment length (16 bytes)
            0x45, 0x78, 0x69, 0x66, 0x00, 0x00, // "Exif\0\0"
            0x49, 0x49, 0x2A, 0x00, // TIFF header (minimal)
            0x08, 0x00, 0x00, 0x00, // IFD offset
            0xFF, 0xD9, // EOI
        ];

        let cursor = Cursor::new(&jpeg_data);
        let result = scan_jpeg_segments(cursor).unwrap();
        assert!(result.is_some());

        let segment_info = result.unwrap();
        assert!(segment_info.has_exif);
        assert_eq!(segment_info.offset, 12); // After SOI(2) + APP1 marker(2) + length(2) + "Exif\0\0"(6) = 12
        assert_eq!(segment_info.length, 8); // 16 - 8 = 8 bytes of TIFF data
    }
}
