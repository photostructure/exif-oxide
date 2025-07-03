//! TIFF-specific processing and validation
//!
//! This module handles TIFF file format processing, including
//! header validation and EXIF data extraction for TIFF files.

use crate::types::{ExifError, Result};
use std::io::{Read, Seek};

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
}
