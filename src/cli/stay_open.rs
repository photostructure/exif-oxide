//! The `-stay_open True -@ -` REPL used by exiftool-vendored.js.
//!
//! Protocol contract (verified against the consumer, 2026-08-30):
//!
//! - Spawn: `exif-oxide -stay_open True -@ -`, no shell
//!   (../exiftool-vendored.js/src/DefaultExiftoolArgs.ts:2, ExifTool.ts:284-291).
//! - Commands arrive as newline-separated argfile lines ending in
//!   `-execute[N]` (ExifToolTask.ts:30-33); the answer is task output followed
//!   by `{ready<N>}\n` on stdout, with stderr flushed FIRST
//!   (exiftool:429-442, esp. :435-439 - exiftool-vendored sets
//!   `streamFlushMillis` citing those lines).
//! - Zero stray output: any non-blank stdout/stderr while NO task is pending
//!   kills the child process (batch-cluster StreamHandler.ts:75-81, :95-100),
//!   so this module never writes outside a command's execution window.
//! - Exit: `-stay_open\nFalse\n` then stdin end
//!   (batch-cluster ProcessTerminator.ts:133-146); NO ready token afterwards
//!   (exiftool:1268-1293 just closes the argfile).

use std::io::{BufRead, Write};
use std::path::Path;

use crate::cli::argfile::{ArgfileEvent, CommandReader};
use crate::cli::options::parse_command;
use crate::formats::extract_metadata;
use crate::types::{ExifData, FilterOptions};

/// Detect the supported stay_open invocation in raw argv (without argv[0]).
///
/// Returns `Some(leftover_args)` when argv contains `-stay_open True|1`
/// followed (not necessarily adjacently) by `-@ -`; the leftover args (any
/// argv entries outside those two pairs) seed the first command, matching
/// ExifTool where pre-`-@` arguments become part of the first command.
///
/// `-stay_open` matches case-insensitively (ExifTool lower-cases option
/// names, exiftool:696); the `-@ -` pair is exact. Other `-@` forms (a real
/// argfile path, or `-@ -` without stay_open) fall through to classic mode,
/// where the parser rejects them loudly.
pub fn detect_stay_open(args: &[String]) -> Option<Vec<String>> {
    let mut stay_open_at = None;
    let mut argfile_at = None;
    let mut i = 0;
    while i + 1 < args.len() {
        if stay_open_at.is_none()
            && args[i].eq_ignore_ascii_case("-stay_open")
            && (args[i + 1].eq_ignore_ascii_case("true") || args[i + 1] == "1")
        {
            stay_open_at = Some(i);
            i += 2;
            continue;
        }
        if stay_open_at.is_some() && argfile_at.is_none() && args[i] == "-@" && args[i + 1] == "-" {
            argfile_at = Some(i);
            i += 2;
            continue;
        }
        i += 1;
    }
    let (s, a) = (stay_open_at?, argfile_at?);
    let mut seed = Vec::new();
    for (idx, arg) in args.iter().enumerate() {
        if idx == s || idx == s + 1 || idx == a || idx == a + 1 {
            continue;
        }
        seed.push(arg.clone());
    }
    Some(seed)
}

/// Run the REPL over the given streams until `-stay_open False` or EOF.
/// Returns the process exit code (always 0: EOF is a clean shutdown by
/// design - documented divergence from ExifTool's busy-poll loop,
/// exiftool:4975-4979).
pub fn run<R: BufRead, W: Write, E: Write>(
    input: R,
    mut out: W,
    mut err: E,
    seed_args: Vec<String>,
) -> i32 {
    let mut reader = CommandReader::new(input);
    reader.seed_args(seed_args);
    loop {
        match reader.next_event() {
            ArgfileEvent::Command(cmd) => {
                execute_command(&cmd.args, &mut out, &mut err);
                // stderr is flushed BEFORE the ready token so the consumer
                // has every diagnostic when it resolves the task
                // (exiftool:429-442; exiftool-vendored streamFlushMillis).
                let _ = err.flush();
                if writeln!(out, "{{ready{}}}", cmd.execute_id).is_err() || out.flush().is_err() {
                    // stdout is gone: the consumer died. Nothing sensible
                    // remains to serve; exit quietly.
                    return 0;
                }
            }
            ArgfileEvent::Exit | ArgfileEvent::Eof => {
                let _ = err.flush();
                let _ = out.flush();
                return 0;
            }
        }
    }
}

/// Parse and run one command, writing its stdout/stderr. Never panics out
/// and never exits: a bad command becomes stderr lines (surfaced as the task
/// error by the consumer) followed by the caller's ready token.
fn execute_command(args: &[String], out: &mut impl Write, err: &mut impl Write) {
    let parsed = parse_command(args);

    // -echo/-echo2 print before any other output (exiftool:1016-1028).
    for line in &parsed.echo_stdout {
        let _ = writeln!(out, "{line}");
    }
    for line in &parsed.echo_stderr {
        let _ = writeln!(err, "{line}");
    }

    // A bad command aborts without output, like ExifTool's $badCmd
    // (exiftool:632, :688); the error text reaches the consumer because a
    // task is pending right now.
    if !parsed.errors.is_empty() {
        for e in &parsed.errors {
            let _ = writeln!(err, "{e}");
        }
        return;
    }

    if parsed.print_version {
        let _ = writeln!(out, "{}", crate::EXIFTOOL_VERSION);
    }

    // An empty or -ver-only command produces no further output - and no
    // "No files specified" complaint (`-execute` sets $helped, exiftool:633).
    if parsed.files.is_empty() {
        return;
    }

    let entries = collect_entries(&parsed.files, &parsed.filter, false, err);
    if !entries.is_empty() {
        match serde_json::to_string_pretty(&entries) {
            Ok(json) => {
                let _ = writeln!(out, "{json}");
            }
            Err(e) => {
                // Should be unreachable (ExifData always serializes); if it
                // happens, surface it as a task error instead of dying.
                let _ = writeln!(err, "Error: failed to serialize JSON - {e}");
            }
        }
    }
}

/// Process one command's files into JSON-ready entries, writing the
/// ExifTool-style stderr line for missing files. Shared by classic argv mode
/// (`main.rs::process_files`) and the stay_open REPL.
///
/// - Missing file: `Error: File not found - $file` on `err`, NO entry
///   (exiftool:2312-2318).
/// - Existing but unparseable: an entry whose error serializes as the
///   `ExifTool:Error` key (probed against vendored ExifTool 13.59).
pub fn collect_entries(
    files: &[String],
    filter: &FilterOptions,
    show_missing: bool,
    err: &mut impl Write,
) -> Vec<ExifData> {
    let mut results = Vec::new();
    for file in files {
        let path = Path::new(file);
        if !path.exists() {
            let _ = writeln!(err, "Error: File not found - {file}");
            continue;
        }
        match extract_metadata(path, show_missing, false, Some(filter.clone())) {
            Ok(metadata) => results.push(metadata),
            Err(e) => {
                tracing::error!("Failed to process {file}: {e}");
                // ExifToolVersion mirrors extract_metadata's gating: present
                // for -all/unfiltered runs, absent for filtered ones (probed:
                // filtered ExifTool output carries neither ExifToolVersion
                // nor the Error key).
                let version = if filter.extract_all {
                    crate::EXIFTOOL_VERSION.to_string()
                } else {
                    String::new()
                };
                let mut entry = ExifData::new(file.clone(), version);
                entry.errors.push(format!("Error processing file: {e}"));
                results.push(entry);
            }
        }
    }
    // The ordered request list decides which tags print their ValueConv
    // value, and the filter decides whether ExifTool:Error/Warning appear.
    for result in &mut results {
        result.prepare_for_serialization(Some(filter));
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_stay_open_consumer_spawn() {
        let args: Vec<String> = ["-stay_open", "True", "-@", "-"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(detect_stay_open(&args), Some(vec![]));
    }

    #[test]
    fn test_detect_stay_open_case_and_one() {
        let args: Vec<String> = ["-STAY_OPEN", "1", "-@", "-"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(detect_stay_open(&args), Some(vec![]));
    }

    #[test]
    fn test_detect_stay_open_keeps_leftover_args() {
        let args: Vec<String> = ["-ver", "-stay_open", "True", "-@", "-"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(detect_stay_open(&args), Some(vec!["-ver".to_string()]));
    }

    #[test]
    fn test_detect_stay_open_requires_both_pairs_in_order() {
        for argv in [
            vec!["-stay_open", "True"],               // no argfile
            vec!["-@", "-"],                          // no stay_open
            vec!["-@", "-", "-stay_open", "True"],    // wrong order: batch argfile mode
            vec!["-stay_open", "False", "-@", "-"],   // not opening
            vec!["-stay_open", "True", "-@", "args"], // real argfile, unsupported
        ] {
            let args: Vec<String> = argv.iter().map(|s| s.to_string()).collect();
            assert_eq!(detect_stay_open(&args), None, "argv: {argv:?}");
        }
    }

    /// Full in-process session: byte-exact stdout, silent stderr.
    #[test]
    fn test_run_session_byte_exact() {
        let input = b"-ver\n-execute\n-ver\n-execute42\n-stay_open\nFalse\n".to_vec();
        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = run(std::io::Cursor::new(input), &mut out, &mut err, vec![]);
        assert_eq!(code, 0);
        assert_eq!(
            String::from_utf8(out).unwrap(),
            format!(
                "{v}\n{{ready}}\n{v}\n{{ready42}}\n",
                v = crate::EXIFTOOL_VERSION
            )
        );
        assert_eq!(err, b"", "stderr must be untouched");
    }

    /// EOF without the exit handshake: clean exit 0, nothing extra written.
    #[test]
    fn test_run_eof_clean() {
        let input = b"-ver\n-execute\n".to_vec();
        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = run(std::io::Cursor::new(input), &mut out, &mut err, vec![]);
        assert_eq!(code, 0);
        assert_eq!(
            String::from_utf8(out).unwrap(),
            format!("{}\n{{ready}}\n", crate::EXIFTOOL_VERSION)
        );
        assert_eq!(err, b"");
    }

    /// Unknown options become task-scoped stderr, the ready token still
    /// arrives, and the session keeps serving.
    #[test]
    fn test_run_bad_command_keeps_session() {
        let input = b"-zz\n-execute1\n-ver\n-execute2\n".to_vec();
        let mut out = Vec::new();
        let mut err = Vec::new();
        run(std::io::Cursor::new(input), &mut out, &mut err, vec![]);
        assert_eq!(
            String::from_utf8(out).unwrap(),
            format!("{{ready1}}\n{}\n{{ready2}}\n", crate::EXIFTOOL_VERSION)
        );
        assert_eq!(String::from_utf8(err).unwrap(), "Unknown option -zz\n");
    }

    /// Missing files: stderr line while the task is pending, no JSON.
    #[test]
    fn test_run_missing_file() {
        let input = b"-json\n-all\n/nonexistent-dir/missing.jpg\n-execute3\n".to_vec();
        let mut out = Vec::new();
        let mut err = Vec::new();
        run(std::io::Cursor::new(input), &mut out, &mut err, vec![]);
        assert_eq!(String::from_utf8(out).unwrap(), "{ready3}\n");
        assert_eq!(
            String::from_utf8(err).unwrap(),
            "Error: File not found - /nonexistent-dir/missing.jpg\n"
        );
    }
}
