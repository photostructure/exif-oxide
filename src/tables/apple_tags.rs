//! Apple tag table with PrintConv mappings
//!
//! EXIFTOOL-SOURCE: lib/Image/ExifTool/Apple.pm
//!
//! This file implements table-driven PrintConv for Apple maker notes,
//! using existing universal PrintConv functions where possible and adding
//! Apple-specific functions only when needed.

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Apple.pm"]

use crate::core::print_conv::PrintConvId;

#[derive(Debug, Clone)]
pub struct AppleTag {
    pub id: u16,
    pub name: &'static str,
    pub print_conv: PrintConvId,
}

/// Apple tag definitions with PrintConv mappings
/// Using existing universal PrintConv functions where ExifTool patterns match
pub const APPLE_TAGS: &[AppleTag] = &[
    AppleTag {
        id: 0x0001,
        name: "MakerNoteVersion",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x0002,
        name: "AEMatrix",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x0003,
        name: "RunTime",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x0004,
        name: "AEStable",
        print_conv: PrintConvId::YesNo,
    }, // 0=No, 1=Yes
    AppleTag {
        id: 0x0005,
        name: "AETarget",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x0006,
        name: "AEAverage",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x0007,
        name: "AFStable",
        print_conv: PrintConvId::YesNo,
    }, // 0=No, 1=Yes
    AppleTag {
        id: 0x0008,
        name: "AccelerationVector",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x000a,
        name: "HDRImageType",
        print_conv: PrintConvId::AppleHDRImageType,
    },
    AppleTag {
        id: 0x000b,
        name: "BurstUUID",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x000c,
        name: "FocusDistanceRange",
        print_conv: PrintConvId::AppleFocusDistanceRange,
    },
    AppleTag {
        id: 0x000f,
        name: "OISMode",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x0011,
        name: "ContentIdentifier",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x0014,
        name: "ImageCaptureType",
        print_conv: PrintConvId::AppleImageCaptureType,
    },
    AppleTag {
        id: 0x0015,
        name: "ImageUniqueID",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x0017,
        name: "LivePhotoVideoIndex",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x0019,
        name: "ImageProcessingFlags",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x001a,
        name: "QualityHint",
        print_conv: PrintConvId::UniversalQualityBasic,
    },
    AppleTag {
        id: 0x001d,
        name: "LuminanceNoiseAmplitude",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x001f,
        name: "PhotosAppFeatureFlags",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x0020,
        name: "ImageCaptureRequestID",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x0021,
        name: "HDRHeadroom",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x0023,
        name: "AFPerformance",
        print_conv: PrintConvId::AppleAFPerformance,
    },
    AppleTag {
        id: 0x0025,
        name: "SceneFlags",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x0026,
        name: "SignalToNoiseRatioType",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x0027,
        name: "SignalToNoiseRatio",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x002b,
        name: "PhotoIdentifier",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x002d,
        name: "ColorTemperature",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x002e,
        name: "CameraType",
        print_conv: PrintConvId::AppleCameraType,
    },
    AppleTag {
        id: 0x002f,
        name: "FocusPosition",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x0030,
        name: "HDRGain",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x0038,
        name: "AFMeasuredDepth",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x003d,
        name: "AFConfidence",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x003e,
        name: "ColorCorrectionMatrix",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x003f,
        name: "GreenGhostMitigationStatus",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x0040,
        name: "SemanticStyle",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x0041,
        name: "SemanticStyleRenderingVer",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x0042,
        name: "SemanticStylePreset",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x004e,
        name: "Apple_0x004e",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x004f,
        name: "Apple_0x004f",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x0054,
        name: "Apple_0x0054",
        print_conv: PrintConvId::None,
    },
    AppleTag {
        id: 0x005a,
        name: "Apple_0x005a",
        print_conv: PrintConvId::None,
    },
];

/// Get Apple tag by ID
pub fn get_apple_tag(tag_id: u16) -> Option<&'static AppleTag> {
    APPLE_TAGS.iter().find(|tag| tag.id == tag_id)
}
