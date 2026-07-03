#![no_main]
//! TIFF container validation and EXIF/XMP extraction.
//!
//! `extract_tiff_exif` takes a `Read + Seek` reader; the header validators and
//! `extract_tiff_xmp` take `&[u8]` directly. This exercises the byte-order
//! sniffing and the length/offset math before the IFD walker (fuzzed
//! separately by `fuzz_exif_ifd`) takes over.
use exif_oxide::formats::{
    extract_tiff_exif, extract_tiff_xmp, get_tiff_endianness, validate_tiff_format,
};
use libfuzzer_sys::fuzz_target;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let _ = validate_tiff_format(data);
    let _ = get_tiff_endianness(data);
    let _ = extract_tiff_xmp(data);
    let _ = extract_tiff_exif(Cursor::new(data));
});
