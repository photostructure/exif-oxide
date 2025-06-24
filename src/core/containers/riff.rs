//! RIFF container format parsing for metadata extraction
//!
//! RIFF (Resource Interchange File Format) is used by:
//! - AVI (Audio Video Interleave) files
//! - WebP image files
//! - WAV audio files

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/RIFF.pm"]

use crate::error::Result;
use std::io::{Read, Seek, SeekFrom};

/// RIFF file signature
const RIFF_SIGNATURE: [u8; 4] = [b'R', b'I', b'F', b'F'];

/// WebP file format identifier
const WEBP_FORMAT: [u8; 4] = [b'W', b'E', b'B', b'P'];

/// AVI file format identifier  
const AVI_FORMAT: [u8; 4] = [b'A', b'V', b'I', b' '];

/// EXIF chunk identifier in WebP
const EXIF_CHUNK: [u8; 4] = [b'E', b'X', b'I', b'F'];

/// XMP chunk identifier in WebP
const XMP_CHUNK: [u8; 4] = [b'X', b'M', b'P', b' '];

/// Result of finding metadata in a RIFF container
#[derive(Debug)]
pub struct RiffMetadataSegment {
    /// The raw metadata (EXIF or XMP data)
    pub data: Vec<u8>,
    /// Offset in the file where the metadata starts
    pub offset: u64,
    /// Type of metadata found
    pub metadata_type: MetadataType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetadataType {
    Exif,
    Xmp,
}

/// RIFF chunk header
#[derive(Debug)]
struct ChunkHeader {
    fourcc: [u8; 4],
    size: u32,
}

/// Find and extract metadata from a RIFF container file
pub fn find_metadata<R: Read + Seek>(reader: &mut R) -> Result<Option<RiffMetadataSegment>> {
    reader.seek(SeekFrom::Start(0))?;

    // Read and verify RIFF header
    let mut signature = [0u8; 4];
    reader.read_exact(&mut signature)?;

    if signature != RIFF_SIGNATURE {
        return Ok(None); // Not a RIFF file
    }

    // Read file size (excluding RIFF header)
    let mut size_bytes = [0u8; 4];
    reader.read_exact(&mut size_bytes)?;
    let _file_size = u32::from_le_bytes(size_bytes);

    // Read format type
    let mut format = [0u8; 4];
    reader.read_exact(&mut format)?;

    match &format {
        &WEBP_FORMAT => find_webp_metadata(reader),
        &AVI_FORMAT => find_avi_metadata(reader),
        _ => Ok(None), // Unsupported RIFF format
    }
}

/// Find metadata in a WebP file
fn find_webp_metadata<R: Read + Seek>(reader: &mut R) -> Result<Option<RiffMetadataSegment>> {
    // WebP stores EXIF and XMP data in specific chunks after the VP8/VP8L/VP8X chunk

    loop {
        let chunk_start = reader.stream_position()?;

        // Try to read chunk header
        let header = match read_chunk_header(reader) {
            Ok(h) => h,
            Err(_) => break, // End of file or read error
        };

        match &header.fourcc {
            &EXIF_CHUNK => {
                // Found EXIF chunk
                let data_offset = reader.stream_position()?;

                // EXIF data in WebP starts with 4 padding bytes, then TIFF header
                let mut padding = [0u8; 4];
                reader.read_exact(&mut padding)?;

                // Read the actual EXIF data (minus padding)
                let exif_size = (header.size - 4) as usize;
                let mut exif_data = vec![0u8; exif_size];
                reader.read_exact(&mut exif_data)?;

                // Validate TIFF header
                if exif_data.len() >= 4 {
                    let tiff_header = &exif_data[0..4];
                    if tiff_header == [0x49, 0x49, 0x2a, 0x00] || // Little endian
                       tiff_header == [0x4d, 0x4d, 0x00, 0x2a]
                    // Big endian
                    {
                        return Ok(Some(RiffMetadataSegment {
                            data: exif_data,
                            offset: data_offset + 4, // After padding
                            metadata_type: MetadataType::Exif,
                        }));
                    }
                }
            }

            &XMP_CHUNK => {
                // Found XMP chunk
                let data_offset = reader.stream_position()?;

                // Read XMP data
                let mut xmp_data = vec![0u8; header.size as usize];
                reader.read_exact(&mut xmp_data)?;

                return Ok(Some(RiffMetadataSegment {
                    data: xmp_data,
                    offset: data_offset,
                    metadata_type: MetadataType::Xmp,
                }));
            }

            _ => {
                // Skip this chunk
                let skip_amount = header.size as i64;
                reader.seek(SeekFrom::Current(skip_amount))?;
            }
        }

        // Align to even byte boundary (RIFF requirement)
        if header.size % 2 == 1 {
            reader.seek(SeekFrom::Current(1))?;
        }

        // Sanity check - don't read beyond reasonable bounds
        if chunk_start > 100 * 1024 * 1024 {
            break;
        }
    }

    Ok(None)
}

/// Find metadata in an AVI file
fn find_avi_metadata<R: Read + Seek>(reader: &mut R) -> Result<Option<RiffMetadataSegment>> {
    // AVI files can store EXIF data in:
    // 1. INFO list chunks
    // 2. _PMX chunks (for XMP)
    // 3. exif chunks within strd chunks

    // For now, we'll implement basic INFO list parsing
    // Full AVI metadata support would require parsing the entire chunk hierarchy

    loop {
        let chunk_start = reader.stream_position()?;

        // Try to read chunk header
        let header = match read_chunk_header(reader) {
            Ok(h) => h,
            Err(_) => break,
        };

        if &header.fourcc == b"LIST" {
            // Read list type
            let mut list_type = [0u8; 4];
            reader.read_exact(&mut list_type)?;

            if &list_type == b"INFO" {
                // INFO lists contain metadata, but not EXIF
                // This is a placeholder for future expansion
            }
        } else if &header.fourcc == b"_PMX" {
            // PMX chunk contains XMP data
            let data_offset = reader.stream_position()?;
            let mut xmp_data = vec![0u8; header.size as usize];
            reader.read_exact(&mut xmp_data)?;

            return Ok(Some(RiffMetadataSegment {
                data: xmp_data,
                offset: data_offset,
                metadata_type: MetadataType::Xmp,
            }));
        } else {
            // Skip this chunk
            reader.seek(SeekFrom::Current(header.size as i64))?;
        }

        // Align to even byte boundary
        if header.size % 2 == 1 {
            reader.seek(SeekFrom::Current(1))?;
        }

        // Sanity check
        if chunk_start > 500 * 1024 * 1024 {
            break;
        }
    }

    Ok(None)
}

/// Read a RIFF chunk header
fn read_chunk_header<R: Read>(reader: &mut R) -> Result<ChunkHeader> {
    let mut fourcc = [0u8; 4];
    reader.read_exact(&mut fourcc)?;

    let mut size_bytes = [0u8; 4];
    reader.read_exact(&mut size_bytes)?;
    let size = u32::from_le_bytes(size_bytes);

    Ok(ChunkHeader { fourcc, size })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_riff_signature_detection() {
        let riff_data = [
            b'R', b'I', b'F', b'F', // RIFF signature
            0x00, 0x00, 0x00, 0x00, // File size
            b'W', b'E', b'B', b'P', // WebP format
        ];

        let mut cursor = Cursor::new(&riff_data);
        let mut signature = [0u8; 4];
        cursor.read_exact(&mut signature).unwrap();

        assert_eq!(signature, RIFF_SIGNATURE);
    }

    #[test]
    fn test_chunk_header_parsing() {
        let chunk_data = [
            b'E', b'X', b'I', b'F', // EXIF chunk
            0x10, 0x00, 0x00, 0x00, // Size: 16 bytes
        ];

        let mut cursor = Cursor::new(&chunk_data);
        let header = read_chunk_header(&mut cursor).unwrap();

        assert_eq!(header.fourcc, EXIF_CHUNK);
        assert_eq!(header.size, 16);
    }

    #[test]
    fn test_non_riff_file() {
        let jpeg_data = [0xFF, 0xD8, 0xFF, 0xE0]; // JPEG SOI

        let mut cursor = Cursor::new(&jpeg_data);
        let result = find_metadata(&mut cursor).unwrap();

        assert!(result.is_none());
    }
}
