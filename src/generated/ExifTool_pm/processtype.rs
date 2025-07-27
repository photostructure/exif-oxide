//! File types determined by process proc during FastScan == 3
//!
//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// File types determined by process proc during FastScan == 3
pub static PROCESS_DETERMINED_TYPES: LazyLock<HashMap<String, bool>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("AIFF".to_string(), true);
    map.insert("EXE".to_string(), true);
    map.insert("Font".to_string(), true);
    map.insert("JPEG".to_string(), true);
    map.insert("PS".to_string(), true);
    map.insert("Real".to_string(), true);
    map.insert("TIFF".to_string(), true);
    map.insert("TXT".to_string(), true);
    map.insert("VCard".to_string(), true);
    map.insert("XMP".to_string(), true);
    map
});

/// Check if key exists in File types determined by process proc during FastScan == 3
pub fn lookup_processtype(key: &String) -> bool {
    PROCESS_DETERMINED_TYPES.contains_key(key)
}
