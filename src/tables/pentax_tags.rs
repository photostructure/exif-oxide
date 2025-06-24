//! Pentax maker note tag definitions with print conversions
//!
//! This module defines all Pentax maker note tags extracted from ExifTool's
//! Pentax.pm module, along with their associated print conversion functions.
//! This table-driven approach eliminates the need to port thousands of lines
//! of Perl conversion code.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Pentax.pm"]

use crate::core::print_conv::PrintConvId;

/// Pentax tag definition with print conversion
#[derive(Debug, Clone)]
pub struct PentaxTag {
    pub id: u16,
    pub name: &'static str,
    pub print_conv: PrintConvId,
}

/// Complete Pentax maker note tag table
///
/// Extracted from ExifTool's %Image::ExifTool::Pentax::Main table
pub const PENTAX_TAGS: &[PentaxTag] = &[
    // Core Pentax tags (0x0000-0x0010)
    PentaxTag {
        id: 0x0000,
        name: "PentaxVersion",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0001,
        name: "PentaxModelType",
        print_conv: PrintConvId::PentaxModelLookup,
    },
    PentaxTag {
        id: 0x0002,
        name: "PreviewImageSize",
        print_conv: PrintConvId::ImageSize,
    },
    PentaxTag {
        id: 0x0003,
        name: "PreviewImageLength",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0004,
        name: "PreviewImageStart",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0005,
        name: "PentaxModelID",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0006,
        name: "Date",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0007,
        name: "Time",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0008,
        name: "Quality",
        print_conv: PrintConvId::Quality,
    },
    PentaxTag {
        id: 0x0009,
        name: "PentaxImageSize",
        print_conv: PrintConvId::ImageSize,
    },
    PentaxTag {
        id: 0x000b,
        name: "PictureMode",
        print_conv: PrintConvId::PentaxPictureMode,
    },
    PentaxTag {
        id: 0x000c,
        name: "FlashMode",
        print_conv: PrintConvId::FlashMode,
    },
    PentaxTag {
        id: 0x000d,
        name: "FocusMode",
        print_conv: PrintConvId::FocusMode,
    },
    PentaxTag {
        id: 0x000e,
        name: "AFPointSelected",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x000f,
        name: "AFPointInFocus",
        print_conv: PrintConvId::None,
    },
    // Camera settings (0x0010-0x0020)
    PentaxTag {
        id: 0x0012,
        name: "ExposureTime",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0013,
        name: "FNumber",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0014,
        name: "ISO",
        print_conv: PrintConvId::IsoSpeed,
    },
    PentaxTag {
        id: 0x0016,
        name: "ExposureCompensation",
        print_conv: PrintConvId::ExposureCompensation,
    },
    PentaxTag {
        id: 0x0017,
        name: "MeteringMode",
        print_conv: PrintConvId::MeteringMode,
    },
    PentaxTag {
        id: 0x0018,
        name: "AutoBracketing",
        print_conv: PrintConvId::OnOff,
    },
    PentaxTag {
        id: 0x0019,
        name: "WhiteBalance",
        print_conv: PrintConvId::WhiteBalance,
    },
    PentaxTag {
        id: 0x001a,
        name: "WhiteBalanceMode",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x001d,
        name: "FocalLength",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x001f,
        name: "Saturation",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0020,
        name: "Contrast",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0021,
        name: "Sharpness",
        print_conv: PrintConvId::None,
    },
    // Location and language (0x0022-0x0030)
    PentaxTag {
        id: 0x0022,
        name: "WorldTimeLocation",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0023,
        name: "HometownCity",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0024,
        name: "DestinationCity",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0025,
        name: "HometownDST",
        print_conv: PrintConvId::YesNo,
    },
    PentaxTag {
        id: 0x0026,
        name: "DestinationDST",
        print_conv: PrintConvId::YesNo,
    },
    PentaxTag {
        id: 0x002d,
        name: "Language",
        print_conv: PrintConvId::None,
    },
    // Color settings (0x0030-0x0040)
    PentaxTag {
        id: 0x0032,
        name: "ColorTemperature",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0037,
        name: "ColorSpace",
        print_conv: PrintConvId::None,
    },
    // Lens information (0x003f, 0x0207, 0x03fd)
    PentaxTag {
        id: 0x003f,
        name: "LensType",
        print_conv: PrintConvId::PentaxLensType,
    },
    PentaxTag {
        id: 0x0207,
        name: "LensInfo",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x03fd,
        name: "LensID",
        print_conv: PrintConvId::None,
    },
    // Additional commonly used tags
    PentaxTag {
        id: 0x0039,
        name: "DigitalFilter",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x003e,
        name: "LensInfo",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0040,
        name: "FlashExposureComp",
        print_conv: PrintConvId::ExposureCompensation,
    },
    PentaxTag {
        id: 0x0041,
        name: "ImageTone",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0047,
        name: "CameraTemperature",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x004d,
        name: "FlashDistance",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x004e,
        name: "CameraOrientation",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x004f,
        name: "PreviewImageBorders",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0072,
        name: "EffectiveLV",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x007e,
        name: "ColorTemperature2",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x007f,
        name: "ColorTempDaylight",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0080,
        name: "ColorTempShade",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0081,
        name: "ColorTempCloudy",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0082,
        name: "ColorTempTungsten",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0083,
        name: "ColorTempFluorescentD",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0084,
        name: "ColorTempFluorescentN",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0085,
        name: "ColorTempFluorescentW",
        print_conv: PrintConvId::None,
    },
    PentaxTag {
        id: 0x0086,
        name: "ColorTempFlash",
        print_conv: PrintConvId::None,
    },
];

/// Lookup a Pentax tag by ID
pub fn get_pentax_tag(id: u16) -> Option<&'static PentaxTag> {
    PENTAX_TAGS.iter().find(|tag| tag.id == id)
}

/// Get all Pentax tag IDs
pub fn get_all_pentax_tag_ids() -> Vec<u16> {
    PENTAX_TAGS.iter().map(|tag| tag.id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pentax_tag_lookup() {
        let tag = get_pentax_tag(0x0001).unwrap();
        assert_eq!(tag.name, "PentaxModelType");
        assert_eq!(tag.print_conv, PrintConvId::PentaxModelLookup);
    }

    #[test]
    fn test_pentax_tag_count() {
        // Verify we have a reasonable number of tags
        assert!(PENTAX_TAGS.len() > 30);
        assert!(PENTAX_TAGS.len() < 200); // Sanity check
    }

    #[test]
    fn test_unknown_tag() {
        assert!(get_pentax_tag(0xFFFF).is_none());
    }

    #[test]
    fn test_tag_id_uniqueness() {
        // Verify no duplicate tag IDs
        let mut ids = Vec::new();
        for tag in PENTAX_TAGS {
            assert!(!ids.contains(&tag.id), "Duplicate tag ID: 0x{:04X}", tag.id);
            ids.push(tag.id);
        }
    }
}
