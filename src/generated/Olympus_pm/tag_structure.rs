//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

//! Olympus tag table structure generated from Olympus.pm
//! ExifTool: Olympus.pm %Olympus::Main
//! Generated at: Sat Jul 19 20:59:48 2025 GMT

/// Olympus data types from %Olympus::Main table
/// Total tags: 119 (conditional: 18, with subdirectories: 4)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OlympusDataType {
    /// 0x0000: MakerNoteVersion
    MakerNoteVersion,
    /// 0x0001: MinoltaCameraSettingsOld
    /// ExifTool: SubDirectory -> Minolta::CameraSettings
    MinoltaCameraSettingsOld,
    /// 0x0003: MinoltaCameraSettings
    /// ExifTool: SubDirectory -> Minolta::CameraSettings
    MinoltaCameraSettings,
    /// 0x0040: CompressedImageSize
    CompressedImageSize,
    /// 0x0081: PreviewImageData
    PreviewImageData,
    /// 0x0088: PreviewImageStart
    PreviewImageStart,
    /// 0x0089: PreviewImageLength
    PreviewImageLength,
    /// 0x0100: ThumbnailImage
    ThumbnailImage,
    /// 0x0104: BodyFirmwareVersion
    BodyFirmwareVersion,
    /// 0x0200: SpecialMode
    SpecialMode,
    /// 0x0201: Quality
    Quality,
    /// 0x0202: Macro
    Macro,
    /// 0x0203: BWMode
    /// Black And White Mode
    BWMode,
    /// 0x0204: DigitalZoom
    DigitalZoom,
    /// 0x0205: FocalPlaneDiagonal
    FocalPlaneDiagonal,
    /// 0x0206: LensDistortionParams
    LensDistortionParams,
    /// 0x0207: CameraType
    CameraType,
    /// 0x0208: TextInfo
    /// ExifTool: SubDirectory -> TextInfo
    TextInfo,
    /// 0x0209: CameraID
    CameraID,
    /// 0x020b: EpsonImageWidth
    EpsonImageWidth,
    /// 0x020c: EpsonImageHeight
    EpsonImageHeight,
    /// 0x020d: EpsonSoftware
    EpsonSoftware,
    /// 0x0280: PreviewImage
    PreviewImage,
    /// 0x0300: PreCaptureFrames
    PreCaptureFrames,
    /// 0x0301: WhiteBoard
    WhiteBoard,
    /// 0x0302: OneTouchWB
    OneTouchWB,
    /// 0x0303: WhiteBalanceBracket
    WhiteBalanceBracket,
    /// 0x0304: WhiteBalanceBias
    WhiteBalanceBias,
    /// 0x0400: SensorArea
    SensorArea,
    /// 0x0401: BlackLevel
    BlackLevel,
    /// 0x0403: SceneMode
    SceneMode,
    /// 0x0404: SerialNumber
    SerialNumber,
    /// 0x0405: Firmware
    Firmware,
    /// 0x0e00: PrintIM
    /// Print Image Matching
    /// ExifTool: SubDirectory -> PrintIM::Main
    PrintIM,
    /// 0x0f00: DataDump
    DataDump,
    /// 0x0f01: DataDump2
    DataDump2,
    /// 0x0f04: ZoomedPreviewStart
    ZoomedPreviewStart,
    /// 0x0f05: ZoomedPreviewLength
    ZoomedPreviewLength,
    /// 0x0f06: ZoomedPreviewSize
    ZoomedPreviewSize,
    /// 0x1000: ShutterSpeedValue
    ShutterSpeedValue,
    /// 0x1001: ISOValue
    ISOValue,
    /// 0x1002: ApertureValue
    ApertureValue,
    /// 0x1003: BrightnessValue
    BrightnessValue,
    /// 0x1004: FlashMode
    FlashMode,
    /// 0x1005: FlashDevice
    FlashDevice,
    /// 0x1006: ExposureCompensation
    ExposureCompensation,
    /// 0x1007: SensorTemperature
    SensorTemperature,
    /// 0x1008: LensTemperature
    LensTemperature,
    /// 0x1009: LightCondition
    LightCondition,
    /// 0x100a: FocusRange
    FocusRange,
    /// 0x100b: FocusMode
    FocusMode,
    /// 0x100c: ManualFocusDistance
    ManualFocusDistance,
    /// 0x100d: ZoomStepCount
    ZoomStepCount,
    /// 0x100e: FocusStepCount
    FocusStepCount,
    /// 0x100f: Sharpness
    Sharpness,
    /// 0x1010: FlashChargeLevel
    FlashChargeLevel,
    /// 0x1011: ColorMatrix
    ColorMatrix,
    /// 0x1012: BlackLevel
    BlackLevel2,
    /// 0x1013: ColorTemperatureBG
    ColorTemperatureBG,
    /// 0x1014: ColorTemperatureRG
    ColorTemperatureRG,
    /// 0x1015: WBMode
    WBMode,
    /// 0x1017: RedBalance
    RedBalance,
    /// 0x1018: BlueBalance
    BlueBalance,
    /// 0x1019: ColorMatrixNumber
    ColorMatrixNumber,
    /// 0x101a: SerialNumber
    SerialNumber2,
    /// 0x101b: ExternalFlashAE1_0
    ExternalFlashAE10,
    /// 0x101c: ExternalFlashAE2_0
    ExternalFlashAE20,
    /// 0x101d: InternalFlashAE1_0
    InternalFlashAE10,
    /// 0x101e: InternalFlashAE2_0
    InternalFlashAE20,
    /// 0x101f: ExternalFlashAE1
    ExternalFlashAE1,
    /// 0x1020: ExternalFlashAE2
    ExternalFlashAE2,
    /// 0x1021: InternalFlashAE1
    InternalFlashAE1,
    /// 0x1022: InternalFlashAE2
    InternalFlashAE2,
    /// 0x1023: FlashExposureComp
    FlashExposureComp,
    /// 0x1024: InternalFlashTable
    InternalFlashTable,
    /// 0x1025: ExternalFlashGValue
    ExternalFlashGValue,
    /// 0x1026: ExternalFlashBounce
    ExternalFlashBounce,
    /// 0x1027: ExternalFlashZoom
    ExternalFlashZoom,
    /// 0x1028: ExternalFlashMode
    ExternalFlashMode,
    /// 0x1029: Contrast
    Contrast,
    /// 0x102a: SharpnessFactor
    SharpnessFactor,
    /// 0x102b: ColorControl
    ColorControl,
    /// 0x102c: ValidBits
    ValidBits,
    /// 0x102d: CoringFilter
    CoringFilter,
    /// 0x102e: OlympusImageWidth
    ImageWidth,
    /// 0x102f: OlympusImageHeight
    ImageHeight,
    /// 0x1030: SceneDetect
    SceneDetect,
    /// 0x1031: SceneArea
    SceneArea,
    /// 0x1033: SceneDetectData
    SceneDetectData,
    /// 0x1034: CompressionRatio
    CompressionRatio,
    /// 0x1035: PreviewImageValid
    PreviewImageValid,
    /// 0x1036: PreviewImageStart
    PreviewImageStart2,
    /// 0x1037: PreviewImageLength
    PreviewImageLength2,
    /// 0x1038: AFResult
    AFResult,
    /// 0x1039: CCDScanMode
    CCDScanMode,
    /// 0x103a: NoiseReduction
    NoiseReduction,
    /// 0x103b: FocusStepInfinity
    FocusStepInfinity,
    /// 0x103c: FocusStepNear
    FocusStepNear,
    /// 0x103d: LightValueCenter
    LightValueCenter,
    /// 0x103e: LightValuePeriphery
    LightValuePeriphery,
    /// 0x103f: FieldCount
    FieldCount,
    /// 0x2010: Equipment
    Equipment,

    /// 0x2020: CameraSettings
    CameraSettings,

    /// 0x2030: RawDevelopment
    RawDevelopment,

    /// 0x2031: RawDev2
    RawDev2,

    /// 0x2040: ImageProcessing
    ImageProcessing,

    /// 0x2050: FocusInfo
    FocusInfo,

    /// 0x2100: Olympus2100
    _2100,

    /// 0x2200: Olympus2200
    _2200,

    /// 0x2300: Olympus2300
    _2300,

    /// 0x2400: Olympus2400
    _2400,

    /// 0x2500: Olympus2500
    _2500,

    /// 0x2600: Olympus2600
    _2600,

    /// 0x2700: Olympus2700
    _2700,

    /// 0x2800: Olympus2800
    _2800,

    /// 0x2900: Olympus2900
    _2900,

    /// 0x3000: RawInfo
    RawInfo,

    /// 0x4000: MainInfo
    MainInfo,

    /// 0x5000: UnknownInfo
    UnknownInfo,
}

impl OlympusDataType {
    /// Get tag ID for this data type
    pub fn tag_id(&self) -> u16 {
        match self {
            OlympusDataType::MakerNoteVersion => 0x0000,
            OlympusDataType::MinoltaCameraSettingsOld => 0x0001,
            OlympusDataType::MinoltaCameraSettings => 0x0003,
            OlympusDataType::CompressedImageSize => 0x0040,
            OlympusDataType::PreviewImageData => 0x0081,
            OlympusDataType::PreviewImageStart => 0x0088,
            OlympusDataType::PreviewImageLength => 0x0089,
            OlympusDataType::ThumbnailImage => 0x0100,
            OlympusDataType::BodyFirmwareVersion => 0x0104,
            OlympusDataType::SpecialMode => 0x0200,
            OlympusDataType::Quality => 0x0201,
            OlympusDataType::Macro => 0x0202,
            OlympusDataType::BWMode => 0x0203,
            OlympusDataType::DigitalZoom => 0x0204,
            OlympusDataType::FocalPlaneDiagonal => 0x0205,
            OlympusDataType::LensDistortionParams => 0x0206,
            OlympusDataType::CameraType => 0x0207,
            OlympusDataType::TextInfo => 0x0208,
            OlympusDataType::CameraID => 0x0209,
            OlympusDataType::EpsonImageWidth => 0x020b,
            OlympusDataType::EpsonImageHeight => 0x020c,
            OlympusDataType::EpsonSoftware => 0x020d,
            OlympusDataType::PreviewImage => 0x0280,
            OlympusDataType::PreCaptureFrames => 0x0300,
            OlympusDataType::WhiteBoard => 0x0301,
            OlympusDataType::OneTouchWB => 0x0302,
            OlympusDataType::WhiteBalanceBracket => 0x0303,
            OlympusDataType::WhiteBalanceBias => 0x0304,
            OlympusDataType::SensorArea => 0x0400,
            OlympusDataType::BlackLevel => 0x0401,
            OlympusDataType::SceneMode => 0x0403,
            OlympusDataType::SerialNumber => 0x0404,
            OlympusDataType::Firmware => 0x0405,
            OlympusDataType::PrintIM => 0x0e00,
            OlympusDataType::DataDump => 0x0f00,
            OlympusDataType::DataDump2 => 0x0f01,
            OlympusDataType::ZoomedPreviewStart => 0x0f04,
            OlympusDataType::ZoomedPreviewLength => 0x0f05,
            OlympusDataType::ZoomedPreviewSize => 0x0f06,
            OlympusDataType::ShutterSpeedValue => 0x1000,
            OlympusDataType::ISOValue => 0x1001,
            OlympusDataType::ApertureValue => 0x1002,
            OlympusDataType::BrightnessValue => 0x1003,
            OlympusDataType::FlashMode => 0x1004,
            OlympusDataType::FlashDevice => 0x1005,
            OlympusDataType::ExposureCompensation => 0x1006,
            OlympusDataType::SensorTemperature => 0x1007,
            OlympusDataType::LensTemperature => 0x1008,
            OlympusDataType::LightCondition => 0x1009,
            OlympusDataType::FocusRange => 0x100a,
            OlympusDataType::FocusMode => 0x100b,
            OlympusDataType::ManualFocusDistance => 0x100c,
            OlympusDataType::ZoomStepCount => 0x100d,
            OlympusDataType::FocusStepCount => 0x100e,
            OlympusDataType::Sharpness => 0x100f,
            OlympusDataType::FlashChargeLevel => 0x1010,
            OlympusDataType::ColorMatrix => 0x1011,
            OlympusDataType::BlackLevel2 => 0x1012,
            OlympusDataType::ColorTemperatureBG => 0x1013,
            OlympusDataType::ColorTemperatureRG => 0x1014,
            OlympusDataType::WBMode => 0x1015,
            OlympusDataType::RedBalance => 0x1017,
            OlympusDataType::BlueBalance => 0x1018,
            OlympusDataType::ColorMatrixNumber => 0x1019,
            OlympusDataType::SerialNumber2 => 0x101a,
            OlympusDataType::ExternalFlashAE10 => 0x101b,
            OlympusDataType::ExternalFlashAE20 => 0x101c,
            OlympusDataType::InternalFlashAE10 => 0x101d,
            OlympusDataType::InternalFlashAE20 => 0x101e,
            OlympusDataType::ExternalFlashAE1 => 0x101f,
            OlympusDataType::ExternalFlashAE2 => 0x1020,
            OlympusDataType::InternalFlashAE1 => 0x1021,
            OlympusDataType::InternalFlashAE2 => 0x1022,
            OlympusDataType::FlashExposureComp => 0x1023,
            OlympusDataType::InternalFlashTable => 0x1024,
            OlympusDataType::ExternalFlashGValue => 0x1025,
            OlympusDataType::ExternalFlashBounce => 0x1026,
            OlympusDataType::ExternalFlashZoom => 0x1027,
            OlympusDataType::ExternalFlashMode => 0x1028,
            OlympusDataType::Contrast => 0x1029,
            OlympusDataType::SharpnessFactor => 0x102a,
            OlympusDataType::ColorControl => 0x102b,
            OlympusDataType::ValidBits => 0x102c,
            OlympusDataType::CoringFilter => 0x102d,
            OlympusDataType::ImageWidth => 0x102e,
            OlympusDataType::ImageHeight => 0x102f,
            OlympusDataType::SceneDetect => 0x1030,
            OlympusDataType::SceneArea => 0x1031,
            OlympusDataType::SceneDetectData => 0x1033,
            OlympusDataType::CompressionRatio => 0x1034,
            OlympusDataType::PreviewImageValid => 0x1035,
            OlympusDataType::PreviewImageStart2 => 0x1036,
            OlympusDataType::PreviewImageLength2 => 0x1037,
            OlympusDataType::AFResult => 0x1038,
            OlympusDataType::CCDScanMode => 0x1039,
            OlympusDataType::NoiseReduction => 0x103a,
            OlympusDataType::FocusStepInfinity => 0x103b,
            OlympusDataType::FocusStepNear => 0x103c,
            OlympusDataType::LightValueCenter => 0x103d,
            OlympusDataType::LightValuePeriphery => 0x103e,
            OlympusDataType::FieldCount => 0x103f,
            OlympusDataType::Equipment => 0x2010,
            OlympusDataType::CameraSettings => 0x2020,
            OlympusDataType::RawDevelopment => 0x2030,
            OlympusDataType::RawDev2 => 0x2031,
            OlympusDataType::ImageProcessing => 0x2040,
            OlympusDataType::FocusInfo => 0x2050,
            OlympusDataType::_2100 => 0x2100,
            OlympusDataType::_2200 => 0x2200,
            OlympusDataType::_2300 => 0x2300,
            OlympusDataType::_2400 => 0x2400,
            OlympusDataType::_2500 => 0x2500,
            OlympusDataType::_2600 => 0x2600,
            OlympusDataType::_2700 => 0x2700,
            OlympusDataType::_2800 => 0x2800,
            OlympusDataType::_2900 => 0x2900,
            OlympusDataType::RawInfo => 0x3000,
            OlympusDataType::MainInfo => 0x4000,
            OlympusDataType::UnknownInfo => 0x5000,
        }
    }

    /// Get data type from tag ID
    pub fn from_tag_id(tag_id: u16) -> Option<OlympusDataType> {
        match tag_id {
            0x0000 => Some(OlympusDataType::MakerNoteVersion),
            0x0001 => Some(OlympusDataType::MinoltaCameraSettingsOld),
            0x0003 => Some(OlympusDataType::MinoltaCameraSettings),
            0x0040 => Some(OlympusDataType::CompressedImageSize),
            0x0081 => Some(OlympusDataType::PreviewImageData),
            0x0088 => Some(OlympusDataType::PreviewImageStart),
            0x0089 => Some(OlympusDataType::PreviewImageLength),
            0x0100 => Some(OlympusDataType::ThumbnailImage),
            0x0104 => Some(OlympusDataType::BodyFirmwareVersion),
            0x0200 => Some(OlympusDataType::SpecialMode),
            0x0201 => Some(OlympusDataType::Quality),
            0x0202 => Some(OlympusDataType::Macro),
            0x0203 => Some(OlympusDataType::BWMode),
            0x0204 => Some(OlympusDataType::DigitalZoom),
            0x0205 => Some(OlympusDataType::FocalPlaneDiagonal),
            0x0206 => Some(OlympusDataType::LensDistortionParams),
            0x0207 => Some(OlympusDataType::CameraType),
            0x0208 => Some(OlympusDataType::TextInfo),
            0x0209 => Some(OlympusDataType::CameraID),
            0x020b => Some(OlympusDataType::EpsonImageWidth),
            0x020c => Some(OlympusDataType::EpsonImageHeight),
            0x020d => Some(OlympusDataType::EpsonSoftware),
            0x0280 => Some(OlympusDataType::PreviewImage),
            0x0300 => Some(OlympusDataType::PreCaptureFrames),
            0x0301 => Some(OlympusDataType::WhiteBoard),
            0x0302 => Some(OlympusDataType::OneTouchWB),
            0x0303 => Some(OlympusDataType::WhiteBalanceBracket),
            0x0304 => Some(OlympusDataType::WhiteBalanceBias),
            0x0400 => Some(OlympusDataType::SensorArea),
            0x0401 => Some(OlympusDataType::BlackLevel),
            0x0403 => Some(OlympusDataType::SceneMode),
            0x0404 => Some(OlympusDataType::SerialNumber),
            0x0405 => Some(OlympusDataType::Firmware),
            0x0e00 => Some(OlympusDataType::PrintIM),
            0x0f00 => Some(OlympusDataType::DataDump),
            0x0f01 => Some(OlympusDataType::DataDump2),
            0x0f04 => Some(OlympusDataType::ZoomedPreviewStart),
            0x0f05 => Some(OlympusDataType::ZoomedPreviewLength),
            0x0f06 => Some(OlympusDataType::ZoomedPreviewSize),
            0x1000 => Some(OlympusDataType::ShutterSpeedValue),
            0x1001 => Some(OlympusDataType::ISOValue),
            0x1002 => Some(OlympusDataType::ApertureValue),
            0x1003 => Some(OlympusDataType::BrightnessValue),
            0x1004 => Some(OlympusDataType::FlashMode),
            0x1005 => Some(OlympusDataType::FlashDevice),
            0x1006 => Some(OlympusDataType::ExposureCompensation),
            0x1007 => Some(OlympusDataType::SensorTemperature),
            0x1008 => Some(OlympusDataType::LensTemperature),
            0x1009 => Some(OlympusDataType::LightCondition),
            0x100a => Some(OlympusDataType::FocusRange),
            0x100b => Some(OlympusDataType::FocusMode),
            0x100c => Some(OlympusDataType::ManualFocusDistance),
            0x100d => Some(OlympusDataType::ZoomStepCount),
            0x100e => Some(OlympusDataType::FocusStepCount),
            0x100f => Some(OlympusDataType::Sharpness),
            0x1010 => Some(OlympusDataType::FlashChargeLevel),
            0x1011 => Some(OlympusDataType::ColorMatrix),
            0x1012 => Some(OlympusDataType::BlackLevel2),
            0x1013 => Some(OlympusDataType::ColorTemperatureBG),
            0x1014 => Some(OlympusDataType::ColorTemperatureRG),
            0x1015 => Some(OlympusDataType::WBMode),
            0x1017 => Some(OlympusDataType::RedBalance),
            0x1018 => Some(OlympusDataType::BlueBalance),
            0x1019 => Some(OlympusDataType::ColorMatrixNumber),
            0x101a => Some(OlympusDataType::SerialNumber2),
            0x101b => Some(OlympusDataType::ExternalFlashAE10),
            0x101c => Some(OlympusDataType::ExternalFlashAE20),
            0x101d => Some(OlympusDataType::InternalFlashAE10),
            0x101e => Some(OlympusDataType::InternalFlashAE20),
            0x101f => Some(OlympusDataType::ExternalFlashAE1),
            0x1020 => Some(OlympusDataType::ExternalFlashAE2),
            0x1021 => Some(OlympusDataType::InternalFlashAE1),
            0x1022 => Some(OlympusDataType::InternalFlashAE2),
            0x1023 => Some(OlympusDataType::FlashExposureComp),
            0x1024 => Some(OlympusDataType::InternalFlashTable),
            0x1025 => Some(OlympusDataType::ExternalFlashGValue),
            0x1026 => Some(OlympusDataType::ExternalFlashBounce),
            0x1027 => Some(OlympusDataType::ExternalFlashZoom),
            0x1028 => Some(OlympusDataType::ExternalFlashMode),
            0x1029 => Some(OlympusDataType::Contrast),
            0x102a => Some(OlympusDataType::SharpnessFactor),
            0x102b => Some(OlympusDataType::ColorControl),
            0x102c => Some(OlympusDataType::ValidBits),
            0x102d => Some(OlympusDataType::CoringFilter),
            0x102e => Some(OlympusDataType::ImageWidth),
            0x102f => Some(OlympusDataType::ImageHeight),
            0x1030 => Some(OlympusDataType::SceneDetect),
            0x1031 => Some(OlympusDataType::SceneArea),
            0x1033 => Some(OlympusDataType::SceneDetectData),
            0x1034 => Some(OlympusDataType::CompressionRatio),
            0x1035 => Some(OlympusDataType::PreviewImageValid),
            0x1036 => Some(OlympusDataType::PreviewImageStart2),
            0x1037 => Some(OlympusDataType::PreviewImageLength2),
            0x1038 => Some(OlympusDataType::AFResult),
            0x1039 => Some(OlympusDataType::CCDScanMode),
            0x103a => Some(OlympusDataType::NoiseReduction),
            0x103b => Some(OlympusDataType::FocusStepInfinity),
            0x103c => Some(OlympusDataType::FocusStepNear),
            0x103d => Some(OlympusDataType::LightValueCenter),
            0x103e => Some(OlympusDataType::LightValuePeriphery),
            0x103f => Some(OlympusDataType::FieldCount),
            0x2010 => Some(OlympusDataType::Equipment),
            0x2020 => Some(OlympusDataType::CameraSettings),
            0x2030 => Some(OlympusDataType::RawDevelopment),
            0x2031 => Some(OlympusDataType::RawDev2),
            0x2040 => Some(OlympusDataType::ImageProcessing),
            0x2050 => Some(OlympusDataType::FocusInfo),
            0x2100 => Some(OlympusDataType::_2100),
            0x2200 => Some(OlympusDataType::_2200),
            0x2300 => Some(OlympusDataType::_2300),
            0x2400 => Some(OlympusDataType::_2400),
            0x2500 => Some(OlympusDataType::_2500),
            0x2600 => Some(OlympusDataType::_2600),
            0x2700 => Some(OlympusDataType::_2700),
            0x2800 => Some(OlympusDataType::_2800),
            0x2900 => Some(OlympusDataType::_2900),
            0x3000 => Some(OlympusDataType::RawInfo),
            0x4000 => Some(OlympusDataType::MainInfo),
            0x5000 => Some(OlympusDataType::UnknownInfo),
            _ => None,
        }
    }

    /// Get the ExifTool tag name
    pub fn name(&self) -> &'static str {
        match self {
            OlympusDataType::MakerNoteVersion => "MakerNoteVersion",
            OlympusDataType::MinoltaCameraSettingsOld => "MinoltaCameraSettingsOld",
            OlympusDataType::MinoltaCameraSettings => "MinoltaCameraSettings",
            OlympusDataType::CompressedImageSize => "CompressedImageSize",
            OlympusDataType::PreviewImageData => "PreviewImageData",
            OlympusDataType::PreviewImageStart => "PreviewImageStart",
            OlympusDataType::PreviewImageLength => "PreviewImageLength",
            OlympusDataType::ThumbnailImage => "ThumbnailImage",
            OlympusDataType::BodyFirmwareVersion => "BodyFirmwareVersion",
            OlympusDataType::SpecialMode => "SpecialMode",
            OlympusDataType::Quality => "Quality",
            OlympusDataType::Macro => "Macro",
            OlympusDataType::BWMode => "BWMode",
            OlympusDataType::DigitalZoom => "DigitalZoom",
            OlympusDataType::FocalPlaneDiagonal => "FocalPlaneDiagonal",
            OlympusDataType::LensDistortionParams => "LensDistortionParams",
            OlympusDataType::CameraType => "CameraType",
            OlympusDataType::TextInfo => "TextInfo",
            OlympusDataType::CameraID => "CameraID",
            OlympusDataType::EpsonImageWidth => "EpsonImageWidth",
            OlympusDataType::EpsonImageHeight => "EpsonImageHeight",
            OlympusDataType::EpsonSoftware => "EpsonSoftware",
            OlympusDataType::PreviewImage => "PreviewImage",
            OlympusDataType::PreCaptureFrames => "PreCaptureFrames",
            OlympusDataType::WhiteBoard => "WhiteBoard",
            OlympusDataType::OneTouchWB => "OneTouchWB",
            OlympusDataType::WhiteBalanceBracket => "WhiteBalanceBracket",
            OlympusDataType::WhiteBalanceBias => "WhiteBalanceBias",
            OlympusDataType::SensorArea => "SensorArea",
            OlympusDataType::BlackLevel => "BlackLevel",
            OlympusDataType::SceneMode => "SceneMode",
            OlympusDataType::SerialNumber => "SerialNumber",
            OlympusDataType::Firmware => "Firmware",
            OlympusDataType::PrintIM => "PrintIM",
            OlympusDataType::DataDump => "DataDump",
            OlympusDataType::DataDump2 => "DataDump2",
            OlympusDataType::ZoomedPreviewStart => "ZoomedPreviewStart",
            OlympusDataType::ZoomedPreviewLength => "ZoomedPreviewLength",
            OlympusDataType::ZoomedPreviewSize => "ZoomedPreviewSize",
            OlympusDataType::ShutterSpeedValue => "ShutterSpeedValue",
            OlympusDataType::ISOValue => "ISOValue",
            OlympusDataType::ApertureValue => "ApertureValue",
            OlympusDataType::BrightnessValue => "BrightnessValue",
            OlympusDataType::FlashMode => "FlashMode",
            OlympusDataType::FlashDevice => "FlashDevice",
            OlympusDataType::ExposureCompensation => "ExposureCompensation",
            OlympusDataType::SensorTemperature => "SensorTemperature",
            OlympusDataType::LensTemperature => "LensTemperature",
            OlympusDataType::LightCondition => "LightCondition",
            OlympusDataType::FocusRange => "FocusRange",
            OlympusDataType::FocusMode => "FocusMode",
            OlympusDataType::ManualFocusDistance => "ManualFocusDistance",
            OlympusDataType::ZoomStepCount => "ZoomStepCount",
            OlympusDataType::FocusStepCount => "FocusStepCount",
            OlympusDataType::Sharpness => "Sharpness",
            OlympusDataType::FlashChargeLevel => "FlashChargeLevel",
            OlympusDataType::ColorMatrix => "ColorMatrix",
            OlympusDataType::BlackLevel2 => "BlackLevel",
            OlympusDataType::ColorTemperatureBG => "ColorTemperatureBG",
            OlympusDataType::ColorTemperatureRG => "ColorTemperatureRG",
            OlympusDataType::WBMode => "WBMode",
            OlympusDataType::RedBalance => "RedBalance",
            OlympusDataType::BlueBalance => "BlueBalance",
            OlympusDataType::ColorMatrixNumber => "ColorMatrixNumber",
            OlympusDataType::SerialNumber2 => "SerialNumber",
            OlympusDataType::ExternalFlashAE10 => "ExternalFlashAE1_0",
            OlympusDataType::ExternalFlashAE20 => "ExternalFlashAE2_0",
            OlympusDataType::InternalFlashAE10 => "InternalFlashAE1_0",
            OlympusDataType::InternalFlashAE20 => "InternalFlashAE2_0",
            OlympusDataType::ExternalFlashAE1 => "ExternalFlashAE1",
            OlympusDataType::ExternalFlashAE2 => "ExternalFlashAE2",
            OlympusDataType::InternalFlashAE1 => "InternalFlashAE1",
            OlympusDataType::InternalFlashAE2 => "InternalFlashAE2",
            OlympusDataType::FlashExposureComp => "FlashExposureComp",
            OlympusDataType::InternalFlashTable => "InternalFlashTable",
            OlympusDataType::ExternalFlashGValue => "ExternalFlashGValue",
            OlympusDataType::ExternalFlashBounce => "ExternalFlashBounce",
            OlympusDataType::ExternalFlashZoom => "ExternalFlashZoom",
            OlympusDataType::ExternalFlashMode => "ExternalFlashMode",
            OlympusDataType::Contrast => "Contrast",
            OlympusDataType::SharpnessFactor => "SharpnessFactor",
            OlympusDataType::ColorControl => "ColorControl",
            OlympusDataType::ValidBits => "ValidBits",
            OlympusDataType::CoringFilter => "CoringFilter",
            OlympusDataType::ImageWidth => "OlympusImageWidth",
            OlympusDataType::ImageHeight => "OlympusImageHeight",
            OlympusDataType::SceneDetect => "SceneDetect",
            OlympusDataType::SceneArea => "SceneArea",
            OlympusDataType::SceneDetectData => "SceneDetectData",
            OlympusDataType::CompressionRatio => "CompressionRatio",
            OlympusDataType::PreviewImageValid => "PreviewImageValid",
            OlympusDataType::PreviewImageStart2 => "PreviewImageStart",
            OlympusDataType::PreviewImageLength2 => "PreviewImageLength",
            OlympusDataType::AFResult => "AFResult",
            OlympusDataType::CCDScanMode => "CCDScanMode",
            OlympusDataType::NoiseReduction => "NoiseReduction",
            OlympusDataType::FocusStepInfinity => "FocusStepInfinity",
            OlympusDataType::FocusStepNear => "FocusStepNear",
            OlympusDataType::LightValueCenter => "LightValueCenter",
            OlympusDataType::LightValuePeriphery => "LightValuePeriphery",
            OlympusDataType::FieldCount => "FieldCount",
            OlympusDataType::Equipment => "Equipment",
            OlympusDataType::CameraSettings => "CameraSettings",
            OlympusDataType::RawDevelopment => "RawDevelopment",
            OlympusDataType::RawDev2 => "RawDev2",
            OlympusDataType::ImageProcessing => "ImageProcessing",
            OlympusDataType::FocusInfo => "FocusInfo",
            OlympusDataType::_2100 => "Olympus2100",
            OlympusDataType::_2200 => "Olympus2200",
            OlympusDataType::_2300 => "Olympus2300",
            OlympusDataType::_2400 => "Olympus2400",
            OlympusDataType::_2500 => "Olympus2500",
            OlympusDataType::_2600 => "Olympus2600",
            OlympusDataType::_2700 => "Olympus2700",
            OlympusDataType::_2800 => "Olympus2800",
            OlympusDataType::_2900 => "Olympus2900",
            OlympusDataType::RawInfo => "RawInfo",
            OlympusDataType::MainInfo => "MainInfo",
            OlympusDataType::UnknownInfo => "UnknownInfo",
        }
    }

    /// Check if this tag has a subdirectory
    pub fn has_subdirectory(&self) -> bool {
        matches!(
            self,
            OlympusDataType::MinoltaCameraSettingsOld
                | OlympusDataType::MinoltaCameraSettings
                | OlympusDataType::TextInfo
                | OlympusDataType::PrintIM
        )
    }

    /// Get the group hierarchy for this tag
    pub fn groups(&self) -> (&'static str, &'static str) {
        match self {
            OlympusDataType::MakerNoteVersion => ("MakerNotes", "Camera"),
            OlympusDataType::MinoltaCameraSettingsOld => ("MakerNotes", "Camera"),
            OlympusDataType::MinoltaCameraSettings => ("MakerNotes", "Camera"),
            OlympusDataType::CompressedImageSize => ("MakerNotes", "Camera"),
            OlympusDataType::PreviewImageData => ("MakerNotes", "Camera"),
            OlympusDataType::PreviewImageStart => ("MakerNotes", "Camera"),
            OlympusDataType::PreviewImageLength => ("MakerNotes", "Camera"),
            OlympusDataType::ThumbnailImage => ("Preview", "Camera"),
            OlympusDataType::BodyFirmwareVersion => ("MakerNotes", "Camera"),
            OlympusDataType::SpecialMode => ("MakerNotes", "Camera"),
            OlympusDataType::Quality => ("MakerNotes", "Camera"),
            OlympusDataType::Macro => ("MakerNotes", "Camera"),
            OlympusDataType::BWMode => ("MakerNotes", "Camera"),
            OlympusDataType::DigitalZoom => ("MakerNotes", "Camera"),
            OlympusDataType::FocalPlaneDiagonal => ("MakerNotes", "Camera"),
            OlympusDataType::LensDistortionParams => ("MakerNotes", "Camera"),
            OlympusDataType::CameraType => ("MakerNotes", "Camera"),
            OlympusDataType::TextInfo => ("MakerNotes", "Camera"),
            OlympusDataType::CameraID => ("MakerNotes", "Camera"),
            OlympusDataType::EpsonImageWidth => ("MakerNotes", "Camera"),
            OlympusDataType::EpsonImageHeight => ("MakerNotes", "Camera"),
            OlympusDataType::EpsonSoftware => ("MakerNotes", "Camera"),
            OlympusDataType::PreviewImage => ("Preview", "Camera"),
            OlympusDataType::PreCaptureFrames => ("MakerNotes", "Camera"),
            OlympusDataType::WhiteBoard => ("MakerNotes", "Camera"),
            OlympusDataType::OneTouchWB => ("MakerNotes", "Camera"),
            OlympusDataType::WhiteBalanceBracket => ("MakerNotes", "Camera"),
            OlympusDataType::WhiteBalanceBias => ("MakerNotes", "Camera"),
            OlympusDataType::SensorArea => ("MakerNotes", "Camera"),
            OlympusDataType::BlackLevel => ("MakerNotes", "Camera"),
            OlympusDataType::SceneMode => ("MakerNotes", "Camera"),
            OlympusDataType::SerialNumber => ("MakerNotes", "Camera"),
            OlympusDataType::Firmware => ("MakerNotes", "Camera"),
            OlympusDataType::PrintIM => ("MakerNotes", "Camera"),
            OlympusDataType::DataDump => ("MakerNotes", "Camera"),
            OlympusDataType::DataDump2 => ("MakerNotes", "Camera"),
            OlympusDataType::ZoomedPreviewStart => ("MakerNotes", "Camera"),
            OlympusDataType::ZoomedPreviewLength => ("MakerNotes", "Camera"),
            OlympusDataType::ZoomedPreviewSize => ("MakerNotes", "Camera"),
            OlympusDataType::ShutterSpeedValue => ("MakerNotes", "Camera"),
            OlympusDataType::ISOValue => ("MakerNotes", "Camera"),
            OlympusDataType::ApertureValue => ("MakerNotes", "Camera"),
            OlympusDataType::BrightnessValue => ("MakerNotes", "Camera"),
            OlympusDataType::FlashMode => ("MakerNotes", "Camera"),
            OlympusDataType::FlashDevice => ("MakerNotes", "Camera"),
            OlympusDataType::ExposureCompensation => ("MakerNotes", "Camera"),
            OlympusDataType::SensorTemperature => ("MakerNotes", "Camera"),
            OlympusDataType::LensTemperature => ("MakerNotes", "Camera"),
            OlympusDataType::LightCondition => ("MakerNotes", "Camera"),
            OlympusDataType::FocusRange => ("MakerNotes", "Camera"),
            OlympusDataType::FocusMode => ("MakerNotes", "Camera"),
            OlympusDataType::ManualFocusDistance => ("MakerNotes", "Camera"),
            OlympusDataType::ZoomStepCount => ("MakerNotes", "Camera"),
            OlympusDataType::FocusStepCount => ("MakerNotes", "Camera"),
            OlympusDataType::Sharpness => ("MakerNotes", "Camera"),
            OlympusDataType::FlashChargeLevel => ("MakerNotes", "Camera"),
            OlympusDataType::ColorMatrix => ("MakerNotes", "Camera"),
            OlympusDataType::BlackLevel2 => ("MakerNotes", "Camera"),
            OlympusDataType::ColorTemperatureBG => ("MakerNotes", "Camera"),
            OlympusDataType::ColorTemperatureRG => ("MakerNotes", "Camera"),
            OlympusDataType::WBMode => ("MakerNotes", "Camera"),
            OlympusDataType::RedBalance => ("MakerNotes", "Camera"),
            OlympusDataType::BlueBalance => ("MakerNotes", "Camera"),
            OlympusDataType::ColorMatrixNumber => ("MakerNotes", "Camera"),
            OlympusDataType::SerialNumber2 => ("MakerNotes", "Camera"),
            OlympusDataType::ExternalFlashAE10 => ("MakerNotes", "Camera"),
            OlympusDataType::ExternalFlashAE20 => ("MakerNotes", "Camera"),
            OlympusDataType::InternalFlashAE10 => ("MakerNotes", "Camera"),
            OlympusDataType::InternalFlashAE20 => ("MakerNotes", "Camera"),
            OlympusDataType::ExternalFlashAE1 => ("MakerNotes", "Camera"),
            OlympusDataType::ExternalFlashAE2 => ("MakerNotes", "Camera"),
            OlympusDataType::InternalFlashAE1 => ("MakerNotes", "Camera"),
            OlympusDataType::InternalFlashAE2 => ("MakerNotes", "Camera"),
            OlympusDataType::FlashExposureComp => ("MakerNotes", "Camera"),
            OlympusDataType::InternalFlashTable => ("MakerNotes", "Camera"),
            OlympusDataType::ExternalFlashGValue => ("MakerNotes", "Camera"),
            OlympusDataType::ExternalFlashBounce => ("MakerNotes", "Camera"),
            OlympusDataType::ExternalFlashZoom => ("MakerNotes", "Camera"),
            OlympusDataType::ExternalFlashMode => ("MakerNotes", "Camera"),
            OlympusDataType::Contrast => ("MakerNotes", "Camera"),
            OlympusDataType::SharpnessFactor => ("MakerNotes", "Camera"),
            OlympusDataType::ColorControl => ("MakerNotes", "Camera"),
            OlympusDataType::ValidBits => ("MakerNotes", "Camera"),
            OlympusDataType::CoringFilter => ("MakerNotes", "Camera"),
            OlympusDataType::ImageWidth => ("MakerNotes", "Camera"),
            OlympusDataType::ImageHeight => ("MakerNotes", "Camera"),
            OlympusDataType::SceneDetect => ("MakerNotes", "Camera"),
            OlympusDataType::SceneArea => ("MakerNotes", "Camera"),
            OlympusDataType::SceneDetectData => ("MakerNotes", "Camera"),
            OlympusDataType::CompressionRatio => ("MakerNotes", "Camera"),
            OlympusDataType::PreviewImageValid => ("MakerNotes", "Camera"),
            OlympusDataType::PreviewImageStart2 => ("MakerNotes", "Camera"),
            OlympusDataType::PreviewImageLength2 => ("MakerNotes", "Camera"),
            OlympusDataType::AFResult => ("MakerNotes", "Camera"),
            OlympusDataType::CCDScanMode => ("MakerNotes", "Camera"),
            OlympusDataType::NoiseReduction => ("MakerNotes", "Camera"),
            OlympusDataType::FocusStepInfinity => ("MakerNotes", "Camera"),
            OlympusDataType::FocusStepNear => ("MakerNotes", "Camera"),
            OlympusDataType::LightValueCenter => ("MakerNotes", "Camera"),
            OlympusDataType::LightValuePeriphery => ("MakerNotes", "Camera"),
            OlympusDataType::FieldCount => ("MakerNotes", "Camera"),
            OlympusDataType::Equipment => ("MakerNotes", "Camera"),
            OlympusDataType::CameraSettings => ("MakerNotes", "Camera"),
            OlympusDataType::RawDevelopment => ("MakerNotes", "Camera"),
            OlympusDataType::RawDev2 => ("MakerNotes", "Camera"),
            OlympusDataType::ImageProcessing => ("MakerNotes", "Camera"),
            OlympusDataType::FocusInfo => ("MakerNotes", "Camera"),
            OlympusDataType::_2100 => ("MakerNotes", "Camera"),
            OlympusDataType::_2200 => ("MakerNotes", "Camera"),
            OlympusDataType::_2300 => ("MakerNotes", "Camera"),
            OlympusDataType::_2400 => ("MakerNotes", "Camera"),
            OlympusDataType::_2500 => ("MakerNotes", "Camera"),
            OlympusDataType::_2600 => ("MakerNotes", "Camera"),
            OlympusDataType::_2700 => ("MakerNotes", "Camera"),
            OlympusDataType::_2800 => ("MakerNotes", "Camera"),
            OlympusDataType::_2900 => ("MakerNotes", "Camera"),
            OlympusDataType::RawInfo => ("MakerNotes", "Camera"),
            OlympusDataType::MainInfo => ("MakerNotes", "Camera"),
            OlympusDataType::UnknownInfo => ("MakerNotes", "Camera"),
        }
    }
}
