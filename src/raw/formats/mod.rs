//! RAW format-specific handlers
//!
//! Each manufacturer has its own module with format-specific processing logic.
//! All handlers follow the Trust ExifTool principle by implementing exact
//! translations of ExifTool's processing logic.

pub mod canon;
pub mod kyocera;
pub mod minolta;
pub mod olympus;
pub mod panasonic;
pub mod sony;

// Future format modules will be added here:
// pub mod nikon;
// pub mod fujifilm;
