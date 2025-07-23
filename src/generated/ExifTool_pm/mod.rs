//! Generated lookup tables from ExifTool.pm
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

pub mod mimetype;
pub mod filetypeext;
pub mod weakmagic;
pub mod createtypes;
pub mod processtype;
pub mod ispc;
pub mod regex_patterns;

// Re-export all lookup functions and constants
pub use mimetype::*;
pub use filetypeext::*;
pub use weakmagic::*;
pub use createtypes::*;
pub use processtype::*;
pub use ispc::*;
pub use regex_patterns::*;
