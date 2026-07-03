#![no_main]
//! JPEG segment scanning and the APP-segment extractors.
//!
//! `scan_jpeg_segments` walks the FF-marker segment chain; the `extract_*`
//! functions pull EXIF (APP1), XMP (APP1), and IPTC (APP13) blocks out of it.
//! All take a `Read + Seek` reader, so each gets a fresh `Cursor` over the same
//! fuzz input. Corrupt segment-length fields are the classic bug source here.
use exif_oxide::formats::{
    extract_jpeg_exif, extract_jpeg_iptc, extract_jpeg_xmp, scan_jpeg_segments,
};
use libfuzzer_sys::fuzz_target;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let _ = scan_jpeg_segments(Cursor::new(data));
    let _ = extract_jpeg_exif(Cursor::new(data));
    let _ = extract_jpeg_xmp(Cursor::new(data));
    let _ = extract_jpeg_iptc(Cursor::new(data));
});
