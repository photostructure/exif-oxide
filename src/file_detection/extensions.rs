//! File extension processing and normalization
//!
//! Handles extension-based file type detection following ExifTool's rules.

use super::FileDetectionError;
use std::path::Path;

/// Get file type candidates based on file extension
/// ExifTool equivalent: GetFileType() in ExifTool.pm:9010-9050
pub fn get_candidates_from_extension(path: &Path) -> Result<Vec<String>, FileDetectionError> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or(FileDetectionError::InvalidPath)?;

    // Normalize extension to uppercase (ExifTool convention)
    let normalized_ext = normalize_extension(extension);

    // Resolve through fileTypeLookup with alias following
    // ExifTool.pm:258-404 %fileTypeLookup hash defines extension mappings
    use crate::generated::ExifTool_pm::file_type_lookup::resolve_file_type;

    // Check if this extension is known to ExifTool
    let is_known_extension = resolve_file_type(&normalized_ext).is_some();

    // For HEIC/HEIF, we need special handling
    // Even if not in the generated lookup, these are valid extensions
    let is_heif_extension = matches!(normalized_ext.as_str(), "HEIC" | "HEIF" | "HIF");

    if is_known_extension || is_heif_extension {
        // For most formats, the extension itself is the file type candidate
        // The formats array tells us what processing module to use, not the file type
        // ExifTool.pm:2940-2950 - GetFileType returns the extension-based type

        // Special case: Some extensions are aliases that should map to a different type
        // These are hardcoded in ExifTool.pm GetFileType()
        match normalized_ext.as_str() {
            "3GP2" => Ok(vec!["3G2".to_string()]), // ExifTool.pm alias
            "MTS" => Ok(vec!["M2TS".to_string()]), // ExifTool.pm alias
            // HEIC/HEIF/HIF extensions should use MOV format for detection
            // ExifTool QuickTime.pm handles these as MOV-based formats
            "HEIC" | "HEIF" | "HIF" => Ok(vec!["MOV".to_string()]),
            _ => Ok(vec![normalized_ext.clone()]), // Use the extension as the type
        }
    } else {
        // Unknown extension - return empty candidates to trigger magic number scanning
        // This matches ExifTool.pm behavior where GetFileType() returns () for unknown extensions
        Ok(vec![])
    }
}

/// Normalize file extension following ExifTool's rules
/// ExifTool equivalent: GetFileExtension() in ExifTool.pm:9013-9040
pub fn normalize_extension(extension: &str) -> String {
    let upper_ext = extension.to_uppercase();

    // ExifTool hardcoded extension conversions
    // These are critical for consistency - TRUST-EXIFTOOL
    match upper_ext.as_str() {
        "TIF" => "TIFF".to_string(), // ExifTool.pm:9019 - hardcoded for TIFF consistency
        "JPG" => "JPEG".to_string(),
        "3GP2" => "3G2".to_string(),
        "AIF" => "AIFF".to_string(),
        _ => upper_ext,
    }
}

/// Check if a file type has a processing module defined
/// This mimics ExifTool's %moduleName hash behavior
pub fn has_processing_module(file_type: &str) -> bool {
    // In ExifTool, having a module means it can be processed even without magic match
    // Notable examples include JXL -> Jpeg2000 module
    // We check if the file type has a defined format/processing path
    use crate::generated::ExifTool_pm::file_type_lookup::resolve_file_type;

    // If resolve_file_type returns Some, it means ExifTool knows how to process this type
    resolve_file_type(file_type).is_some()
}
