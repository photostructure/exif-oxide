//! Canon-specific EXIF processing coordinator
//!
//! This module coordinates Canon manufacturer-specific processing,
//! dispatching to specialized sub-modules for different aspects.
//!
//! **Trust ExifTool**: This code translates ExifTool's Canon processing verbatim
//! without any improvements or simplifications. Every algorithm, magic number, and
//! quirk is copied exactly as documented in the ExifTool source.
//!
//! Primary ExifTool References:
//! - lib/Image/ExifTool/Canon.pm - Canon tag tables and processing
//! - lib/Image/ExifTool/MakerNotes.pm - Canon MakerNote detection and offset fixing

pub mod af_info;
pub mod binary_data;
pub mod offset_schemes;
pub mod tags;
pub mod tiff_footer;

// Re-export commonly used binary_data functions for easier access
pub use binary_data::{
    create_canon_camera_settings_table, extract_binary_data_tags, extract_binary_value,
    find_canon_camera_settings_tag,
};
// Re-export offset scheme functions
pub use offset_schemes::{detect_canon_signature, detect_offset_scheme, CanonOffsetScheme};
// Re-export tag name functions
pub use tags::get_canon_tag_name;

use crate::types::Result;
use tracing::debug;

// CameraSettings functions are provided by the binary_data module

// extract_camera_settings function is provided by the binary_data module

/// Process Canon MakerNotes data
/// ExifTool: lib/Image/ExifTool/Canon.pm Canon MakerNote processing
/// This function processes Canon MakerNotes as an IFD structure to extract Canon-specific tags
pub fn process_canon_makernotes(
    exif_reader: &mut crate::exif::ExifReader,
    dir_start: usize,
    size: usize,
) -> Result<()> {
    use crate::types::DirectoryInfo;

    debug!(
        "Processing Canon MakerNotes: start={:#x}, size={}",
        dir_start, size
    );

    // Canon MakerNotes are structured as a standard IFD
    // ExifTool: Canon.pm Main table processes Canon tags as subdirectories
    let dir_info = DirectoryInfo {
        name: "Canon".to_string(),
        dir_start,
        dir_len: size,
        base: exif_reader.base,
        data_pos: 0,
        allow_reprocess: true, // Allow reprocessing same address as Canon processor
    };

    // Process the Canon MakerNotes IFD to extract individual Canon tags
    // This will extract tags like CanonCameraSettings, CanonShotInfo, etc.
    exif_reader.process_subdirectory(&dir_info)?;

    debug!("Canon MakerNotes processing completed");
    Ok(())
}

// Unit tests are in a separate module
#[cfg(test)]
mod tests;
