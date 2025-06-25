//! Hasselblad maker note tag definitions
//!
//! Based on ExifTool's MakerNotes.pm comments for known Hasselblad tags.
//! Hasselblad uses a simple IFD structure with only a few documented tags.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/MakerNotes.pm"]

use crate::core::print_conv::PrintConvId;

/// Hasselblad maker note tag definition
#[derive(Debug, Clone)]
pub struct HasselbladTag {
    pub id: u16,
    pub name: &'static str,
    pub print_conv: PrintConvId,
}

/// Hasselblad maker note tag table
/// Based on comments in ExifTool's MakerNotes.pm
pub const HASSELBLAD_TAGS: &[HasselbladTag] = &[
    HasselbladTag {
        id: 0x0011,
        name: "SensorCode",
        print_conv: PrintConvId::None, // Raw value, no conversion
    },
    HasselbladTag {
        id: 0x0012,
        name: "CameraModelID",
        print_conv: PrintConvId::None, // Raw value, no conversion
    },
    HasselbladTag {
        id: 0x0015,
        name: "CameraModelName",
        print_conv: PrintConvId::None, // String value, no conversion needed
    },
    HasselbladTag {
        id: 0x0016,
        name: "CoatingCode",
        print_conv: PrintConvId::None, // Raw value, no conversion
    },
];

/// Get Hasselblad tag definition by ID
pub fn get_hasselblad_tag(id: u16) -> Option<&'static HasselbladTag> {
    HASSELBLAD_TAGS.iter().find(|tag| tag.id == id)
}

/// Get all Hasselblad tag IDs
pub fn get_hasselblad_tag_ids() -> Vec<u16> {
    HASSELBLAD_TAGS.iter().map(|tag| tag.id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hasselblad_tag_lookup() {
        // Test known tags
        assert!(get_hasselblad_tag(0x0011).is_some());
        assert_eq!(get_hasselblad_tag(0x0011).unwrap().name, "SensorCode");

        assert!(get_hasselblad_tag(0x0012).is_some());
        assert_eq!(get_hasselblad_tag(0x0012).unwrap().name, "CameraModelID");

        assert!(get_hasselblad_tag(0x0015).is_some());
        assert_eq!(get_hasselblad_tag(0x0015).unwrap().name, "CameraModelName");

        assert!(get_hasselblad_tag(0x0016).is_some());
        assert_eq!(get_hasselblad_tag(0x0016).unwrap().name, "CoatingCode");

        // Test unknown tag
        assert!(get_hasselblad_tag(0x9999).is_none());
    }

    #[test]
    fn test_hasselblad_tag_count() {
        assert_eq!(HASSELBLAD_TAGS.len(), 4);
        let ids = get_hasselblad_tag_ids();
        assert_eq!(ids.len(), 4);
        assert!(ids.contains(&0x0011));
        assert!(ids.contains(&0x0012));
        assert!(ids.contains(&0x0015));
        assert!(ids.contains(&0x0016));
    }
}
