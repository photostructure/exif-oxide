//! Serialization support for TagValue, including ExifTool-compatible JSON numeric detection

use super::TagValue;
use regex::Regex;
use serde::{Serializer, Serialize};
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
            TagValue::F64(v) => serializer.serialize_f64(*v),
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
            TagValue::U8Array(arr) => arr.serialize(serializer),
            TagValue::U16Array(arr) => arr.serialize(serializer),
            TagValue::U32Array(arr) => arr.serialize(serializer),
            TagValue::F64Array(arr) => arr.serialize(serializer),
            TagValue::Rational(num, denom) => {
                // ExifTool: GetRational64u automatically divides numerator by denominator
                // lib/Image/ExifTool.pm:6017-6023 - returns RoundFloat($ratNumer / $ratDenom, 10)
                if *denom == 0 {
                    // ExifTool: returns 'inf' for division by zero with non-zero numerator
                    if *num == 0 {
                        serializer.serialize_str("undef") // ExifTool: 0/0 case
                    } else {
                        serializer.serialize_str("inf") // ExifTool: n/0 case
                    }
                } else {
                    // ExifTool: Normal case - divide and serialize as float with 10 significant digits
                    let result = *num as f64 / *denom as f64;
                    serializer.serialize_f64(result)
                }
            }
            TagValue::SRational(num, denom) => {
                // ExifTool: GetRational64s automatically divides numerator by denominator (signed version)
                // lib/Image/ExifTool.pm:6017-6023 - same logic as GetRational64u but for signed values
                if *denom == 0 {
                    // ExifTool: returns 'inf' for division by zero with non-zero numerator
                    if *num == 0 {
                        serializer.serialize_str("undef") // ExifTool: 0/0 case
                    } else {
                        serializer.serialize_str("inf") // ExifTool: n/0 case
                    }
                } else {
                    // ExifTool: Normal case - divide and serialize as float
                    let result = *num as f64 / *denom as f64;
                    serializer.serialize_f64(result)
                }
            }
            TagValue::RationalArray(arr) => {
                // ExifTool: Convert each rational to decimal like GetRational64u
                let converted: Vec<serde_json::Value> = arr
                    .iter()
                    .map(|(num, denom)| {
                        if *denom == 0 {
                            if *num == 0 {
                                serde_json::Value::String("undef".to_string())
                            } else {
                                serde_json::Value::String("inf".to_string())
                            }
                        } else {
                            serde_json::Value::Number(
                                serde_json::Number::from_f64(*num as f64 / *denom as f64)
                                    .unwrap_or_else(|| serde_json::Number::from(0)),
                            )
                        }
                    })
                    .collect();
                converted.serialize(serializer)
            }
            TagValue::SRationalArray(arr) => {
                // ExifTool: Convert each signed rational to decimal like GetRational64s
                let converted: Vec<serde_json::Value> = arr
                    .iter()
                    .map(|(num, denom)| {
                        if *denom == 0 {
                            if *num == 0 {
                                serde_json::Value::String("undef".to_string())
                            } else {
                                serde_json::Value::String("inf".to_string())
                            }
                        } else {
                            serde_json::Value::Number(
                                serde_json::Number::from_f64(*num as f64 / *denom as f64)
                                    .unwrap_or_else(|| serde_json::Number::from(0)),
                            )
                        }
                    })
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