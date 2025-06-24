//! Tag table definitions (auto-generated from ExifTool)

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm"]
#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Canon.pm"]
#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Olympus.pm"]
#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Nikon.pm"]
#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Pentax.pm"]

// Include the generated tag definitions
include!(concat!(env!("OUT_DIR"), "/generated_tags.rs"));

#[cfg(test)]
mod tests;
