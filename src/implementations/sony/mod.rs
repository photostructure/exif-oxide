//! Sony implementation module
//!
//! This module provides Sony-specific EXIF processing implementations,
//! following ExifTool's Sony.pm logic exactly.

pub mod makernote_detection;
pub mod tags;

// Re-export key functions for use by other modules
pub use makernote_detection::{detect_sony_signature, is_sony_makernote, SonySignature};
pub use tags::{get_sony_tag_name, get_sony_namespace, is_sony_tag};