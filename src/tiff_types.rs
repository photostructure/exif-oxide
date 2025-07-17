//! TIFF format types and structures
//!
//! This module contains the fundamental TIFF data structures used throughout the EXIF parsing
//! process, translating ExifTool's TIFF format definitions from lib/Image/ExifTool/Exif.pm.
//! These types handle byte order detection, format validation, and IFD entry parsing.

use crate::types::{ExifError, Result};

/// TIFF format types mapping to ExifTool's format system
/// ExifTool: lib/Image/ExifTool/Exif.pm @formatName array
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TiffFormat {
    Byte = 1,       // int8u
    Ascii = 2,      // string
    Short = 3,      // int16u
    Long = 4,       // int32u
    Rational = 5,   // rational64u
    SByte = 6,      // int8s
    Undefined = 7,  // undef
    SShort = 8,     // int16s
    SLong = 9,      // int32s
    SRational = 10, // rational64s
    Float = 11,     // float
    Double = 12,    // double
    Ifd = 13,       // ifd
}

impl TiffFormat {
    /// Get byte size for this format type
    /// ExifTool: lib/Image/ExifTool/Exif.pm @formatSize array
    pub fn byte_size(self) -> usize {
        match self {
            TiffFormat::Byte | TiffFormat::Ascii | TiffFormat::SByte | TiffFormat::Undefined => 1,
            TiffFormat::Short | TiffFormat::SShort => 2,
            TiffFormat::Long | TiffFormat::SLong | TiffFormat::Float | TiffFormat::Ifd => 4,
            TiffFormat::Rational | TiffFormat::SRational | TiffFormat::Double => 8,
        }
    }

    /// Create from format number, following ExifTool's validation
    /// ExifTool: lib/Image/ExifTool/Exif.pm:6352 format validation
    pub fn from_u16(format: u16) -> Result<Self> {
        match format {
            1 => Ok(TiffFormat::Byte),
            2 => Ok(TiffFormat::Ascii),
            3 => Ok(TiffFormat::Short),
            4 => Ok(TiffFormat::Long),
            5 => Ok(TiffFormat::Rational),
            6 => Ok(TiffFormat::SByte),
            7 => Ok(TiffFormat::Undefined),
            8 => Ok(TiffFormat::SShort),
            9 => Ok(TiffFormat::SLong),
            10 => Ok(TiffFormat::SRational),
            11 => Ok(TiffFormat::Float),
            12 => Ok(TiffFormat::Double),
            13 => Ok(TiffFormat::Ifd),
            _ => Err(ExifError::ParseError(format!(
                "Invalid TIFF format type: {format}"
            ))),
        }
    }
}

/// Byte order for TIFF data
/// ExifTool: lib/Image/ExifTool/Exif.pm TIFF header validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    LittleEndian, // "II" - Intel format
    BigEndian,    // "MM" - Motorola format
}

impl ByteOrder {
    /// Read u16 value respecting byte order
    pub fn read_u16(self, data: &[u8], offset: usize) -> Result<u16> {
        if offset + 2 > data.len() {
            return Err(ExifError::ParseError("Not enough data for u16".to_string()));
        }
        let bytes = &data[offset..offset + 2];
        Ok(match self {
            ByteOrder::LittleEndian => u16::from_le_bytes([bytes[0], bytes[1]]),
            ByteOrder::BigEndian => u16::from_be_bytes([bytes[0], bytes[1]]),
        })
    }

    /// Read u32 value respecting byte order  
    pub fn read_u32(self, data: &[u8], offset: usize) -> Result<u32> {
        if offset + 4 > data.len() {
            return Err(ExifError::ParseError("Not enough data for u32".to_string()));
        }
        let bytes = &data[offset..offset + 4];
        Ok(match self {
            ByteOrder::LittleEndian => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            ByteOrder::BigEndian => u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        })
    }
}

/// TIFF header structure
/// ExifTool: lib/Image/ExifTool/Exif.pm TIFF header validation
#[derive(Debug, Clone)]
pub struct TiffHeader {
    pub byte_order: ByteOrder,
    pub magic: u16,       // Should be 42 (0x002A) for TIFF or 85 (0x0055) for RW2
    pub ifd0_offset: u32, // Offset to first IFD
}

impl TiffHeader {
    /// Parse TIFF header from data
    /// ExifTool: lib/Image/ExifTool/Exif.pm:6174-6248 header processing
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            return Err(ExifError::ParseError(
                "TIFF header too short (need 8 bytes)".to_string(),
            ));
        }

        // Detect byte order from first 2 bytes
        let byte_order = match &data[0..2] {
            [0x49, 0x49] => ByteOrder::LittleEndian, // "II"
            [0x4D, 0x4D] => ByteOrder::BigEndian,    // "MM"
            _ => {
                return Err(ExifError::ParseError(
                    "Invalid TIFF byte order marker".to_string(),
                ))
            }
        };

        // Read magic number (should be 42 for standard TIFF or 85 for RW2)
        // ExifTool: Panasonic RW2 files use magic number 85 (0x0055) instead of 42 (0x002A)
        let magic = byte_order.read_u16(data, 2)?;
        if magic != 42 && magic != 85 {
            return Err(ExifError::ParseError(format!(
                "Invalid TIFF magic number: {magic} (expected 42 or 85 for RW2)"
            )));
        }

        // Read IFD0 offset
        let ifd0_offset = byte_order.read_u32(data, 4)?;

        Ok(TiffHeader {
            byte_order,
            magic,
            ifd0_offset,
        })
    }
}

/// IFD entry structure (12 bytes each)
/// ExifTool: lib/Image/ExifTool/Exif.pm:6347-6351 entry reading
#[derive(Debug, Clone)]
pub struct IfdEntry {
    pub tag_id: u16,
    pub format: TiffFormat,
    pub count: u32,
    pub value_or_offset: u32,
}

impl IfdEntry {
    /// Parse IFD entry from 12-byte data block
    /// ExifTool: lib/Image/ExifTool/Exif.pm:6348-6350 entry structure
    pub fn parse(data: &[u8], offset: usize, byte_order: ByteOrder) -> Result<Self> {
        if offset + 12 > data.len() {
            return Err(ExifError::ParseError(
                "Not enough data for IFD entry".to_string(),
            ));
        }

        let tag_id = byte_order.read_u16(data, offset)?;
        let format_num = byte_order.read_u16(data, offset + 2)?;
        let format = TiffFormat::from_u16(format_num)?;
        let count = byte_order.read_u32(data, offset + 4)?;
        let value_or_offset = byte_order.read_u32(data, offset + 8)?;

        Ok(IfdEntry {
            tag_id,
            format,
            count,
            value_or_offset,
        })
    }

    /// Calculate total size of this entry's data
    /// ExifTool: lib/Image/ExifTool/Exif.pm:6390 size calculation
    pub fn data_size(&self) -> u32 {
        // Protect against overflow with large count values
        self.count.saturating_mul(self.format.byte_size() as u32)
    }

    /// Check if value is stored inline (≤4 bytes) or as offset
    /// ExifTool: lib/Image/ExifTool/Exif.pm:6392 inline vs offset logic
    pub fn is_inline(&self) -> bool {
        self.data_size() <= 4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiff_format_byte_size() {
        assert_eq!(TiffFormat::Byte.byte_size(), 1);
        assert_eq!(TiffFormat::Short.byte_size(), 2);
        assert_eq!(TiffFormat::Long.byte_size(), 4);
        assert_eq!(TiffFormat::Rational.byte_size(), 8);
    }

    #[test]
    fn test_tiff_format_from_u16() {
        assert_eq!(TiffFormat::from_u16(1).unwrap(), TiffFormat::Byte);
        assert_eq!(TiffFormat::from_u16(2).unwrap(), TiffFormat::Ascii);
        assert_eq!(TiffFormat::from_u16(3).unwrap(), TiffFormat::Short);
        assert!(TiffFormat::from_u16(99).is_err());
    }

    #[test]
    fn test_byte_order_read() {
        let data = [0x12, 0x34, 0x56, 0x78];

        // Little-endian
        let le = ByteOrder::LittleEndian;
        assert_eq!(le.read_u16(&data, 0).unwrap(), 0x3412);
        assert_eq!(le.read_u32(&data, 0).unwrap(), 0x78563412);

        // Big-endian
        let be = ByteOrder::BigEndian;
        assert_eq!(be.read_u16(&data, 0).unwrap(), 0x1234);
        assert_eq!(be.read_u32(&data, 0).unwrap(), 0x12345678);
    }

    #[test]
    fn test_tiff_header_parse() {
        // Little-endian TIFF header
        let le_data = [
            0x49, 0x49, // "II" - little-endian
            0x2A, 0x00, // Magic: 42 (LE)
            0x08, 0x00, 0x00, 0x00, // IFD0 offset: 8 (LE)
        ];

        let header = TiffHeader::parse(&le_data).unwrap();
        assert_eq!(header.byte_order, ByteOrder::LittleEndian);
        assert_eq!(header.magic, 42);
        assert_eq!(header.ifd0_offset, 8);

        // Big-endian TIFF header
        let be_data = [
            0x4D, 0x4D, // "MM" - big-endian
            0x00, 0x2A, // Magic: 42 (BE)
            0x00, 0x00, 0x00, 0x08, // IFD0 offset: 8 (BE)
        ];

        let header = TiffHeader::parse(&be_data).unwrap();
        assert_eq!(header.byte_order, ByteOrder::BigEndian);
        assert_eq!(header.magic, 42);
        assert_eq!(header.ifd0_offset, 8);
    }

    #[test]
    fn test_ifd_entry_parse() {
        let data = [
            0x01, 0x0F, // Tag ID: 0x010F (Make)
            0x00, 0x02, // Format: 2 (ASCII)
            0x00, 0x00, 0x00, 0x06, // Count: 6
            0x00, 0x00, 0x00, 0x26, // Value/Offset: 0x26
        ];

        let entry = IfdEntry::parse(&data, 0, ByteOrder::BigEndian).unwrap();
        assert_eq!(entry.tag_id, 0x010F);
        assert_eq!(entry.format, TiffFormat::Ascii);
        assert_eq!(entry.count, 6);
        assert_eq!(entry.value_or_offset, 0x26);
        assert!(!entry.is_inline()); // 6 bytes > 4, so not inline
    }
}
