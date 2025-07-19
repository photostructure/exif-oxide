//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

//! Nikon tag table structure generated from Nikon.pm
//! ExifTool: Nikon.pm %Nikon::Main
//! Generated at: Sat Jul 19 20:43:17 2025 GMT

/// Nikon data types from %Nikon::Main table
/// Total tags: 111 (conditional: 12, with subdirectories: 22)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NikonDataType {
    /// 0x0001: MakerNoteVersion
    MakerNoteVersion,
    /// 0x0002: ISO
    ISO,
    /// 0x0003: ColorMode
    ColorMode,
    /// 0x0004: Quality
    Quality,
    /// 0x0005: WhiteBalance
    WhiteBalance,
    /// 0x0006: Sharpness
    Sharpness,
    /// 0x0007: FocusMode
    FocusMode,
    /// 0x0008: FlashSetting
    FlashSetting,
    /// 0x0009: FlashType
    FlashType,
    /// 0x000b: WhiteBalanceFineTune
    WhiteBalanceFineTune,
    /// 0x000c: WB_RBLevels
    WBRBLevels,
    /// 0x000d: ProgramShift
    ProgramShift,
    /// 0x000e: ExposureDifference
    ExposureDifference,
    /// 0x000f: ISOSelection
    ISOSelection,
    /// 0x0010: DataDump
    DataDump,
    /// 0x0011: PreviewIFD
    /// ExifTool: SubDirectory -> PreviewIFD
    PreviewIFD,
    /// 0x0012: FlashExposureComp
    /// Flash Exposure Compensation
    FlashExposureComp,
    /// 0x0013: ISOSetting
    ISOSetting,
    /// 0x0014: ColorBalanceA
    ColorBalanceA,

    /// 0x0016: ImageBoundary
    ImageBoundary,
    /// 0x0017: ExternalFlashExposureComp
    ExternalFlashExposureComp,
    /// 0x0018: FlashExposureBracketValue
    FlashExposureBracketValue,
    /// 0x0019: ExposureBracketValue
    ExposureBracketValue,
    /// 0x001a: ImageProcessing
    ImageProcessing,
    /// 0x001b: CropHiSpeed
    CropHiSpeed,
    /// 0x001c: ExposureTuning
    ExposureTuning,
    /// 0x001d: SerialNumber
    SerialNumber,
    /// 0x001e: ColorSpace
    ColorSpace,
    /// 0x001f: VRInfo
    /// ExifTool: SubDirectory -> VRInfo
    VRInfo,
    /// 0x0020: ImageAuthentication
    ImageAuthentication,
    /// 0x0021: FaceDetect
    /// ExifTool: SubDirectory -> FaceDetect
    FaceDetect,
    /// 0x0022: ActiveD-Lighting
    ActiveDLighting,
    /// 0x0023: PictureControlData
    PictureControlData,

    /// 0x0024: WorldTime
    /// ExifTool: SubDirectory -> WorldTime
    WorldTime,
    /// 0x0025: ISOInfo
    /// ExifTool: SubDirectory -> ISOInfo
    ISOInfo,
    /// 0x002a: VignetteControl
    VignetteControl,
    /// 0x002b: DistortInfo
    /// ExifTool: SubDirectory -> DistortInfo
    DistortInfo,
    /// 0x002c: UnknownInfo
    /// ExifTool: SubDirectory -> UnknownInfo
    UnknownInfo,
    /// 0x0032: UnknownInfo2
    /// ExifTool: SubDirectory -> UnknownInfo2
    UnknownInfo2,
    /// 0x0034: ShutterMode
    ShutterMode,
    /// 0x0035: HDRInfo
    HDRInfo,

    /// 0x0037: MechanicalShutterCount
    MechanicalShutterCount,
    /// 0x0039: LocationInfo
    /// ExifTool: SubDirectory -> LocationInfo
    LocationInfo,
    /// 0x003d: BlackLevel
    BlackLevel,
    /// 0x003e: ImageSizeRAW
    ImageSizeRAW,
    /// 0x003f: WhiteBalanceFineTune
    WhiteBalanceFineTune2,
    /// 0x0044: JPGCompression
    JPGCompression,
    /// 0x0045: CropArea
    CropArea,
    /// 0x004e: NikonSettings
    /// ExifTool: SubDirectory -> NikonSettings::Main
    Settings,
    /// 0x004f: ColorTemperatureAuto
    ColorTemperatureAuto,
    /// 0x0051: MakerNotes0x51
    /// ExifTool: SubDirectory -> MakerNotes0x51
    MakerNotes0x51,
    /// 0x0056: MakerNotes0x56
    /// ExifTool: SubDirectory -> MakerNotes0x56
    MakerNotes0x56,
    /// 0x0080: ImageAdjustment
    ImageAdjustment,
    /// 0x0081: ToneComp
    ToneComp,
    /// 0x0082: AuxiliaryLens
    AuxiliaryLens,
    /// 0x0083: LensType
    LensType,
    /// 0x0084: Lens
    Lens,
    /// 0x0085: ManualFocusDistance
    ManualFocusDistance,
    /// 0x0086: DigitalZoom
    DigitalZoom,
    /// 0x0087: FlashMode
    FlashMode,
    /// 0x0088: AFInfo
    AFInfo,

    /// 0x0089: ShootingMode
    ShootingMode,
    /// 0x008b: LensFStops
    LensFStops,
    /// 0x008c: ContrastCurve
    ContrastCurve,
    /// 0x008d: ColorHue
    ColorHue,
    /// 0x008f: SceneMode
    SceneMode,
    /// 0x0090: LightSource
    LightSource,
    /// 0x0091: ShotInfoD40
    ShotInfoD40,

    /// 0x0092: HueAdjustment
    HueAdjustment,
    /// 0x0093: NEFCompression
    NEFCompression,
    /// 0x0094: SaturationAdj
    SaturationAdj,
    /// 0x0095: NoiseReduction
    NoiseReduction,
    /// 0x0096: NEFLinearizationTable
    NEFLinearizationTable,
    /// 0x0097: ColorBalance0100
    ColorBalance0100,

    /// 0x0098: LensData0100
    LensData0100,

    /// 0x0099: RawImageCenter
    RawImageCenter,
    /// 0x009a: SensorPixelSize
    SensorPixelSize,
    /// 0x009c: SceneAssist
    SceneAssist,
    /// 0x009d: DateStampMode
    DateStampMode,
    /// 0x009e: RetouchHistory
    RetouchHistory,
    /// 0x00a0: SerialNumber
    SerialNumber2,
    /// 0x00a2: ImageDataSize
    ImageDataSize,
    /// 0x00a5: ImageCount
    ImageCount,
    /// 0x00a6: DeletedImageCount
    DeletedImageCount,
    /// 0x00a7: ShutterCount
    ShutterCount,
    /// 0x00a8: FlashInfo0100
    FlashInfo0100,

    /// 0x00a9: ImageOptimization
    ImageOptimization,
    /// 0x00aa: Saturation
    Saturation,
    /// 0x00ab: VariProgram
    VariProgram,
    /// 0x00ac: ImageStabilization
    ImageStabilization,
    /// 0x00ad: AFResponse
    AFResponse,
    /// 0x00b0: MultiExposure
    MultiExposure,

    /// 0x00b1: HighISONoiseReduction
    HighISONoiseReduction,
    /// 0x00b3: ToningEffect
    ToningEffect,
    /// 0x00b6: PowerUpTime
    PowerUpTime,
    /// 0x00b7: AFInfo2
    AFInfo2,

    /// 0x00b8: FileInfo
    FileInfo,

    /// 0x00b9: AFTune
    /// ExifTool: SubDirectory -> AFTune
    AFTune,
    /// 0x00bb: RetouchInfo
    /// ExifTool: SubDirectory -> RetouchInfo
    RetouchInfo,
    /// 0x00bd: PictureControlData
    /// ExifTool: SubDirectory -> PictureControl
    PictureControlData2,
    /// 0x00bf: SilentPhotography
    SilentPhotography,
    /// 0x00c3: BarometerInfo
    /// ExifTool: SubDirectory -> BarometerInfo
    BarometerInfo,
    /// 0x0e00: PrintIM
    /// Print Image Matching
    /// ExifTool: SubDirectory -> PrintIM::Main
    PrintIM,
    /// 0x0e01: NikonCaptureData
    /// ExifTool: SubDirectory -> NikonCapture::Main
    CaptureData,
    /// 0x0e09: NikonCaptureVersion
    CaptureVersion,
    /// 0x0e0e: NikonCaptureOffsets
    /// ExifTool: SubDirectory -> CaptureOffsets
    CaptureOffsets,
    /// 0x0e10: NikonScanIFD
    /// ExifTool: SubDirectory -> Scan
    ScanIFD,
    /// 0x0e13: NikonCaptureEditVersions
    CaptureEditVersions,

    /// 0x0e1d: NikonICCProfile
    /// ExifTool: SubDirectory -> ICC_Profile::Main
    ICCProfile,
    /// 0x0e1e: NikonCaptureOutput
    /// ExifTool: SubDirectory -> CaptureOutput
    CaptureOutput,
    /// 0x0e22: NEFBitDepth
    NEFBitDepth,
}

impl NikonDataType {
    /// Get tag ID for this data type
    pub fn tag_id(&self) -> u16 {
        match self {
            NikonDataType::MakerNoteVersion => 0x0001,
            NikonDataType::ISO => 0x0002,
            NikonDataType::ColorMode => 0x0003,
            NikonDataType::Quality => 0x0004,
            NikonDataType::WhiteBalance => 0x0005,
            NikonDataType::Sharpness => 0x0006,
            NikonDataType::FocusMode => 0x0007,
            NikonDataType::FlashSetting => 0x0008,
            NikonDataType::FlashType => 0x0009,
            NikonDataType::WhiteBalanceFineTune => 0x000b,
            NikonDataType::WBRBLevels => 0x000c,
            NikonDataType::ProgramShift => 0x000d,
            NikonDataType::ExposureDifference => 0x000e,
            NikonDataType::ISOSelection => 0x000f,
            NikonDataType::DataDump => 0x0010,
            NikonDataType::PreviewIFD => 0x0011,
            NikonDataType::FlashExposureComp => 0x0012,
            NikonDataType::ISOSetting => 0x0013,
            NikonDataType::ColorBalanceA => 0x0014,
            NikonDataType::ImageBoundary => 0x0016,
            NikonDataType::ExternalFlashExposureComp => 0x0017,
            NikonDataType::FlashExposureBracketValue => 0x0018,
            NikonDataType::ExposureBracketValue => 0x0019,
            NikonDataType::ImageProcessing => 0x001a,
            NikonDataType::CropHiSpeed => 0x001b,
            NikonDataType::ExposureTuning => 0x001c,
            NikonDataType::SerialNumber => 0x001d,
            NikonDataType::ColorSpace => 0x001e,
            NikonDataType::VRInfo => 0x001f,
            NikonDataType::ImageAuthentication => 0x0020,
            NikonDataType::FaceDetect => 0x0021,
            NikonDataType::ActiveDLighting => 0x0022,
            NikonDataType::PictureControlData => 0x0023,
            NikonDataType::WorldTime => 0x0024,
            NikonDataType::ISOInfo => 0x0025,
            NikonDataType::VignetteControl => 0x002a,
            NikonDataType::DistortInfo => 0x002b,
            NikonDataType::UnknownInfo => 0x002c,
            NikonDataType::UnknownInfo2 => 0x0032,
            NikonDataType::ShutterMode => 0x0034,
            NikonDataType::HDRInfo => 0x0035,
            NikonDataType::MechanicalShutterCount => 0x0037,
            NikonDataType::LocationInfo => 0x0039,
            NikonDataType::BlackLevel => 0x003d,
            NikonDataType::ImageSizeRAW => 0x003e,
            NikonDataType::WhiteBalanceFineTune2 => 0x003f,
            NikonDataType::JPGCompression => 0x0044,
            NikonDataType::CropArea => 0x0045,
            NikonDataType::Settings => 0x004e,
            NikonDataType::ColorTemperatureAuto => 0x004f,
            NikonDataType::MakerNotes0x51 => 0x0051,
            NikonDataType::MakerNotes0x56 => 0x0056,
            NikonDataType::ImageAdjustment => 0x0080,
            NikonDataType::ToneComp => 0x0081,
            NikonDataType::AuxiliaryLens => 0x0082,
            NikonDataType::LensType => 0x0083,
            NikonDataType::Lens => 0x0084,
            NikonDataType::ManualFocusDistance => 0x0085,
            NikonDataType::DigitalZoom => 0x0086,
            NikonDataType::FlashMode => 0x0087,
            NikonDataType::AFInfo => 0x0088,
            NikonDataType::ShootingMode => 0x0089,
            NikonDataType::LensFStops => 0x008b,
            NikonDataType::ContrastCurve => 0x008c,
            NikonDataType::ColorHue => 0x008d,
            NikonDataType::SceneMode => 0x008f,
            NikonDataType::LightSource => 0x0090,
            NikonDataType::ShotInfoD40 => 0x0091,
            NikonDataType::HueAdjustment => 0x0092,
            NikonDataType::NEFCompression => 0x0093,
            NikonDataType::SaturationAdj => 0x0094,
            NikonDataType::NoiseReduction => 0x0095,
            NikonDataType::NEFLinearizationTable => 0x0096,
            NikonDataType::ColorBalance0100 => 0x0097,
            NikonDataType::LensData0100 => 0x0098,
            NikonDataType::RawImageCenter => 0x0099,
            NikonDataType::SensorPixelSize => 0x009a,
            NikonDataType::SceneAssist => 0x009c,
            NikonDataType::DateStampMode => 0x009d,
            NikonDataType::RetouchHistory => 0x009e,
            NikonDataType::SerialNumber2 => 0x00a0,
            NikonDataType::ImageDataSize => 0x00a2,
            NikonDataType::ImageCount => 0x00a5,
            NikonDataType::DeletedImageCount => 0x00a6,
            NikonDataType::ShutterCount => 0x00a7,
            NikonDataType::FlashInfo0100 => 0x00a8,
            NikonDataType::ImageOptimization => 0x00a9,
            NikonDataType::Saturation => 0x00aa,
            NikonDataType::VariProgram => 0x00ab,
            NikonDataType::ImageStabilization => 0x00ac,
            NikonDataType::AFResponse => 0x00ad,
            NikonDataType::MultiExposure => 0x00b0,
            NikonDataType::HighISONoiseReduction => 0x00b1,
            NikonDataType::ToningEffect => 0x00b3,
            NikonDataType::PowerUpTime => 0x00b6,
            NikonDataType::AFInfo2 => 0x00b7,
            NikonDataType::FileInfo => 0x00b8,
            NikonDataType::AFTune => 0x00b9,
            NikonDataType::RetouchInfo => 0x00bb,
            NikonDataType::PictureControlData2 => 0x00bd,
            NikonDataType::SilentPhotography => 0x00bf,
            NikonDataType::BarometerInfo => 0x00c3,
            NikonDataType::PrintIM => 0x0e00,
            NikonDataType::CaptureData => 0x0e01,
            NikonDataType::CaptureVersion => 0x0e09,
            NikonDataType::CaptureOffsets => 0x0e0e,
            NikonDataType::ScanIFD => 0x0e10,
            NikonDataType::CaptureEditVersions => 0x0e13,
            NikonDataType::ICCProfile => 0x0e1d,
            NikonDataType::CaptureOutput => 0x0e1e,
            NikonDataType::NEFBitDepth => 0x0e22,
        }
    }

    /// Get data type from tag ID
    pub fn from_tag_id(tag_id: u16) -> Option<NikonDataType> {
        match tag_id {
            0x0001 => Some(NikonDataType::MakerNoteVersion),
            0x0002 => Some(NikonDataType::ISO),
            0x0003 => Some(NikonDataType::ColorMode),
            0x0004 => Some(NikonDataType::Quality),
            0x0005 => Some(NikonDataType::WhiteBalance),
            0x0006 => Some(NikonDataType::Sharpness),
            0x0007 => Some(NikonDataType::FocusMode),
            0x0008 => Some(NikonDataType::FlashSetting),
            0x0009 => Some(NikonDataType::FlashType),
            0x000b => Some(NikonDataType::WhiteBalanceFineTune),
            0x000c => Some(NikonDataType::WBRBLevels),
            0x000d => Some(NikonDataType::ProgramShift),
            0x000e => Some(NikonDataType::ExposureDifference),
            0x000f => Some(NikonDataType::ISOSelection),
            0x0010 => Some(NikonDataType::DataDump),
            0x0011 => Some(NikonDataType::PreviewIFD),
            0x0012 => Some(NikonDataType::FlashExposureComp),
            0x0013 => Some(NikonDataType::ISOSetting),
            0x0014 => Some(NikonDataType::ColorBalanceA),
            0x0016 => Some(NikonDataType::ImageBoundary),
            0x0017 => Some(NikonDataType::ExternalFlashExposureComp),
            0x0018 => Some(NikonDataType::FlashExposureBracketValue),
            0x0019 => Some(NikonDataType::ExposureBracketValue),
            0x001a => Some(NikonDataType::ImageProcessing),
            0x001b => Some(NikonDataType::CropHiSpeed),
            0x001c => Some(NikonDataType::ExposureTuning),
            0x001d => Some(NikonDataType::SerialNumber),
            0x001e => Some(NikonDataType::ColorSpace),
            0x001f => Some(NikonDataType::VRInfo),
            0x0020 => Some(NikonDataType::ImageAuthentication),
            0x0021 => Some(NikonDataType::FaceDetect),
            0x0022 => Some(NikonDataType::ActiveDLighting),
            0x0023 => Some(NikonDataType::PictureControlData),
            0x0024 => Some(NikonDataType::WorldTime),
            0x0025 => Some(NikonDataType::ISOInfo),
            0x002a => Some(NikonDataType::VignetteControl),
            0x002b => Some(NikonDataType::DistortInfo),
            0x002c => Some(NikonDataType::UnknownInfo),
            0x0032 => Some(NikonDataType::UnknownInfo2),
            0x0034 => Some(NikonDataType::ShutterMode),
            0x0035 => Some(NikonDataType::HDRInfo),
            0x0037 => Some(NikonDataType::MechanicalShutterCount),
            0x0039 => Some(NikonDataType::LocationInfo),
            0x003d => Some(NikonDataType::BlackLevel),
            0x003e => Some(NikonDataType::ImageSizeRAW),
            0x003f => Some(NikonDataType::WhiteBalanceFineTune2),
            0x0044 => Some(NikonDataType::JPGCompression),
            0x0045 => Some(NikonDataType::CropArea),
            0x004e => Some(NikonDataType::Settings),
            0x004f => Some(NikonDataType::ColorTemperatureAuto),
            0x0051 => Some(NikonDataType::MakerNotes0x51),
            0x0056 => Some(NikonDataType::MakerNotes0x56),
            0x0080 => Some(NikonDataType::ImageAdjustment),
            0x0081 => Some(NikonDataType::ToneComp),
            0x0082 => Some(NikonDataType::AuxiliaryLens),
            0x0083 => Some(NikonDataType::LensType),
            0x0084 => Some(NikonDataType::Lens),
            0x0085 => Some(NikonDataType::ManualFocusDistance),
            0x0086 => Some(NikonDataType::DigitalZoom),
            0x0087 => Some(NikonDataType::FlashMode),
            0x0088 => Some(NikonDataType::AFInfo),
            0x0089 => Some(NikonDataType::ShootingMode),
            0x008b => Some(NikonDataType::LensFStops),
            0x008c => Some(NikonDataType::ContrastCurve),
            0x008d => Some(NikonDataType::ColorHue),
            0x008f => Some(NikonDataType::SceneMode),
            0x0090 => Some(NikonDataType::LightSource),
            0x0091 => Some(NikonDataType::ShotInfoD40),
            0x0092 => Some(NikonDataType::HueAdjustment),
            0x0093 => Some(NikonDataType::NEFCompression),
            0x0094 => Some(NikonDataType::SaturationAdj),
            0x0095 => Some(NikonDataType::NoiseReduction),
            0x0096 => Some(NikonDataType::NEFLinearizationTable),
            0x0097 => Some(NikonDataType::ColorBalance0100),
            0x0098 => Some(NikonDataType::LensData0100),
            0x0099 => Some(NikonDataType::RawImageCenter),
            0x009a => Some(NikonDataType::SensorPixelSize),
            0x009c => Some(NikonDataType::SceneAssist),
            0x009d => Some(NikonDataType::DateStampMode),
            0x009e => Some(NikonDataType::RetouchHistory),
            0x00a0 => Some(NikonDataType::SerialNumber2),
            0x00a2 => Some(NikonDataType::ImageDataSize),
            0x00a5 => Some(NikonDataType::ImageCount),
            0x00a6 => Some(NikonDataType::DeletedImageCount),
            0x00a7 => Some(NikonDataType::ShutterCount),
            0x00a8 => Some(NikonDataType::FlashInfo0100),
            0x00a9 => Some(NikonDataType::ImageOptimization),
            0x00aa => Some(NikonDataType::Saturation),
            0x00ab => Some(NikonDataType::VariProgram),
            0x00ac => Some(NikonDataType::ImageStabilization),
            0x00ad => Some(NikonDataType::AFResponse),
            0x00b0 => Some(NikonDataType::MultiExposure),
            0x00b1 => Some(NikonDataType::HighISONoiseReduction),
            0x00b3 => Some(NikonDataType::ToningEffect),
            0x00b6 => Some(NikonDataType::PowerUpTime),
            0x00b7 => Some(NikonDataType::AFInfo2),
            0x00b8 => Some(NikonDataType::FileInfo),
            0x00b9 => Some(NikonDataType::AFTune),
            0x00bb => Some(NikonDataType::RetouchInfo),
            0x00bd => Some(NikonDataType::PictureControlData2),
            0x00bf => Some(NikonDataType::SilentPhotography),
            0x00c3 => Some(NikonDataType::BarometerInfo),
            0x0e00 => Some(NikonDataType::PrintIM),
            0x0e01 => Some(NikonDataType::CaptureData),
            0x0e09 => Some(NikonDataType::CaptureVersion),
            0x0e0e => Some(NikonDataType::CaptureOffsets),
            0x0e10 => Some(NikonDataType::ScanIFD),
            0x0e13 => Some(NikonDataType::CaptureEditVersions),
            0x0e1d => Some(NikonDataType::ICCProfile),
            0x0e1e => Some(NikonDataType::CaptureOutput),
            0x0e22 => Some(NikonDataType::NEFBitDepth),
            _ => None,
        }
    }

    /// Get the ExifTool tag name
    pub fn name(&self) -> &'static str {
        match self {
            NikonDataType::MakerNoteVersion => "MakerNoteVersion",
            NikonDataType::ISO => "ISO",
            NikonDataType::ColorMode => "ColorMode",
            NikonDataType::Quality => "Quality",
            NikonDataType::WhiteBalance => "WhiteBalance",
            NikonDataType::Sharpness => "Sharpness",
            NikonDataType::FocusMode => "FocusMode",
            NikonDataType::FlashSetting => "FlashSetting",
            NikonDataType::FlashType => "FlashType",
            NikonDataType::WhiteBalanceFineTune => "WhiteBalanceFineTune",
            NikonDataType::WBRBLevels => "WB_RBLevels",
            NikonDataType::ProgramShift => "ProgramShift",
            NikonDataType::ExposureDifference => "ExposureDifference",
            NikonDataType::ISOSelection => "ISOSelection",
            NikonDataType::DataDump => "DataDump",
            NikonDataType::PreviewIFD => "PreviewIFD",
            NikonDataType::FlashExposureComp => "FlashExposureComp",
            NikonDataType::ISOSetting => "ISOSetting",
            NikonDataType::ColorBalanceA => "ColorBalanceA",
            NikonDataType::ImageBoundary => "ImageBoundary",
            NikonDataType::ExternalFlashExposureComp => "ExternalFlashExposureComp",
            NikonDataType::FlashExposureBracketValue => "FlashExposureBracketValue",
            NikonDataType::ExposureBracketValue => "ExposureBracketValue",
            NikonDataType::ImageProcessing => "ImageProcessing",
            NikonDataType::CropHiSpeed => "CropHiSpeed",
            NikonDataType::ExposureTuning => "ExposureTuning",
            NikonDataType::SerialNumber => "SerialNumber",
            NikonDataType::ColorSpace => "ColorSpace",
            NikonDataType::VRInfo => "VRInfo",
            NikonDataType::ImageAuthentication => "ImageAuthentication",
            NikonDataType::FaceDetect => "FaceDetect",
            NikonDataType::ActiveDLighting => "ActiveD-Lighting",
            NikonDataType::PictureControlData => "PictureControlData",
            NikonDataType::WorldTime => "WorldTime",
            NikonDataType::ISOInfo => "ISOInfo",
            NikonDataType::VignetteControl => "VignetteControl",
            NikonDataType::DistortInfo => "DistortInfo",
            NikonDataType::UnknownInfo => "UnknownInfo",
            NikonDataType::UnknownInfo2 => "UnknownInfo2",
            NikonDataType::ShutterMode => "ShutterMode",
            NikonDataType::HDRInfo => "HDRInfo",
            NikonDataType::MechanicalShutterCount => "MechanicalShutterCount",
            NikonDataType::LocationInfo => "LocationInfo",
            NikonDataType::BlackLevel => "BlackLevel",
            NikonDataType::ImageSizeRAW => "ImageSizeRAW",
            NikonDataType::WhiteBalanceFineTune2 => "WhiteBalanceFineTune",
            NikonDataType::JPGCompression => "JPGCompression",
            NikonDataType::CropArea => "CropArea",
            NikonDataType::Settings => "NikonSettings",
            NikonDataType::ColorTemperatureAuto => "ColorTemperatureAuto",
            NikonDataType::MakerNotes0x51 => "MakerNotes0x51",
            NikonDataType::MakerNotes0x56 => "MakerNotes0x56",
            NikonDataType::ImageAdjustment => "ImageAdjustment",
            NikonDataType::ToneComp => "ToneComp",
            NikonDataType::AuxiliaryLens => "AuxiliaryLens",
            NikonDataType::LensType => "LensType",
            NikonDataType::Lens => "Lens",
            NikonDataType::ManualFocusDistance => "ManualFocusDistance",
            NikonDataType::DigitalZoom => "DigitalZoom",
            NikonDataType::FlashMode => "FlashMode",
            NikonDataType::AFInfo => "AFInfo",
            NikonDataType::ShootingMode => "ShootingMode",
            NikonDataType::LensFStops => "LensFStops",
            NikonDataType::ContrastCurve => "ContrastCurve",
            NikonDataType::ColorHue => "ColorHue",
            NikonDataType::SceneMode => "SceneMode",
            NikonDataType::LightSource => "LightSource",
            NikonDataType::ShotInfoD40 => "ShotInfoD40",
            NikonDataType::HueAdjustment => "HueAdjustment",
            NikonDataType::NEFCompression => "NEFCompression",
            NikonDataType::SaturationAdj => "SaturationAdj",
            NikonDataType::NoiseReduction => "NoiseReduction",
            NikonDataType::NEFLinearizationTable => "NEFLinearizationTable",
            NikonDataType::ColorBalance0100 => "ColorBalance0100",
            NikonDataType::LensData0100 => "LensData0100",
            NikonDataType::RawImageCenter => "RawImageCenter",
            NikonDataType::SensorPixelSize => "SensorPixelSize",
            NikonDataType::SceneAssist => "SceneAssist",
            NikonDataType::DateStampMode => "DateStampMode",
            NikonDataType::RetouchHistory => "RetouchHistory",
            NikonDataType::SerialNumber2 => "SerialNumber",
            NikonDataType::ImageDataSize => "ImageDataSize",
            NikonDataType::ImageCount => "ImageCount",
            NikonDataType::DeletedImageCount => "DeletedImageCount",
            NikonDataType::ShutterCount => "ShutterCount",
            NikonDataType::FlashInfo0100 => "FlashInfo0100",
            NikonDataType::ImageOptimization => "ImageOptimization",
            NikonDataType::Saturation => "Saturation",
            NikonDataType::VariProgram => "VariProgram",
            NikonDataType::ImageStabilization => "ImageStabilization",
            NikonDataType::AFResponse => "AFResponse",
            NikonDataType::MultiExposure => "MultiExposure",
            NikonDataType::HighISONoiseReduction => "HighISONoiseReduction",
            NikonDataType::ToningEffect => "ToningEffect",
            NikonDataType::PowerUpTime => "PowerUpTime",
            NikonDataType::AFInfo2 => "AFInfo2",
            NikonDataType::FileInfo => "FileInfo",
            NikonDataType::AFTune => "AFTune",
            NikonDataType::RetouchInfo => "RetouchInfo",
            NikonDataType::PictureControlData2 => "PictureControlData",
            NikonDataType::SilentPhotography => "SilentPhotography",
            NikonDataType::BarometerInfo => "BarometerInfo",
            NikonDataType::PrintIM => "PrintIM",
            NikonDataType::CaptureData => "NikonCaptureData",
            NikonDataType::CaptureVersion => "NikonCaptureVersion",
            NikonDataType::CaptureOffsets => "NikonCaptureOffsets",
            NikonDataType::ScanIFD => "NikonScanIFD",
            NikonDataType::CaptureEditVersions => "NikonCaptureEditVersions",
            NikonDataType::ICCProfile => "NikonICCProfile",
            NikonDataType::CaptureOutput => "NikonCaptureOutput",
            NikonDataType::NEFBitDepth => "NEFBitDepth",
        }
    }

    /// Check if this tag has a subdirectory
    pub fn has_subdirectory(&self) -> bool {
        matches!(
            self,
            NikonDataType::PreviewIFD
                | NikonDataType::VRInfo
                | NikonDataType::FaceDetect
                | NikonDataType::WorldTime
                | NikonDataType::ISOInfo
                | NikonDataType::DistortInfo
                | NikonDataType::UnknownInfo
                | NikonDataType::UnknownInfo2
                | NikonDataType::LocationInfo
                | NikonDataType::Settings
                | NikonDataType::MakerNotes0x51
                | NikonDataType::MakerNotes0x56
                | NikonDataType::AFTune
                | NikonDataType::RetouchInfo
                | NikonDataType::PictureControlData2
                | NikonDataType::BarometerInfo
                | NikonDataType::PrintIM
                | NikonDataType::CaptureData
                | NikonDataType::CaptureOffsets
                | NikonDataType::ScanIFD
                | NikonDataType::ICCProfile
                | NikonDataType::CaptureOutput
        )
    }

    /// Get the group hierarchy for this tag
    pub fn groups(&self) -> (&'static str, &'static str) {
        match self {
            NikonDataType::MakerNoteVersion => ("MakerNotes", "Camera"),
            NikonDataType::ISO => ("Image", "Camera"),
            NikonDataType::ColorMode => ("MakerNotes", "Camera"),
            NikonDataType::Quality => ("MakerNotes", "Camera"),
            NikonDataType::WhiteBalance => ("MakerNotes", "Camera"),
            NikonDataType::Sharpness => ("MakerNotes", "Camera"),
            NikonDataType::FocusMode => ("MakerNotes", "Camera"),
            NikonDataType::FlashSetting => ("MakerNotes", "Camera"),
            NikonDataType::FlashType => ("MakerNotes", "Camera"),
            NikonDataType::WhiteBalanceFineTune => ("MakerNotes", "Camera"),
            NikonDataType::WBRBLevels => ("MakerNotes", "Camera"),
            NikonDataType::ProgramShift => ("MakerNotes", "Camera"),
            NikonDataType::ExposureDifference => ("MakerNotes", "Camera"),
            NikonDataType::ISOSelection => ("MakerNotes", "Camera"),
            NikonDataType::DataDump => ("MakerNotes", "Camera"),
            NikonDataType::PreviewIFD => ("PreviewIFD", "Camera"),
            NikonDataType::FlashExposureComp => ("MakerNotes", "Camera"),
            NikonDataType::ISOSetting => ("MakerNotes", "Camera"),
            NikonDataType::ColorBalanceA => ("MakerNotes", "Camera"),
            NikonDataType::ImageBoundary => ("MakerNotes", "Camera"),
            NikonDataType::ExternalFlashExposureComp => ("MakerNotes", "Camera"),
            NikonDataType::FlashExposureBracketValue => ("MakerNotes", "Camera"),
            NikonDataType::ExposureBracketValue => ("MakerNotes", "Camera"),
            NikonDataType::ImageProcessing => ("MakerNotes", "Camera"),
            NikonDataType::CropHiSpeed => ("MakerNotes", "Camera"),
            NikonDataType::ExposureTuning => ("MakerNotes", "Camera"),
            NikonDataType::SerialNumber => ("MakerNotes", "Camera"),
            NikonDataType::ColorSpace => ("MakerNotes", "Camera"),
            NikonDataType::VRInfo => ("MakerNotes", "Camera"),
            NikonDataType::ImageAuthentication => ("MakerNotes", "Camera"),
            NikonDataType::FaceDetect => ("MakerNotes", "Camera"),
            NikonDataType::ActiveDLighting => ("MakerNotes", "Camera"),
            NikonDataType::PictureControlData => ("MakerNotes", "Camera"),
            NikonDataType::WorldTime => ("MakerNotes", "Camera"),
            NikonDataType::ISOInfo => ("MakerNotes", "Camera"),
            NikonDataType::VignetteControl => ("MakerNotes", "Camera"),
            NikonDataType::DistortInfo => ("MakerNotes", "Camera"),
            NikonDataType::UnknownInfo => ("MakerNotes", "Camera"),
            NikonDataType::UnknownInfo2 => ("MakerNotes", "Camera"),
            NikonDataType::ShutterMode => ("MakerNotes", "Camera"),
            NikonDataType::HDRInfo => ("MakerNotes", "Camera"),
            NikonDataType::MechanicalShutterCount => ("MakerNotes", "Camera"),
            NikonDataType::LocationInfo => ("MakerNotes", "Camera"),
            NikonDataType::BlackLevel => ("MakerNotes", "Camera"),
            NikonDataType::ImageSizeRAW => ("MakerNotes", "Camera"),
            NikonDataType::WhiteBalanceFineTune2 => ("MakerNotes", "Camera"),
            NikonDataType::JPGCompression => ("MakerNotes", "Camera"),
            NikonDataType::CropArea => ("MakerNotes", "Camera"),
            NikonDataType::Settings => ("MakerNotes", "Camera"),
            NikonDataType::ColorTemperatureAuto => ("MakerNotes", "Camera"),
            NikonDataType::MakerNotes0x51 => ("MakerNotes", "Camera"),
            NikonDataType::MakerNotes0x56 => ("MakerNotes", "Camera"),
            NikonDataType::ImageAdjustment => ("MakerNotes", "Camera"),
            NikonDataType::ToneComp => ("MakerNotes", "Camera"),
            NikonDataType::AuxiliaryLens => ("MakerNotes", "Camera"),
            NikonDataType::LensType => ("MakerNotes", "Camera"),
            NikonDataType::Lens => ("MakerNotes", "Camera"),
            NikonDataType::ManualFocusDistance => ("MakerNotes", "Camera"),
            NikonDataType::DigitalZoom => ("MakerNotes", "Camera"),
            NikonDataType::FlashMode => ("MakerNotes", "Camera"),
            NikonDataType::AFInfo => ("MakerNotes", "Camera"),
            NikonDataType::ShootingMode => ("MakerNotes", "Camera"),
            NikonDataType::LensFStops => ("MakerNotes", "Camera"),
            NikonDataType::ContrastCurve => ("MakerNotes", "Camera"),
            NikonDataType::ColorHue => ("MakerNotes", "Camera"),
            NikonDataType::SceneMode => ("MakerNotes", "Camera"),
            NikonDataType::LightSource => ("MakerNotes", "Camera"),
            NikonDataType::ShotInfoD40 => ("MakerNotes", "Camera"),
            NikonDataType::HueAdjustment => ("MakerNotes", "Camera"),
            NikonDataType::NEFCompression => ("MakerNotes", "Camera"),
            NikonDataType::SaturationAdj => ("MakerNotes", "Camera"),
            NikonDataType::NoiseReduction => ("MakerNotes", "Camera"),
            NikonDataType::NEFLinearizationTable => ("MakerNotes", "Camera"),
            NikonDataType::ColorBalance0100 => ("MakerNotes", "Camera"),
            NikonDataType::LensData0100 => ("MakerNotes", "Camera"),
            NikonDataType::RawImageCenter => ("MakerNotes", "Camera"),
            NikonDataType::SensorPixelSize => ("MakerNotes", "Camera"),
            NikonDataType::SceneAssist => ("MakerNotes", "Camera"),
            NikonDataType::DateStampMode => ("MakerNotes", "Camera"),
            NikonDataType::RetouchHistory => ("MakerNotes", "Camera"),
            NikonDataType::SerialNumber2 => ("MakerNotes", "Camera"),
            NikonDataType::ImageDataSize => ("MakerNotes", "Camera"),
            NikonDataType::ImageCount => ("MakerNotes", "Camera"),
            NikonDataType::DeletedImageCount => ("MakerNotes", "Camera"),
            NikonDataType::ShutterCount => ("MakerNotes", "Camera"),
            NikonDataType::FlashInfo0100 => ("MakerNotes", "Camera"),
            NikonDataType::ImageOptimization => ("MakerNotes", "Camera"),
            NikonDataType::Saturation => ("MakerNotes", "Camera"),
            NikonDataType::VariProgram => ("MakerNotes", "Camera"),
            NikonDataType::ImageStabilization => ("MakerNotes", "Camera"),
            NikonDataType::AFResponse => ("MakerNotes", "Camera"),
            NikonDataType::MultiExposure => ("MakerNotes", "Camera"),
            NikonDataType::HighISONoiseReduction => ("MakerNotes", "Camera"),
            NikonDataType::ToningEffect => ("MakerNotes", "Camera"),
            NikonDataType::PowerUpTime => ("Time", "Camera"),
            NikonDataType::AFInfo2 => ("MakerNotes", "Camera"),
            NikonDataType::FileInfo => ("MakerNotes", "Camera"),
            NikonDataType::AFTune => ("MakerNotes", "Camera"),
            NikonDataType::RetouchInfo => ("MakerNotes", "Camera"),
            NikonDataType::PictureControlData2 => ("MakerNotes", "Camera"),
            NikonDataType::SilentPhotography => ("MakerNotes", "Camera"),
            NikonDataType::BarometerInfo => ("MakerNotes", "Camera"),
            NikonDataType::PrintIM => ("MakerNotes", "Camera"),
            NikonDataType::CaptureData => ("MakerNotes", "Camera"),
            NikonDataType::CaptureVersion => ("MakerNotes", "Camera"),
            NikonDataType::CaptureOffsets => ("MakerNotes", "Camera"),
            NikonDataType::ScanIFD => ("NikonScan", "Camera"),
            NikonDataType::CaptureEditVersions => ("MakerNotes", "Camera"),
            NikonDataType::ICCProfile => ("MakerNotes", "Camera"),
            NikonDataType::CaptureOutput => ("MakerNotes", "Camera"),
            NikonDataType::NEFBitDepth => ("MakerNotes", "Camera"),
        }
    }
}
