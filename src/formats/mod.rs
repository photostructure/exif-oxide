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
pub use jpeg::{
    extract_jpeg_exif, extract_jpeg_xmp, scan_jpeg_segments, JpegSegment, JpegSegmentInfo,
};
pub use tiff::{extract_tiff_exif, extract_tiff_xmp, get_tiff_endianness, validate_tiff_format};

use crate::exif::ExifReader;
use crate::file_detection::FileTypeDetector;
use crate::generated::{EXIF_MAIN_TAGS, REQUIRED_PRINT_CONV, REQUIRED_VALUE_CONV};
use crate::types::{ExifData, Result, TagEntry, TagValue};
use crate::xmp::XmpProcessor;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

/// Extract metadata from a file (Milestone 1: real file I/O with JPEG detection)
///
/// This function now implements real file reading and JPEG segment scanning.
/// It detects JPEG files by magic bytes and locates EXIF data in APP1 segments.
pub fn extract_metadata(path: &Path, show_missing: bool, show_warnings: bool) -> Result<ExifData> {
    // Ensure conversions are registered
    crate::init();

    // Open file with buffered reading for performance
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // Detect file type using the new ExifTool-compatible detector
    let detector = FileTypeDetector::new();
    let detection_result = detector.detect_file_type(path, &mut reader)?;

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
        print: TagValue::String(filename),
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
        print: TagValue::String(directory),
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
        print: TagValue::string(file_size.to_string()),
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
            print: TagValue::String(formatted),
        });
    }

    // Add FileType and FileTypeExtension using ExifTool-compatible values
    // Note: We'll store the initial file type here, but it may be overridden later
    // (e.g., NEF -> NRW during TIFF processing)
    let mut file_type = detection_result.file_type.clone();
    tag_entries.push(TagEntry {
        group: "File".to_string(),
        group1: "File".to_string(),
        name: "FileType".to_string(),
        value: TagValue::String(file_type.clone()),
        print: TagValue::String(file_type.clone()),
    });

    // FileTypeExtension follows ExifTool's logic exactly
    // Source: ExifTool.pm:9583 - $self->FoundTag('FileTypeExtension', uc $normExt);
    // Raw value: uppercase, PrintConv: lowercase (PrintConv => 'lc $val')
    let (file_type_ext_raw, file_type_ext_print) = {
        use crate::generated::ExifTool_pm::filetypeext::lookup_file_type_extensions;

        // First check ExifTool's %fileTypeExt mapping for special cases
        let norm_ext = lookup_file_type_extensions(&detection_result.file_type)
            .unwrap_or(&detection_result.file_type); // Default to file_type if no mapping

        // ExifTool applies uc() (uppercase) to the raw value
        let raw_value = norm_ext.to_uppercase();

        // ExifTool applies PrintConv => 'lc $val' (lowercase) for display
        let print_value = norm_ext.to_lowercase();

        (raw_value, print_value)
    };

    tag_entries.push(TagEntry {
        group: "File".to_string(),
        group1: "File".to_string(),
        name: "FileTypeExtension".to_string(),
        value: TagValue::String(file_type_ext_raw),
        print: TagValue::String(file_type_ext_print),
    });

    let mime_type = detection_result.mime_type.clone();
    tag_entries.push(TagEntry {
        group: "File".to_string(),
        group1: "File".to_string(),
        name: "MIMEType".to_string(),
        value: TagValue::String(mime_type.clone()),
        print: TagValue::String(mime_type),
    });

    // Format-specific processing based on the detected format
    match detection_result.format.as_str() {
        "RAW" => {
            // RAW format processing (Milestone 17a: Kyocera RAW support)
            // Reset reader to start of file
            reader.seek(SeekFrom::Start(0))?;

            // Read entire file for RAW processing
            let mut raw_data = Vec::new();
            reader.read_to_end(&mut raw_data)?;

            // Process RAW data using RAW processor
            let raw_processor = crate::raw::RawProcessor::new();
            let mut exif_reader = ExifReader::new();

            // Store the original file type for format detection
            exif_reader.set_file_type(detection_result.file_type.clone());

            match raw_processor.process_raw(&mut exif_reader, &raw_data, &detection_result) {
                Ok(()) => {
                    // Successfully processed RAW - extract all found tags using new TagEntry API
                    let mut raw_tag_entries = exif_reader.get_all_tag_entries();

                    // Append RAW tag entries to our collection
                    tag_entries.append(&mut raw_tag_entries);

                    // Also populate legacy tags for backward compatibility
                    let raw_tags = exif_reader.get_all_tags();
                    for (key, value) in raw_tags {
                        tags.insert(key, value);
                    }

                    // Add RAW processing warnings as tags for debugging
                    if show_warnings {
                        for (i, warning) in exif_reader.get_warnings().iter().enumerate() {
                            tags.insert(
                                format!("Warning:RawWarning{i}"),
                                TagValue::String(warning.clone()),
                            );
                        }
                    }
                }
                Err(e) => {
                    // Failed to parse RAW - add error information
                    tags.insert(
                        "Warning:RawParseError".to_string(),
                        TagValue::string(format!("Failed to parse RAW: {e}")),
                    );
                }
            }
        }
        "JPEG" => {
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
                            if show_warnings {
                                for (i, warning) in exif_reader.get_warnings().iter().enumerate() {
                                    tags.insert(
                                        format!("Warning:ExifWarning{i}"),
                                        TagValue::String(warning.clone()),
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            // Failed to parse EXIF - add error information
                            tags.insert(
                                "Warning:ExifParseError".to_string(),
                                TagValue::string(format!("Failed to parse EXIF: {e}")),
                            );
                        }
                    }
                }
                None => {
                    // No EXIF data found
                    tags.insert(
                        "System:ExifDetectionStatus".to_string(),
                        "No EXIF data found in JPEG".into(),
                    );
                }
            }

            // Extract XMP data (handles both regular and Extended XMP)
            reader.seek(SeekFrom::Start(0))?;
            match extract_jpeg_xmp(&mut reader) {
                Ok(xmp_data) => {
                    // Process XMP data with XmpProcessor
                    let mut xmp_processor = XmpProcessor::new();
                    match xmp_processor.process_xmp_data(&xmp_data) {
                        Ok(xmp_entry) => {
                            // Add structured XMP TagEntry
                            tag_entries.push(xmp_entry);

                            // Add XMP detection status
                            tags.insert(
                                "System:XmpDetectionStatus".to_string(),
                                TagValue::String(format!(
                                    "XMP data found ({} bytes total)",
                                    xmp_data.len()
                                )),
                            );
                        }
                        Err(e) => {
                            // Failed to parse XMP - add error information
                            tags.insert(
                                "Warning:XmpParseError".to_string(),
                                TagValue::string(format!("Failed to parse XMP: {e}")),
                            );
                        }
                    }
                }
                Err(e) if e.to_string().contains("No XMP data found") => {
                    // No XMP data found (not an error)
                    tags.insert(
                        "System:XmpDetectionStatus".to_string(),
                        "No XMP data found in JPEG".into(),
                    );
                }
                Err(e) => {
                    // Real error scanning for XMP
                    tags.insert(
                        "Warning:XmpScanError".to_string(),
                        TagValue::string(format!("Error scanning for XMP: {e}")),
                    );
                }
            }
        }
        "TIFF" | "ORF" => {
            // For TIFF-based files (including NEF, NRW, CR2, ORF, etc.), process as TIFF
            // Reset reader to start of file
            reader.seek(SeekFrom::Start(0))?;

            // Read entire file for TIFF processing
            let mut tiff_data = Vec::new();
            reader.read_to_end(&mut tiff_data)?;

            // Parse TIFF/EXIF data
            let mut exif_reader = ExifReader::new();

            // Store the original file type for NEF/NRW detection
            exif_reader.set_file_type(detection_result.file_type.clone());

            match exif_reader.parse_exif_data(&tiff_data) {
                Ok(()) => {
                    // Check if file type was overridden during processing
                    if let Some(new_file_type) = exif_reader.get_overridden_file_type() {
                        // Update file type tags with the overridden value
                        file_type = new_file_type.clone();

                        // Find and update the FileType tag entry
                        for entry in &mut tag_entries {
                            if entry.name == "FileType" {
                                entry.value = TagValue::String(file_type.clone());
                                entry.print = TagValue::String(file_type.clone());
                            } else if entry.name == "FileTypeExtension" {
                                entry.value = TagValue::String(file_type.to_lowercase());
                                entry.print = TagValue::String(file_type.to_lowercase());
                            } else if entry.name == "MIMEType" {
                                // Update MIME type for NRW
                                if file_type == "NRW" {
                                    entry.value = "image/x-nikon-nrw".into();
                                    entry.print = "image/x-nikon-nrw".into();
                                }
                            }
                        }
                    }

                    // Extract all found tags using new TagEntry API
                    let mut exif_tag_entries = exif_reader.get_all_tag_entries();

                    // Append EXIF tag entries to our collection
                    tag_entries.append(&mut exif_tag_entries);

                    // Also populate legacy tags for backward compatibility
                    let exif_tags = exif_reader.get_all_tags();
                    for (key, value) in exif_tags {
                        tags.insert(key, value);
                    }

                    // Add EXIF processing warnings as tags for debugging
                    if show_warnings {
                        for (i, warning) in exif_reader.get_warnings().iter().enumerate() {
                            tags.insert(
                                format!("Warning:ExifWarning{i}"),
                                TagValue::String(warning.clone()),
                            );
                        }
                    }
                }
                Err(e) => {
                    // Failed to parse TIFF - add error information
                    tags.insert(
                        "Warning:TiffParseError".to_string(),
                        TagValue::string(format!("Failed to parse TIFF: {e}")),
                    );
                }
            }

            // Check for XMP data in TIFF IFD0
            match extract_tiff_xmp(&tiff_data) {
                Ok(Some(xmp_data)) => {
                    // Process XMP data with XmpProcessor
                    let mut xmp_processor = XmpProcessor::new();
                    match xmp_processor.process_xmp_data(&xmp_data) {
                        Ok(xmp_entry) => {
                            // Add structured XMP TagEntry
                            tag_entries.push(xmp_entry);

                            // Add XMP detection status
                            tags.insert(
                                "System:XmpDetectionStatus".to_string(),
                                TagValue::String(format!(
                                    "XMP data found in TIFF IFD0 tag 0x02bc, length {} bytes",
                                    xmp_data.len()
                                )),
                            );
                        }
                        Err(e) => {
                            // Failed to parse XMP - add error information
                            tags.insert(
                                "Warning:XmpParseError".to_string(),
                                TagValue::string(format!("Failed to parse XMP: {e}")),
                            );
                        }
                    }
                }
                Ok(None) => {
                    // No XMP data found
                    tags.insert(
                        "System:XmpDetectionStatus".to_string(),
                        "No XMP data found in TIFF".into(),
                    );
                }
                Err(e) => {
                    // Error extracting XMP
                    tags.insert(
                        "Warning:XmpExtractionError".to_string(),
                        TagValue::string(format!("Error extracting XMP: {e}")),
                    );
                }
            }
        }
        "XMP" => {
            // Standalone XMP file processing
            reader.seek(SeekFrom::Start(0))?;
            let mut xmp_data = Vec::new();
            reader.read_to_end(&mut xmp_data)?;

            // Process XMP data with XmpProcessor
            let mut xmp_processor = XmpProcessor::new();
            match xmp_processor.process_xmp_data(&xmp_data) {
                Ok(xmp_entry) => {
                    // Add structured XMP TagEntry
                    tag_entries.push(xmp_entry);

                    // Add XMP detection status
                    tags.insert(
                        "System:XmpDetectionStatus".to_string(),
                        TagValue::String(format!(
                            "XMP data processed from standalone file, length {} bytes",
                            xmp_data.len()
                        )),
                    );
                }
                Err(e) => {
                    // Failed to parse XMP - add error information
                    tags.insert(
                        "Warning:XmpParseError".to_string(),
                        TagValue::string(format!("Failed to parse XMP: {e}")),
                    );
                }
            }
        }
        "MRW" | "RW2" | "RWL" => {
            // RAW format processing (Milestone 17b: Minolta MRW and Panasonic RW2 support)
            // Reset reader to start of file
            reader.seek(SeekFrom::Start(0))?;
            // Read entire file for RAW processing
            let mut raw_data = Vec::new();
            reader.read_to_end(&mut raw_data)?;
            // Process RAW data using RAW processor
            let raw_processor = crate::raw::RawProcessor::new();
            let mut exif_reader = ExifReader::new();
            // Store the original file type for format detection
            exif_reader.set_file_type(detection_result.file_type.clone());
            match raw_processor.process_raw(&mut exif_reader, &raw_data, &detection_result) {
                Ok(()) => {
                    // Successfully processed RAW - extract all found tags using new TagEntry API
                    let mut raw_tag_entries = exif_reader.get_all_tag_entries();
                    // Append RAW tag entries to our collection
                    tag_entries.append(&mut raw_tag_entries);
                    // Also populate legacy tags for backward compatibility
                    let raw_tags = exif_reader.get_all_tags();
                    for (key, value) in raw_tags {
                        tags.insert(key, value);
                    }
                    // Add RAW processing warnings as tags for debugging
                    if show_warnings {
                        for (i, warning) in exif_reader.get_warnings().iter().enumerate() {
                            tags.insert(
                                format!("Warning:RawWarning{i}"),
                                TagValue::String(warning.clone()),
                            );
                        }
                    }
                }
                Err(e) => {
                    // Failed to parse RAW - add error information
                    tags.insert(
                        "Warning:RawParseError".to_string(),
                        TagValue::string(format!(
                            "Failed to parse {} RAW: {e}",
                            detection_result.format
                        )),
                    );
                }
            }
        }
        _ => {
            // Other formats not yet supported
            tags.insert(
                "System:ExifDetectionStatus".to_string(),
                TagValue::string(format!(
                    "Format {} not yet supported for EXIF extraction",
                    detection_result.format
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
        let result = extract_metadata(path, false, false);
        assert!(result.is_err());
        // Should be an IO error for file not found
        assert!(result.is_err());
    }
}
