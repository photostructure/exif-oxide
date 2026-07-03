#![no_main]
//! GIF logical screen descriptor parsing.
use exif_oxide::formats::parse_gif_screen_descriptor;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = parse_gif_screen_descriptor(data);
});
