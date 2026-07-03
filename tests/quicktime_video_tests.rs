//! QuickTime/MOV video read compatibility tests
//!
//! Reading any QuickTime container video currently yields ONLY `File:` group tags
//! because no atom walker exists (`src/formats/mod.rs` falls into the "MOV not yet
//! supported" arm). These tests pin the values ExifTool produces for the supported
//! `QuickTime:*` slice plus the video-driven Composites, so they will stay RED until
//! the walker (TPP tasks 2-4) lands and then guard against regressions.
//!
//! Ground truth = the committed reference snapshots in generated/exiftool-json/:
//!   test-images/apple/IMG_3755.MOV         -> test_images_apple_IMG_3755_MOV.json
//!   test-images/canon/eos_500d.mov         -> test_images_canon_eos_500d_mov.json
//!   third-party/exiftool/t/images/QuickTime.mov -> third_party_exiftool_t_images_QuickTime_mov.json
//!
//! Snapshots were generated WITHOUT `-api QuickTimeUTC` (dates are gmtime, no TZ
//! suffix on the binary mvhd/tkhd/mdhd dates) and in numeric GPS mode
//! (`-GPSLatitude# ...`), so Composite GPS lat/lon/alt are JSON numbers while
//! GPSAltitudeRef is the PrintConv string "Above Sea Level".
//! See tools/generate_exiftool_json.sh and QuickTime.pm.

#![cfg(feature = "integration-tests")]

use exif_oxide::extract_metadata_json;
use serde_json::{json, Value};

fn tags_for(path: &str) -> Value {
    extract_metadata_json(path).unwrap_or_else(|e| panic!("extract failed for {path}: {e}"))
}

fn assert_tag(tags: &Value, key: &str, expected: Value) {
    let actual = tags
        .get(key)
        .unwrap_or_else(|| panic!("missing tag {key} (extraction succeeded but key not present)"));
    assert_eq!(actual, &expected, "tag {key} mismatch");
}

/// IMG_3755.MOV carries Apple Keys/ItemList metadata + GPS, so it exercises every
/// supported QuickTime tag plus all the video-driven Composites.
#[test]
#[ignore = "RED until the QuickTime atom walker lands (TPP 20260703-P1-quicktime-video-read Tasks 2-4); validated to fail on missing tags 2026-07-03"]
fn apple_img3755_quicktime_and_composite_tags_match_exiftool() {
    // Snapshot: generated/exiftool-json/test_images_apple_IMG_3755_MOV.json
    let tags = tags_for("test-images/apple/IMG_3755.MOV");

    // --- 17 supported QuickTime:* tags ---
    // mvhd (MovieHeader, QuickTime.pm:1343): dates are gmtime, Duration via TimeScale.
    assert_tag(&tags, "QuickTime:CreateDate", json!("2025:06:24 22:24:45"));
    assert_tag(&tags, "QuickTime:ModifyDate", json!("2025:06:24 22:24:47"));
    assert_tag(&tags, "QuickTime:Duration", json!("2.96 s"));
    // tkhd (TrackHeader, QuickTime.pm:1493): first (video) track wins (Priority=>0).
    assert_tag(
        &tags,
        "QuickTime:TrackCreateDate",
        json!("2025:06:24 22:24:45"),
    );
    assert_tag(
        &tags,
        "QuickTime:TrackModifyDate",
        json!("2025:06:24 22:24:47"),
    );
    assert_tag(&tags, "QuickTime:TrackDuration", json!("2.96 s"));
    assert_tag(&tags, "QuickTime:ImageWidth", json!(1920));
    assert_tag(&tags, "QuickTime:ImageHeight", json!(1440));
    // stsd/VisualSampleDesc (QuickTime.pm:7585) CompressorName idx 25.
    assert_tag(&tags, "QuickTime:CompressorName", json!("H.264"));
    // mdhd (MediaHeader, QuickTime.pm:7239): MediaDuration via MediaTS (LAST track wins).
    assert_tag(
        &tags,
        "QuickTime:MediaCreateDate",
        json!("2025:06:24 22:24:45"),
    );
    assert_tag(
        &tags,
        "QuickTime:MediaModifyDate",
        json!("2025:06:24 22:24:47"),
    );
    assert_tag(&tags, "QuickTime:MediaDuration", json!("0.00 s"));
    // hdlr (Handler, QuickTime.pm:8391): last hdlr wins (default priority).
    assert_tag(
        &tags,
        "QuickTime:HandlerDescription",
        json!("Core Media Data Handler"),
    );
    // Keys/ItemList indirection (ProcessKeys, QuickTime.pm:9779).
    assert_tag(&tags, "QuickTime:Make", json!("Apple"));
    assert_tag(&tags, "QuickTime:Model", json!("iPhone 13 Pro"));
    // Software is a JSON number in the snapshot (18.5), NOT a string.
    assert_tag(&tags, "QuickTime:Software", json!(18.5));
    // CreationDate (Keys creationdate:6683, %iso8601Date) keeps local-with-TZ form.
    assert_tag(
        &tags,
        "QuickTime:CreationDate",
        json!("2025:06:24 15:24:45-07:00"),
    );

    // --- Video-driven Composites ---
    assert_tag(&tags, "Composite:ImageSize", json!("1920x1440"));
    assert_tag(&tags, "Composite:Megapixels", json!(2.8));
    // Rotation via CalcRotation (QuickTime.pm:8797) from the vide track's matrix.
    assert_tag(&tags, "Composite:Rotation", json!(90));
    // GPS from location.ISO6709 (ConvertISO6709:8884), numeric mode -> JSON numbers.
    assert_tag(&tags, "Composite:GPSLatitude", json!(37.5044));
    assert_tag(&tags, "Composite:GPSLongitude", json!(-122.4763));
    assert_tag(&tags, "Composite:GPSAltitude", json!(25.247));
    // GPSAltitudeRef is a PrintConv string even in numeric mode.
    assert_tag(&tags, "Composite:GPSAltitudeRef", json!("Above Sea Level"));
    assert_tag(&tags, "Composite:GPSPosition", json!("37.5044 -122.4763"));
}

/// eos_500d.mov has NO Apple Keys metadata: proves the walker decodes mvhd/tkhd
/// (dates, duration, dimensions) without depending on ItemList/Keys.
#[test]
#[ignore = "RED until the QuickTime atom walker lands (TPP 20260703-P1-quicktime-video-read Tasks 2-4); validated to fail on missing tags 2026-07-03"]
fn canon_eos500d_mov_core_tags_match_exiftool() {
    // Snapshot: generated/exiftool-json/test_images_canon_eos_500d_mov.json
    let tags = tags_for("test-images/canon/eos_500d.mov");

    assert_tag(&tags, "QuickTime:CreateDate", json!("2009:05:11 13:08:49"));
    assert_tag(&tags, "QuickTime:Duration", json!("7.50 s"));
    assert_tag(&tags, "QuickTime:ImageWidth", json!(1920));
}

/// The tiny ExifTool test fixture (3871 bytes, no Keys atoms) is another
/// Apple-metadata-free proof of the core mvhd/tkhd path.
#[test]
#[ignore = "RED until the QuickTime atom walker lands (TPP 20260703-P1-quicktime-video-read Tasks 2-4); validated to fail on missing tags 2026-07-03"]
fn exiftool_quicktime_mov_core_tags_match_exiftool() {
    // Snapshot: generated/exiftool-json/third_party_exiftool_t_images_QuickTime_mov.json
    let tags = tags_for("third-party/exiftool/t/images/QuickTime.mov");

    assert_tag(&tags, "QuickTime:CreateDate", json!("2005:08:11 14:03:54"));
    assert_tag(&tags, "QuickTime:Duration", json!("4.97 s"));
    assert_tag(&tags, "QuickTime:ImageWidth", json!(320));
}

// ---------------------------------------------------------------------------
// Unit-test targets for later tasks. The implementation modules
// (src/implementations/quicktime.rs, GPS ISO6709 conversion, ConvertDuration)
// do NOT exist yet, so these are #[ignore]d stubs that document the exact cases
// to cover. Un-ignore and wire to the real functions as each task lands, rather
// than leaving compile errors referencing modules that don't exist.
// ---------------------------------------------------------------------------

/// Task 4: port ConvertISO6709 (QuickTime.pm:8884) exactly.
///
/// ```text
/// Three regex forms (with Perl `+0` numification):
///   1. "+37.5044-122.4763+025.247/"        -> lat 37.5044, lon -122.4763, alt 25.247
///   2. "+3730.264-12228.578/"  (DDMM.M...) -> DD + MM/60 per component
///   3. "+373015.84-1221334.68/" (DDMMSS..) -> DD + MM/60 + SS/3600 per component
/// Composite split (8668-8696):
///   GPSAltitude = abs(alt); GPSAltitudeRef = alt < 0 ? "Below Sea Level" : "Above Sea Level".
/// ```
#[test]
#[ignore = "Task 4: ConvertISO6709 / GPS composite split not implemented yet"]
fn convert_iso6709_three_forms() {
    todo!("wire to src/implementations/quicktime.rs ConvertISO6709 port");
}

/// Task 2: the %timeInfo RawConv (QuickTime.pm:243-293) patches "brain-dead
/// software" that writes 1970-epoch instead of 1904-epoch dates: subtract
/// 2082844800 ONLY when `$val >= 2082844800`, else warn and pass through.
#[test]
#[ignore = "Task 2: %timeInfo 1970-epoch RawConv patch not implemented yet"]
fn timeinfo_1970_epoch_patch() {
    todo!("wire to src/implementations/quicktime.rs %timeInfo RawConv port");
}

/// Task 2: ConvertDuration (ExifTool.pm:6877) renders durations < 60 s as
/// e.g. "2.96 s" after ValueConv divides raw ticks by TimeScale
/// (%durationInfo, QuickTime.pm:314-317).
#[test]
#[ignore = "Task 2: ConvertDuration not implemented yet"]
fn convert_duration_seconds() {
    todo!("wire to src/implementations/quicktime.rs ConvertDuration port");
}
