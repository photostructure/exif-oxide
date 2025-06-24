//! HEIF/HEIC/MP4 container parsing to extract EXIF data from QuickTime atoms
//!
//! HEIF, HEIC, and MP4 files use QuickTime container format with nested atoms.
//! EXIF data is stored in 'meta' atom within 'Exif' item.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/QuickTime.pm"]

use crate::error::Result;
use std::io::{Read, Seek, SeekFrom};

/// QuickTime container formats use big-endian 32-bit sizes
const ATOM_HEADER_SIZE: usize = 8; // 4 bytes size + 4 bytes type

/// Result of finding EXIF data in a HEIF/QuickTime container
#[derive(Debug)]
pub struct HeifExifSegment {
    /// The raw EXIF data (ready for IFD parsing, starts with TIFF header)
    pub data: Vec<u8>,
    /// Offset in the file where the EXIF data starts
    pub offset: u64,
}

/// QuickTime atom header
#[derive(Debug)]
struct AtomHeader {
    size: u64,
    atom_type: [u8; 4],
}

/// Find and extract EXIF data from a HEIF/HEIC/MP4 container file
pub fn find_exif_atom<R: Read + Seek>(reader: &mut R) -> Result<Option<HeifExifSegment>> {
    reader.seek(SeekFrom::Start(0))?;

    // Look for 'meta' atom which contains metadata
    if let Some(meta_atom) = find_atom(reader, b"meta", 0, None)? {
        // Parse meta atom to find EXIF item
        return find_exif_in_meta(reader, meta_atom.offset, meta_atom.size);
    }

    Ok(None)
}

/// Search for a specific atom type in the file
fn find_atom<R: Read + Seek>(
    reader: &mut R,
    target_type: &[u8; 4],
    start_offset: u64,
    container_end: Option<u64>,
) -> Result<Option<AtomInfo>> {
    reader.seek(SeekFrom::Start(start_offset))?;

    while let Ok(header) = read_atom_header(reader) {
        let current_pos = reader.stream_position()?;
        let atom_start = current_pos - ATOM_HEADER_SIZE as u64;

        // Check if we've reached the end of a container
        if let Some(end) = container_end {
            if atom_start >= end {
                break;
            }
        }

        if &header.atom_type == target_type {
            return Ok(Some(AtomInfo {
                offset: atom_start,
                size: header.size,
                _atom_type: header.atom_type,
            }));
        }

        // Skip to next atom
        if header.size == 0 {
            break; // Size 0 means "to end of file"
        }

        let next_offset = atom_start + header.size;
        reader.seek(SeekFrom::Start(next_offset))?;
    }

    Ok(None)
}

/// Find EXIF data within a 'meta' atom
fn find_exif_in_meta<R: Read + Seek>(
    reader: &mut R,
    meta_offset: u64,
    meta_size: u64,
) -> Result<Option<HeifExifSegment>> {
    // Meta atom has version/flags (4 bytes) after the header
    let meta_data_start = meta_offset + ATOM_HEADER_SIZE as u64 + 4;
    let meta_end = meta_offset + meta_size;

    reader.seek(SeekFrom::Start(meta_data_start))?;

    // Look for 'iinf' (item info) atom to find EXIF item ID
    if let Some(_iinf_atom) = find_atom(reader, b"iinf", meta_data_start, Some(meta_end))? {
        // In a full implementation, we would parse the iinf atom to find
        // the item ID for "Exif" type items, then use iloc to find the data.
        // For now, we'll use a simpler approach and look for common patterns.
    }

    // Look for 'iloc' (item location) atom
    if let Some(iloc_atom) = find_atom(reader, b"iloc", meta_data_start, Some(meta_end))? {
        // Parse iloc to find EXIF data locations
        if let Some(exif_data) = parse_iloc_for_exif(reader, iloc_atom.offset, iloc_atom.size)? {
            return Ok(Some(exif_data));
        }
    }

    // Fallback: Look for raw TIFF headers in the meta atom
    // This handles simpler cases where EXIF is embedded directly
    find_raw_exif_in_range(reader, meta_data_start, meta_end)
}

/// Parse an 'iloc' (item location) atom to find EXIF data
fn parse_iloc_for_exif<R: Read + Seek>(
    reader: &mut R,
    iloc_offset: u64,
    _iloc_size: u64,
) -> Result<Option<HeifExifSegment>> {
    // Skip atom header and version/flags
    reader.seek(SeekFrom::Start(iloc_offset + ATOM_HEADER_SIZE as u64 + 4))?;

    // Read iloc structure (simplified - real parsing is more complex)
    let mut params = [0u8; 4];
    reader.read_exact(&mut params)?;

    let offset_size = ((params[0] & 0xF0) >> 4) as usize;
    let length_size = (params[0] & 0x0F) as usize;
    let _base_offset_size = ((params[1] & 0xF0) >> 4) as usize;
    let _index_size = (params[1] & 0x0F) as usize;

    if offset_size == 0 || length_size == 0 || offset_size > 8 || length_size > 8 {
        return Ok(None); // Invalid or unsupported iloc format
    }

    // Read item count
    let mut item_count_bytes = [0u8; 4];
    reader.read_exact(&mut item_count_bytes)?;
    let item_count = if params[2] & 0x80 != 0 {
        // 32-bit item count
        u32::from_be_bytes(item_count_bytes)
    } else {
        // 16-bit item count
        u16::from_be_bytes([item_count_bytes[2], item_count_bytes[3]]) as u32
    };

    // Limit to reasonable number of items to prevent DoS
    if item_count > 1000 {
        return Ok(None);
    }

    // Parse items looking for one that might contain EXIF data
    for _ in 0..item_count {
        // Read item ID (simplified - real format is more complex)
        let mut item_id_bytes = [0u8; 4];
        reader.read_exact(&mut item_id_bytes)?;
        let _item_id = u32::from_be_bytes(item_id_bytes);

        // Skip construction method and data reference index
        reader.seek(SeekFrom::Current(4))?;

        // Read base offset (if present)
        if _base_offset_size > 0 {
            reader.seek(SeekFrom::Current(_base_offset_size as i64))?;
        }

        // Read extent count
        let mut extent_count_bytes = [0u8; 2];
        reader.read_exact(&mut extent_count_bytes)?;
        let extent_count = u16::from_be_bytes(extent_count_bytes);

        if extent_count > 100 {
            continue; // Skip items with too many extents
        }

        // Read extents
        for _ in 0..extent_count {
            // Read extent index (if present)
            if _index_size > 0 {
                reader.seek(SeekFrom::Current(_index_size as i64))?;
            }

            // Read extent offset
            let mut offset_bytes = vec![0u8; offset_size];
            reader.read_exact(&mut offset_bytes)?;
            let mut offset = 0u64;
            for &byte in &offset_bytes {
                offset = (offset << 8) | byte as u64;
            }

            // Read extent length
            let mut length_bytes = vec![0u8; length_size];
            reader.read_exact(&mut length_bytes)?;
            let mut length = 0u64;
            for &byte in &length_bytes {
                length = (length << 8) | byte as u64;
            }

            // Check if this extent contains EXIF data
            if (8..=1024 * 1024).contains(&length) {
                // Reasonable EXIF size
                let current_pos = reader.stream_position()?;

                // Read potential EXIF data
                reader.seek(SeekFrom::Start(offset))?;
                let mut potential_exif = vec![0u8; length as usize];
                if reader.read_exact(&mut potential_exif).is_ok() {
                    // Check for TIFF header
                    if potential_exif.len() >= 4 {
                        let tiff_header = &potential_exif[0..4];
                        if tiff_header == [0x49, 0x49, 0x2a, 0x00] || // Little endian
                           tiff_header == [0x4d, 0x4d, 0x00, 0x2a]
                        {
                            // Big endian
                            return Ok(Some(HeifExifSegment {
                                data: potential_exif,
                                offset,
                            }));
                        }
                    }
                }

                // Restore position
                reader.seek(SeekFrom::Start(current_pos))?;
            }
        }
    }

    Ok(None)
}

/// Fallback: Search for raw TIFF headers in a range (for simpler HEIF files)
fn find_raw_exif_in_range<R: Read + Seek>(
    reader: &mut R,
    start: u64,
    end: u64,
) -> Result<Option<HeifExifSegment>> {
    const SEARCH_CHUNK_SIZE: usize = 4096;
    let mut buffer = vec![0u8; SEARCH_CHUNK_SIZE];

    let mut pos = start;
    while pos < end {
        reader.seek(SeekFrom::Start(pos))?;

        let to_read = std::cmp::min(SEARCH_CHUNK_SIZE, (end - pos) as usize);
        let bytes_read = reader.read(&mut buffer[..to_read])?;

        if bytes_read < 4 {
            break;
        }

        // Look for TIFF headers
        for i in 0..bytes_read - 3 {
            let potential_header = &buffer[i..i + 4];
            if potential_header == [0x49, 0x49, 0x2a, 0x00] || // Little endian
               potential_header == [0x4d, 0x4d, 0x00, 0x2a]
            {
                // Big endian

                // Found potential TIFF header, try to read more data
                let exif_start = pos + i as u64;
                let max_exif_size = std::cmp::min(1024 * 1024, end - exif_start) as usize;

                reader.seek(SeekFrom::Start(exif_start))?;
                let mut exif_data = vec![0u8; max_exif_size];
                let actual_read = reader.read(&mut exif_data)?;
                exif_data.truncate(actual_read);

                if exif_data.len() >= 8 {
                    return Ok(Some(HeifExifSegment {
                        data: exif_data,
                        offset: exif_start,
                    }));
                }
            }
        }

        pos += (bytes_read - 3) as u64; // Overlap to avoid missing headers across chunks
    }

    Ok(None)
}

/// Read a QuickTime atom header
fn read_atom_header<R: Read>(reader: &mut R) -> Result<AtomHeader> {
    let mut size_bytes = [0u8; 4];
    reader.read_exact(&mut size_bytes)?;
    let size32 = u32::from_be_bytes(size_bytes);

    let mut atom_type = [0u8; 4];
    reader.read_exact(&mut atom_type)?;

    let size = if size32 == 1 {
        // Extended size (64-bit)
        let mut extended_size = [0u8; 8];
        reader.read_exact(&mut extended_size)?;
        u64::from_be_bytes(extended_size)
    } else if size32 == 0 {
        // Size extends to end of file
        0
    } else {
        size32 as u64
    };

    Ok(AtomHeader { size, atom_type })
}

/// Information about a found atom
#[derive(Debug)]
struct AtomInfo {
    offset: u64,
    size: u64,
    _atom_type: [u8; 4],
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_atom_header_parsing() {
        let atom_data = [
            0x00, 0x00, 0x00, 0x20, // Size: 32 bytes
            b'm', b'e', b't', b'a', // Type: meta
        ];

        let mut cursor = Cursor::new(atom_data);
        let header = read_atom_header(&mut cursor).unwrap();

        assert_eq!(header.size, 32);
        assert_eq!(header.atom_type, [b'm', b'e', b't', b'a']);
    }

    #[test]
    fn test_extended_atom_header() {
        let atom_data = [
            0x00, 0x00, 0x00, 0x01, // Size: 1 (indicates extended size)
            b'm', b'e', b't', b'a', // Type: meta
            0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, // Extended size: 65536
        ];

        let mut cursor = Cursor::new(atom_data);
        let header = read_atom_header(&mut cursor).unwrap();

        assert_eq!(header.size, 65536);
        assert_eq!(header.atom_type, [b'm', b'e', b't', b'a']);
    }

    #[test]
    fn test_zero_size_atom() {
        let atom_data = [
            0x00, 0x00, 0x00, 0x00, // Size: 0 (extends to end of file)
            b'm', b'd', b'a', b't', // Type: mdat
        ];

        let mut cursor = Cursor::new(atom_data);
        let header = read_atom_header(&mut cursor).unwrap();

        assert_eq!(header.size, 0);
        assert_eq!(header.atom_type, [b'm', b'd', b'a', b't']);
    }

    #[test]
    fn test_incomplete_atom_header() {
        let incomplete_data = [0x00, 0x00, 0x00]; // Only 3 bytes

        let mut cursor = Cursor::new(incomplete_data);
        let result = read_atom_header(&mut cursor);

        assert!(result.is_err());
    }
}
