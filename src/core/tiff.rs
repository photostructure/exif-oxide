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

/// Find and extract IFD data from a TIFF-based file
/// 
/// TIFF-based files include:
/// - TIFF (.tif/.tiff) 
/// - Canon CR2 (.cr2)
/// - Nikon NEF (.nef)
/// - Sony ARW (.arw)
/// - And many other RAW formats
pub fn find_ifd_data<R: Read + Seek>(reader: &mut R) -> Result<Option<TiffSegment>> {
    // Seek to the beginning
    reader.seek(SeekFrom::Start(0))?;
    
    // Read TIFF header (first 4 bytes)
    let mut header = [0u8; 4];
    reader.read_exact(&mut header)?;
    
    // Verify this is a valid TIFF file
    if header != TIFF_LITTLE_ENDIAN && header != TIFF_BIG_ENDIAN {
        return Ok(None); // Not a TIFF file
    }
    
    // For TIFF files, we need to read the entire file since IFDs can reference
    // data anywhere in the file, and we want to preserve the original file structure
    reader.seek(SeekFrom::Start(0))?;
    let mut data = Vec::new();
    reader.read_to_end(&mut data)?;
    
    // Validate minimum size (TIFF header + IFD offset)
    if data.len() < 8 {
        return Err(Error::InvalidData("TIFF file too small".into()));
    }
    
    // Parse the IFD offset to ensure the file is valid
    let is_little_endian = header == TIFF_LITTLE_ENDIAN;
    let ifd_offset = if is_little_endian {
        u32::from_le_bytes([data[4], data[5], data[6], data[7]])
    } else {
        u32::from_be_bytes([data[4], data[5], data[6], data[7]])
    } as usize;
    
    // Validate IFD offset is within file bounds
    if ifd_offset >= data.len() {
        return Err(Error::InvalidData("Invalid IFD offset in TIFF header".into()));
    }
    
    // Special handling for Canon CR2 files
    // CR2 files have "CR" marker at offset 8 after the standard TIFF header
    if data.len() >= 10 && &data[8..10] == b"CR" {
        // This is a Canon CR2 file - the IFD structure is valid
        // but we need to handle CR2-specific quirks in the IFD parser
    }
    
    Ok(Some(TiffSegment {
        data,
        offset: 0, // TIFF data starts at file beginning
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
            // Minimal IFD would follow here
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
            // Minimal IFD would follow here
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
            b'C', b'R',             // CR2 marker
            0x02, 0x00, 0x00, 0x00, // CR2 version
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