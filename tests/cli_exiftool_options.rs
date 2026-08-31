//! Classic-argv conformance with ExifTool's option handling.
//!
//! These tests are deliberately asset-free (they build their own temp files)
//! and carry no feature gate so CI always runs them.
//!
//! The consumer contract they protect is exiftool-vendored.js:
//! - `VersionTask.ts:7` requires `-ver` output to match
//!   `/^\d{1,3}\.\d{1,3}(?:\.\d{1,3})?$/`.
//! - `ReadTask.ts` sends option/value pairs (`-api <val>`, `-use MWG`, ...)
//!   that must never be mistaken for file paths: a phantom JSON entry with
//!   `SourceFile: "Filter=..."` makes ReadTask throw
//!   (`ReadTask.ts:192-197` rejects unexpected SourceFile values).

use std::io::Write;
use std::process::Command;

fn exif_oxide() -> Command {
    Command::new(env!("CARGO_BIN_EXE_exif-oxide"))
}

/// Create a temp file with unrecognizable content; extraction may fail, but
/// the JSON array must still contain exactly one entry for this path.
fn scratch_file() -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().expect("create temp file");
    f.write_all(&[0x00, 0x01, 0x02, 0x03])
        .expect("write temp file");
    f
}

/// `-ver` must print the emulated ExifTool version, not the crate version.
/// ExifTool: exiftool:14 (`my $version = '13.59';`), printed at exiftool:779-793.
#[test]
fn test_ver_prints_exiftool_version_constant() {
    let out = exif_oxide().arg("-ver").output().expect("run -ver");
    assert!(out.status.success(), "-ver must exit 0");
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert_eq!(
        stdout,
        format!("{}\n", exif_oxide::EXIFTOOL_VERSION),
        "-ver must print the ExifTool compatibility version"
    );
    // exiftool-vendored.js VersionTask.ts:7
    let version = stdout.trim();
    let parts: Vec<&str> = version.split('.').collect();
    assert!(
        (2..=3).contains(&parts.len())
            && parts
                .iter()
                .all(|p| !p.is_empty() && p.len() <= 3 && p.bytes().all(|b| b.is_ascii_digit())),
        "-ver output {version:?} must match VersionTask's /^\\d{{1,3}}\\.\\d{{1,3}}(?:\\.\\d{{1,3}})?$/"
    );
}

/// Option values (the exiftool-vendored.js default ReadTask payload) must not
/// be treated as file paths. ExifTool skips option values via %optArgs
/// (exiftool:260-300).
#[test]
fn test_option_values_do_not_become_files() {
    let f = scratch_file();
    let path = f.path().to_str().unwrap().to_string();
    // Subset of the real ReadTask payload (ReadTask.ts:118-158), stdin framing aside.
    let out = exif_oxide()
        .args([
            "-json",
            "-fast",
            "-api",
            "Filter=if (Image::ExifTool::IsUTF8(\\$_) < 0) { }",
            "-api",
            "struct=1",
            "-use",
            "MWG",
            "-api",
            "keepUTCTime",
            "-all",
            &path,
            "-ignoreMinorErrors",
        ])
        .output()
        .expect("run exif-oxide");
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout must be JSON ({e}): {stdout:?}"));
    let entries = json.as_array().expect("JSON array");
    assert_eq!(
        entries.len(),
        1,
        "exactly one entry (option values must not become phantom files): {json}"
    );
    assert_eq!(
        entries[0]["SourceFile"].as_str(),
        Some(path.as_str()),
        "SourceFile must echo the requested path"
    );
}

/// ExifTool flags the consumer sends (or that -G-mode emulation promises to
/// tolerate) are accepted as no-ops instead of "Unknown option" exits.
#[test]
fn test_compat_flags_accepted_as_noops() {
    let f = scratch_file();
    let path = f.path().to_str().unwrap().to_string();
    let out = exif_oxide()
        .args([
            "-j",
            "-json",
            "-G",
            "-G1",
            "-g",
            "-struct",
            "-q",
            "-a",
            "-e",
            "-m",
            "-ignoreMinorErrors",
            "-fast",
            "-fast2",
            "-duplicates",
            "-all",
            &path,
        ])
        .output()
        .expect("run exif-oxide");
    assert!(
        out.status.success(),
        "compat flags must not be rejected; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).expect("JSON stdout");
    assert_eq!(json.as_array().expect("array").len(), 1);
}

/// `-x TAG` (exclusion) is accepted and ignored in M1a (decision 5,
/// _todo/20260830-P1-stay-open-m1a.md); its value must not become a file.
#[test]
fn test_exclude_value_consumed() {
    let f = scratch_file();
    let path = f.path().to_str().unwrap().to_string();
    let out = exif_oxide()
        .args(["-x", "Composite:ImageSize", "-all", &path])
        .output()
        .expect("run exif-oxide");
    assert!(out.status.success());
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).expect("JSON stdout");
    let entries = json.as_array().expect("array");
    assert_eq!(entries.len(), 1, "-x value must not become a file: {json}");
    assert_eq!(entries[0]["SourceFile"].as_str(), Some(path.as_str()));
}

/// The unknown-option guard must survive the option table: short junk that is
/// neither an option nor a wildcard still fails loudly (classic mode).
#[test]
fn test_unknown_short_option_still_rejected() {
    let f = scratch_file();
    let path = f.path().to_str().unwrap().to_string();
    let out = exif_oxide()
        .args(["-zz", &path])
        .output()
        .expect("run exif-oxide");
    assert!(!out.status.success(), "-zz must be rejected");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Unknown option -zz"),
        "stderr must name the option: {stderr:?}"
    );
}
