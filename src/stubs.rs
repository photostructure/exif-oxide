//! Temporary stub types to allow compilation
//!
//! These are placeholder types that allow the project to compile while
//! the full implementations are being developed. They should be replaced
//! with proper generated types from the codegen system.
//!
//! TODO: Replace all these stubs with properly generated types

/// Canon data type enum stub
/// TODO: Generate from Canon.pm tag structure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonDataType {
    Main,
    CameraSettings,
    FocalLength,
    ShotInfo,
    Panorama,
    ImageType,
    FirmwareVersion,
    FileNumber,
    OwnerName,
    SerialNumber,
    CameraInfo,
    ModelID,
    PictureInfo,
    ThumbnailImageValidArea,
    SerialNumberFormat,
    SuperMacro,
    DateStampMode,
    MyColors,
    FirmwareRevision,
    Categories,
    FaceDetect1,
    FaceDetect2,
    AFInfo,
    ContrastInfo,
    ImageUniqueID,
    RawDataOffset,
    OriginalDecisionDataOffset,
    CustomFunctions,
    PersonalFunctions,
    PersonalFunctionValues,
    FileInfo,
    LensModel,
    InternalSerialNumber,
    DustRemovalData,
    CropInfo,
    CustomFunctions2,
    AspectInfo,
    ProcessingInfo,
    ToneInfo,
    MeasuredColor,
    ColorTemp,
    ColorSpace,
    VRDOffset,
    SensorInfo,
    ColorData,
}

/// Olympus data type enum stub
/// TODO: Generate from Olympus.pm tag structure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OlympusDataType {
    Main,
    Equipment,
    CameraSettings,
    RawDevelopment,
    RawDev2,
    ImageProcessing,
    FocusInfo,
    RawInfo,
    MainInfo,
    UnknownInfo,
}

impl OlympusDataType {
    /// Get the tag ID for this data type
    pub fn tag_id(&self) -> u16 {
        match self {
            OlympusDataType::Main => 0x0000,
            OlympusDataType::Equipment => 0x2010,
            OlympusDataType::CameraSettings => 0x2020,
            OlympusDataType::RawDevelopment => 0x2030,
            OlympusDataType::RawDev2 => 0x2031,
            OlympusDataType::ImageProcessing => 0x2040,
            OlympusDataType::FocusInfo => 0x2050,
            OlympusDataType::RawInfo => 0x3000,
            OlympusDataType::MainInfo => 0x4000,
            OlympusDataType::UnknownInfo => 0x5000,
        }
    }
}

/// FujiFilm FFMV table stub
/// TODO: Generate from FujiFilm.pm process binary data
pub struct FujiFilmFFMVTable {
    pub first_entry: u16,
}

impl FujiFilmFFMVTable {
    pub fn new() -> Self {
        Self { first_entry: 0 }
    }

    pub fn get_tag_name(&self, _offset: usize) -> Option<&'static str> {
        // Stub implementation
        None
    }

    pub fn get_format(&self, _offset: usize) -> Option<&'static str> {
        // Stub implementation
        None
    }
}

/// Canon conditional tags stub
/// TODO: Generate from Canon.pm conditional tags
pub struct CanonConditionalTags;

impl CanonConditionalTags {
    pub fn new() -> Self {
        Self
    }

    pub fn resolve_tag(&self, _tag_id: &str, _context: &()) -> Option<ResolvedTag> {
        // Stub implementation
        None
    }
}

/// Resolved tag structure stub
#[derive(Debug, Clone)]
pub struct ResolvedTag {
    pub name: String,
    pub format: Option<String>,
    pub writable: bool,
    pub subdirectory: bool,
}

/// FujiFilm model detection stub
/// TODO: Generate from FujiFilm.pm model detection
pub struct FujiFilmModelDetection;

impl FujiFilmModelDetection {
    pub fn new(_model: &str) -> Self {
        Self
    }

    pub fn resolve_conditional_tag(&self, _tag_id: &str, _context: &()) -> Option<String> {
        // Stub implementation
        None
    }
}

/// ConditionalContext stub
/// TODO: Generate as part of conditional tag system
#[derive(Debug, Clone)]
pub struct ConditionalContext {
    pub make: Option<String>,
    pub model: Option<String>,
    pub count: Option<u32>,
    pub format: Option<String>,
    pub binary_data: Option<Vec<u8>>,
}

/// FujiFilm ConditionalContext stub
/// TODO: Generate as part of FujiFilm conditional tag system
#[derive(Debug, Clone)]
pub struct FujiFilmConditionalContext {
    pub make: Option<String>,
    pub model: Option<String>,
    pub count: Option<u32>,
    pub format: Option<String>,
}
