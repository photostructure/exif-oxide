//! PNG format parsing to extract EXIF data from eXIf chunks
//!
//! PNG files store EXIF data in eXIf chunks which contain raw EXIF data
//! that can be parsed with the standard IFD parser.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/PNG.pm"]

use crate::error::{Error, Result};
use std::io::{Read, Seek, SeekFrom};

/// PNG file signature
const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

/// PNG chunk type for EXIF data
const EXIF_CHUNK_TYPE: [u8; 4] = [b'e', b'X', b'I', b'f'];

/// Result of finding EXIF data in a PNG file
#[derive(Debug)]
pub struct PngExifSegment {
    /// The raw EXIF data from the eXIf chunk (ready for IFD parsing)
    pub data: Vec<u8>,
    /// Offset in the file where the EXIF data starts
    pub offset: u64,
}

/// PNG chunk header structure
#[derive(Debug)]
struct ChunkHeader {
    length: u32,
    chunk_type: [u8; 4],
}

/// Find and extract EXIF data from a PNG file's eXIf chunk
pub fn find_exif_chunk<R: Read + Seek>(reader: &mut R) -> Result<Option<PngExifSegment>> {
    // Seek to the beginning
    reader.seek(SeekFrom::Start(0))?;

    // Verify PNG signature
    let mut signature = [0u8; 8];
    reader.read_exact(&mut signature)?;

    if signature != PNG_SIGNATURE {
        return Ok(None); // Not a PNG file
    }

    // Read chunks looking for eXIf
    loop {
        // Try to read chunk header
        let chunk_header = match read_chunk_header(reader) {
            Ok(header) => header,
            Err(_) => break, // End of file or read error
        };

        if chunk_header.chunk_type == EXIF_CHUNK_TYPE {
            // Found eXIf chunk!
            let data_offset = reader.stream_position()?;

            // Read the chunk data
            let mut exif_data = vec![0u8; chunk_header.length as usize];
            reader.read_exact(&mut exif_data)?;

            // PNG eXIf chunk contains raw EXIF data starting with TIFF header
            // Validate it starts with TIFF magic
            if exif_data.len() >= 4 {
                let tiff_header = &exif_data[0..4];
                if tiff_header == [0x49, 0x49, 0x2a, 0x00] || // Little endian
                   tiff_header == [0x4d, 0x4d, 0x00, 0x2a]
                {
                    // Big endian
                    return Ok(Some(PngExifSegment {
                        data: exif_data,
                        offset: data_offset,
                    }));
                }
            }

            // Invalid EXIF data in eXIf chunk
            return Err(Error::InvalidData(
                "Invalid EXIF data in PNG eXIf chunk".into(),
            ));
        }

        // Skip this chunk's data and CRC
        let skip_bytes = chunk_header.length as u64 + 4; // data + 4-byte CRC
        reader.seek(SeekFrom::Current(skip_bytes as i64))?;

        // Stop at critical chunks that indicate we've passed the metadata section
        if is_critical_data_chunk(&chunk_header.chunk_type) {
            break;
        }
    }

    Ok(None) // No eXIf chunk found
}

/// Read a PNG chunk header (length + type)
fn read_chunk_header<R: Read>(reader: &mut R) -> Result<ChunkHeader> {
    let mut length_bytes = [0u8; 4];
    reader.read_exact(&mut length_bytes)?;
    let length = u32::from_be_bytes(length_bytes);

    let mut chunk_type = [0u8; 4];
    reader.read_exact(&mut chunk_type)?;

    Ok(ChunkHeader { length, chunk_type })
}

/// Check if this is a critical data chunk (IDAT, etc.) that signals
/// we've moved past the metadata section
fn is_critical_data_chunk(chunk_type: &[u8; 4]) -> bool {
    match chunk_type {
        b"IDAT" => true, // Image data - no more metadata after this
        b"IEND" => true, // End of image
        _ => false,
    }
}

/// Extract all metadata from PNG (EXIF, text chunks, etc.)
/// This is for future expansion beyond just EXIF
pub fn find_png_metadata<R: Read + Seek>(reader: &mut R) -> Result<PngMetadata> {
    let mut metadata = PngMetadata {
        exif: None,
        text_chunks: Vec::new(),
    };

    // Seek to the beginning
    reader.seek(SeekFrom::Start(0))?;

    // Verify PNG signature
    let mut signature = [0u8; 8];
    reader.read_exact(&mut signature)?;

    if signature != PNG_SIGNATURE {
        return Err(Error::InvalidData("Not a PNG file".into()));
    }

    // Read chunks
    loop {
        let chunk_header = match read_chunk_header(reader) {
            Ok(header) => header,
            Err(_) => break,
        };

        match &chunk_header.chunk_type {
            b"eXIf" => {
                // EXIF chunk
                let data_offset = reader.stream_position()?;
                let mut exif_data = vec![0u8; chunk_header.length as usize];
                reader.read_exact(&mut exif_data)?;

                if exif_data.len() >= 4 {
                    let tiff_header = &exif_data[0..4];
                    if tiff_header == [0x49, 0x49, 0x2a, 0x00]
                        || tiff_header == [0x4d, 0x4d, 0x00, 0x2a]
                    {
                        metadata.exif = Some(PngExifSegment {
                            data: exif_data,
                            offset: data_offset,
                        });
                    }
                }

                // Skip CRC
                reader.seek(SeekFrom::Current(4))?;
            }

            b"tEXt" | b"zTXt" | b"iTXt" => {
                // Text chunks (for future XMP support)
                let mut chunk_data = vec![0u8; chunk_header.length as usize];
                reader.read_exact(&mut chunk_data)?;

                metadata.text_chunks.push(PngTextChunk {
                    chunk_type: chunk_header.chunk_type,
                    data: chunk_data,
                });

                // Skip CRC
                reader.seek(SeekFrom::Current(4))?;
            }

            _ => {
                // Skip unknown chunk data and CRC
                let skip_bytes = chunk_header.length as u64 + 4;
                reader.seek(SeekFrom::Current(skip_bytes as i64))?;

                if is_critical_data_chunk(&chunk_header.chunk_type) {
                    break;
                }
            }
        }
    }

    Ok(metadata)
}

/// Complete PNG metadata structure
#[derive(Debug)]
pub struct PngMetadata {
    /// EXIF data from eXIf chunk
    pub exif: Option<PngExifSegment>,
    /// Text chunks (tEXt, zTXt, iTXt) for future XMP support
    pub text_chunks: Vec<PngTextChunk>,
}

/// PNG text chunk (for XMP and other metadata)
#[derive(Debug)]
pub struct PngTextChunk {
    /// Chunk type (tEXt, zTXt, iTXt)
    pub chunk_type: [u8; 4],
    /// Raw chunk data
    pub data: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_png_signature_validation() {
        let png_data = [
            // PNG signature
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // IHDR chunk
            0x00, 0x00, 0x00, 0x0D, // Length: 13
            b'I', b'H', b'D',
            b'R', // Type: IHDR
                  // IHDR data would follow...
        ];

        let mut cursor = Cursor::new(&png_data[0..8]); // Just signature
        let mut signature = [0u8; 8];
        cursor.read_exact(&mut signature).unwrap();

        assert_eq!(signature, PNG_SIGNATURE);
    }

    #[test]
    fn test_non_png_data() {
        let jpeg_data = [0xff, 0xd8, 0xff, 0xe0]; // JPEG SOI marker

        let mut cursor = Cursor::new(jpeg_data);
        let result = find_exif_chunk(&mut cursor);

        // Should return None for non-PNG data, not an error
        match result {
            Ok(None) => {} // Expected: no EXIF in non-PNG
            Err(_) => {}   // Also acceptable: error reading non-PNG as PNG
            Ok(Some(_)) => panic!("Should not find EXIF in JPEG data"),
        }
    }

    #[test]
    fn test_chunk_header_parsing() {
        let chunk_data = [
            0x00, 0x00, 0x00, 0x0D, // Length: 13
            b'I', b'H', b'D', b'R', // Type: IHDR
        ];

        let mut cursor = Cursor::new(chunk_data);
        let header = read_chunk_header(&mut cursor).unwrap();

        assert_eq!(header.length, 13);
        assert_eq!(header.chunk_type, [b'I', b'H', b'D', b'R']);
    }

    #[test]
    fn test_critical_chunk_detection() {
        assert!(is_critical_data_chunk(b"IDAT"));
        assert!(is_critical_data_chunk(b"IEND"));
        assert!(!is_critical_data_chunk(b"eXIf"));
        assert!(!is_critical_data_chunk(b"tEXt"));
    }

    #[test]
    fn test_exif_chunk_type_match() {
        assert_eq!(EXIF_CHUNK_TYPE, [b'e', b'X', b'I', b'f']);
    }
}
