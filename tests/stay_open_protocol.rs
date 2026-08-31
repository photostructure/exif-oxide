//! End-to-end conformance tests for `-stay_open True -@ -` mode.
//!
//! Deliberately asset-free and NOT feature-gated so CI always runs them: the
//! protocol framing must never regress, because the exiftool-vendored.js
//! consumer kills the child process on any deviation:
//!
//! - batch-cluster `Task.ts:86-101` matches the literal `{ready}` (or
//!   `{ready<id>}`) as a substring of accumulated stdout.
//! - batch-cluster `StreamHandler.ts:75-81,:95-100`: non-blank stdout/stderr
//!   with NO task pending kills the child.
//! - `Parser.ts:29-36`: during the startup `-ver` task, ANY non-blank stderr
//!   is fatal.
//! - The exit handshake is `-stay_open\nFalse\n` then stdin end
//!   (batch-cluster `ProcessTerminator.ts:133-146`).
//!
//! ExifTool's reference behavior: `{ready$id}\n` emission exiftool:429-442
//! (stderr flushed first, :435-439); `-stay_open False` closes the argfile
//! and exits with NO ready token (exiftool:1268-1293).

use std::io::{Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// A spawned exif-oxide in stay_open mode with stdout/stderr reader threads.
struct StayOpenSession {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: Arc<Mutex<Vec<u8>>>,
    stderr: Arc<Mutex<Vec<u8>>>,
}

impl StayOpenSession {
    fn spawn() -> Self {
        Self::spawn_with_args(&["-stay_open", "True", "-@", "-"])
    }

    fn spawn_with_args(args: &[&str]) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_exif-oxide"))
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn exif-oxide");
        let stdin = child.stdin.take();
        let stdout = Arc::new(Mutex::new(Vec::new()));
        let stderr = Arc::new(Mutex::new(Vec::new()));
        let mut out_pipe = child.stdout.take().expect("stdout pipe");
        let mut err_pipe = child.stderr.take().expect("stderr pipe");
        {
            let stdout = Arc::clone(&stdout);
            std::thread::spawn(move || {
                let mut buf = [0u8; 4096];
                while let Ok(n) = out_pipe.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    stdout.lock().unwrap().extend_from_slice(&buf[..n]);
                }
            });
        }
        {
            let stderr = Arc::clone(&stderr);
            std::thread::spawn(move || {
                let mut buf = [0u8; 4096];
                while let Ok(n) = err_pipe.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    stderr.lock().unwrap().extend_from_slice(&buf[..n]);
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

    fn send(&mut self, text: &str) {
        let stdin = self.stdin.as_mut().expect("stdin open");
        stdin.write_all(text.as_bytes()).expect("write stdin");
        stdin.flush().expect("flush stdin");
    }

    fn stdout_string(&self) -> String {
        String::from_utf8_lossy(&self.stdout.lock().unwrap()).into_owned()
    }

    fn stderr_string(&self) -> String {
        String::from_utf8_lossy(&self.stderr.lock().unwrap()).into_owned()
    }

    /// Wait until accumulated stdout contains `marker` `count` times, the way
    /// batch-cluster scans for `{ready}` (Task.ts:86-101).
    fn wait_for_ready(&self, marker: &str, count: usize) {
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            let s = self.stdout_string();
            if s.matches(marker).count() >= count {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for {count}x {marker:?}; stdout so far: {s:?}; stderr: {:?}",
                self.stderr_string()
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// Close stdin (EOF) and reap the child, asserting a clean exit.
    fn expect_clean_exit(mut self) -> (String, String) {
        drop(self.stdin.take());
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            match self.child.try_wait().expect("try_wait") {
                Some(status) => {
                    assert!(
                        status.success(),
                        "expected exit 0, got {status:?}; stderr: {:?}",
                        self.stderr_string()
                    );
                    break;
                }
                None => {
                    assert!(
                        Instant::now() < deadline,
                        "child did not exit; stdout: {:?} stderr: {:?}",
                        self.stdout_string(),
                        self.stderr_string()
                    );
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }
        // Give the reader threads a beat to drain the closed pipes.
        std::thread::sleep(Duration::from_millis(50));
        (self.stdout_string(), self.stderr_string())
    }
}

/// The consumer's startup health check: `-ver\n-execute\n` must produce
/// exactly `<version>\n{ready}\n` on stdout and NOTHING on stderr
/// (Parser.ts:29-36 makes any stderr fatal during this task; VersionTask.ts:7
/// pins the version shape).
#[test]
fn test_ver_cycle_exact_output() {
    let mut s = StayOpenSession::spawn();
    s.send("-ver\n-execute\n");
    s.wait_for_ready("{ready}", 1);
    assert_eq!(
        s.stdout_string(),
        format!("{}\n{{ready}}\n", exif_oxide::EXIFTOOL_VERSION)
    );
    assert_eq!(s.stderr_string(), "", "stderr must be completely silent");
    s.send("-stay_open\nFalse\n");
    let (stdout, stderr) = s.expect_clean_exit();
    assert_eq!(
        stdout,
        format!("{}\n{{ready}}\n", exif_oxide::EXIFTOOL_VERSION),
        "-stay_open False must not emit a ready token (exiftool:1268-1293)"
    );
    assert_eq!(stderr, "");
}

/// Numbered `-execute123` echoes the id in the ready token: `{ready123}`
/// (exiftool:629-631 captures the digits, :438 interpolates them).
#[test]
fn test_numbered_execute_ready_token() {
    let mut s = StayOpenSession::spawn();
    s.send("-ver\n-execute123\n");
    s.wait_for_ready("{ready123}", 1);
    assert_eq!(
        s.stdout_string(),
        format!("{}\n{{ready123}}\n", exif_oxide::EXIFTOOL_VERSION)
    );
    s.send("-stay_open\nFalse\n");
    s.expect_clean_exit();
}

/// A missing file produces the ExifTool stderr line, no JSON, and still
/// completes the task with its ready token - the pool must survive
/// (exiftool:2312-2318; the consumer surfaces the stderr line as the task
/// error, ExifToolTask.ts:59-71).
#[test]
fn test_missing_file_cycle() {
    let mut s = StayOpenSession::spawn();
    s.send("-ver\n-execute1\n");
    s.wait_for_ready("{ready1}", 1);
    s.send("-json\n-all\n/nonexistent-dir/missing.jpg\n-execute2\n");
    s.wait_for_ready("{ready2}", 1);
    let stdout = s.stdout_string();
    assert!(
        !stdout.contains('['),
        "no JSON output for a missing file: {stdout:?}"
    );
    assert!(
        s.stderr_string()
            .contains("Error: File not found - /nonexistent-dir/missing.jpg"),
        "stderr: {:?}",
        s.stderr_string()
    );
    s.send("-stay_open\nFalse\n");
    s.expect_clean_exit();
}

/// EOF on stdin exits cleanly with code 0 and no extra output. This is a
/// documented divergence from ExifTool, which spins forever polling a closed
/// argfile (exiftool:4975-4979); a clean exit is strictly friendlier to
/// process supervisors and never observed by batch-cluster, which always
/// sends `-stay_open False` first (ProcessTerminator.ts:133-146).
#[test]
fn test_eof_exits_cleanly() {
    let mut s = StayOpenSession::spawn();
    s.send("-ver\n-execute\n");
    s.wait_for_ready("{ready}", 1);
    let before = s.stdout_string();
    let (stdout, stderr) = s.expect_clean_exit();
    assert_eq!(stdout, before, "EOF must not produce trailing output");
    assert_eq!(stderr, "");
}

/// An unknown option in a command surfaces on stderr while the task is
/// pending (so the consumer maps it to a task error), produces no JSON, and
/// the session keeps serving subsequent commands.
#[test]
fn test_bad_option_is_task_error_not_process_death() {
    let mut s = StayOpenSession::spawn();
    s.send("-zz\n-execute7\n");
    s.wait_for_ready("{ready7}", 1);
    assert!(s.stderr_string().contains("Unknown option -zz"));
    // The session must still be alive and serving.
    s.send("-ver\n-execute8\n");
    s.wait_for_ready("{ready8}", 1);
    assert!(s
        .stdout_string()
        .contains(&format!("{}\n{{ready8}}", exif_oxide::EXIFTOOL_VERSION)));
    s.send("-stay_open\nFalse\n");
    s.expect_clean_exit();
}

/// Whole-session transcript: several cycles, then exit - accumulated stdout
/// must EXACTLY equal the expected concatenation. Zero stray bytes is the
/// contract that keeps batch-cluster's process pool alive
/// (StreamHandler.ts:75-81).
#[test]
fn test_whole_session_stdout_byte_exact() {
    let mut s = StayOpenSession::spawn();
    s.send("-ver\n-execute\n");
    s.wait_for_ready("{ready}", 1);
    s.send("-echo\nhello world\n-execute55\n");
    s.wait_for_ready("{ready55}", 1);
    s.send("-ver\n-execute56\n");
    s.wait_for_ready("{ready56}", 1);
    s.send("-stay_open\nFalse\n");
    let (stdout, stderr) = s.expect_clean_exit();
    assert_eq!(
        stdout,
        format!(
            "{ver}\n{{ready}}\nhello world\n{{ready55}}\n{ver}\n{{ready56}}\n",
            ver = exif_oxide::EXIFTOOL_VERSION
        )
    );
    assert_eq!(stderr, "");
}

/// The argfile reader must not treat an option's VALUE as a command
/// terminator: a literal `-execute` line following `-if` is data
/// (exiftool ReadStayOpen:4950-4963 via %optArgs:260-300).
#[test]
fn test_option_value_execute_not_terminator() {
    let mut s = StayOpenSession::spawn();
    // -if consumes the next argument, so the first `-execute` line is its
    // value; only the second one terminates the command.
    s.send("-if\n-execute\n-ver\n-execute9\n");
    s.wait_for_ready("{ready9}", 1);
    assert_eq!(
        s.stdout_string(),
        format!("{}\n{{ready9}}\n", exif_oxide::EXIFTOOL_VERSION)
    );
    s.send("-stay_open\nFalse\n");
    s.expect_clean_exit();
}
