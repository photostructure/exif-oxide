//! Serialization support for TagValue, including ExifTool-compatible JSON numeric detection

use crate::core::TagValue;
use regex::Regex;
use serde::{Serialize, Serializer};
use std::sync::LazyLock;

/// Check if a string matches ExifTool's JSON numeric pattern
/// ExifTool: exiftool:3762 EscapeJSON function
/// Regex: /^-?(\d|[1-9]\d{1,14})(\.\d{1,16})?(e[-+]?\d{1,3})?$/i
///
/// ## Why String→Regex→Number (Not Direct Numeric Types)?
///
/// PrintConv functions return strings that may be numeric ("2.8") or descriptive ("Unknown").
/// ExifTool's proven architecture: Raw → ValueConv → PrintConv → String → EscapeJSON → JSON
/// This regex gracefully handles mixed outputs without complex tag categorization that would
/// drift from ExifTool compatibility and miss edge cases in real-world camera firmware.
///
/// From ExifTool source:
/// ```perl
/// sub EscapeJSON($;$)
/// {
///     my ($str, $quote) = @_;
///     unless ($quote) {
///         # JSON boolean (true or false)
///         return lc($str) if $str =~ /^(true|false)$/i and $json < 2;
///         # JSON/PHP number (see json.org for numerical format)
///         return $str if $str =~ /^-?(\d|[1-9]\d{1,14})(\.\d{1,16})?(e[-+]?\d{1,3})?$/i;
///     }
///     # ... string escaping logic
/// }
/// ```
pub fn is_json_numeric_string(s: &str) -> bool {
    // ExifTool: exiftool:3762 - exact regex from EscapeJSON function
    // JSON/PHP number format validation per json.org specification
    static NUMERIC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^-?(\d|[1-9]\d{1,14})(\.\d{1,16})?(e[-+]?\d{1,3})?$")
            .expect("Invalid ExifTool numeric regex")
    });

    // ExifTool: Case-insensitive matching (note the 'i' flag in ExifTool regex)
    NUMERIC_REGEX.is_match(&s.to_lowercase())
}

/// Render an f64 as JSON the way Perl stringifies a number: a whole number has
/// no fractional part, so 4.0 prints as `4`, not `4.0`.
///
/// ExifTool emits Perl's own stringification of a numeric value - EscapeJSON
/// (exiftool:3801) returns `$str` unchanged once it matches the JSON number
/// pattern - and Perl prints `4` for `4/1`. Verified against the vendored
/// ExifTool: `exiftool -FNumber# -FocalLength# -j test-images/canon/eos_60d.jpg`
/// reports `"FNumber": 4, "FocalLength": 55`.
///
/// Values at or beyond 2^63 stay floats: `as i64` would saturate there and
/// silently change the number. Non-finite values have no JSON representation and
/// become `null`, which is what serde_json already does for them.
fn perl_number(v: f64) -> serde_json::Value {
    // i64::MAX as f64 rounds up to exactly 2^63, so the upper bound is exclusive.
    if v.is_finite() && v.fract() == 0.0 && v >= (i64::MIN as f64) && v < (i64::MAX as f64) {
        serde_json::Value::from(v as i64)
    } else {
        serde_json::Number::from_f64(v).map_or(serde_json::Value::Null, serde_json::Value::Number)
    }
}

/// Render one rational as JSON, matching ExifTool's GetRational64u/GetRational64s
/// (lib/Image/ExifTool.pm:6114-6120): a zero denominator yields the strings
/// 'undef' (0/0) or 'inf', otherwise the quotient is emitted as a number.
fn rational_to_json(num: f64, denom: f64) -> serde_json::Value {
    if denom == 0.0 {
        let marker = if num == 0.0 { "undef" } else { "inf" };
        serde_json::Value::String(marker.to_string())
    } else {
        perl_number(num / denom)
    }
}

impl Serialize for TagValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            TagValue::U8(v) => serializer.serialize_u8(*v),
            TagValue::U16(v) => serializer.serialize_u16(*v),
            TagValue::U32(v) => serializer.serialize_u32(*v),
            TagValue::U64(v) => serializer.serialize_u64(*v),
            TagValue::I16(v) => serializer.serialize_i16(*v),
            TagValue::I32(v) => serializer.serialize_i32(*v),
            TagValue::F64(v) => perl_number(*v).serialize(serializer),
            TagValue::String(s) => {
                // ExifTool: exiftool:3762 EscapeJSON function - JSON numeric conversion
                // If string matches JSON number pattern, return unquoted (as number)
                // This matches ExifTool's behavior for JSON output format
                if is_json_numeric_string(s) {
                    // ExifTool: Simply returns the string unquoted for JSON numbers
                    // Parse to ensure proper JSON number format in Rust
                    if let Ok(int_val) = s.parse::<i64>() {
                        return serializer.serialize_i64(int_val);
                    }
                    if let Ok(float_val) = s.parse::<f64>() {
                        if float_val.is_finite() {
                            return serializer.serialize_f64(float_val);
                        }
                    }
                }

                // ExifTool: Falls through to string escaping if not numeric
                serializer.serialize_str(s)
            }
            TagValue::Bool(b) => serializer.serialize_bool(*b),
            TagValue::U8Array(arr) => arr.serialize(serializer),
            TagValue::U16Array(arr) => arr.serialize(serializer),
            TagValue::U32Array(arr) => arr.serialize(serializer),
            TagValue::F64Array(arr) => arr
                .iter()
                .map(|v| perl_number(*v))
                .collect::<Vec<_>>()
                .serialize(serializer),
            // ExifTool: GetRational64u/GetRational64s divide numerator by denominator
            // (lib/Image/ExifTool.pm:6114-6120).
            TagValue::Rational(num, denom) => {
                rational_to_json(*num as f64, *denom as f64).serialize(serializer)
            }
            TagValue::SRational(num, denom) => {
                rational_to_json(*num as f64, *denom as f64).serialize(serializer)
            }
            TagValue::RationalArray(arr) => {
                // ExifTool: Convert each rational to decimal like GetRational64u
                let converted: Vec<serde_json::Value> = arr
                    .iter()
                    .map(|(num, denom)| rational_to_json(*num as f64, *denom as f64))
                    .collect();
                converted.serialize(serializer)
            }
            TagValue::SRationalArray(arr) => {
                // ExifTool: Convert each signed rational to decimal like GetRational64s
                let converted: Vec<serde_json::Value> = arr
                    .iter()
                    .map(|(num, denom)| rational_to_json(*num as f64, *denom as f64))
                    .collect();
                converted.serialize(serializer)
            }
            TagValue::Binary(data) => data.serialize(serializer),
            TagValue::Object(map) => map.serialize(serializer),
            TagValue::Array(values) => values.serialize(serializer),
            TagValue::Empty => serializer.serialize_str("undef"), // ExifTool compatibility
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::TagValue;

    fn json(v: &TagValue) -> String {
        serde_json::to_string(v).expect("TagValue serializes")
    }

    /// ExifTool emits Perl's stringification of a number, so a whole-numbered
    /// float prints without a fractional part.
    ///
    /// Probes against the vendored ExifTool (third-party/exiftool/exiftool):
    ///   $ exiftool -FNumber# -FocalLength# -ExposureTime# -j \
    ///         test-images/canon/eos_60d.jpg
    ///     "FNumber": 4, "FocalLength": 55, "ExposureTime": 0.0005
    ///   $ perl -e 'print 4/1'    -> 4
    ///   $ perl -e 'print 15/10'  -> 1.5
    #[test]
    fn test_f64_whole_numbers_serialize_without_trailing_zero() {
        assert_eq!(json(&TagValue::F64(4.0)), "4");
        assert_eq!(json(&TagValue::F64(55.0)), "55");
        assert_eq!(json(&TagValue::F64(0.0)), "0");
        assert_eq!(json(&TagValue::F64(-7.0)), "-7");
    }

    /// Fractional values keep every digit they need.
    #[test]
    fn test_f64_fractional_values_are_unchanged() {
        assert_eq!(json(&TagValue::F64(1.5)), "1.5");
        assert_eq!(json(&TagValue::F64(2.965)), "2.965");
        assert_eq!(json(&TagValue::F64(0.5)), "0.5");
        assert_eq!(json(&TagValue::F64(0.0005)), "0.0005");
        assert_eq!(json(&TagValue::F64(-2.8)), "-2.8");
    }

    /// Whole-number floats too large for i64 must not be funnelled through the
    /// integer path, where `as i64` would saturate at i64::MAX and change the
    /// value. Perl prints these in exponent form (`perl -e 'print 1e19'` ->
    /// `1e+19`); serde_json writes `1e19`. Both are the same JSON number.
    #[test]
    fn test_f64_beyond_i64_range_round_trips() {
        for v in [1e19_f64, -1e19_f64, 1e300_f64, f64::MAX, f64::MIN] {
            let s = json(&TagValue::F64(v));
            assert_eq!(
                serde_json::from_str::<f64>(&s).unwrap(),
                v,
                "{v} must round-trip, serialized as {s}"
            );
        }
    }

    /// Non-finite floats are unreachable from Perl (division by zero dies there),
    /// and serde_json maps them to JSON `null`. Pinned so the whole-number branch
    /// is never allowed to swallow them: NaN and infinity have no integer form.
    #[test]
    fn test_non_finite_f64_stays_null() {
        assert_eq!(json(&TagValue::F64(f64::INFINITY)), "null");
        assert_eq!(json(&TagValue::F64(f64::NEG_INFINITY)), "null");
        assert_eq!(json(&TagValue::F64(f64::NAN)), "null");
    }

    /// Rationals divide before serializing (ExifTool: GetRational64u,
    /// lib/Image/ExifTool.pm:6017-6023), so they hit the same rule.
    ///   $ exiftool -XResolution# -j test-images/canon/eos_60d.jpg -> 72
    #[test]
    fn test_rational_whole_numbers_serialize_without_trailing_zero() {
        assert_eq!(json(&TagValue::Rational(72, 1)), "72");
        assert_eq!(json(&TagValue::SRational(-72, 1)), "-72");
        assert_eq!(json(&TagValue::Rational(1, 2)), "0.5");
        assert_eq!(
            json(&TagValue::RationalArray(vec![(72, 1), (1, 2)])),
            "[72,0.5]"
        );
        assert_eq!(
            json(&TagValue::SRationalArray(vec![(-72, 1), (1, 2)])),
            "[-72,0.5]"
        );
    }

    #[test]
    fn test_f64_array_serializes_whole_numbers_without_trailing_zero() {
        assert_eq!(json(&TagValue::F64Array(vec![4.0, 1.5])), "[4,1.5]");
    }

    /// PrintConv results arrive as strings and ExifTool emits numeric-looking ones
    /// unquoted (EscapeJSON, exiftool:3801), so "8.0" must not collapse to "8" the
    /// way an F64 does. This is why PrintFNumber returns sprintf's string.
    ///   $ exiftool -FNumber -j test-images/canon/eos_1ds_mark_ii.jpg -> 8.0
    #[test]
    fn test_numeric_strings_do_not_collapse_to_integers() {
        assert_eq!(json(&TagValue::String("8.0".to_string())), "8.0");
        assert_eq!(json(&TagValue::String("4".to_string())), "4");
        assert_eq!(json(&TagValue::String("Off".to_string())), "\"Off\"");
    }

    /// KNOWN GAP, pinned so it is visible rather than surprising: ExifTool returns
    /// the matched string *unchanged* (`return $str`, exiftool:3810), but this
    /// branch parses to f64 and re-renders, so digits beyond f64's shortest
    /// round-trip form are lost.
    ///
    ///   $ exiftool -EXIF:Software -G -j test-images/casio/ex_zr100.jpg
    ///     "EXIF:Software": 1.00        <- exif-oxide emits 1.0
    ///
    /// Affects 53 literals across generated/exiftool-json (mostly `EXIF:Software`
    /// and `Composite:Megapixels`). Fixing it needs the output pipeline to carry
    /// the literal token - serde_json's `Value` round-trip in
    /// `extract_metadata_json` discards it - so it is deliberately out of scope
    /// for the division/whole-number-float fix.
    #[test]
    fn test_numeric_strings_lose_redundant_precision() {
        assert_eq!(json(&TagValue::String("1.00".to_string())), "1.0");
        assert_eq!(json(&TagValue::String("0.70".to_string())), "0.7");
    }

    /// Display is the `to_string()` path used by generated string concatenation
    /// (e.g. Canon SelfTimer's `... . ' s'`). Rust's f64 Display already matches
    /// Perl for these; lock it in so the fix can rely on it.
    #[test]
    fn test_display_matches_perl_stringification() {
        assert_eq!(TagValue::F64(1.5).to_string(), "1.5");
        assert_eq!(TagValue::F64(4.0).to_string(), "4");
        assert_eq!(TagValue::F64(10.0).to_string(), "10");
    }
}
