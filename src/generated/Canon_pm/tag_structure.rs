//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

//! Canon tag table structure generated from Canon.pm
//! ExifTool: Canon.pm %Canon::Main
//! Generated at: Sat Jul 19 16:13:39 2025 GMT

/// Canon data types from %Canon::Main table
/// Total tags: 84 (conditional: 6, with subdirectories: 43)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanonDataType {
    /// 0x0001: CanonCameraSettings
    /// ExifTool: SubDirectory -> CameraSettings
    CameraSettings,
    /// 0x0002: CanonFocalLength
    /// ExifTool: SubDirectory -> FocalLength
    FocalLength,
    /// 0x0003: CanonFlashInfo
    FlashInfo,
    /// 0x0004: CanonShotInfo
    /// ExifTool: SubDirectory -> ShotInfo
    ShotInfo,
    /// 0x0005: CanonPanorama
    /// ExifTool: SubDirectory -> Panorama
    Panorama,
    /// 0x0006: CanonImageType
    ImageType,
    /// 0x0007: CanonFirmwareVersion
    FirmwareVersion,
    /// 0x0008: FileNumber
    FileNumber,
    /// 0x0009: OwnerName
    OwnerName,
    /// 0x000a: UnknownD30
    /// ExifTool: SubDirectory -> UnknownD30
    UnknownD30,
    /// 0x000c: SerialNumber
    SerialNumber,

    /// 0x000d: CanonCameraInfo1D
    CameraInfo1D,

    /// 0x000e: CanonFileLength
    FileLength,
    /// 0x000f: CustomFunctions1D
    CustomFunctions1D,

    /// 0x0010: CanonModelID
    ModelID,
    /// 0x0011: MovieInfo
    /// ExifTool: SubDirectory -> MovieInfo
    MovieInfo,
    /// 0x0012: CanonAFInfo
    /// ExifTool: SubDirectory -> AFInfo
    AFInfo,
    /// 0x0013: ThumbnailImageValidArea
    ThumbnailImageValidArea,
    /// 0x0015: SerialNumberFormat
    SerialNumberFormat,
    /// 0x001a: SuperMacro
    SuperMacro,
    /// 0x001c: DateStampMode
    DateStampMode,
    /// 0x001d: MyColors
    /// ExifTool: SubDirectory -> MyColors
    MyColors,
    /// 0x001e: FirmwareRevision
    FirmwareRevision,
    /// 0x0023: Categories
    Categories,
    /// 0x0024: FaceDetect1
    /// ExifTool: SubDirectory -> FaceDetect1
    FaceDetect1,
    /// 0x0025: FaceDetect2
    /// ExifTool: SubDirectory -> FaceDetect2
    FaceDetect2,
    /// 0x0026: CanonAFInfo2
    /// ExifTool: SubDirectory -> AFInfo2
    AFInfo2,
    /// 0x0027: ContrastInfo
    /// ExifTool: SubDirectory -> ContrastInfo
    ContrastInfo,
    /// 0x0028: ImageUniqueID
    ImageUniqueID,
    /// 0x0029: WBInfo
    /// ExifTool: SubDirectory -> WBInfo
    WBInfo,
    /// 0x002f: FaceDetect3
    /// ExifTool: SubDirectory -> FaceDetect3
    FaceDetect3,
    /// 0x0035: TimeInfo
    /// ExifTool: SubDirectory -> TimeInfo
    TimeInfo,
    /// 0x0038: BatteryType
    BatteryType,
    /// 0x003c: AFInfo3
    /// ExifTool: SubDirectory -> AFInfo2
    AFInfo3,
    /// 0x0081: RawDataOffset
    RawDataOffset,
    /// 0x0082: RawDataLength
    RawDataLength,
    /// 0x0083: OriginalDecisionDataOffset
    OriginalDecisionDataOffset,
    /// 0x0090: CustomFunctions1D
    /// ExifTool: SubDirectory -> CanonCustom::Functions1D
    CustomFunctions1D2,
    /// 0x0091: PersonalFunctions
    /// ExifTool: SubDirectory -> CanonCustom::PersonalFuncs
    PersonalFunctions,
    /// 0x0092: PersonalFunctionValues
    /// ExifTool: SubDirectory -> CanonCustom::PersonalFuncValues
    PersonalFunctionValues,
    /// 0x0093: CanonFileInfo
    /// ExifTool: SubDirectory -> FileInfo
    FileInfo,
    /// 0x0094: AFPointsInFocus1D
    AFPointsInFocus1D,
    /// 0x0095: LensModel
    LensModel,
    /// 0x0096: SerialInfo
    SerialInfo,

    /// 0x0097: DustRemovalData
    DustRemovalData,
    /// 0x0098: CropInfo
    /// ExifTool: SubDirectory -> CropInfo
    CropInfo,
    /// 0x0099: CustomFunctions2
    /// ExifTool: SubDirectory -> CanonCustom::Functions2
    CustomFunctions2,
    /// 0x009a: AspectInfo
    /// ExifTool: SubDirectory -> AspectInfo
    AspectInfo,
    /// 0x00a0: ProcessingInfo
    /// ExifTool: SubDirectory -> Processing
    ProcessingInfo,
    /// 0x00a1: ToneCurveTable
    ToneCurveTable,
    /// 0x00a2: SharpnessTable
    SharpnessTable,
    /// 0x00a3: SharpnessFreqTable
    SharpnessFreqTable,
    /// 0x00a4: WhiteBalanceTable
    WhiteBalanceTable,
    /// 0x00a9: ColorBalance
    /// ExifTool: SubDirectory -> ColorBalance
    ColorBalance,
    /// 0x00aa: MeasuredColor
    /// ExifTool: SubDirectory -> MeasuredColor
    MeasuredColor,
    /// 0x00ae: ColorTemperature
    ColorTemperature,
    /// 0x00b0: CanonFlags
    /// ExifTool: SubDirectory -> Flags
    Flags,
    /// 0x00b1: ModifiedInfo
    /// ExifTool: SubDirectory -> ModifiedInfo
    ModifiedInfo,
    /// 0x00b2: ToneCurveMatching
    ToneCurveMatching,
    /// 0x00b3: WhiteBalanceMatching
    WhiteBalanceMatching,
    /// 0x00b4: ColorSpace
    ColorSpace,
    /// 0x00b6: PreviewImageInfo
    /// ExifTool: SubDirectory -> PreviewImageInfo
    PreviewImageInfo,
    /// 0x00d0: VRDOffset
    VRDOffset,
    /// 0x00e0: SensorInfo
    /// ExifTool: SubDirectory -> SensorInfo
    SensorInfo,
    /// 0x4001: ColorData1
    ColorData1,

    /// 0x4002: CRWParam
    CRWParam,
    /// 0x4003: ColorInfo
    /// ExifTool: SubDirectory -> ColorInfo
    ColorInfo,
    /// 0x4005: Flavor
    Flavor,
    /// 0x4008: PictureStyleUserDef
    PictureStyleUserDef,
    /// 0x4009: PictureStylePC
    PictureStylePC,
    /// 0x4010: CustomPictureStyleFileName
    CustomPictureStyleFileName,
    /// 0x4013: AFMicroAdj
    /// ExifTool: SubDirectory -> AFMicroAdj
    AFMicroAdj,
    /// 0x4015: VignettingCorr
    VignettingCorr,

    /// 0x4016: VignettingCorr2
    /// ExifTool: SubDirectory -> VignettingCorr2
    VignettingCorr2,
    /// 0x4018: LightingOpt
    /// ExifTool: SubDirectory -> LightingOpt
    LightingOpt,
    /// 0x4019: LensInfo
    /// ExifTool: SubDirectory -> LensInfo
    LensInfo,
    /// 0x4020: AmbienceInfo
    /// ExifTool: SubDirectory -> Ambience
    AmbienceInfo,
    /// 0x4021: MultiExp
    /// ExifTool: SubDirectory -> MultiExp
    MultiExp,
    /// 0x4024: FilterInfo
    /// ExifTool: SubDirectory -> FilterInfo
    FilterInfo,
    /// 0x4025: HDRInfo
    /// ExifTool: SubDirectory -> HDRInfo
    HDRInfo,
    /// 0x4026: LogInfo
    /// ExifTool: SubDirectory -> LogInfo
    LogInfo,
    /// 0x4028: AFConfig
    /// ExifTool: SubDirectory -> AFConfig
    AFConfig,
    /// 0x403f: RawBurstModeRoll
    /// ExifTool: SubDirectory -> RawBurstInfo
    RawBurstModeRoll,
    /// 0x4059: LevelInfo
    /// ExifTool: SubDirectory -> LevelInfo
    LevelInfo,
}

impl CanonDataType {
    /// Get tag ID for this data type
    pub fn tag_id(&self) -> u16 {
        match self {
            CanonDataType::CameraSettings => 0x0001,
            CanonDataType::FocalLength => 0x0002,
            CanonDataType::FlashInfo => 0x0003,
            CanonDataType::ShotInfo => 0x0004,
            CanonDataType::Panorama => 0x0005,
            CanonDataType::ImageType => 0x0006,
            CanonDataType::FirmwareVersion => 0x0007,
            CanonDataType::FileNumber => 0x0008,
            CanonDataType::OwnerName => 0x0009,
            CanonDataType::UnknownD30 => 0x000a,
            CanonDataType::SerialNumber => 0x000c,
            CanonDataType::CameraInfo1D => 0x000d,
            CanonDataType::FileLength => 0x000e,
            CanonDataType::CustomFunctions1D => 0x000f,
            CanonDataType::ModelID => 0x0010,
            CanonDataType::MovieInfo => 0x0011,
            CanonDataType::AFInfo => 0x0012,
            CanonDataType::ThumbnailImageValidArea => 0x0013,
            CanonDataType::SerialNumberFormat => 0x0015,
            CanonDataType::SuperMacro => 0x001a,
            CanonDataType::DateStampMode => 0x001c,
            CanonDataType::MyColors => 0x001d,
            CanonDataType::FirmwareRevision => 0x001e,
            CanonDataType::Categories => 0x0023,
            CanonDataType::FaceDetect1 => 0x0024,
            CanonDataType::FaceDetect2 => 0x0025,
            CanonDataType::AFInfo2 => 0x0026,
            CanonDataType::ContrastInfo => 0x0027,
            CanonDataType::ImageUniqueID => 0x0028,
            CanonDataType::WBInfo => 0x0029,
            CanonDataType::FaceDetect3 => 0x002f,
            CanonDataType::TimeInfo => 0x0035,
            CanonDataType::BatteryType => 0x0038,
            CanonDataType::AFInfo3 => 0x003c,
            CanonDataType::RawDataOffset => 0x0081,
            CanonDataType::RawDataLength => 0x0082,
            CanonDataType::OriginalDecisionDataOffset => 0x0083,
            CanonDataType::CustomFunctions1D2 => 0x0090,
            CanonDataType::PersonalFunctions => 0x0091,
            CanonDataType::PersonalFunctionValues => 0x0092,
            CanonDataType::FileInfo => 0x0093,
            CanonDataType::AFPointsInFocus1D => 0x0094,
            CanonDataType::LensModel => 0x0095,
            CanonDataType::SerialInfo => 0x0096,
            CanonDataType::DustRemovalData => 0x0097,
            CanonDataType::CropInfo => 0x0098,
            CanonDataType::CustomFunctions2 => 0x0099,
            CanonDataType::AspectInfo => 0x009a,
            CanonDataType::ProcessingInfo => 0x00a0,
            CanonDataType::ToneCurveTable => 0x00a1,
            CanonDataType::SharpnessTable => 0x00a2,
            CanonDataType::SharpnessFreqTable => 0x00a3,
            CanonDataType::WhiteBalanceTable => 0x00a4,
            CanonDataType::ColorBalance => 0x00a9,
            CanonDataType::MeasuredColor => 0x00aa,
            CanonDataType::ColorTemperature => 0x00ae,
            CanonDataType::Flags => 0x00b0,
            CanonDataType::ModifiedInfo => 0x00b1,
            CanonDataType::ToneCurveMatching => 0x00b2,
            CanonDataType::WhiteBalanceMatching => 0x00b3,
            CanonDataType::ColorSpace => 0x00b4,
            CanonDataType::PreviewImageInfo => 0x00b6,
            CanonDataType::VRDOffset => 0x00d0,
            CanonDataType::SensorInfo => 0x00e0,
            CanonDataType::ColorData1 => 0x4001,
            CanonDataType::CRWParam => 0x4002,
            CanonDataType::ColorInfo => 0x4003,
            CanonDataType::Flavor => 0x4005,
            CanonDataType::PictureStyleUserDef => 0x4008,
            CanonDataType::PictureStylePC => 0x4009,
            CanonDataType::CustomPictureStyleFileName => 0x4010,
            CanonDataType::AFMicroAdj => 0x4013,
            CanonDataType::VignettingCorr => 0x4015,
            CanonDataType::VignettingCorr2 => 0x4016,
            CanonDataType::LightingOpt => 0x4018,
            CanonDataType::LensInfo => 0x4019,
            CanonDataType::AmbienceInfo => 0x4020,
            CanonDataType::MultiExp => 0x4021,
            CanonDataType::FilterInfo => 0x4024,
            CanonDataType::HDRInfo => 0x4025,
            CanonDataType::LogInfo => 0x4026,
            CanonDataType::AFConfig => 0x4028,
            CanonDataType::RawBurstModeRoll => 0x403f,
            CanonDataType::LevelInfo => 0x4059,
        }
    }

    /// Get data type from tag ID
    pub fn from_tag_id(tag_id: u16) -> Option<CanonDataType> {
        match tag_id {
            0x0001 => Some(CanonDataType::CameraSettings),
            0x0002 => Some(CanonDataType::FocalLength),
            0x0003 => Some(CanonDataType::FlashInfo),
            0x0004 => Some(CanonDataType::ShotInfo),
            0x0005 => Some(CanonDataType::Panorama),
            0x0006 => Some(CanonDataType::ImageType),
            0x0007 => Some(CanonDataType::FirmwareVersion),
            0x0008 => Some(CanonDataType::FileNumber),
            0x0009 => Some(CanonDataType::OwnerName),
            0x000a => Some(CanonDataType::UnknownD30),
            0x000c => Some(CanonDataType::SerialNumber),
            0x000d => Some(CanonDataType::CameraInfo1D),
            0x000e => Some(CanonDataType::FileLength),
            0x000f => Some(CanonDataType::CustomFunctions1D),
            0x0010 => Some(CanonDataType::ModelID),
            0x0011 => Some(CanonDataType::MovieInfo),
            0x0012 => Some(CanonDataType::AFInfo),
            0x0013 => Some(CanonDataType::ThumbnailImageValidArea),
            0x0015 => Some(CanonDataType::SerialNumberFormat),
            0x001a => Some(CanonDataType::SuperMacro),
            0x001c => Some(CanonDataType::DateStampMode),
            0x001d => Some(CanonDataType::MyColors),
            0x001e => Some(CanonDataType::FirmwareRevision),
            0x0023 => Some(CanonDataType::Categories),
            0x0024 => Some(CanonDataType::FaceDetect1),
            0x0025 => Some(CanonDataType::FaceDetect2),
            0x0026 => Some(CanonDataType::AFInfo2),
            0x0027 => Some(CanonDataType::ContrastInfo),
            0x0028 => Some(CanonDataType::ImageUniqueID),
            0x0029 => Some(CanonDataType::WBInfo),
            0x002f => Some(CanonDataType::FaceDetect3),
            0x0035 => Some(CanonDataType::TimeInfo),
            0x0038 => Some(CanonDataType::BatteryType),
            0x003c => Some(CanonDataType::AFInfo3),
            0x0081 => Some(CanonDataType::RawDataOffset),
            0x0082 => Some(CanonDataType::RawDataLength),
            0x0083 => Some(CanonDataType::OriginalDecisionDataOffset),
            0x0090 => Some(CanonDataType::CustomFunctions1D2),
            0x0091 => Some(CanonDataType::PersonalFunctions),
            0x0092 => Some(CanonDataType::PersonalFunctionValues),
            0x0093 => Some(CanonDataType::FileInfo),
            0x0094 => Some(CanonDataType::AFPointsInFocus1D),
            0x0095 => Some(CanonDataType::LensModel),
            0x0096 => Some(CanonDataType::SerialInfo),
            0x0097 => Some(CanonDataType::DustRemovalData),
            0x0098 => Some(CanonDataType::CropInfo),
            0x0099 => Some(CanonDataType::CustomFunctions2),
            0x009a => Some(CanonDataType::AspectInfo),
            0x00a0 => Some(CanonDataType::ProcessingInfo),
            0x00a1 => Some(CanonDataType::ToneCurveTable),
            0x00a2 => Some(CanonDataType::SharpnessTable),
            0x00a3 => Some(CanonDataType::SharpnessFreqTable),
            0x00a4 => Some(CanonDataType::WhiteBalanceTable),
            0x00a9 => Some(CanonDataType::ColorBalance),
            0x00aa => Some(CanonDataType::MeasuredColor),
            0x00ae => Some(CanonDataType::ColorTemperature),
            0x00b0 => Some(CanonDataType::Flags),
            0x00b1 => Some(CanonDataType::ModifiedInfo),
            0x00b2 => Some(CanonDataType::ToneCurveMatching),
            0x00b3 => Some(CanonDataType::WhiteBalanceMatching),
            0x00b4 => Some(CanonDataType::ColorSpace),
            0x00b6 => Some(CanonDataType::PreviewImageInfo),
            0x00d0 => Some(CanonDataType::VRDOffset),
            0x00e0 => Some(CanonDataType::SensorInfo),
            0x4001 => Some(CanonDataType::ColorData1),
            0x4002 => Some(CanonDataType::CRWParam),
            0x4003 => Some(CanonDataType::ColorInfo),
            0x4005 => Some(CanonDataType::Flavor),
            0x4008 => Some(CanonDataType::PictureStyleUserDef),
            0x4009 => Some(CanonDataType::PictureStylePC),
            0x4010 => Some(CanonDataType::CustomPictureStyleFileName),
            0x4013 => Some(CanonDataType::AFMicroAdj),
            0x4015 => Some(CanonDataType::VignettingCorr),
            0x4016 => Some(CanonDataType::VignettingCorr2),
            0x4018 => Some(CanonDataType::LightingOpt),
            0x4019 => Some(CanonDataType::LensInfo),
            0x4020 => Some(CanonDataType::AmbienceInfo),
            0x4021 => Some(CanonDataType::MultiExp),
            0x4024 => Some(CanonDataType::FilterInfo),
            0x4025 => Some(CanonDataType::HDRInfo),
            0x4026 => Some(CanonDataType::LogInfo),
            0x4028 => Some(CanonDataType::AFConfig),
            0x403f => Some(CanonDataType::RawBurstModeRoll),
            0x4059 => Some(CanonDataType::LevelInfo),
            _ => None,
        }
    }

    /// Get the ExifTool tag name
    pub fn name(&self) -> &'static str {
        match self {
            CanonDataType::CameraSettings => "CanonCameraSettings",
            CanonDataType::FocalLength => "CanonFocalLength",
            CanonDataType::FlashInfo => "CanonFlashInfo",
            CanonDataType::ShotInfo => "CanonShotInfo",
            CanonDataType::Panorama => "CanonPanorama",
            CanonDataType::ImageType => "CanonImageType",
            CanonDataType::FirmwareVersion => "CanonFirmwareVersion",
            CanonDataType::FileNumber => "FileNumber",
            CanonDataType::OwnerName => "OwnerName",
            CanonDataType::UnknownD30 => "UnknownD30",
            CanonDataType::SerialNumber => "SerialNumber",
            CanonDataType::CameraInfo1D => "CanonCameraInfo1D",
            CanonDataType::FileLength => "CanonFileLength",
            CanonDataType::CustomFunctions1D => "CustomFunctions1D",
            CanonDataType::ModelID => "CanonModelID",
            CanonDataType::MovieInfo => "MovieInfo",
            CanonDataType::AFInfo => "CanonAFInfo",
            CanonDataType::ThumbnailImageValidArea => "ThumbnailImageValidArea",
            CanonDataType::SerialNumberFormat => "SerialNumberFormat",
            CanonDataType::SuperMacro => "SuperMacro",
            CanonDataType::DateStampMode => "DateStampMode",
            CanonDataType::MyColors => "MyColors",
            CanonDataType::FirmwareRevision => "FirmwareRevision",
            CanonDataType::Categories => "Categories",
            CanonDataType::FaceDetect1 => "FaceDetect1",
            CanonDataType::FaceDetect2 => "FaceDetect2",
            CanonDataType::AFInfo2 => "CanonAFInfo2",
            CanonDataType::ContrastInfo => "ContrastInfo",
            CanonDataType::ImageUniqueID => "ImageUniqueID",
            CanonDataType::WBInfo => "WBInfo",
            CanonDataType::FaceDetect3 => "FaceDetect3",
            CanonDataType::TimeInfo => "TimeInfo",
            CanonDataType::BatteryType => "BatteryType",
            CanonDataType::AFInfo3 => "AFInfo3",
            CanonDataType::RawDataOffset => "RawDataOffset",
            CanonDataType::RawDataLength => "RawDataLength",
            CanonDataType::OriginalDecisionDataOffset => "OriginalDecisionDataOffset",
            CanonDataType::CustomFunctions1D2 => "CustomFunctions1D",
            CanonDataType::PersonalFunctions => "PersonalFunctions",
            CanonDataType::PersonalFunctionValues => "PersonalFunctionValues",
            CanonDataType::FileInfo => "CanonFileInfo",
            CanonDataType::AFPointsInFocus1D => "AFPointsInFocus1D",
            CanonDataType::LensModel => "LensModel",
            CanonDataType::SerialInfo => "SerialInfo",
            CanonDataType::DustRemovalData => "DustRemovalData",
            CanonDataType::CropInfo => "CropInfo",
            CanonDataType::CustomFunctions2 => "CustomFunctions2",
            CanonDataType::AspectInfo => "AspectInfo",
            CanonDataType::ProcessingInfo => "ProcessingInfo",
            CanonDataType::ToneCurveTable => "ToneCurveTable",
            CanonDataType::SharpnessTable => "SharpnessTable",
            CanonDataType::SharpnessFreqTable => "SharpnessFreqTable",
            CanonDataType::WhiteBalanceTable => "WhiteBalanceTable",
            CanonDataType::ColorBalance => "ColorBalance",
            CanonDataType::MeasuredColor => "MeasuredColor",
            CanonDataType::ColorTemperature => "ColorTemperature",
            CanonDataType::Flags => "CanonFlags",
            CanonDataType::ModifiedInfo => "ModifiedInfo",
            CanonDataType::ToneCurveMatching => "ToneCurveMatching",
            CanonDataType::WhiteBalanceMatching => "WhiteBalanceMatching",
            CanonDataType::ColorSpace => "ColorSpace",
            CanonDataType::PreviewImageInfo => "PreviewImageInfo",
            CanonDataType::VRDOffset => "VRDOffset",
            CanonDataType::SensorInfo => "SensorInfo",
            CanonDataType::ColorData1 => "ColorData1",
            CanonDataType::CRWParam => "CRWParam",
            CanonDataType::ColorInfo => "ColorInfo",
            CanonDataType::Flavor => "Flavor",
            CanonDataType::PictureStyleUserDef => "PictureStyleUserDef",
            CanonDataType::PictureStylePC => "PictureStylePC",
            CanonDataType::CustomPictureStyleFileName => "CustomPictureStyleFileName",
            CanonDataType::AFMicroAdj => "AFMicroAdj",
            CanonDataType::VignettingCorr => "VignettingCorr",
            CanonDataType::VignettingCorr2 => "VignettingCorr2",
            CanonDataType::LightingOpt => "LightingOpt",
            CanonDataType::LensInfo => "LensInfo",
            CanonDataType::AmbienceInfo => "AmbienceInfo",
            CanonDataType::MultiExp => "MultiExp",
            CanonDataType::FilterInfo => "FilterInfo",
            CanonDataType::HDRInfo => "HDRInfo",
            CanonDataType::LogInfo => "LogInfo",
            CanonDataType::AFConfig => "AFConfig",
            CanonDataType::RawBurstModeRoll => "RawBurstModeRoll",
            CanonDataType::LevelInfo => "LevelInfo",
        }
    }

    /// Check if this tag has a subdirectory
    pub fn has_subdirectory(&self) -> bool {
        matches!(
            self,
            CanonDataType::CameraSettings
                | CanonDataType::FocalLength
                | CanonDataType::ShotInfo
                | CanonDataType::Panorama
                | CanonDataType::UnknownD30
                | CanonDataType::MovieInfo
                | CanonDataType::AFInfo
                | CanonDataType::MyColors
                | CanonDataType::FaceDetect1
                | CanonDataType::FaceDetect2
                | CanonDataType::AFInfo2
                | CanonDataType::ContrastInfo
                | CanonDataType::WBInfo
                | CanonDataType::FaceDetect3
                | CanonDataType::TimeInfo
                | CanonDataType::AFInfo3
                | CanonDataType::CustomFunctions1D2
                | CanonDataType::PersonalFunctions
                | CanonDataType::PersonalFunctionValues
                | CanonDataType::FileInfo
                | CanonDataType::CropInfo
                | CanonDataType::CustomFunctions2
                | CanonDataType::AspectInfo
                | CanonDataType::ProcessingInfo
                | CanonDataType::ColorBalance
                | CanonDataType::MeasuredColor
                | CanonDataType::Flags
                | CanonDataType::ModifiedInfo
                | CanonDataType::PreviewImageInfo
                | CanonDataType::SensorInfo
                | CanonDataType::ColorInfo
                | CanonDataType::AFMicroAdj
                | CanonDataType::VignettingCorr2
                | CanonDataType::LightingOpt
                | CanonDataType::LensInfo
                | CanonDataType::AmbienceInfo
                | CanonDataType::MultiExp
                | CanonDataType::FilterInfo
                | CanonDataType::HDRInfo
                | CanonDataType::LogInfo
                | CanonDataType::AFConfig
                | CanonDataType::RawBurstModeRoll
                | CanonDataType::LevelInfo
        )
    }

    /// Get the group hierarchy for this tag
    pub fn groups(&self) -> (&'static str, &'static str) {
        match self {
            CanonDataType::CameraSettings => ("MakerNotes", "Camera"),
            CanonDataType::FocalLength => ("MakerNotes", "Camera"),
            CanonDataType::FlashInfo => ("MakerNotes", "Camera"),
            CanonDataType::ShotInfo => ("MakerNotes", "Camera"),
            CanonDataType::Panorama => ("MakerNotes", "Camera"),
            CanonDataType::ImageType => ("Image", "Camera"),
            CanonDataType::FirmwareVersion => ("MakerNotes", "Camera"),
            CanonDataType::FileNumber => ("Image", "Camera"),
            CanonDataType::OwnerName => ("MakerNotes", "Camera"),
            CanonDataType::UnknownD30 => ("MakerNotes", "Camera"),
            CanonDataType::SerialNumber => ("MakerNotes", "Camera"),
            CanonDataType::CameraInfo1D => ("MakerNotes", "Camera"),
            CanonDataType::FileLength => ("Image", "Camera"),
            CanonDataType::CustomFunctions1D => ("MakerNotes", "Camera"),
            CanonDataType::ModelID => ("MakerNotes", "Camera"),
            CanonDataType::MovieInfo => ("MakerNotes", "Camera"),
            CanonDataType::AFInfo => ("MakerNotes", "Camera"),
            CanonDataType::ThumbnailImageValidArea => ("MakerNotes", "Camera"),
            CanonDataType::SerialNumberFormat => ("MakerNotes", "Camera"),
            CanonDataType::SuperMacro => ("MakerNotes", "Camera"),
            CanonDataType::DateStampMode => ("MakerNotes", "Camera"),
            CanonDataType::MyColors => ("MakerNotes", "Camera"),
            CanonDataType::FirmwareRevision => ("MakerNotes", "Camera"),
            CanonDataType::Categories => ("MakerNotes", "Camera"),
            CanonDataType::FaceDetect1 => ("MakerNotes", "Camera"),
            CanonDataType::FaceDetect2 => ("MakerNotes", "Camera"),
            CanonDataType::AFInfo2 => ("MakerNotes", "Camera"),
            CanonDataType::ContrastInfo => ("MakerNotes", "Camera"),
            CanonDataType::ImageUniqueID => ("Image", "Camera"),
            CanonDataType::WBInfo => ("MakerNotes", "Camera"),
            CanonDataType::FaceDetect3 => ("MakerNotes", "Camera"),
            CanonDataType::TimeInfo => ("MakerNotes", "Camera"),
            CanonDataType::BatteryType => ("MakerNotes", "Camera"),
            CanonDataType::AFInfo3 => ("MakerNotes", "Camera"),
            CanonDataType::RawDataOffset => ("MakerNotes", "Camera"),
            CanonDataType::RawDataLength => ("MakerNotes", "Camera"),
            CanonDataType::OriginalDecisionDataOffset => ("MakerNotes", "Camera"),
            CanonDataType::CustomFunctions1D2 => ("MakerNotes", "Camera"),
            CanonDataType::PersonalFunctions => ("MakerNotes", "Camera"),
            CanonDataType::PersonalFunctionValues => ("MakerNotes", "Camera"),
            CanonDataType::FileInfo => ("MakerNotes", "Camera"),
            CanonDataType::AFPointsInFocus1D => ("MakerNotes", "Camera"),
            CanonDataType::LensModel => ("MakerNotes", "Camera"),
            CanonDataType::SerialInfo => ("MakerNotes", "Camera"),
            CanonDataType::DustRemovalData => ("MakerNotes", "Camera"),
            CanonDataType::CropInfo => ("MakerNotes", "Camera"),
            CanonDataType::CustomFunctions2 => ("MakerNotes", "Camera"),
            CanonDataType::AspectInfo => ("MakerNotes", "Camera"),
            CanonDataType::ProcessingInfo => ("MakerNotes", "Camera"),
            CanonDataType::ToneCurveTable => ("MakerNotes", "Camera"),
            CanonDataType::SharpnessTable => ("MakerNotes", "Camera"),
            CanonDataType::SharpnessFreqTable => ("MakerNotes", "Camera"),
            CanonDataType::WhiteBalanceTable => ("MakerNotes", "Camera"),
            CanonDataType::ColorBalance => ("MakerNotes", "Camera"),
            CanonDataType::MeasuredColor => ("MakerNotes", "Camera"),
            CanonDataType::ColorTemperature => ("MakerNotes", "Camera"),
            CanonDataType::Flags => ("MakerNotes", "Camera"),
            CanonDataType::ModifiedInfo => ("MakerNotes", "Camera"),
            CanonDataType::ToneCurveMatching => ("MakerNotes", "Camera"),
            CanonDataType::WhiteBalanceMatching => ("MakerNotes", "Camera"),
            CanonDataType::ColorSpace => ("MakerNotes", "Camera"),
            CanonDataType::PreviewImageInfo => ("MakerNotes", "Camera"),
            CanonDataType::VRDOffset => ("MakerNotes", "Camera"),
            CanonDataType::SensorInfo => ("MakerNotes", "Camera"),
            CanonDataType::ColorData1 => ("MakerNotes", "Camera"),
            CanonDataType::CRWParam => ("MakerNotes", "Camera"),
            CanonDataType::ColorInfo => ("MakerNotes", "Camera"),
            CanonDataType::Flavor => ("MakerNotes", "Camera"),
            CanonDataType::PictureStyleUserDef => ("MakerNotes", "Camera"),
            CanonDataType::PictureStylePC => ("MakerNotes", "Camera"),
            CanonDataType::CustomPictureStyleFileName => ("MakerNotes", "Camera"),
            CanonDataType::AFMicroAdj => ("MakerNotes", "Camera"),
            CanonDataType::VignettingCorr => ("MakerNotes", "Camera"),
            CanonDataType::VignettingCorr2 => ("MakerNotes", "Camera"),
            CanonDataType::LightingOpt => ("MakerNotes", "Camera"),
            CanonDataType::LensInfo => ("MakerNotes", "Camera"),
            CanonDataType::AmbienceInfo => ("MakerNotes", "Camera"),
            CanonDataType::MultiExp => ("MakerNotes", "Camera"),
            CanonDataType::FilterInfo => ("MakerNotes", "Camera"),
            CanonDataType::HDRInfo => ("MakerNotes", "Camera"),
            CanonDataType::LogInfo => ("MakerNotes", "Camera"),
            CanonDataType::AFConfig => ("MakerNotes", "Camera"),
            CanonDataType::RawBurstModeRoll => ("MakerNotes", "Camera"),
            CanonDataType::LevelInfo => ("MakerNotes", "Camera"),
        }
    }
}
