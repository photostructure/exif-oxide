#![no_main]
//! Secondary, lower-frequency whole-file target (Option B).
//!
//! Drives the top-level `extract_metadata` dispatch: format detection plus
//! whichever format parser detection selects. This is the only target that can
//! catch detection/dispatch bugs (e.g. bytes detected as one format but routed
//! into another parser). It is deliberately slower per iteration because
//! `extract_metadata` takes a `&Path`, so each input is written to a tempfile
//! first — keep its CI time budget small so it never starves the fast
//! in-memory per-format targets.
use exif_oxide::formats::extract_metadata;
use libfuzzer_sys::fuzz_target;
use std::io::Write;
use tempfile::NamedTempFile;

fuzz_target!(|data: &[u8]| {
    let Ok(mut tmp) = NamedTempFile::new() else {
        return;
    };
    if tmp.write_all(data).is_err() {
        return;
    }
    if tmp.flush().is_err() {
        return;
    }
    let _ = extract_metadata(tmp.path(), false, false, None);
});
