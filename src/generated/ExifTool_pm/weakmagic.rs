//! File types with weak magic number recognition (MP3)
//! 
//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// File types with weak magic number recognition (MP3)
pub static WEAK_MAGIC_FILE_TYPES: LazyLock<HashMap<String, bool>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("MP3".to_string(), true);
    map
});

/// Check if key exists in File types with weak magic number recognition (MP3)
pub fn lookup_weakmagic(key: &String) -> bool {
    WEAK_MAGIC_FILE_TYPES.contains_key(key)
}
