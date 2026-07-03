//! Snapshot-oracle tests: assert exif-oxide output matches the committed ExifTool
//! JSON snapshots in `generated/exiftool-json/` for specific, high-value tags.
//!
//! These snapshots are produced by ExifTool with the numeric (`-Tag#`) flags used by
//! the compat suite (see `src/compat/mod.rs`), so string-valued tags like
//! `Composite:GPSPosition` carry ExifTool's exact Perl `%.15g` stringification and sign.
//!
//! Unlike the numeric-normalizing compat comparison, these assertions are byte-exact,
//! which is what catches sign/precision divergences in composite string tags.

use serde_json::Value;
use std::fs;

/// Load a committed ExifTool snapshot object for a test image.
fn load_snapshot(snapshot_name: &str) -> Value {
    let path = format!("generated/exiftool-json/{snapshot_name}");
    let contents =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read snapshot {path}: {e}"));
    serde_json::from_str(&contents)
        .unwrap_or_else(|e| panic!("failed to parse snapshot {path}: {e}"))
}

/// Extract a single tag's value from an exif-oxide result / ExifTool snapshot object.
fn tag<'a>(obj: &'a Value, key: &str) -> Option<&'a Value> {
    obj.as_object().and_then(|m| m.get(key))
}

/// Composite:GPSPosition must match ExifTool byte-for-byte, including the west/south
/// negative sign and Perl's `%.15g` numeric formatting.
///
/// Regression: exif-oxide previously emitted the two UNSIGNED EXIF coordinates joined with
/// Rust's default f64 Display, e.g. "40.59359722222222 122.38015" instead of ExifTool's
/// "40.5935972222222 -122.38015" (dropped longitude sign + one extra digit of precision).
#[test]
fn gps_position_matches_exiftool_snapshot() {
    let cases = [
        (
            "test-images/apple/iphone_13_pro.jpg",
            "test_images_apple_iphone_13_pro_jpg.json",
        ),
        (
            "test-images/apple/IMG_3755.JPG",
            "test_images_apple_IMG_3755_JPG.json",
        ),
    ];

    for (image, snapshot_name) in cases {
        let snapshot = load_snapshot(snapshot_name);
        let expected = tag(&snapshot, "Composite:GPSPosition")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("snapshot {snapshot_name} missing Composite:GPSPosition"));

        let actual_json = exif_oxide::extract_metadata_json(image)
            .unwrap_or_else(|e| panic!("extract_metadata_json({image}) failed: {e}"));
        let actual = tag(&actual_json, "Composite:GPSPosition")
            .and_then(Value::as_str)
            .unwrap_or_else(|| {
                panic!("exif-oxide output for {image} missing Composite:GPSPosition")
            });

        assert_eq!(
            actual, expected,
            "Composite:GPSPosition mismatch for {image}"
        );
    }
}
