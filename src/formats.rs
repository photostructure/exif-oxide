//! File format detection and basic parsing
//!
//! This module handles file format detection and provides the foundation
//! for format-specific parsing. For Milestone 0a, this is mostly placeholder
//! functionality that will be expanded in later milestones.

use crate::types::{ExifData, ExifError, Result, TagValue};
use std::collections::HashMap;
use std::path::Path;

/// Detect file format from file extension and magic bytes
///
/// This is a simplified version that will be expanded in Milestone 1
/// when we implement actual file I/O and magic byte detection.
pub fn detect_file_format(path: &Path) -> Result<FileFormat> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match extension.as_str() {
        "jpg" | "jpeg" => Ok(FileFormat::Jpeg),
        "tif" | "tiff" => Ok(FileFormat::Tiff),
        "cr2" | "cr3" | "crw" => Ok(FileFormat::CanonRaw),
        "nef" => Ok(FileFormat::NikonRaw),
        "arw" => Ok(FileFormat::SonyRaw),
        "dng" => Ok(FileFormat::Dng),
        _ => Err(ExifError::Unsupported(format!(
            "Unsupported file format: {extension}"
        ))),
    }
}

/// Supported file formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    Jpeg,
    Tiff,
    CanonRaw,
    NikonRaw,
    SonyRaw,
    Dng,
}

impl FileFormat {
    /// Get the MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            FileFormat::Jpeg => "image/jpeg",
            FileFormat::Tiff => "image/tiff",
            FileFormat::CanonRaw => "image/x-canon-cr2",
            FileFormat::NikonRaw => "image/x-nikon-nef",
            FileFormat::SonyRaw => "image/x-sony-arw",
            FileFormat::Dng => "image/x-adobe-dng",
        }
    }

    /// Get the typical file extension
    pub fn extension(&self) -> &'static str {
        match self {
            FileFormat::Jpeg => "jpg",
            FileFormat::Tiff => "tif",
            FileFormat::CanonRaw => "cr2",
            FileFormat::NikonRaw => "nef",
            FileFormat::SonyRaw => "arw",
            FileFormat::Dng => "dng",
        }
    }
}

/// Extract metadata from a file (Milestone 0a: mock implementation)
///
/// This function provides the interface that will be implemented in later
/// milestones. For now, it returns mock data to satisfy the CLI requirements.
pub fn extract_metadata(path: &Path, show_missing: bool) -> Result<ExifData> {
    let format = detect_file_format(path)?;

    // Create mock metadata based on file format
    let mut tags = HashMap::new();

    // Basic file information (mock)
    tags.insert(
        "FileName".to_string(),
        TagValue::String(
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        ),
    );

    tags.insert(
        "Directory".to_string(),
        TagValue::String(
            path.parent()
                .unwrap_or_else(|| Path::new("."))
                .to_string_lossy()
                .to_string(),
        ),
    );

    tags.insert(
        "FileSize".to_string(),
        TagValue::String("MOCK - Not implemented".to_string()),
    );
    tags.insert(
        "FileModifyDate".to_string(),
        TagValue::String("MOCK - Not implemented".to_string()),
    );
    tags.insert(
        "FileAccessDate".to_string(),
        TagValue::String("MOCK - Not implemented".to_string()),
    );
    tags.insert(
        "FilePermissions".to_string(),
        TagValue::String("MOCK - Not implemented".to_string()),
    );
    tags.insert(
        "FileType".to_string(),
        TagValue::String(format!("MOCK - {format:?}")),
    );
    tags.insert(
        "FileTypeExtension".to_string(),
        TagValue::String(format.extension().to_string()),
    );
    tags.insert(
        "MIMEType".to_string(),
        TagValue::String(format.mime_type().to_string()),
    );

    // Format-specific mock data
    match format {
        FileFormat::Jpeg => {
            tags.insert("ImageWidth".to_string(), TagValue::U32(3000));
            tags.insert("ImageHeight".to_string(), TagValue::U32(2000));
            tags.insert(
                "Make".to_string(),
                TagValue::String("MOCK Camera Make".to_string()),
            );
            tags.insert(
                "Model".to_string(),
                TagValue::String("MOCK Camera Model".to_string()),
            );
            tags.insert("Orientation".to_string(), TagValue::U16(1));
        }
        FileFormat::CanonRaw => {
            tags.insert("Make".to_string(), TagValue::String("Canon".to_string()));
            tags.insert(
                "Model".to_string(),
                TagValue::String("MOCK Canon Model".to_string()),
            );
            tags.insert("ImageWidth".to_string(), TagValue::U32(5184));
            tags.insert("ImageHeight".to_string(), TagValue::U32(3456));
        }
        _ => {
            // Generic RAW file mock data
            tags.insert(
                "Make".to_string(),
                TagValue::String("MOCK Manufacturer".to_string()),
            );
            tags.insert(
                "Model".to_string(),
                TagValue::String("MOCK Camera Model".to_string()),
            );
        }
    }

    let missing_implementations = if show_missing {
        Some(vec![
            "Real file I/O and size detection".to_string(),
            "Magic byte file type detection".to_string(),
            "JPEG segment parsing".to_string(),
            "EXIF header parsing".to_string(),
            "IFD (Image File Directory) parsing".to_string(),
            "Tag value extraction and type conversion".to_string(),
            "PrintConv implementations (human-readable values)".to_string(),
            "ValueConv implementations (logical value conversion)".to_string(),
            "MakerNote parsing".to_string(),
            "Subdirectory following".to_string(),
            "GPS coordinate conversion".to_string(),
            "Date/time parsing".to_string(),
        ])
    } else {
        None
    };

    Ok(ExifData {
        source_file: path.to_string_lossy().to_string(),
        exif_tool_version: "0.1.0-oxide".to_string(),
        tags,
        errors: vec![], // No errors in mock implementation
        missing_implementations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_file_format_detection() {
        assert_eq!(
            detect_file_format(&PathBuf::from("test.jpg")).unwrap(),
            FileFormat::Jpeg
        );
        assert_eq!(
            detect_file_format(&PathBuf::from("test.CR2")).unwrap(),
            FileFormat::CanonRaw
        );
        assert!(detect_file_format(&PathBuf::from("test.unknown")).is_err());
    }

    #[test]
    fn test_format_properties() {
        assert_eq!(FileFormat::Jpeg.mime_type(), "image/jpeg");
        assert_eq!(FileFormat::Jpeg.extension(), "jpg");
        assert_eq!(FileFormat::CanonRaw.mime_type(), "image/x-canon-cr2");
    }

    #[test]
    fn test_extract_metadata() {
        let path = PathBuf::from("test.jpg");
        let metadata = extract_metadata(&path, false).unwrap();

        assert_eq!(metadata.source_file, "test.jpg");
        assert_eq!(metadata.exif_tool_version, "0.1.0-oxide");
        assert!(metadata.tags.contains_key("FileName"));
        assert!(metadata.tags.contains_key("FileType"));
        assert!(metadata.missing_implementations.is_none());

        // Test with --show-missing
        let metadata_with_missing = extract_metadata(&path, true).unwrap();
        assert!(metadata_with_missing.missing_implementations.is_some());
    }
}
