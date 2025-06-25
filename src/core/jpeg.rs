//! JPEG parsing to extract EXIF data

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/JPEG.pm"]

use crate::error::{Error, Result};
use crate::tables::app_segments::{identify_app_segment, AppSegmentRule, FormatHandler};
use std::io::{Read, Seek, SeekFrom};

/// JPEG segment markers
const MARKER_SOI: u8 = 0xD8; // Start of Image
const MARKER_SOS: u8 = 0xDA; // Start of Scan (image data follows)
const MARKER_EOI: u8 = 0xD9; // End of Image

/// Maximum size of APP segments (64KB minus 2 bytes for length)
const MAX_APP_SIZE: usize = 65533;

/// Result of finding EXIF data in a JPEG
#[derive(Debug)]
pub struct ExifSegment {
    /// The raw EXIF data (without the APP1 header or "Exif\0\0" signature)
    pub data: Vec<u8>,
    /// Offset in the file where the EXIF data starts
    pub offset: u64,
}

/// Result of finding metadata segments in a JPEG
#[derive(Debug)]
pub struct JpegMetadata {
    /// EXIF segment if found (backward compatibility)
    pub exif: Option<ExifSegment>,
    /// XMP segments if found (can be multiple for extended XMP) (backward compatibility)
    pub xmp: Vec<XmpSegment>,
    /// MPF segment if found (Multi-Picture Format) (backward compatibility)
    pub mpf: Option<MpfSegment>,
    /// GPMF segments if found (GoPro metadata, can be multiple) (backward compatibility)
    pub gpmf: Vec<GpmfSegment>,
    /// Comprehensive APP segments with table-driven identification
    pub app_segments: Vec<AppSegment>,
}

/// XMP segment found in JPEG
#[derive(Debug)]
pub struct XmpSegment {
    /// The raw XMP data (without the APP1 header or XMP signature)
    pub data: Vec<u8>,
    /// Offset in the file where the XMP data starts
    pub offset: u64,
    /// Whether this is an extended XMP segment
    pub is_extended: bool,
}

/// MPF segment found in JPEG APP2
#[derive(Debug)]
pub struct MpfSegment {
    /// The raw MPF data (without the APP2 header or "MPF\0" signature)
    pub data: Vec<u8>,
    /// Offset in the file where the MPF data starts
    pub offset: u64,
}

/// GPMF segment found in JPEG APP6 (GoPro metadata)
#[derive(Debug)]
pub struct GpmfSegment {
    /// The raw GPMF data (without the APP6 header or "GoPro" signature)
    pub data: Vec<u8>,
    /// Offset in the file where the GPMF data starts
    pub offset: u64,
}

/// Comprehensive APP segment found in JPEG
#[derive(Debug)]
pub struct AppSegment {
    /// APP segment number (0-15)
    pub segment_number: u8,
    /// The raw segment data (without the APP header and length)
    pub data: Vec<u8>,
    /// Offset in the file where the segment data starts
    pub offset: u64,
    /// Identified format rule if recognized
    pub rule: Option<&'static AppSegmentRule>,
    /// The data without signature/header (for recognized formats)
    pub parsed_data: Option<Vec<u8>>,
}

/// Find and extract EXIF data from a JPEG file
pub fn find_exif_segment<R: Read + Seek>(reader: &mut R) -> Result<Option<ExifSegment>> {
    let metadata = find_metadata_segments(reader)?;
    Ok(metadata.exif)
}

/// Find and extract MPF data from a JPEG file
pub fn find_mpf_segment<R: Read + Seek>(reader: &mut R) -> Result<Option<MpfSegment>> {
    let metadata = find_metadata_segments(reader)?;
    Ok(metadata.mpf)
}

/// Find and extract all metadata segments (EXIF, XMP, MPF, and GPMF) from a JPEG file
pub fn find_metadata_segments<R: Read + Seek>(reader: &mut R) -> Result<JpegMetadata> {
    let mut metadata = JpegMetadata {
        exif: None,
        xmp: Vec::new(),
        mpf: None,
        gpmf: Vec::new(),
        app_segments: Vec::new(),
    };
    // Check JPEG SOI marker
    let mut marker = [0u8; 2];
    reader.read_exact(&mut marker)?;
    if marker != [0xFF, MARKER_SOI] {
        return Err(Error::InvalidJpeg("Not a JPEG file".into()));
    }

    // Scan through JPEG segments
    loop {
        // Read marker
        let mut marker_bytes = [0u8; 2];
        reader.read_exact(&mut marker_bytes)?;

        // JPEG markers must start with 0xFF
        if marker_bytes[0] != 0xFF {
            return Err(Error::InvalidJpeg("Invalid JPEG marker".into()));
        }

        let marker = marker_bytes[1];

        // Handle different marker types
        match marker {
            // These markers have no data (but exclude EOI which is 0xD9)
            0xD0..=0xD8 | 0x01 => continue,

            // End of Image
            MARKER_EOI => return Ok(metadata),

            // Start of Scan - no more metadata after this
            MARKER_SOS => return Ok(metadata),

            // All other markers have a length field
            _ => {
                // Read segment length (big-endian)
                let mut len_bytes = [0u8; 2];
                reader.read_exact(&mut len_bytes)?;
                let segment_len = u16::from_be_bytes(len_bytes) as usize;

                if segment_len < 2 {
                    return Err(Error::InvalidJpeg("Invalid segment length".into()));
                }

                // Length includes the 2 bytes we just read
                let data_len = segment_len - 2;

                // Check if this is an APP segment (0xE0-0xEF)
                if (0xE0..=0xEF).contains(&marker) {
                    // Calculate APP segment number (APP0 = 0xE0, APP1 = 0xE1, etc.)
                    let app_number = marker - 0xE0;

                    // Process APP segment with table-driven identification
                    if let Err(_e) =
                        process_app_segment(reader, &mut metadata, app_number, data_len)
                    {
                        // If APP segment processing fails, skip it but continue
                        reader.seek(SeekFrom::Current(data_len as i64))?;
                    }
                } else {
                    // Not an APP segment, skip it
                    reader.seek(SeekFrom::Current(data_len as i64))?;
                }
            }
        }
    }
}

/// Process an APP segment using table-driven identification
fn process_app_segment<R: Read + Seek>(
    reader: &mut R,
    metadata: &mut JpegMetadata,
    app_number: u8,
    data_len: usize,
) -> Result<()> {
    if data_len > MAX_APP_SIZE {
        return Err(Error::InvalidJpeg(format!(
            "APP{} segment too large",
            app_number
        )));
    }

    // Read the segment data
    let mut data = vec![0u8; data_len];
    reader.read_exact(&mut data)?;
    let offset = reader.stream_position()? - data_len as u64;

    // Try to identify the segment format using lookup tables
    let rule = identify_app_segment(app_number, &data);

    // Parse data based on identified format
    let parsed_data = if let Some(rule) = rule {
        parse_segment_data(rule, &data)
    } else {
        None
    };

    // Create comprehensive APP segment entry
    let app_segment = AppSegment {
        segment_number: app_number,
        data: data.clone(),
        offset,
        rule,
        parsed_data: parsed_data.clone(),
    };
    metadata.app_segments.push(app_segment);

    // Maintain backward compatibility by populating legacy fields
    if let Some(rule) = rule {
        match rule.format_handler {
            FormatHandler::EXIF => {
                if let Some(parsed) = parsed_data {
                    metadata.exif = Some(ExifSegment {
                        data: parsed,
                        offset: offset + 6, // EXIF signature is 6 bytes: "Exif\0\0"
                    });
                }
            }
            FormatHandler::XMP => {
                if let Some(parsed) = parsed_data {
                    metadata.xmp.push(XmpSegment {
                        data: parsed,
                        offset: offset + 29, // XMP signature is 29 bytes: "http://ns.adobe.com/xap/1.0/\0"
                        is_extended: false,
                    });
                }
            }
            FormatHandler::ExtendedXMP => {
                if let Some(parsed) = parsed_data {
                    metadata.xmp.push(XmpSegment {
                        data: parsed,
                        offset: offset + 35, // Extended XMP signature is 35 bytes
                        is_extended: true,
                    });
                }
            }
            FormatHandler::MPF => {
                if let Some(parsed) = parsed_data {
                    metadata.mpf = Some(MpfSegment {
                        data: parsed,
                        offset: offset + rule.signature.len() as u64,
                    });
                }
            }
            FormatHandler::GoPro => {
                if let Some(parsed) = parsed_data {
                    metadata.gpmf.push(GpmfSegment {
                        data: parsed,
                        offset: offset + rule.signature.len() as u64,
                    });
                }
            }
            _ => {
                // Other formats don't have legacy equivalents
            }
        }
    }

    Ok(())
}

/// Parse segment data based on identified format rule
fn parse_segment_data(rule: &AppSegmentRule, data: &[u8]) -> Option<Vec<u8>> {
    match rule.format_handler {
        FormatHandler::EXIF => {
            // EXIF signature is "Exif\0\0" (6 bytes), but table only has "Exif\0" (5 bytes)
            // We need to skip the full 6-byte signature
            if data.len() >= 6 && &data[0..6] == b"Exif\0\0" {
                Some(data[6..].to_vec())
            } else {
                None
            }
        }
        FormatHandler::XMP => {
            // XMP signature is "http://ns.adobe.com/xap/1.0/\0" (29 bytes), but table only has "http" (4 bytes)
            // We need to skip the full 29-byte signature
            const XMP_SIGNATURE: &[u8] = b"http://ns.adobe.com/xap/1.0/\0";
            if data.len() >= XMP_SIGNATURE.len() && &data[0..XMP_SIGNATURE.len()] == XMP_SIGNATURE {
                Some(data[XMP_SIGNATURE.len()..].to_vec())
            } else {
                None
            }
        }
        FormatHandler::ExtendedXMP => {
            // Extended XMP signature is "http://ns.adobe.com/xmp/extension/\0" (35 bytes)
            const XMP_EXTENSION_SIGNATURE: &[u8] = b"http://ns.adobe.com/xmp/extension/\0";
            if data.len() >= XMP_EXTENSION_SIGNATURE.len()
                && &data[0..XMP_EXTENSION_SIGNATURE.len()] == XMP_EXTENSION_SIGNATURE
            {
                Some(data[XMP_EXTENSION_SIGNATURE.len()..].to_vec())
            } else {
                None
            }
        }
        _ => {
            if !rule.signature.is_empty() && data.len() >= rule.signature.len() {
                // For most formats, remove the signature prefix
                Some(data[rule.signature.len()..].to_vec())
            } else {
                // For custom conditions or empty signatures, return the full data
                Some(data.to_vec())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_not_jpeg() {
        let data = b"PNG\x89PNG";
        let mut cursor = Cursor::new(data);
        let result = find_exif_segment(&mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_jpeg_no_exif() {
        // Minimal JPEG with just SOI and EOI
        let data = [0xFF, 0xD8, 0xFF, 0xD9];
        let mut cursor = Cursor::new(&data);
        let result = find_exif_segment(&mut cursor).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_jpeg_with_exif() {
        // JPEG with APP1 EXIF segment
        let mut data = vec![];
        data.extend_from_slice(&[0xFF, 0xD8]); // SOI
        data.extend_from_slice(&[0xFF, 0xE1]); // APP1 marker
        data.extend_from_slice(&[0x00, 0x0E]); // Length = 14 (2 + 6 + 6)
        data.extend_from_slice(b"Exif\0\0"); // EXIF signature
        data.extend_from_slice(b"IIMM\0*"); // Fake TIFF header (6 bytes)
        data.extend_from_slice(&[0xFF, 0xD9]); // EOI

        let mut cursor = Cursor::new(&data);
        let result = find_exif_segment(&mut cursor).unwrap();
        assert!(result.is_some());

        let segment = result.unwrap();
        assert_eq!(segment.data, b"IIMM\0*");
        assert_eq!(segment.offset, 12); // Position after APP1 header + "Exif\0\0"
    }

    #[test]
    fn test_jpeg_with_xmp() {
        // JPEG with APP1 XMP segment
        let xmp_sig = b"http://ns.adobe.com/xap/1.0/\0";
        let xmp_data = b"<x:xmpmeta>test</x:xmpmeta>";

        let mut data = vec![];
        data.extend_from_slice(&[0xFF, 0xD8]); // SOI
        data.extend_from_slice(&[0xFF, 0xE1]); // APP1 marker
        let length = (2 + xmp_sig.len() + xmp_data.len()) as u16;
        data.extend_from_slice(&length.to_be_bytes()); // Length
        data.extend_from_slice(xmp_sig); // XMP signature
        data.extend_from_slice(xmp_data); // XMP data
        data.extend_from_slice(&[0xFF, 0xD9]); // EOI

        let mut cursor = Cursor::new(&data);
        let metadata = find_metadata_segments(&mut cursor).unwrap();

        assert!(metadata.exif.is_none());
        assert_eq!(metadata.xmp.len(), 1);
        assert_eq!(metadata.xmp[0].data, xmp_data);
        assert!(!metadata.xmp[0].is_extended);
    }

    #[test]
    fn test_jpeg_with_both_exif_and_xmp() {
        let xmp_sig = b"http://ns.adobe.com/xap/1.0/\0";
        let xmp_data = b"<x:xmpmeta>test</x:xmpmeta>";

        let mut data = vec![];
        data.extend_from_slice(&[0xFF, 0xD8]); // SOI

        // EXIF segment
        data.extend_from_slice(&[0xFF, 0xE1]); // APP1 marker
        data.extend_from_slice(&[0x00, 0x0E]); // Length = 14
        data.extend_from_slice(b"Exif\0\0"); // EXIF signature
        data.extend_from_slice(b"IIMM\0*"); // Fake TIFF header

        // XMP segment
        data.extend_from_slice(&[0xFF, 0xE1]); // APP1 marker
        let length = (2 + xmp_sig.len() + xmp_data.len()) as u16;
        data.extend_from_slice(&length.to_be_bytes()); // Length
        data.extend_from_slice(xmp_sig); // XMP signature
        data.extend_from_slice(xmp_data); // XMP data

        data.extend_from_slice(&[0xFF, 0xD9]); // EOI

        let mut cursor = Cursor::new(&data);
        let metadata = find_metadata_segments(&mut cursor).unwrap();

        assert!(metadata.exif.is_some());
        assert_eq!(metadata.exif.unwrap().data, b"IIMM\0*");
        assert_eq!(metadata.xmp.len(), 1);
        assert_eq!(metadata.xmp[0].data, xmp_data);
    }
}
