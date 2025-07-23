//! File types that can be created from scratch (XMP, ICC, MIE, VRD, etc.)
//! 
//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// File types that can be created from scratch (XMP, ICC, MIE, VRD, etc.)
pub static CREATABLE_FILE_TYPES: LazyLock<HashMap<String, bool>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("DR4".to_string(), true);
    map.insert("EXIF".to_string(), true);
    map.insert("EXV".to_string(), true);
    map.insert("ICC".to_string(), true);
    map.insert("MIE".to_string(), true);
    map.insert("VRD".to_string(), true);
    map.insert("XMP".to_string(), true);
    map
});

/// Check if key exists in File types that can be created from scratch (XMP, ICC, MIE, VRD, etc.)
pub fn lookup_createtypes(key: &String) -> bool {
    CREATABLE_FILE_TYPES.contains_key(key)
}
