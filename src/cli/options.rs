//! ExifTool-style command parsing that returns instead of printing or exiting.
//!
//! `parse_command` is the single arg parser for both CLI modes. It must never
//! call `process::exit`, print, or panic: in `-stay_open` mode a parse problem
//! is a per-command error surfaced on stderr while the task is pending, not a
//! process death (ExifTool: "NEVER say die", exiftool:348).
//!
//! The option set mirrors the vendored ExifTool script's argument loop
//! (third-party/exiftool/exiftool). Options are matched before the tag-request
//! fallthrough, exactly like ExifTool, where `-TagName` is the last resort
//! after every option pattern has failed (exiftool:689-1500).
//!
//! Only options exif-oxide can honor (or safely ignore while emulating
//! `exiftool -j -struct -G` output) are accepted; everything else still falls
//! through to the strict tag-request classifier so junk fails loudly.

use crate::hash::ImageHashType;
use crate::types::{FilterOptions, TagRequest};

/// The result of parsing one command's arguments.
///
/// Nothing here has been printed or executed; the caller decides how errors
/// and echoes surface (classic mode: stderr + exit codes; stay_open mode:
/// per-task stderr lines followed by `{ready}`).
#[derive(Debug, Clone, Default)]
pub struct ParsedCommand {
    /// Positional file paths (includes the `-` stdin marker).
    pub files: Vec<String>,
    /// Tag filtering built from the request list, in command-line order.
    pub filter: FilterOptions,
    /// `-ver` was seen: print [`crate::EXIFTOOL_VERSION`] before processing
    /// files. ExifTool prints the version and still processes any files given
    /// (exiftool:779-793 runs inside the normal command loop).
    pub print_version: bool,
    /// `-echo TEXT` lines, printed to stdout before processing
    /// (exiftool:1016-1028 prints them during option parsing).
    pub echo_stdout: Vec<String>,
    /// `-echo2 TEXT` lines, printed to stderr before processing.
    pub echo_stderr: Vec<String>,
    /// Parse errors ("Unknown option -x", missing option values). Non-empty
    /// means the command is bad and must not execute; ExifTool sets `$badCmd`
    /// and flushes the remaining arguments unparsed (exiftool:688), so at most
    /// one error is recorded per command.
    pub errors: Vec<String>,
}

/// Parse one command's arguments the way ExifTool's argument loop does,
/// without printing or exiting.
pub fn parse_command<S: AsRef<str>>(args: &[S]) -> ParsedCommand {
    let mut files: Vec<String> = Vec::new();
    let mut tag_requests: Vec<TagRequest> = Vec::new();
    let mut cmd = ParsedCommand::default();
    // -api requesttags=imagedatahash / imagehashtype=X (ExifTool forum topic
    // 14706; sent by exiftool-vendored.js ReadTask.ts:140-142).
    let mut compute_image_hash = false;
    let mut image_hash_type = ImageHashType::Md5;

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_ref();
        i += 1;

        // ExifTool flushes the rest of a bad command without parsing it
        // (exiftool:688 `next if $badCmd`), so one bad option => one error.
        if !cmd.errors.is_empty() {
            continue;
        }

        let Some(rest) = arg.strip_prefix('-') else {
            files.push(arg.to_string());
            continue;
        };
        if rest.is_empty() {
            // Bare "-": read from stdin (exiftool:1392).
            files.push(arg.to_string());
            continue;
        }

        // ExifTool lowercases into $a for the case-insensitive matches and
        // keeps $_ for the case-sensitive ones (exiftool:696).
        let lower = rest.to_ascii_lowercase();

        // --- no-value options -------------------------------------------------
        if lower == "ver" {
            // exiftool:779: `$a eq 'ver'`. Printed by the caller.
            cmd.print_version = true;
        } else if lower == "j" || lower == "json" {
            // exiftool:940 `/^(csv|j(son)?)(\+?=.*)?$/i`: JSON output is the
            // only format exif-oxide produces, so this is a no-op.
        } else if lower == "struct" || lower == "-struct" {
            // exiftool:1294 `/^(-)?struct$/i`: we always emit structured
            // output (`-struct` on, `--struct` accepted but ignored).
        } else if is_group_flag(&lower) {
            // exiftool:1078 `/^(g)(roupHeadings|roupNames)?([\d:]*)$/i`:
            // -g/-G/-G1/-g0:1/... exif-oxide emulates ONLY `-G` output mode
            // (decision 3, _todo/20260830-P1-stay-open-m1a.md), so all group
            // flags are accepted no-ops.
        } else if lower == "a" || lower == "-a" || lower == "duplicates" || lower == "-duplicates" {
            // exiftool:874 `/^(-?)(a|duplicates)$/i` (accept both polarities).
        } else if rest == "e" || rest == "-e" || lower == "composite" || lower == "-composite" {
            // exiftool:1010-1011: -e/--composite disable and --e/-composite
            // enable Composite tags; case-sensitive `e` (uppercase -E is
            // escapeHTML, which we do not support).
        } else if lower == "q" || lower == "quiet" {
            // exiftool:1227 `/^q(uiet)?$/i`.
        } else if lower == "m" || lower == "ignoreminorerrors" {
            // exiftool:1179 `/^(m|ignoreminorerrors)$/i`. [minor] gating is
            // deferred to M3 (decision 4).
        } else if is_fast_flag(&lower) {
            // exiftool:1063 `/^fast(\d*)$/i`: we have no slow-scan mode to
            // turn off, so FastScan levels are accepted no-ops.

            // --- value-taking options ---------------------------------------------
        } else if lower == "api" {
            // exiftool:875-885: `-api OPT[=VAL]`.
            match next_value(args, &mut i) {
                None => cmd
                    .errors
                    .push("Expecting option name for -api option".to_string()),
                Some(opt) => {
                    let (name, value) = match opt.split_once('=') {
                        Some((n, v)) => (n, v),
                        // exiftool:878: a bare option name means value 1.
                        None => (opt.as_str(), "1"),
                    };
                    if name.eq_ignore_ascii_case("requesttags") {
                        // ExifTool RequestTags accepts a comma-separated list.
                        if value
                            .split(',')
                            .any(|t| t.trim().eq_ignore_ascii_case("imagedatahash"))
                        {
                            compute_image_hash = true;
                        }
                    } else if name.eq_ignore_ascii_case("imagehashtype") {
                        match value.to_ascii_uppercase().as_str() {
                            "MD5" => image_hash_type = ImageHashType::Md5,
                            "SHA256" => image_hash_type = ImageHashType::Sha256,
                            "SHA512" => image_hash_type = ImageHashType::Sha512,
                            other => cmd.errors.push(format!(
                                "Invalid -api imagehashtype value '{other}' (expected MD5, SHA256, or SHA512)"
                            )),
                        }
                    }
                    // All other API options (Filter=..., struct=1, keepUTCTime,
                    // geolocation, ...) are accepted and ignored in M1a.
                }
            }
        } else if lower == "use" {
            // exiftool:1308-1310. MWG composite semantics are M3; the value is
            // consumed so it can't become a phantom file path.
            if next_value(args, &mut i).is_none() {
                cmd.errors
                    .push("Expecting module name for -use option".to_string());
            }
        } else if rest == "x" || lower == "exclude" {
            // exiftool:1366-1368; case-sensitive `x` (uppercase -X is XML
            // output, unsupported). Exclusion semantics are M3 (decision 5):
            // accept and ignore.
            if next_value(args, &mut i).is_none() {
                cmd.errors
                    .push("Expecting tag name for -x option".to_string());
            }
        } else if lower == "charset" {
            // exiftool:907-908: optional value, consumed only when the next
            // argument does not start with a dash.
            if i < args.len() && !args[i].as_ref().starts_with('-') {
                i += 1;
            }
        } else if let Some(n) = echo_number(&lower) {
            // exiftool:1016-1028 `/^echo(\d)?$/i`: value consumed even for
            // invalid numbers; missing value is silently ignored.
            if let Some(text) = next_value(args, &mut i) {
                match n {
                    1 => cmd.echo_stdout.push(text),
                    2 => cmd.echo_stderr.push(text),
                    // -echo3/-echo4 print after processing; skipped in M1a.
                    // n > 4 draws a Warn in ExifTool; ignored here.
                    _ => {}
                }
            }
        } else if rest == "w" || lower == "textout" {
            // exiftool %optArgs '-w' (output file format). Unsupported: the
            // value is consumed and ignored so it can't become a file path.
            if next_value(args, &mut i).is_none() {
                cmd.errors
                    .push("Expecting argument for -w option".to_string());
            }
        } else if rest == "d" || lower == "dateformat" {
            // Case-sensitive `d` (uppercase -D is decimal tag IDs, unsupported).
            if next_value(args, &mut i).is_none() {
                cmd.errors
                    .push("Expecting argument for -d option".to_string());
            }
        } else if rest == "c" || lower == "coordformat" {
            if next_value(args, &mut i).is_none() {
                cmd.errors
                    .push("Expecting argument for -c option".to_string());
            }
        } else if is_if_flag(&lower) {
            // exiftool:1131-1140 `/^if(\d*)$/i`: conditions are unsupported in
            // M1a; the expression is consumed and ignored (every file is
            // processed as if the condition passed).
            if next_value(args, &mut i).is_none() {
                cmd.errors
                    .push("Expecting expression for -if option".to_string());
            }
        } else if lower == "stay_open" {
            // exiftool:1268-1293. In a parsed command this is only meaningful
            // when a -@ argfile is active; the stay_open REPL intercepts the
            // pair before commands are parsed, so here it's accepted and
            // ignored (`-stay_open True` without `-@` does nothing in ExifTool
            // either until a -@ option arrives).
            if next_value(args, &mut i).is_none() {
                cmd.errors
                    .push("Expecting argument for -stay_open option".to_string());
            }
        } else if rest == "@" {
            // Nested/na argfiles are out of scope for M1a; fail loudly instead
            // of treating the value as a file path.
            let _ = next_value(args, &mut i);
            cmd.errors
                .push("The -@ option is only supported as '-stay_open True -@ -'".to_string());

        // --- tag-request fallthrough ------------------------------------------
        } else {
            // Keep this classification byte-for-byte with the parity
            // regression suites (tests below and src/compat/filtering.rs).
            //
            // ExifTool's own guard is `length $_ eq 1 and $_ ne '*'`
            // (exiftool:1393): a one-character argument is an error unless it
            // is exactly `*`, and anything longer is a tag request. exif-oxide
            // is stricter about two-character arguments because it implements
            // none of ExifTool's remaining short options (-n, -b, -s, ...),
            // and failing loudly on those beats silently matching no tags. A
            // wildcard can never be one of those options, so `-*`, `-*#` and
            // `-?#` are let through - matching ExifTool, which accepts all
            // three and rejects only a bare `-?`.
            let filter_arg = rest;
            let is_tag_request = match filter_arg.len() {
                1 => filter_arg == "*",
                2 => FilterOptions::has_wildcard(filter_arg),
                _ => true,
            };
            if !is_tag_request {
                cmd.errors.push(format!("Unknown option {arg}"));
                continue;
            }

            // Record the request in command-line order; classification happens
            // in FilterOptions::from_requests so the CLI and the compat filter
            // parser cannot drift apart.
            tag_requests.push(TagRequest::parse(filter_arg));
        }
    }

    cmd.filter = FilterOptions::from_requests(tag_requests);
    if compute_image_hash {
        cmd.filter.compute_image_hash = true;
        cmd.filter.image_hash_type = image_hash_type;
    }
    cmd.files = files;
    cmd
}

/// Consume the next argument as an option value, if there is one.
fn next_value<S: AsRef<str>>(args: &[S], i: &mut usize) -> Option<String> {
    if *i < args.len() {
        let v = args[*i].as_ref().to_string();
        *i += 1;
        Some(v)
    } else {
        None
    }
}

/// `/^(g)(roupHeadings|roupNames)?([\d:]*)$/i` (exiftool:1078).
fn is_group_flag(lower: &str) -> bool {
    let Some(rest) = lower.strip_prefix('g') else {
        return false;
    };
    let rest = rest
        .strip_prefix("roupheadings")
        .or_else(|| rest.strip_prefix("roupnames"))
        .unwrap_or(rest);
    rest.chars().all(|c| c.is_ascii_digit() || c == ':')
}

/// `/^fast(\d*)$/i` (exiftool:1063).
fn is_fast_flag(lower: &str) -> bool {
    lower
        .strip_prefix("fast")
        .is_some_and(|d| d.chars().all(|c| c.is_ascii_digit()))
}

/// `/^if(\d*)$/i` (exiftool:1131).
fn is_if_flag(lower: &str) -> bool {
    lower
        .strip_prefix("if")
        .is_some_and(|d| d.chars().all(|c| c.is_ascii_digit()))
}

/// `/^echo(\d)?$/i` (exiftool:1016): returns the echo number (default 1), or
/// None when the argument isn't an echo option at all.
fn echo_number(lower: &str) -> Option<u8> {
    let rest = lower.strip_prefix("echo")?;
    match rest.len() {
        0 => Some(1),
        1 => rest
            .chars()
            .next()
            .and_then(|c| c.to_digit(10))
            // Perl's `$1 || 1` (exiftool:1017): a literal 0 counts as 1.
            .map(|d| if d == 0 { 1 } else { d as u8 }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Adapter that preserves the shape of the original main.rs parity suite:
    /// the assertions below are byte-for-byte from that suite and protect the
    /// tag-request classification against drift.
    fn parse_exiftool_args(args: Vec<&str>) -> (Vec<String>, FilterOptions) {
        let parsed = parse_command(&args);
        assert!(
            parsed.errors.is_empty(),
            "unexpected parse errors: {:?}",
            parsed.errors
        );
        (parsed.files, parsed.filter)
    }

    #[test]
    fn test_parse_exiftool_args_files_before_tags() {
        let (files, filter_opts) = parse_exiftool_args(vec![
            "image1.jpg",
            "image2.png",
            "-FNumber#",
            "-ExposureTime#",
        ]);

        assert_eq!(files, vec!["image1.jpg", "image2.png"]);
        assert!(filter_opts.requested_tags.contains(&"FNumber".to_string()));
        assert!(filter_opts
            .requested_tags
            .contains(&"ExposureTime".to_string()));
        assert_eq!(
            filter_opts.tag_requests,
            vec![
                TagRequest::new("FNumber", true),
                TagRequest::new("ExposureTime", true),
            ]
        );
        assert_eq!(filter_opts.requested_tags.len(), 2);
    }

    #[test]
    fn test_parse_exiftool_args_group_all_patterns() {
        let (files, filter_opts) = parse_exiftool_args(vec!["image.jpg", "-File:all", "-EXIF:all"]);

        assert_eq!(files, vec!["image.jpg"]);
        assert!(filter_opts
            .group_all_patterns
            .contains(&"File:all".to_string()));
        assert!(filter_opts
            .group_all_patterns
            .contains(&"EXIF:all".to_string()));
        assert_eq!(filter_opts.group_all_patterns.len(), 2);
        assert!(!filter_opts.extract_all);
    }

    #[test]
    fn test_parse_exiftool_args_extract_all() {
        let (files, filter_opts) = parse_exiftool_args(vec!["image.jpg", "-all"]);

        assert_eq!(files, vec!["image.jpg"]);
        assert!(filter_opts.extract_all);
        assert!(filter_opts.requested_tags.is_empty());
        assert!(filter_opts.group_all_patterns.is_empty());
    }

    #[test]
    fn test_parse_exiftool_args_numeric_tags() {
        let (files, filter_opts) =
            parse_exiftool_args(vec!["image.jpg", "-Orientation#", "-FNumber"]);

        assert_eq!(files, vec!["image.jpg"]);
        assert!(filter_opts
            .requested_tags
            .contains(&"Orientation".to_string()));
        assert!(filter_opts.requested_tags.contains(&"FNumber".to_string()));
        assert!(filter_opts.should_use_numeric("Orientation", "EXIF"));
        assert!(!filter_opts.should_use_numeric("FNumber", "EXIF"));
    }

    /// The request list keeps command-line order, and each request keeps its own `#`
    /// flag, because the first request that matches a tag decides how it is printed.
    ///
    /// Probed against vendored ExifTool 13.59 with test-images/apple/IMG_3755.MOV:
    /// `exiftool -j -Duration "-*Duration*#"` => `"Duration": "2.96 s"`
    /// `exiftool -j "-*Duration*#" -Duration` => `"Duration": 2.965`
    #[test]
    fn test_parse_exiftool_args_preserves_request_order() {
        let (_, print_first) = parse_exiftool_args(vec!["video.mov", "-Duration", "-*Duration*#"]);
        assert_eq!(
            print_first.tag_requests,
            vec![
                TagRequest::new("Duration", false),
                TagRequest::new("*Duration*", true),
            ]
        );
        assert!(!print_first.should_use_numeric("Duration", "QuickTime"));
        assert!(print_first.should_use_numeric("TrackDuration", "QuickTime"));

        let (_, numeric_first) =
            parse_exiftool_args(vec!["video.mov", "-*Duration*#", "-Duration"]);
        assert_eq!(
            numeric_first.tag_requests,
            vec![
                TagRequest::new("*Duration*", true),
                TagRequest::new("Duration", false),
            ]
        );
        assert!(numeric_first.should_use_numeric("Duration", "QuickTime"));
    }

    /// `-Group:all#` must strip the `#` before classifying, or `EXIF:all` is mistaken
    /// for a tag name and the group is never extracted.
    ///
    /// Probed (ExifTool 13.59, test-images/canon/eos_5d_mark_iii.jpg):
    /// `exiftool -j -G "-EXIF:all#"` => 54 EXIF tags with `"EXIF:Orientation": 1`
    #[test]
    fn test_parse_exiftool_args_group_all_numeric() {
        let (_, filter_opts) = parse_exiftool_args(vec!["image.jpg", "-EXIF:all#"]);

        assert_eq!(filter_opts.group_all_patterns, vec!["EXIF:all"]);
        assert!(!filter_opts.extract_all);
        assert!(filter_opts.should_extract_tag("Orientation", "EXIF"));
        assert!(filter_opts.should_use_numeric("Orientation", "EXIF"));
    }

    /// `-Group:*` stays a glob so the group is actually extracted, and `--TAG` is
    /// never silently turned into `-TAG`.
    ///
    /// Probed (ExifTool 13.59, test-images/canon/eos_5d_mark_iii.jpg):
    /// `exiftool -j -G "-EXIF:*"`  => the whole EXIF group
    /// `exiftool -j -G "--GPS*"`   => every tag EXCEPT the GPS* ones (exclusion)
    #[test]
    fn test_parse_exiftool_args_group_glob_and_double_dash() {
        let (_, group_glob) = parse_exiftool_args(vec!["image.jpg", "-EXIF:*"]);
        assert_eq!(group_glob.glob_patterns, vec!["EXIF:*"]);
        assert!(group_glob.group_all_patterns.is_empty());
        assert!(group_glob.should_extract_tag("Make", "EXIF"));

        let (_, excluded) = parse_exiftool_args(vec!["image.jpg", "--GPS*"]);
        assert!(
            !excluded.should_extract_tag("GPSVersionID", "EXIF"),
            "--GPS* is ExifTool's exclusion syntax, never an inclusion"
        );

        let (_, all_alias) = parse_exiftool_args(vec!["image.jpg", "--all"]);
        assert!(all_alias.extract_all, "--all is a spelling of -all");
    }

    /// The unknown-option guard used to reject every argument of two characters or
    /// fewer, which swallowed `-*` and `-*#` - legitimate ExifTool requests for every
    /// tag.
    ///
    /// ExifTool: lib/Image/ExifTool.pm:5367 (`$tag =~ /^(\*|all)$/i`) makes `*` a
    /// request for all tags, and exiftool:1393
    /// (`length $_ eq 1 and $_ ne '*' and Error(...)`) is the guard that lets it
    /// through. Verified: `exiftool -j -G "-*#" canon/eos_rebel_t3i.jpg` returns every
    /// tag with its ValueConv value, identical to `exiftool -j -G "-all#"`; `-*`, `-?*`,
    /// `-*?` and `-?#` are likewise accepted while a bare `-?` is "Unknown option".
    /// Before this guard was relaxed, `exif-oxide "-*#" image.jpg` printed
    /// "Unknown option -*#" and exited 1.
    #[test]
    fn test_parse_exiftool_args_bare_wildcard() {
        let (files, filter_opts) = parse_exiftool_args(vec!["image.jpg", "-*#"]);

        assert_eq!(files, vec!["image.jpg"]);
        assert_eq!(filter_opts.tag_requests, vec![TagRequest::new("*", true)]);
        assert!(filter_opts.should_extract_tag("Orientation", "EXIF"));
        assert!(filter_opts.should_use_numeric("Orientation", "EXIF"));

        let (files, filter_opts) = parse_exiftool_args(vec!["image.jpg", "-*"]);

        assert_eq!(files, vec!["image.jpg"]);
        assert_eq!(filter_opts.tag_requests, vec![TagRequest::new("*", false)]);
        assert!(filter_opts.should_extract_tag("Orientation", "EXIF"));
        assert!(!filter_opts.should_use_numeric("Orientation", "EXIF"));

        // Two-character wildcard requests are accepted too, matching ExifTool.
        let (_, filter_opts) = parse_exiftool_args(vec!["image.jpg", "-?#"]);
        assert!(filter_opts.glob_patterns.contains(&"?".to_string()));
    }

    /// `-all#` is `-all` with numeric output: every tag, ValueConv values.
    ///
    /// ExifTool: lib/Image/ExifTool.pm:5364 strips the `#`, :5367 expands `all`.
    /// Verified: `exiftool -j -G "-all#" canon/eos_rebel_t3i.jpg`.
    #[test]
    fn test_parse_exiftool_args_all_numeric() {
        let (files, filter_opts) = parse_exiftool_args(vec!["image.jpg", "-all#"]);

        assert_eq!(files, vec!["image.jpg"]);
        assert!(
            !filter_opts.extract_all,
            "-all# must stay a filtered request so the numeric selection still applies"
        );
        assert_eq!(filter_opts.tag_requests, vec![TagRequest::new("all", true)]);
        assert!(filter_opts.should_extract_tag("Orientation", "EXIF"));
        assert!(filter_opts.should_use_numeric("Orientation", "EXIF"));
    }

    #[test]
    fn test_parse_exiftool_args_edge_cases() {
        // Test with stdin marker "-"
        let (files, filter_opts) = parse_exiftool_args(vec!["-", "-FNumber"]);
        assert_eq!(files, vec!["-"]);
        assert!(filter_opts.requested_tags.contains(&"FNumber".to_string()));

        // Test with no filters (should default to extract_all)
        let (files, filter_opts) = parse_exiftool_args(vec!["image.jpg"]);
        assert_eq!(files, vec!["image.jpg"]);
        assert!(filter_opts.extract_all);
    }

    #[test]
    fn test_parse_exiftool_args_compatibility_flags() {
        // Test ExifTool compatibility flags are ignored as no-ops
        let (files, filter_opts) =
            parse_exiftool_args(vec!["image.jpg", "-j", "-struct", "-G", "-FNumber"]);

        // Should have only the image file, compatibility flags ignored
        assert_eq!(files, vec!["image.jpg"]);
        // Should only have the FNumber tag, not the compatibility flags
        assert_eq!(filter_opts.requested_tags.len(), 1);
        assert!(filter_opts.requested_tags.contains(&"FNumber".to_string()));
        assert!(!filter_opts.extract_all);
    }

    #[test]
    fn test_parse_exiftool_args_compatibility_flags_only() {
        // Test with only compatibility flags (should default to extract_all)
        let (files, filter_opts) = parse_exiftool_args(vec!["image.jpg", "-j", "-struct", "-G"]);

        // Should have only the image file
        assert_eq!(files, vec!["image.jpg"]);
        // Since no actual tag filters were specified, should default to extract_all
        assert!(filter_opts.extract_all);
        assert!(filter_opts.requested_tags.is_empty());
    }

    #[test]
    fn test_parse_exiftool_args_boundary_lengths() {
        // Test boundary cases for filter length validation - only valid 3+ char tags accepted
        let (files, filter_opts) = parse_exiftool_args(vec!["image.jpg", "-abc"]);

        // Should have only the image file
        assert_eq!(files, vec!["image.jpg"]);
        // Should only have the 3-character tag
        assert_eq!(filter_opts.requested_tags.len(), 1);
        assert!(filter_opts.requested_tags.contains(&"abc".to_string()));
        assert!(!filter_opts.extract_all);
    }

    #[test]
    fn test_parse_exiftool_args_all_compatibility_flags() {
        // Test all compatibility flags together
        let (files, filter_opts) =
            parse_exiftool_args(vec!["image.jpg", "-j", "-struct", "-G", "-MIMEType"]);

        // Should have only the image file
        assert_eq!(files, vec!["image.jpg"]);
        // Should only have the valid tag, compatibility flags ignored
        assert_eq!(filter_opts.requested_tags.len(), 1);
        assert!(filter_opts.requested_tags.contains(&"MIMEType".to_string()));
        assert!(!filter_opts.extract_all);
    }

    // ---- new option-table behavior (M1a) -----------------------------------

    /// The full exiftool-vendored.js default ReadTask payload must parse with
    /// only the target file as a positional argument.
    /// Payload shape: ../exiftool-vendored.js/src/ReadTask.ts:118-158.
    #[test]
    fn test_readtask_default_payload_option_values_consumed() {
        let parsed = parse_command(&[
            "-json",
            "-fast",
            "-api",
            crate::cli::READTASK_UTF8_FILTER,
            "-api",
            "struct=1",
            "-use",
            "MWG",
            "-api",
            "keepUTCTime",
            "-*Duration*#",
            "-GPSAltitude#",
            "-GPSLatitude#",
            "-GPSLongitude#",
            "-GPSPosition#",
            "-GeolocationPosition#",
            "-Orientation#",
            "-all",
            "/tmp/photo.jpg",
            "-ignoreMinorErrors",
        ]);
        assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);
        assert_eq!(parsed.files, vec!["/tmp/photo.jpg"]);
        // -all plus earlier numeric requests: everything is extracted, and the
        // numeric requests still control printing.
        assert!(parsed.filter.extract_all);
        assert!(parsed.filter.should_use_numeric("Orientation", "EXIF"));
        assert!(parsed
            .filter
            .should_use_numeric("TrackDuration", "QuickTime"));
    }

    /// PhotoStructure adds `-x Group:Tag` pairs and the image-hash API options.
    #[test]
    fn test_api_image_hash_mapping_and_exclusions() {
        let parsed = parse_command(&[
            "-api",
            "requesttags=imagedatahash",
            "-api",
            "imagehashtype=SHA256",
            "-x",
            "Composite:LensSpec",
            "-exclude",
            "XMP:HistoryChanged",
            "-all",
            "photo.jpg",
        ]);
        assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);
        assert_eq!(parsed.files, vec!["photo.jpg"]);
        assert!(parsed.filter.compute_image_hash);
        assert_eq!(parsed.filter.image_hash_type, ImageHashType::Sha256);
    }

    /// `-charset` takes an optional value: consumed only when the next
    /// argument does not start with a dash (exiftool:908).
    #[test]
    fn test_charset_optional_value() {
        let parsed = parse_command(&["-charset", "filename=utf8", "-all", "photo.jpg"]);
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.files, vec!["photo.jpg"]);

        let parsed = parse_command(&["-charset", "-Orientation", "photo.jpg"]);
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.files, vec!["photo.jpg"]);
        assert!(parsed
            .filter
            .requested_tags
            .contains(&"Orientation".to_string()));
    }

    /// `-echo`/`-echo2` values are captured for the caller to print
    /// (exiftool:1016-1028); `-echo3`/`-echo4` are consumed and skipped.
    #[test]
    fn test_echo_capture() {
        let parsed = parse_command(&[
            "-echo",
            "to stdout",
            "-echo2",
            "to stderr",
            "-echo3",
            "skipped",
            "-echo4",
            "skipped",
            "photo.jpg",
        ]);
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.echo_stdout, vec!["to stdout"]);
        assert_eq!(parsed.echo_stderr, vec!["to stderr"]);
        assert_eq!(parsed.files, vec!["photo.jpg"]);
    }

    /// A value line that happens to look like an option (`-execute` after
    /// `-if`) is still just a value; and `-ver` is a flag, not an exit.
    #[test]
    fn test_if_consumes_option_shaped_value() {
        let parsed = parse_command(&["-if", "-execute", "-ver"]);
        assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);
        assert!(parsed.print_version);
        assert!(parsed.files.is_empty());
    }

    /// Unknown options abort the command and flush the remaining args, exactly
    /// one error recorded (exiftool:688, :1393).
    #[test]
    fn test_unknown_option_flushes_remainder() {
        let parsed = parse_command(&["-zz", "-qq", "-Orientation", "photo.jpg"]);
        assert_eq!(parsed.errors, vec!["Unknown option -zz"]);
        assert!(
            parsed.files.is_empty(),
            "args after a bad option are flushed"
        );
        assert!(parsed.filter.requested_tags.is_empty());
    }

    /// `--` (empty tag request) stays rejected, matching the old CLI.
    #[test]
    fn test_double_dash_alone_rejected() {
        let parsed = parse_command(&["--", "photo.jpg"]);
        assert_eq!(parsed.errors, vec!["Unknown option --"]);
    }

    /// In-command `-stay_open` pairs are accepted no-ops (the stay_open REPL
    /// intercepts them before parsing; classic mode ignores them like ExifTool
    /// does without an active argfile, exiftool:1268-1293).
    #[test]
    fn test_stay_open_pair_ignored_in_command() {
        let parsed = parse_command(&["-stay_open", "True", "photo.jpg"]);
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.files, vec!["photo.jpg"]);
    }

    /// `-@` outside the supported spawn form fails loudly instead of turning
    /// its value into a file path.
    #[test]
    fn test_argfile_option_rejected_in_command() {
        let parsed = parse_command(&["-@", "args.txt", "photo.jpg"]);
        assert_eq!(
            parsed.errors,
            vec!["The -@ option is only supported as '-stay_open True -@ -'"]
        );
        assert!(parsed.files.is_empty());
    }

    /// Missing option values are command errors, not silent successes.
    #[test]
    fn test_missing_option_value_is_error() {
        let parsed = parse_command(&["-api"]);
        assert_eq!(parsed.errors, vec!["Expecting option name for -api option"]);
        let parsed = parse_command(&["-use"]);
        assert_eq!(parsed.errors, vec!["Expecting module name for -use option"]);
        let parsed = parse_command(&["-x"]);
        assert_eq!(parsed.errors, vec!["Expecting tag name for -x option"]);
    }
}
