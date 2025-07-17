//! Generated lookup tables from Canon.pm
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool/Canon.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

pub mod canonimagesize;
pub mod canonlenstypes;
pub mod canonmodelid;
pub mod canonquality;
pub mod canonwhitebalance;
pub mod main_inline;
pub mod offon;
pub mod panorama_inline;
pub mod picturestyles;
pub mod userdefstyles;

// Re-export all lookup functions and constants
pub use canonimagesize::*;
pub use canonlenstypes::*;
pub use canonmodelid::*;
pub use canonquality::*;
pub use canonwhitebalance::*;
pub use main_inline::*;
pub use offon::*;
pub use panorama_inline::*;
pub use picturestyles::*;
pub use userdefstyles::*;
