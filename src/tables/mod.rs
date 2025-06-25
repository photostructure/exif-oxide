//! Tag table definitions (auto-generated from ExifTool)

#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Exif.pm"]
#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Apple.pm"]
#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Canon.pm"]
#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/FujiFilm.pm"]
#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/MakerNotes.pm"]
#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Olympus.pm"]
#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Nikon.pm"]
#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Panasonic.pm"]
#![doc = "EXIFTOOL-SOURCE: lib/Image/ExifTool/Pentax.pm"]

// Include the generated tag definitions
include!(concat!(env!("OUT_DIR"), "/generated_tags.rs"));

// Table-driven tag definitions with PrintConv
pub mod app_segments; // APP segment identification tables (auto-generated)
pub mod apple_tags;
pub mod canon_tags;
pub mod casio_tags;
pub mod dji_tags;
pub mod exif_tags; // Standard EXIF tags with PrintConv
pub mod fujifilm_tags;
pub mod gopro_tags;
pub mod hasselblad_tags;
pub mod kodak_tags;
pub mod minolta_tags;
pub mod nikon_tags;
pub mod olympus_tags;
pub mod panasonic_tags;
pub mod pentax_tags;
pub mod ricoh_tags;
pub mod samsung_tags;
pub mod sony_tags;

#[cfg(test)]
mod tests;
