#![no_main]
//! ISO-BMFF box parsing for AVIF/HEIC.
//!
//! `parse_box_header` reads a single box's size/type (including the 64-bit
//! extended-size escape); `extract_avif_dimensions` and
//! `extract_heic_dimensions_primary_item` walk the full nested box tree
//! (meta/iprp/ipco/ispe, pitm, iinf, ipma) to resolve the primary item's
//! dimensions. Attacker-controlled 64-bit box sizes and self-referential
//! offsets are the danger here.
use exif_oxide::formats::{
    extract_avif_dimensions, extract_heic_dimensions_primary_item, parse_box_header,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = parse_box_header(data, 0);
    let _ = extract_avif_dimensions(data);
    let _ = extract_heic_dimensions_primary_item(data);
});
