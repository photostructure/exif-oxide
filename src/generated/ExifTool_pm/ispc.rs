//! Operating systems that use PC-style file paths
//!
//! Auto-generated from third-party/exiftool/lib/Image/ExifTool.pm
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashSet;
use std::sync::LazyLock;

// Generated pc operating systems boolean set
// Source: ExifTool Image::ExifTool %isPC
// Description: Operating systems that use PC-style file paths

/// Static data for operating systems that use pc-style file paths set (6 entries)
static PC_OPERATING_SYSTEMS_DATA: &[&str] =
    &["MSWin32", "NetWare", "cygwin", "dos", "os2", "symbian"];

/// Operating systems that use PC-style file paths boolean set table
/// Built from static data on first access
pub static PC_OPERATING_SYSTEMS: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| PC_OPERATING_SYSTEMS_DATA.iter().copied().collect());

/// Check if a file type is in the operating systems that use pc-style file paths set
pub fn is_pc_operating_systems(file_type: &str) -> bool {
    PC_OPERATING_SYSTEMS.contains(file_type)
}
