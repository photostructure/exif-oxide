//! PNG chunks containing image data (IDAT, JDAT, JDAA)
//!
//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// PNG chunks containing image data (IDAT, JDAT, JDAA)
pub static PNG_DATA_CHUNKS: LazyLock<HashMap<String, bool>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("IDAT".to_string(), true);
    map.insert("JDAA".to_string(), true);
    map.insert("JDAT".to_string(), true);
    map
});

/// Check if key exists in PNG chunks containing image data (IDAT, JDAT, JDAA)
pub fn lookup_isdatchunk(key: &String) -> bool {
    PNG_DATA_CHUNKS.contains_key(key)
}
