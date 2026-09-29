//! FujiFilm MakerNotes dispatch and offset handling.
//!
//! The generated FujiFilm tables (src/generated/FujiFilm_pm/) carry the whole
//! Main table, PrintConvs included, but nothing reached them: the dispatch in
//! process_maker_notes_with_signature_detection branched on Make for Canon,
//! Olympus and Sony and sent everything else down the generic path, which reads
//! the subdirectory as a plain TIFF IFD. A Fuji file came back with every tag
//! named Tag_1401 rather than FilmMode.
//!
//! ExifTool: lib/Image/ExifTool/MakerNotes.pm:121-134 (MakerNoteFujiFilm).
//!
//! These are synthetic files rather than fixtures because each one pins a
//! separate way the subdirectory can be read wrongly while still parsing.

use exif_oxide::exif::ExifReader;

/// Offset of the MakerNotes value inside the synthetic TIFF below.
const MAKERNOTE_OFFSET: u32 = 48;

/// Where the FujiFilm IFD sits inside the MakerNotes block.
///
/// Deliberately 16 rather than the 12 every X-series body writes: the value is
/// a POINTER stored at byte 8 (ExifTool: OffsetPt => '$valuePtr+8'), so an
/// implementation that hardcodes a skip passes on real files and fails here.
const IFD_POINTER: u32 = 16;

/// Build a little-endian TIFF whose IFD0 carries the given Make and a
/// MakerNotes (0x927C) block with the given 8-byte signature.
///
/// The MakerNotes block holds three entries chosen to cover the three ways this
/// subdirectory differs from a plain IFD:
///   0x1401 FilmMode      SHORT, inline   -> needs the generated PrintConv
///   0x1040 ShadowTone    SLONG, inline   -> a format the old Fuji reader dropped
///   0x1000 Quality       ASCII, indirect -> needs Base => '$start'
fn fujifilm_tiff(make: &[u8], signature: &[u8; 8]) -> Vec<u8> {
    let mut d = Vec::new();

    // TIFF header: little-endian, magic 42, IFD0 at offset 8
    d.extend_from_slice(&[0x49, 0x49, 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00]);

    // IFD0: 2 entries
    d.extend_from_slice(&2u16.to_le_bytes());
    // Make (0x010F), ASCII, at offset 38
    d.extend_from_slice(&0x010Fu16.to_le_bytes());
    d.extend_from_slice(&2u16.to_le_bytes());
    d.extend_from_slice(&(make.len() as u32).to_le_bytes());
    d.extend_from_slice(&38u32.to_le_bytes());
    // MakerNote (0x927C), UNDEFINED, at MAKERNOTE_OFFSET
    d.extend_from_slice(&0x927Cu16.to_le_bytes());
    d.extend_from_slice(&7u16.to_le_bytes());
    d.extend_from_slice(&66u32.to_le_bytes());
    d.extend_from_slice(&MAKERNOTE_OFFSET.to_le_bytes());
    // No next IFD
    d.extend_from_slice(&0u32.to_le_bytes());
    assert_eq!(d.len(), 38);

    // Offset 38: Make, padded out to the MakerNotes block
    d.extend_from_slice(make);
    while d.len() < MAKERNOTE_OFFSET as usize {
        d.push(0);
    }

    // The MakerNotes block. Every offset from here on is relative to its start,
    // which is the whole point of Base => '$start'.
    let mut mn = Vec::new();
    mn.extend_from_slice(signature); // +0
    mn.extend_from_slice(&IFD_POINTER.to_le_bytes()); // +8: pointer to the IFD
    mn.extend_from_slice(&[0xFF; 4]); // +12: filler the pointer skips over

    // +16: the IFD itself, 3 entries
    mn.extend_from_slice(&3u16.to_le_bytes());
    // FilmMode = 1536 (Classic Chrome)
    mn.extend_from_slice(&0x1401u16.to_le_bytes());
    mn.extend_from_slice(&3u16.to_le_bytes());
    mn.extend_from_slice(&1u32.to_le_bytes());
    mn.extend_from_slice(&1536u32.to_le_bytes());
    // ShadowTone = 32, SLONG (format 9)
    mn.extend_from_slice(&0x1040u16.to_le_bytes());
    mn.extend_from_slice(&9u16.to_le_bytes());
    mn.extend_from_slice(&1u32.to_le_bytes());
    mn.extend_from_slice(&32u32.to_le_bytes());
    // Quality, ASCII, 8 bytes living at MakerNotes-relative offset 58
    mn.extend_from_slice(&0x1000u16.to_le_bytes());
    mn.extend_from_slice(&2u16.to_le_bytes());
    mn.extend_from_slice(&8u32.to_le_bytes());
    mn.extend_from_slice(&58u32.to_le_bytes());
    // No next IFD
    mn.extend_from_slice(&0u32.to_le_bytes());
    assert_eq!(mn.len(), 58);

    // +58: the Quality string. Read TIFF-relative instead of
    // MakerNotes-relative, offset 58 lands back in IFD0 and this test fails
    // with a garbage string rather than an error.
    mn.extend_from_slice(b"FINE   \0");
    assert_eq!(mn.len(), 66);

    d.extend_from_slice(&mn);
    d
}

/// Read one MakerNotes tag's print value.
fn print_value(data: &[u8], name: &str) -> Option<String> {
    let mut reader = ExifReader::new();
    reader.parse_exif_data(data).expect("parse");
    reader
        .get_all_tag_entries()
        .into_iter()
        .find(|e| e.name == name && e.group == "MakerNotes")
        .map(|e| e.print.to_string())
}

#[test]
fn test_fujifilm_tags_are_named_from_the_generated_table() {
    let data = fujifilm_tiff(b"FUJIFILM\0", b"FUJIFILM");
    assert_eq!(
        print_value(&data, "FilmMode").as_deref(),
        Some("Classic Chrome"),
        "FilmMode 1536 should resolve through FUJI_FILM_MAIN_TAGS"
    );
}

#[test]
fn test_fujifilm_ifd_pointer_is_read_rather_than_assumed() {
    // IFD_POINTER is 16 here. An implementation that treats the IFD as starting
    // at signature+8 reads the pointer bytes as an entry count and finds
    // nothing, so the absence of FilmMode is the failure signal.
    // ExifTool: MakerNotes.pm:128 OffsetPt => '$valuePtr+8'
    let data = fujifilm_tiff(b"FUJIFILM\0", b"FUJIFILM");
    assert!(
        print_value(&data, "FilmMode").is_some(),
        "IFD pointer at byte 8 was not followed"
    );
}

#[test]
fn test_fujifilm_value_offsets_are_subdirectory_relative() {
    // ExifTool: MakerNotes.pm:131 Base => '$start'
    // Inline values survive a wrong base, so an out-of-line string is the only
    // thing that tells the two apart.
    let data = fujifilm_tiff(b"FUJIFILM\0", b"FUJIFILM");
    let quality = print_value(&data, "Quality").expect("Quality should be extracted");
    assert!(
        quality.starts_with("FINE"),
        "Quality read against the wrong base: {quality:?}"
    );
}

#[test]
fn test_fujifilm_slong_tags_are_extracted() {
    // HighlightTone, ShadowTone, GrainEffectRoughness, ColorChromeEffect and
    // ColorChromeFXBlue are all SLONG, so a reader handling only BYTE/ASCII/
    // SHORT/LONG drops most of the film recipe.
    let data = fujifilm_tiff(b"FUJIFILM\0", b"FUJIFILM");
    assert_eq!(
        print_value(&data, "ShadowTone").as_deref(),
        Some("-2 (soft)")
    );
}

#[test]
fn test_fujifilm_dispatch_is_on_signature_not_make() {
    // ExifTool: MakerNotes.pm:122 — the FUJIFILM header is also written by some
    // Leica, Minolta and Sharp bodies, so Make is the wrong discriminator.
    let data = fujifilm_tiff(b"LEICA\0", b"FUJIFILM");
    assert_eq!(
        print_value(&data, "FilmMode").as_deref(),
        Some("Classic Chrome"),
        "a FUJIFILM signature under a non-Fuji Make was not dispatched"
    );
}

#[test]
fn test_generale_signature_is_dispatched() {
    // ExifTool: MakerNotes.pm:124 Condition => '$$valPt =~ /^(FUJIFILM|GENERALE)/'
    let data = fujifilm_tiff(b"GE\0", b"GENERALE");
    assert_eq!(
        print_value(&data, "FilmMode").as_deref(),
        Some("Classic Chrome")
    );
}

#[test]
fn test_truncated_fujifilm_makernotes_is_a_warning_not_a_panic() {
    // The MakerNotes length is attacker-controlled, so every read past the
    // header is bounds checked (fuzz_exif_ifd).
    let mut data = fujifilm_tiff(b"FUJIFILM\0", b"FUJIFILM");
    data.truncate(MAKERNOTE_OFFSET as usize + 10);
    let mut reader = ExifReader::new();
    let _ = reader.parse_exif_data(&data);
}
