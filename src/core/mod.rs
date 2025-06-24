//! Core parsing functionality

pub mod binary_data;
pub mod containers;
pub mod endian;
pub mod heif;
pub mod ifd;
pub mod jpeg;
pub mod mpf;
pub mod png;
pub mod tiff;
pub mod types;

use crate::detection::{detect_file_type, FileType};
use crate::error::Result;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;

pub use endian::Endian;
pub use types::*;

/// Type of metadata found in a segment
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetadataType {
    Exif,
    Mpf,
    Xmp,
    Iptc,
}

/// Unified metadata segment that can hold EXIF data from any format
#[derive(Debug)]
pub struct MetadataSegment {
    /// Raw EXIF/IFD data ready for parsing
    pub data: Vec<u8>,
    /// Offset in the file where this data was found
    pub offset: u64,
    /// Source format that provided this data
    pub source_format: FileType,
    /// Type of metadata in this segment
    pub metadata_type: MetadataType,
}

/// Collection of metadata segments from a file
#[derive(Debug)]
pub struct MetadataCollection {
    /// EXIF segment if found
    pub exif: Option<MetadataSegment>,
    /// MPF segment if found
    pub mpf: Option<MetadataSegment>,
    /// XMP segments (can be multiple)
    pub xmp: Vec<MetadataSegment>,
    /// IPTC segment if found
    pub iptc: Option<MetadataSegment>,
}

/// Find all metadata segments from any supported file format
///
/// This function returns all available metadata segments (EXIF, MPF, XMP, etc.)
pub fn find_all_metadata_segments<P: AsRef<Path>>(path: P) -> Result<MetadataCollection> {
    let mut file = File::open(&path)?;
    find_all_metadata_segments_from_reader(&mut file)
}

/// Find metadata segment from any supported file format (backward compatibility)
///
/// This is the central dispatch function that replaces all hardcoded calls to jpeg::find_exif_segment.
/// It detects the file format and calls the appropriate parser.
pub fn find_metadata_segment<P: AsRef<Path>>(path: P) -> Result<Option<MetadataSegment>> {
    let collection = find_all_metadata_segments(path)?;
    Ok(collection.exif)
}

/// Find all metadata segments from a reader
pub fn find_all_metadata_segments_from_reader<R: Read + Seek>(
    reader: &mut R,
) -> Result<MetadataCollection> {
    // Read first 1KB for format detection
    let mut detection_buffer = vec![0u8; 1024];
    let bytes_read = reader.read(&mut detection_buffer)?;
    detection_buffer.truncate(bytes_read);

    // Detect file format
    let file_info = detect_file_type(&detection_buffer)?;
    let format = file_info.file_type;

    // Reset reader to beginning
    reader.seek(std::io::SeekFrom::Start(0))?;

    // Dispatch to appropriate parser based on format
    match format {
        FileType::JPEG => {
            let jpeg_metadata = jpeg::find_metadata_segments(reader)?;

            let mut collection = MetadataCollection {
                exif: None,
                mpf: None,
                xmp: Vec::new(),
                iptc: None,
            };

            // Convert JPEG segments to MetadataSegment
            if let Some(exif) = jpeg_metadata.exif {
                collection.exif = Some(MetadataSegment {
                    data: exif.data,
                    offset: exif.offset,
                    source_format: format,
                    metadata_type: MetadataType::Exif,
                });
            }

            if let Some(mpf) = jpeg_metadata.mpf {
                collection.mpf = Some(MetadataSegment {
                    data: mpf.data,
                    offset: mpf.offset,
                    source_format: format,
                    metadata_type: MetadataType::Mpf,
                });
            }

            for xmp in jpeg_metadata.xmp {
                collection.xmp.push(MetadataSegment {
                    data: xmp.data,
                    offset: xmp.offset,
                    source_format: format,
                    metadata_type: MetadataType::Xmp,
                });
            }

            Ok(collection)
        }

        // For non-JPEG formats, we only have EXIF data for now
        _ => {
            let mut collection = MetadataCollection {
                exif: None,
                mpf: None,
                xmp: Vec::new(),
                iptc: None,
            };

            // Use existing single-segment logic for other formats
            if let Some(segment) = find_metadata_segment_from_reader_internal(reader, format)? {
                collection.exif = Some(segment);
            }

            Ok(collection)
        }
    }
}

/// Find metadata segment from a reader (for files already opened) - backward compatibility
pub fn find_metadata_segment_from_reader<R: Read + Seek>(
    reader: &mut R,
) -> Result<Option<MetadataSegment>> {
    let collection = find_all_metadata_segments_from_reader(reader)?;
    Ok(collection.exif)
}

/// Internal function to find single metadata segment for non-JPEG formats
fn find_metadata_segment_from_reader_internal<R: Read + Seek>(
    reader: &mut R,
    format: FileType,
) -> Result<Option<MetadataSegment>> {
    // Dispatch to appropriate parser based on format
    match format {
        FileType::JPEG => {
            // JPEG is handled by find_all_metadata_segments_from_reader
            unreachable!("JPEG should be handled by find_all_metadata_segments_from_reader")
        }

        FileType::PNG => {
            if let Some(segment) = png::find_exif_chunk(reader)? {
                Ok(Some(MetadataSegment {
                    data: segment.data,
                    offset: segment.offset,
                    source_format: format,
                    metadata_type: MetadataType::Exif,
                }))
            } else {
                Ok(None)
            }
        }

        // TIFF-based formats (TIFF, CR2, NEF, ARW, etc.)
        FileType::TIFF
        | FileType::CR2
        | FileType::NEF
        | FileType::ARW
        | FileType::SR2
        | FileType::ORF
        | FileType::PEF
        | FileType::RW2
        | FileType::DNG
        | FileType::RAF
        | FileType::X3F
        | FileType::CRW => {
            if let Some(segment) = tiff::find_ifd_data(reader)? {
                Ok(Some(MetadataSegment {
                    data: segment.data,
                    offset: segment.offset,
                    source_format: format,
                    metadata_type: MetadataType::Exif,
                }))
            } else {
                Ok(None)
            }
        }

        // HEIF/HEIC/MP4 container formats
        FileType::HEIF
        | FileType::HEIC
        | FileType::AVIF
        | FileType::MP4
        | FileType::MOV
        | FileType::CR3 => {
            if let Some(segment) = heif::find_exif_atom(reader)? {
                Ok(Some(MetadataSegment {
                    data: segment.data,
                    offset: segment.offset,
                    source_format: format,
                    metadata_type: MetadataType::Exif,
                }))
            } else {
                Ok(None)
            }
        }

        // RIFF container formats (WebP, AVI)
        FileType::WEBP | FileType::AVI => {
            if let Some(segment) = containers::riff::find_metadata(reader)? {
                Ok(Some(MetadataSegment {
                    data: segment.data,
                    offset: segment.offset,
                    source_format: format,
                    metadata_type: MetadataType::Exif,
                }))
            } else {
                Ok(None)
            }
        }

        // Additional QuickTime variants handled by quicktime parser
        FileType::ThreeGPP | FileType::ThreeGPP2 | FileType::M4V => {
            if let Some(segment) = containers::quicktime::find_metadata(reader)? {
                Ok(Some(MetadataSegment {
                    data: segment.data,
                    offset: segment.offset,
                    source_format: format,
                    metadata_type: MetadataType::Exif,
                }))
            } else {
                Ok(None)
            }
        }

        // Formats not yet supported for metadata extraction
        FileType::GIF
        | FileType::BMP
        | FileType::HEIFS
        | FileType::HEICS
        | FileType::CRM
        | FileType::ARQ
        | FileType::SRF
        | FileType::RAW
        | FileType::RWL
        | FileType::ThreeFR
        | FileType::FFF
        | FileType::IIQ
        | FileType::GPR
        | FileType::ERF
        | FileType::DCR
        | FileType::K25
        | FileType::KDC
        | FileType::MEF
        | FileType::MRW
        | FileType::SRW
        | FileType::NRW
        | FileType::Unknown => {
            // These formats either don't contain EXIF data or are not yet implemented
            Ok(None)
        }
    }
}
