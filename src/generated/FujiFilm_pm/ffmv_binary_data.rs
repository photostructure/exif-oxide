//! Auto-generated from ExifTool source
//! DO NOT EDIT MANUALLY - changes will be overwritten by codegen

//! FujiFilm ProcessBinaryData table FFMV generated from FujiFilm.pm
//! ExifTool: FujiFilm.pm %FujiFilm::FFMV
//! Generated at: Sat Jul 19 20:59:47 2025 GMT

use std::collections::HashMap;
use std::sync::LazyLock;

/// FujiFilm ProcessBinaryData table for FFMV
/// Total tags: 1
#[derive(Debug, Clone)]
pub struct FujiFilmFFMVTable {
    pub first_entry: i32,                     // 0
    pub groups: (&'static str, &'static str), // ("MakerNotes", "Camera")
}

/// Tag definitions for FujiFilm FFMV
pub static FFMV_TAGS: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(0, "MovieStreamName"); // 0x00: MovieStreamName
    map
});

/// Format specifications for FujiFilm FFMV
pub static FFMV_FORMATS: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(0, "string[34]"); // 0x00: MovieStreamName
    map
});

impl FujiFilmFFMVTable {
    /// Create new table instance
    pub fn new() -> Self {
        Self {
            first_entry: 0,
            groups: ("MakerNotes", "Camera"),
        }
    }

    /// Get tag name for offset
    pub fn get_tag_name(&self, offset: u16) -> Option<&'static str> {
        FFMV_TAGS.get(&offset).copied()
    }

    /// Get format specification for offset
    pub fn get_format(&self, offset: u16) -> Option<&'static str> {
        FFMV_FORMATS.get(&offset).copied()
    }

    /// Get all valid offsets for this table
    pub fn get_offsets(&self) -> Vec<u16> {
        FFMV_TAGS.keys().copied().collect()
    }
}

impl Default for FujiFilmFFMVTable {
    fn default() -> Self {
        Self::new()
    }
}
