//! XMP read-time value conversions
//!
//! Exact ports of ExifTool's two-layer XMP conversion, applied in
//! [`crate::xmp::processor`] as values are flattened out of the RDF/XML.
//!
//! ExifTool stores XMP values as raw packet text and converts them on read:
//!
//!  - **Layer 1** (structural, keyed on `Writable`) runs in FoundXMP
//!    (XMP.pm:3673-3687) before the value is stored: `rational` values go
//!    through [`convert_rational`] (XMP.pm:3400-3417) and `date` values through
//!    [`convert_xmp_date`] (XMP.pm:3383-3394). `real`/`integer`/`string` are
//!    left untouched — only `rational` is numeric-converted here.
//!  - **Layer 2** (per-tag ValueConv/PrintConv) runs afterwards for the `exif`
//!    namespace photo cluster (Image::ExifTool::XMP::exif, XMP.pm:2042-2166) via
//!    [`apply_exif_photo_conv`], reusing the existing EXIF conversion ports.

use std::sync::LazyLock;

use regex::Regex;

use crate::core::XmpTagInfo;
use crate::implementations::print_conv::{
    exposuretime_print_conv, fnumber_print_conv, print_fraction,
};
use crate::implementations::value_conv::{apex_aperture_value_conv, apex_shutter_speed_value_conv};
use crate::types::TagValue;

// NOTE: Perl's `$` matches before a trailing "\n"; Rust's does not. These
// regexes only see values already trimmed by the XML parser
// (trim_text(true) in processor.rs parse_xmp_xml), so the difference is
// unreachable — do not feed them untrimmed packet text.

/// XMP.pm:3402 `m{^(-?\d+)/(-?\d+)$}`
static RATIONAL_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(-?\d+)/(-?\d+)$").unwrap());

/// XMP.pm:3385 `^(\d{4})-(\d{2})-(\d{2})[T ](\d{2}:\d{2})(:\d{2})?\s*(\S*)$`
static FULL_DATE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\d{4})-(\d{2})-(\d{2})[T ](\d{2}:\d{2})(:\d{2})?\s*(\S*)$").unwrap()
});

/// XMP.pm:3390 `^(\d{4})(-\d{2}){0,2}` (partial date, not anchored at end)
static PARTIAL_DATE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d{4})(-\d{2}){0,2}").unwrap());

/// Layer 1: apply the `Writable`-driven conversion ExifTool performs in FoundXMP
/// (XMP.pm:3673-3687) before storing an XMP value.
///
/// Only raw scalar text is converted; RDF containers (arrays/objects) and any
/// non-`rational`/`date` format pass through unchanged.
pub fn apply_writable_conversion(tag_info: Option<&XmpTagInfo>, value: TagValue) -> TagValue {
    let Some(info) = tag_info else {
        return value;
    };
    let Some(writable) = info.writable else {
        return value;
    };
    let s = match &value {
        TagValue::String(s) => s.clone(),
        _ => return value,
    };
    match writable {
        // XMP.pm:3676 (($new or $fmt eq 'rational') and ConvertRational($val))
        "rational" => convert_rational(&s).unwrap_or(value),
        // XMP.pm:3681 ConvertXMPDate($val, $new) if $new or $fmt eq 'date'
        "date" => convert_xmp_date(&s),
        _ => value,
    }
}

/// Port of ExifTool XMP.pm:3400-3417 `ConvertRational`.
///
/// Parses `"N/D"`; returns the numeric quotient (integer-valued results stay
/// integers, matching ExifTool's JSON), `"inf"` for a non-zero numerator over a
/// zero denominator, or `"undef"` for `0/0`. Returns `None` when the string is
/// not a rational, so the caller keeps the original value.
pub fn convert_rational(val: &str) -> Option<TagValue> {
    let caps = RATIONAL_RE.captures(val)?;
    // The regex guarantees digit runs; parse as f64 to avoid overflow on wide
    // numerators (ExifTool computes `$1 / $2` in Perl's floating arithmetic).
    let num: f64 = caps[1].parse().ok()?;
    let den: f64 = caps[2].parse().ok()?;
    Some(if den != 0.0 {
        numeric_value(num / den)
    } else if num != 0.0 {
        TagValue::string("inf")
    } else {
        TagValue::string("undef")
    })
}

/// Port of ExifTool XMP.pm:3383-3394 `ConvertXMPDate` (with `$unsure` false, as it
/// is for every known `date` tag reached via FoundXMP).
///
/// Converts ISO `YYYY-MM-DDThh:mm[:ss][tz]` to EXIF `YYYY:MM:DD hh:mm[:ss][tz]`,
/// keeping optional seconds, fractional seconds, and the timezone verbatim; a bare
/// `YYYY[-MM[-DD]]` has its dashes turned into colons.
pub fn convert_xmp_date(val: &str) -> TagValue {
    if let Some(caps) = FULL_DATE_RE.captures(val) {
        // XMP.pm:3387 my $s = $5 || '';  (seconds may be missing)
        let seconds = caps.get(5).map_or("", |m| m.as_str());
        let timezone = caps.get(6).map_or("", |m| m.as_str());
        // XMP.pm:3388 $val = "$1:$2:$3 $4$s$6";
        return TagValue::string(format!(
            "{}:{}:{} {}{}{}",
            &caps[1], &caps[2], &caps[3], &caps[4], seconds, timezone
        ));
    }
    if PARTIAL_DATE_RE.is_match(val) {
        // XMP.pm:3391 $val =~ tr/-/:/;
        return TagValue::string(val.replace('-', ":"));
    }
    TagValue::string(val.to_string())
}

/// Layer 2: per-tag ValueConv + PrintConv for the XMP `exif` namespace photo
/// cluster (Image::ExifTool::XMP::exif, XMP.pm:2042-2166). `value` is the Layer-1
/// (`ConvertRational`) numeric value.
///
/// Returns `Some((value, print))` for the handled tags — `value` is the
/// ValueConv result (unchanged where the tag has no ValueConv) and `print` is the
/// PrintConv result. Returns `None` for every other tag so the caller keeps the
/// value and applies the generic Simple-lookup PrintConv path.
pub fn apply_exif_photo_conv(name: &str, value: &TagValue) -> Option<(TagValue, TagValue)> {
    match name {
        // XMP.pm:2042-2046 ExposureTime: PrintExposureTime, no ValueConv.
        "ExposureTime" => Some((value.clone(), exposuretime_print_conv(value, None))),
        // XMP.pm:2047-2051 FNumber: PrintFNumber, no ValueConv.
        "FNumber" => Some((value.clone(), fnumber_print_conv(value, None))),
        // XMP.pm:2081-2087 ShutterSpeedValue: ValueConv 'abs($val)<100 ? 1/(2**$val) : 0',
        // PrintConv PrintExposureTime. apex_shutter_speed_value_conv is the exact
        // 2**(-$val) port carrying the same abs()<100 guard.
        "ShutterSpeedValue" => {
            let converted =
                apex_shutter_speed_value_conv(value, None).unwrap_or_else(|_| value.clone());
            let print = exposuretime_print_conv(&converted, None);
            Some((converted, print))
        }
        // XMP.pm:2088-2094 / 2103-2109 ApertureValue & MaxApertureValue:
        // ValueConv 'sqrt(2) ** $val' (== 2**($val/2), apex_aperture_value_conv),
        // PrintConv 'sprintf("%.1f",$val)'.
        "ApertureValue" | "MaxApertureValue" => {
            let converted = apex_aperture_value_conv(value, None).unwrap_or_else(|_| value.clone());
            let print = aperture_print_conv(&converted);
            Some((converted, print))
        }
        // XMP.pm:2161-2166 FocalLength: PrintConv 'sprintf("%.1f mm",$val)', no ValueConv.
        "FocalLength" => Some((value.clone(), focal_length_print_conv(value))),
        // XMP.pm:2096-2101 ExposureBiasValue (Name 'ExposureCompensation'):
        // PrintConv 'Image::ExifTool::Exif::PrintFraction($val)', no ValueConv.
        "ExposureCompensation" => Some((value.clone(), print_fraction(value, None))),
        // XMP.pm:2110-2115 SubjectDistance:
        // PrintConv '$val =~ /^(inf|undef)$/ ? $val : "$val m"', no ValueConv.
        "SubjectDistance" => Some((value.clone(), subject_distance_print_conv(value))),
        _ => None,
    }
}

/// XMP.pm:2113 SubjectDistance PrintConv `$val =~ /^(inf|undef)$/ ? $val : "$val m"`.
///
/// The `inf`/`undef` strings come from [`convert_rational`] on zero-denominator
/// values and pass through; everything else gets the ` m` unit suffix.
fn subject_distance_print_conv(value: &TagValue) -> TagValue {
    match value {
        TagValue::String(s) if s == "inf" || s == "undef" => value.clone(),
        _ => TagValue::string(format!("{value} m")),
    }
}

/// XMP.pm:2090 ApertureValue/MaxApertureValue PrintConv `sprintf("%.1f",$val)`.
///
/// ExifTool emits the sprintf result as a JSON number (e.g. `8.0`), so we round to
/// one decimal place and keep a numeric TagValue to match that serialization.
fn aperture_print_conv(value: &TagValue) -> TagValue {
    match value.as_f64() {
        Some(v) => TagValue::F64((v * 10.0).round() / 10.0),
        None => value.clone(),
    }
}

/// XMP.pm:2163 FocalLength PrintConv `sprintf("%.1f mm",$val)`.
fn focal_length_print_conv(value: &TagValue) -> TagValue {
    match value.as_f64() {
        Some(v) => TagValue::string(format!("{v:.1} mm")),
        None => value.clone(),
    }
}

/// Integer-valued quotients stay integers (ExifTool encodes `8/1` as JSON `8`, not
/// `8.0`); fractional results are floats. Positive whole values beyond i32 (e.g.
/// `4294967295/1` distance sentinels) use U64 to keep the integer JSON encoding.
fn numeric_value(quotient: f64) -> TagValue {
    if quotient.fract() == 0.0 && quotient.abs() <= i32::MAX as f64 {
        TagValue::I32(quotient as i32)
    } else if quotient.fract() == 0.0 && quotient > 0.0 && quotient <= u64::MAX as f64 {
        TagValue::U64(quotient as u64)
    } else {
        TagValue::F64(quotient)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_rational_integer_quotient() {
        // "8/1" -> 8 (integer, matching ExifTool's JSON encoding)
        assert_eq!(convert_rational("8/1"), Some(TagValue::I32(8)));
    }

    #[test]
    fn convert_rational_fractional_quotient() {
        // "123/10" -> 12.3, "8321928/1000000" -> 8.321928
        assert_eq!(convert_rational("123/10"), Some(TagValue::F64(12.3)));
        assert_eq!(
            convert_rational("8321928/1000000"),
            Some(TagValue::F64(8.321928))
        );
    }

    #[test]
    fn convert_rational_negative() {
        assert_eq!(convert_rational("-1/2"), Some(TagValue::F64(-0.5)));
    }

    #[test]
    fn convert_rational_zero_denominator() {
        // XMP.pm:3410-3414: non-zero numerator -> 'inf', 0/0 -> 'undef'
        assert_eq!(convert_rational("1/0"), Some(TagValue::string("inf")));
        assert_eq!(convert_rational("0/0"), Some(TagValue::string("undef")));
    }

    #[test]
    fn convert_rational_non_rational_passes_through() {
        assert_eq!(convert_rational("hello"), None);
        assert_eq!(convert_rational("1.5"), None);
        assert_eq!(convert_rational("1/2/3"), None);
    }

    #[test]
    fn convert_xmp_date_full_with_timezone() {
        assert_eq!(
            convert_xmp_date("2005-06-08T12:05:36+01:00"),
            TagValue::string("2005:06:08 12:05:36+01:00")
        );
    }

    #[test]
    fn convert_xmp_date_trailing_z() {
        assert_eq!(
            convert_xmp_date("2018-01-15T22:22:01Z"),
            TagValue::string("2018:01:15 22:22:01Z")
        );
    }

    #[test]
    fn convert_xmp_date_fractional_seconds() {
        assert_eq!(
            convert_xmp_date("2018-01-15T14:22:02.185"),
            TagValue::string("2018:01:15 14:22:02.185")
        );
    }

    #[test]
    fn convert_xmp_date_no_seconds() {
        // XMP.pm:3387 seconds group optional -> "$s" empty
        assert_eq!(
            convert_xmp_date("2024-01-15T10:30"),
            TagValue::string("2024:01:15 10:30")
        );
    }

    #[test]
    fn exposure_compensation_print_fraction() {
        // exif:ExposureBiasValue "-1/3": Layer 1 -> -0.333..., PrintFraction -> "-1/3"
        // (XMP.pm:2096-2101; regression caught in review — raw text happened to
        // match ExifTool before Layer 1 existed)
        let layer1 = convert_rational("-1/3").unwrap();
        let (_, print) = apply_exif_photo_conv("ExposureCompensation", &layer1).unwrap();
        assert_eq!(print, TagValue::string("-1/3"));
        // zero keeps the numeric JSON form (see print_fraction's zero branch)
        let zero = convert_rational("0/1").unwrap();
        let (_, print) = apply_exif_photo_conv("ExposureCompensation", &zero).unwrap();
        assert_eq!(print, TagValue::F64(0.0));
    }

    #[test]
    fn subject_distance_unit_suffix() {
        // XMP.pm:2110-2115: "10/1" -> "10 m"; inf/undef pass through unsuffixed
        let layer1 = convert_rational("10/1").unwrap();
        let (_, print) = apply_exif_photo_conv("SubjectDistance", &layer1).unwrap();
        assert_eq!(print, TagValue::string("10 m"));
        let inf = convert_rational("1/0").unwrap();
        let (_, print) = apply_exif_photo_conv("SubjectDistance", &inf).unwrap();
        assert_eq!(print, TagValue::string("inf"));
    }

    #[test]
    fn convert_rational_large_whole_quotient_stays_integer() {
        // e.g. exifEX ApproximateFocusDistance sentinel 4294967295/1
        assert_eq!(
            convert_rational("4294967295/1"),
            Some(TagValue::U64(4294967295))
        );
    }

    #[test]
    fn convert_xmp_date_bare_date() {
        // XMP.pm:3390-3391 partial date -> tr/-/:/
        assert_eq!(
            convert_xmp_date("2005-06-08"),
            TagValue::string("2005:06:08")
        );
        assert_eq!(convert_xmp_date("2005-06"), TagValue::string("2005:06"));
        assert_eq!(convert_xmp_date("2005"), TagValue::string("2005"));
    }
}
