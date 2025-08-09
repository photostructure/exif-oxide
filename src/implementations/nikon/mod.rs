//! Nikon-specific EXIF processing coordinator
//!
//! This module coordinates Nikon manufacturer-specific processing,
//! dispatching to specialized sub-modules for different aspects.
//!
//! **Trust ExifTool**: This code translates ExifTool's Nikon processing verbatim
//! without any improvements or simplifications.
//!
//! ExifTool Reference: lib/Image/ExifTool/Nikon.pm (14,191 lines)
//!
//! This module handles Nikon's sophisticated maker note format including:
//! - Multiple format versions (Format1, Format2, Format3)
//! - Advanced encryption system with serial number and shutter count keys
//! - Comprehensive lens database (618 entries)
//! - Model-specific subdirectories and processing
//! - Complex AF grid systems
//!
//! ## Module Organization
//!
//! Following the proven Canon pattern:
//! - `detection.rs` - Multi-format maker note detection and signature validation
//! - `offset_schemes.rs` - Format-specific offset calculations and base management
//! - `encryption.rs` - Encryption key management and encrypted section processing
//! - `ifd.rs` - IFD parsing, tag extraction, and standard tag processing
//! - `tags.rs` - Primary tag ID mappings and model-specific tables
//! - `lens_database.rs` - 618-entry lens ID lookup system
//! - `tests.rs` - Comprehensive unit tests for all components

pub mod af_processing;
pub mod binary_data_extraction;
pub mod detection;
pub mod encrypted_processing;
pub mod encryption;
pub mod ifd;
pub mod lens_database;
pub mod offset_schemes;
pub mod tags;

// Re-export commonly used functions for easier access
pub use af_processing::{process_nikon_af_info, NikonAfSystem};
pub use detection::{detect_nikon_format, detect_nikon_signature};
pub use encrypted_processing::{
    process_encrypted_colorbalance, process_encrypted_lensdata, process_encrypted_shotinfo,
    NikonCameraModel,
};
pub use encryption::{process_encrypted_sections, NikonEncryptionKeys};
pub use ifd::{prescan_for_encryption_keys, process_standard_nikon_tags};
pub use lens_database::lookup_nikon_lens;
pub use offset_schemes::calculate_nikon_base_offset;
pub use tags::{get_nikon_tag_name, select_nikon_tag_table};

use crate::exif::ExifReader;
use crate::types::{ExifError, Result};
use tracing::debug;

/// Main entry point for Nikon MakerNote processing
/// ExifTool: Nikon.pm ProcessNikon function (lines 890-1200+)
pub fn process_nikon_makernotes(reader: &mut ExifReader, offset: usize) -> Result<()> {
    debug!("Processing Nikon MakerNotes at offset {:#x}", offset);

    // Get data length for validation
    let data_len = reader.get_data_len();

    // Validate basic data availability
    if offset >= data_len {
        return Err(ExifError::ParseError(format!(
            "Nikon MakerNote offset {offset:#x} beyond data bounds ({data_len})"
        )));
    }

    // Get a small sample for format detection (avoiding long-term borrow)
    let format_data = {
        let data = reader.get_data();
        if offset + 10 <= data.len() {
            data[offset..offset + 10].to_vec()
        } else {
            data[offset..].to_vec()
        }
    };

    // Phase 1: Detect Nikon format version
    let format = detection::detect_nikon_format(&format_data)
        .ok_or_else(|| ExifError::ParseError("Unable to detect Nikon format".to_string()))?;

    debug!("Detected Nikon format: {:?}", format);

    // Phase 2: Calculate base offset using format-specific scheme
    let base_offset = offset_schemes::calculate_nikon_base_offset(format, offset);

    debug!(
        "Calculated Nikon base offset: {:#x} (format: {:?})",
        base_offset, format
    );

    // Phase 3: Initialize encryption key management
    // For Phase 2, use a generic model name - will be updated from actual tags
    let mut encryption_keys = encryption::NikonEncryptionKeys::new("Nikon Camera".to_string());

    // Phase 4: Pre-scan for encryption keys (Phase 2 implementation)
    // ExifTool: Nikon.pm lines 1234-1267 - pre-scan for SerialNumber (0x001d) and ShutterCount (0x00a7)
    ifd::prescan_for_encryption_keys(reader, base_offset, &mut encryption_keys)?;

    // Phase 5: Process standard Nikon tags (Phase 2 implementation)
    ifd::process_standard_nikon_tags(reader, base_offset, &encryption_keys)?;

    // Phase 6: Process encrypted sections (Phase 2 implementation)
    encryption::process_encrypted_sections(reader, base_offset, &mut encryption_keys)?;

    Ok(())
}

/// Find Nikon tag ID by name from the tag kit system
/// Used for applying PrintConv to subdirectory-extracted tags
fn find_nikon_tag_id_by_name(tag_name: &str) -> Option<u32> {
    use crate::generated::Nikon_pm::main_tags::NIKON_MAIN_TAGS;

    // Search through all Nikon tag kit entries to find matching name
    for (&tag_id, tag_def) in NIKON_MAIN_TAGS.iter() {
        if tag_def.name == tag_name {
            return Some(tag_id.into());
        }
    }
    None
}

/// Process Nikon subdirectory tags using the generic subdirectory processing system
/// ExifTool: Nikon.pm SubDirectory processing for binary data expansion
pub fn process_nikon_subdirectory_tags(exif_reader: &mut ExifReader) -> Result<()> {
    use crate::exif::subdirectory_processing::process_subdirectories_with_printconv;
    use crate::generated::Nikon_pm::tag_kit;

    debug!("Processing Nikon subdirectory tags using generic system");

    // Use the generic subdirectory processing with Nikon-specific functions
    // Fix Group1 assignment: Use "Nikon" as namespace for group1="Nikon" instead of "MakerNotes"
    // TODO: P07 Task E - Replace tag_kit functions with manufacturer-specific implementations
    // process_subdirectories_with_printconv(
    //     exif_reader,
    //     "Nikon",
    //     "Nikon",
    //     tag_kit::has_subdirectory,
    //     tag_kit::process_subdirectory,
    //     tag_kit::apply_print_conv,
    //     find_nikon_tag_id_by_name,
    // )?;

    debug!("Nikon subdirectory processing completed");
    Ok(())
}
