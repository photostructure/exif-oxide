//! Byte order (endianness) handling

use byteorder::{ByteOrder, LittleEndian, BigEndian};

/// Byte order for binary data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    Little,
    Big,
}

impl Endian {
    /// Read u16 from bytes
    pub fn read_u16(&self, data: &[u8]) -> u16 {
        match self {
            Endian::Little => LittleEndian::read_u16(data),
            Endian::Big => BigEndian::read_u16(data),
        }
    }
    
    /// Read u32 from bytes
    pub fn read_u32(&self, data: &[u8]) -> u32 {
        match self {
            Endian::Little => LittleEndian::read_u32(data),
            Endian::Big => BigEndian::read_u32(data),
        }
    }
    
    /// Detect endianness from TIFF header
    pub fn from_tiff_header(data: &[u8]) -> Option<Self> {
        if data.len() < 2 {
            return None;
        }
        
        match &data[0..2] {
            b"II" => Some(Endian::Little),  // Intel byte order
            b"MM" => Some(Endian::Big),     // Motorola byte order
            _ => None,
        }
    }
}