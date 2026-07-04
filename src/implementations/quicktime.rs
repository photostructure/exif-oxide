//! QuickTime/MOV read-time value conversions
//!
//! Exact ports of the ExifTool QuickTime metadata conversions used by the
//! streaming atom walker in [`crate::formats::quicktime`]. Every function cites
//! the ExifTool source it translates (Trust ExifTool: docs/TRUST-EXIFTOOL.md).
//!
//! Two layers live here:
//!
//!  - **Core helpers** ([`patch_time_zero`], [`convert_unix_time`],
//!    [`convert_duration`], [`fix_wrong_format`]) are plain, testable ports the
//!    walker calls directly with the binary-table state it holds (movie
//!    TimeScale, per-mdhd MediaTS, box version).
//!  - **`(val, ctx)` registry wrappers** ([`convert_unix_time_quicktime`],
//!    [`convert_duration_print_conv`], [`media_duration_print_conv`]) match the
//!    codegen conversion-function signature so the generated `ast_*` stubs
//!    resolve to real calls. They are registered in
//!    `codegen/src/impl_registry/{valueconv,printconv}_registry.rs`.
//!
//! ExifTool ProcessBinaryData applies a per-field `RawConv` before `ValueConv`,
//! but the codegen pipeline only emits `ValueConv`/`PrintConv` (the
//! data-member-setting `RawConv`s are dropped). So the `%timeInfo` 1970-epoch
//! `RawConv` patch is folded into [`convert_unix_time_quicktime`] here, and the
//! walker applies the equivalent core helpers explicitly.

use crate::types::{ExifContext, Result, TagValue};

/// Seconds between the QuickTime epoch (1904-01-01) and the Unix epoch
/// (1970-01-01). ExifTool spells this `(66 * 365 + 17) * 24 * 3600`
/// (QuickTime.pm:259, MovieHeader RawConv:1361).
pub const QT_EPOCH_OFFSET: i64 = (66 * 365 + 17) * 24 * 3600; // 2082844800

/// Port of the `%timeInfo` RawConv (QuickTime.pm:257-269, duplicated for
/// CreateDate at MovieHeader:1359-1371).
///
/// QuickTime dates are seconds since 1904-01-01. Brain-dead software sometimes
/// writes 1970-epoch values instead; ExifTool subtracts the 66-year offset only
/// when `$val >= $offset` (a genuine 1904 value), otherwise it leaves the value
/// alone (and warns). `$val == 0` stays 0 → ConvertUnixTime renders it as the
/// zero date.
///
/// Accepted divergences from the Perl (review-vetted 2026-07-03):
/// - The RawConv also subtracts when the `QuickTimeUTC` *option* is set
///   (`$val >= $offset or $$self{OPTIONS}{QuickTimeUTC}`); exif-oxide exposes no
///   such option, so that branch is unreachable and deliberately not ported.
///   (Note: CR3's forced-UTC behavior lives in the ValueConv's
///   `FileType eq "CR3"` test, NOT in this RawConv branch.)
/// - Raw values above `i64::MAX` (v1 boxes are int64u) wrap negative and render
///   the zero date via ConvertUnixTime's range check; Perl instead produces
///   platform-dependent gmtime-overflow garbage. No real file has such dates.
pub fn patch_time_zero(raw: i64) -> i64 {
    if raw == 0 {
        return 0;
    }
    if raw >= QT_EPOCH_OFFSET {
        raw - QT_EPOCH_OFFSET
    } else {
        // "Patched incorrect time zero for QuickTime date/time tag" — ExifTool
        // keeps the (1970-epoch) value as-is rather than shifting it.
        raw
    }
}

/// Port of `ConvertUnixTime` (ExifTool.pm:6784) for the QuickTime case: integer
/// seconds, no fractional part (`$dec` resolves to 0 and is stripped), and no
/// `-api QuickTimeUTC`, so `$toLocal` is false → `gmtime` with an empty timezone
/// suffix. `$time == 0` renders the ExifTool zero date verbatim.
///
/// `to_local` is threaded through for the future CR3 path (which forces UTC
/// conversion to local time); the MOV walker always passes `false`, matching the
/// snapshots (tools/generate_exiftool_json.sh omits QuickTimeUTC).
pub fn convert_unix_time(unix: i64, to_local: bool) -> String {
    // ExifTool.pm:6787
    if unix == 0 {
        return "0000:00:00 00:00:00".to_string();
    }
    let Some(dt) = chrono::DateTime::from_timestamp(unix, 0) else {
        // Out-of-range timestamp: fall back to the zero date rather than panic.
        return "0000:00:00 00:00:00".to_string();
    };
    if to_local {
        // CR3 path (out of scope for this TPP): localtime + TimeZoneString.
        let local = dt.with_timezone(&chrono::Local);
        local.format("%Y:%m:%d %H:%M:%S%:z").to_string()
    } else {
        // ExifTool.pm:6798-6800,6808 — gmtime, `$tz = ''`.
        dt.format("%Y:%m:%d %H:%M:%S").to_string()
    }
}

/// Port of `ConvertDuration` (ExifTool.pm:6877-6895).
///
/// Renders a duration already divided into seconds (the `%durationInfo`/
/// MediaDuration ValueConv divides raw ticks by the TimeScale). Under 30 s it is
/// `"X.XX s"`; `0` is `"0 s"`; longer durations become `H:MM:SS` (with a leading
/// `N days ` when over 24 h). Negative values carry a `-` sign.
pub fn convert_duration(seconds: f64) -> String {
    // ExifTool.pm:6880-6881: `return $time unless IsFloat($time)` / `'0 s' if 0`.
    if !seconds.is_finite() {
        return seconds.to_string();
    }
    if seconds == 0.0 {
        return "0 s".to_string();
    }
    // ExifTool.pm:6882: sign extraction, then work with the absolute value.
    let (sign, mut time) = if seconds > 0.0 {
        ("", seconds)
    } else {
        ("-", -seconds)
    };
    // ExifTool.pm:6883
    if time < 30.0 {
        return format!("{sign}{time:.2} s");
    }
    // ExifTool.pm:6884-6894
    time += 0.5; // round off to nearest second
    let mut h = (time / 3600.0).trunc() as i64;
    time -= h as f64 * 3600.0;
    let m = (time / 60.0).trunc() as i64;
    time -= m as f64 * 60.0;
    let mut sign = sign.to_string();
    if h > 24 {
        let d = h / 24;
        h -= d * 24;
        sign = format!("{sign}{d} days ");
    }
    format!("{sign}{h}:{m:02}:{:02}", time.trunc() as i64)
}

/// Port of `FixWrongFormat` (QuickTime.pm:8872-8877), used by tkhd
/// ImageWidth/ImageHeight (TrackHeader:1575,1580).
///
/// The dimensions are 16.16 fixed-point int32u; Pentax writes them in the wrong
/// byte order. ExifTool: `return undef unless $val;` (0 → no tag) then
/// `$val & 0xfff00000 ? unpack('n',pack('N',$val)) : $val` — if the high bits are
/// set the value was byte-swapped, so take the low 16 bits big-endian; otherwise
/// the value is already the integer dimension (`1920 << 16` arrives as 1920).
pub fn fix_wrong_format(val: u32) -> Option<u32> {
    if val == 0 {
        return None;
    }
    if val & 0xfff0_0000 != 0 {
        // unpack('n', pack('N', $val)): high 16 bits of the big-endian packing.
        Some(val >> 16)
    } else {
        Some(val)
    }
}

// ---------------------------------------------------------------------------
// (val, ctx) registry wrappers — resolve the generated conversion stubs.
// ---------------------------------------------------------------------------

/// Does this context request UTC→local conversion? ExifTool ValueConv:
/// `$self->Options("QuickTimeUTC") || $$self{FileType} eq "CR3"` (%timeInfo:280).
/// The MOV walker passes `None`, so this is `false`.
fn wants_local_time(ctx: Option<&ExifContext>) -> bool {
    let Some(ctx) = ctx else {
        return false;
    };
    if ctx
        .get_data_member("QuickTimeUTC")
        .map(|v| v.is_truthy())
        .unwrap_or(false)
    {
        return true;
    }
    ctx.get_data_member("FileType")
        .and_then(|v| v.as_string())
        .map(|s| s == "CR3")
        .unwrap_or(false)
}

/// ValueConv wrapper for QuickTime date tags. Registered for the expression
/// `ConvertUnixTime($val, $self->Options("QuickTimeUTC") || $$self{FileType} eq "CR3")`
/// (QuickTime.pm:280).
///
/// Folds the `%timeInfo` RawConv 1970-epoch patch (QuickTime.pm:257) and
/// ConvertUnixTime (ExifTool.pm:6784), because codegen drops the DATAMEMBER
/// RawConv. The walker also drives the [`patch_time_zero`]/[`convert_unix_time`]
/// core helpers directly, so both paths agree.
pub fn convert_unix_time_quicktime(val: &TagValue, ctx: Option<&ExifContext>) -> Result<TagValue> {
    let Some(raw) = val.as_i64() else {
        return Ok(val.clone());
    };
    let unix = patch_time_zero(raw);
    Ok(TagValue::string(convert_unix_time(
        unix,
        wants_local_time(ctx),
    )))
}

/// PrintConv wrapper for the movie/track duration tags. Registered for
/// `$$self{TimeScale} ? ConvertDuration($val) : $val` (%durationInfo:315).
pub fn convert_duration_print_conv(val: &TagValue, ctx: Option<&ExifContext>) -> TagValue {
    duration_print_conv(val, ctx, "TimeScale")
}

/// PrintConv wrapper for MediaDuration. Registered for
/// `$$self{MediaTS} ? ConvertDuration($val) : $val` (MediaHeader:7271).
pub fn media_duration_print_conv(val: &TagValue, ctx: Option<&ExifContext>) -> TagValue {
    duration_print_conv(val, ctx, "MediaTS")
}

/// Shared body for the two duration PrintConv wrappers: format via
/// [`convert_duration`] when the timescale data member is truthy, else pass the
/// (already ValueConv-divided) value through unchanged, mirroring ExifTool's
/// `$$self{TimeScale} ? ConvertDuration($val) : $val`.
fn duration_print_conv(val: &TagValue, ctx: Option<&ExifContext>, member: &str) -> TagValue {
    let ts_truthy = ctx
        .and_then(|c| c.get_data_member(member))
        .map(|v| v.is_truthy())
        .unwrap_or(false);
    match val.as_f64() {
        Some(seconds) if ts_truthy => TagValue::string(convert_duration(seconds)),
        _ => val.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_time_zero_1904_and_1970() {
        // Genuine 1904-epoch value (>= offset) is shifted to Unix seconds.
        // 3324892129 (eos_500d mvhd CreateDate) -> 1242047329.
        assert_eq!(patch_time_zero(3_324_892_129), 1_242_047_329);
        // Brain-dead 1970-epoch value (< offset) is left untouched (ExifTool warns).
        assert_eq!(patch_time_zero(1_000_000_000), 1_000_000_000);
        // Zero stays zero (renders as the zero date).
        assert_eq!(patch_time_zero(0), 0);
    }

    #[test]
    fn convert_unix_time_gmtime() {
        assert_eq!(
            convert_unix_time(1_242_047_329, false),
            "2009:05:11 13:08:49"
        );
        assert_eq!(
            convert_unix_time(1_280_504_639, false),
            "2010:07:30 15:43:59"
        );
        assert_eq!(convert_unix_time(0, false), "0000:00:00 00:00:00");
    }

    #[test]
    fn convert_duration_seconds_and_hms() {
        assert_eq!(convert_duration(15000.0 / 2000.0), "7.50 s");
        assert_eq!(convert_duration(2980.0 / 600.0), "4.97 s");
        assert_eq!(convert_duration(0.0), "0 s");
        // A tiny non-zero duration rounds to "0.00 s" (apple metadata track).
        assert_eq!(convert_duration(0.0001), "0.00 s");
        // >= 30 s switches to H:MM:SS.
        assert_eq!(convert_duration(90.0), "0:01:30");
        // Negative sign preserved.
        assert_eq!(convert_duration(-2.96), "-2.96 s");
    }

    #[test]
    fn fix_wrong_format_fixed_point() {
        // 1920 << 16 has high bits set -> unpack('n',pack('N')) == val >> 16 == 1920.
        assert_eq!(fix_wrong_format(1920 << 16), Some(1920));
        assert_eq!(fix_wrong_format(1080 << 16), Some(1080));
        // 0 -> None (no tag, so audio tracks don't clobber ImageWidth).
        assert_eq!(fix_wrong_format(0), None);
        // Small raw value (Pentax's wrong format): high bits clear -> returned as-is.
        assert_eq!(fix_wrong_format(320), Some(320));
    }
}
