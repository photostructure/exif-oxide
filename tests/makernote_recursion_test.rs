//! Regression test for fuzz_exif_ifd stack overflow: a MakerNotes IFD whose
//! entries contain another MakerNotes tag (0x927C) pointing back at itself
//! recursed forever through parse_ifd -> parse_ifd_entry ->
//! process_maker_notes_with_signature_detection -> process_canon_makernotes ->
//! parse_ifd. The PROCESSED recursion guard only lived in
//! process_subdirectory, which the manufacturer MakerNotes dispatch bypasses.
//!
//! ExifTool: ProcessDirectory's $$self{PROCESSED} tracking catches this
//! (lib/Image/ExifTool.pm), because MakerNotes subdirectories route through
//! ProcessDirectory. Reproducer minimized from
//! fuzz/artifacts/fuzz_exif_ifd/crash-19018097e26099c5f0b6ea0a564055f15773c223.

use exif_oxide::exif::ExifReader;

/// Build a minimal little-endian TIFF whose IFD0 has the given Make (6 bytes
/// including NUL) and a MakerNotes (0x927C) entry pointing at an IFD that
/// contains another MakerNotes entry pointing back at the same IFD — a
/// self-referential cycle.
fn self_referential_makernote(make: &[u8; 6]) -> Vec<u8> {
    let mut d = Vec::new();
    // TIFF header: little-endian, magic 42, IFD0 at offset 8
    d.extend_from_slice(&[0x49, 0x49, 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00]);
    // IFD0: 2 entries
    d.extend_from_slice(&2u16.to_le_bytes());
    // Entry 1: Make (0x010F), ASCII, count 6, value at offset 38
    d.extend_from_slice(&0x010Fu16.to_le_bytes());
    d.extend_from_slice(&2u16.to_le_bytes());
    d.extend_from_slice(&6u32.to_le_bytes());
    d.extend_from_slice(&38u32.to_le_bytes());
    // Entry 2: MakerNote (0x927C), UNDEFINED, count 18, value at offset 46
    d.extend_from_slice(&0x927Cu16.to_le_bytes());
    d.extend_from_slice(&7u16.to_le_bytes());
    d.extend_from_slice(&18u32.to_le_bytes());
    d.extend_from_slice(&46u32.to_le_bytes());
    // Next IFD offset: none
    d.extend_from_slice(&0u32.to_le_bytes());
    // Offset 38: Make value
    d.extend_from_slice(make);
    // Offset 44: padding
    d.extend_from_slice(&[0, 0]);
    // Offset 46: MakerNotes IFD, 1 entry pointing back at offset 46
    d.extend_from_slice(&1u16.to_le_bytes());
    d.extend_from_slice(&0x927Cu16.to_le_bytes());
    d.extend_from_slice(&7u16.to_le_bytes());
    d.extend_from_slice(&18u32.to_le_bytes());
    d.extend_from_slice(&46u32.to_le_bytes());
    d.extend_from_slice(&0u32.to_le_bytes());
    assert_eq!(d.len(), 64);
    d
}

#[test]
fn test_self_referential_makernote_does_not_recurse_forever() {
    let data = self_referential_makernote(b"Canon\0");
    let mut reader = ExifReader::new();
    // Must terminate (the circular reference is skipped with a warning, per
    // ExifTool's PROCESSED handling), not overflow the stack.
    let _ = reader.parse_exif_data(&data);
}

#[test]
fn test_generic_makernote_is_not_skipped_as_circular() {
    // Regression for a bug in the first version of the recursion guard: it
    // registered the MakerNotes address in `processed` before dispatch, but
    // the generic (non-Canon/Olympus/Sony) path delegates to
    // process_subdirectory, which does its own PROCESSED check with the same
    // address (when base == 0) — so every generic MakerNotes directory was
    // wrongly skipped as a "circular reference" on first visit. The guard may
    // only pre-register for the manufacturer paths that bypass
    // process_subdirectory.
    let mut d = Vec::new();
    d.extend_from_slice(&[0x49, 0x49, 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00]);
    d.extend_from_slice(&2u16.to_le_bytes());
    // Make = "Nokon\0" — deliberately not a specially-dispatched manufacturer
    d.extend_from_slice(&0x010Fu16.to_le_bytes());
    d.extend_from_slice(&2u16.to_le_bytes());
    d.extend_from_slice(&6u32.to_le_bytes());
    d.extend_from_slice(&38u32.to_le_bytes());
    // MakerNote (0x927C), UNDEFINED, pointing at a benign 1-entry IFD
    d.extend_from_slice(&0x927Cu16.to_le_bytes());
    d.extend_from_slice(&7u16.to_le_bytes());
    d.extend_from_slice(&18u32.to_le_bytes());
    d.extend_from_slice(&46u32.to_le_bytes());
    d.extend_from_slice(&0u32.to_le_bytes());
    d.extend_from_slice(b"Nokon\0");
    d.extend_from_slice(&[0, 0]);
    // Offset 46: MakerNotes IFD with one harmless SHORT entry (tag 0x0001)
    d.extend_from_slice(&1u16.to_le_bytes());
    d.extend_from_slice(&0x0001u16.to_le_bytes());
    d.extend_from_slice(&3u16.to_le_bytes());
    d.extend_from_slice(&1u32.to_le_bytes());
    d.extend_from_slice(&42u32.to_le_bytes());
    d.extend_from_slice(&0u32.to_le_bytes());
    assert_eq!(d.len(), 64);

    let mut reader = ExifReader::new();
    let _ = reader.parse_exif_data(&d);
    // The first visit to a legitimate MakerNotes directory must not be
    // flagged as circular.
    assert!(
        !reader
            .get_warnings()
            .iter()
            .any(|w| w.contains("Circular reference")),
        "legitimate generic MakerNotes was skipped as circular: {:?}",
        reader.get_warnings()
    );
}
