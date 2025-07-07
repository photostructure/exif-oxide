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
    pub has_xmp: bool,
}

/// Scan JPEG file for all APP1 segments containing EXIF or XMP data
///
/// Returns information about the first APP1 segment found, prioritizing EXIF over XMP.
/// This scans all APP1 segments to handle files with multiple APP1 segments (both EXIF and XMP).
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
    let mut found_exif: Option<JpegSegmentInfo> = None;
    let mut found_xmp: Option<JpegSegmentInfo> = None;

    loop {
        // Read segment marker
        let mut marker_bytes = [0u8; 2];
        if reader.read_exact(&mut marker_bytes).is_err() {
            // End of file reached
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
                    // APP1 segment - check for EXIF or XMP
                    let segment_start = current_pos; // Start of segment data

                    // Try EXIF first (6 bytes: "Exif\0\0")
                    let mut exif_header = [0u8; 6];
                    if reader.read_exact(&mut exif_header).is_ok()
                        && &exif_header[0..4] == b"Exif"
                        && exif_header[4] == 0
                        && exif_header[5] == 0
                    {
                        // Found EXIF - store it and continue scanning
                        found_exif = Some(JpegSegmentInfo {
                            segment_type: segment,
                            offset: current_pos + 6, // After "Exif\0\0" (6 bytes)
                            length: length - 8, // Subtract segment length header (2 bytes) + "Exif\0\0" (6 bytes) = 8 total
                            has_exif: true,
                            has_xmp: false,
                        });
                    } else {
                        // Reset and try XMP (29 bytes: "http://ns.adobe.com/xap/1.0/\0")
                        reader.seek(SeekFrom::Start(segment_start))?;
                        let mut xmp_header = [0u8; 29];
                        if reader.read_exact(&mut xmp_header).is_ok()
                            && &xmp_header == b"http://ns.adobe.com/xap/1.0/\0"
                        {
                            // Found XMP - store it and continue scanning
                            found_xmp = Some(JpegSegmentInfo {
                                segment_type: segment,
                                offset: current_pos + 29, // After XMP identifier(29)
                                length: length - 31, // Subtract segment length header(2) + XMP identifier(29)
                                has_exif: false,
                                has_xmp: true,
                            });
                        }
                    }

                    // Reset to start of segment data for skipping
                    reader.seek(SeekFrom::Start(segment_start))?;
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

    // Prioritize EXIF over XMP (following ExifTool behavior)
    Ok(found_exif.or(found_xmp))
}

/// Scan JPEG file for all XMP segments
///
/// Returns all APP1 segments containing XMP data. For regular XMP, there's usually
/// just one segment. For Extended XMP, there may be multiple segments that need reassembly.
pub fn scan_jpeg_xmp_segments<R: Read + Seek>(mut reader: R) -> Result<Vec<JpegSegmentInfo>> {
    // Verify JPEG magic bytes
    let mut magic = [0u8; 2];
    reader.read_exact(&mut magic)?;
    if magic != [0xFF, 0xD8] {
        return Err(ExifError::InvalidFormat(
            "Not a valid JPEG file (missing 0xFFD8 magic bytes)".to_string(),
        ));
    }

    let mut xmp_segments = Vec::new();
    let mut current_pos = 2u64; // After SOI marker

    loop {
        // Read segment marker
        let mut marker_bytes = [0u8; 2];
        if reader.read_exact(&mut marker_bytes).is_err() {
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
            JpegSegment::Soi => continue,
            JpegSegment::Eoi | JpegSegment::Sos => break,
            JpegSegment::App(1) => {
                // Read segment length
                let mut length_bytes = [0u8; 2];
                reader.read_exact(&mut length_bytes)?;
                let length = u16::from_be_bytes(length_bytes);
                current_pos += 2;

                let segment_start = current_pos;

                // Check for regular XMP identifier
                let mut xmp_header = [0u8; 29];
                if reader.read_exact(&mut xmp_header).is_ok()
                    && &xmp_header == b"http://ns.adobe.com/xap/1.0/\0"
                {
                    xmp_segments.push(JpegSegmentInfo {
                        segment_type: segment,
                        offset: current_pos + 29,
                        length: length - 31, // Subtract length header + identifier
                        has_exif: false,
                        has_xmp: true,
                    });

                    // Skip to next segment
                    let remaining = (length - 31) as u64;
                    reader.seek(SeekFrom::Current(remaining as i64))?;
                    current_pos += length as u64;
                    continue;
                }

                // Reset and check for Extended XMP identifier
                reader.seek(SeekFrom::Start(segment_start))?;
                let mut ext_xmp_header = [0u8; 35];
                if reader.read_exact(&mut ext_xmp_header).is_ok()
                    && &ext_xmp_header[0..35] == b"http://ns.adobe.com/xmp/extension/\0"
                {
                    // Extended XMP segment - store for later reassembly
                    xmp_segments.push(JpegSegmentInfo {
                        segment_type: segment,
                        offset: current_pos + 35,
                        length: length - 37, // Subtract length header + identifier
                        has_exif: false,
                        has_xmp: true,
                    });

                    // Skip to next segment
                    let remaining = (length - 37) as u64;
                    reader.seek(SeekFrom::Current(remaining as i64))?;
                    current_pos += length as u64;
                    continue;
                }

                // Not XMP - skip this APP1 segment
                reader.seek(SeekFrom::Start(segment_start))?;
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

    Ok(xmp_segments)
}

/// Extract XMP data from JPEG file
///
/// This function scans for APP1 segments containing XMP data and returns
/// the raw XMP packet(s). For Extended XMP, multiple segments are reassembled.
pub fn extract_jpeg_xmp<R: Read + Seek>(mut reader: R) -> Result<Vec<u8>> {
    let xmp_segments = scan_jpeg_xmp_segments(&mut reader)?;

    if xmp_segments.is_empty() {
        return Err(ExifError::InvalidFormat(
            "No XMP data found in JPEG file".to_string(),
        ));
    }

    // For now, just handle the first XMP segment (regular XMP)
    // TODO: Handle Extended XMP reassembly in future implementation
    let first_segment = &xmp_segments[0];

    reader.seek(SeekFrom::Start(first_segment.offset))?;
    let mut xmp_data = vec![0u8; first_segment.length as usize];
    reader.read_exact(&mut xmp_data)?;

    Ok(xmp_data)
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
        assert!(!segment_info.has_xmp);
        assert_eq!(segment_info.offset, 12); // After SOI(2) + APP1 marker(2) + length(2) + "Exif\0\0"(6) = 12
        assert_eq!(segment_info.length, 8); // 16 - 8 = 8 bytes of TIFF data
    }

    #[test]
    fn test_scan_jpeg_segments_with_app1_xmp() {
        // JPEG with APP1 segment containing XMP
        let xmp_identifier = b"http://ns.adobe.com/xap/1.0/\0"; // 29 bytes
        let xmp_packet = b"<?xml?><x:xmpmeta></x:xmpmeta>"; // 30 bytes
        let segment_length = 2 + xmp_identifier.len() + xmp_packet.len(); // length field (2) + identifier + packet

        let mut jpeg_data = vec![
            0xFF,
            0xD8, // SOI
            0xFF,
            0xE1, // APP1 marker
            (segment_length >> 8) as u8,
            (segment_length & 0xFF) as u8, // Segment length
        ];

        // XMP identifier and packet
        jpeg_data.extend_from_slice(xmp_identifier);
        jpeg_data.extend_from_slice(xmp_packet);

        // EOI
        jpeg_data.extend_from_slice(&[0xFF, 0xD9]);

        let cursor = Cursor::new(&jpeg_data);
        let result = scan_jpeg_segments(cursor).unwrap();
        assert!(result.is_some());

        let segment_info = result.unwrap();
        assert!(!segment_info.has_exif);
        assert!(segment_info.has_xmp);
        // Offset should be after SOI(2) + APP1 marker(2) + length(2) + XMP identifier(29) = 35
        assert_eq!(segment_info.offset, 35);
        // Length should be segment_length - length_field(2) - identifier(29) = 30
        assert_eq!(segment_info.length, 30);
    }

    #[test]
    fn test_scan_jpeg_xmp_segments() {
        // JPEG with XMP segment
        let xmp_identifier = b"http://ns.adobe.com/xap/1.0/\0"; // 29 bytes
        let xmp_packet = b"<?xml?><x:xmpmeta></x:xmpmeta>"; // 30 bytes
        let segment_length = 2 + xmp_identifier.len() + xmp_packet.len(); // 2 + 29 + 30 = 61

        let mut jpeg_data = vec![
            0xFF,
            0xD8, // SOI
            0xFF,
            0xE1, // APP1 marker
            (segment_length >> 8) as u8,
            (segment_length & 0xFF) as u8, // Segment length
        ];

        // XMP identifier and packet
        jpeg_data.extend_from_slice(xmp_identifier);
        jpeg_data.extend_from_slice(xmp_packet);

        // EOI
        jpeg_data.extend_from_slice(&[0xFF, 0xD9]);

        let cursor = Cursor::new(&jpeg_data);
        let result = scan_jpeg_xmp_segments(cursor).unwrap();
        assert_eq!(result.len(), 1);

        let segment_info = &result[0];
        assert!(segment_info.has_xmp);
        assert_eq!(segment_info.length, 30); // Just the XMP packet size
    }

    #[test]
    fn test_extract_jpeg_xmp() {
        // JPEG with XMP segment
        let xmp_identifier = b"http://ns.adobe.com/xap/1.0/\0"; // 29 bytes
        let xmp_packet = b"<?xml?><x:xmpmeta></x:xmpmeta>"; // 30 bytes
        let segment_length = 2 + xmp_identifier.len() + xmp_packet.len(); // 2 + 29 + 30 = 61

        let mut jpeg_data = vec![
            0xFF,
            0xD8, // SOI
            0xFF,
            0xE1, // APP1 marker
            (segment_length >> 8) as u8,
            (segment_length & 0xFF) as u8, // Segment length
        ];

        // XMP identifier and packet
        jpeg_data.extend_from_slice(xmp_identifier);
        jpeg_data.extend_from_slice(xmp_packet);

        // EOI
        jpeg_data.extend_from_slice(&[0xFF, 0xD9]);

        let cursor = Cursor::new(&jpeg_data);
        let result = extract_jpeg_xmp(cursor);
        assert!(result.is_ok());

        let xmp_data = result.unwrap();
        assert_eq!(xmp_data, xmp_packet);
    }
}
