//! JPEG parsing to extract EXIF data

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/JPEG.pm"]

use crate::error::{Error, Result};
use std::io::{Read, Seek, SeekFrom};

/// JPEG segment markers
const MARKER_SOI: u8 = 0xD8; // Start of Image
const MARKER_APP1: u8 = 0xE1; // APP1 segment (contains EXIF/XMP)
const MARKER_APP2: u8 = 0xE2; // APP2 segment (contains MPF/FlashPix)
const MARKER_APP6: u8 = 0xE6; // APP6 segment (contains GPMF for GoPro)
                              // const MARKER_APP13: u8 = 0xED; // APP13 segment (contains IPTC/Photoshop) - Reserved for future use
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
    /// EXIF segment if found
    pub exif: Option<ExifSegment>,
    /// XMP segments if found (can be multiple for extended XMP)
    pub xmp: Vec<XmpSegment>,
    /// MPF segment if found (Multi-Picture Format)
    pub mpf: Option<MpfSegment>,
    /// GPMF segments if found (GoPro metadata, can be multiple)
    pub gpmf: Vec<GpmfSegment>,
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
    };
    // Check JPEG SOI marker
    let mut marker = [0u8; 2];
    reader.read_exact(&mut marker)?;
    if marker != [0xFF, MARKER_SOI] {
        return Err(Error::InvalidJpeg("Not a JPEG file".into()));
    }

    const XMP_SIGNATURE: &[u8] = b"http://ns.adobe.com/xap/1.0/\0";
    const XMP_EXTENSION_SIGNATURE: &[u8] = b"http://ns.adobe.com/xmp/extension/\0";
    const MPF_SIGNATURE: &[u8] = b"MPF\0";
    const GPMF_SIGNATURE: &[u8] = b"GoPro";

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

                if marker == MARKER_APP1 {
                    // This might be our EXIF segment
                    if data_len > MAX_APP_SIZE {
                        return Err(Error::InvalidJpeg("APP1 segment too large".into()));
                    }

                    // Read the segment data
                    let mut data = vec![0u8; data_len];
                    reader.read_exact(&mut data)?;

                    // Check for EXIF signature
                    if data.len() >= 6 && &data[0..6] == b"Exif\0\0" {
                        // Found EXIF data!
                        let offset = reader.stream_position()? - data_len as u64;
                        metadata.exif = Some(ExifSegment {
                            data: data[6..].to_vec(), // Skip "Exif\0\0" header
                            offset: offset + 6,       // Offset to actual EXIF data
                        });
                    }
                    // Check for XMP signature
                    else if data.len() >= XMP_SIGNATURE.len()
                        && &data[0..XMP_SIGNATURE.len()] == XMP_SIGNATURE
                    {
                        // Found standard XMP data!
                        let offset = reader.stream_position()? - data_len as u64;
                        metadata.xmp.push(XmpSegment {
                            data: data[XMP_SIGNATURE.len()..].to_vec(),
                            offset: offset + XMP_SIGNATURE.len() as u64,
                            is_extended: false,
                        });
                    }
                    // Check for Extended XMP signature
                    else if data.len() >= XMP_EXTENSION_SIGNATURE.len()
                        && &data[0..XMP_EXTENSION_SIGNATURE.len()] == XMP_EXTENSION_SIGNATURE
                    {
                        // Found extended XMP data!
                        let offset = reader.stream_position()? - data_len as u64;
                        metadata.xmp.push(XmpSegment {
                            data: data[XMP_EXTENSION_SIGNATURE.len()..].to_vec(),
                            offset: offset + XMP_EXTENSION_SIGNATURE.len() as u64,
                            is_extended: true,
                        });
                    }
                    // Continue searching for more segments
                } else if marker == MARKER_APP2 {
                    // This might be our MPF segment
                    if data_len > MAX_APP_SIZE {
                        return Err(Error::InvalidJpeg("APP2 segment too large".into()));
                    }

                    // Read the segment data
                    let mut data = vec![0u8; data_len];
                    reader.read_exact(&mut data)?;

                    // Check for MPF signature
                    if data.len() >= MPF_SIGNATURE.len()
                        && &data[0..MPF_SIGNATURE.len()] == MPF_SIGNATURE
                    {
                        // Found MPF data!
                        let offset = reader.stream_position()? - data_len as u64;
                        metadata.mpf = Some(MpfSegment {
                            data: data[MPF_SIGNATURE.len()..].to_vec(), // Skip "MPF\0" header
                            offset: offset + MPF_SIGNATURE.len() as u64, // Offset to actual MPF data
                        });
                    }
                    // Continue searching for more segments
                } else if marker == MARKER_APP6 {
                    // This might be our GPMF segment (GoPro metadata)
                    if data_len > MAX_APP_SIZE {
                        return Err(Error::InvalidJpeg("APP6 segment too large".into()));
                    }

                    // Read the segment data
                    let mut data = vec![0u8; data_len];
                    reader.read_exact(&mut data)?;

                    // Check for GoPro GPMF signature
                    if data.len() >= GPMF_SIGNATURE.len()
                        && &data[0..GPMF_SIGNATURE.len()] == GPMF_SIGNATURE
                    {
                        // Found GPMF data!
                        let offset = reader.stream_position()? - data_len as u64;
                        metadata.gpmf.push(GpmfSegment {
                            data: data[GPMF_SIGNATURE.len()..].to_vec(), // Skip "GoPro" header
                            offset: offset + GPMF_SIGNATURE.len() as u64, // Offset to actual GPMF data
                        });
                    }
                    // Continue searching for more segments
                } else {
                    // Not APP1, APP2, or APP6, skip this segment
                    reader.seek(SeekFrom::Current(data_len as i64))?;
                }
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
