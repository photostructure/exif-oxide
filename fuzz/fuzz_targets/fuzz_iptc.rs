#![no_main]
//! IPTC IIM parsing.
//!
//! `parse_iptc_from_app13` unwraps the Photoshop APP13 8BIM wrapper and hands
//! the IPTC resource to `parse_iptc_metadata`, which walks the IIM
//! dataset-marker records. Both entry points take `&[u8]`; the fuzz input is
//! fed to each so mutation can explore both the wrapped and unwrapped forms.
use exif_oxide::formats::{parse_iptc_from_app13, parse_iptc_metadata};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = parse_iptc_from_app13(data);
    let _ = parse_iptc_metadata(data);
});
