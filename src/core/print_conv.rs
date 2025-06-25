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

    /// Apple-specific conversions
    AppleHDRImageType,
    AppleImageCaptureType,
    AppleCameraType,

    /// EXIF-specific conversions
    ExposureTime, // 0x829a - "1/125" formatting
    FNumber,     // 0x829d - "f/4.0" formatting
    FocalLength, // 0x920a - "23.0 mm" formatting
    DateTime,    // Date/time formatting
    Resolution,  // Resolution formatting
    Compression, // Compression lookup
    ColorSpace,  // Color space lookup

    /// Universal EXIF conversions (used by all manufacturers)
    Flash, // Flash mode (24 standard values from EXIF spec)
    LightSource,        // Light source for white balance (Universal lookup)
    Orientation,        // Image orientation (8 standard values)
    ExposureProgram,    // Exposure program mode (Auto/Manual/Aperture/Shutter Priority)
    ExifColorSpace,     // EXIF color space (sRGB/Adobe RGB/Uncalibrated)
    UniversalParameter, // Normal/Low/High pattern (Contrast/Saturation/Sharpness)
    ExifWhiteBalance,   // EXIF white balance (Auto/Manual)
    ExposureMode,       // Exposure mode (Auto/Manual/Auto bracket)
    ResolutionUnit,     // Resolution units (None/inches/cm)

    /// Universal patterns used by 3+ manufacturers
    UniversalOnOffAuto, // 0=Off, 1=On, 2=Auto (6 manufacturers, 25+ tags)
    UniversalNoiseReduction, // 0=Off, 1=Low, 2=Normal, 3=High, 4=Auto

    /// GPMF (GoPro Metadata Format) specific conversions
    GpmfAccelerometer,
    GpmfAccelerometerMatrix,
    GpmfAspectRatioUnwarped,
    GpmfAspectRatioWarped,
    GpmfAttitude,
    GpmfAttitudeTarget,
    GpmfAudioProtuneOption,
    GpmfAudioSetting,
    GpmfAutoBoostScore,
    GpmfAutoISOMax,
    GpmfAutoISOMin,
    GpmfAutoLowLightDuration,
    GpmfAutoRotation,
    GpmfBatteryStatus,
    GpmfBitrateSetting,
    GpmfCameraSerialNumber,
    GpmfCameraTemperature,
    GpmfCaptureDelayTimer,
    GpmfChapterNumber,
    GpmfColorMode,
    GpmfColorTemperatures,
    GpmfComments,
    GpmfController,
    GpmfControlLevel,
    GpmfCoyoteSense,
    GpmfCoyoteStatus,
    GpmfCreationDate,
    GpmfDeviceContainer,
    GpmfDeviceName,
    GpmfDiagonalFieldOfView,
    GpmfDigitalZoom,
    GpmfDigitalZoomAmount,
    GpmfDigitalZoomOn,
    GpmfDurationSetting,
    GpmfElectronicImageStabilization,
    GpmfEscapeStatus,
    GpmfExposureCompensation,
    GpmfExposureTimes,
    GpmfExposureType,
    GpmfFaceDetected,
    GpmfFaceNumbers,
    GpmfFieldOfView,
    GpmfFirmwareVersion,
    GpmfGPSAltitudeSystem,
    GpmfGPSDateTime,
    GpmfGPSHPositioningError,
    GpmfGPSInfo,
    GpmfGPSInfo9,
    GpmfGPSMeasureMode,
    GpmfGPSPos,
    GpmfGPSRaw,
    GpmfGyroscope,
    GpmfHDRSetting,
    GpmfHindsightSettings,
    GpmfHorizonControl,
    GpmfImageSensorGain,
    GpmfInputOrientation,
    GpmfInputUniformity,
    GpmfISOSpeeds,
    GpmfLensProjection,
    GpmfLocalPositionNED,
    GpmfLumaAverage,
    GpmfMagnetometer,
    GpmfMappingXCoefficients,
    GpmfMappingXMode,
    GpmfMappingYCoefficients,
    GpmfMappingYMode,
    GpmfMediaMode,
    GpmfMediaUniqueID,
    GpmfMetadataVersion,
    GpmfMicrophoneWet,
    GpmfModel,
    GpmfNestedSignalStream,
    GpmfOtherFirmware,
    GpmfOutputOrientation,
    GpmfPhotoResolution,
    GpmfPolynomialCoefficients,
    GpmfPolynomialPower,
    GpmfPowerProfile,
    GpmfPrediminantHue,
    GpmfProtune,
    GpmfProtuneISOMode,
    GpmfRate,
    GpmfScaledIMU,
    GpmfScaledPressure,
    GpmfScaleFactor,
    GpmfSceneClassification,
    GpmfScheduleCaptureTime,
    GpmfSensorReadoutTime,
    GpmfSharpness,
    GpmfSIUnits,
    GpmfSpeedRampSetting,
    GpmfStreamName,
    GpmfSystemTime,
    GpmfTimeStamp,
    GpmfTimeZone,
    GpmfUnits,
    GpmfVisualFlightRulesHUD,
    GpmfWhiteBalance,
    GpmfWhiteBalanceRGB,
    GpmfWindProcessing,
    GpmfZoomScaleNormalization,

    // Panasonic-specific conversions
    PanasonicIntelligentExposure,
    PanasonicBracketSettings,
    PanasonicFilterEffect,
    PanasonicPanasonicExifVersion,
    PanasonicMonochromeFilterEffect,
    PanasonicPrintIM,
    PanasonicMakerNoteVersion,
    PanasonicFilmMode,
    PanasonicWBShiftGM,
    PanasonicAccelerometerY,
    PanasonicWBRedLevel,
    PanasonicContrast,
    PanasonicFlashCurtain,
    PanasonicAudio,
    PanasonicCity,
    PanasonicSensorType,
    PanasonicSelfTimer,
    PanasonicTextStamp,
    PanasonicShutterType,
    PanasonicOpticalZoomMode,
    PanasonicWBShiftAB,
    PanasonicHDR,
    PanasonicDarkFocusEnvironment,
    PanasonicState,
    PanasonicIntelligentResolution,
    PanasonicVideoBurstMode,
    PanasonicShootingMode,
    PanasonicSceneMode,
    PanasonicLocation,
    PanasonicColorEffect,
    PanasonicLensType,
    PanasonicTravelDay,
    PanasonicColorTempKelvin,
    PanasonicRecognizedFaceFlags,
    PanasonicWBShiftCreativeControl,
    PanasonicTimerRecording,
    PanasonicFocusBracket,
    PanasonicPostFocusMerging,
    PanasonicBatteryLevel,
    PanasonicSharpness,
    PanasonicAFSubjectDetection,
    PanasonicWBBlueLevel,
    PanasonicTransform,
    PanasonicFaceDetInfo,
    PanasonicBabyName,
    PanasonicTitle,
    PanasonicFaceRecInfo,
    PanasonicManometerPressure,
    PanasonicPanasonicImageWidth,
    PanasonicFacesDetected,
    PanasonicRotation,
    PanasonicFirmwareVersion,
    PanasonicSequenceNumber,
    PanasonicImageStabilization,
    PanasonicMergedImages,
    PanasonicAccelerometerZ,
    PanasonicSweepPanoramaFieldOfView,
    PanasonicBabyAge,
    PanasonicWBShiftIntelligentAuto,
    PanasonicAccelerometerX,
    PanasonicHighlightShadow,
    PanasonicVideoBurstResolution,
    PanasonicSaturation,
    PanasonicVideoPreburst,
    PanasonicFlashFired,
    PanasonicVideoFrameRate,
    PanasonicRollAngle,
    PanasonicMacroMode,
    PanasonicCountry,
    PanasonicCameraOrientation,
    PanasonicIntelligentD_Range,
    PanasonicNoiseReductionStrength,
    PanasonicInternalSerialNumber,
    PanasonicTimeSincePowerOn,
    PanasonicWorldTimeLocation,
    PanasonicDataDump,
    PanasonicFocusMode,
    PanasonicProgramISO,
    PanasonicNoiseReduction,
    PanasonicAccessorySerialNumber,
    PanasonicPhotoStyle,
    PanasonicClearRetouchValue,
    PanasonicMinimumISO,
    PanasonicHighlightWarning,
    PanasonicSweepPanoramaDirection,
    PanasonicAFAssistLamp,
    PanasonicLongExposureNoiseReduction,
    PanasonicAFPointPosition,
    PanasonicOutputLUT,
    PanasonicPanasonicImageHeight,
    PanasonicLandmark,
    PanasonicBurstSpeed,
    PanasonicCity2,
    PanasonicMonochromeGrainEffect,
    PanasonicISO,
    PanasonicTimeInfo,
    PanasonicAccessoryType,
    PanasonicPitchAngle,
    PanasonicWBGreenLevel,
    PanasonicFlashBias,
    PanasonicMultiExposure,
    PanasonicColorMode,
    PanasonicModelLookup,
    PanasonicFlashWarning,
    PanasonicDiffractionCorrection,
    PanasonicInternalNDFilter,
    PanasonicTimeStamp,

    /// Casio-specific conversions
    CasioRecordingMode,
    CasioFocusMode,
    CasioFlashIntensity,
    CasioObjectDistance,
    CasioDigitalZoom,
    CasioSharpness,
    CasioContrast,
    CasioSaturation,
    CasioISO,
    CasioFirmwareDate,
    CasioEnhancement,
    CasioColorFilter,
    CasioAFPoint,
    CasioPrintIM,

    /// Kodak-specific conversions
    KodakModelLookup,
    KodakKodakImageWidth,
    KodakKodakImageHeight,
    KodakYearCreated,
    KodakMonthDayCreated,
    KodakTimeCreated,
    KodakBurstMode2,
    KodakVariousModes,
    KodakVariousModes2,
    KodakShutterMode,
    KodakPanoramaMode,
    KodakExposureTime,
    KodakFNumber,
    KodakExposureCompensation,
    KodakFocusMode,
    KodakDistance1,
    KodakDistance2,
    KodakDistance3,
    KodakDistance4,
    KodakSubjectDistance,
    KodakISOSetting,
    KodakISO,
    KodakTotalZoom,
    KodakDateTimeStamp,
    KodakColorMode,
    KodakDigitalZoom,
    KodakSharpness,

    /// Minolta-specific conversions
    MinoltaMakerNoteVersion,
    MinoltaMinoltaCameraSettingsOld,
    MinoltaMinoltaCameraSettings,
    MinoltaMinoltaCameraSettings7D,
    MinoltaCameraInfoA100,
    MinoltaWBInfoA100,
    MinoltaCompressedImageSize,
    MinoltaPreviewImageStart,
    MinoltaPreviewImageLength,
    MinoltaSceneMode,
    MinoltaFlashExposureComp,
    MinoltaTeleconverter,
    MinoltaImageStabilization,
    MinoltaZoneMatching,
    MinoltaColorTemperature,
    MinoltaMinoltaLensTypes,
    MinoltaColorCompensationFilter,
    MinoltaPrintIM,
    MinoltaMinoltaCameraSettings2,
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
        // Apple-specific conversions
        PrintConvId::AppleHDRImageType => match as_u32(value) {
            Some(3) => "HDR Image".to_string(),
            Some(4) => "Original Image".to_string(),
            _ => format!("Unknown ({})", exif_value_to_string(value)),
        },

        PrintConvId::AppleImageCaptureType => match as_u32(value) {
            Some(1) => "ProRAW".to_string(),
            Some(2) => "Portrait".to_string(),
            Some(10) => "Photo".to_string(),
            Some(11) => "Manual Focus".to_string(),
            Some(12) => "Scene".to_string(),
            _ => format!("Unknown ({})", exif_value_to_string(value)),
        },

        PrintConvId::AppleCameraType => match as_u32(value) {
            Some(0) => "Back Wide Angle".to_string(),
            Some(1) => "Back Normal".to_string(),
            Some(6) => "Front".to_string(),
            _ => format!("Unknown ({})", exif_value_to_string(value)),
        },

        // EXIF-specific conversions
        PrintConvId::ExposureTime => format_exposure_time(value),
        PrintConvId::FNumber => format_f_number(value),
        PrintConvId::FocalLength => format_focal_length(value),
        PrintConvId::DateTime => format_datetime(value),
        PrintConvId::Resolution => format_resolution(value),
        PrintConvId::Compression => format_compression(value),
        PrintConvId::ColorSpace => format_color_space(value),

        // Universal EXIF conversions
        PrintConvId::Flash => format_flash(value),
        PrintConvId::LightSource => format_light_source(value),
        PrintConvId::Orientation => format_orientation(value),
        PrintConvId::ExposureProgram => format_exposure_program(value),
        PrintConvId::MeteringMode => format_metering_mode(value),
        PrintConvId::ExifColorSpace => format_exif_color_space(value),
        PrintConvId::UniversalParameter => format_universal_parameter(value),
        PrintConvId::ExifWhiteBalance => format_exif_white_balance(value),
        PrintConvId::ExposureMode => format_exposure_mode(value),
        PrintConvId::ResolutionUnit => format_resolution_unit(value),

        // Universal patterns used by 3+ manufacturers
        PrintConvId::UniversalOnOffAuto => match as_u32(value) {
            Some(0) => "Off".to_string(),
            Some(1) => "On".to_string(),
            Some(2) => "Auto".to_string(),
            _ => format!("Unknown ({})", exif_value_to_string(value)),
        },

        PrintConvId::UniversalNoiseReduction => match as_u32(value) {
            Some(0) => "Off".to_string(),
            Some(1) => "Low".to_string(),
            Some(2) => "Normal".to_string(),
            Some(3) => "High".to_string(),
            Some(4) => "Auto".to_string(),
            _ => format!("Unknown ({})", exif_value_to_string(value)),
        },

        // GPMF conversions - for now, return raw value
        // TODO: Implement GPMF-specific conversion functions based on ExifTool's GoPro.pm
        PrintConvId::GpmfAccelerometer => exif_value_to_string(value),
        PrintConvId::GpmfAccelerometerMatrix => exif_value_to_string(value),
        PrintConvId::GpmfAspectRatioUnwarped => exif_value_to_string(value),
        PrintConvId::GpmfAspectRatioWarped => exif_value_to_string(value),
        PrintConvId::GpmfAttitude => exif_value_to_string(value),
        PrintConvId::GpmfAttitudeTarget => exif_value_to_string(value),
        PrintConvId::GpmfAudioProtuneOption => exif_value_to_string(value),
        PrintConvId::GpmfAudioSetting => exif_value_to_string(value),
        PrintConvId::GpmfAutoBoostScore => exif_value_to_string(value),
        PrintConvId::GpmfAutoISOMax => exif_value_to_string(value),
        PrintConvId::GpmfAutoISOMin => exif_value_to_string(value),
        PrintConvId::GpmfAutoLowLightDuration => exif_value_to_string(value),
        PrintConvId::GpmfAutoRotation => exif_value_to_string(value),
        PrintConvId::GpmfBatteryStatus => exif_value_to_string(value),
        PrintConvId::GpmfBitrateSetting => exif_value_to_string(value),
        PrintConvId::GpmfCameraSerialNumber => exif_value_to_string(value),
        PrintConvId::GpmfCameraTemperature => exif_value_to_string(value),
        PrintConvId::GpmfCaptureDelayTimer => exif_value_to_string(value),
        PrintConvId::GpmfChapterNumber => exif_value_to_string(value),
        PrintConvId::GpmfColorMode => exif_value_to_string(value),
        PrintConvId::GpmfColorTemperatures => exif_value_to_string(value),
        PrintConvId::GpmfComments => exif_value_to_string(value),
        PrintConvId::GpmfController => exif_value_to_string(value),
        PrintConvId::GpmfControlLevel => exif_value_to_string(value),
        PrintConvId::GpmfCoyoteSense => exif_value_to_string(value),
        PrintConvId::GpmfCoyoteStatus => exif_value_to_string(value),
        PrintConvId::GpmfCreationDate => exif_value_to_string(value),
        PrintConvId::GpmfDeviceContainer => exif_value_to_string(value),
        PrintConvId::GpmfDeviceName => exif_value_to_string(value),
        PrintConvId::GpmfDiagonalFieldOfView => exif_value_to_string(value),
        PrintConvId::GpmfDigitalZoom => exif_value_to_string(value),
        PrintConvId::GpmfDigitalZoomAmount => exif_value_to_string(value),
        PrintConvId::GpmfDigitalZoomOn => exif_value_to_string(value),
        PrintConvId::GpmfDurationSetting => exif_value_to_string(value),
        PrintConvId::GpmfElectronicImageStabilization => exif_value_to_string(value),
        PrintConvId::GpmfEscapeStatus => exif_value_to_string(value),
        PrintConvId::GpmfExposureCompensation => exif_value_to_string(value),
        PrintConvId::GpmfExposureTimes => exif_value_to_string(value),
        PrintConvId::GpmfExposureType => exif_value_to_string(value),
        PrintConvId::GpmfFaceDetected => exif_value_to_string(value),
        PrintConvId::GpmfFaceNumbers => exif_value_to_string(value),
        PrintConvId::GpmfFieldOfView => exif_value_to_string(value),
        PrintConvId::GpmfFirmwareVersion => exif_value_to_string(value),
        PrintConvId::GpmfGPSAltitudeSystem => exif_value_to_string(value),
        PrintConvId::GpmfGPSDateTime => exif_value_to_string(value),
        PrintConvId::GpmfGPSHPositioningError => exif_value_to_string(value),
        PrintConvId::GpmfGPSInfo => exif_value_to_string(value),
        PrintConvId::GpmfGPSInfo9 => exif_value_to_string(value),
        PrintConvId::GpmfGPSMeasureMode => exif_value_to_string(value),
        PrintConvId::GpmfGPSPos => exif_value_to_string(value),
        PrintConvId::GpmfGPSRaw => exif_value_to_string(value),
        PrintConvId::GpmfGyroscope => exif_value_to_string(value),
        PrintConvId::GpmfHDRSetting => exif_value_to_string(value),
        PrintConvId::GpmfHindsightSettings => exif_value_to_string(value),
        PrintConvId::GpmfHorizonControl => exif_value_to_string(value),
        PrintConvId::GpmfImageSensorGain => exif_value_to_string(value),
        PrintConvId::GpmfInputOrientation => exif_value_to_string(value),
        PrintConvId::GpmfInputUniformity => exif_value_to_string(value),
        PrintConvId::GpmfISOSpeeds => exif_value_to_string(value),
        PrintConvId::GpmfLensProjection => exif_value_to_string(value),
        PrintConvId::GpmfLocalPositionNED => exif_value_to_string(value),
        PrintConvId::GpmfLumaAverage => exif_value_to_string(value),
        PrintConvId::GpmfMagnetometer => exif_value_to_string(value),
        PrintConvId::GpmfMappingXCoefficients => exif_value_to_string(value),
        PrintConvId::GpmfMappingXMode => exif_value_to_string(value),
        PrintConvId::GpmfMappingYCoefficients => exif_value_to_string(value),
        PrintConvId::GpmfMappingYMode => exif_value_to_string(value),
        PrintConvId::GpmfMediaMode => exif_value_to_string(value),
        PrintConvId::GpmfMediaUniqueID => exif_value_to_string(value),
        PrintConvId::GpmfMetadataVersion => exif_value_to_string(value),
        PrintConvId::GpmfMicrophoneWet => exif_value_to_string(value),
        PrintConvId::GpmfModel => exif_value_to_string(value),
        PrintConvId::GpmfNestedSignalStream => exif_value_to_string(value),
        PrintConvId::GpmfOtherFirmware => exif_value_to_string(value),
        PrintConvId::GpmfOutputOrientation => exif_value_to_string(value),
        PrintConvId::GpmfPhotoResolution => exif_value_to_string(value),
        PrintConvId::GpmfPolynomialCoefficients => exif_value_to_string(value),
        PrintConvId::GpmfPolynomialPower => exif_value_to_string(value),
        PrintConvId::GpmfPowerProfile => exif_value_to_string(value),
        PrintConvId::GpmfPrediminantHue => exif_value_to_string(value),
        PrintConvId::GpmfProtune => exif_value_to_string(value),
        PrintConvId::GpmfProtuneISOMode => exif_value_to_string(value),
        PrintConvId::GpmfRate => exif_value_to_string(value),
        PrintConvId::GpmfScaledIMU => exif_value_to_string(value),
        PrintConvId::GpmfScaledPressure => exif_value_to_string(value),
        PrintConvId::GpmfScaleFactor => exif_value_to_string(value),
        PrintConvId::GpmfSceneClassification => exif_value_to_string(value),
        PrintConvId::GpmfScheduleCaptureTime => exif_value_to_string(value),
        PrintConvId::GpmfSensorReadoutTime => exif_value_to_string(value),
        PrintConvId::GpmfSharpness => exif_value_to_string(value),
        PrintConvId::GpmfSIUnits => exif_value_to_string(value),
        PrintConvId::GpmfSpeedRampSetting => exif_value_to_string(value),
        PrintConvId::GpmfStreamName => exif_value_to_string(value),
        PrintConvId::GpmfSystemTime => exif_value_to_string(value),
        PrintConvId::GpmfTimeStamp => exif_value_to_string(value),
        PrintConvId::GpmfTimeZone => exif_value_to_string(value),
        PrintConvId::GpmfUnits => exif_value_to_string(value),
        PrintConvId::GpmfVisualFlightRulesHUD => exif_value_to_string(value),
        PrintConvId::GpmfWhiteBalance => exif_value_to_string(value),
        PrintConvId::GpmfWhiteBalanceRGB => exif_value_to_string(value),
        PrintConvId::GpmfWindProcessing => exif_value_to_string(value),
        PrintConvId::GpmfZoomScaleNormalization => exif_value_to_string(value),

        // Casio-specific conversions
        PrintConvId::CasioRecordingMode => match as_u32(value) {
            Some(1) => "Single Shutter".to_string(),
            Some(2) => "Panorama".to_string(),
            Some(3) => "Night Scene".to_string(),
            Some(4) => "Portrait".to_string(),
            Some(5) => "Landscape".to_string(),
            Some(7) => "Sport".to_string(),
            Some(10) => "Night".to_string(),
            Some(15) => "Copy".to_string(),
            _ => format!("Unknown ({})", exif_value_to_string(value)),
        },

        PrintConvId::CasioFocusMode => match as_u32(value) {
            Some(2) => "Macro".to_string(),
            Some(3) => "Auto".to_string(),
            Some(4) => "Manual".to_string(),
            Some(5) => "Infinity".to_string(),
            _ => format!("Unknown ({})", exif_value_to_string(value)),
        },

        PrintConvId::CasioFlashIntensity => match as_u32(value) {
            Some(11) => "Weak".to_string(),
            Some(13) => "Normal".to_string(),
            Some(15) => "Strong".to_string(),
            _ => format!("Unknown ({})", exif_value_to_string(value)),
        },

        PrintConvId::CasioObjectDistance => match as_u32(value) {
            Some(0) => "Infinity".to_string(),
            Some(n) => format!("{} mm", n),
            _ => exif_value_to_string(value),
        },

        PrintConvId::CasioDigitalZoom => match as_u32(value) {
            Some(0x10000) => "Off".to_string(),
            Some(0x10001) => "2x".to_string(),
            Some(0x20000) => "2x".to_string(),
            Some(0x40000) => "4x".to_string(),
            _ => format!("Unknown ({})", exif_value_to_string(value)),
        },

        PrintConvId::CasioSharpness => match as_u32(value) {
            Some(0) => "Normal".to_string(),
            Some(1) => "Soft".to_string(),
            Some(2) => "Hard".to_string(),
            _ => format!("Unknown ({})", exif_value_to_string(value)),
        },

        PrintConvId::CasioContrast => match as_u32(value) {
            Some(0) => "Normal".to_string(),
            Some(1) => "Low".to_string(),
            Some(2) => "High".to_string(),
            _ => format!("Unknown ({})", exif_value_to_string(value)),
        },

        PrintConvId::CasioSaturation => match as_u32(value) {
            Some(0) => "Normal".to_string(),
            Some(1) => "Low".to_string(),
            Some(2) => "High".to_string(),
            _ => format!("Unknown ({})", exif_value_to_string(value)),
        },

        PrintConvId::CasioISO => match as_u32(value) {
            Some(3) => "50".to_string(),
            Some(4) => "64".to_string(),
            Some(6) => "100".to_string(),
            Some(9) => "200".to_string(),
            _ => format!("Unknown ({})", exif_value_to_string(value)),
        },

        PrintConvId::CasioFirmwareDate => {
            // Format as date string if possible
            exif_value_to_string(value)
        }

        PrintConvId::CasioEnhancement => match as_u32(value) {
            Some(1) => "Off".to_string(),
            Some(2) => "Red".to_string(),
            Some(3) => "Green".to_string(),
            Some(4) => "Blue".to_string(),
            Some(5) => "Flesh Tones".to_string(),
            _ => format!("Unknown ({})", exif_value_to_string(value)),
        },

        PrintConvId::CasioColorFilter => match as_u32(value) {
            Some(1) => "Off".to_string(),
            Some(2) => "Black & White".to_string(),
            Some(3) => "Sepia".to_string(),
            Some(4) => "Red".to_string(),
            Some(5) => "Green".to_string(),
            Some(6) => "Blue".to_string(),
            Some(7) => "Yellow".to_string(),
            Some(8) => "Pink".to_string(),
            Some(9) => "Purple".to_string(),
            _ => format!("Unknown ({})", exif_value_to_string(value)),
        },

        PrintConvId::CasioAFPoint => match as_u32(value) {
            Some(1) => "Center".to_string(),
            Some(2) => "Auto".to_string(),
            _ => format!("Unknown ({})", exif_value_to_string(value)),
        },

        PrintConvId::CasioPrintIM => {
            // PrintIM data is usually complex, return raw for now
            exif_value_to_string(value)
        }

        // Kodak-specific conversions
        PrintConvId::KodakModelLookup => {
            // For now, return raw value - would need to implement Kodak model lookup table
            exif_value_to_string(value)
        }

        PrintConvId::KodakKodakImageWidth | PrintConvId::KodakKodakImageHeight => {
            // Image dimensions - just format as number
            match as_u32(value) {
                Some(dim) => dim.to_string(),
                _ => exif_value_to_string(value),
            }
        }

        PrintConvId::KodakYearCreated => match as_u32(value) {
            Some(year) => year.to_string(),
            _ => exif_value_to_string(value),
        },

        PrintConvId::KodakMonthDayCreated => {
            // Format: MM:DD
            exif_value_to_string(value)
        }

        PrintConvId::KodakTimeCreated => {
            // Format: HH:MM:SS.ss
            exif_value_to_string(value)
        }

        PrintConvId::KodakBurstMode2 => {
            // Unknown tag - return raw for now
            exif_value_to_string(value)
        }

        PrintConvId::KodakVariousModes | PrintConvId::KodakVariousModes2 => {
            // Various mode settings - would need lookup table
            exif_value_to_string(value)
        }

        PrintConvId::KodakShutterMode => {
            // Shutter mode - would need lookup table
            exif_value_to_string(value)
        }

        PrintConvId::KodakPanoramaMode => match as_u32(value) {
            Some(0) => "Off".to_string(),
            Some(1) => "On".to_string(),
            _ => exif_value_to_string(value),
        },

        PrintConvId::KodakExposureTime => {
            // Exposure time - return raw for now
            exif_value_to_string(value)
        }

        PrintConvId::KodakFNumber => {
            // F-number - return raw for now
            exif_value_to_string(value)
        }

        PrintConvId::KodakExposureCompensation => {
            // Exposure compensation - return raw for now
            exif_value_to_string(value)
        }

        PrintConvId::KodakFocusMode => match as_u32(value) {
            Some(0) => "Normal".to_string(),
            Some(1) => "Macro".to_string(),
            _ => exif_value_to_string(value),
        },

        PrintConvId::KodakDistance1
        | PrintConvId::KodakDistance2
        | PrintConvId::KodakDistance3
        | PrintConvId::KodakDistance4 => {
            // Distance measurements - return raw for now
            exif_value_to_string(value)
        }

        PrintConvId::KodakSubjectDistance => {
            // Subject distance - return raw for now
            exif_value_to_string(value)
        }

        PrintConvId::KodakISOSetting | PrintConvId::KodakISO => {
            // ISO values - return raw for now
            exif_value_to_string(value)
        }

        PrintConvId::KodakTotalZoom => {
            // Total zoom - return raw for now
            exif_value_to_string(value)
        }

        PrintConvId::KodakDateTimeStamp => {
            // Date/time stamp - return raw for now
            exif_value_to_string(value)
        }

        PrintConvId::KodakColorMode => {
            // Color mode - would need lookup table
            exif_value_to_string(value)
        }

        PrintConvId::KodakDigitalZoom => {
            // Digital zoom - return raw for now
            exif_value_to_string(value)
        }

        PrintConvId::KodakSharpness => {
            // Sharpness - return raw for now
            exif_value_to_string(value)
        }

        // Minolta-specific conversions (stub implementations for now)
        PrintConvId::MinoltaMakerNoteVersion => exif_value_to_string(value),
        PrintConvId::MinoltaMinoltaCameraSettingsOld => exif_value_to_string(value),
        PrintConvId::MinoltaMinoltaCameraSettings => exif_value_to_string(value),
        PrintConvId::MinoltaMinoltaCameraSettings7D => exif_value_to_string(value),
        PrintConvId::MinoltaCameraInfoA100 => exif_value_to_string(value),
        PrintConvId::MinoltaWBInfoA100 => exif_value_to_string(value),
        PrintConvId::MinoltaCompressedImageSize => exif_value_to_string(value),
        PrintConvId::MinoltaPreviewImageStart => exif_value_to_string(value),
        PrintConvId::MinoltaPreviewImageLength => exif_value_to_string(value),
        PrintConvId::MinoltaSceneMode => exif_value_to_string(value),
        PrintConvId::MinoltaFlashExposureComp => exif_value_to_string(value),
        PrintConvId::MinoltaTeleconverter => exif_value_to_string(value),
        PrintConvId::MinoltaImageStabilization => exif_value_to_string(value),
        PrintConvId::MinoltaZoneMatching => exif_value_to_string(value),
        PrintConvId::MinoltaColorTemperature => exif_value_to_string(value),
        PrintConvId::MinoltaMinoltaLensTypes => exif_value_to_string(value),
        PrintConvId::MinoltaColorCompensationFilter => exif_value_to_string(value),
        PrintConvId::MinoltaPrintIM => exif_value_to_string(value),
        PrintConvId::MinoltaMinoltaCameraSettings2 => exif_value_to_string(value),

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
        ExifValue::Undefined(data) => {
            // Handle common case where Undefined data contains a numeric value
            match data.len() {
                1 => Some(data[0] as u32),
                2 => Some(u16::from_le_bytes([data[0], data[1]]) as u32),
                4 => Some(u32::from_le_bytes([data[0], data[1], data[2], data[3]])),
                _ => None,
            }
        }
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
        ExifValue::Undefined(data) => {
            // Handle common case where Undefined data is actually a null-terminated string
            if let Some(null_pos) = data.iter().position(|&b| b == 0) {
                // Data contains null terminator - treat as string
                match std::str::from_utf8(&data[..null_pos]) {
                    Ok(s) => s.to_string(),
                    Err(_) => format!("Undefined({})", data.len()),
                }
            } else {
                // No null terminator - check if it's printable ASCII
                if data
                    .iter()
                    .all(|&b| b.is_ascii() && (b.is_ascii_graphic() || b.is_ascii_whitespace()))
                {
                    match std::str::from_utf8(data) {
                        Ok(s) => s.to_string(),
                        Err(_) => format!("Undefined({})", data.len()),
                    }
                } else {
                    format!("Undefined({})", data.len())
                }
            }
        }
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

/// EXIF exposure time formatting - converts rational to "1/125" format
fn format_exposure_time(value: &ExifValue) -> String {
    match value {
        ExifValue::Rational(num, den) => {
            if *den == 1 {
                format!("{}", num)
            } else if *num == 1 {
                format!("1/{}", den)
            } else {
                let exposure = *num as f64 / *den as f64;
                if exposure >= 1.0 {
                    format!("{:.1}", exposure)
                } else {
                    format!("1/{:.0}", 1.0 / exposure)
                }
            }
        }
        _ => exif_value_to_string(value),
    }
}

/// EXIF F-number formatting - converts rational to "f/4.0" format
fn format_f_number(value: &ExifValue) -> String {
    match value {
        ExifValue::Rational(num, den) => {
            if *den == 0 {
                "undef".to_string()
            } else {
                let f_val = *num as f64 / *den as f64;
                format!("f/{:.1}", f_val)
            }
        }
        _ => exif_value_to_string(value),
    }
}

/// EXIF focal length formatting - converts rational to "23.0 mm" format
fn format_focal_length(value: &ExifValue) -> String {
    match value {
        ExifValue::Rational(num, den) => {
            if *den == 0 {
                "undef".to_string()
            } else {
                let focal_length = *num as f64 / *den as f64;
                format!("{:.1} mm", focal_length)
            }
        }
        _ => exif_value_to_string(value),
    }
}

/// EXIF orientation formatting - converts numeric value to orientation description
fn format_orientation(value: &ExifValue) -> String {
    match as_u32(value) {
        Some(1) => "Horizontal (normal)".to_string(),
        Some(2) => "Mirror horizontal".to_string(),
        Some(3) => "Rotate 180".to_string(),
        Some(4) => "Mirror vertical".to_string(),
        Some(5) => "Mirror horizontal and rotate 270 CW".to_string(),
        Some(6) => "Rotate 90 CW".to_string(),
        Some(7) => "Mirror horizontal and rotate 90 CW".to_string(),
        Some(8) => "Rotate 270 CW".to_string(),
        _ => format!("Unknown ({})", exif_value_to_string(value)),
    }
}

/// EXIF date/time formatting
fn format_datetime(value: &ExifValue) -> String {
    // For now, just return the string as-is
    // In a full implementation, this would handle various date formats
    exif_value_to_string(value)
}

/// EXIF resolution formatting - handles X and Y resolution values
fn format_resolution(value: &ExifValue) -> String {
    match value {
        ExifValue::Rational(num, den) => {
            if *den == 0 {
                "undef".to_string()
            } else {
                let resolution = *num as f64 / *den as f64;
                format!("{:.0}", resolution)
            }
        }
        _ => exif_value_to_string(value),
    }
}

/// EXIF compression formatting - converts numeric compression values to names
fn format_compression(value: &ExifValue) -> String {
    match as_u32(value) {
        Some(1) => "Uncompressed".to_string(),
        Some(2) => "CCITT 1D".to_string(),
        Some(3) => "T4/Group 3 Fax".to_string(),
        Some(4) => "T6/Group 4 Fax".to_string(),
        Some(5) => "LZW".to_string(),
        Some(6) => "JPEG (old-style)".to_string(),
        Some(7) => "JPEG".to_string(),
        Some(8) => "Adobe Deflate".to_string(),
        Some(9) => "JBIG B&W".to_string(),
        Some(10) => "JBIG Color".to_string(),
        Some(32766) => "Next".to_string(),
        Some(32767) => "Sony ARW Compressed".to_string(),
        Some(32773) => "PackBits".to_string(),
        Some(34712) => "JPEG2000".to_string(),
        _ => format!("Unknown ({})", exif_value_to_string(value)),
    }
}

/// EXIF color space formatting - converts numeric color space values to names
fn format_color_space(value: &ExifValue) -> String {
    match as_u32(value) {
        Some(1) => "sRGB".to_string(),
        Some(2) => "Adobe RGB".to_string(),
        Some(65535) => "Uncalibrated".to_string(),
        _ => format!("Unknown ({})", exif_value_to_string(value)),
    }
}

/// Flash mode conversion - 24 standard EXIF flash values
/// EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm %flash hash
fn format_flash(value: &ExifValue) -> String {
    match as_u32(value) {
        Some(0x00) => "No Flash".to_string(),
        Some(0x01) => "Fired".to_string(),
        Some(0x05) => "Fired, Return not detected".to_string(),
        Some(0x07) => "Fired, Return detected".to_string(),
        Some(0x08) => "On, Did not fire".to_string(),
        Some(0x09) => "On, Fired".to_string(),
        Some(0x0d) => "On, Return not detected".to_string(),
        Some(0x0f) => "On, Return detected".to_string(),
        Some(0x10) => "Off, Did not fire".to_string(),
        Some(0x14) => "Off, Did not fire, Return not detected".to_string(),
        Some(0x18) => "Auto, Did not fire".to_string(),
        Some(0x19) => "Auto, Fired".to_string(),
        Some(0x1d) => "Auto, Fired, Return not detected".to_string(),
        Some(0x1f) => "Auto, Fired, Return detected".to_string(),
        Some(0x20) => "No flash function".to_string(),
        Some(0x30) => "Off, No flash function".to_string(),
        Some(0x41) => "Fired, Red-eye reduction".to_string(),
        Some(0x45) => "Fired, Red-eye reduction, Return not detected".to_string(),
        Some(0x47) => "Fired, Red-eye reduction, Return detected".to_string(),
        Some(0x49) => "On, Red-eye reduction".to_string(),
        Some(0x4d) => "On, Red-eye reduction, Return not detected".to_string(),
        Some(0x4f) => "On, Red-eye reduction, Return detected".to_string(),
        Some(0x50) => "Off, Red-eye reduction".to_string(),
        Some(0x58) => "Auto, Did not fire, Red-eye reduction".to_string(),
        Some(0x59) => "Auto, Fired, Red-eye reduction".to_string(),
        Some(0x5d) => "Auto, Fired, Red-eye reduction, Return not detected".to_string(),
        Some(0x5f) => "Auto, Fired, Red-eye reduction, Return detected".to_string(),
        _ => format!("Unknown ({})", exif_value_to_string(value)),
    }
}

/// Light source conversion for white balance
/// EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm %lightSource hash
fn format_light_source(value: &ExifValue) -> String {
    match as_u32(value) {
        Some(0) => "Unknown".to_string(),
        Some(1) => "Daylight".to_string(),
        Some(2) => "Fluorescent".to_string(),
        Some(3) => "Tungsten (Incandescent)".to_string(),
        Some(4) => "Flash".to_string(),
        Some(9) => "Fine Weather".to_string(),
        Some(10) => "Cloudy".to_string(),
        Some(11) => "Shade".to_string(),
        Some(12) => "Daylight Fluorescent".to_string(), // (D 5700 - 7100K)
        Some(13) => "Day White Fluorescent".to_string(), // (N 4600 - 5500K)
        Some(14) => "Cool White Fluorescent".to_string(), // (W 3800 - 4500K)
        Some(15) => "White Fluorescent".to_string(),    // (WW 3250 - 3800K)
        Some(16) => "Warm White Fluorescent".to_string(), // (L 2600 - 3250K)
        Some(17) => "Standard Light A".to_string(),
        Some(18) => "Standard Light B".to_string(),
        Some(19) => "Standard Light C".to_string(),
        Some(20) => "D55".to_string(),
        Some(21) => "D65".to_string(),
        Some(22) => "D75".to_string(),
        Some(23) => "D50".to_string(),
        Some(24) => "ISO Studio Tungsten".to_string(),
        Some(255) => "Other".to_string(),
        _ => format!("Unknown ({})", exif_value_to_string(value)),
    }
}

/// Exposure program conversion
/// EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm tag 0x8822 PrintConv
fn format_exposure_program(value: &ExifValue) -> String {
    match as_u32(value) {
        Some(0) => "Not Defined".to_string(),
        Some(1) => "Manual".to_string(),
        Some(2) => "Program AE".to_string(),
        Some(3) => "Aperture-priority AE".to_string(),
        Some(4) => "Shutter speed priority AE".to_string(),
        Some(5) => "Creative (Slow speed)".to_string(),
        Some(6) => "Action (High speed)".to_string(),
        Some(7) => "Portrait".to_string(),
        Some(8) => "Landscape".to_string(),
        Some(9) => "Bulb".to_string(),
        _ => format!("Unknown ({})", exif_value_to_string(value)),
    }
}

/// Metering mode conversion
/// EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm tag 0x9207 PrintConv
fn format_metering_mode(value: &ExifValue) -> String {
    match as_u32(value) {
        Some(0) => "Unknown".to_string(),
        Some(1) => "Average".to_string(),
        Some(2) => "Center-weighted average".to_string(),
        Some(3) => "Spot".to_string(),
        Some(4) => "Multi-spot".to_string(),
        Some(5) => "Multi-segment".to_string(),
        Some(6) => "Partial".to_string(),
        Some(255) => "Other".to_string(),
        _ => format!("Unknown ({})", exif_value_to_string(value)),
    }
}

/// EXIF color space conversion (different from generic ColorSpace)
/// EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm tag 0xa001 PrintConv
fn format_exif_color_space(value: &ExifValue) -> String {
    match as_u32(value) {
        Some(1) => "sRGB".to_string(),
        Some(2) => "Adobe RGB".to_string(),
        Some(0xffff) => "Uncalibrated".to_string(),
        Some(0xfffe) => "ICC Profile".to_string(), // Sony
        Some(0xfffd) => "Wide Gamut RGB".to_string(), // Sony
        _ => format!("Unknown ({})", exif_value_to_string(value)),
    }
}

/// Universal parameter conversion (Normal/Low/High pattern)
/// Used by Contrast (0xa408), Saturation (0xa409), Sharpness (0xa40a)
/// EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm multiple tags with same pattern
fn format_universal_parameter(value: &ExifValue) -> String {
    match as_u32(value) {
        Some(0) => "Normal".to_string(),
        Some(1) => "Low".to_string(),  // or "Soft" for Sharpness
        Some(2) => "High".to_string(), // or "Hard" for Sharpness
        _ => format!("Unknown ({})", exif_value_to_string(value)),
    }
}

/// EXIF white balance conversion  
/// EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm tag 0xa403 PrintConv
fn format_exif_white_balance(value: &ExifValue) -> String {
    match as_u32(value) {
        Some(0) => "Auto".to_string(),
        Some(1) => "Manual".to_string(),
        _ => format!("Unknown ({})", exif_value_to_string(value)),
    }
}

/// Exposure mode conversion
/// EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm tag 0xa402 PrintConv
fn format_exposure_mode(value: &ExifValue) -> String {
    match as_u32(value) {
        Some(0) => "Auto".to_string(),
        Some(1) => "Manual".to_string(),
        Some(2) => "Auto bracket".to_string(),
        _ => format!("Unknown ({})", exif_value_to_string(value)),
    }
}

/// Resolution unit conversion
/// EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm tag 0x0128 PrintConv
fn format_resolution_unit(value: &ExifValue) -> String {
    match as_u32(value) {
        Some(1) => "None".to_string(),
        Some(2) => "inches".to_string(),
        Some(3) => "cm".to_string(),
        _ => format!("Unknown ({})", exif_value_to_string(value)),
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

    #[test]
    fn test_flash_conversion() {
        // Test comprehensive flash mode conversion (24 standard EXIF values)
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0x00), PrintConvId::Flash),
            "No Flash"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0x01), PrintConvId::Flash),
            "Fired"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0x19), PrintConvId::Flash),
            "Auto, Fired"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0x59), PrintConvId::Flash),
            "Auto, Fired, Red-eye reduction"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0x5f), PrintConvId::Flash),
            "Auto, Fired, Red-eye reduction, Return detected"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0x99), PrintConvId::Flash),
            "Unknown (153)"
        );
    }

    #[test]
    fn test_light_source_conversion() {
        // Test light source for white balance
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0), PrintConvId::LightSource),
            "Unknown"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(1), PrintConvId::LightSource),
            "Daylight"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(3), PrintConvId::LightSource),
            "Tungsten (Incandescent)"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(21), PrintConvId::LightSource),
            "D65"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(255), PrintConvId::LightSource),
            "Other"
        );
    }

    #[test]
    fn test_exposure_program_conversion() {
        // Test exposure program modes
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0), PrintConvId::ExposureProgram),
            "Not Defined"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(1), PrintConvId::ExposureProgram),
            "Manual"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(2), PrintConvId::ExposureProgram),
            "Program AE"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(3), PrintConvId::ExposureProgram),
            "Aperture-priority AE"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(9), PrintConvId::ExposureProgram),
            "Bulb"
        );
    }

    #[test]
    fn test_metering_mode_conversion() {
        // Test metering mode conversion
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0), PrintConvId::MeteringMode),
            "Unknown"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(2), PrintConvId::MeteringMode),
            "Center-weighted average"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(3), PrintConvId::MeteringMode),
            "Spot"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(5), PrintConvId::MeteringMode),
            "Multi-segment"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(255), PrintConvId::MeteringMode),
            "Other"
        );
    }

    #[test]
    fn test_exif_color_space_conversion() {
        // Test EXIF-specific color space (includes Sony non-standard values)
        assert_eq!(
            apply_print_conv(&ExifValue::U32(1), PrintConvId::ExifColorSpace),
            "sRGB"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(2), PrintConvId::ExifColorSpace),
            "Adobe RGB"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0xffff), PrintConvId::ExifColorSpace),
            "Uncalibrated"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0xfffe), PrintConvId::ExifColorSpace),
            "ICC Profile"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0xfffd), PrintConvId::ExifColorSpace),
            "Wide Gamut RGB"
        );
    }

    #[test]
    fn test_universal_parameter_conversion() {
        // Test shared Normal/Low/High pattern used by Contrast, Saturation, Sharpness
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0), PrintConvId::UniversalParameter),
            "Normal"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(1), PrintConvId::UniversalParameter),
            "Low" // "Soft" for Sharpness
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(2), PrintConvId::UniversalParameter),
            "High" // "Hard" for Sharpness
        );
    }

    #[test]
    fn test_exif_white_balance_conversion() {
        // Test EXIF white balance (different from generic WhiteBalance)
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0), PrintConvId::ExifWhiteBalance),
            "Auto"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(1), PrintConvId::ExifWhiteBalance),
            "Manual"
        );
    }

    #[test]
    fn test_exposure_mode_conversion() {
        // Test exposure mode
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0), PrintConvId::ExposureMode),
            "Auto"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(1), PrintConvId::ExposureMode),
            "Manual"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(2), PrintConvId::ExposureMode),
            "Auto bracket"
        );
    }

    #[test]
    fn test_resolution_unit_conversion() {
        // Test resolution units
        assert_eq!(
            apply_print_conv(&ExifValue::U32(1), PrintConvId::ResolutionUnit),
            "None"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(2), PrintConvId::ResolutionUnit),
            "inches"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(3), PrintConvId::ResolutionUnit),
            "cm"
        );
    }

    #[test]
    fn test_universal_on_off_auto_conversion() {
        // Test UniversalOnOffAuto pattern (0=Off, 1=On, 2=Auto)
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0), PrintConvId::UniversalOnOffAuto),
            "Off"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(1), PrintConvId::UniversalOnOffAuto),
            "On"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(2), PrintConvId::UniversalOnOffAuto),
            "Auto"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(99), PrintConvId::UniversalOnOffAuto),
            "Unknown (99)"
        );

        // Test with different value types
        assert_eq!(
            apply_print_conv(&ExifValue::U16(1), PrintConvId::UniversalOnOffAuto),
            "On"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U8(2), PrintConvId::UniversalOnOffAuto),
            "Auto"
        );
    }

    #[test]
    fn test_universal_noise_reduction_conversion() {
        // Test UniversalNoiseReduction pattern (0=Off, 1=Low, 2=Normal, 3=High, 4=Auto)
        assert_eq!(
            apply_print_conv(&ExifValue::U32(0), PrintConvId::UniversalNoiseReduction),
            "Off"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(1), PrintConvId::UniversalNoiseReduction),
            "Low"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(2), PrintConvId::UniversalNoiseReduction),
            "Normal"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(3), PrintConvId::UniversalNoiseReduction),
            "High"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(4), PrintConvId::UniversalNoiseReduction),
            "Auto"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U32(99), PrintConvId::UniversalNoiseReduction),
            "Unknown (99)"
        );

        // Test with different value types
        assert_eq!(
            apply_print_conv(&ExifValue::U16(3), PrintConvId::UniversalNoiseReduction),
            "High"
        );
        assert_eq!(
            apply_print_conv(&ExifValue::U8(1), PrintConvId::UniversalNoiseReduction),
            "Low"
        );
    }
}
