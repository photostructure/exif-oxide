//! File format detection and processing
//!
//! This module handles different image file formats and extracts
//! metadata from each according to format-specific requirements.

mod detection;
mod jpeg;
mod tiff;

pub use detection::{
    detect_file_format, detect_file_format_from_path, get_format_properties, FileFormat,
};
pub use jpeg::{extract_jpeg_exif, scan_jpeg_segments, JpegSegment, JpegSegmentInfo};
pub use tiff::{extract_tiff_exif, get_tiff_endianness, validate_tiff_format};

use crate::exif::ExifReader;
use crate::generated::{EXIF_MAIN_TAGS, REQUIRED_PRINT_CONV, REQUIRED_VALUE_CONV};
use crate::types::{ExifData, Result, TagEntry, TagValue};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

/// Extract metadata from a file (Milestone 1: real file I/O with JPEG detection)
///
/// This function now implements real file reading and JPEG segment scanning.
/// It detects JPEG files by magic bytes and locates EXIF data in APP1 segments.
pub fn extract_metadata(path: &Path, show_missing: bool) -> Result<ExifData> {
    // Ensure conversions are registered
    crate::init();

    // Open file with buffered reading for performance
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // Detect format using magic bytes
    let format = detect_file_format(&mut reader)?;

    // Get actual file metadata
    let file_metadata = std::fs::metadata(path)?;
    let file_size = file_metadata.len();

    let mut tags = HashMap::new();
    let mut tag_entries = Vec::new();

    // Basic file information (now real data) - create as TagEntry objects
    let filename = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    tag_entries.push(TagEntry {
        group: "File".to_string(),
        group1: "File".to_string(),
        name: "FileName".to_string(),
        value: TagValue::String(filename.clone()),
        print: filename,
    });

    let directory = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_string_lossy()
        .to_string();
    tag_entries.push(TagEntry {
        group: "File".to_string(),
        group1: "File".to_string(),
        name: "Directory".to_string(),
        value: TagValue::String(directory.clone()),
        print: directory,
    });

    // Handle file size - use U32 if it fits, otherwise F64 for large files
    let file_size_value = if file_size <= u32::MAX as u64 {
        TagValue::U32(file_size as u32)
    } else {
        TagValue::F64(file_size as f64)
    };
    tag_entries.push(TagEntry {
        group: "File".to_string(),
        group1: "File".to_string(),
        name: "FileSize".to_string(),
        value: file_size_value,
        print: file_size.to_string(),
    });

    // Format file modification time to match ExifTool format: "YYYY:MM:DD HH:MM:SS±TZ:TZ"
    // ExifTool.pm formats this as local time with timezone offset
    if let Ok(modified) = file_metadata.modified() {
        use chrono::{DateTime, Local};
        let datetime: DateTime<Local> = modified.into();
        // Format to match ExifTool exactly: "2025:06:30 10:16:40-07:00"
        let formatted = datetime.format("%Y:%m:%d %H:%M:%S%:z").to_string();
        tag_entries.push(TagEntry {
            group: "File".to_string(),
            group1: "File".to_string(),
            name: "FileModifyDate".to_string(),
            value: TagValue::String(formatted.clone()),
            print: formatted,
        });
    }

    // Add FileType and FileTypeExtension using ExifTool-compatible values
    let file_type = format.file_type().to_string();
    tag_entries.push(TagEntry {
        group: "File".to_string(),
        group1: "File".to_string(),
        name: "FileType".to_string(),
        value: TagValue::String(file_type.clone()),
        print: file_type,
    });

    let file_type_ext = format.file_type_extension().to_string();
    tag_entries.push(TagEntry {
        group: "File".to_string(),
        group1: "File".to_string(),
        name: "FileTypeExtension".to_string(),
        value: TagValue::String(file_type_ext.clone()),
        print: file_type_ext,
    });

    let mime_type = format.mime_type().to_string();
    tag_entries.push(TagEntry {
        group: "File".to_string(),
        group1: "File".to_string(),
        name: "MIMEType".to_string(),
        value: TagValue::String(mime_type.clone()),
        print: mime_type,
    });

    // Format-specific processing
    match format {
        FileFormat::Jpeg => {
            // Scan for EXIF data in JPEG segments
            match scan_jpeg_segments(&mut reader)? {
                Some(segment_info) => {
                    let exif_status = format!(
                        "EXIF data found in APP1 segment at offset {:#x}, length {} bytes",
                        segment_info.offset, segment_info.length
                    );

                    // Add EXIF detection status
                    tags.insert(
                        "System:ExifDetectionStatus".to_string(),
                        TagValue::String(exif_status),
                    );

                    // Extract actual EXIF data using our new ExifReader
                    reader.seek(SeekFrom::Start(segment_info.offset))?;
                    let mut exif_data = vec![0u8; segment_info.length as usize];
                    reader.read_exact(&mut exif_data)?;

                    // Parse EXIF data
                    let mut exif_reader = ExifReader::new();
                    match exif_reader.parse_exif_data(&exif_data) {
                        Ok(()) => {
                            // Successfully parsed EXIF - extract all found tags using new TagEntry API
                            let mut exif_tag_entries = exif_reader.get_all_tag_entries();

                            // Append EXIF tag entries to our collection
                            tag_entries.append(&mut exif_tag_entries);

                            // Also populate legacy tags for backward compatibility
                            let exif_tags = exif_reader.get_all_tags();
                            for (key, value) in exif_tags {
                                tags.insert(key, value);
                            }

                            // Add EXIF processing warnings as tags for debugging
                            for (i, warning) in exif_reader.get_warnings().iter().enumerate() {
                                tags.insert(
                                    format!("Warning:ExifWarning{i}"),
                                    TagValue::String(warning.clone()),
                                );
                            }
                        }
                        Err(e) => {
                            // Failed to parse EXIF - add error information
                            tags.insert(
                                "Warning:ExifParseError".to_string(),
                                TagValue::String(format!("Failed to parse EXIF: {e}")),
                            );
                        }
                    }
                }
                None => {
                    // No EXIF data found
                    tags.insert(
                        "System:ExifDetectionStatus".to_string(),
                        TagValue::String("No EXIF data found in JPEG".to_string()),
                    );
                }
            }
        }
        FileFormat::Tiff => {
            // For TIFF, the entire file is potentially EXIF data
            tags.insert(
                "System:ExifDetectionStatus".to_string(),
                TagValue::String(
                    "TIFF format detected (EXIF parsing deferred to future milestone)".to_string(),
                ),
            );
        }
        _ => {
            // Other formats not yet supported
            tags.insert(
                "System:ExifDetectionStatus".to_string(),
                TagValue::String(format!(
                    "Format {format:?} not yet supported for EXIF extraction"
                )),
            );
        }
    }

    // Collect any missing required tags for --show-missing functionality
    let missing_implementations = if show_missing {
        let mut missing = Vec::new();

        // Check for missing mainstream tags
        for tag_def in EXIF_MAIN_TAGS.iter() {
            let tag_found = tags.keys().any(|key| {
                key.contains(&format!("Tag_{:04X}", tag_def.id))
                    || key.contains(&format!("{:#x}", tag_def.id))
            });

            if !tag_found {
                missing.push(format!("Tag_{:04X}", tag_def.id));
            }
        }

        // Check for missing required ValueConv functions
        for conv_id in REQUIRED_VALUE_CONV.iter() {
            // These would be checked during value conversion
            // For now, just note that we need to implement them
            missing.push(format!("ValueConv_{conv_id}"));
        }

        // Check for missing required PrintConv functions
        for conv_id in REQUIRED_PRINT_CONV.iter() {
            // These would be checked during print conversion
            // For now, just note that we need to implement them
            missing.push(format!("PrintConv_{conv_id}"));
        }

        if missing.is_empty() {
            None
        } else {
            Some(missing)
        }
    } else {
        None
    };

    // Create final ExifData structure
    let source_file = path.to_string_lossy().to_string();
    let mut exif_data = ExifData::new(source_file, env!("CARGO_PKG_VERSION").to_string());

    // Set tag entries (new API)
    exif_data.tags = tag_entries;

    // Set legacy tags for backward compatibility
    exif_data.legacy_tags = tags;

    // Set missing implementations if requested
    exif_data.missing_implementations = missing_implementations;

    Ok(exif_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_metadata_nonexistent_file() {
        let path = Path::new("nonexistent_file.jpg");
        let result = extract_metadata(path, false);
        assert!(result.is_err());
        // Should be an IO error for file not found
        assert!(result.is_err());
    }
}
