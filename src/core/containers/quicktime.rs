//! QuickTime/MP4 container format parsing for metadata extraction
//!
//! QuickTime container format is used by:
//! - MOV (QuickTime movie) files
//! - MP4 (MPEG-4) files
//! - M4V, M4A (MPEG-4 variants)
//! - 3GP (3GPP) files

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/QuickTime.pm"]

use crate::error::Result;
use std::io::{Read, Seek, SeekFrom};

/// QuickTime file type atoms that indicate valid files
const VALID_FTYP_BRANDS: &[&[u8; 4]] = &[
    b"qt  ", // QuickTime
    b"mp41", b"mp42", b"isom", // MP4 variants
    b"M4V ", b"M4A ", b"M4B ", // iTunes variants
    b"3gp4", b"3gp5", b"3gp6", // 3GPP variants
    b"mmp4", // Mobile MP4
];

/// Result of finding metadata in a QuickTime container
#[derive(Debug)]
pub struct QuickTimeMetadataSegment {
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
    QuickTimeMetadata,
}

/// QuickTime atom header
#[derive(Debug)]
struct AtomHeader {
    size: u64,
    atom_type: [u8; 4],
}

/// Find and extract metadata from a QuickTime/MP4 container file
pub fn find_metadata<R: Read + Seek>(reader: &mut R) -> Result<Option<QuickTimeMetadataSegment>> {
    reader.seek(SeekFrom::Start(0))?;

    // First, verify this is a valid QuickTime/MP4 file by checking ftyp atom
    if !verify_quicktime_file(reader)? {
        return Ok(None);
    }

    // Seek back to start for metadata search
    reader.seek(SeekFrom::Start(0))?;

    // Look for metadata in common locations:
    // 1. moov/meta atom (standard metadata location)
    // 2. moov/udta atom (user data)
    // 3. uuid atoms with known EXIF/XMP signatures

    if let Some(moov_atom) = find_atom(reader, b"moov", 0, None)? {
        // Search within moov atom
        if let Some(metadata) = search_moov_for_metadata(reader, moov_atom.offset, moov_atom.size)?
        {
            return Ok(Some(metadata));
        }
    }

    // Search for UUID atoms that might contain EXIF/XMP
    reader.seek(SeekFrom::Start(0))?;
    if let Some(metadata) = search_uuid_atoms(reader)? {
        return Ok(Some(metadata));
    }

    Ok(None)
}

/// Verify this is a valid QuickTime/MP4 file
fn verify_quicktime_file<R: Read + Seek>(reader: &mut R) -> Result<bool> {
    // Skip first atom size/type to look for ftyp
    reader.seek(SeekFrom::Start(0))?;

    // Read first atom header
    let header = match read_atom_header(reader) {
        Ok(h) => h,
        Err(_) => return Ok(false),
    };

    // First atom is often ftyp, but not always
    if &header.atom_type == b"ftyp" {
        // Read major brand
        let mut brand = [0u8; 4];
        reader.read_exact(&mut brand)?;

        // Check if it's a known brand
        for valid_brand in VALID_FTYP_BRANDS {
            if brand == **valid_brand {
                return Ok(true);
            }
        }
    }

    // If first atom isn't ftyp, look for it in first 1KB
    reader.seek(SeekFrom::Start(0))?;
    if find_atom(reader, b"ftyp", 0, Some(1024))?.is_some() {
        return Ok(true);
    }

    Ok(false)
}

/// Search moov atom for metadata
fn search_moov_for_metadata<R: Read + Seek>(
    reader: &mut R,
    moov_offset: u64,
    moov_size: u64,
) -> Result<Option<QuickTimeMetadataSegment>> {
    let moov_end = moov_offset + moov_size;

    // Look for meta atom within moov
    if let Some(meta_atom) = find_atom(reader, b"meta", moov_offset + 8, Some(moov_end))? {
        // Meta atom has version/flags after header
        reader.seek(SeekFrom::Start(meta_atom.offset + 8 + 4))?;

        // Look for hdlr atom to verify this is metadata
        if let Some(_hdlr) = find_atom(
            reader,
            b"hdlr",
            meta_atom.offset + 12,
            Some(meta_atom.offset + meta_atom.size),
        )? {
            // Look for ilst (item list) which contains the actual metadata
            if let Some(ilst) = find_atom(
                reader,
                b"ilst",
                meta_atom.offset + 12,
                Some(meta_atom.offset + meta_atom.size),
            )? {
                // Parse ilst for EXIF/XMP data
                if let Some(metadata) = parse_ilst_metadata(reader, ilst.offset, ilst.size)? {
                    return Ok(Some(metadata));
                }
            }
        }
    }

    // Look for udta (user data) atom within moov
    if let Some(udta_atom) = find_atom(reader, b"udta", moov_offset + 8, Some(moov_end))? {
        // Look for EXIF atom within udta
        if let Some(exif_atom) = find_atom(
            reader,
            b"Exif",
            udta_atom.offset + 8,
            Some(udta_atom.offset + udta_atom.size),
        )? {
            // Read EXIF data
            reader.seek(SeekFrom::Start(exif_atom.offset + 8))?;
            let mut exif_data = vec![0u8; (exif_atom.size - 8) as usize];
            reader.read_exact(&mut exif_data)?;

            // EXIF in MOV has 4-byte offset prefix
            if exif_data.len() > 4 {
                let offset_bytes = [exif_data[0], exif_data[1], exif_data[2], exif_data[3]];
                let offset = u32::from_be_bytes(offset_bytes) as usize;

                if offset < exif_data.len() {
                    let actual_exif = exif_data[offset..].to_vec();

                    // Validate TIFF header
                    if actual_exif.len() >= 4 {
                        let tiff_header = &actual_exif[0..4];
                        if tiff_header == [0x49, 0x49, 0x2a, 0x00]
                            || tiff_header == [0x4d, 0x4d, 0x00, 0x2a]
                        {
                            return Ok(Some(QuickTimeMetadataSegment {
                                data: actual_exif,
                                offset: exif_atom.offset + 8 + offset as u64,
                                metadata_type: MetadataType::Exif,
                            }));
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Search for UUID atoms that might contain EXIF/XMP
fn search_uuid_atoms<R: Read + Seek>(reader: &mut R) -> Result<Option<QuickTimeMetadataSegment>> {
    // Known UUID values for EXIF/XMP
    const EXIF_UUID: [u8; 16] = [
        0x05, 0x37, 0xcd, 0xab, 0x9d, 0x0c, 0x44, 0x31, 0xa7, 0x2a, 0xfa, 0x56, 0x1f, 0x2a, 0x11,
        0x3e,
    ];
    const XMP_UUID: [u8; 16] = [
        0xbe, 0x7a, 0xcf, 0xcb, 0x97, 0xa9, 0x42, 0xe8, 0x9c, 0x71, 0x99, 0x94, 0x91, 0xe3, 0xaf,
        0xac,
    ];

    let file_size = reader.seek(SeekFrom::End(0))?;
    reader.seek(SeekFrom::Start(0))?;

    while reader.stream_position()? < file_size {
        let pos = reader.stream_position()?;

        let header = match read_atom_header(reader) {
            Ok(h) => h,
            Err(_) => break,
        };

        if &header.atom_type == b"uuid" && header.size >= 24 {
            // Read UUID
            let mut uuid = [0u8; 16];
            reader.read_exact(&mut uuid)?;

            if uuid == EXIF_UUID {
                // EXIF UUID - read EXIF data
                let exif_size = (header.size - 24) as usize; // minus header and UUID
                let mut exif_data = vec![0u8; exif_size];
                reader.read_exact(&mut exif_data)?;

                // Validate TIFF header
                if exif_data.len() >= 4 {
                    let tiff_header = &exif_data[0..4];
                    if tiff_header == [0x49, 0x49, 0x2a, 0x00]
                        || tiff_header == [0x4d, 0x4d, 0x00, 0x2a]
                    {
                        return Ok(Some(QuickTimeMetadataSegment {
                            data: exif_data,
                            offset: pos + 24, // After header and UUID
                            metadata_type: MetadataType::Exif,
                        }));
                    }
                }
            } else if uuid == XMP_UUID {
                // XMP UUID - read XMP data
                let xmp_size = (header.size - 24) as usize;
                let mut xmp_data = vec![0u8; xmp_size];
                reader.read_exact(&mut xmp_data)?;

                return Ok(Some(QuickTimeMetadataSegment {
                    data: xmp_data,
                    offset: pos + 24,
                    metadata_type: MetadataType::Xmp,
                }));
            } else {
                // Skip rest of this UUID atom
                let skip = header.size - 24;
                reader.seek(SeekFrom::Current(skip as i64))?;
            }
        } else {
            // Skip this atom
            if header.size > 8 {
                reader.seek(SeekFrom::Current((header.size - 8) as i64))?;
            } else if header.size == 0 {
                // Size 0 means to end of file
                break;
            }
        }
    }

    Ok(None)
}

/// Parse ilst (item list) for metadata
fn parse_ilst_metadata<R: Read + Seek>(
    reader: &mut R,
    ilst_offset: u64,
    ilst_size: u64,
) -> Result<Option<QuickTimeMetadataSegment>> {
    let ilst_end = ilst_offset + ilst_size;
    let mut pos = ilst_offset + 8; // Skip ilst header

    reader.seek(SeekFrom::Start(pos))?;

    while pos < ilst_end {
        reader.seek(SeekFrom::Start(pos))?;

        let header = match read_atom_header(reader) {
            Ok(h) => h,
            Err(_) => break,
        };

        // Look for known metadata atoms
        // QuickTime metadata is complex - for now just detect presence
        // Full implementation would parse each metadata type

        pos += header.size;
    }

    Ok(None)
}

/// Find a specific atom type in the file
fn find_atom<R: Read + Seek>(
    reader: &mut R,
    target_type: &[u8; 4],
    start_offset: u64,
    max_offset: Option<u64>,
) -> Result<Option<AtomInfo>> {
    reader.seek(SeekFrom::Start(start_offset))?;

    while let Ok(header) = read_atom_header(reader) {
        let current_pos = reader.stream_position()?;
        let atom_start = current_pos - 8;

        // Check if we've exceeded max offset
        if let Some(max) = max_offset {
            if atom_start >= max {
                break;
            }
        }

        if &header.atom_type == target_type {
            return Ok(Some(AtomInfo {
                offset: atom_start,
                size: header.size,
            }));
        }

        // Skip to next atom
        if header.size > 8 {
            reader.seek(SeekFrom::Current((header.size - 8) as i64))?;
        } else if header.size == 0 {
            break; // Size 0 means to end of file
        } else {
            break; // Invalid size
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_atom_header_parsing() {
        let atom_data = [
            0x00, 0x00, 0x00, 0x20, // Size: 32 bytes
            b'm', b'o', b'o', b'v', // Type: moov
        ];

        let mut cursor = Cursor::new(&atom_data);
        let header = read_atom_header(&mut cursor).unwrap();

        assert_eq!(header.size, 32);
        assert_eq!(header.atom_type, [b'm', b'o', b'o', b'v']);
    }

    #[test]
    fn test_extended_atom_header() {
        let atom_data = [
            0x00, 0x00, 0x00, 0x01, // Size: 1 (extended)
            b'm', b'd', b'a', b't', // Type: mdat
            0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, // Extended size: 65536
        ];

        let mut cursor = Cursor::new(&atom_data);
        let header = read_atom_header(&mut cursor).unwrap();

        assert_eq!(header.size, 65536);
        assert_eq!(header.atom_type, [b'm', b'd', b'a', b't']);
    }

    #[test]
    fn test_ftyp_brand_validation() {
        // Test valid MP4 brand
        assert!(VALID_FTYP_BRANDS.contains(&b"mp42"));

        // Test valid QuickTime brand
        assert!(VALID_FTYP_BRANDS.contains(&b"qt  "));
    }
}
