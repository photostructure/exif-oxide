//! Generated lookup tables from XMP.pm
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/XMP.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

pub mod nsuri;
pub mod xmpns;
pub mod charname;
pub mod charnum;
pub mod stdxlatns;

// Re-export all lookup functions and constants
pub use nsuri::*;
pub use xmpns::*;
pub use charname::*;
pub use charnum::*;
pub use stdxlatns::*;
