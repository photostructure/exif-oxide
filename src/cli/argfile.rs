//! `-stay_open` argfile reading: ExifTool's `FilterArgfileLine` and
//! `ReadStayOpen` (third-party/exiftool/exiftool:4896-4918, :4925-4987),
//! ported line-for-line and verified against the Perl (probed 2026-08-30 by
//! running the original regexes; the `$`/`@` backslash quirk below is real).
//!
//! The reader owns the input stream and chunks filtered lines into complete
//! commands. A command ends at a line matching `-execute[N]` UNLESS that line
//! is the value of a preceding value-taking option (`-if`, `-api`, ...):
//! that's what the `%optArgs` table is for (exiftool:260-300 - "used only to
//! skip over these arguments when reading -stay_open ARGFILE").

use std::collections::VecDeque;
use std::io::BufRead;

/// One complete command, terminated by `-execute[N]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    /// The filtered argument lines, in order (without the `-execute` line).
    pub args: Vec<String>,
    /// The digits captured from `-execute<id>` (empty for a bare `-execute`).
    /// Echoed back in the `{ready<id>}` token (exiftool:629-631, :438).
    pub execute_id: String,
}

/// What the argfile stream produced next.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgfileEvent {
    /// A command to execute (then answer with `{ready<id>}`).
    Command(Command),
    /// `-stay_open False|0`: flush and exit WITHOUT a ready token
    /// (exiftool:1279-1287 closes the argfile; no `{ready}` is printed for
    /// the exit handshake - verified against batch-cluster
    /// ProcessTerminator.ts:133-146, which never waits for one).
    Exit,
    /// End of input. ExifTool spins forever re-polling a closed argfile
    /// (exiftool:4975-4979); we deliberately diverge and report EOF so the
    /// caller can exit 0 - batch-cluster always sends `-stay_open False`
    /// first, so a consumer never observes the difference.
    Eof,
}

/// Streaming reader that turns argfile lines into [`ArgfileEvent`]s.
pub struct CommandReader<R: BufRead> {
    input: R,
    /// Args accumulated for the command in progress.
    pending: Vec<String>,
    /// `Some(last_opt)` when the NEXT line is the value of `last_opt`
    /// (lower-cased), mirroring ExifTool's `$optArgs`/`$lastOpt` pair
    /// (exiftool:4934, :4950-4963).
    awaiting_value_for: Option<String>,
    /// Events produced by `accept` but not yet returned (seeded argv args may
    /// contain a whole command).
    queued: VecDeque<ArgfileEvent>,
}

impl<R: BufRead> CommandReader<R> {
    pub fn new(input: R) -> Self {
        Self {
            input,
            pending: Vec::new(),
            awaiting_value_for: None,
            queued: VecDeque::new(),
        }
    }

    /// Seed arguments that arrived on the command line before `-@ -`
    /// (ExifTool prepends leftover @ARGV to the first command). They run
    /// through the same state machine as argfile lines, so a value-taking
    /// option at the end correctly consumes the first stdin line.
    pub fn seed_args(&mut self, args: Vec<String>) {
        for arg in args {
            self.accept(arg);
        }
    }

    /// Block until the stream yields the next event.
    pub fn next_event(&mut self) -> ArgfileEvent {
        loop {
            if let Some(event) = self.queued.pop_front() {
                return event;
            }
            let mut raw = Vec::new();
            // Blocking read of one line; lossy UTF-8 (documented limitation:
            // the consumer always writes UTF-8 paths).
            match self.input.read_until(b'\n', &mut raw) {
                Ok(0) => return ArgfileEvent::Eof,
                Ok(_) => {}
                // Treat read errors like EOF: there is nothing more to serve,
                // and any stderr chatter here (no task pending) would kill
                // the consumer's process pool (batch-cluster
                // StreamHandler.ts:95-100).
                Err(_) => return ArgfileEvent::Eof,
            }
            let line = String::from_utf8_lossy(&raw);
            let Some(arg) = filter_argfile_line(&line) else {
                continue; // comment or blank line
            };
            self.accept(arg);
        }
    }

    /// Feed one filtered argument through the ReadStayOpen state machine.
    fn accept(&mut self, arg: String) {
        // Is this the value of the previous option? (checked FIRST, exactly
        // like ExifTool's `if ($optArgs)` branch, :4950-4954 - this is what
        // keeps a value line spelled `-execute` from terminating a command).
        if let Some(last_opt) = self.awaiting_value_for.take() {
            if last_opt == "-stay_open" {
                // Processed immediately (exiftool:4954 triggers argument
                // processing; the main loop handles the value, :1268-1293).
                if arg.eq_ignore_ascii_case("false") || arg == "0" {
                    // Divergence: ExifTool first processes any buffered
                    // partial command (:1283-1288); we exit immediately. The
                    // consumer only ever sends the pair on its own
                    // (ProcessTerminator.ts:133-146).
                    self.queued.push_back(ArgfileEvent::Exit);
                    return;
                }
                // True/1 while already open: ExifTool warns "-stay_open
                // already active" (exiftool:1276) - we stay silent because an
                // unsolicited stderr line between tasks kills the consumer's
                // pool (StreamHandler.ts:95-100). Invalid values likewise
                // (exiftool:1290 warns "Invalid argument for -stay_open").
                return;
            }
            // Any other option value (including `-@ <file>`: nested argfiles
            // are rejected later by the command parser, which runs while a
            // task is pending and may safely write stderr).
            self.pending.push(arg);
            return;
        }

        let lower = arg.to_ascii_lowercase();
        if is_execute(&lower) {
            let execute_id = lower["-execute".len()..].to_string();
            self.queued.push_back(ArgfileEvent::Command(Command {
                args: std::mem::take(&mut self.pending),
                execute_id,
            }));
            return;
        }
        if lower == "-stay_open" {
            // The pair is intercepted; the option itself is not stored.
            self.awaiting_value_for = Some(lower);
            return;
        }
        if takes_value(&arg) {
            self.awaiting_value_for = Some(lower);
        }
        self.pending.push(arg);
    }
}

/// `/^-execute\d*$/` on the lower-cased argument (exiftool:4962).
fn is_execute(lower: &str) -> bool {
    lower
        .strip_prefix("-execute")
        .is_some_and(|d| d.chars().all(|c| c.is_ascii_digit()))
}

/// ExifTool's `%optArgs` lookup (exiftool:260-300, :4955-4960): does this
/// argument consume the NEXT argfile line as its value?
///
/// Value 0 entries (`-charset`, `-lang`, and the case guards `-D`, `-P`,
/// `-X`) exist in ExifTool's table precisely so the argfile reader does NOT
/// skip a line for them; they translate to `false` here.
fn takes_value(arg: &str) -> bool {
    // 1) exact (case-sensitive) lookup
    if let Some(v) = opt_args_exact(arg) {
        return v;
    }
    // 2) lower-case lookup
    let lower = arg.to_ascii_lowercase();
    if let Some(v) = opt_args_exact(&lower) {
        return v;
    }
    // 3) trailing-number handling: `-echo2` matches `-echo#`, `-efile2!`
    //    matches `-efile#!` (exiftool:4959: /^(.*?)\d+(!?)$/ => "$1#$2")
    if let Some(rest) = lower.strip_suffix('!') {
        if let Some(stripped) = strip_trailing_digits(rest) {
            return opt_args_exact(&format!("{stripped}#!")).unwrap_or(false);
        }
    }
    if let Some(stripped) = strip_trailing_digits(&lower) {
        return opt_args_exact(&format!("{stripped}#")).unwrap_or(false);
    }
    false
}

/// Returns the prefix before a non-empty run of trailing digits, or None.
fn strip_trailing_digits(s: &str) -> Option<&str> {
    let trimmed = s.trim_end_matches(|c: char| c.is_ascii_digit());
    (trimmed.len() < s.len()).then_some(trimmed)
}

/// The `%optArgs` table, ported verbatim (exiftool:260-300). `true` = the
/// option takes its value from the next argfile line; `false` entries are
/// the explicit 0 values ExifTool lists to stop the lower-case fallback from
/// matching (`-D` vs `-d`, ...) or because the value is optional and cannot
/// begin with a dash (`-charset`, `-lang`).
fn opt_args_exact(key: &str) -> Option<bool> {
    Some(match key {
        "-tagsfromfile" | "-addtagsfromfile" | "-alltagsfromfile" => true,
        "-@" => true,
        "-api" => true,
        "-c" | "-coordformat" => true,
        "-charset" => false,
        "-config" => true,
        "-csvdelim" => true,
        "-d" | "-dateformat" => true,
        "-D" => false,
        "-diff" => true,
        "-echo" | "-echo#" => true,
        "-efile" | "-efile#" | "-efile!" | "-efile#!" => true,
        "-ext" | "--ext" | "-ext+" | "--ext+" => true,
        "-extension" | "--extension" | "-extension+" | "--extension+" => true,
        "-fileorder" | "-fileorder#" => true,
        "-file#" => true,
        "-geotag" => true,
        "-globaltimeshift" => true,
        "-i" | "-ignore" => true,
        "-if" | "-if#" => true,
        "-lang" => false,
        "-listitem" => true,
        "-o" | "-out" => true,
        "-p" | "-printformat" | "-p-" | "-printformat-" => true,
        "-P" => false,
        "-password" => true,
        "-require" => true,
        "-sep" | "-separator" => true,
        "-srcfile" => true,
        "-stay_open" => true,
        "-use" => true,
        "-userparam" => true,
        "-w" | "-w!" | "-w+" | "-w+!" | "-w!+" => true,
        "-textout" | "-textout!" | "-textout+" | "-textout+!" | "-textout!+" => true,
        "-tagout" | "-tagout!" | "-tagout+" | "-tagout+!" | "-tagout!+" => true,
        "-wext" => true,
        "-wm" | "-writemode" => true,
        "-x" | "-exclude" => true,
        "-X" => false,
        _ => return None,
    })
}

/// Port of ExifTool's `FilterArgfileLine` (exiftool:4896-4918).
///
/// Returns `None` for comments and blank lines. `#[CSTR]` lines are C-string
/// unescaped. Verified against the original Perl (probed 2026-08-30):
///
/// - `#[CSTR]a\nb`  => "a\nb" (real newline)
/// - `#[CSTR]a"b`   => `a"b`
/// - `#[CSTR]a$b`   => `a\$b`  (bare `$`/`@` GAIN a backslash - the Perl
///   escapes them in pass 1 and its `%esc` table has no entry to unescape
///   them in pass 2; we replicate the quirk exactly)
/// - `#[CSTR]a\\nb` => `a\nb` (literal backslash + n)
/// - trailing `\`   => kept
/// - `  -Tag  \r\n` => `-Tag  ` (leading whitespace stripped, trailing spaces
///   KEPT, CR/LF stripped)
/// - `-Comment = x` => `-Comment=x` (whitespace around assignment collapsed)
pub fn filter_argfile_line(line: &str) -> Option<String> {
    if let Some(rest) = line.strip_prefix('#') {
        // Comment lines begin with '#' unless tagged #[CSTR]
        let cstr = rest.strip_prefix("[CSTR]")?;
        let cstr = cstr.trim_end_matches(['\r', '\n']);
        Some(unescape_cstr(cstr))
    } else {
        let arg = line.trim_start_matches([' ', '\t', '\n', '\r', '\x0b', '\x0c']);
        let arg = arg.trim_end_matches(['\r', '\n']);
        let arg = normalize_assignment(arg);
        if arg.is_empty() {
            None
        } else {
            Some(arg)
        }
    }
}

/// The combined effect of ExifTool's two-pass C-string processing
/// (exiftool:4903-4912): known escapes are decoded, unknown escapes are kept
/// verbatim, and bare `"`, `$`, `@` behave as probed above.
fn unescape_cstr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('a') => out.push('\x07'),
                Some('b') => out.push('\x08'),
                Some('f') => out.push('\x0c'),
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    // Unknown escape: kept verbatim ($esc{$1}||'\\'.$1)
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'), // trailing backslash survives
            }
        } else if c == '$' || c == '@' {
            // Escaped in pass 1, never unescaped in pass 2 (no %esc entry).
            out.push('\\');
            out.push(c);
        } else {
            out.push(c);
        }
    }
    out
}

/// `s/^(-[-_0-9A-Z:]+#?)\s*([-+<]?=) ?/$1$2/i` (exiftool:4915): remove
/// whitespace before, and a single space after, `=`, `+=`, `-=` or `<=` in a
/// tag assignment argument.
fn normalize_assignment(arg: &str) -> String {
    let bytes = arg.as_bytes();
    if bytes.first() != Some(&b'-') {
        return arg.to_string();
    }
    // (-[-_0-9A-Z:]+#?) - the tag-name part, case-insensitive
    let mut i = 1;
    while i < bytes.len()
        && (bytes[i].is_ascii_alphanumeric() || matches!(bytes[i], b'-' | b'_' | b':'))
    {
        i += 1;
    }
    if i == 1 {
        return arg.to_string(); // no tag chars after '-'
    }
    if i < bytes.len() && bytes[i] == b'#' {
        i += 1;
    }
    let name_end = i;
    // \s*
    while i < bytes.len() && (bytes[i] as char).is_ascii_whitespace() {
        i += 1;
    }
    // ([-+<]?=)
    let op_start = i;
    if i < bytes.len() && matches!(bytes[i], b'-' | b'+' | b'<') {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b'=' {
        return arg.to_string(); // not an assignment; leave untouched
    }
    i += 1;
    let op = &arg[op_start..i];
    // a single optional space after the operator
    if i < bytes.len() && bytes[i] == b' ' {
        i += 1;
    }
    format!("{}{}{}", &arg[..name_end], op, &arg[i..])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn reader(input: &str) -> CommandReader<Cursor<Vec<u8>>> {
        CommandReader::new(Cursor::new(input.as_bytes().to_vec()))
    }

    // ---- filter_argfile_line (all cases probed against the Perl) ----------

    #[test]
    fn test_filter_comments_and_blanks() {
        assert_eq!(filter_argfile_line("# a comment\n"), None);
        assert_eq!(filter_argfile_line("\n"), None);
        assert_eq!(filter_argfile_line("   \n"), None);
        assert_eq!(filter_argfile_line("\r\n"), None);
    }

    #[test]
    fn test_filter_strips_whitespace_and_crlf() {
        assert_eq!(
            filter_argfile_line("  -TagName  \r\n").as_deref(),
            Some("-TagName  "),
            "leading whitespace stripped, trailing spaces kept, CRLF stripped"
        );
        assert_eq!(filter_argfile_line("-ver\r\n").as_deref(), Some("-ver"));
        assert_eq!(filter_argfile_line("-ver").as_deref(), Some("-ver"));
    }

    #[test]
    fn test_filter_assignment_normalization() {
        assert_eq!(
            filter_argfile_line("-Comment = hello there\n").as_deref(),
            Some("-Comment=hello there")
        );
        assert_eq!(
            filter_argfile_line("-Comment= hello\n").as_deref(),
            Some("-Comment=hello")
        );
        assert_eq!(
            filter_argfile_line("-Rating+= 3\n").as_deref(),
            Some("-Rating+=3")
        );
        // Not an assignment: untouched.
        assert_eq!(
            filter_argfile_line("-api Filter= x\n").as_deref(),
            Some("-api Filter= x"),
            "space inside the name part stops the pattern"
        );
    }

    #[test]
    fn test_filter_cstr_unescape() {
        assert_eq!(
            filter_argfile_line("#[CSTR]a\\nb\n").as_deref(),
            Some("a\nb")
        );
        assert_eq!(
            filter_argfile_line("#[CSTR]tab\\there\n").as_deref(),
            Some("tab\there")
        );
        assert_eq!(
            filter_argfile_line("#[CSTR]a\"b\n").as_deref(),
            Some("a\"b")
        );
        // The probed Perl quirk: bare $ and @ gain a backslash.
        assert_eq!(
            filter_argfile_line("#[CSTR]a$b\n").as_deref(),
            Some("a\\$b")
        );
        assert_eq!(
            filter_argfile_line("#[CSTR]a@b\n").as_deref(),
            Some("a\\@b")
        );
        // Literal backslash sequences.
        assert_eq!(
            filter_argfile_line("#[CSTR]a\\\\nb\n").as_deref(),
            Some("a\\nb")
        );
        assert_eq!(
            filter_argfile_line("#[CSTR]trail\\\\\n").as_deref(),
            Some("trail\\")
        );
        assert_eq!(
            filter_argfile_line("#[CSTR]unknown\\qesc\n").as_deref(),
            Some("unknown\\qesc")
        );
        // An empty CSTR line is a real (empty) argument in ExifTool: only the
        // non-CSTR branch discards empties.
        assert_eq!(filter_argfile_line("#[CSTR]\n").as_deref(), Some(""));
    }

    // ---- command chunking -------------------------------------------------

    #[test]
    fn test_simple_ver_command() {
        let mut r = reader("-ver\n-execute\n");
        assert_eq!(
            r.next_event(),
            ArgfileEvent::Command(Command {
                args: vec!["-ver".to_string()],
                execute_id: String::new(),
            })
        );
        assert_eq!(r.next_event(), ArgfileEvent::Eof);
    }

    #[test]
    fn test_numbered_execute_id() {
        let mut r = reader("-ver\n-EXECUTE7\n");
        match r.next_event() {
            ArgfileEvent::Command(cmd) => assert_eq!(cmd.execute_id, "7"),
            other => panic!("expected command, got {other:?}"),
        }
    }

    /// The exact consumer payload: the Utf8JsonFilter one-liner is an option
    /// VALUE and must never be split, dropped, or treated as a file.
    #[test]
    fn test_readtask_payload_chunking_verbatim() {
        let filter = crate::cli::READTASK_UTF8_FILTER;
        let input = format!(
            "-json\n-fast\n-api\n{filter}\n-api\nstruct=1\n-use\nMWG\n-api\nkeepUTCTime\n\
             -*Duration*#\n-GPSAltitude#\n-GPSLatitude#\n-GPSLongitude#\n-GPSPosition#\n\
             -GeolocationPosition#\n-Orientation#\n-all\n/tmp/photo.jpg\n-ignoreMinorErrors\n-execute\n"
        );
        let mut r = reader(&input);
        match r.next_event() {
            ArgfileEvent::Command(cmd) => {
                assert_eq!(cmd.execute_id, "");
                assert_eq!(cmd.args.len(), 20);
                assert_eq!(cmd.args[2], "-api");
                assert_eq!(cmd.args[3], filter, "filter value passed through verbatim");
                assert_eq!(cmd.args[18], "/tmp/photo.jpg");
                assert_eq!(cmd.args[19], "-ignoreMinorErrors");
            }
            other => panic!("expected command, got {other:?}"),
        }
    }

    /// A value line that spells `-execute` is NOT a terminator
    /// (exiftool:4950-4963: option values are skipped via %optArgs).
    #[test]
    fn test_option_value_not_a_terminator() {
        let mut r = reader("-if\n-execute\n-ver\n-execute2\n");
        match r.next_event() {
            ArgfileEvent::Command(cmd) => {
                assert_eq!(cmd.args, vec!["-if", "-execute", "-ver"]);
                assert_eq!(cmd.execute_id, "2");
            }
            other => panic!("expected command, got {other:?}"),
        }
    }

    /// `-charset` has an OPTIONAL argument (0 in %optArgs) precisely so the
    /// argfile reader does NOT consume the next line as its value: a
    /// following `-execute` line terminates the command.
    #[test]
    fn test_charset_does_not_consume_next_line() {
        let mut r = reader("-charset\n-execute\n");
        match r.next_event() {
            ArgfileEvent::Command(cmd) => assert_eq!(cmd.args, vec!["-charset"]),
            other => panic!("expected command, got {other:?}"),
        }
    }

    /// Trailing-number handling: `-echo2` matches the `-echo#` table entry
    /// and consumes its value line (exiftool:4959).
    #[test]
    fn test_echo2_consumes_value() {
        let mut r = reader("-echo2\n-execute\n-execute3\n");
        match r.next_event() {
            ArgfileEvent::Command(cmd) => {
                assert_eq!(
                    cmd.args,
                    vec!["-echo2", "-execute"],
                    "the -execute line is -echo2's value"
                );
                assert_eq!(cmd.execute_id, "3");
            }
            other => panic!("expected command, got {other:?}"),
        }
    }

    /// Case guards from %optArgs: `-d` takes a value, `-D` does not.
    #[test]
    fn test_case_sensitive_optargs_guards() {
        let mut r = reader("-d\n-execute\n-execute4\n");
        match r.next_event() {
            ArgfileEvent::Command(cmd) => {
                assert_eq!(cmd.args, vec!["-d", "-execute"]);
                assert_eq!(cmd.execute_id, "4");
            }
            other => panic!("expected command, got {other:?}"),
        }
        let mut r = reader("-D\n-execute5\n");
        match r.next_event() {
            ArgfileEvent::Command(cmd) => {
                assert_eq!(cmd.args, vec!["-D"], "-D takes no value");
                assert_eq!(cmd.execute_id, "5");
            }
            other => panic!("expected command, got {other:?}"),
        }
    }

    #[test]
    fn test_stay_open_false_exits_without_ready() {
        let mut r = reader("-stay_open\nFalse\n-ver\n-execute\n");
        assert_eq!(r.next_event(), ArgfileEvent::Exit);
        let mut r = reader("-stay_open\n0\n");
        assert_eq!(r.next_event(), ArgfileEvent::Exit);
        let mut r = reader("-stay_open\nFALSE\n");
        assert_eq!(r.next_event(), ArgfileEvent::Exit);
    }

    /// Redundant `-stay_open True` mid-stream is swallowed silently
    /// (divergence: ExifTool warns "-stay_open already active",
    /// exiftool:1276 - but an unsolicited stderr line between tasks kills
    /// the consumer's process pool, so we say nothing).
    #[test]
    fn test_redundant_stay_open_true_ignored() {
        let mut r = reader("-stay_open\nTrue\n-ver\n-execute\n");
        assert_eq!(
            r.next_event(),
            ArgfileEvent::Command(Command {
                args: vec!["-ver".to_string()],
                execute_id: String::new(),
            })
        );
    }

    #[test]
    fn test_eof_mid_command() {
        let mut r = reader("-ver\n");
        assert_eq!(
            r.next_event(),
            ArgfileEvent::Eof,
            "an unterminated command is never executed"
        );
    }

    #[test]
    fn test_comments_and_blanks_between_args() {
        let mut r = reader("# comment\n-ver\n\n   \n-execute\n");
        assert_eq!(
            r.next_event(),
            ArgfileEvent::Command(Command {
                args: vec!["-ver".to_string()],
                execute_id: String::new(),
            })
        );
    }

    #[test]
    fn test_seed_args_prepend_to_first_command() {
        let mut r = reader("-execute\n");
        r.seed_args(vec!["-ver".to_string()]);
        assert_eq!(
            r.next_event(),
            ArgfileEvent::Command(Command {
                args: vec!["-ver".to_string()],
                execute_id: String::new(),
            })
        );
    }
}
