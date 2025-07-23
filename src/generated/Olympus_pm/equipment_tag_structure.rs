//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

//! Olympus tag table structure generated from Olympus.pm
//! ExifTool: Olympus.pm %Olympus::Equipment

/// Olympus data types from %Olympus::Equipment table
/// Total tags: 25 (conditional: 0, with subdirectories: 0)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OlympusEquipmentDataType {
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

impl OlympusEquipmentDataType {
    /// Get tag ID for this data type
    pub fn tag_id(&self) -> u16 {
        match self {
            OlympusEquipmentDataType::EquipmentVersion => 0x0000,
            OlympusEquipmentDataType::CameraType2 => 0x0100,
            OlympusEquipmentDataType::SerialNumber => 0x0101,
            OlympusEquipmentDataType::InternalSerialNumber => 0x0102,
            OlympusEquipmentDataType::FocalPlaneDiagonal => 0x0103,
            OlympusEquipmentDataType::BodyFirmwareVersion => 0x0104,
            OlympusEquipmentDataType::LensType => 0x0201,
            OlympusEquipmentDataType::LensSerialNumber => 0x0202,
            OlympusEquipmentDataType::LensModel => 0x0203,
            OlympusEquipmentDataType::LensFirmwareVersion => 0x0204,
            OlympusEquipmentDataType::MaxApertureAtMinFocal => 0x0205,
            OlympusEquipmentDataType::MaxApertureAtMaxFocal => 0x0206,
            OlympusEquipmentDataType::MinFocalLength => 0x0207,
            OlympusEquipmentDataType::MaxFocalLength => 0x0208,
            OlympusEquipmentDataType::MaxAperture => 0x020a,
            OlympusEquipmentDataType::LensProperties => 0x020b,
            OlympusEquipmentDataType::Extender => 0x0301,
            OlympusEquipmentDataType::ExtenderSerialNumber => 0x0302,
            OlympusEquipmentDataType::ExtenderModel => 0x0303,
            OlympusEquipmentDataType::ExtenderFirmwareVersion => 0x0304,
            OlympusEquipmentDataType::ConversionLens => 0x0403,
            OlympusEquipmentDataType::FlashType => 0x1000,
            OlympusEquipmentDataType::FlashModel => 0x1001,
            OlympusEquipmentDataType::FlashFirmwareVersion => 0x1002,
            OlympusEquipmentDataType::FlashSerialNumber => 0x1003,
        }
    }

    /// Get data type from tag ID
    pub fn from_tag_id(tag_id: u16) -> Option<OlympusEquipmentDataType> {
        match tag_id {
            0x0000 => Some(OlympusEquipmentDataType::EquipmentVersion),
            0x0100 => Some(OlympusEquipmentDataType::CameraType2),
            0x0101 => Some(OlympusEquipmentDataType::SerialNumber),
            0x0102 => Some(OlympusEquipmentDataType::InternalSerialNumber),
            0x0103 => Some(OlympusEquipmentDataType::FocalPlaneDiagonal),
            0x0104 => Some(OlympusEquipmentDataType::BodyFirmwareVersion),
            0x0201 => Some(OlympusEquipmentDataType::LensType),
            0x0202 => Some(OlympusEquipmentDataType::LensSerialNumber),
            0x0203 => Some(OlympusEquipmentDataType::LensModel),
            0x0204 => Some(OlympusEquipmentDataType::LensFirmwareVersion),
            0x0205 => Some(OlympusEquipmentDataType::MaxApertureAtMinFocal),
            0x0206 => Some(OlympusEquipmentDataType::MaxApertureAtMaxFocal),
            0x0207 => Some(OlympusEquipmentDataType::MinFocalLength),
            0x0208 => Some(OlympusEquipmentDataType::MaxFocalLength),
            0x020a => Some(OlympusEquipmentDataType::MaxAperture),
            0x020b => Some(OlympusEquipmentDataType::LensProperties),
            0x0301 => Some(OlympusEquipmentDataType::Extender),
            0x0302 => Some(OlympusEquipmentDataType::ExtenderSerialNumber),
            0x0303 => Some(OlympusEquipmentDataType::ExtenderModel),
            0x0304 => Some(OlympusEquipmentDataType::ExtenderFirmwareVersion),
            0x0403 => Some(OlympusEquipmentDataType::ConversionLens),
            0x1000 => Some(OlympusEquipmentDataType::FlashType),
            0x1001 => Some(OlympusEquipmentDataType::FlashModel),
            0x1002 => Some(OlympusEquipmentDataType::FlashFirmwareVersion),
            0x1003 => Some(OlympusEquipmentDataType::FlashSerialNumber),
            _ => None,
        }
    }

    /// Get the ExifTool tag name
    pub fn name(&self) -> &'static str {
        match self {
            OlympusEquipmentDataType::EquipmentVersion => "EquipmentVersion",
            OlympusEquipmentDataType::CameraType2 => "CameraType2",
            OlympusEquipmentDataType::SerialNumber => "SerialNumber",
            OlympusEquipmentDataType::InternalSerialNumber => "InternalSerialNumber",
            OlympusEquipmentDataType::FocalPlaneDiagonal => "FocalPlaneDiagonal",
            OlympusEquipmentDataType::BodyFirmwareVersion => "BodyFirmwareVersion",
            OlympusEquipmentDataType::LensType => "LensType",
            OlympusEquipmentDataType::LensSerialNumber => "LensSerialNumber",
            OlympusEquipmentDataType::LensModel => "LensModel",
            OlympusEquipmentDataType::LensFirmwareVersion => "LensFirmwareVersion",
            OlympusEquipmentDataType::MaxApertureAtMinFocal => "MaxApertureAtMinFocal",
            OlympusEquipmentDataType::MaxApertureAtMaxFocal => "MaxApertureAtMaxFocal",
            OlympusEquipmentDataType::MinFocalLength => "MinFocalLength",
            OlympusEquipmentDataType::MaxFocalLength => "MaxFocalLength",
            OlympusEquipmentDataType::MaxAperture => "MaxAperture",
            OlympusEquipmentDataType::LensProperties => "LensProperties",
            OlympusEquipmentDataType::Extender => "Extender",
            OlympusEquipmentDataType::ExtenderSerialNumber => "ExtenderSerialNumber",
            OlympusEquipmentDataType::ExtenderModel => "ExtenderModel",
            OlympusEquipmentDataType::ExtenderFirmwareVersion => "ExtenderFirmwareVersion",
            OlympusEquipmentDataType::ConversionLens => "ConversionLens",
            OlympusEquipmentDataType::FlashType => "FlashType",
            OlympusEquipmentDataType::FlashModel => "FlashModel",
            OlympusEquipmentDataType::FlashFirmwareVersion => "FlashFirmwareVersion",
            OlympusEquipmentDataType::FlashSerialNumber => "FlashSerialNumber",
        }
    }

    /// Check if this tag has a subdirectory
    pub fn has_subdirectory(&self) -> bool {
        false
    }

    /// Get the group hierarchy for this tag
    pub fn groups(&self) -> (&'static str, &'static str) {
        match self {
            OlympusEquipmentDataType::EquipmentVersion => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::CameraType2 => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::SerialNumber => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::InternalSerialNumber => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::FocalPlaneDiagonal => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::BodyFirmwareVersion => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::LensType => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::LensSerialNumber => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::LensModel => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::LensFirmwareVersion => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::MaxApertureAtMinFocal => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::MaxApertureAtMaxFocal => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::MinFocalLength => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::MaxFocalLength => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::MaxAperture => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::LensProperties => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::Extender => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::ExtenderSerialNumber => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::ExtenderModel => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::ExtenderFirmwareVersion => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::ConversionLens => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::FlashType => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::FlashModel => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::FlashFirmwareVersion => ("MakerNotes", "Camera"),
            OlympusEquipmentDataType::FlashSerialNumber => ("MakerNotes", "Camera"),
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
