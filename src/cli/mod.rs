//! ExifTool-compatible command-line handling.
//!
//! This lives in the library (not `main.rs`) so both CLI modes share it and
//! unit tests don't need to spawn the binary:
//!
//! - classic argv mode (`exif-oxide -Orientation# image.jpg`), and
//! - `-stay_open True -@ -` REPL mode used by exiftool-vendored.js.
//!
//! The split mirrors ExifTool's own layering: an argfile reader that chunks
//! `-stay_open` input into commands (exiftool `ReadStayOpen`, :4925-4987) and
//! an option parser that turns one command's args into work
//! (exiftool's `Command:` loop, :400+).

pub mod argfile;
pub mod options;
pub mod stay_open;

pub use options::{parse_command, ParsedCommand};
pub use stay_open::{collect_entries, detect_stay_open};

/// The exact one-line UTF-8-repair Perl filter exiftool-vendored.js sends as
/// the value of `-api` in every default ReadTask payload
/// (../exiftool-vendored.js/src/Utf8JsonFilter.ts:13-53, joined to one line).
///
/// Pinned here as a contract fixture: parser and argfile tests feed it
/// verbatim to prove an option value is never mistaken for a file path or a
/// command terminator.
pub const READTASK_UTF8_FILTER: &str = r#"Filter=if (Image::ExifTool::IsUTF8(\$_) < 0) { my $binary; { package DB; for (my $i = 0; ; ++$i) { my @caller = caller($i); last unless @caller; next unless $caller[3] eq "Image::ExifTool::Filter"; my $arg = $DB::args[2]; if (ref($arg) && ref($$arg) eq "SCALAR") { $binary = 1; last; } } } unless ($binary) { my $raw = $_; tr/\0//d; Image::ExifTool::XMP::FixUTF8( \$_, Image::ExifTool::PackUTF8(0xfffd) ) if Image::ExifTool::IsUTF8(\$_) < 0; $_ = { "__etvInvalidUtf8V1" => { replacement => "s:" . $_, rawBase64 => "b64:" . Image::ExifTool::XMP::EncodeBase64($raw, 1) } }; } }"#;
