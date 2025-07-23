//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

//! Olympus tag table structure generated from Olympus.pm
//! ExifTool: Olympus.pm %Olympus::Equipment

/// Olympus data types from %Olympus::Equipment table
/// Total tags: 25 (conditional: 0, with subdirectories: 0)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OlympusDataType {
    /// 0x0000: EquipmentVersion
    EquipmentVersion,
    /// 0x0100: CameraType2
    CameraType2,
    /// 0x0101: SerialNumber
    SerialNumber,
    /// 0x0102: InternalSerialNumber
    InternalSerialNumber,
    /// 0x0103: FocalPlaneDiagonal
    FocalPlaneDiagonal,
    /// 0x0104: BodyFirmwareVersion
    BodyFirmwareVersion,
    /// 0x0201: LensType
    LensType,
    /// 0x0202: LensSerialNumber
    LensSerialNumber,
    /// 0x0203: LensModel
    LensModel,
    /// 0x0204: LensFirmwareVersion
    LensFirmwareVersion,
    /// 0x0205: MaxApertureAtMinFocal
    MaxApertureAtMinFocal,
    /// 0x0206: MaxApertureAtMaxFocal
    MaxApertureAtMaxFocal,
    /// 0x0207: MinFocalLength
    MinFocalLength,
    /// 0x0208: MaxFocalLength
    MaxFocalLength,
    /// 0x020a: MaxAperture
    MaxAperture,
    /// 0x020b: LensProperties
    LensProperties,
    /// 0x0301: Extender
    Extender,
    /// 0x0302: ExtenderSerialNumber
    ExtenderSerialNumber,
    /// 0x0303: ExtenderModel
    ExtenderModel,
    /// 0x0304: ExtenderFirmwareVersion
    ExtenderFirmwareVersion,
    /// 0x0403: ConversionLens
    ConversionLens,
    /// 0x1000: FlashType
    FlashType,
    /// 0x1001: FlashModel
    FlashModel,
    /// 0x1002: FlashFirmwareVersion
    FlashFirmwareVersion,
    /// 0x1003: FlashSerialNumber
    FlashSerialNumber,
}

impl OlympusDataType {
    /// Get tag ID for this data type
    pub fn tag_id(&self) -> u16 {
        match self {
            OlympusDataType::EquipmentVersion => 0x0000,
            OlympusDataType::CameraType2 => 0x0100,
            OlympusDataType::SerialNumber => 0x0101,
            OlympusDataType::InternalSerialNumber => 0x0102,
            OlympusDataType::FocalPlaneDiagonal => 0x0103,
            OlympusDataType::BodyFirmwareVersion => 0x0104,
            OlympusDataType::LensType => 0x0201,
            OlympusDataType::LensSerialNumber => 0x0202,
            OlympusDataType::LensModel => 0x0203,
            OlympusDataType::LensFirmwareVersion => 0x0204,
            OlympusDataType::MaxApertureAtMinFocal => 0x0205,
            OlympusDataType::MaxApertureAtMaxFocal => 0x0206,
            OlympusDataType::MinFocalLength => 0x0207,
            OlympusDataType::MaxFocalLength => 0x0208,
            OlympusDataType::MaxAperture => 0x020a,
            OlympusDataType::LensProperties => 0x020b,
            OlympusDataType::Extender => 0x0301,
            OlympusDataType::ExtenderSerialNumber => 0x0302,
            OlympusDataType::ExtenderModel => 0x0303,
            OlympusDataType::ExtenderFirmwareVersion => 0x0304,
            OlympusDataType::ConversionLens => 0x0403,
            OlympusDataType::FlashType => 0x1000,
            OlympusDataType::FlashModel => 0x1001,
            OlympusDataType::FlashFirmwareVersion => 0x1002,
            OlympusDataType::FlashSerialNumber => 0x1003,
        }
    }

    /// Get data type from tag ID
    pub fn from_tag_id(tag_id: u16) -> Option<OlympusDataType> {
        match tag_id {
            0x0000 => Some(OlympusDataType::EquipmentVersion),
            0x0100 => Some(OlympusDataType::CameraType2),
            0x0101 => Some(OlympusDataType::SerialNumber),
            0x0102 => Some(OlympusDataType::InternalSerialNumber),
            0x0103 => Some(OlympusDataType::FocalPlaneDiagonal),
            0x0104 => Some(OlympusDataType::BodyFirmwareVersion),
            0x0201 => Some(OlympusDataType::LensType),
            0x0202 => Some(OlympusDataType::LensSerialNumber),
            0x0203 => Some(OlympusDataType::LensModel),
            0x0204 => Some(OlympusDataType::LensFirmwareVersion),
            0x0205 => Some(OlympusDataType::MaxApertureAtMinFocal),
            0x0206 => Some(OlympusDataType::MaxApertureAtMaxFocal),
            0x0207 => Some(OlympusDataType::MinFocalLength),
            0x0208 => Some(OlympusDataType::MaxFocalLength),
            0x020a => Some(OlympusDataType::MaxAperture),
            0x020b => Some(OlympusDataType::LensProperties),
            0x0301 => Some(OlympusDataType::Extender),
            0x0302 => Some(OlympusDataType::ExtenderSerialNumber),
            0x0303 => Some(OlympusDataType::ExtenderModel),
            0x0304 => Some(OlympusDataType::ExtenderFirmwareVersion),
            0x0403 => Some(OlympusDataType::ConversionLens),
            0x1000 => Some(OlympusDataType::FlashType),
            0x1001 => Some(OlympusDataType::FlashModel),
            0x1002 => Some(OlympusDataType::FlashFirmwareVersion),
            0x1003 => Some(OlympusDataType::FlashSerialNumber),
            _ => None,
        }
    }

    /// Get the ExifTool tag name
    pub fn name(&self) -> &'static str {
        match self {
            OlympusDataType::EquipmentVersion => "EquipmentVersion",
            OlympusDataType::CameraType2 => "CameraType2",
            OlympusDataType::SerialNumber => "SerialNumber",
            OlympusDataType::InternalSerialNumber => "InternalSerialNumber",
            OlympusDataType::FocalPlaneDiagonal => "FocalPlaneDiagonal",
            OlympusDataType::BodyFirmwareVersion => "BodyFirmwareVersion",
            OlympusDataType::LensType => "LensType",
            OlympusDataType::LensSerialNumber => "LensSerialNumber",
            OlympusDataType::LensModel => "LensModel",
            OlympusDataType::LensFirmwareVersion => "LensFirmwareVersion",
            OlympusDataType::MaxApertureAtMinFocal => "MaxApertureAtMinFocal",
            OlympusDataType::MaxApertureAtMaxFocal => "MaxApertureAtMaxFocal",
            OlympusDataType::MinFocalLength => "MinFocalLength",
            OlympusDataType::MaxFocalLength => "MaxFocalLength",
            OlympusDataType::MaxAperture => "MaxAperture",
            OlympusDataType::LensProperties => "LensProperties",
            OlympusDataType::Extender => "Extender",
            OlympusDataType::ExtenderSerialNumber => "ExtenderSerialNumber",
            OlympusDataType::ExtenderModel => "ExtenderModel",
            OlympusDataType::ExtenderFirmwareVersion => "ExtenderFirmwareVersion",
            OlympusDataType::ConversionLens => "ConversionLens",
            OlympusDataType::FlashType => "FlashType",
            OlympusDataType::FlashModel => "FlashModel",
            OlympusDataType::FlashFirmwareVersion => "FlashFirmwareVersion",
            OlympusDataType::FlashSerialNumber => "FlashSerialNumber",
        }
    }

    /// Check if this tag has a subdirectory
    pub fn has_subdirectory(&self) -> bool {
        false
    }

    /// Get the group hierarchy for this tag
    pub fn groups(&self) -> (&'static str, &'static str) {
        match self {
            OlympusDataType::EquipmentVersion => ("MakerNotes", "Camera"),
            OlympusDataType::CameraType2 => ("MakerNotes", "Camera"),
            OlympusDataType::SerialNumber => ("MakerNotes", "Camera"),
            OlympusDataType::InternalSerialNumber => ("MakerNotes", "Camera"),
            OlympusDataType::FocalPlaneDiagonal => ("MakerNotes", "Camera"),
            OlympusDataType::BodyFirmwareVersion => ("MakerNotes", "Camera"),
            OlympusDataType::LensType => ("MakerNotes", "Camera"),
            OlympusDataType::LensSerialNumber => ("MakerNotes", "Camera"),
            OlympusDataType::LensModel => ("MakerNotes", "Camera"),
            OlympusDataType::LensFirmwareVersion => ("MakerNotes", "Camera"),
            OlympusDataType::MaxApertureAtMinFocal => ("MakerNotes", "Camera"),
            OlympusDataType::MaxApertureAtMaxFocal => ("MakerNotes", "Camera"),
            OlympusDataType::MinFocalLength => ("MakerNotes", "Camera"),
            OlympusDataType::MaxFocalLength => ("MakerNotes", "Camera"),
            OlympusDataType::MaxAperture => ("MakerNotes", "Camera"),
            OlympusDataType::LensProperties => ("MakerNotes", "Camera"),
            OlympusDataType::Extender => ("MakerNotes", "Camera"),
            OlympusDataType::ExtenderSerialNumber => ("MakerNotes", "Camera"),
            OlympusDataType::ExtenderModel => ("MakerNotes", "Camera"),
            OlympusDataType::ExtenderFirmwareVersion => ("MakerNotes", "Camera"),
            OlympusDataType::ConversionLens => ("MakerNotes", "Camera"),
            OlympusDataType::FlashType => ("MakerNotes", "Camera"),
            OlympusDataType::FlashModel => ("MakerNotes", "Camera"),
            OlympusDataType::FlashFirmwareVersion => ("MakerNotes", "Camera"),
            OlympusDataType::FlashSerialNumber => ("MakerNotes", "Camera"),
        }
    }
}

/// Get tag name for Equipment subdirectory
/// ExifTool: Olympus.pm %Olympus::Equipment table
pub fn get_equipment_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        0x0000 => Some("EquipmentVersion"),
        0x0100 => Some("CameraType2"),
        0x0101 => Some("SerialNumber"),
        0x0102 => Some("InternalSerialNumber"),
        0x0103 => Some("FocalPlaneDiagonal"),
        0x0104 => Some("BodyFirmwareVersion"),
        0x0201 => Some("LensType"),
        0x0202 => Some("LensSerialNumber"),
        0x0203 => Some("LensModel"),
        0x0204 => Some("LensFirmwareVersion"),
        0x0205 => Some("MaxApertureAtMinFocal"),
        0x0206 => Some("MaxApertureAtMaxFocal"),
        0x0207 => Some("MinFocalLength"),
        0x0208 => Some("MaxFocalLength"),
        0x020a => Some("MaxAperture"),
        0x020b => Some("LensProperties"),
        0x0301 => Some("Extender"),
        0x0302 => Some("ExtenderSerialNumber"),
        0x0303 => Some("ExtenderModel"),
        0x0304 => Some("ExtenderFirmwareVersion"),
        0x0403 => Some("ConversionLens"),
        0x1000 => Some("FlashType"),
        0x1001 => Some("FlashModel"),
        0x1002 => Some("FlashFirmwareVersion"),
        0x1003 => Some("FlashSerialNumber"),
        _ => None,
    }
}
