//! XMP value-conversion compatibility tests
//!
//! XMP tags are stored in the packet as raw RDF text (e.g. `FNumber` = "8/1",
//! `DateTimeOriginal` = "2005-06-08T12:05:36+01:00"). ExifTool applies two layers
//! of read-time conversion (XMP.pm):
//!
//!  1. Format-driven, keyed on `Writable` (FoundXMP, XMP.pm:3673-3687):
//!     `rational` -> ConvertRational (XMP.pm:3400-3417),
//!     `date`     -> ConvertXMPDate  (XMP.pm:3383-3394).
//!  2. Per-tag ValueConv/PrintConv for the `exif` namespace photo cluster
//!     (XMP.pm:2042-2166): ShutterSpeedValue/ApertureValue APEX conversions,
//!     FNumber/ExposureTime/FocalLength formatting.
//!
//! These tests assert the fully-converted values ExifTool produces, taken from the
//! reference snapshots in generated/exiftool-json/. Ground truth for the two images:
//!   test-images/canon/eos_1ds_mark_ii.jpg  (rational + exif cluster + dates)
//!   test-images/apple/iphone_x.jpg         (GPS rational + date w/ fractional secs)

use exif_oxide::extract_metadata_json;
use serde_json::{json, Value};

fn tags_for(path: &str) -> Value {
    extract_metadata_json(path).unwrap_or_else(|e| panic!("extract failed for {path}: {e}"))
}

fn assert_tag(tags: &Value, key: &str, expected: Value) {
    let actual = tags
        .get(key)
        .unwrap_or_else(|| panic!("missing tag {key} (present: extraction succeeded but no key)"));
    assert_eq!(actual, &expected, "tag {key} mismatch");
}

#[test]
fn canon_xmp_exif_cluster_matches_exiftool() {
    // Snapshot: generated/exiftool-json/test_images_canon_eos_1ds_mark_ii_jpg.json
    let tags = tags_for("test-images/canon/eos_1ds_mark_ii.jpg");

    // Layer 2: ValueConv + PrintConv (exif namespace)
    assert_tag(&tags, "XMP:FNumber", json!(8.0)); // "8/1" -> 8 -> PrintFNumber -> 8.0
    assert_tag(&tags, "XMP:ApertureValue", json!(8.0)); // "6/1" -> sqrt(2)**6 -> 8.0
    assert_tag(&tags, "XMP:ShutterSpeedValue", json!("1/320")); // "8321928/1000000" -> 1/(2**val) -> "1/320"
    assert_tag(&tags, "XMP:ExposureTime", json!("1/320")); // "1/320" -> 0.003125 -> "1/320"
    assert_tag(&tags, "XMP:FocalLength", json!("38.0 mm")); // "38/1" -> 38 -> "38.0 mm"

    // Layer 1: date reformat (exif + xmp namespaces)
    assert_tag(
        &tags,
        "XMP:DateTimeOriginal",
        json!("2005:06:08 12:05:36+01:00"),
    );
    assert_tag(&tags, "XMP:CreateDate", json!("2005:08:11 09:37:58+01:00"));
    assert_tag(&tags, "XMP:ModifyDate", json!("2005:08:11 09:38:38+01:00"));
}

#[test]
fn iphone_xmp_gps_and_date_match_exiftool() {
    // Snapshot: generated/exiftool-json/test_images_apple_iphone_x_jpg.json
    let tags = tags_for("test-images/apple/iphone_x.jpg");

    // GPSAltitude: rational Layer 1 only, numeric mode (no " m"): "123/10" -> 12.3
    assert_tag(&tags, "XMP:GPSAltitude", json!(12.3));
    // GPSDateTime (renamed GPSTimeStamp): date Layer 1, trailing "Z" preserved
    assert_tag(&tags, "XMP:GPSDateTime", json!("2018:01:15 22:22:01Z"));
    // CreateDate: date Layer 1 with fractional seconds preserved verbatim
    assert_tag(&tags, "XMP:CreateDate", json!("2018:01:15 14:22:02.185"));
}
