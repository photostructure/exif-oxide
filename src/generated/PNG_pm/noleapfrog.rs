//! PNG chunks that shouldn't be moved across during editing
//! 
//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

use std::collections::HashMap;
use std::sync::LazyLock;

/// PNG chunks that shouldn't be moved across during editing
pub static PNG_NO_LEAPFROG_CHUNKS: LazyLock<HashMap<String, bool>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("BASI".to_string(), true);
    map.insert("CLON".to_string(), true);
    map.insert("DHDR".to_string(), true);
    map.insert("IEND".to_string(), true);
    map.insert("IHDR".to_string(), true);
    map.insert("JHDR".to_string(), true);
    map.insert("MAGN".to_string(), true);
    map.insert("MEND".to_string(), true);
    map.insert("PAST".to_string(), true);
    map.insert("SAVE".to_string(), true);
    map.insert("SEEK".to_string(), true);
    map.insert("SHOW".to_string(), true);
    map
});

/// Check if key exists in PNG chunks that shouldn't be moved across during editing
pub fn lookup_noleapfrog(key: &String) -> bool {
    PNG_NO_LEAPFROG_CHUNKS.contains_key(key)
}
