//! GIF file format processing
//!
//! This module handles GIF (Graphics Interchange Format) file processing,
//! extracting metadata from GIF Logical Screen Descriptor following ExifTool's implementation.
//!
//! Reference: third-party/exiftool/lib/Image/ExifTool/GIF.pm

use crate::types::{Result, TagEntry, TagValue};

/// GIF file signatures
const GIF87A_SIGNATURE: &[u8] = b"GIF87a";
const GIF89A_SIGNATURE: &[u8] = b"GIF89a";

/// GIF Logical Screen Descriptor data structure
/// ExifTool reference: GIF.pm:105-138 (Screen table)
#[derive(Debug, Clone)]
pub struct ScreenDescriptor {
    pub image_width: u16,
    pub image_height: u16,
    pub flags: u8,
    pub background_color: u8,
    pub pixel_aspect_ratio: u8,
}

impl ScreenDescriptor {
    /// Check if image has a global color map
    /// ExifTool reference: GIF.pm:118-121 (HasColorMap)
    pub fn has_color_map(&self) -> bool {
        (self.flags & 0x80) != 0
    }

    /// Get color resolution depth
    /// ExifTool reference: GIF.pm:122-126 (ColorResolutionDepth)
    pub fn color_resolution_depth(&self) -> u8 {
        ((self.flags & 0x70) >> 4) + 1
    }

    /// Get bits per pixel
    /// ExifTool reference: GIF.pm:127-131 (BitsPerPixel)
    pub fn bits_per_pixel(&self) -> u8 {
        (self.flags & 0x07) + 1
    }

    /// Get pixel aspect ratio as a float
    /// ExifTool reference: GIF.pm:134-137 (PixelAspectRatio)
    pub fn pixel_aspect_ratio_float(&self) -> Option<f32> {
        if self.pixel_aspect_ratio == 0 {
            None
        } else {
            Some((self.pixel_aspect_ratio as f32 + 15.0) / 64.0)
        }
    }
}

/// Parse GIF Logical Screen Descriptor to extract image dimensions and properties
///
/// GIF file structure:
/// - GIF signature: 6 bytes ("GIF87a" or "GIF89a")
/// - Logical Screen Descriptor: 7 bytes
///   - Width: 2 bytes little-endian (u16)
///   - Height: 2 bytes little-endian (u16)
///   - Flags: 1 byte (global color table flag, color resolution, sort flag, global color table size)
///   - Background Color Index: 1 byte
///   - Pixel Aspect Ratio: 1 byte
///
/// ExifTool reference: GIF.pm:105-138 (Screen descriptor processing)
pub fn parse_gif_screen_descriptor(data: &[u8]) -> Result<ScreenDescriptor> {
    // Verify minimum file length for signature + screen descriptor
    if data.len() < 13 {
        return Err(crate::types::ExifError::InvalidFormat(
            "GIF file too short for signature and screen descriptor".to_string(),
        ));
    }

    // Verify GIF signature
    let signature = &data[..6];
    if signature != GIF87A_SIGNATURE && signature != GIF89A_SIGNATURE {
        return Err(crate::types::ExifError::InvalidFormat(format!(
            "Invalid GIF signature: expected GIF87a or GIF89a, got {}",
            String::from_utf8_lossy(signature)
        )));
    }

    // Parse Logical Screen Descriptor (7 bytes starting at offset 6)
    // ExifTool reference: GIF.pm:109-116 (ImageWidth at offset 0, ImageHeight at offset 2)
    let screen_desc_start = 6;

    // Extract dimensions - little-endian u16 values
    // ExifTool uses Format => 'int16u' which is little-endian
    let image_width = u16::from_le_bytes([data[screen_desc_start], data[screen_desc_start + 1]]);
    let image_height =
        u16::from_le_bytes([data[screen_desc_start + 2], data[screen_desc_start + 3]]);

    // Extract additional screen descriptor fields
    let flags = data[screen_desc_start + 4];
    let background_color = data[screen_desc_start + 5];
    let pixel_aspect_ratio = data[screen_desc_start + 6];

    Ok(ScreenDescriptor {
        image_width,
        image_height,
        flags,
        background_color,
        pixel_aspect_ratio,
    })
}

/// Create GIF TagEntry objects from Screen Descriptor data
///
/// Following ExifTool's GIF group assignment and tag naming conventions.
/// ExifTool assigns GIF metadata to "GIF" group, not "File" group.
///
/// ExifTool reference: GIF.pm:105-138 (Screen table with GROUPS => { 2 => 'Image' })
pub fn create_gif_tag_entries(screen_desc: &ScreenDescriptor) -> Vec<TagEntry> {
    let mut entries = Vec::new();

    // GIF:ImageWidth - ExifTool GIF.pm:109-112
    entries.push(TagEntry {
        group: "GIF".to_string(),
        group1: "GIF".to_string(),
        name: "ImageWidth".to_string(),
        value: TagValue::U16(screen_desc.image_width),
        print: TagValue::U16(screen_desc.image_width),
    });

    // GIF:ImageHeight - ExifTool GIF.pm:113-116
    entries.push(TagEntry {
        group: "GIF".to_string(),
        group1: "GIF".to_string(),
        name: "ImageHeight".to_string(),
        value: TagValue::U16(screen_desc.image_height),
        print: TagValue::U16(screen_desc.image_height),
    });

    // GIF:HasColorMap - ExifTool GIF.pm:117-121 (with PrintConv)
    entries.push(TagEntry {
        group: "GIF".to_string(),
        group1: "GIF".to_string(),
        name: "HasColorMap".to_string(),
        value: TagValue::String(
            if screen_desc.has_color_map() {
                "1"
            } else {
                "0"
            }
            .to_string(),
        ),
        print: TagValue::String(
            if screen_desc.has_color_map() {
                "Yes"
            } else {
                "No"
            }
            .to_string(),
        ),
    });

    // GIF:ColorResolutionDepth - ExifTool GIF.pm:122-126
    entries.push(TagEntry {
        group: "GIF".to_string(),
        group1: "GIF".to_string(),
        name: "ColorResolutionDepth".to_string(),
        value: TagValue::String(screen_desc.color_resolution_depth().to_string()),
        print: TagValue::String(screen_desc.color_resolution_depth().to_string()),
    });

    // GIF:BitsPerPixel - ExifTool GIF.pm:127-131
    entries.push(TagEntry {
        group: "GIF".to_string(),
        group1: "GIF".to_string(),
        name: "BitsPerPixel".to_string(),
        value: TagValue::U8(screen_desc.bits_per_pixel()),
        print: TagValue::U8(screen_desc.bits_per_pixel()),
    });

    // GIF:BackgroundColor - ExifTool GIF.pm:132
    entries.push(TagEntry {
        group: "GIF".to_string(),
        group1: "GIF".to_string(),
        name: "BackgroundColor".to_string(),
        value: TagValue::String(screen_desc.background_color.to_string()),
        print: TagValue::String(screen_desc.background_color.to_string()),
    });

    // GIF:PixelAspectRatio - ExifTool GIF.pm:133-137 (with conversion)
    if let Some(aspect_ratio) = screen_desc.pixel_aspect_ratio_float() {
        entries.push(TagEntry {
            group: "GIF".to_string(),
            group1: "GIF".to_string(),
            name: "PixelAspectRatio".to_string(),
            value: TagValue::String(screen_desc.pixel_aspect_ratio.to_string()),
            print: TagValue::String(format!("{:.3}", aspect_ratio)),
        });
    }

    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gif_signature_validation() {
        let gif87a = b"GIF87a\x00\x00\x00\x00\x00\x00\x00";
        let gif89a = b"GIF89a\x00\x00\x00\x00\x00\x00\x00";
        let invalid = b"PNG\x00\x00\x00\x00\x00\x00\x00\x00\x00";

        assert!(parse_gif_screen_descriptor(gif87a).is_ok());
        assert!(parse_gif_screen_descriptor(gif89a).is_ok());
        assert!(parse_gif_screen_descriptor(invalid).is_err());
    }

    #[test]
    fn test_screen_descriptor_parsing() {
        // Create test GIF data: GIF89a + 663x475 dimensions + flags
        let test_data = [
            b'G', b'I', b'F', b'8', b'9', b'a', // GIF89a signature
            0x97, 0x02, // Width: 663 (little-endian)
            0xDB, 0x01, // Height: 475 (little-endian)
            0xF7, // Flags: 0xF7 (has color map, 8-bit color resolution, 256 colors)
            0xFF, // Background color: 255
            0x00, // Pixel aspect ratio: 0 (undefined)
        ];

        let result = parse_gif_screen_descriptor(&test_data).unwrap();

        assert_eq!(result.image_width, 663);
        assert_eq!(result.image_height, 475);
        assert_eq!(result.flags, 0xF7);
        assert_eq!(result.background_color, 255);
        assert_eq!(result.pixel_aspect_ratio, 0);

        // Test flag parsing
        assert!(result.has_color_map());
        assert_eq!(result.color_resolution_depth(), 8);
        assert_eq!(result.bits_per_pixel(), 8);
        assert_eq!(result.pixel_aspect_ratio_float(), None);
    }

    #[test]
    fn test_create_gif_tag_entries() {
        let screen_desc = ScreenDescriptor {
            image_width: 663,
            image_height: 475,
            flags: 0xF7,
            background_color: 255,
            pixel_aspect_ratio: 0,
        };

        let entries = create_gif_tag_entries(&screen_desc);

        // Should have 6 or 7 entries depending on pixel aspect ratio
        assert!(entries.len() >= 6);

        // Check ImageWidth entry
        let width_entry = entries.iter().find(|e| e.name == "ImageWidth").unwrap();
        assert_eq!(width_entry.group, "GIF");
        assert_eq!(width_entry.value, TagValue::U16(663));

        // Check ImageHeight entry
        let height_entry = entries.iter().find(|e| e.name == "ImageHeight").unwrap();
        assert_eq!(height_entry.group, "GIF");
        assert_eq!(height_entry.value, TagValue::U16(475));

        // Check HasColorMap entry (should have PrintConv applied)
        let color_map_entry = entries.iter().find(|e| e.name == "HasColorMap").unwrap();
        assert_eq!(color_map_entry.print, TagValue::String("Yes".to_string()));
    }
}
