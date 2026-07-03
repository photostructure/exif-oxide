#![no_main]
//! PNG IHDR chunk parsing.
//!
//! `parse_png_ihdr` reads the signature and the first chunk's
//! width/height/bit-depth/color-type fields. Truncated or lying chunk-length
//! fields are the thing to catch here.
use exif_oxide::formats::parse_png_ihdr;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = parse_png_ihdr(data);
});
