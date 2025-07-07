//! TIFF-specific processing and validation
//!
//! This module handles TIFF file format processing, including
//! header validation and EXIF data extraction for TIFF files.

use crate::types::{ExifError, Result};
use std::io::{Read, Seek};

/// XMP tag in TIFF IFD0
const TIFF_XMP_TAG: u16 = 0x02BC; // 700 decimal

/// Extract EXIF data from TIFF file
///
/// For TIFF files, the entire file is essentially EXIF/TIFF data,
/// so we validate the TIFF header and return the file contents.
pub fn extract_tiff_exif<R: Read + Seek>(mut reader: R) -> Result<Vec<u8>> {
    // Read the entire file for TIFF processing
    let mut tiff_data = Vec::new();
    reader.read_to_end(&mut tiff_data)?;

    // Validate minimum TIFF header size
    if tiff_data.len() < 8 {
        return Err(ExifError::InvalidFormat(
            "TIFF file too small for valid header".to_string(),
        ));
    }

    // Validate TIFF magic bytes
    let header = &tiff_data[0..4];
    match header {
        [0x49, 0x49, 0x2A, 0x00] => {
            // Little-endian TIFF
        }
        [0x4D, 0x4D, 0x00, 0x2A] => {
            // Big-endian TIFF
        }
        _ => {
            return Err(ExifError::InvalidFormat(
                "Invalid TIFF magic bytes".to_string(),
            ));
        }
    }

    Ok(tiff_data)
}

/// Validate TIFF file format
///
/// Checks if the provided data represents a valid TIFF file
/// by examining the magic bytes and basic header structure.
pub fn validate_tiff_format(data: &[u8]) -> Result<()> {
    if data.len() < 8 {
        return Err(ExifError::InvalidFormat(
            "TIFF data too small for valid header".to_string(),
        ));
    }

    // Check TIFF magic bytes
    let header = &data[0..4];
    match header {
        [0x49, 0x49, 0x2A, 0x00] | [0x4D, 0x4D, 0x00, 0x2A] => Ok(()),
        _ => Err(ExifError::InvalidFormat(
            "Invalid TIFF magic bytes".to_string(),
        )),
    }
}

/// Get TIFF endianness from header
///
/// Returns true for little-endian, false for big-endian
pub fn get_tiff_endianness(data: &[u8]) -> Result<bool> {
    if data.len() < 2 {
        return Err(ExifError::InvalidFormat(
            "TIFF data too small to determine endianness".to_string(),
        ));
    }

    match &data[0..2] {
        [0x49, 0x49] => Ok(true),  // Little-endian
        [0x4D, 0x4D] => Ok(false), // Big-endian
        _ => Err(ExifError::InvalidFormat(
            "Invalid TIFF endianness markers".to_string(),
        )),
    }
}

/// Extract XMP data from TIFF file
///
/// Scans the TIFF IFD0 for the XMP tag (0x02bc) and extracts the XMP packet.
/// Returns the raw XMP data if found.
pub fn extract_tiff_xmp(data: &[u8]) -> Result<Option<Vec<u8>>> {
    // Validate minimum TIFF header size
    if data.len() < 8 {
        return Ok(None);
    }

    // Determine endianness and read IFD0 offset
    let (is_little_endian, ifd0_offset) = match &data[0..4] {
        [0x49, 0x49, 0x2A, 0x00] => {
            // Little-endian TIFF
            let offset = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
            (true, offset)
        }
        [0x4D, 0x4D, 0x00, 0x2A] => {
            // Big-endian TIFF
            let offset = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
            (false, offset)
        }
        _ => return Ok(None), // Invalid TIFF magic
    };

    // Check if IFD0 offset is valid
    if ifd0_offset as usize + 2 > data.len() {
        return Ok(None);
    }

    // Read number of directory entries in IFD0
    let entry_count = if is_little_endian {
        u16::from_le_bytes([data[ifd0_offset as usize], data[ifd0_offset as usize + 1]])
    } else {
        u16::from_be_bytes([data[ifd0_offset as usize], data[ifd0_offset as usize + 1]])
    };

    // Each IFD entry is 12 bytes
    let entries_start = ifd0_offset as usize + 2;
    let entries_end = entries_start + (entry_count as usize * 12);

    if entries_end > data.len() {
        return Ok(None);
    }

    // Scan IFD entries for XMP tag (0x02bc)
    for i in 0..entry_count {
        let entry_offset = entries_start + (i as usize * 12);

        // Read tag ID
        let tag_id = if is_little_endian {
            u16::from_le_bytes([data[entry_offset], data[entry_offset + 1]])
        } else {
            u16::from_be_bytes([data[entry_offset], data[entry_offset + 1]])
        };

        if tag_id == TIFF_XMP_TAG {
            // Found XMP tag - read data type, count, and value/offset
            let data_type = if is_little_endian {
                u16::from_le_bytes([data[entry_offset + 2], data[entry_offset + 3]])
            } else {
                u16::from_be_bytes([data[entry_offset + 2], data[entry_offset + 3]])
            };

            let count = if is_little_endian {
                u32::from_le_bytes([
                    data[entry_offset + 4],
                    data[entry_offset + 5],
                    data[entry_offset + 6],
                    data[entry_offset + 7],
                ])
            } else {
                u32::from_be_bytes([
                    data[entry_offset + 4],
                    data[entry_offset + 5],
                    data[entry_offset + 6],
                    data[entry_offset + 7],
                ])
            };

            // XMP should be data type 1 (BYTE) or 7 (UNDEFINED)
            if data_type == 1 || data_type == 7 {
                let value_offset = if is_little_endian {
                    u32::from_le_bytes([
                        data[entry_offset + 8],
                        data[entry_offset + 9],
                        data[entry_offset + 10],
                        data[entry_offset + 11],
                    ])
                } else {
                    u32::from_be_bytes([
                        data[entry_offset + 8],
                        data[entry_offset + 9],
                        data[entry_offset + 10],
                        data[entry_offset + 11],
                    ])
                };

                // Extract XMP data
                let xmp_start = value_offset as usize;
                let xmp_end = xmp_start + count as usize;

                if xmp_end <= data.len() {
                    return Ok(Some(data[xmp_start..xmp_end].to_vec()));
                }
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_extract_tiff_exif_little_endian() {
        let tiff_data = [
            0x49, 0x49, 0x2A, 0x00, // TIFF LE header
            0x08, 0x00, 0x00, 0x00, // IFD offset
            0x00, 0x00, // Empty IFD
            0x00, 0x00, 0x00, 0x00, // Next IFD = none
        ];
        let cursor = Cursor::new(&tiff_data);
        let result = extract_tiff_exif(cursor).unwrap();
        assert_eq!(result, tiff_data);
    }

    #[test]
    fn test_extract_tiff_exif_big_endian() {
        let tiff_data = [
            0x4D, 0x4D, 0x00, 0x2A, // TIFF BE header
            0x00, 0x00, 0x00, 0x08, // IFD offset
            0x00, 0x00, // Empty IFD
            0x00, 0x00, 0x00, 0x00, // Next IFD = none
        ];
        let cursor = Cursor::new(&tiff_data);
        let result = extract_tiff_exif(cursor).unwrap();
        assert_eq!(result, tiff_data);
    }

    #[test]
    fn test_extract_tiff_exif_too_small() {
        let small_data = [0x49, 0x49]; // Too small
        let cursor = Cursor::new(&small_data);
        let result = extract_tiff_exif(cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_tiff_exif_invalid_magic() {
        let invalid_data = [
            0x12, 0x34, 0x56, 0x78, // Invalid magic
            0x08, 0x00, 0x00, 0x00, // IFD offset
        ];
        let cursor = Cursor::new(&invalid_data);
        let result = extract_tiff_exif(cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_tiff_format() {
        // Valid little-endian TIFF
        let le_tiff = [0x49, 0x49, 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00];
        assert!(validate_tiff_format(&le_tiff).is_ok());

        // Valid big-endian TIFF
        let be_tiff = [0x4D, 0x4D, 0x00, 0x2A, 0x00, 0x00, 0x00, 0x08];
        assert!(validate_tiff_format(&be_tiff).is_ok());

        // Invalid magic bytes
        let invalid = [0x12, 0x34, 0x56, 0x78, 0x08, 0x00, 0x00, 0x00];
        assert!(validate_tiff_format(&invalid).is_err());

        // Too small
        let small = [0x49, 0x49];
        assert!(validate_tiff_format(&small).is_err());
    }

    #[test]
    fn test_get_tiff_endianness() {
        // Little-endian
        let le_data = [0x49, 0x49, 0x2A, 0x00];
        assert!(get_tiff_endianness(&le_data).unwrap());

        // Big-endian
        let be_data = [0x4D, 0x4D, 0x00, 0x2A];
        assert!(!get_tiff_endianness(&be_data).unwrap());

        // Invalid
        let invalid = [0x12, 0x34];
        assert!(get_tiff_endianness(&invalid).is_err());

        // Too small
        let small = [0x49];
        assert!(get_tiff_endianness(&small).is_err());
    }

    #[test]
    fn test_extract_tiff_xmp_no_xmp() {
        // TIFF with no XMP tag
        let tiff_data = [
            0x49, 0x49, 0x2A, 0x00, // TIFF LE header
            0x08, 0x00, 0x00, 0x00, // IFD offset
            0x00, 0x00, // Empty IFD (0 entries)
            0x00, 0x00, 0x00, 0x00, // Next IFD = none
        ];
        let result = extract_tiff_xmp(&tiff_data).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_tiff_xmp_with_xmp() {
        // Create a minimal TIFF with XMP tag
        let xmp_data = b"<?xml?><x:xmpmeta></x:xmpmeta>";
        let xmp_offset = 26u32; // After header(8) + IFD entry count(2) + entry(12) + next IFD(4) = 26

        let mut tiff_data = vec![
            0x49,
            0x49,
            0x2A,
            0x00, // TIFF LE header
            0x08,
            0x00,
            0x00,
            0x00, // IFD offset (8)
            0x01,
            0x00, // 1 IFD entry
            // XMP tag entry (12 bytes)
            0xBC,
            0x02, // Tag ID (0x02BC = 700)
            0x01,
            0x00, // Data type (BYTE)
            xmp_data.len() as u8,
            0x00,
            0x00,
            0x00, // Count (XMP data length)
            (xmp_offset & 0xFF) as u8,
            (xmp_offset >> 8) as u8,
            (xmp_offset >> 16) as u8,
            (xmp_offset >> 24) as u8, // Value offset
            0x00,
            0x00,
            0x00,
            0x00, // Next IFD = none
        ];

        // Append XMP data at the offset
        tiff_data.extend_from_slice(xmp_data);

        let result = extract_tiff_xmp(&tiff_data).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), xmp_data);
    }

    #[test]
    fn test_extract_tiff_xmp_invalid_tiff() {
        let invalid_data = [0x12, 0x34, 0x56, 0x78];
        let result = extract_tiff_xmp(&invalid_data).unwrap();
        assert!(result.is_none());
    }
}
