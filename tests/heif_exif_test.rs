//! EXIF extraction from ISO BMFF containers (HEIF, HEIC, AVIF).
//!
//! Before this, a HEIF file produced dimensions and nothing else: the ispe box
//! was read, and the EXIF item sitting in the same meta box was never located.
//! A Fuji .HIF came back with 17 tags and no camera, lens or exposure at all.
//!
//! ExifTool: QuickTime.pm:9127-9195 (ParseItemLocation) and 9345-9483
//! (HandleItemInfo).

use exif_oxide::formats::{extract_exif_item, parse_iloc_box};

/// Build a minimal HEIF holding one Exif item.
///
/// `infe_version` selects the item-ID width, which is the detail that broke
/// every real file: versions 0, 1 and 2 store a 16-bit ID and only version 3
/// stores a 32-bit one (ExifTool: QuickTime.pm:9246-9256).
fn heif_with_exif(infe_version: u8, exif_payload: &[u8]) -> Vec<u8> {
    fn boxed(box_type: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&((body.len() + 8) as u32).to_be_bytes());
        b.extend_from_slice(box_type);
        b.extend_from_slice(body);
        b
    }

    const EXIF_ITEM_ID: u32 = 768;

    // infe: version, 3 flag bytes, item ID, protection index, type
    let mut infe = vec![infe_version, 0, 0, 0];
    if infe_version <= 2 {
        infe.extend_from_slice(&(EXIF_ITEM_ID as u16).to_be_bytes());
    } else {
        infe.extend_from_slice(&EXIF_ITEM_ID.to_be_bytes());
    }
    infe.extend_from_slice(&0u16.to_be_bytes()); // protection index
    infe.extend_from_slice(b"Exif");
    infe.push(0); // item name
    let infe = boxed(b"infe", &infe);

    // iinf version 0: 16-bit entry count, then the infe boxes
    let mut iinf = vec![0, 0, 0, 0];
    iinf.extend_from_slice(&1u16.to_be_bytes());
    iinf.extend_from_slice(&infe);
    let iinf = boxed(b"iinf", &iinf);

    // iloc version 1, offsets and lengths 4 bytes wide, no base offset.
    // The extent offset is absolute in the file, so the whole file has to be
    // laid out before it can be filled in.
    let mut iloc = vec![1u8, 0, 0, 0];
    iloc.extend_from_slice(&0x4400u16.to_be_bytes()); // offset 4, length 4, base 0, index 0
    iloc.extend_from_slice(&1u16.to_be_bytes()); // item count
    iloc.extend_from_slice(&(EXIF_ITEM_ID as u16).to_be_bytes());
    iloc.extend_from_slice(&0u16.to_be_bytes()); // construction method 0 (file offset)
    iloc.extend_from_slice(&0u16.to_be_bytes()); // data reference index
    iloc.extend_from_slice(&1u16.to_be_bytes()); // extent count
    let offset_patch_at = iloc.len();
    iloc.extend_from_slice(&0u32.to_be_bytes()); // extent offset, patched below
    iloc.extend_from_slice(&(exif_payload.len() as u32).to_be_bytes());
    let iloc = boxed(b"iloc", &iloc);

    // meta is a FullBox: 4 bytes of version and flags before its children
    let mut meta = vec![0, 0, 0, 0];
    meta.extend_from_slice(&iinf);
    let iloc_start_in_meta = meta.len();
    meta.extend_from_slice(&iloc);
    let meta = boxed(b"meta", &meta);

    let mut ftyp = Vec::new();
    ftyp.extend_from_slice(b"heix");
    ftyp.extend_from_slice(&0u32.to_be_bytes());
    ftyp.extend_from_slice(b"mif1heix");
    let ftyp = boxed(b"ftyp", &ftyp);

    let mut file = Vec::new();
    file.extend_from_slice(&ftyp);
    let meta_start = file.len();
    file.extend_from_slice(&meta);
    let payload_offset = file.len() as u32;
    file.extend_from_slice(exif_payload);

    // Patch the extent offset now that the payload's position is known.
    // iloc_start_in_meta is measured inside the meta BODY, which already
    // includes its 4 version/flags bytes, so only the two box headers are added.
    let patch = meta_start + 8 + iloc_start_in_meta + 8 + offset_patch_at;
    file[patch..patch + 4].copy_from_slice(&payload_offset.to_be_bytes());

    file
}

/// A TIFF holding a single IFD0 entry: Make = "FUJIFILM".
fn tiff_with_make() -> Vec<u8> {
    let mut d = Vec::new();
    d.extend_from_slice(&[0x49, 0x49, 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00]);
    d.extend_from_slice(&1u16.to_le_bytes());
    d.extend_from_slice(&0x010Fu16.to_le_bytes());
    d.extend_from_slice(&2u16.to_le_bytes());
    d.extend_from_slice(&9u32.to_le_bytes());
    d.extend_from_slice(&26u32.to_le_bytes());
    d.extend_from_slice(&0u32.to_le_bytes());
    d.extend_from_slice(b"FUJIFILM\0");
    d
}

/// The usual payload shape: a 4-byte big-endian skip count, then "Exif\0\0",
/// then the TIFF header. ExifTool: QuickTime.pm:9471-9473.
fn exif_payload(tiff: &[u8]) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&6u32.to_be_bytes());
    payload.extend_from_slice(b"Exif\0\0");
    payload.extend_from_slice(tiff);
    payload
}

#[test]
fn test_exif_item_is_found_in_a_heif() {
    let file = heif_with_exif(2, &exif_payload(&tiff_with_make()));
    let item = extract_exif_item(&file)
        .expect("parse")
        .expect("an Exif item");
    assert!(
        item.tiff_data.starts_with(b"II\x2a\0"),
        "payload header was not skipped: {:?}",
        &item.tiff_data[..4.min(item.tiff_data.len())]
    );
    assert_eq!(
        item.tiff_offset as usize + item.tiff_data.len(),
        file.len(),
        "tiff_offset must be the payload's real position in the file"
    );
}

#[test]
fn test_infe_version_2_uses_a_16_bit_item_id() {
    // ExifTool: QuickTime.pm:9246-9256. Reading version 2 as 32-bit is what made
    // every real HEIC and HEIF fail: the entry is 13 bytes of content, a 32-bit
    // read needs 14, so the item was discarded and no Exif item was ever seen.
    let file = heif_with_exif(2, &exif_payload(&tiff_with_make()));
    assert!(
        extract_exif_item(&file).expect("parse").is_some(),
        "a version 2 infe entry was not read"
    );
}

#[test]
fn test_infe_version_3_uses_a_32_bit_item_id() {
    let file = heif_with_exif(3, &exif_payload(&tiff_with_make()));
    assert!(
        extract_exif_item(&file).expect("parse").is_some(),
        "a version 3 infe entry was not read"
    );
}

#[test]
fn test_payload_without_the_exif_header_is_accepted() {
    // ExifTool: QuickTime.pm:9463-9464 warns "Missing Exif header" and treats
    // the payload as starting at the TIFF header.
    let file = heif_with_exif(2, &tiff_with_make());
    let item = extract_exif_item(&file)
        .expect("parse")
        .expect("an Exif item");
    assert!(item.tiff_data.starts_with(b"II\x2a\0"));
}

#[test]
fn test_a_file_with_no_exif_item_is_not_an_error() {
    // An AVIF with no EXIF is ordinary, so this reports absence rather than
    // failing the whole extraction.
    let mut file = heif_with_exif(2, &exif_payload(&tiff_with_make()));
    // Turn the item type into something that is not Exif.
    let pos = file
        .windows(4)
        .position(|w| w == b"Exif")
        .expect("the item type");
    file[pos..pos + 4].copy_from_slice(b"hvc1");
    assert!(extract_exif_item(&file).expect("parse").is_none());
}

#[test]
fn test_iloc_field_widths_come_from_the_box() {
    // The nibbles at bytes 4..6 give the byte width of the offset, length,
    // base-offset and index fields, so the stride is per-file.
    // ExifTool: QuickTime.pm:9142-9146
    let mut iloc = vec![1u8, 0, 0, 0];
    iloc.extend_from_slice(&0x4400u16.to_be_bytes());
    iloc.extend_from_slice(&1u16.to_be_bytes());
    iloc.extend_from_slice(&7u16.to_be_bytes()); // item ID
    iloc.extend_from_slice(&0u16.to_be_bytes()); // construction method
    iloc.extend_from_slice(&0u16.to_be_bytes()); // data reference index
    iloc.extend_from_slice(&1u16.to_be_bytes()); // extent count
    iloc.extend_from_slice(&0x1234u32.to_be_bytes());
    iloc.extend_from_slice(&0x40u32.to_be_bytes());

    let locations = parse_iloc_box(&iloc).expect("parse");
    assert_eq!(locations.len(), 1);
    assert_eq!(locations[0].item_id, 7);
    assert_eq!(locations[0].extents.len(), 1);
    assert_eq!(locations[0].extents[0].offset, 0x1234);
    assert_eq!(locations[0].extents[0].length, 0x40);
}

#[test]
fn test_truncated_boxes_do_not_panic() {
    let file = heif_with_exif(2, &exif_payload(&tiff_with_make()));
    for cut in [12, 30, 48, 70, file.len() - 4] {
        let _ = extract_exif_item(&file[..cut]);
    }
    let _ = parse_iloc_box(&[1, 0, 0, 0, 0x44, 0x00, 0, 1]);
}
