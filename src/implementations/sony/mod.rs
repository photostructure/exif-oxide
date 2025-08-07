//! Sony implementation module
//!
//! This module provides Sony-specific EXIF processing implementations,
//! following ExifTool's Sony.pm logic exactly.

pub mod makernote_detection;
pub mod tags;

// Re-export key functions for use by other modules
pub use makernote_detection::{detect_sony_signature, is_sony_makernote, SonySignature};
pub use tags::{get_sony_namespace, get_sony_tag_name, is_sony_tag};

use crate::exif::ExifReader;
use crate::types::Result;
use tracing::debug;

/// Find Sony tag ID by name from the tag kit system
/// Used for applying PrintConv to subdirectory-extracted tags
fn find_sony_tag_id_by_name(tag_name: &str) -> Option<u32> {
    use crate::generated::sony::main_tags::SONY_MAIN_TAGS as SONY_PM_TAG_KITS;

    // Search through all Sony tag kit entries to find matching name
    for (&tag_id, tag_def) in SONY_PM_TAG_KITS.iter() {
        if tag_def.name == tag_name {
            return Some(tag_id);
        }
    }
    None
}

/// Process Sony subdirectory tags using the generic subdirectory processing system
/// ExifTool: Sony.pm SubDirectory processing for binary data expansion
pub fn process_sony_subdirectory_tags(exif_reader: &mut ExifReader) -> Result<()> {
    use crate::exif::subdirectory_processing::process_subdirectories_with_printconv;
    use crate::generated::sony::main_tags;

    debug!("Processing Sony subdirectory tags using generic system");

    // Use the generic subdirectory processing with Sony-specific functions
    // Fix Group1 assignment: Use "Sony" as namespace for group1="Sony" instead of "MakerNotes"
    // TODO: Task E - Replace tag_kit functions with manufacturer-specific implementations
    // process_subdirectories_with_printconv(
    //     exif_reader,
    //     "Sony",
    //     "Sony",
    //     tag_kit::has_subdirectory,
    //     tag_kit::process_subdirectory,
    //     tag_kit::apply_print_conv,
    //     find_sony_tag_id_by_name,
    // )?;

    debug!("Sony subdirectory processing completed");
    Ok(())
}
