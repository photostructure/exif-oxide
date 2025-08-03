//! File format detection and processing
//!
//! This module handles different image file formats and extracts
//! metadata from each according to format-specific requirements.

mod avif;
mod detection;
mod gif;
mod iptc;
mod jpeg;
mod png;
mod tiff;

pub use avif::{create_avif_tag_entries, extract_avif_dimensions, AvifImageProperties};
pub use detection::{
    detect_file_format, detect_file_format_from_path, get_format_properties, FileFormat,
};
pub use gif::{create_gif_tag_entries, parse_gif_screen_descriptor, ScreenDescriptor};
pub use iptc::{parse_iptc_from_app13, parse_iptc_metadata};
pub use jpeg::{
    extract_jpeg_exif, extract_jpeg_iptc, extract_jpeg_xmp, scan_jpeg_segments, JpegSegment,
    JpegSegmentInfo, SofData,
};
pub use tiff::{extract_tiff_exif, extract_tiff_xmp, get_tiff_endianness, validate_tiff_format};

use crate::exif::ExifReader;
use crate::file_detection::FileTypeDetector;
use crate::types::{ExifData, FilterOptions, Result, TagEntry, TagValue};
use crate::xmp::XmpProcessor;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use tracing::debug;

/// Extract metadata from a file with optional tag filtering and value formatting control
///
/// This function implements real file reading and format-specific parsing with support for:
/// - Tag filtering (specific tags, groups, -all patterns)
/// - Performance optimization (early return for File-only requests)
/// - Value formatting control (PrintConv vs ValueConv with # suffix)
///
/// # Arguments
/// * `path` - Path to the image file
/// * `show_missing` - Include missing implementation information in output
/// * `show_warnings` - Include parsing warnings in output  
/// * `filter_options` - Optional filtering and formatting configuration (None = extract all tags)
///
/// # Examples
/// ```no_run
/// use std::path::Path;
/// use exif_oxide::types::FilterOptions;
/// use exif_oxide::formats::extract_metadata;
///
/// // Extract all tags (backward compatibility)
/// let all_tags = extract_metadata(Path::new("image.jpg"), false, false, None).unwrap();
///
/// // Extract only MIMEType (performance optimized)
/// let filter = FilterOptions::tags_only(vec!["MIMEType".to_string()]);
/// let mime_only = extract_metadata(Path::new("image.jpg"), false, false, Some(filter)).unwrap();
/// ```
pub fn extract_metadata(
    path: &Path,
    show_missing: bool,
    show_warnings: bool,
    filter_options: Option<FilterOptions>,
) -> Result<ExifData> {
    // Ensure conversions are registered
    crate::init();

    // Use default filter options if none provided (backward compatibility)
    let filter_opts = filter_options.clone().unwrap_or_default();

    // PERFORMANCE OPTIMIZATION: Check if only File group tags are requested
    // This allows early return without expensive format-specific parsing
    if filter_opts.is_file_group_only() {
        debug!("File-only request detected - skipping format-specific parsing for performance");
        return extract_file_tags_only(path, show_missing, &filter_opts);
    }

    // Open file with buffered reading for performance
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // Detect file type using the new ExifTool-compatible detector
    let detector = FileTypeDetector::new();
    let detection_result = detector.detect_file_type(path, &mut reader)?;

    // Get actual file metadata
    let file_metadata = std::fs::metadata(path)?;
    let file_size = file_metadata.len();

    let mut tags = IndexMap::new();
    let mut tag_entries = Vec::new();

    // Basic file information (now real data) - create as TagEntry objects with filtering
    if filter_opts.should_extract_tag("FileName", "File") {
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        tag_entries.push(TagEntry {
            group: "File".to_string(),
            group1: "System".to_string(),
            name: "FileName".to_string(),
            value: TagValue::String(filename.clone()),
            print: TagValue::String(filename),
        });
    }

    if filter_opts.should_extract_tag("Directory", "File") {
        let directory = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_string_lossy()
            .to_string();
        tag_entries.push(TagEntry {
            group: "File".to_string(),
            group1: "System".to_string(),
            name: "Directory".to_string(),
            value: TagValue::String(directory.clone()),
            print: TagValue::String(directory),
        });
    }

    // FileSize - return raw bytes as per user requirement
    // Store as string for the numeric value (ExifTool compatibility)
    if filter_opts.should_extract_tag("FileSize", "File") {
        tag_entries.push(TagEntry {
            group: "File".to_string(),
            group1: "System".to_string(),
            name: "FileSize".to_string(),
            value: TagValue::U64(file_size),
            print: TagValue::U64(file_size),
        });
    }

    // Format file modification time to match ExifTool format: "YYYY:MM:DD HH:MM:SS±TZ:TZ"
    // ExifTool.pm formats this as local time with timezone offset
    if filter_opts.should_extract_tag("FileModifyDate", "File") {
        if let Ok(modified) = file_metadata.modified() {
            use chrono::{DateTime, Local};
            let datetime: DateTime<Local> = modified.into();
            // Format to match ExifTool exactly: "2025:06:30 10:16:40-07:00"
            let formatted = datetime.format("%Y:%m:%d %H:%M:%S%:z").to_string();
            tag_entries.push(TagEntry {
                group: "File".to_string(),
                group1: "System".to_string(),
                name: "FileModifyDate".to_string(),
                value: TagValue::String(formatted.clone()),
                print: TagValue::String(formatted),
            });
        }
    }

    // Add FileAccessDate - ExifTool.pm:1427
    if filter_opts.should_extract_tag("FileAccessDate", "File") {
        if let Ok(accessed) = file_metadata.accessed() {
            use chrono::{DateTime, Local};
            let datetime: DateTime<Local> = accessed.into();
            let formatted = datetime.format("%Y:%m:%d %H:%M:%S%:z").to_string();
            tag_entries.push(TagEntry {
                group: "File".to_string(),
                group1: "System".to_string(),
                name: "FileAccessDate".to_string(),
                value: TagValue::String(formatted.clone()),
                print: TagValue::String(formatted),
            });
        }
    }

    // Add FileCreateDate (Windows/macOS) or FileInodeChangeDate (Unix)
    // ExifTool.pm:1437 and 1463
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        if filter_opts.should_extract_tag("FileCreateDate", "File") {
            if let Ok(created) = file_metadata.created() {
                use chrono::{DateTime, Local};
                let datetime: DateTime<Local> = created.into();
                let formatted = datetime.format("%Y:%m:%d %H:%M:%S%:z").to_string();
                tag_entries.push(TagEntry {
                    group: "File".to_string(),
                    group1: "System".to_string(),
                    name: "FileCreateDate".to_string(),
                    value: TagValue::String(formatted.clone()),
                    print: TagValue::String(formatted),
                });
            }
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        // On Unix systems, use ctime as FileInodeChangeDate
        // This represents when the inode was last changed (not creation time)
        // ExifTool.pm:2860-2861 uses stat[10] which is ctime
        if filter_opts.should_extract_tag("FileInodeChangeDate", "File") {
            if let Some(ctime) = get_unix_ctime(path) {
                use chrono::{Local, TimeZone};
                let datetime = Local.timestamp_opt(ctime as i64, 0).single();
                if let Some(datetime) = datetime {
                    let formatted = datetime.format("%Y:%m:%d %H:%M:%S%:z").to_string();
                    tag_entries.push(TagEntry {
                        group: "File".to_string(),
                        group1: "System".to_string(),
                        name: "FileInodeChangeDate".to_string(),
                        value: TagValue::String(formatted.clone()),
                        print: TagValue::String(formatted),
                    });
                }
            }
        }
    }

    // Add FilePermissions - ExifTool.pm:1473-1517
    // Format as Unix permissions string like "-rw-rw-r--"
    #[cfg(unix)]
    {
        if filter_opts.should_extract_tag("FilePermissions", "File") {
            use std::os::unix::fs::PermissionsExt;
            let mode = file_metadata.permissions().mode();
            let permissions_str = format_unix_permissions(mode);
            tag_entries.push(TagEntry {
                group: "File".to_string(),
                group1: "System".to_string(),
                name: "FilePermissions".to_string(),
                value: TagValue::String(permissions_str.clone()),
                print: TagValue::String(permissions_str),
            });
        }
    }

    #[cfg(not(unix))]
    {
        // On non-Unix systems (like Windows), ExifTool shows different attributes
        // For now, we'll skip this as it requires Win32API::File
        // TODO: Implement Windows file attributes when Win32 API is available
    }

    // Add FileType and FileTypeExtension using ExifTool-compatible values
    // Note: We'll store the initial file type here, but it may be overridden later
    // (e.g., NEF -> NRW during TIFF processing)
    let mut file_type = detection_result.file_type.clone();
    if filter_opts.should_extract_tag("FileType", "File") {
        tag_entries.push(TagEntry {
            group: "File".to_string(),
            group1: "File".to_string(),
            name: "FileType".to_string(),
            value: TagValue::String(file_type.clone()),
            print: TagValue::String(file_type.clone()),
        });
    }

    // FileTypeExtension follows ExifTool's logic exactly
    // Source: ExifTool.pm:9583 - $self->FoundTag('FileTypeExtension', uc $normExt);
    // Raw value: uppercase, PrintConv: lowercase (PrintConv => 'lc $val')
    if filter_opts.should_extract_tag("FileTypeExtension", "File") {
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
    }

    if filter_opts.should_extract_tag("MIMEType", "File") {
        let mime_type = detection_result.mime_type.clone();
        tag_entries.push(TagEntry {
            group: "File".to_string(),
            group1: "File".to_string(),
            name: "MIMEType".to_string(),
            value: TagValue::String(mime_type.clone()),
            print: TagValue::String(mime_type),
        });
    }

    // PERFORMANCE OPTIMIZATION: Skip format-specific processing if only File group tags are requested
    // Check if any non-File group tags are needed (EXIF, MakerNotes, etc.)
    let needs_format_processing = !filter_opts.extract_all
        && filter_opts.has_specific_requests()
        && (filter_opts
            .requested_tags
            .iter()
            .any(|tag| !is_file_group_tag(tag))
            || filter_opts
                .requested_groups
                .iter()
                .any(|group| group.to_lowercase() != "file")
            || filter_opts
                .group_all_patterns
                .iter()
                .any(|pattern| !pattern.to_lowercase().starts_with("file:"))
            || filter_opts.glob_patterns.iter().any(|pattern| {
                // Check if glob pattern could match non-File group tags
                let pattern_lower = pattern.to_lowercase();
                !(pattern_lower == "file:*"
                    || pattern_lower.starts_with("file")
                    || matches!(
                        pattern_lower.as_str(),
                        "filename*"
                            | "directory*"
                            | "filesize*"
                            | "filemodifydate*"
                            | "fileaccessdate*"
                            | "fileinodechangedate*"
                            | "filecreatedate*"
                            | "filepermissions*"
                            | "filetype*"
                            | "filetypeextension*"
                            | "mimetype*"
                    ))
            }));

    // Only do format-specific processing if needed
    if filter_opts.extract_all || needs_format_processing {
        debug!(
            "Format-specific processing required - extracting {} format data",
            detection_result.format
        );

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

                        // Add ExifByteOrder tag if EXIF data was present
                        add_exif_byte_order_tag(&exif_reader, &mut tag_entries);

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
                // Scan for EXIF data in JPEG segments and extract SOF data
                let (segment_info_opt, sof_data_opt) = scan_jpeg_segments(&mut reader)?;

                // Process SOF data first to add dimension tags
                if let Some(sof) = sof_data_opt {
                    // Add ImageWidth from SOF
                    tag_entries.push(TagEntry {
                        group: "File".to_string(),
                        group1: "File".to_string(),
                        name: "ImageWidth".to_string(),
                        value: TagValue::U16(sof.image_width),
                        print: TagValue::U16(sof.image_width),
                    });

                    // Add ImageHeight from SOF
                    tag_entries.push(TagEntry {
                        group: "File".to_string(),
                        group1: "File".to_string(),
                        name: "ImageHeight".to_string(),
                        value: TagValue::U16(sof.image_height),
                        print: TagValue::U16(sof.image_height),
                    });

                    // Add BitsPerSample from SOF
                    tag_entries.push(TagEntry {
                        group: "File".to_string(),
                        group1: "File".to_string(),
                        name: "BitsPerSample".to_string(),
                        value: TagValue::U16(sof.bits_per_sample as u16),
                        print: TagValue::String(sof.bits_per_sample.to_string()),
                    });

                    // Add ColorComponents from SOF
                    tag_entries.push(TagEntry {
                        group: "File".to_string(),
                        group1: "File".to_string(),
                        name: "ColorComponents".to_string(),
                        value: TagValue::U16(sof.color_components as u16),
                        print: TagValue::String(sof.color_components.to_string()),
                    });

                    // Add YCbCrSubSampling if available
                    if let Some(subsampling) = sof.ycbcr_subsampling {
                        tag_entries.push(TagEntry {
                            group: "File".to_string(),
                            group1: "File".to_string(),
                            name: "YCbCrSubSampling".to_string(),
                            value: TagValue::String(subsampling.clone()),
                            print: TagValue::String(subsampling),
                        });
                    }

                    // Add EncodingProcess
                    // Note: ExifTool uses a PrintConv for this, but for now we'll use the raw value
                    tag_entries.push(TagEntry {
                        group: "File".to_string(),
                        group1: "File".to_string(),
                        name: "EncodingProcess".to_string(),
                        value: TagValue::U16(sof.encoding_process as u16),
                        print: TagValue::String(sof.encoding_process.to_string()),
                    });
                }

                match segment_info_opt {
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

                                // Add ExifByteOrder tag
                                add_exif_byte_order_tag(&exif_reader, &mut tag_entries);

                                // Add EXIF processing warnings as tags for debugging
                                if show_warnings {
                                    for (i, warning) in
                                        exif_reader.get_warnings().iter().enumerate()
                                    {
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

                // Extract IPTC data from APP13 segments (Adobe Photoshop Image Resource Blocks)
                reader.seek(SeekFrom::Start(0))?;
                match extract_jpeg_iptc(&mut reader) {
                    Ok(iptc_tags) => {
                        let iptc_count = iptc_tags.len();
                        if !iptc_tags.is_empty() {
                            // Convert IPTC tags to TagEntry format and add to collection
                            for (tag_name, tag_value) in iptc_tags {
                                tag_entries.push(TagEntry {
                                    group: "IPTC".to_string(),
                                    group1: "IPTC".to_string(),
                                    name: tag_name
                                        .strip_prefix("IPTC:")
                                        .unwrap_or(&tag_name)
                                        .to_string(),
                                    value: tag_value.clone(),
                                    print: tag_value,
                                });
                            }

                            // Add IPTC detection status
                            tags.insert(
                                "System:IptcDetectionStatus".to_string(),
                                TagValue::String(format!(
                                    "IPTC data found in APP13 segment ({} tags extracted)",
                                    iptc_count
                                )),
                            );
                        } else {
                            // No IPTC data found
                            tags.insert(
                                "System:IptcDetectionStatus".to_string(),
                                "No IPTC data found in JPEG".into(),
                            );
                        }
                    }
                    Err(e) if e.to_string().contains("No APP13 segment") => {
                        // No APP13 segment found (not an error)
                        tags.insert(
                            "System:IptcDetectionStatus".to_string(),
                            "No APP13 segment found in JPEG".into(),
                        );
                    }
                    Err(e) => {
                        // Real error scanning for IPTC
                        tags.insert(
                            "Warning:IptcScanError".to_string(),
                            TagValue::string(format!("Error scanning for IPTC: {e}")),
                        );
                    }
                }

                // Extract XMP data (handles both regular and Extended XMP)
                reader.seek(SeekFrom::Start(0))?;
                match extract_jpeg_xmp(&mut reader) {
                    Ok(xmp_data) => {
                        // Process XMP data with XmpProcessor - individual tag extraction
                        let mut xmp_processor = XmpProcessor::new();
                        match xmp_processor.process_xmp_data_individual(&xmp_data) {
                            Ok(xmp_tag_entries) => {
                                // Add individual XMP TagEntry objects ("XMP:TagName" format)
                                tag_entries.extend(xmp_tag_entries);

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
                        // Extract TIFF dimensions for TIFF-based RAW files (ARW, CR2, etc.)
                        // This extracts ImageWidth/ImageHeight from TIFF IFD0 tags 0x0100/0x0101
                        // ExifTool: lib/Image/ExifTool/Exif.pm:460-473 (ImageWidth/ImageHeight tags)
                        if matches!(
                            detection_result.file_type.as_str(),
                            "ARW" | "CR2" | "NEF" | "NRW" | "DNG" | "ORF" | "RW2"
                        ) {
                            if let Err(e) = crate::raw::utils::extract_tiff_dimensions(
                                &mut exif_reader,
                                &tiff_data,
                            ) {
                                // Log error but don't fail processing
                                tracing::debug!("Failed to extract TIFF dimensions: {}", e);
                            }

                            // For RW2 files: Extract JPEG preview dimensions to create File:ImageWidth/ImageHeight tags
                            // This must be done after TIFF processing to access the JpgFromRaw binary data
                            if detection_result.file_type == "RW2" {
                                tracing::debug!("Processing RW2 file: attempting to extract JPEG preview dimensions");
                                if let Some(jpeg_preview_dimensions) =
                                    extract_rw2_jpeg_preview_dimensions(&exif_reader, &tiff_data)
                                {
                                    // Create File:ImageWidth and File:ImageHeight tags from JPEG preview
                                    // ExifTool: File group tags come from embedded JPEG SOF data, not sensor borders
                                    tag_entries.push(TagEntry {
                                        group: "File".to_string(),
                                        group1: "File".to_string(),
                                        name: "ImageWidth".to_string(),
                                        value: TagValue::U16(jpeg_preview_dimensions.0),
                                        print: TagValue::U16(jpeg_preview_dimensions.0),
                                    });
                                    tag_entries.push(TagEntry {
                                        group: "File".to_string(),
                                        group1: "File".to_string(),
                                        name: "ImageHeight".to_string(),
                                        value: TagValue::U16(jpeg_preview_dimensions.1),
                                        print: TagValue::U16(jpeg_preview_dimensions.1),
                                    });
                                    tracing::debug!("Added File:ImageWidth/ImageHeight tags from JPEG preview: {}x{}", 
                                              jpeg_preview_dimensions.0, jpeg_preview_dimensions.1);
                                } else {
                                    tracing::debug!(
                                        "Failed to extract JPEG preview dimensions from RW2 file"
                                    );
                                }
                            }
                        }

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

                        // Add ExifByteOrder tag
                        add_exif_byte_order_tag(&exif_reader, &mut tag_entries);

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
                        // Process XMP data with XmpProcessor - individual tag extraction
                        let mut xmp_processor = XmpProcessor::new();
                        match xmp_processor.process_xmp_data_individual(&xmp_data) {
                            Ok(xmp_tag_entries) => {
                                // Add individual XMP TagEntry objects ("XMP:TagName" format)
                                tag_entries.extend(xmp_tag_entries);

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

                // Process XMP data with XmpProcessor - individual tag extraction
                let mut xmp_processor = XmpProcessor::new();
                match xmp_processor.process_xmp_data_individual(&xmp_data) {
                    Ok(xmp_tag_entries) => {
                        // Add individual XMP TagEntry objects ("XMP:TagName" format)
                        tag_entries.extend(xmp_tag_entries);

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
                tracing::debug!(
                    "Processing RAW file with type: {}",
                    detection_result.file_type
                );
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

                        // For RW2 files: Extract JPEG preview dimensions to create File:ImageWidth/ImageHeight tags
                        // ExifTool creates File group tags from embedded JPEG preview (JpgFromRaw tag)
                        if detection_result.file_type == "RW2" {
                            tracing::debug!(
                            "Processing RW2 file: attempting to extract JPEG preview dimensions"
                        );
                            if let Some(jpeg_preview_dimensions) =
                                extract_rw2_jpeg_preview_dimensions(&exif_reader, &raw_data)
                            {
                                // Create File:ImageWidth and File:ImageHeight tags from JPEG preview
                                // ExifTool: File group tags come from embedded JPEG SOF data, not sensor borders
                                tag_entries.push(TagEntry {
                                    group: "File".to_string(),
                                    group1: "File".to_string(),
                                    name: "ImageWidth".to_string(),
                                    value: TagValue::U16(jpeg_preview_dimensions.0),
                                    print: TagValue::U16(jpeg_preview_dimensions.0),
                                });
                                tag_entries.push(TagEntry {
                                    group: "File".to_string(),
                                    group1: "File".to_string(),
                                    name: "ImageHeight".to_string(),
                                    value: TagValue::U16(jpeg_preview_dimensions.1),
                                    print: TagValue::U16(jpeg_preview_dimensions.1),
                                });
                            }
                        }
                        // Also populate legacy tags for backward compatibility
                        let raw_tags = exif_reader.get_all_tags();
                        for (key, value) in raw_tags {
                            tags.insert(key, value);
                        }

                        // Add ExifByteOrder tag if EXIF data was present
                        add_exif_byte_order_tag(&exif_reader, &mut tag_entries);
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
            "CR2" | "CRW" | "CR3" => {
                // Canon RAW format processing (Milestone 17d: Canon CR2, CRW, CR3 support)
                // CR2 is TIFF-based and will be processed by the TIFF handler through maker notes
                // CRW is a legacy custom format, CR3 is MOV-based

                if detection_result.file_type == "CR2" {
                    // CR2 is TIFF-based, so process it as TIFF
                    // The Canon-specific processing happens through maker notes
                    reader.seek(SeekFrom::Start(0))?;
                    let mut tiff_data = Vec::new();
                    reader.read_to_end(&mut tiff_data)?;

                    let mut exif_reader = ExifReader::new();
                    exif_reader.set_file_type(detection_result.file_type.clone());

                    match exif_reader.parse_exif_data(&tiff_data) {
                        Ok(()) => {
                            // Extract all found tags using new TagEntry API
                            let mut exif_tag_entries = exif_reader.get_all_tag_entries();
                            tag_entries.append(&mut exif_tag_entries);

                            // Also populate legacy tags for backward compatibility
                            let exif_tags = exif_reader.get_all_tags();
                            for (key, value) in exif_tags {
                                tags.insert(key, value);
                            }

                            // Add ExifByteOrder tag
                            add_exif_byte_order_tag(&exif_reader, &mut tag_entries);

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
                            tags.insert(
                                "Warning:CR2ParseError".to_string(),
                                TagValue::string(format!("Failed to parse CR2: {e}")),
                            );
                        }
                    }
                } else {
                    // CRW and CR3 are non-TIFF formats - use RAW processor
                    reader.seek(SeekFrom::Start(0))?;
                    let mut raw_data = Vec::new();
                    reader.read_to_end(&mut raw_data)?;

                    let raw_processor = crate::raw::RawProcessor::new();
                    let mut exif_reader = ExifReader::new();
                    exif_reader.set_file_type(detection_result.file_type.clone());

                    match raw_processor.process_raw(&mut exif_reader, &raw_data, &detection_result)
                    {
                        Ok(()) => {
                            let mut raw_tag_entries = exif_reader.get_all_tag_entries();
                            tag_entries.append(&mut raw_tag_entries);

                            let raw_tags = exif_reader.get_all_tags();
                            for (key, value) in raw_tags {
                                tags.insert(key, value);
                            }

                            // Add ExifByteOrder tag if EXIF data was present
                            add_exif_byte_order_tag(&exif_reader, &mut tag_entries);

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
                            tags.insert(
                                "Warning:CanonRawParseError".to_string(),
                                TagValue::string(format!(
                                    "Failed to parse Canon {} RAW: {e}",
                                    detection_result.file_type
                                )),
                            );
                        }
                    }
                }
            }
            "PNG" => {
                // PNG format processing - extract dimensions from IHDR chunk
                // Reset reader to start of file
                reader.seek(SeekFrom::Start(0))?;

                // Read entire PNG file for IHDR processing
                let mut png_data = Vec::new();
                reader.read_to_end(&mut png_data)?;

                // Parse PNG IHDR chunk to extract dimensions and metadata
                match png::parse_png_ihdr(&png_data) {
                    Ok(ihdr) => {
                        // Create PNG tag entries using ExifTool-compatible structure
                        // PNG tags are assigned to "PNG" group (not "File" group like JPEG)
                        let mut png_tag_entries = png::create_png_tag_entries(&ihdr);

                        // Append PNG tag entries to our collection
                        tag_entries.append(&mut png_tag_entries);

                        // Add PNG processing status
                        tags.insert(
                            "System:PngDetectionStatus".to_string(),
                            TagValue::String(format!(
                                "PNG IHDR processed: {}x{} {} {}",
                                ihdr.width,
                                ihdr.height,
                                ihdr.color_type_description(),
                                ihdr.bit_depth
                            )),
                        );
                    }
                    Err(e) => {
                        // Failed to parse PNG IHDR - add error information
                        tags.insert(
                            "Warning:PngParseError".to_string(),
                            TagValue::string(format!("Failed to parse PNG IHDR: {e}")),
                        );
                    }
                }
            }
            "GIF" => {
                // GIF format processing - extract dimensions from Logical Screen Descriptor
                // Reset reader to start of file
                reader.seek(SeekFrom::Start(0))?;

                // Read entire GIF file for screen descriptor processing
                let mut gif_data = Vec::new();
                reader.read_to_end(&mut gif_data)?;

                // Parse GIF Logical Screen Descriptor to extract dimensions and metadata
                match gif::parse_gif_screen_descriptor(&gif_data) {
                    Ok(screen_desc) => {
                        // Create GIF tag entries using ExifTool-compatible structure
                        // GIF tags are assigned to "GIF" group (following ExifTool.pm:GIF.pm)
                        let mut gif_tag_entries = gif::create_gif_tag_entries(&screen_desc);

                        // Append GIF tag entries to our collection
                        tag_entries.append(&mut gif_tag_entries);

                        // Add GIF processing status
                        tags.insert(
                            "System:GifDetectionStatus".to_string(),
                            TagValue::String(format!(
                                "GIF screen descriptor processed: {}x{} {} colors",
                                screen_desc.image_width,
                                screen_desc.image_height,
                                if screen_desc.has_color_map() {
                                    format!("{}", 2_u32.pow(screen_desc.bits_per_pixel() as u32))
                                } else {
                                    "no color map".to_string()
                                }
                            )),
                        );
                    }
                    Err(e) => {
                        // Failed to parse GIF screen descriptor - add error information
                        tags.insert(
                            "Warning:GifParseError".to_string(),
                            TagValue::string(format!("Failed to parse GIF screen descriptor: {e}")),
                        );
                    }
                }
            }
            "MOV" => {
                // ISO Base Media File Format processing (QuickTime, MP4, AVIF, HEIF, etc.)
                // Check file type to determine specific processing
                match detection_result.file_type.as_str() {
                    "AVIF" => {
                        // AVIF format processing - extract dimensions from ispe box
                        // Reset reader to start of file
                        reader.seek(SeekFrom::Start(0))?;

                        // Read entire AVIF file for ISO box processing
                        let mut avif_data = Vec::new();
                        reader.read_to_end(&mut avif_data)?;

                        // Extract AVIF dimensions from ispe box following ExifTool's implementation
                        // Reference: QuickTime.pm:2946-2959 (ispe box processing)
                        match avif::extract_avif_dimensions(&avif_data) {
                            Ok(props) => {
                                // Create AVIF tag entries using ExifTool-compatible structure
                                // AVIF image dimensions are assigned to "File" group in ExifTool
                                let mut avif_tag_entries = avif::create_avif_tag_entries(&props);

                                // Append AVIF tag entries to our collection
                                tag_entries.append(&mut avif_tag_entries);

                                // Add AVIF processing status
                                tags.insert(
                                    "System:AvifDetectionStatus".to_string(),
                                    TagValue::String(format!(
                                        "AVIF ispe box processed: {}x{}",
                                        props.width, props.height
                                    )),
                                );
                            }
                            Err(e) => {
                                // Failed to parse AVIF ispe box - add error information
                                tags.insert(
                                    "Warning:AvifParseError".to_string(),
                                    TagValue::string(format!("Failed to parse AVIF ispe box: {e}")),
                                );
                            }
                        }
                    }
                    "HEIC" | "HEIF" => {
                        // HEIC/HEIF format processing - extract dimensions from ispe box
                        // HEIC and HEIF use the same ISO Base Media File Format and ispe box structure as AVIF
                        // Reference: QuickTime.pm:2946-2959 (ispe box processing)
                        reader.seek(SeekFrom::Start(0))?;

                        // Read entire HEIC/HEIF file for ISO box processing
                        let mut file_data = Vec::new();
                        reader.read_to_end(&mut file_data)?;

                        // Extract dimensions using ExifTool's primary item detection logic
                        // Implements complete pitm/iinf/ipma box processing to identify the main image
                        // 1. Parse pitm box to get primary item ID (QuickTime.pm:3550-3557)
                        // 2. Parse iinf box to build item information map (QuickTime.pm:3730-3740)
                        // 3. Parse ipma box to associate items with properties (QuickTime.pm:10320-10380)
                        // 4. Use DOC_NUM logic to determine primary vs sub-document ispe boxes (QuickTime.pm:6450-6460)
                        //
                        // ExifTool reference: QuickTime.pm:2946-2959 (ispe processing with DOC_NUM check)
                        match avif::extract_heic_dimensions_primary_item(&file_data) {
                            Ok(props) => {
                                // Create File:ImageWidth and File:ImageHeight tag entries
                                // ExifTool creates File group tags for HEIC/HEIF dimensions
                                let file_type = &detection_result.file_type;
                                let mut heic_tag_entries = vec![
                                    TagEntry {
                                        group: "File".to_string(),
                                        group1: "File".to_string(),
                                        name: "ImageWidth".to_string(),
                                        value: TagValue::U32(props.width),
                                        print: TagValue::U32(props.width),
                                    },
                                    TagEntry {
                                        group: "File".to_string(),
                                        group1: "File".to_string(),
                                        name: "ImageHeight".to_string(),
                                        value: TagValue::U32(props.height),
                                        print: TagValue::U32(props.height),
                                    },
                                ];

                                // Append HEIC/HEIF tag entries to our collection
                                tag_entries.append(&mut heic_tag_entries);

                                // Add HEIC/HEIF processing status
                                tags.insert(
                                    format!("System:{}DetectionStatus", file_type),
                                    TagValue::String(format!(
                                        "{} ispe box processed: {}x{}",
                                        file_type, props.width, props.height
                                    )),
                                );
                            }
                            Err(e) => {
                                // Failed to parse HEIC/HEIF ispe box - add error information
                                tags.insert(
                                    format!("Warning:{}ParseError", detection_result.file_type),
                                    TagValue::string(format!(
                                        "Failed to parse {} ispe box: {e}",
                                        detection_result.file_type
                                    )),
                                );
                            }
                        }
                    }
                    _ => {
                        // Other MOV-based formats not yet supported (MP4, QuickTime, HEIF, etc.)
                        tags.insert(
                            "System:ExifDetectionStatus".to_string(),
                            TagValue::string(format!(
                                "MOV-based format {} not yet supported for dimension extraction",
                                detection_result.file_type
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
    } else {
        debug!("Skipping format-specific processing - only File group tags requested");
    }

    // Collect any missing PrintConv/ValueConv implementations for --show-missing functionality
    let missing_implementations = if show_missing {
        let missing_convs = crate::implementations::missing::get_missing_conversions();

        if missing_convs.is_empty() {
            None
        } else {
            // Group by expression and collect tag information
            use std::collections::HashMap;
            let mut grouped: HashMap<String, Vec<(u32, String, String)>> = HashMap::new();

            for miss in &missing_convs {
                grouped.entry(miss.expression.clone()).or_default().push((
                    miss.tag_id,
                    miss.tag_name.clone(),
                    miss.group.clone(),
                ));
            }

            // Format as strings
            let mut missing_strs = Vec::new();
            for (expr, tag_info) in grouped {
                let conv_type = if missing_convs.iter().any(|m| {
                    m.expression == expr
                        && matches!(
                            m.conv_type,
                            crate::implementations::missing::ConversionType::PrintConv
                        )
                }) {
                    "PrintConv"
                } else {
                    "ValueConv"
                };

                let tags_str = tag_info
                    .iter()
                    .map(|(_, tag_name, group)| format!("{}:{}", group, tag_name))
                    .collect::<Vec<_>>()
                    .join(", ");

                missing_strs.push(format!(
                    "{}: {} [used by tags: {}]",
                    conv_type, expr, tags_str
                ));
            }

            Some(missing_strs)
        }
    } else {
        None
    };

    // Create final ExifData structure
    let source_file = path.to_string_lossy().to_string();
    // P12: Only include ExifToolVersion when not filtering (matches ExifTool behavior)
    let version = if filter_options.as_ref().is_none_or(|f| f.extract_all) {
        env!("CARGO_PKG_VERSION").to_string()
    } else {
        String::new() // Empty version when filtering
    };
    let mut exif_data = ExifData::new(source_file, version);

    // ⚠️ CRITICAL ARCHITECTURE: Build composite tags after all tags (including File group tags) are available
    // This ensures File:ImageWidth/ImageHeight are available for Composite:ImageSize dependency resolution
    // DO NOT move this back to EXIF processing - it will break composite tag dependency resolution
    // See: docs/todo/P10a-exif-required-tags.md "Composite:ImageSize Architecture Fix"
    let composite_tags = build_composite_tags_from_entries(&tag_entries);

    // Add composite tags to the tag_entries collection
    let mut all_tag_entries = tag_entries;
    all_tag_entries.extend(composite_tags);

    // Apply XMP/EXIF precedence rules following ExifTool's Priority system
    // This must happen after all tag extraction but before filtering
    all_tag_entries = apply_exiftool_precedence_rules(all_tag_entries);

    // P12: CENTRAL FILTERING CHOKEPOINT - Apply ExifTool-style filtering
    // Matches ExifTool's FoundTag architecture: all tags go through single filtering point
    let filtered_tag_entries = if let Some(filter_opts) = &filter_options {
        if filter_opts.extract_all {
            // No filtering requested - return all tags
            all_tag_entries
        } else {
            // Apply allowlist filtering like ExifTool's REQ_TAG_LOOKUP + IgnoreTags={all => 1}
            all_tag_entries
                .into_iter()
                .filter_map(|mut tag_entry| {
                    if filter_opts.should_extract_tag(&tag_entry.name, &tag_entry.group) {
                        // P12: Apply numeric value control (# suffix) - use ValueConv instead of PrintConv
                        if filter_opts.should_use_numeric(&tag_entry.name) {
                            tag_entry.print = tag_entry.value.clone();
                        }
                        Some(tag_entry)
                    } else {
                        None
                    }
                })
                .collect()
        }
    } else {
        // No filter options - return all tags (backward compatibility)
        all_tag_entries
    };

    // Set tag entries (new API)
    exif_data.tags = filtered_tag_entries;

    // P12: Filter legacy tags using same logic as tag_entries
    let filtered_legacy_tags = if let Some(filter_opts) = &filter_options {
        if filter_opts.extract_all {
            tags
        } else {
            // Apply same filtering to legacy tags
            tags.into_iter()
                .filter(|(tag_key, _)| {
                    // Parse "Group:TagName" format from legacy keys
                    if let Some((group, tag_name)) = tag_key.split_once(':') {
                        filter_opts.should_extract_tag(tag_name, group)
                    } else {
                        // Tags without group prefix (like "SourceFile") - keep for compatibility
                        true
                    }
                })
                .collect()
        }
    } else {
        tags
    };

    // Set legacy tags for backward compatibility
    exif_data.legacy_tags = filtered_legacy_tags;

    // Set missing implementations if requested
    exif_data.missing_implementations = missing_implementations;

    Ok(exif_data)
}

/// Add ExifByteOrder tag based on TIFF header information
/// ExifTool.pm:1795-1805 - ExifByteOrder tag
fn add_exif_byte_order_tag(exif_reader: &ExifReader, tag_entries: &mut Vec<TagEntry>) {
    if let Some(header) = &exif_reader.header {
        use crate::tiff_types::ByteOrder;

        let byte_order_str = match header.byte_order {
            ByteOrder::LittleEndian => "Little-endian (Intel, II)",
            ByteOrder::BigEndian => "Big-endian (Motorola, MM)",
        };

        tag_entries.push(TagEntry {
            group: "File".to_string(),
            group1: "File".to_string(),
            name: "ExifByteOrder".to_string(),
            value: TagValue::String(byte_order_str.to_string()),
            print: TagValue::String(byte_order_str.to_string()),
        });
    }
}

/// Format Unix file permissions to match ExifTool's format
/// ExifTool.pm:1486-1517 - Converts octal mode to rwx string
#[cfg(unix)]
fn format_unix_permissions(mode: u32) -> String {
    let file_type = match mode & 0o170000 {
        0o010000 => 'p', // FIFO
        0o020000 => 'c', // character special file
        0o040000 => 'd', // directory
        0o060000 => 'b', // block special file
        0o120000 => 'l', // sym link
        0o140000 => 's', // socket link
        _ => '-',        // regular file
    };

    let mut permissions = String::with_capacity(10);
    permissions.push(file_type);

    // Owner permissions
    permissions.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    permissions.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    permissions.push(if mode & 0o100 != 0 { 'x' } else { '-' });

    // Group permissions
    permissions.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    permissions.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    permissions.push(if mode & 0o010 != 0 { 'x' } else { '-' });

    // Other permissions
    permissions.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    permissions.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    permissions.push(if mode & 0o001 != 0 { 'x' } else { '-' });

    permissions
}

/// Extract JPEG preview dimensions from RW2 file's JpgFromRaw tag
/// ExifTool: PanasonicRaw.pm tag 0x002e contains embedded JPEG with preview dimensions
fn extract_rw2_jpeg_preview_dimensions(
    exif_reader: &crate::exif::ExifReader,
    raw_data: &[u8],
) -> Option<(u16, u16)> {
    use crate::types::TagValue;
    use std::io::Cursor;
    use tracing::debug;

    debug!("extract_rw2_jpeg_preview_dimensions: Looking for JpgFromRaw tag (0x002e)");

    // Get JpgFromRaw tag (0x002e) offset from ExifReader
    // This tag contains the offset to embedded JPEG data
    let jpeg_offset = match exif_reader.get_tag_across_namespaces(0x002e) {
        Some(TagValue::Binary(binary_data)) => {
            // The binary data should contain the JPEG data directly
            // For RW2 files, ExifTool processes this as raw JPEG data
            debug!("Found JpgFromRaw binary data: {} bytes", binary_data.len());
            if let Ok((_segment_info, sof_data)) =
                crate::formats::jpeg::scan_jpeg_segments(&mut Cursor::new(binary_data))
            {
                if let Some(sof) = sof_data {
                    debug!(
                        "Found JPEG SOF dimensions: {}x{}",
                        sof.image_width, sof.image_height
                    );
                    return Some((sof.image_width, sof.image_height));
                } else {
                    debug!("No SOF data found in JPEG");
                }
            } else {
                debug!("Failed to parse JPEG segments");
            }
            return None;
        }
        Some(TagValue::U32(offset)) => *offset as usize,
        Some(TagValue::U64(offset)) => *offset as usize,
        _ => return None,
    };

    // If we have an offset, read JPEG data from that position in the file
    if jpeg_offset >= raw_data.len() {
        return None;
    }

    // The JPEG data starts at the offset - scan for JPEG segments
    let jpeg_data = &raw_data[jpeg_offset..];

    // Parse JPEG segments to find SOF data with dimensions
    if let Ok((_segment_info, Some(sof))) =
        crate::formats::jpeg::scan_jpeg_segments(&mut Cursor::new(jpeg_data))
    {
        Some((sof.image_width, sof.image_height))
    } else {
        None
    }
}

/// Build composite tags from a collection of TagEntry objects
/// This enables composite tag processing after all tags (including File group tags) are available
fn build_composite_tags_from_entries(tag_entries: &[TagEntry]) -> Vec<TagEntry> {
    // Convert TagEntry collection to the format expected by composite tag processing
    let mut available_tags = HashMap::new();

    for entry in tag_entries {
        // Add with group prefix (e.g., "File:ImageWidth")
        let prefixed_name = format!("{}:{}", entry.group, entry.name);
        available_tags.insert(prefixed_name, entry.value.clone());

        // Also add without group prefix for broader matching (e.g., "ImageWidth")
        available_tags.insert(entry.name.clone(), entry.value.clone());
    }

    // Delegate to the composite tag processing system
    let computed_composites = crate::composite_tags::resolve_and_compute_composites(available_tags);

    // Convert results back to TagEntry format
    let mut composite_tag_entries = Vec::new();
    for (composite_name, value) in computed_composites {
        // Parse the composite name to extract the tag name (remove "Composite:" prefix if present)
        let tag_name = if composite_name.starts_with("Composite:") {
            composite_name
                .strip_prefix("Composite:")
                .unwrap_or(&composite_name)
        } else {
            &composite_name
        };

        // Apply PrintConv to get human-readable representation
        // For now, assume value and print are the same - PrintConv will be applied during tag processing
        composite_tag_entries.push(TagEntry {
            group: "Composite".to_string(),
            group1: "Composite".to_string(),
            name: tag_name.to_string(),
            value: value.clone(),
            print: value, // PrintConv already applied by composite processing
        });
    }

    composite_tag_entries
}

/// Extract only File group tags for performance optimization
/// This function handles requests for File-only tags without expensive format parsing
fn extract_file_tags_only(
    path: &Path,
    show_missing: bool,
    filter_opts: &FilterOptions,
) -> Result<ExifData> {
    use tracing::debug;

    debug!("Extracting File group tags only for: {}", path.display());

    // Get actual file metadata
    let file_metadata = std::fs::metadata(path)?;
    let file_size = file_metadata.len();

    let mut tag_entries = Vec::new();

    // Basic file information - only extract tags that match the filter
    let filename = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    if filter_opts.should_extract_tag("FileName", "File") {
        tag_entries.push(TagEntry {
            group: "File".to_string(),
            group1: "System".to_string(),
            name: "FileName".to_string(),
            value: TagValue::String(filename.clone()),
            print: TagValue::String(filename),
        });
    }

    let directory = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_string_lossy()
        .to_string();

    if filter_opts.should_extract_tag("Directory", "File") {
        tag_entries.push(TagEntry {
            group: "File".to_string(),
            group1: "System".to_string(),
            name: "Directory".to_string(),
            value: TagValue::String(directory.clone()),
            print: TagValue::String(directory),
        });
    }

    if filter_opts.should_extract_tag("FileSize", "File") {
        tag_entries.push(TagEntry {
            group: "File".to_string(),
            group1: "System".to_string(),
            name: "FileSize".to_string(),
            value: TagValue::U64(file_size),
            print: TagValue::U64(file_size),
        });
    }

    // File timestamps
    if filter_opts.should_extract_tag("FileModifyDate", "File") {
        if let Ok(modified) = file_metadata.modified() {
            use chrono::{DateTime, Local};
            let datetime: DateTime<Local> = modified.into();
            let formatted = datetime.format("%Y:%m:%d %H:%M:%S%:z").to_string();
            tag_entries.push(TagEntry {
                group: "File".to_string(),
                group1: "System".to_string(),
                name: "FileModifyDate".to_string(),
                value: TagValue::String(formatted.clone()),
                print: TagValue::String(formatted),
            });
        }
    }

    if filter_opts.should_extract_tag("FileAccessDate", "File") {
        if let Ok(accessed) = file_metadata.accessed() {
            use chrono::{DateTime, Local};
            let datetime: DateTime<Local> = accessed.into();
            let formatted = datetime.format("%Y:%m:%d %H:%M:%S%:z").to_string();
            tag_entries.push(TagEntry {
                group: "File".to_string(),
                group1: "System".to_string(),
                name: "FileAccessDate".to_string(),
                value: TagValue::String(formatted.clone()),
                print: TagValue::String(formatted),
            });
        }
    }

    // File permissions (Unix only)
    #[cfg(unix)]
    if filter_opts.should_extract_tag("FilePermissions", "File") {
        use std::os::unix::fs::PermissionsExt;
        let mode = file_metadata.permissions().mode();
        let permissions_str = format_unix_permissions(mode);
        tag_entries.push(TagEntry {
            group: "File".to_string(),
            group1: "System".to_string(),
            name: "FilePermissions".to_string(),
            value: TagValue::String(permissions_str.clone()),
            print: TagValue::String(permissions_str),
        });
    }

    // For FileType, FileTypeExtension, and MIMEType, we need basic file detection
    // This is much lighter than full format parsing
    if filter_opts.should_extract_tag("FileType", "File")
        || filter_opts.should_extract_tag("FileTypeExtension", "File")
        || filter_opts.should_extract_tag("MIMEType", "File")
    {
        // We need to open the file briefly for magic number detection
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let detector = FileTypeDetector::new();
        let detection_result = detector.detect_file_type(path, &mut reader)?;

        if filter_opts.should_extract_tag("FileType", "File") {
            let file_type = detection_result.file_type.clone();
            tag_entries.push(TagEntry {
                group: "File".to_string(),
                group1: "File".to_string(),
                name: "FileType".to_string(),
                value: TagValue::String(file_type.clone()),
                print: TagValue::String(file_type),
            });
        }

        if filter_opts.should_extract_tag("FileTypeExtension", "File") {
            use crate::generated::ExifTool_pm::filetypeext::lookup_file_type_extensions;
            let norm_ext = lookup_file_type_extensions(&detection_result.file_type)
                .unwrap_or(&detection_result.file_type);
            let raw_value = norm_ext.to_uppercase();
            let print_value = norm_ext.to_lowercase();

            tag_entries.push(TagEntry {
                group: "File".to_string(),
                group1: "File".to_string(),
                name: "FileTypeExtension".to_string(),
                value: TagValue::String(raw_value),
                print: TagValue::String(print_value),
            });
        }

        if filter_opts.should_extract_tag("MIMEType", "File") {
            let mime_type = detection_result.mime_type.clone();
            tag_entries.push(TagEntry {
                group: "File".to_string(),
                group1: "File".to_string(),
                name: "MIMEType".to_string(),
                value: TagValue::String(mime_type.clone()),
                print: TagValue::String(mime_type),
            });
        }
    }

    // Create final ExifData structure
    let source_file = path.to_string_lossy().to_string();
    let mut exif_data = ExifData::new(source_file, env!("CARGO_PKG_VERSION").to_string());

    // Set tag entries
    exif_data.tags = tag_entries;

    // Handle missing implementations if requested
    if show_missing {
        exif_data.missing_implementations = Some(vec![
            "File-only extraction mode: format-specific parsing skipped for performance"
                .to_string(),
        ]);
    }

    debug!(
        "File-only extraction completed with {} tags",
        exif_data.tags.len()
    );
    Ok(exif_data)
}

/// Check if a tag name belongs to the File group (for performance optimization)
fn is_file_group_tag(tag_name: &str) -> bool {
    matches!(
        tag_name.to_lowercase().as_str(),
        "filename"
            | "directory"
            | "filesize"
            | "filemodifydate"
            | "fileaccessdate"
            | "fileinodechangedate"
            | "filecreatedate"
            | "filepermissions"
            | "filetype"
            | "filetypeextension"
            | "mimetype"
            | "imagewidth"
            | "imageheight"
            | "bitspersample"
            | "colorcomponents"
            | "ycbcrsubsampling"
            | "encodingprocess"
            | "exifbyteorder"
    )
}

/// Get Unix ctime (inode change time) from file path
/// This matches ExifTool.pm:2860-2861 which uses stat[10] for ctime
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn get_unix_ctime(path: &Path) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;

    // Get Unix metadata to access ctime
    match std::fs::metadata(path) {
        Ok(metadata) => {
            // ctime() returns the inode change time (stat[10] in Perl's stat function)
            // This is exactly what ExifTool uses for FileInodeChangeDate on Unix systems
            Some(metadata.ctime() as u64)
        }
        Err(_) => None,
    }
}

/// Apply ExifTool-style precedence rules to resolve conflicts between EXIF and XMP tags
///
/// Following ExifTool's Priority system:
/// - EXIF tags generally have Priority 1 (higher)
/// - XMP tags generally have Priority 0 (lower)
/// - Higher priority tags override lower priority tags
/// - Equal priority: most recent (last extracted) wins
/// - File group tags have highest priority
///
/// This ensures that EXIF:Make beats XMP:Make, EXIF:CreateDate beats XMP:CreateDate, etc.
fn apply_exiftool_precedence_rules(tag_entries: Vec<TagEntry>) -> Vec<TagEntry> {
    use std::collections::HashMap;

    // Group tags by name to find conflicts
    let mut tag_groups: HashMap<String, Vec<TagEntry>> = HashMap::new();
    let mut exempt_tags: Vec<TagEntry> = Vec::new();

    for tag_entry in tag_entries {
        // GPS coordinate tags should not be subject to precedence rules
        // ExifTool shows both EXIF:GPSLatitude and Composite:GPSLatitude
        if is_gps_coordinate_tag(&tag_entry.name) {
            exempt_tags.push(tag_entry);
        } else {
            tag_groups
                .entry(tag_entry.name.clone())
                .or_default()
                .push(tag_entry);
        }
    }

    let mut resolved_tags = Vec::new();

    for (tag_name, mut conflicting_tags) in tag_groups {
        if conflicting_tags.len() == 1 {
            // No conflict - add the single tag
            resolved_tags.extend(conflicting_tags);
        } else {
            // Multiple tags with same name - apply precedence rules

            // Sort by priority (highest first)
            conflicting_tags.sort_by(|a, b| {
                get_tag_priority(&a.group, &a.name)
                    .cmp(&get_tag_priority(&b.group, &b.name))
                    .reverse()
            });

            // Take the highest priority tag as the winner
            if let Some(winner) = conflicting_tags.first() {
                resolved_tags.push(winner.clone());

                // TODO: Optionally add lower priority tags as numbered duplicates (e.g., "Make (1)")
                // This would require checking if duplicates are requested (like ExifTool's -a flag)
                // For now, we follow the default ExifTool behavior of only showing the highest priority tag
            }

            // Debug logging for precedence decisions
            if conflicting_tags.len() > 1 {
                let winner_group = &conflicting_tags[0].group;
                let loser_groups: Vec<String> = conflicting_tags[1..]
                    .iter()
                    .map(|t| t.group.clone())
                    .collect();
                tracing::debug!(
                    "Tag precedence: {} - winner: {}, overridden: {:?}",
                    tag_name,
                    winner_group,
                    loser_groups
                );
            }
        }
    }

    // Add exempt tags (GPS coordinates) that bypass precedence rules
    resolved_tags.extend(exempt_tags);

    resolved_tags
}

/// Check if a tag is a GPS coordinate tag that should bypass precedence rules
/// ExifTool shows both EXIF and Composite versions of GPS coordinate tags
fn is_gps_coordinate_tag(tag_name: &str) -> bool {
    matches!(tag_name, "GPSLatitude" | "GPSLongitude" | "GPSAltitude")
}

/// Get ExifTool-style priority for a tag based on its group
/// Higher numbers = higher priority (will override lower priority tags)
fn get_tag_priority(group: &str, tag_name: &str) -> u8 {
    match group {
        // File group has highest priority (cannot be overridden)
        "File" => 10,

        // EXIF groups have high priority (Priority 1 in ExifTool)
        "EXIF" | "ExifIFD" | "IFD0" | "IFD1" => 5,

        // MakerNotes have high priority
        "MakerNotes" | "Canon" | "Nikon" | "Sony" | "Olympus" | "Panasonic" => 5,

        // Composite tags have variable priority - most are high
        "Composite" => 4,

        // APP segments have moderate priority
        "APP" | "APP1" | "APP13" => 3,

        // XMP has variable priority - some tags are high priority, most are low
        "XMP" => {
            if is_xmp_high_priority_tag(tag_name) {
                2 // Higher than normal XMP but still lower than EXIF
            } else {
                1 // Low priority (Priority 0 in ExifTool) - won't override EXIF
            }
        }

        // Other formats have moderate priority
        "IPTC" => 2,
        "QuickTime" => 3,
        "PNG" | "GIF" => 3,
        "RIFF" => 3,

        // Default priority for unknown groups
        _ => 2,
    }
}

/// Check if an XMP tag should have higher priority than normal XMP tags
/// Some XMP tags are more authoritative than their EXIF counterparts
fn is_xmp_high_priority_tag(tag_name: &str) -> bool {
    match tag_name {
        // XMP rights and attribution tags are more authoritative
        "License" | "AttributionName" | "AttributionURL" | "UseGuidelines" => true,

        // XMP region and people tags have no EXIF equivalent
        "RegionList" | "PersonInImage" | "PersonInImageName" => true,

        // XMP hierarchical keywords are more structured than EXIF keywords
        "HierarchicalKeywords" | "HierarchicalSubject" => true,

        // Most other XMP tags defer to EXIF
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xmp_exif_precedence_rules() {
        // Test that EXIF tags take precedence over XMP tags following ExifTool behavior
        let exif_tag = TagEntry {
            group: "EXIF".to_string(),
            group1: "EXIF".to_string(),
            name: "Make".to_string(),
            value: TagValue::String("Canon".to_string()),
            print: TagValue::String("Canon".to_string()),
        };

        let xmp_tag = TagEntry {
            group: "XMP".to_string(),
            group1: "XMP".to_string(),
            name: "Make".to_string(),
            value: TagValue::String("Nikon".to_string()),
            print: TagValue::String("Nikon".to_string()),
        };

        let file_tag = TagEntry {
            group: "File".to_string(),
            group1: "File".to_string(),
            name: "ImageWidth".to_string(),
            value: TagValue::U16(3000),
            print: TagValue::U16(3000),
        };

        let xmp_imagewidth = TagEntry {
            group: "XMP".to_string(),
            group1: "XMP".to_string(),
            name: "ImageWidth".to_string(),
            value: TagValue::U16(2000),
            print: TagValue::U16(2000),
        };

        let tags = vec![exif_tag, xmp_tag, file_tag, xmp_imagewidth];
        let resolved = apply_exiftool_precedence_rules(tags);

        // Should have 2 tags total (Make and ImageWidth)
        assert_eq!(resolved.len(), 2);

        // Find Make tag - should be EXIF version (Canon), not XMP version (Nikon)
        let make_tag = resolved.iter().find(|t| t.name == "Make");
        assert!(
            make_tag.is_some(),
            "Should have Make tag after precedence resolution"
        );
        if let Some(tag) = make_tag {
            assert_eq!(tag.group, "EXIF", "EXIF:Make should win over XMP:Make");
            assert_eq!(
                tag.value.as_string(),
                Some("Canon"),
                "Should have EXIF value"
            );
        }

        // Find ImageWidth tag - should be File version (3000), not XMP version (2000)
        let width_tag = resolved.iter().find(|t| t.name == "ImageWidth");
        assert!(
            width_tag.is_some(),
            "Should have ImageWidth tag after precedence resolution"
        );
        if let Some(tag) = width_tag {
            assert_eq!(
                tag.group, "File",
                "File:ImageWidth should win over XMP:ImageWidth"
            );
            assert_eq!(tag.value.as_u16(), Some(3000), "Should have File value");
        }
    }

    #[test]
    fn test_xmp_high_priority_tags() {
        // Test that certain XMP tags have higher priority due to being more authoritative
        let exif_keywords = TagEntry {
            group: "EXIF".to_string(),
            group1: "EXIF".to_string(),
            name: "Keywords".to_string(),
            value: TagValue::String("simple keywords".to_string()),
            print: TagValue::String("simple keywords".to_string()),
        };

        let xmp_hierarchical = TagEntry {
            group: "XMP".to_string(),
            group1: "XMP".to_string(),
            name: "HierarchicalKeywords".to_string(),
            value: TagValue::Array(vec![TagValue::String("Nature|Wildlife".to_string())]),
            print: TagValue::Array(vec![TagValue::String("Nature|Wildlife".to_string())]),
        };

        let tags = vec![exif_keywords, xmp_hierarchical];
        let resolved = apply_exiftool_precedence_rules(tags);

        // Should have both tags since they have different names
        assert_eq!(resolved.len(), 2);

        // Verify both tags are present
        assert!(resolved.iter().any(|t| t.name == "Keywords"));
        assert!(resolved.iter().any(|t| t.name == "HierarchicalKeywords"));
    }

    #[test]
    fn test_precedence_priority_values() {
        // Test that priority values follow ExifTool's hierarchy
        assert!(get_tag_priority("File", "ImageWidth") > get_tag_priority("EXIF", "ImageWidth"));
        assert!(get_tag_priority("EXIF", "Make") > get_tag_priority("XMP", "Make"));
        assert!(get_tag_priority("Canon", "LensModel") > get_tag_priority("XMP", "LensModel"));
        assert!(get_tag_priority("Composite", "Aperture") > get_tag_priority("XMP", "Aperture"));

        // Test XMP high priority tags
        assert!(get_tag_priority("XMP", "License") > get_tag_priority("XMP", "CreateDate"));
        assert!(get_tag_priority("XMP", "RegionList") > get_tag_priority("XMP", "Rating"));
    }

    #[test]
    fn test_extract_metadata_nonexistent_file() {
        let path = Path::new("nonexistent_file.jpg");
        let result = extract_metadata(path, false, false, None);
        assert!(result.is_err());
        // Should be an IO error for file not found
        assert!(result.is_err());
    }
}
