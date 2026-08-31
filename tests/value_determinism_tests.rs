//! Regression tests for value-level nondeterminism across identical runs.
//!
//! Repeated runs of the same binary on the same file must return identical tag
//! VALUES (tracked in _todo/20260830-P1-nondeterministic-output.md). The flips
//! were driven by HashMap iteration order, whose seed differs per *process* —
//! in-process repetition alone cannot reproduce them. Each test therefore
//! spawns the compiled CLI several times and asserts both cross-run stability
//! and the ExifTool-correct pinned value; an in-process check via the library
//! API is included as well.
//!
//! ExifTool references for the pinned values:
//! - Composite:FocalLength35efl must be built AFTER Composite:ScaleFactor35efl
//!   (BuildCompositeTags defers a composite that Desire's an unbuilt Composite;
//!   lib/Image/ExifTool.pm:3973-3975, 4074-4078), so the 35mm scale factor is
//!   always applied: 5.7mm * 4.5614 = 26.
//! - MakerNotes:FileNumber comes only from Canon Main tag 0x8
//!   (lib/Image/ExifTool/Canon.pm:1261-1267); tag 0x1 is the CanonCameraSettings
//!   SubDirectory (Canon.pm:1226-1232) and contributes no FileNumber.

#![cfg(feature = "integration-tests")]

use std::path::Path;
use std::process::Command;

/// Number of CLI invocations per test. Pre-fix, the bad value appeared in
/// roughly half of all processes, so 8 runs make a silent pass vanishingly
/// unlikely while staying fast.
const RUNS: usize = 8;

/// Run the CLI on `file` and return the first (only) JSON object.
fn run_cli(file: &str) -> serde_json::Value {
    let output = Command::new(env!("CARGO_BIN_EXE_exif-oxide"))
        .arg(file)
        .output()
        .expect("failed to spawn exif-oxide binary");
    assert!(
        output.status.success(),
        "exif-oxide exited with {:?} on {}",
        output.status,
        file
    );
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("binary emitted invalid JSON");
    parsed
        .as_array()
        .and_then(|a| a.first())
        .expect("expected a one-element JSON array")
        .clone()
}

/// Collect `key` from RUNS separate CLI processes.
fn collect_values(file: &str, key: &str) -> Vec<serde_json::Value> {
    (0..RUNS)
        .map(|_| {
            run_cli(file)
                .get(key)
                .unwrap_or(&serde_json::Value::Null)
                .clone()
        })
        .collect()
}

fn assert_stable_and_pinned(file: &str, key: &str, expected: &serde_json::Value) {
    let values = collect_values(file, key);
    for (i, v) in values.iter().enumerate() {
        assert_eq!(
            v, expected,
            "{key} on {file}: run {i} returned {v} (expected {expected}); all runs: {values:?}"
        );
    }
}

#[test]
fn composite_focal_length_35efl_applies_scale_factor_every_run() {
    let file = "test-images/apple/iphone_13_pro.jpg";
    assert!(Path::new(file).exists(), "missing test asset {file}");
    // ExifTool: "5.7 mm (35 mm equivalent: 26.0 mm)" — the 35efl value is 26.
    // Pre-fix this flipped between 26 and the unscaled 5.7 depending on whether
    // ScaleFactor35efl happened to be built before FocalLength35efl.
    assert_stable_and_pinned(file, "Composite:FocalLength35efl", &serde_json::json!(26));
}

#[test]
fn canon_file_number_not_overwritten_by_camera_settings_entry() {
    let file = "test-images/canon/powershot_s110.jpg";
    assert!(Path::new(file).exists(), "missing test asset {file}");
    // ExifTool: MakerNotes:FileNumber = "130-0112" (Canon Main tag 0x8 value
    // 1300112). Pre-fix a bogus FileNumber derived from CanonCameraSettings
    // entry 8 (65535 -> "6-5535") raced with the real one.
    assert_stable_and_pinned(
        file,
        "MakerNotes:FileNumber",
        &serde_json::json!("130-0112"),
    );
}

/// Two XMP properties mapping to the same output name must resolve the way
/// ExifTool's FoundTag does (lib/Image/ExifTool.pm:9514-9585). Both the
/// tiff and exif namespace tables carry a table-level `PRIORITY => 0`
/// (XMP.pm:1900, XMP.pm:1992), so both NativeDigest properties have priority
/// 0 and the FIRST one in document order keeps the base name (an existing
/// 0-priority tag is promoted to 1 at ExifTool.pm:9544-9551, and the
/// 0-priority newcomer loses the >= test at :9564).
///
/// Both pins verified against the vendored exiftool:
///   exiftool -j -G test-resources/native-digest-collision.xmp
///     -> exif value (exif element appears first)
///   exiftool -j -G test-resources/native-digest-collision-reversed.xmp
///     -> tiff value (tiff element appears first)
///
/// Each extraction builds fresh HashMaps (per-instance hash seeds), so the
/// pre-fix winner flipped WITHIN a process; 20 in-process iterations suffice.
#[test]
fn xmp_native_digest_resolves_duplicates_like_exiftool() {
    let cases = [
        (
            "test-resources/native-digest-collision.xmp",
            "36864,40960;EXIFDIGEST",
        ),
        (
            "test-resources/native-digest-collision-reversed.xmp",
            "256,257;TIFFDIGEST",
        ),
    ];
    for (file, expected) in cases {
        assert!(Path::new(file).exists(), "missing fixture {file}");
        for i in 0..20 {
            let mut exif_data =
                exif_oxide::formats::extract_metadata(Path::new(file), false, false, None)
                    .unwrap_or_else(|e| panic!("extract_metadata failed on {file}: {e}"));
            exif_data.prepare_for_serialization(None);
            let json = serde_json::to_value(&exif_data).expect("serialization failed");
            let got = json
                .get("XMP:NativeDigest")
                .unwrap_or(&serde_json::Value::Null);
            assert_eq!(
                got,
                &serde_json::json!(expected),
                "iteration {i} on {file}: first property in document order must win"
            );
        }
    }
}

/// A tag whose table key differs from its display name must still contribute
/// its priority metadata: dc:source (table key "source", stored as "Source")
/// carries `Avoid => 1` (XMP.pm:1034, priority 0) while photoshop:Source
/// (XMP.pm:1314) has the default priority 1 — so photoshop wins even when its
/// element comes FIRST and dc's comes second (a naive last-wins would pick
/// dc). Verified: `exiftool -j -G test-resources/source-collision.xmp`
/// reports "PSSOURCE" for both document orders.
#[test]
fn xmp_source_priority_survives_display_name_storage() {
    let file = "test-resources/source-collision.xmp";
    assert!(Path::new(file).exists(), "missing fixture {file}");
    for i in 0..20 {
        let mut exif_data =
            exif_oxide::formats::extract_metadata(Path::new(file), false, false, None)
                .unwrap_or_else(|e| panic!("extract_metadata failed on {file}: {e}"));
        exif_data.prepare_for_serialization(None);
        let json = serde_json::to_value(&exif_data).expect("serialization failed");
        let got = json.get("XMP:Source").unwrap_or(&serde_json::Value::Null);
        assert_eq!(
            got,
            &serde_json::json!("PSSOURCE"),
            "iteration {i}: photoshop:Source must beat Avoid'd dc:source"
        );
    }
}

/// Same Avoid rule on a real camera file (the originally recorded repro):
/// the embedded XMP carries both exif:NativeDigest and tiff:NativeDigest.
/// ExifTool (`exiftool -j -G test-images/canon/eos_1ds_mark_ii.jpg`) reports
/// the exif one; pre-fix our output flipped between the two per process.
#[test]
fn xmp_native_digest_stable_on_real_file() {
    let file = "test-images/canon/eos_1ds_mark_ii.jpg";
    assert!(Path::new(file).exists(), "missing test asset {file}");
    let expected = serde_json::json!(
        "36864,40960,40961,37121,37122,40962,40963,37510,40964,36867,36868,33434,33437,\
         34850,34852,34855,34856,37377,37378,37379,37380,37381,37382,37383,37384,37385,\
         37386,37396,41483,41484,41486,41487,41488,41492,41493,41495,41728,41729,41730,\
         41985,41986,41987,41988,41989,41990,41991,41992,41993,41994,41995,41996,42016,\
         0,2,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,20,22,23,24,25,26,27,28,30;\
         71B6AB26895A9BAA5A9923A697E7E3D2"
    );
    assert_stable_and_pinned(file, "XMP:NativeDigest", &expected);
}

/// In-process check via the library API: repeated extraction in one process
/// must agree with the pinned values too (guards non-seed-related ordering).
#[test]
fn in_process_extraction_is_stable_and_correct() {
    let cases: &[(&str, &str, serde_json::Value)] = &[
        (
            "test-images/apple/iphone_13_pro.jpg",
            "Composite:FocalLength35efl",
            serde_json::json!(26),
        ),
        (
            "test-images/canon/powershot_s110.jpg",
            "MakerNotes:FileNumber",
            serde_json::json!("130-0112"),
        ),
    ];
    for _ in 0..3 {
        for (file, key, expected) in cases {
            let mut exif_data =
                exif_oxide::formats::extract_metadata(Path::new(file), false, false, None)
                    .unwrap_or_else(|e| panic!("extract_metadata failed on {file}: {e}"));
            exif_data.prepare_for_serialization(None);
            let json = serde_json::to_value(&exif_data).expect("serialization failed");
            let got = json.get(*key).unwrap_or(&serde_json::Value::Null);
            assert_eq!(
                got, expected,
                "{key} on {file} (in-process) returned {got}, expected {expected}"
            );
        }
    }
}
