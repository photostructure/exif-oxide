//! TIFF utilities for reading basic tag information
//! Used for NEF/NRW detection and other TIFF-based format analysis

use std::io::{Read, Seek, SeekFrom};

/// TIFF IFD tag numbers we care about
pub const TAG_COMPRESSION: u16 = 0x0103;
pub const TAG_MAKE: u16 = 0x010F;
pub const TAG_MODEL: u16 = 0x0110;

/// TIFF compression values
pub const COMPRESSION_JPEG: u16 = 6;

/// Read a TIFF IFD and extract specific tag values
/// Returns (compression, has_nef_linearization_table) - Note: has_nef_linearization_table is deprecated and always false
pub fn read_tiff_ifd0_info<R: Read + Seek>(reader: &mut R) -> Option<(Option<u16>, bool)> {
    let mut header = [0u8; 8];
    reader.seek(SeekFrom::Start(0)).ok()?;
    reader.read_exact(&mut header).ok()?;

    // Check TIFF magic and get byte order
    let (little_endian, valid) = match &header[0..4] {
        b"II\x2a\x00" => (true, true),  // Little-endian TIFF
        b"MM\x00\x2a" => (false, true), // Big-endian TIFF
        _ => (false, false),
    };

    if !valid {
        return None;
    }

    // Get IFD0 offset
    let ifd_offset = if little_endian {
        u32::from_le_bytes([header[4], header[5], header[6], header[7]])
    } else {
        u32::from_be_bytes([header[4], header[5], header[6], header[7]])
    } as u64;

    // Seek to IFD0
    reader.seek(SeekFrom::Start(ifd_offset)).ok()?;

    // Read number of directory entries
    let mut entry_count_bytes = [0u8; 2];
    reader.read_exact(&mut entry_count_bytes).ok()?;
    let entry_count = if little_endian {
        u16::from_le_bytes(entry_count_bytes)
    } else {
        u16::from_be_bytes(entry_count_bytes)
    };

    let mut compression = None;

    // Read each directory entry (12 bytes each)
    for _ in 0..entry_count {
        let mut entry = [0u8; 12];
        reader.read_exact(&mut entry).ok()?;

        let tag = if little_endian {
            u16::from_le_bytes([entry[0], entry[1]])
        } else {
            u16::from_be_bytes([entry[0], entry[1]])
        };

        if tag == TAG_COMPRESSION {
            // Type should be SHORT (3) and count should be 1
            let value_offset = if little_endian {
                u16::from_le_bytes([entry[8], entry[9]])
            } else {
                u16::from_be_bytes([entry[8], entry[9]])
            };
            compression = Some(value_offset);
        }
    }

    // Return compression value and false for has_nef_linearization (deprecated)
    Some((compression, false))
}
