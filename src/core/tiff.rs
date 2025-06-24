//! TIFF/RAW format parsing to extract IFD data
//!
//! TIFF-based formats (TIFF, CR2, NEF, ARW, etc.) store EXIF data directly in IFD structure.
//! Unlike JPEG, they don't use wrapper segments - the file starts with the TIFF header.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm ProcessTIFF"]

use crate::error::{Error, Result};
use std::io::{Read, Seek, SeekFrom};

/// TIFF magic numbers for endianness detection
const TIFF_LITTLE_ENDIAN: [u8; 4] = [0x49, 0x49, 0x2a, 0x00]; // "II*\0"
const TIFF_BIG_ENDIAN: [u8; 4] = [0x4d, 0x4d, 0x00, 0x2a]; // "MM\0*"

/// Result of finding IFD data in a TIFF file
#[derive(Debug)]
pub struct TiffSegment {
    /// The raw TIFF/IFD data starting from the TIFF header
    pub data: Vec<u8>,
    /// Offset in the file where the TIFF data starts (usually 0)
    pub offset: u64,
}

/// Options for TIFF parsing
#[derive(Debug, Clone, Copy)]
pub enum TiffParseMode {
    /// Read entire file (needed for binary extraction)
    FullFile,
    /// Only read IFD chain (memory-efficient for metadata only)
    MetadataOnly,
}

/// Find and extract IFD data from a TIFF-based file
///
/// TIFF-based files include:
/// - TIFF (.tif/.tiff)
/// - Canon CR2 (.cr2)
/// - Nikon NEF (.nef)
/// - Sony ARW (.arw)
/// - And many other RAW formats
pub fn find_ifd_data<R: Read + Seek>(reader: &mut R) -> Result<Option<TiffSegment>> {
    // Default to full file mode for compatibility
    find_ifd_data_with_mode(reader, TiffParseMode::FullFile)
}

/// Find and extract IFD data with specified parse mode
pub fn find_ifd_data_with_mode<R: Read + Seek>(
    reader: &mut R,
    mode: TiffParseMode,
) -> Result<Option<TiffSegment>> {
    // Seek to the beginning
    reader.seek(SeekFrom::Start(0))?;

    // Read TIFF header (first 4 bytes)
    let mut header = [0u8; 4];
    reader.read_exact(&mut header)?;

    // Verify this is a valid TIFF file
    if header != TIFF_LITTLE_ENDIAN && header != TIFF_BIG_ENDIAN {
        return Ok(None); // Not a TIFF file
    }

    match mode {
        TiffParseMode::FullFile => {
            // For full file mode, read everything (needed for binary extraction)
            reader.seek(SeekFrom::Start(0))?;
            let mut data = Vec::new();
            reader.read_to_end(&mut data)?;

            // Validate minimum size (TIFF header + IFD offset)
            if data.len() < 8 {
                return Err(Error::InvalidData("TIFF file too small".into()));
            }

            parse_tiff_data(data, 0)
        }
        TiffParseMode::MetadataOnly => {
            // For metadata-only mode, just read the IFD chain
            read_ifd_chain_optimized(reader, header)
        }
    }
}

/// Parse TIFF data that's already in memory
fn parse_tiff_data(data: Vec<u8>, offset: u64) -> Result<Option<TiffSegment>> {
    // Extract header for endianness check
    if data.len() < 8 {
        return Err(Error::InvalidData("TIFF data too small".into()));
    }

    let header = [data[0], data[1], data[2], data[3]];

    // Parse the IFD offset to ensure the file is valid
    let is_little_endian = header == TIFF_LITTLE_ENDIAN;
    let ifd_offset = if is_little_endian {
        u32::from_le_bytes([data[4], data[5], data[6], data[7]])
    } else {
        u32::from_be_bytes([data[4], data[5], data[6], data[7]])
    } as usize;

    // Validate IFD offset is within file bounds
    if ifd_offset >= data.len() {
        return Err(Error::InvalidData(
            "Invalid IFD offset in TIFF header".into(),
        ));
    }

    // Special handling for Canon CR2 files
    // CR2 files have "CR" marker at offset 8 after the standard TIFF header
    if data.len() >= 10 && &data[8..10] == b"CR" {
        // This is a Canon CR2 file - the IFD structure is valid
        // but we need to handle CR2-specific quirks in the IFD parser
    }

    Ok(Some(TiffSegment { data, offset }))
}

/// Memory-efficient IFD chain reading
fn read_ifd_chain_optimized<R: Read + Seek>(
    reader: &mut R,
    header: [u8; 4],
) -> Result<Option<TiffSegment>> {
    use crate::core::endian::Endian;

    const MAX_IFD_DEPTH: usize = 10;
    const MAX_IFD_SIZE: usize = 65536;

    // Determine endianness
    let endian = if header == TIFF_LITTLE_ENDIAN {
        Endian::Little
    } else {
        Endian::Big
    };

    // Read IFD offset from header
    let mut offset_bytes = [0u8; 4];
    reader.read_exact(&mut offset_bytes)?;
    let first_ifd_offset = endian.read_u32(&offset_bytes);

    // Build optimized data starting with header
    let mut optimized_data = Vec::with_capacity(MAX_IFD_SIZE);
    optimized_data.extend_from_slice(&header);
    optimized_data.extend_from_slice(&offset_bytes);

    // Read IFD chain
    let mut current_offset = first_ifd_offset;
    let mut depth = 0;

    while current_offset != 0 && depth < MAX_IFD_DEPTH {
        // Seek to IFD
        reader.seek(SeekFrom::Start(current_offset as u64))?;

        // Read entry count
        let mut count_bytes = [0u8; 2];
        reader.read_exact(&mut count_bytes)?;
        let entry_count = endian.read_u16(&count_bytes);

        // Sanity check
        if entry_count > 1000 {
            return Err(Error::InvalidData("Invalid IFD entry count".into()));
        }

        // Calculate IFD size
        let ifd_size = 2 + (entry_count as usize * 12) + 4;

        // Read entire IFD
        reader.seek(SeekFrom::Start(current_offset as u64))?;
        let mut ifd_data = vec![0u8; ifd_size];
        reader.read_exact(&mut ifd_data)?;

        // Get next IFD offset
        let next_offset_pos = 2 + (entry_count as usize * 12);
        let next_offset = endian.read_u32(&ifd_data[next_offset_pos..next_offset_pos + 4]);

        // Add IFD data to optimized buffer
        optimized_data.extend_from_slice(&ifd_data);

        current_offset = next_offset;
        depth += 1;
    }

    Ok(Some(TiffSegment {
        data: optimized_data,
        offset: 0,
    }))
}

/// Detect if a TIFF file is actually a specific RAW format
///
/// Many RAW formats use TIFF container but have specific signatures:
/// - Canon CR2: "CR" at offset 8
/// - Some formats: Check Make tag in IFD0
pub fn detect_raw_format<R: Read + Seek>(reader: &mut R) -> Result<Option<String>> {
    reader.seek(SeekFrom::Start(0))?;

    // Read enough to check for format-specific markers
    let mut buffer = [0u8; 16];
    let bytes_read = reader.read(&mut buffer)?;

    if bytes_read < 8 {
        return Ok(None);
    }

    // Check for TIFF header first
    if buffer[0..4] != TIFF_LITTLE_ENDIAN && buffer[0..4] != TIFF_BIG_ENDIAN {
        return Ok(None);
    }

    // Check for Canon CR2 marker
    if bytes_read >= 10 && &buffer[8..10] == b"CR" {
        return Ok(Some("CR2".to_string()));
    }

    // For other formats, we would need to parse the IFD to check the Make tag
    // This is left for future implementation when we need more specific detection

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_tiff_little_endian_detection() {
        let tiff_data = [
            0x49, 0x49, 0x2a, 0x00, // TIFF little-endian header
            0x08, 0x00, 0x00, 0x00, // IFD offset = 8
            0x00, 0x00, // IFD entry count = 0 (minimal valid IFD)
            0x00, 0x00, 0x00, 0x00, // Next IFD offset = 0 (no next IFD)
        ];

        let mut cursor = Cursor::new(tiff_data);
        let result = find_ifd_data(&mut cursor).unwrap();

        assert!(result.is_some());
        let segment = result.unwrap();
        assert_eq!(segment.offset, 0);
        assert_eq!(segment.data.len(), tiff_data.len());
    }

    #[test]
    fn test_tiff_big_endian_detection() {
        let tiff_data = [
            0x4d, 0x4d, 0x00, 0x2a, // TIFF big-endian header
            0x00, 0x00, 0x00, 0x08, // IFD offset = 8
            0x00, 0x00, // IFD entry count = 0 (minimal valid IFD)
            0x00, 0x00, 0x00, 0x00, // Next IFD offset = 0 (no next IFD)
        ];

        let mut cursor = Cursor::new(tiff_data);
        let result = find_ifd_data(&mut cursor).unwrap();

        assert!(result.is_some());
        let segment = result.unwrap();
        assert_eq!(segment.offset, 0);
        assert_eq!(segment.data.len(), tiff_data.len());
    }

    #[test]
    fn test_non_tiff_data() {
        let jpeg_data = [0xff, 0xd8, 0xff, 0xe0]; // JPEG SOI marker

        let mut cursor = Cursor::new(jpeg_data);
        let result = find_ifd_data(&mut cursor).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_cr2_detection() {
        let cr2_data = [
            0x49, 0x49, 0x2a, 0x00, // TIFF little-endian header
            0x10, 0x00, 0x00, 0x00, // IFD offset = 16
            b'C', b'R', // CR2 marker
            0x02, 0x00, 0x00,
            0x00, // CR2 version
                  // More data would follow...
        ];

        let mut cursor = Cursor::new(cr2_data);
        let format = detect_raw_format(&mut cursor).unwrap();

        assert_eq!(format, Some("CR2".to_string()));
    }

    #[test]
    fn test_invalid_tiff_too_small() {
        let too_small = [0x49, 0x49, 0x2a]; // Incomplete TIFF header

        let mut cursor = Cursor::new(too_small);
        let result = find_ifd_data(&mut cursor);

        assert!(result.is_err());
    }
}
