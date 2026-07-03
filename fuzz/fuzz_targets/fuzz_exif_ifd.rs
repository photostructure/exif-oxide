#![no_main]
//! Highest-value target: the shared TIFF/EXIF IFD walker.
//!
//! `ExifReader::parse_exif_data` is where JPEG, TIFF, DNG and every RAW format
//! (Canon, Sony, Olympus, Panasonic, Minolta, Kyocera) funnel into after their
//! own header/offset handling. The input is raw TIFF-structured bytes (an
//! `II`/`MM` header, magic 42, IFD offset chain) — not a JPEG/RAW container.
//!
//! A malformed IFD (bad entry count, an offset that points past EOF, a negative
//! or absurd component count) must return `Err`, never panic or allocate
//! unbounded. Seed corpus is TIFF-based files, which are valid input verbatim.
use exif_oxide::exif::ExifReader;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut reader = ExifReader::new();
    let _ = reader.parse_exif_data(data);
});
