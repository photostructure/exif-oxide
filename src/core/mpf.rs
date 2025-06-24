//! Multi-Picture Format (MPF) parsing
//!
//! MPF is used by modern cameras to store multiple images (previews, thumbnails)
//! in a single JPEG file. It's stored in APP2 segments with "MPF\0" signature.

use crate::core::endian::Endian;
use crate::core::ifd::{IfdParser, TiffHeader};
use crate::core::types::ExifValue;
use crate::error::Result;
use std::collections::HashMap;

/// MPF tag definitions based on CIPA DC-007 specification
pub mod tags {
    pub const MPF_VERSION: u16 = 0xB000;
    pub const NUMBER_OF_IMAGES: u16 = 0xB001;
    pub const MP_IMAGE_LIST: u16 = 0xB002;
    pub const IMAGE_UID_LIST: u16 = 0xB003;
    pub const TOTAL_FRAMES: u16 = 0xB004;
    pub const MP_INDIVIDUAL_NUM: u16 = 0xB101;
    pub const PAN_ORIENTATION: u16 = 0xB201;
    pub const PAN_OVERLAP_H: u16 = 0xB202;
    pub const PAN_OVERLAP_V: u16 = 0xB203;
    pub const BASE_VIEWPOINT_NUM: u16 = 0xB204;
    pub const CONVERGENCE_ANGLE: u16 = 0xB205;
    pub const BASELINE_LENGTH: u16 = 0xB206;
}

/// MPF image types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MpfImageType {
    LargeThumbnailVGA,
    LargeThumbnailFullHD,
    MultiFramePanorama,
    MultiFrameDisparity,
    MultiAngle,
    BaselineMPPrimary,
    OriginalPreservation,
    Unknown(u32),
}

impl From<u32> for MpfImageType {
    fn from(value: u32) -> Self {
        match value {
            0x010001 => MpfImageType::LargeThumbnailVGA,
            0x010002 => MpfImageType::LargeThumbnailFullHD,
            0x020001 => MpfImageType::MultiFramePanorama,
            0x020002 => MpfImageType::MultiFrameDisparity,
            0x020003 => MpfImageType::MultiAngle,
            0x030000 => MpfImageType::BaselineMPPrimary,
            0x040000 => MpfImageType::OriginalPreservation,
            _ => MpfImageType::Unknown(value),
        }
    }
}

/// MPF image entry (16 bytes per entry)
#[derive(Debug, Clone)]
pub struct MpfImageEntry {
    /// Flags (representative, dependent child/parent)
    pub flags: u8,
    /// Image format (0 = JPEG)
    pub format: u8,
    /// Image type
    pub image_type: MpfImageType,
    /// Length of the image data
    pub length: u32,
    /// Offset from the start of MPF segment
    pub offset: u32,
    /// Dependent image 1 entry number
    pub dependent1: u16,
    /// Dependent image 2 entry number
    pub dependent2: u16,
}

/// Parsed MPF data
#[derive(Debug)]
pub struct ParsedMpf {
    /// IFD entries from the MPF
    pub entries: HashMap<u16, ExifValue>,
    /// Parsed image entries from the MPImageList tag
    pub images: Vec<MpfImageEntry>,
    /// Byte order
    pub byte_order: Endian,
}

impl ParsedMpf {
    /// Parse MPF data from raw bytes
    pub fn parse(data: Vec<u8>) -> Result<Self> {
        // Parse TIFF header to get byte order
        let header = TiffHeader::parse(&data)?;
        let byte_order = header.byte_order;

        // MPF uses standard TIFF structure
        let parsed_ifd = IfdParser::parse(data)?;

        // Extract image list if present
        let images = if let Some(ExifValue::Undefined(image_list_data)) =
            parsed_ifd.entries().get(&tags::MP_IMAGE_LIST)
        {
            parse_mp_image_list(image_list_data, byte_order)?
        } else {
            Vec::new()
        };

        Ok(ParsedMpf {
            entries: parsed_ifd.entries().clone(),
            images,
            byte_order,
        })
    }

    /// Get the number of images in this MPF
    pub fn number_of_images(&self) -> Option<u32> {
        match self.entries.get(&tags::NUMBER_OF_IMAGES) {
            Some(ExifValue::U32(n)) => Some(*n),
            Some(ExifValue::U16(n)) => Some(*n as u32),
            _ => None,
        }
    }

    /// Get a specific tag value
    pub fn get_tag(&self, tag_id: u16) -> Option<&ExifValue> {
        self.entries.get(&tag_id)
    }
}

/// Parse the MP Image List (16 bytes per entry)
fn parse_mp_image_list(data: &[u8], byte_order: Endian) -> Result<Vec<MpfImageEntry>> {
    let mut images = Vec::new();
    let entry_size = 16;
    let num_entries = data.len() / entry_size;

    for i in 0..num_entries {
        let offset = i * entry_size;
        if offset + entry_size > data.len() {
            break;
        }

        let entry_data = &data[offset..offset + entry_size];

        // Parse 4-byte attribute field
        let attributes = byte_order.read_u32(&entry_data[0..4]);
        let flags = ((attributes >> 24) & 0xFF) as u8;
        let format = ((attributes >> 16) & 0xFF) as u8;
        let image_type = MpfImageType::from(attributes & 0xFFFFFF);

        // Parse rest of the entry
        let length = byte_order.read_u32(&entry_data[4..8]);
        let offset = byte_order.read_u32(&entry_data[8..12]);
        let dependent1 = byte_order.read_u16(&entry_data[12..14]);
        let dependent2 = byte_order.read_u16(&entry_data[14..16]);

        images.push(MpfImageEntry {
            flags,
            format,
            image_type,
            length,
            offset,
            dependent1,
            dependent2,
        });
    }

    Ok(images)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mpf_image_type() {
        assert_eq!(
            MpfImageType::from(0x010001),
            MpfImageType::LargeThumbnailVGA
        );
        assert_eq!(
            MpfImageType::from(0x010002),
            MpfImageType::LargeThumbnailFullHD
        );
        assert_eq!(
            MpfImageType::from(0x999999),
            MpfImageType::Unknown(0x999999)
        );
    }
}
