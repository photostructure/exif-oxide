//! Replays exiftool-vendored.js' EXACT default ReadTask payload through a
//! single `-stay_open True -@ -` session, three real files plus a
//! missing-file cycle, and checks every consumer-visible invariant:
//!
//! - one JSON array with exactly one object per successful cycle, whose
//!   `SourceFile` echoes the requested path (ReadTask.ts:192-197 throws on
//!   any other SourceFile);
//! - errors/warnings appear ONLY as `ExifTool:Error`/`ExifTool:Warning`
//!   (M1a decision 3) - never as a lowercase `errors` array (invisible:
//!   ReadTask.ts:111 overwrites it) and never as invented
//!   `System:*DetectionStatus`/`Warning:Xxx` keys;
//! - `ExifToolVersion` reports the emulated ExifTool version;
//! - no cross-task residue: each cycle's output mentions only its own file.
//!
//! Payload shape verified against the consumer source
//! (../exiftool-vendored.js/src/ReadTask.ts:118-158 `ReadTask.for`,
//! ExifToolTask.ts:30-33 `renderCommand`, DefaultExifToolOptions.ts:73-120:
//! useMWG=true, struct=1, keepUTCTime=true, ignoreMinorErrors=true,
//! readArgs=["-fast"], the seven default numericTags).

#![cfg(feature = "integration-tests")]

use std::io::{Read, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The exact one-line UTF-8-repair filter from Utf8JsonFilter.ts (pinned in
/// the library as a contract fixture).
const UTF8_FILTER: &str = exif_oxide::cli::READTASK_UTF8_FILTER;

struct Session {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: Arc<Mutex<Vec<u8>>>,
    stderr: Arc<Mutex<Vec<u8>>>,
}

impl Session {
    fn spawn() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_exif-oxide"))
            .args(["-stay_open", "True", "-@", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn exif-oxide");
        let stdin = child.stdin.take();
        let stdout = Arc::new(Mutex::new(Vec::new()));
        let stderr = Arc::new(Mutex::new(Vec::new()));
        for (pipe, sink) in [
            (
                Box::new(child.stdout.take().unwrap()) as Box<dyn Read + Send>,
                Arc::clone(&stdout),
            ),
            (
                Box::new(child.stderr.take().unwrap()) as Box<dyn Read + Send>,
                Arc::clone(&stderr),
            ),
        ] {
            let mut pipe = pipe;
            std::thread::spawn(move || {
                let mut buf = [0u8; 8192];
                while let Ok(n) = pipe.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    sink.lock().unwrap().extend_from_slice(&buf[..n]);
                }
            });
        }
        Self {
            child,
            stdin,
            stdout,
            stderr,
        }
    }

    fn stdout_string(&self) -> String {
        String::from_utf8_lossy(&self.stdout.lock().unwrap()).into_owned()
    }

    fn stderr_string(&self) -> String {
        String::from_utf8_lossy(&self.stderr.lock().unwrap()).into_owned()
    }

    /// Render one ReadTask exactly like ExifToolTask.renderCommand
    /// (args joined by \n, then -ignoreMinorErrors, then -execute<id>),
    /// send it, and return this cycle's stdout (ready token stripped).
    fn read_task(&mut self, source_file: &str, execute_id: u32) -> String {
        // ReadTask.for (ReadTask.ts:118-158) with default options,
        // non-Windows (Utf8FilenameCharsetArgs is empty off-Windows).
        let mut args: Vec<String> = vec!["-json".into(), "-fast".into()];
        args.push("-api".into());
        args.push(UTF8_FILTER.into());
        args.push("-api".into());
        args.push("struct=1".into());
        args.push("-use".into());
        args.push("MWG".into());
        args.push("-api".into());
        args.push("keepUTCTime".into());
        for tag in [
            "*Duration*",
            "GPSAltitude",
            "GPSLatitude",
            "GPSLongitude",
            "GPSPosition",
            "GeolocationPosition",
            "Orientation",
        ] {
            args.push(format!("-{tag}#"));
        }
        args.push("-all".into());
        args.push(source_file.into());
        // renderCommand (ExifToolTask.ts:30-33)
        args.push("-ignoreMinorErrors".into());
        args.push(format!("-execute{execute_id}"));

        let before = self.stdout_string().len();
        let payload = args.join("\n") + "\n";
        let stdin = self.stdin.as_mut().expect("stdin");
        stdin.write_all(payload.as_bytes()).expect("write");
        stdin.flush().expect("flush");

        let marker = format!("{{ready{execute_id}}}\n");
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            let s = self.stdout_string();
            if let Some(pos) = s[before..].find(&marker) {
                return s[before..before + pos].to_string();
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for {marker:?}; stdout: {s:?}; stderr: {:?}",
                self.stderr_string()
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn shutdown(mut self) {
        let stdin = self.stdin.as_mut().expect("stdin");
        stdin.write_all(b"-stay_open\nFalse\n").expect("write");
        stdin.flush().expect("flush");
        drop(self.stdin.take());
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            if let Some(status) = self.child.try_wait().expect("try_wait") {
                assert!(status.success(), "exit status {status:?}");
                return;
            }
            assert!(Instant::now() < deadline, "child did not exit");
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

/// Assert one successful cycle's invariants; returns the entry for extra
/// per-file checks.
fn check_entry(cycle_stdout: &str, source_file: &str) -> serde_json::Value {
    let json: serde_json::Value = serde_json::from_str(cycle_stdout)
        .unwrap_or_else(|e| panic!("cycle output must be JSON ({e}): {cycle_stdout:?}"));
    let entries = json.as_array().expect("JSON array");
    assert_eq!(entries.len(), 1, "exactly one entry: {json}");
    let obj = entries[0].as_object().expect("object");

    // ReadTask.ts:192-197: an unexpected SourceFile throws.
    assert_eq!(
        obj["SourceFile"].as_str(),
        Some(source_file),
        "SourceFile must echo the request"
    );

    // Decision 2: the emulated ExifTool version.
    assert_eq!(
        obj["ExifToolVersion"].as_str(),
        Some(exif_oxide::EXIFTOOL_VERSION)
    );

    // Decision 3: errors/warnings ONLY as ExifTool:Error / ExifTool:Warning.
    let bad: Vec<&String> = obj
        .keys()
        .filter(|k| {
            *k == "errors"
                || *k == "warnings"
                || k.starts_with("System:")
                || k.starts_with("Warning:")
        })
        .collect();
    assert!(
        bad.is_empty(),
        "forbidden keys {bad:?} in cycle for {source_file}"
    );

    entries[0].clone()
}

#[test]
fn test_readtask_replay_three_files_and_missing() {
    let files = [
        "test-images/canon/eos_5d_mark_iii.jpg",
        "test-images/apple/IMG_3755.MOV",
        "test-images/example.gif",
    ];
    // The consumer resolves to absolute paths (ReadTask.ts:116).
    let abs: Vec<String> = files
        .iter()
        .map(|f| {
            Path::new(f)
                .canonicalize()
                .unwrap_or_else(|e| panic!("test asset {f} missing: {e}"))
                .to_str()
                .unwrap()
                .to_string()
        })
        .collect();

    let mut s = Session::spawn();

    // Startup health check, like the consumer's VersionTask.
    let ver_out = {
        let before = s.stdout_string().len();
        let stdin = s.stdin.as_mut().unwrap();
        stdin.write_all(b"-ver\n-execute0\n").unwrap();
        stdin.flush().unwrap();
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            let out = s.stdout_string();
            if let Some(pos) = out[before..].find("{ready0}\n") {
                break out[before..before + pos].to_string();
            }
            assert!(Instant::now() < deadline, "no {{ready0}}: {out:?}");
            std::thread::sleep(Duration::from_millis(10));
        }
    };
    assert_eq!(ver_out, format!("{}\n", exif_oxide::EXIFTOOL_VERSION));
    assert_eq!(
        s.stderr_string(),
        "",
        "any stderr during the startup task is fatal (Parser.ts:29-36)"
    );

    let mut entries = Vec::new();
    for (i, file) in abs.iter().enumerate() {
        let cycle = s.read_task(file, (i + 1) as u32);
        let entry = check_entry(&cycle, file);
        // No cross-task residue: this cycle mentions no other request's file.
        for other in abs.iter().filter(|o| *o != file) {
            assert!(
                !cycle.contains(other.as_str()),
                "cycle for {file} leaked {other}"
            );
        }
        entries.push(entry);
    }

    // Basic sanity per file type so the replay proves real extraction, not
    // just protocol plumbing.
    assert_eq!(entries[0]["File:MIMEType"].as_str(), Some("image/jpeg"));
    assert!(
        entries[0]["EXIF:Orientation"].is_number(),
        "-Orientation# must yield the numeric ValueConv value: {}",
        entries[0]["EXIF:Orientation"]
    );
    assert_eq!(
        entries[1]["File:MIMEType"].as_str(),
        Some("video/quicktime")
    );
    assert!(
        entries[1]["QuickTime:Duration"].is_number(),
        "-*Duration*# must yield numeric durations: {}",
        entries[1]["QuickTime:Duration"]
    );
    assert_eq!(entries[2]["File:MIMEType"].as_str(), Some("image/gif"));

    // Missing-file cycle: stderr line, no JSON, session stays healthy.
    let stderr_before = s.stderr_string().len();
    let cycle = s.read_task("/nonexistent-dir/replay-missing.jpg", 99);
    assert_eq!(cycle, "", "no JSON for a missing file");
    let stderr_tail = s.stderr_string()[stderr_before..].to_string();
    assert!(
        stderr_tail.contains("Error: File not found - /nonexistent-dir/replay-missing.jpg"),
        "stderr: {stderr_tail:?}"
    );

    // And one more real read to prove the pool survives the error.
    let cycle = s.read_task(&abs[0], 100);
    check_entry(&cycle, &abs[0]);

    s.shutdown();
}

/// Differential framing check against the vendored ExifTool: both children
/// answer a `-ver` cycle with `<version>\n{ready5}\n` and exit silently on
/// `-stay_open False`. This pins our framing to the reference
/// implementation, not just to our own expectations.
#[test]
fn test_framing_matches_vendored_exiftool() {
    fn ver_cycle(program: &str, args: &[&str]) -> (String, String) {
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| panic!("spawn {program}: {e}"));
        let mut stdin = child.stdin.take().unwrap();
        stdin
            .write_all(b"-ver\n-execute5\n-stay_open\nFalse\n")
            .unwrap();
        drop(stdin);
        let out = child.wait_with_output().expect("wait");
        assert!(out.status.success(), "{program} exit: {:?}", out.status);
        (
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    }

    let (ours_out, ours_err) = ver_cycle(
        env!("CARGO_BIN_EXE_exif-oxide"),
        &["-stay_open", "True", "-@", "-"],
    );
    let (ref_out, ref_err) = ver_cycle(
        "third-party/exiftool/exiftool",
        &["-stay_open", "True", "-@", "-"],
    );

    assert_eq!(
        ours_out, ref_out,
        "framing must match the vendored ExifTool byte for byte"
    );
    assert_eq!(ours_err, "", "our stderr must be silent");
    assert_eq!(ref_err, "", "reference stderr silent (sanity)");
    assert_eq!(
        ours_out,
        format!("{}\n{{ready5}}\n", exif_oxide::EXIFTOOL_VERSION)
    );
}
