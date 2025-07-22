//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

//! Olympus tag table structure generated from Olympus.pm
//! ExifTool: Olympus.pm %Olympus::Main

/// Olympus data types from %Olympus::Main table
/// Total tags: 119 (conditional: 9, with subdirectories: 11)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OlympusDataType {
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

    /// Get data type from tag ID
    pub fn from_tag_id(tag_id: u16) -> Option<OlympusDataType> {
        match tag_id {
            0x2010 => Some(OlympusDataType::Equipment),
            0x2020 => Some(OlympusDataType::CameraSettings),
            0x2030 => Some(OlympusDataType::RawDevelopment),
            0x2031 => Some(OlympusDataType::RawDev2),
            0x2040 => Some(OlympusDataType::ImageProcessing),
            0x2050 => Some(OlympusDataType::FocusInfo),
            0x3000 => Some(OlympusDataType::RawInfo),
            0x4000 => Some(OlympusDataType::MainInfo),
            0x5000 => Some(OlympusDataType::UnknownInfo),
            _ => None,
        }
    }

    /// Get the ExifTool tag name
    pub fn name(&self) -> &'static str {
        match self {
            OlympusDataType::Equipment => "Equipment",
            OlympusDataType::CameraSettings => "CameraSettings",
            OlympusDataType::RawDevelopment => "RawDevelopment",
            OlympusDataType::RawDev2 => "RawDev2",
            OlympusDataType::ImageProcessing => "ImageProcessing",
            OlympusDataType::FocusInfo => "FocusInfo",
            OlympusDataType::RawInfo => "RawInfo",
            OlympusDataType::MainInfo => "MainInfo",
            OlympusDataType::UnknownInfo => "UnknownInfo",
        }
    }

    /// Check if this tag has a subdirectory
    pub fn has_subdirectory(&self) -> bool {
        match self {
            OlympusDataType::Equipment => true,
            OlympusDataType::CameraSettings => true,
            OlympusDataType::RawDevelopment => true,
            OlympusDataType::RawDev2 => true,
            OlympusDataType::ImageProcessing => true,
            OlympusDataType::FocusInfo => true,
            OlympusDataType::RawInfo => true,
            _ => false,
        }
    }

    /// Get the group hierarchy for this tag
    pub fn groups(&self) -> (&'static str, &'static str) {
        ("MakerNotes", "Camera")
    }
}

/// Get tag name for Main subdirectory
/// ExifTool: Olympus.pm %Olympus::Main table
pub fn get_main_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        0x2010 => Some("Equipment"),
        0x2020 => Some("CameraSettings"),
        0x2030 => Some("RawDevelopment"),
        0x2031 => Some("RawDev2"),
        0x2040 => Some("ImageProcessing"),
        0x2050 => Some("FocusInfo"),
        0x3000 => Some("RawInfo"),
        0x4000 => Some("MainInfo"),
        0x5000 => Some("UnknownInfo"),
        _ => None,
    }
}
