//! Operating systems that use PC-style file paths
//! 
//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// Operating systems that use PC-style file paths
pub static PC_OPERATING_SYSTEMS: LazyLock<HashMap<String, bool>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("MSWin32".to_string(), true);
    map.insert("NetWare".to_string(), true);
    map.insert("cygwin".to_string(), true);
    map.insert("dos".to_string(), true);
    map.insert("os2".to_string(), true);
    map.insert("symbian".to_string(), true);
    map
});

/// Check if key exists in Operating systems that use PC-style file paths
pub fn lookup_ispc(key: &String) -> bool {
    PC_OPERATING_SYSTEMS.contains_key(key)
}
