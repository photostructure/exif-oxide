//! PNG chunks containing text metadata (tEXt, zTXt, iTXt, eXIf)
//!
//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// PNG chunks containing text metadata (tEXt, zTXt, iTXt, eXIf)
pub static PNG_TEXT_CHUNKS: LazyLock<HashMap<String, bool>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("eXIf".to_string(), true);
    map.insert("iTXt".to_string(), true);
    map.insert("tEXt".to_string(), true);
    map.insert("zTXt".to_string(), true);
    map
});

/// Check if key exists in PNG chunks containing text metadata (tEXt, zTXt, iTXt, eXIf)
pub fn lookup_istxtchunk(key: &String) -> bool {
    PNG_TEXT_CHUNKS.contains_key(key)
}
