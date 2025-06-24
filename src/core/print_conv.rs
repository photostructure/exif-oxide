//! Print conversion functions for EXIF tag values
//!
//! This module provides a table-driven system for converting raw EXIF values
//! into human-readable strings, matching ExifTool's PrintConv functionality.
//! Instead of porting thousands of lines of Perl conversion code, we identify
//! common patterns and create reusable conversion functions.

use crate::core::ExifValue;

/// Enumeration of all print conversion functions
///
/// This enum captures all unique PrintConv patterns found across ExifTool's
/// manufacturer modules. By cataloging these patterns, we can reuse conversion
/// logic across all manufacturers instead of duplicating it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)] // Allow ExifTool-style naming (e.g., WB_RGGBLevels)
pub enum PrintConvId {
    /// No conversion - return raw value as string
    None,

    /// Simple on/off conversion (0=Off, 1=On)
    OnOff,

    /// Yes/No conversion (0=No, 1=Yes)  
    YesNo,

    /// Image size conversion (width x height)
    ImageSize,

    /// Quality settings (1=Best, 2=Better, 3=Good, etc.)
    Quality,

    /// Flash mode lookup
    FlashMode,

    /// Focus mode lookup  
    FocusMode,

    /// White balance lookup
    WhiteBalance,

    /// Metering mode lookup
    MeteringMode,

    /// ISO speed conversion
    IsoSpeed,

    /// Exposure compensation (+/- EV)
    ExposureCompensation,

    /// Pentax-specific conversions
    PentaxModelLookup,
    PentaxPictureMode,
    PentaxLensType,

    /// Canon-specific conversions  
    CanonCameraSettings,
    CanonImageType,
    CanonLensType,
    CanonModelLookup,
    CanonUserDefPictureStyle, // Shared variant (consolidates userDefStyles references)
    CanonPictureStyle,        // Shared variant (consolidates pictureStyles references)

    // Canon tag-specific conversions (generated variants)
    CanonCanonCameraSettings,
    CanonCanonFocalLength,
    CanonCanonFlashInfo,
    CanonCanonShotInfo,
    CanonCanonPanorama,
    CanonCanonImageType,
    CanonCanonFirmwareVersion,
    CanonFileNumber,
    CanonOwnerName,
    CanonUnknownD30,
    CanonCanonFileLength,
    CanonMovieInfo,
    CanonCanonAFInfo,
    CanonThumbnailImageValidArea,
    CanonSerialNumberFormat,
    CanonSuperMacro,
    CanonDateStampMode,
    CanonMyColors,
    CanonFirmwareRevision,
    CanonCategories,
    CanonFaceDetect1,
    CanonFaceDetect2,
    CanonCanonAFInfo2,
    CanonContrastInfo,
    CanonImageUniqueID,
    CanonWBInfo,
    CanonFaceDetect3,
    CanonTimeInfo,
    CanonBatteryType,
    CanonAFInfo3,
    CanonRawDataOffset,
    CanonRawDataLength,
    CanonOriginalDecisionDataOffset,
    CanonCustomFunctions1D,
    CanonPersonalFunctions,
    CanonPersonalFunctionValues,
    CanonCanonFileInfo,
    CanonAFPointsInFocus1D,
    CanonDustRemovalData,
    CanonCropInfo,
    CanonCustomFunctions2,
    CanonAspectInfo,
    CanonProcessingInfo,
    CanonToneCurveTable,
    CanonSharpnessTable,
    CanonSharpnessFreqTable,
    CanonColorBalance,
    CanonMeasuredColor,
    CanonColorTemperature,
    CanonCanonFlags,
    CanonModifiedInfo,
    CanonToneCurveMatching,
    CanonColorSpace,
    CanonPreviewImageInfo,
    CanonVRDOffset,
    CanonSensorInfo,
    CanonCRWParam,
    CanonColorInfo,
    CanonFlavor,
    CanonPictureStyleUserDef,
    CanonPictureStylePC,
    CanonCustomPictureStyleFileName,
    CanonAFMicroAdj,
    CanonVignettingCorr2,
    CanonLightingOpt,
    CanonAmbienceInfo,
    CanonMultiExp,
    CanonFilterInfo,
    CanonHDRInfo,
    CanonLogInfo,
    CanonAFConfig,
    CanonRawBurstModeRoll,
    CanonLevelInfo,

    /// Nikon-specific conversions
    NikonLensType,
    NikonFlashMode,

    // Auto-generated Nikon PrintConvId variants (from extract printconv-tables Nikon.pm)
    NikonMakerNoteVersion,
    NikonISO,
    NikonColorMode,
    NikonSharpness,
    NikonFocusMode,
    NikonFlashSetting,
    NikonFlashType,
    NikonWB_RBLevels,
    NikonProgramShift,
    NikonExposureDifference,
    NikonISOSelection,
    NikonDataDump,
    NikonPreviewIFD,
    NikonFlashExposureComp,
    NikonISOSetting,
    NikonImageBoundary,
    NikonExternalFlashExposureComp,
    NikonFlashExposureBracketValue,
    NikonExposureBracketValue,
    NikonImageProcessing,
    NikonCropHiSpeed,
    NikonExposureTuning,
    NikonSerialNumber,
    NikonColorSpace,
    NikonVRInfo,
    NikonFaceDetect,
    #[allow(non_camel_case_types)]
    NikonActiveD_Lighting, // Using underscore instead of hyphen for valid Rust identifier
    NikonWorldTime,
    NikonISOInfo,
    NikonVignetteControl,
    NikonDistortInfo,
    NikonUnknownInfo,
    NikonUnknownInfo2,
    NikonShutterMode,
    NikonMechanicalShutterCount,
    NikonLocationInfo,
    NikonBlackLevel,
    NikonImageSizeRAW,
    NikonJPGCompression,
    NikonCropArea,
    NikonNikonSettings,
    NikonColorTemperatureAuto,
    NikonBarometerInfo,
    NikonNikonICCProfile,
    NikonContrastCurve,
    NikonToningEffect,
    NikonNEFBitDepth,
    NikonVariProgram,
    NikonPrintIM,
    NikonRetouchInfo,
    NikonShutterCount,
    NikonRawImageCenter,
    NikonMakerNotes0x51,
    NikonMakerNotes0x56,
    NikonNEFLinearizationTable,
    NikonAFResponse,
    NikonSensorPixelSize,
    NikonNikonCaptureOffsets,
    NikonPowerUpTime,
    NikonAFTune,
    NikonDeletedImageCount,
    NikonNikonCaptureOutput,
    NikonShootingMode,
    NikonSceneMode,
    NikonPictureControlData,
    NikonNikonCaptureData,
    NikonImageDataSize,
    NikonNikonCaptureVersion,
    NikonNikonScanIFD,
    NikonNefCompression,
    NikonManualFocusDistance,
    NikonDateStampMode,
    NikonImageAdjustment,
    NikonSaturation,
    NikonImageStabilization,
    NikonLightSource,
    NikonImageOptimization,
    NikonImageCount,
    NikonToneComp,
    NikonNoiseReduction,
    NikonSaturationAdj,
    NikonRetouchHistory,
    NikonDigitalZoom,
    NikonHueAdjustment,
    NikonHighISONoiseReduction,
    NikonColorHue,

    /// Sony-specific conversions
    SonyLensType,
    SonySceneMode,

    /// Olympus-specific conversions (generated variants)
    OlympusMakerNoteVersion,
    OlympusMinoltaCameraSettingsOld,
    OlympusMinoltaCameraSettings,
    OlympusCompressedImageSize,
    OlympusPreviewImageData,
    OlympusPreviewImageStart,
    OlympusPreviewImageLength,
    OlympusThumbnailImage,
    OlympusBodyFirmwareVersion,
    OlympusSpecialMode,
    OlympusJPEGQual,
    OlympusMacro,
    OlympusBWMode,
    OlympusDigitalZoom,
    OlympusFocalPlaneDiagonal,
    OlympusLensDistortionParams,
    OlympusCameraType,
    OlympusCameraID,
    OlympusOneTouchWB,
    OlympusShutterSpeedValue,
    OlympusISOValue,
    OlympusApertureValue,
    OlympusBrightnessValue,
    OlympusFlashMode,
    OlympusFlashDevice,
    OlympusExposureCompensation,
    OlympusSensorTemperature,
    OlympusLensTemperature,
    OlympusLightSource,
    OlympusFocusRange,
    OlympusFocusMode,
    OlympusManualFocusDistance,
    OlympusZoomStepCount,
    OlympusFocusStepCount,
    OlympusSharpness,
    OlympusFlashChargeLevel,
    OlympusColorMatrix,
    OlympusBlackLevel,
    OlympusWhiteBalance,
    OlympusBlueBalance,
    OlympusRedBalance,
    OlympusColorMatrixNumber,
    OlympusSerialNumber,
    OlympusExternalFlashAE1_0,
    OlympusExternalFlashAE2_0,
    OlympusInternalFlashAE1_0,
    OlympusInternalFlashAE2_0,
    OlympusFlashExposureComp,
    OlympusAutoExposureLock,
    OlympusAutoWhiteBalanceLock,
    OlympusAutoFocus,
    OlympusNoiseReduction,
    OlympusColorControl,
    OlympusValidBits,
    OlympusCoringFilter,
    OlympusCoringValue,
    OlympusImageWidth,
    OlympusImageHeight,
    OlympusOriginalManufacturer,
    OlympusDataDump,
    OlympusDataDump2,
    OlympusZoomedPreviewStart,
    OlympusZoomedPreviewLength,
    OlympusZoomedPreviewSize,
    OlympusPreviewFormat,
    OlympusSceneDetect,
    OlympusSceneArea,
    OlympusSceneDetectData,
    OlympusCompressionRatio,
    OlympusPreviewImageValid,
    OlympusPreviewImageStart2,
    OlympusPreviewImageLength2,
    OlympusAFResult,
    OlympusCCDScanMode,
    OlympusNoiseFilter,
    OlympusArtFilter,
    OlympusMagicFilter,
    OlympusPictureMode,
    OlympusPictureModeContrast,
    OlympusPictureModeSaturation,
    OlympusPictureModeSharpness,
    OlympusPictureModeBWFilter,
    OlympusPictureModeTone,
    OlympusPictureModeEffect,
    OlympusColorTemperatureRG,
    OlympusColorTemperatureBG,
    OlympusContrast,

    // Additional Olympus variants (missing from generated list)
    OlympusLensType,
    OlympusTextInfo,
    OlympusEpsonImageWidth,
    OlympusEpsonImageHeight,
    OlympusEpsonSoftware,
    OlympusPreCaptureFrames,
    OlympusWhiteBoard,
    OlympusSensorArea,
    OlympusFirmware,
    OlympusPrintIM,
    OlympusLightCondition,
    OlympusWBMode,
    OlympusExternalFlashAE1,
    OlympusExternalFlashAE2,
    OlympusInternalFlashAE1,
    OlympusInternalFlashAE2,
    OlympusInternalFlashTable,
    OlympusExternalFlashGValue,
    OlympusExternalFlashZoom,
    OlympusSharpnessFactor,
    OlympusOlympusImageWidth,
    OlympusOlympusImageHeight,
    OlympusSceneMode,
    OlympusFocusStepInfinity,
    OlympusFocusStepNear,
    OlympusLightValueCenter,
    OlympusLightValuePeriphery,
    OlympusFieldCount,

    /// Pentax-specific conversions (optimized with shared lookups)  
    PentaxFNumber, // Shared aperture formatting (sprintf "%.1f")
    PentaxExposureTime,      // Shared exposure time formatting
    PentaxSensitivityAdjust, // Shared exposure compensation formatting (sprintf "%+.1f")

    // Individual Pentax PrintConv variants (generated)
    PentaxPentaxVersion,
    PentaxPreviewImageSize,
    PentaxPreviewImageLength,
    PentaxPreviewImageStart,
    PentaxDate,
    PentaxTime,
    PentaxPentaxImageSize,
    PentaxFocusPosition,
    PentaxISO,
    PentaxLightReading,
    PentaxBlueBalance,
    PentaxRedBalance,
    PentaxDigitalZoom,
    PentaxSaturation,
    PentaxContrast,
    PentaxSharpness,
    PentaxWorldTimeLocation,
    PentaxHometownCity,
    PentaxDestinationCity,
    PentaxHometownDST,
    PentaxDestinationDST,
    PentaxDSPFirmwareVersion,
    PentaxCPUFirmwareVersion,
    PentaxFrameNumber,
    PentaxImageEditing,
    PentaxDriveMode,
    PentaxSensorSize,
    PentaxColorSpace,
    PentaxImageAreaOffset,
    PentaxRawImageSize,
    PentaxAFPointsInFocus,
    PentaxDataScaling,
    PentaxPreviewImageBorders,
    PentaxImageEditCount,
    PentaxCameraTemperature,
    PentaxImageTone,
    PentaxColorTemperature,
    PentaxColorTempDaylight,
    PentaxColorTempShade,
    PentaxColorTempCloudy,
    PentaxColorTempTungsten,
    PentaxColorTempFluorescentD,
    PentaxColorTempFluorescentN,
    PentaxColorTempFluorescentW,
    PentaxColorTempFlash,
    PentaxShutterCount,
    PentaxFaceInfo,
    PentaxRawDevelopmentProcess,
    PentaxHue,
    PentaxAWBInfo,
    PentaxDynamicRangeExpansion,
    PentaxTimeInfo,
    PentaxHighLowKeyAdj,
    PentaxContrastHighlight,
    PentaxContrastShadow,
    PentaxHighISONoiseReduction,
    PentaxAFAdjustment,
    PentaxMonochromeFilterEffect,
    PentaxMonochromeToning,
    PentaxFaceDetect,
    PentaxFaceDetectFrameSize,
    PentaxISOAutoMinSpeed,
    PentaxCrossProcess,
    PentaxWhiteLevel,
    PentaxBleachBypassToning,
    PentaxAspectRatio,
    PentaxBlurControl,
    PentaxHDR,
    PentaxShutterType,
    PentaxIntervalShooting,
    PentaxClarityControl,
    PentaxBlackPoint,
    PentaxWhitePoint,
    PentaxColorMatrixA,
    PentaxColorMatrixB,
    PentaxWB_RGGBLevelsDaylight,
    PentaxWB_RGGBLevelsShade,
    PentaxWB_RGGBLevelsCloudy,
    PentaxWB_RGGBLevelsTungsten,
    PentaxWB_RGGBLevelsFluorescentD,
    PentaxWB_RGGBLevelsFluorescentN,
    PentaxWB_RGGBLevelsFluorescentW,
    PentaxWB_RGGBLevelsFlash,
    PentaxCameraInfo,
    PentaxBatteryInfo,
    PentaxSaturationInfo,
    PentaxColorMatrixA2,
    PentaxColorMatrixB2,
    PentaxAFInfo,
    PentaxHuffmanTable,
    PentaxKelvinWB,
    PentaxColorInfo,
    PentaxEVStepInfo,
    PentaxShotInfo,
    PentaxFacePos,
    PentaxFaceSize,
    PentaxSerialNumber,
    PentaxWBLevels,
    PentaxArtist,
    PentaxCopyright,
    PentaxFirmwareVersion,
    PentaxContrastDetectAFArea,
    PentaxCrossProcessParams,
    PentaxPixelShiftInfo,
    PentaxAFPointInfo,
    PentaxDataDump,
    PentaxToneCurve,
    PentaxToneCurves,
    PentaxUnknownBlock,
    PentaxPrintIM,

    /// Sony-specific conversions (generated variants)
    SonyDynamicRangeOptimizer_9401,
    SonyHDR,
    SonySony_0x940f,
    SonyFlashExposureComp,
    SonyShadows,
    SonyPreviewImage,
    SonyWBShiftAB_GM_Precise,
    SonySony_0x940d,
    SonyAFIlluminator,
    SonyTag202a,
    SonyMultiBurstImageWidth,
    SonySharpnessRange,
    SonyFocusLocation2,
    SonyPrintIM,
    SonyRAWFileType,
    SonyAnti_Blur,
    SonyExposureMode,
    SonyFocusLocation,
    SonyColorTemperature,
    SonyLateralChromaticAberration,
    SonyMultiFrameNREffect,
    SonyTag9401,
    SonyRating,
    SonyExposureStandardAdjustment,
    SonySequenceNumber,
    SonyHighISONoiseReduction2,
    SonyWBShiftAB_GM,
    SonyHighISONoiseReduction,
    SonySony_0x940b,
    SonyDistortionCorrectionSetting,
    SonyMinoltaMakerNote,
    SonyMultiBurstImageHeight,
    SonyShotInfo,
    SonyTag9403,
    SonyPictureEffect,
    SonyFlashAction,
    SonyPixelShiftInfo,
    SonySerialNumber,
    SonySharpness,
    SonyContrast,
    SonyJPEG_HEIFSwitch,
    SonySony_0x9407,
    SonyReleaseMode,
    SonyVignettingCorrection,
    SonySony_0x9411,
    SonyHiddenInfo,
    SonyPanorama,
    SonyAFTracking,
    SonyFlexibleSpotPosition,
    SonyLongExposureNoiseReduction,
    SonyBrightness,
    SonyModelLookup,
    SonyImage,
    SonyColorCompensationFilter,
    SonyVariableLowPassFilter,
    SonyFocusFrameSize,
    SonyDynamicRangeOptimizer,
    SonySonyLensTypes,
    SonySony_0x9408,
    SonySony_0x9409,
    SonyPrioritySetInAWB,
    SonyFocusMode,
    SonyFade,
    SonyFileFormat,
    SonyCreativeStyle,
    SonySaturation,
    SonyZoneMatching,
    SonyPreviewImageSize,
    SonyHighlights,
    SonyClarity,
    SonySony_0x9416,
    SonyFlashLevel,
    SonySoftSkinEffect,
    SonyFullImageSize,
    SonyTag900b,
}

/// Apply print conversion to an EXIF value
pub fn apply_print_conv(value: &ExifValue, conv_id: PrintConvId) -> String {
    match conv_id {
        PrintConvId::None => exif_value_to_string(value),

        PrintConvId::OnOff => match as_u32(value) {
            Some(0) => "Off".to_string(),
            Some(1) => "On".to_string(),
            _ => exif_value_to_string(value),
        },

        PrintConvId::YesNo => match as_u32(value) {
            Some(0) => "No".to_string(),
            Some(1) => "Yes".to_string(),
            _ => exif_value_to_string(value),
        },

        PrintConvId::ImageSize => {
            // Handle different value formats for image size
            match value {
                ExifValue::U16Array(arr) if arr.len() >= 2 => {
                    format!("{}x{}", arr[0], arr[1])
                }
                ExifValue::U32Array(arr) if arr.len() >= 2 => {
                    format!("{}x{}", arr[0], arr[1])
                }
                _ => exif_value_to_string(value),
            }
        }

        PrintConvId::Quality => match as_u32(value) {
            Some(1) => "Best".to_string(),
            Some(2) => "Better".to_string(),
            Some(3) => "Good".to_string(),
            Some(4) => "Normal".to_string(),
            Some(5) => "Economy".to_string(),
            _ => exif_value_to_string(value),
        },

        PrintConvId::FlashMode => match as_u32(value) {
            Some(0) => "Auto".to_string(),
            Some(1) => "On".to_string(),
            Some(2) => "Off".to_string(),
            Some(3) => "Red-eye reduction".to_string(),
            _ => exif_value_to_string(value),
        },

        PrintConvId::FocusMode => match as_u32(value) {
            Some(0) => "Auto".to_string(),
            Some(1) => "Manual".to_string(),
            Some(2) => "Macro".to_string(),
            Some(3) => "Single".to_string(),
            Some(4) => "Continuous".to_string(),
            _ => exif_value_to_string(value),
        },

        PrintConvId::WhiteBalance => match as_u32(value) {
            Some(0) => "Auto".to_string(),
            Some(1) => "Daylight".to_string(),
            Some(2) => "Shade".to_string(),
            Some(3) => "Fluorescent".to_string(),
            Some(4) => "Tungsten".to_string(),
            Some(5) => "Manual".to_string(),
            _ => exif_value_to_string(value),
        },

        PrintConvId::MeteringMode => match as_u32(value) {
            Some(0) => "Multi-segment".to_string(),
            Some(1) => "Center-weighted".to_string(),
            Some(2) => "Spot".to_string(),
            _ => exif_value_to_string(value),
        },

        PrintConvId::IsoSpeed => {
            // Handle various ISO representations
            match as_u32(value) {
                Some(n) if n < 50 => format!("ISO {}", 50 << n), // Power of 2 encoding
                Some(n) => format!("ISO {}", n),                 // Direct value
                _ => exif_value_to_string(value),
            }
        }

        PrintConvId::ExposureCompensation => {
            match as_i32(value) {
                Some(n) => {
                    let ev = n as f32 / 3.0; // Common 1/3 EV steps
                    if ev >= 0.0 {
                        format!("+{:.1} EV", ev)
                    } else {
                        format!("{:.1} EV", ev)
                    }
                }
                _ => exif_value_to_string(value),
            }
        }

        // Canon shared conversions (consolidate multiple references)
        PrintConvId::CanonLensType => canon_lens_type_lookup(value),
        PrintConvId::CanonUserDefPictureStyle => canon_user_def_picture_style_lookup(value),
        PrintConvId::CanonPictureStyle => canon_picture_style_lookup(value),

        // Pentax shared conversions (optimized to eliminate duplicates)
        PrintConvId::PentaxModelLookup => pentax_model_lookup(value),
        PrintConvId::PentaxLensType => pentax_lens_type(value),
        PrintConvId::PentaxPictureMode => pentax_picture_mode(value),
        PrintConvId::PentaxFNumber => pentax_fnumber_format(value),
        PrintConvId::PentaxExposureTime => pentax_exposure_time_format(value),
        PrintConvId::PentaxSensitivityAdjust => pentax_sensitivity_adjust_format(value),

        // Manufacturer-specific conversions will be implemented as needed
        _ => {
            // For now, return raw value for unimplemented conversions
            // TODO: Implement remaining conversion functions
            exif_value_to_string(value)
        }
    }
}

/// Helper to extract u32 value from ExifValue
fn as_u32(value: &ExifValue) -> Option<u32> {
    match value {
        ExifValue::U32(n) => Some(*n),
        ExifValue::U16(n) => Some(*n as u32),
        ExifValue::U8(n) => Some(*n as u32),
        ExifValue::I32(n) if *n >= 0 => Some(*n as u32),
        ExifValue::I16(n) if *n >= 0 => Some(*n as u32),
        _ => None,
    }
}

/// Helper to extract i32 value from ExifValue
fn as_i32(value: &ExifValue) -> Option<i32> {
    match value {
        ExifValue::I32(n) => Some(*n),
        ExifValue::I16(n) => Some(*n as i32),
        ExifValue::U32(n) if *n <= i32::MAX as u32 => Some(*n as i32),
        ExifValue::U16(n) => Some(*n as i32),
        ExifValue::U8(n) => Some(*n as i32),
        _ => None,
    }
}

/// Convert ExifValue to a simple string representation
fn exif_value_to_string(value: &ExifValue) -> String {
    match value {
        ExifValue::Ascii(s) => s.clone(),
        ExifValue::U8(n) => n.to_string(),
        ExifValue::U8Array(arr) => format!("{:?}", arr),
        ExifValue::U16(n) => n.to_string(),
        ExifValue::U16Array(arr) => format!("{:?}", arr),
        ExifValue::U32(n) => n.to_string(),
        ExifValue::U32Array(arr) => format!("{:?}", arr),
        ExifValue::I16(n) => n.to_string(),
        ExifValue::I16Array(arr) => format!("{:?}", arr),
        ExifValue::I32(n) => n.to_string(),
        ExifValue::I32Array(arr) => format!("{:?}", arr),
        ExifValue::Rational(num, den) => {
            if *den == 1 {
                num.to_string()
            } else {
                format!("{}/{}", num, den)
            }
        }
        ExifValue::RationalArray(arr) => {
            let strs: Vec<String> = arr
                .iter()
                .map(|(num, den)| {
                    if *den == 1 {
                        num.to_string()
                    } else {
                        format!("{}/{}", num, den)
                    }
                })
                .collect();
            format!("[{}]", strs.join(", "))
        }
        ExifValue::SignedRational(num, den) => {
            if *den == 1 {
                num.to_string()
            } else {
                format!("{}/{}", num, den)
            }
        }
        ExifValue::SignedRationalArray(arr) => {
            let strs: Vec<String> = arr
                .iter()
                .map(|(num, den)| {
                    if *den == 1 {
                        num.to_string()
                    } else {
                        format!("{}/{}", num, den)
                    }
                })
                .collect();
            format!("[{}]", strs.join(", "))
        }
        ExifValue::Undefined(data) => format!("Undefined({})", data.len()),
        ExifValue::BinaryData(len) => format!("BinaryData({})", len),
    }
}

/// Pentax model lookup conversion
fn pentax_model_lookup(value: &ExifValue) -> String {
    // Simplified version - in practice this would be a large lookup table
    // generated from ExifTool's %pentaxModelType hash
    match as_u32(value) {
        Some(0x12926) => "Optio 330/430".to_string(),
        Some(0x12958) => "Optio 230".to_string(),
        Some(0x12962) => "Optio 330GS".to_string(),
        Some(0x1296c) => "Optio 450/550".to_string(),
        Some(0x12971) => "*ist D".to_string(),
        Some(0x12994) => "*ist DS".to_string(),
        Some(0x129b2) => "Optio S".to_string(),
        Some(0x129bc) => "Optio S V1.01".to_string(),
        // ... would contain hundreds more entries from ExifTool
        _ => format!("Unknown ({})", exif_value_to_string(value)),
    }
}

/// Pentax picture mode conversion
fn pentax_picture_mode(value: &ExifValue) -> String {
    match as_u32(value) {
        Some(0) => "Auto".to_string(),
        Some(1) => "Night-scene".to_string(),
        Some(2) => "Manual".to_string(),
        Some(3) => "Multiple-exposure".to_string(),
        Some(5) => "Portrait".to_string(),
        Some(6) => "Landscape".to_string(),
        Some(8) => "Sport".to_string(),
        Some(9) => "Macro".to_string(),
        Some(11) => "Soft".to_string(),
        Some(12) => "Surf & Snow".to_string(),
        Some(13) => "Sunset or Candlelight".to_string(),
        Some(14) => "Autumn".to_string(),
        Some(15) => "Fireworks".to_string(),
        Some(17) => "Dynamic (Enhanced)".to_string(),
        Some(18) => "Objects in Motion".to_string(),
        Some(19) => "Text".to_string(),
        _ => format!("Unknown ({})", exif_value_to_string(value)),
    }
}

/// Pentax lens type conversion  
fn pentax_lens_type(value: &ExifValue) -> String {
    // This would be generated from ExifTool's %pentaxLensTypes hash
    // For now, a simplified version
    match exif_value_to_string(value).as_str() {
        "0 0" => "M-42 or No Lens".to_string(),
        "1 0" => "K or M Lens".to_string(),
        "2 0" => "A Series Lens".to_string(),
        "3 0" => "Sigma".to_string(),
        _ => format!("Unknown ({})", exif_value_to_string(value)),
    }
}

/// Canon lens type lookup (shared by 25 tags)
/// Consolidates all references to %canonLensTypes in ExifTool
fn canon_lens_type_lookup(value: &ExifValue) -> String {
    match as_u32(value) {
        Some(val) => match val {
            // Core Canon lens types from ExifTool %canonLensTypes
            1 => "Canon EF 50mm f/1.8".to_string(),
            2 => "Canon EF 28mm f/2.8 or Sigma Lens".to_string(),
            3 => "Canon EF 135mm f/2.8 Soft".to_string(),
            4 => "Canon EF 35-105mm f/3.5-4.5 or Sigma Lens".to_string(),
            5 => "Canon EF 35-70mm f/3.5-4.5".to_string(),
            6 => "Canon EF 28-70mm f/3.5-4.5 or Sigma or Tokina Lens".to_string(),
            7 => "Canon EF 100-300mm f/5.6L".to_string(),
            8 => "Canon EF 100-300mm f/5.6 or Sigma or Tokina Lens".to_string(),
            9 => "Canon EF 70-210mm f/4".to_string(),
            10 => "Canon EF 50mm f/2.5 Macro or Sigma Lens".to_string(),
            11 => "Canon EF 35mm f/2".to_string(),
            13 => "Canon EF 15mm f/2.8 Fisheye".to_string(),
            14 => "Canon EF 50-200mm f/3.5-4.5L".to_string(),
            15 => "Canon EF 50-200mm f/3.5-4.5".to_string(),
            16 => "Canon EF 35-135mm f/3.5-4.5".to_string(),
            17 => "Canon EF 35-70mm f/3.5-4.5A".to_string(),
            18 => "Canon EF 28-70mm f/3.5-4.5".to_string(),
            20 => "Canon EF 100-200mm f/4.5A".to_string(),
            21 => "Canon EF 80-200mm f/2.8L".to_string(),
            22 => "Canon EF 20-35mm f/2.8L or Tokina Lens".to_string(),
            23 => "Canon EF 35-105mm f/3.5-4.5".to_string(),
            24 => "Canon EF 35-80mm f/4-5.6 Power Zoom".to_string(),
            25 => "Canon EF 35-80mm f/4-5.6 Power Zoom".to_string(),
            26 => "Canon EF 100mm f/2.8 Macro or Other Lens".to_string(),
            27 => "Canon EF 35-80mm f/4-5.6".to_string(),
            28 => "Canon EF 80-200mm f/4.5-5.6 or Tamron Lens".to_string(),
            // More entries would be added here for completeness
            _ => format!("Unknown Lens ({})", val),
        },
        None => format!("Unknown ({})", exif_value_to_string(value)),
    }
}

/// Canon user-defined picture style lookup (shared by 9 tags)
/// Consolidates all references to %userDefStyles in ExifTool
fn canon_user_def_picture_style_lookup(value: &ExifValue) -> String {
    match as_u32(value) {
        Some(val) => match val {
            0x41 => "PC 1".to_string(),
            0x42 => "PC 2".to_string(),
            0x43 => "PC 3".to_string(),
            0x81 => "Standard".to_string(),
            0x82 => "Portrait".to_string(),
            0x83 => "Landscape".to_string(),
            0x84 => "Neutral".to_string(),
            0x85 => "Faithful".to_string(),
            0x86 => "Monochrome".to_string(),
            0x87 => "Auto".to_string(),
            _ => format!("Unknown ({})", val),
        },
        None => format!("Unknown ({})", exif_value_to_string(value)),
    }
}

/// Canon picture style lookup (shared by 18 tags)
/// Consolidates all references to %pictureStyles in ExifTool
fn canon_picture_style_lookup(value: &ExifValue) -> String {
    match as_u32(value) {
        Some(val) => match val {
            0x00 => "None".to_string(),
            0x01 => "Standard".to_string(),
            0x02 => "Portrait".to_string(),
            0x03 => "High Saturation".to_string(),
            0x04 => "Adobe RGB".to_string(),
            0x05 => "Low Saturation".to_string(),
            0x06 => "CM Set 1".to_string(),
            0x07 => "CM Set 2".to_string(),
            0x21 => "User Def. 1".to_string(),
            0x22 => "User Def. 2".to_string(),
            0x23 => "User Def. 3".to_string(),
            _ => format!("Unknown ({})", val),
        },
        None => format!("Unknown ({})", exif_value_to_string(value)),
    }
}

/// Pentax F-number formatting (shared by 15 tags)
/// Consolidates all sprintf("%.1f") patterns in ExifTool Pentax.pm
fn pentax_fnumber_format(value: &ExifValue) -> String {
    match as_u32(value) {
        Some(n) => {
            let f_num = n as f32 / 10.0;
            format!("{:.1}", f_num)
        }
        _ => exif_value_to_string(value),
    }
}

/// Pentax exposure time formatting (shared by multiple tags)
/// Handles ExifTool's exposure time conversion patterns
fn pentax_exposure_time_format(value: &ExifValue) -> String {
    match as_u32(value) {
        Some(n) if n > 0 => {
            let exposure = 24000000.0 / n as f32;
            if exposure >= 1.0 {
                format!("{:.1}", exposure)
            } else {
                format!("1/{:.0}", 1.0 / exposure)
            }
        }
        _ => exif_value_to_string(value),
    }
}

/// Pentax sensitivity adjustment formatting (shared by 4 tags)
/// Consolidates sprintf("%+.1f") patterns for exposure compensation
fn pentax_sensitivity_adjust_format(value: &ExifValue) -> String {
    match as_i32(value) {
        Some(n) if n != 0 => {
            let adjustment = n as f32 / 10.0;
            format!("{:+.1}", adjustment)
        }
        Some(0) => "0".to_string(),
        _ => exif_value_to_string(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_on_off_conversion() {
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0), PrintConvId::OnOff),
            "Off"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(1), PrintConvId::OnOff),
            "On"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(2), PrintConvId::OnOff),
            "2"
        );
    }

    #[test]
    fn test_image_size_conversion() {
        let size = ExifValue::U16Array(vec![1920, 1080]);
        assert_eq!(apply_print_conv(&size, PrintConvId::ImageSize), "1920x1080");
    }

    #[test]
    fn test_pentax_picture_mode() {
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0), PrintConvId::PentaxPictureMode),
            "Auto"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(5), PrintConvId::PentaxPictureMode),
            "Portrait"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(999), PrintConvId::PentaxPictureMode),
            "Unknown (999)"
        );
    }

    #[test]
    fn test_canon_shared_lookup_optimization() {
        // Test shared CanonLensType lookup (used by 25 tags)
        assert_eq!(
            apply_print_conv(&ExifValue::U32(1), PrintConvId::CanonLensType),
            "Canon EF 50mm f/1.8"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(21), PrintConvId::CanonLensType),
            "Canon EF 80-200mm f/2.8L"
        );

        // Test shared CanonUserDefPictureStyle lookup (used by 9 tags)
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0x81), PrintConvId::CanonUserDefPictureStyle),
            "Standard"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0x86), PrintConvId::CanonUserDefPictureStyle),
            "Monochrome"
        );

        // Test shared CanonPictureStyle lookup (used by 18 tags)
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0x01), PrintConvId::CanonPictureStyle),
            "Standard"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0x04), PrintConvId::CanonPictureStyle),
            "Adobe RGB"
        );

        // Test universal OnOff shared by Canon and other manufacturers (22 references)
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0), PrintConvId::OnOff),
            "Off"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(1), PrintConvId::OnOff),
            "On"
        );
    }

    #[test]
    fn test_pentax_shared_optimizations() {
        // Test shared PentaxFNumber formatting (used by 15 tags)
        assert_eq!(
            apply_print_conv(&ExifValue::U32(280), PrintConvId::PentaxFNumber),
            "28.0"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(56), PrintConvId::PentaxFNumber),
            "5.6"
        );

        // Test shared PentaxSensitivityAdjust formatting (used by 4 tags)
        assert_eq!(
            apply_print_conv(&ExifValue::I32(10), PrintConvId::PentaxSensitivityAdjust),
            "+1.0"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::I32(-15), PrintConvId::PentaxSensitivityAdjust),
            "-1.5"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::I32(0), PrintConvId::PentaxSensitivityAdjust),
            "0"
        );

        // Test shared PentaxModelLookup (used by multiple model tags)
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0x12971), PrintConvId::PentaxModelLookup),
            "*ist D"
        );

        // Test shared PentaxLensType (used by multiple lens tags)
        assert_eq!(
            apply_print_conv(
                &ExifValue::Ascii("1 0".to_string()),
                PrintConvId::PentaxLensType
            ),
            "K or M Lens"
        );
    }

    #[test]
    fn test_no_conversion() {
        assert_eq!(
            apply_print_conv(&ExifValue::U32(42), PrintConvId::None),
            "42"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::Ascii("test".to_string()), PrintConvId::None),
            "test"
        );
    }
}
