//! RAW format-specific handlers
//!
//! Each manufacturer has its own module with format-specific processing logic.
//! All handlers follow the Trust ExifTool principle by implementing exact 
//! translations of ExifTool's processing logic.

pub mod kyocera;

// Future format modules will be added here:
// pub mod canon;
// pub mod nikon; 
// pub mod sony;
// pub mod olympus;
// pub mod panasonic;
// pub mod fujifilm;