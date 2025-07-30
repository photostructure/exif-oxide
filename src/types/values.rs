//! Tag value types and conversion utilities
//!
//! This module defines the core `TagValue` enum that represents all possible
//! EXIF tag values after parsing, along with its conversion methods and
//! display formatting.

use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;

/// Represents a tag value that can be of various types
///
/// ExifTool handles many different data types. This enum represents
/// the possible values a tag can contain after parsing.
///
/// # Creating String TagValues
///
/// There are several convenient ways to create string TagValues:
///
/// ```
/// use exif_oxide::types::TagValue;
///
/// // Most ergonomic - using From trait
/// let tag1: TagValue = "Hello".into();
/// let tag2 = TagValue::from("World");
///
/// // Using the convenience method
/// let tag3 = TagValue::string("Foo");
///
/// // Traditional way (still works)
/// let tag4 = TagValue::String("Bar".to_string());
///
/// // All create the same type
/// assert_eq!(tag1, TagValue::String("Hello".to_string()));
/// assert_eq!(tag2, TagValue::String("World".to_string()));
/// assert_eq!(tag3, TagValue::String("Foo".to_string()));
/// assert_eq!(tag4, TagValue::String("Bar".to_string()));
/// ```
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum TagValue {
    /// Unsigned 8-bit integer
    U8(u8),
    /// Unsigned 16-bit integer  
    U16(u16),
    /// Unsigned 32-bit integer
    U32(u32),
    /// Unsigned 64-bit integer
    U64(u64),
    /// Signed 16-bit integer
    I16(i16),
    /// Signed 32-bit integer
    I32(i32),
    /// Floating point number
    F64(f64),
    /// Text string
    String(String),
    /// Array of unsigned 8-bit integers (for binary data)
    U8Array(Vec<u8>),
    /// Array of unsigned 16-bit integers
    U16Array(Vec<u16>),
    /// Array of unsigned 32-bit integers  
    U32Array(Vec<u32>),
    /// Array of floating point numbers (for rational arrays)
    F64Array(Vec<f64>),
    /// Rational number as numerator/denominator pair (RATIONAL format)
    /// ExifTool: Format type 5 (rational64u) - 2x uint32
    Rational(u32, u32),
    /// Signed rational number as numerator/denominator pair (SRATIONAL format)  
    /// ExifTool: Format type 10 (rational64s) - 2x int32
    SRational(i32, i32),
    /// Array of rational numbers for multi-value tags like GPS coordinates
    /// ExifTool: GPSLatitude/GPSLongitude arrays [degrees/1, minutes/1, seconds/100]
    RationalArray(Vec<(u32, u32)>),
    /// Array of signed rational numbers
    SRationalArray(Vec<(i32, i32)>),
    /// Raw binary data when type is unknown
    Binary(Vec<u8>),
    /// Nested object for structured data (e.g., XMP structures)
    /// Used for hierarchical metadata like ContactInfo, LocationCreated
    Object(HashMap<String, TagValue>),
    /// Array of heterogeneous values (e.g., XMP RDF containers)
    /// Used for RDF Bag/Seq containers and mixed-type arrays
    Array(Vec<TagValue>),
}

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
fn is_json_numeric_string(s: &str) -> bool {
    use regex::Regex;
    use std::sync::LazyLock;

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
        }
    }
}

impl TagValue {
    /// Convert to u8 if possible
    pub fn as_u8(&self) -> Option<u8> {
        match self {
            TagValue::U8(v) => Some(*v),
            _ => None,
        }
    }

    /// Convert to u16 if possible
    pub fn as_u16(&self) -> Option<u16> {
        match self {
            TagValue::U16(v) => Some(*v),
            TagValue::U8(v) => Some(*v as u16),
            _ => None,
        }
    }

    /// Convert to u32 if possible
    pub fn as_u32(&self) -> Option<u32> {
        match self {
            TagValue::U32(v) => Some(*v),
            TagValue::U16(v) => Some(*v as u32),
            TagValue::U8(v) => Some(*v as u32),
            _ => None,
        }
    }

    /// Convert to string if possible
    pub fn as_string(&self) -> Option<&str> {
        match self {
            TagValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Convert to f64 if possible
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            TagValue::F64(v) => Some(*v),
            TagValue::U32(v) => Some(*v as f64),
            TagValue::U16(v) => Some(*v as f64),
            TagValue::U8(v) => Some(*v as f64),
            TagValue::Rational(num, denom) => {
                if *denom != 0 {
                    Some(*num as f64 / *denom as f64)
                } else {
                    None // Division by zero
                }
            }
            TagValue::SRational(num, denom) => {
                if *denom != 0 {
                    Some(*num as f64 / *denom as f64)
                } else {
                    None // Division by zero
                }
            }
            _ => None,
        }
    }

    /// Convert to rational tuple if possible
    pub fn as_rational(&self) -> Option<(u32, u32)> {
        match self {
            TagValue::Rational(num, denom) => Some((*num, *denom)),
            _ => None,
        }
    }

    /// Convert to signed rational tuple if possible
    pub fn as_srational(&self) -> Option<(i32, i32)> {
        match self {
            TagValue::SRational(num, denom) => Some((*num, *denom)),
            _ => None,
        }
    }

    // rational_to_decimal REMOVED in Milestone 8e
    // GPS coordinate conversion moved to Composite tag system

    // gps_to_decimal_with_ref REMOVED in Milestone 8e
    // GPS coordinate conversion moved to Composite tag system

    /// Get as object (HashMap) if this is an Object variant
    pub fn as_object(&self) -> Option<&HashMap<String, TagValue>> {
        match self {
            TagValue::Object(map) => Some(map),
            _ => None,
        }
    }

    /// Get as mutable object (HashMap) if this is an Object variant
    pub fn as_object_mut(&mut self) -> Option<&mut HashMap<String, TagValue>> {
        match self {
            TagValue::Object(map) => Some(map),
            _ => None,
        }
    }

    /// Get as array if this is an Array variant
    pub fn as_array(&self) -> Option<&Vec<TagValue>> {
        match self {
            TagValue::Array(vec) => Some(vec),
            _ => None,
        }
    }

    /// Get as mutable array if this is an Array variant
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<TagValue>> {
        match self {
            TagValue::Array(vec) => Some(vec),
            _ => None,
        }
    }
}

impl std::fmt::Display for TagValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TagValue::U8(v) => write!(f, "{v}"),
            TagValue::U16(v) => write!(f, "{v}"),
            TagValue::U32(v) => write!(f, "{v}"),
            TagValue::U64(v) => write!(f, "{v}"),
            TagValue::I16(v) => write!(f, "{v}"),
            TagValue::I32(v) => write!(f, "{v}"),
            TagValue::F64(v) => write!(f, "{v}"),
            TagValue::String(s) => write!(f, "{s}"),
            TagValue::U8Array(arr) => write!(f, "{arr:?}"),
            TagValue::U16Array(arr) => write!(f, "{arr:?}"),
            TagValue::U32Array(arr) => write!(f, "{arr:?}"),
            TagValue::F64Array(arr) => write!(f, "{arr:?}"),
            TagValue::Rational(num, denom) => {
                if *denom == 0 {
                    write!(f, "{num}/0 (inf)")
                } else if *denom == 1 {
                    write!(f, "{num}")
                } else {
                    write!(f, "{num}/{denom}")
                }
            }
            TagValue::SRational(num, denom) => {
                if *denom == 0 {
                    write!(f, "{num}/0 (inf)")
                } else if *denom == 1 {
                    write!(f, "{num}")
                } else {
                    write!(f, "{num}/{denom}")
                }
            }
            TagValue::RationalArray(arr) => {
                write!(f, "[")?;
                for (i, (num, denom)) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    if *denom == 0 {
                        write!(f, "{num}/0")?;
                    } else if *denom == 1 {
                        write!(f, "{num}")?;
                    } else {
                        write!(f, "{num}/{denom}")?;
                    }
                }
                write!(f, "]")
            }
            TagValue::SRationalArray(arr) => {
                write!(f, "[")?;
                for (i, (num, denom)) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    if *denom == 0 {
                        write!(f, "{num}/0")?;
                    } else if *denom == 1 {
                        write!(f, "{num}")?;
                    } else {
                        write!(f, "{num}/{denom}")?;
                    }
                }
                write!(f, "]")
            }
            TagValue::Binary(data) => write!(f, "[{} bytes of binary data]", data.len()),
            TagValue::Object(map) => {
                // For display, show as JSON-like structure
                write!(f, "{{")?;
                let mut first = true;
                for (key, value) in map {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, r#""{key}": {value}"#)?;
                    first = false;
                }
                write!(f, "}}")
            }
            TagValue::Array(values) => {
                // For display, show as JSON-like array
                write!(f, "[")?;
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{value}")?;
                }
                write!(f, "]")
            }
        }
    }
}

// Convenience implementations for easier TagValue creation
impl From<&str> for TagValue {
    fn from(s: &str) -> Self {
        TagValue::String(s.to_string())
    }
}

impl From<String> for TagValue {
    fn from(s: String) -> Self {
        TagValue::String(s)
    }
}

impl From<&String> for TagValue {
    fn from(s: &String) -> Self {
        TagValue::String(s.clone())
    }
}

impl TagValue {
    /// Convenience method for creating a string TagValue
    ///
    /// # Examples
    ///
    /// ```
    /// use exif_oxide::types::TagValue;
    ///
    /// let tag_value = TagValue::string("Hello");
    /// assert_eq!(tag_value, TagValue::String("Hello".to_string()));
    /// ```
    pub fn string<S: Into<String>>(s: S) -> Self {
        TagValue::String(s.into())
    }

    /// Create a TagValue that matches ExifTool's JSON numeric detection
    ///
    /// ExifTool applies a regex to detect if a string should be output as a JSON number.
    /// This function mimics ExifTool's behavior: if the string matches the numeric pattern,
    /// it returns an F64 variant; otherwise it returns a String variant.
    ///
    /// The regex pattern from ExifTool exiftool:3762:
    /// `^-?(\d|[1-9]\d{1,14})(\.\d{1,16})?(e[-+]?\d{1,3})?$`
    ///
    /// # Examples
    ///
    /// ```
    /// use exif_oxide::types::TagValue;
    ///
    /// // Numeric strings become F64 values
    /// let numeric = TagValue::string_with_numeric_detection("14.0");
    /// assert_eq!(numeric, TagValue::F64(14.0));
    ///
    /// // Non-numeric strings remain strings  
    /// let text = TagValue::string_with_numeric_detection("24.0 mm");
    /// assert_eq!(text, TagValue::String("24.0 mm".to_string()));
    /// ```
    pub fn string_with_numeric_detection<S: Into<String>>(s: S) -> Self {
        use regex::Regex;
        use std::sync::LazyLock;

        // ExifTool numeric detection regex - matches valid JSON numbers
        // From exiftool:3762: ^-?(\d|[1-9]\d{1,14})(\.\d{1,16})?(e[-+]?\d{1,3})?$
        static NUMERIC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"^-?(\d|[1-9]\d{1,14})(\.\d{1,16})?(e[-+]?\d{1,3})?$")
                .expect("Invalid numeric detection regex")
        });

        let string_val = s.into();

        // Check if string matches ExifTool's numeric pattern
        if NUMERIC_REGEX.is_match(&string_val) {
            // First try to parse as integer to preserve integer types
            // This matches ExifTool's JSON output format where integers are integers, not floats
            if let Ok(int_val) = string_val.parse::<i64>() {
                // Use appropriate integer type based on value range
                if int_val >= 0 {
                    if int_val <= u16::MAX as i64 {
                        return TagValue::U16(int_val as u16);
                    } else if int_val <= u32::MAX as i64 {
                        return TagValue::U32(int_val as u32);
                    } else {
                        return TagValue::U64(int_val as u64);
                    }
                } else if int_val >= i16::MIN as i64 {
                    return TagValue::I16(int_val as i16);
                } else if int_val >= i32::MIN as i64 {
                    return TagValue::I32(int_val as i32);
                } else {
                    // For very large negative numbers, fall back to F64
                    return TagValue::F64(int_val as f64);
                }
            }

            // If not a valid integer, try to parse as f64
            if let Ok(numeric_val) = string_val.parse::<f64>() {
                return TagValue::F64(numeric_val);
            }
        }

        // Not numeric or failed to parse - return as string
        TagValue::String(string_val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tagvalue_from_str() {
        let tag_value: TagValue = "Hello".into();
        assert_eq!(tag_value, TagValue::String("Hello".to_string()));
    }

    #[test]
    fn test_tagvalue_from_string() {
        let s = "World".to_string();
        let tag_value = TagValue::from(s);
        assert_eq!(tag_value, TagValue::String("World".to_string()));
    }

    #[test]
    fn test_tagvalue_from_string_ref() {
        let s = "Test".to_string();
        let tag_value = TagValue::from(&s);
        assert_eq!(tag_value, TagValue::String("Test".to_string()));
    }

    #[test]
    fn test_tagvalue_string_method() {
        let tag_value = TagValue::string("Convenience");
        assert_eq!(tag_value, TagValue::String("Convenience".to_string()));
    }

    #[test]
    fn test_tagvalue_string_method_with_owned_string() {
        let s = "Owned".to_string();
        let tag_value = TagValue::string(s);
        assert_eq!(tag_value, TagValue::String("Owned".to_string()));
    }

    #[test]
    fn test_all_string_creation_methods_equivalent() {
        let str_literal = "test";

        let tag1: TagValue = str_literal.into();
        let tag2 = TagValue::from(str_literal);
        let tag3 = TagValue::string(str_literal);
        let tag4 = TagValue::String(str_literal.to_string());

        assert_eq!(tag1, tag2);
        assert_eq!(tag2, tag3);
        assert_eq!(tag3, tag4);
    }

    #[test]
    fn test_object_variant() {
        let mut map = HashMap::new();
        map.insert("city".to_string(), TagValue::string("New York"));
        map.insert("country".to_string(), TagValue::string("USA"));

        let tag_value = TagValue::Object(map);

        assert!(tag_value.as_object().is_some());
        assert_eq!(tag_value.as_object().unwrap().len(), 2);
        assert_eq!(
            tag_value
                .as_object()
                .unwrap()
                .get("city")
                .unwrap()
                .as_string(),
            Some("New York")
        );
    }

    #[test]
    fn test_array_variant() {
        let values = vec![
            TagValue::string("keyword1"),
            TagValue::string("keyword2"),
            TagValue::U32(123),
        ];

        let tag_value = TagValue::Array(values);

        assert!(tag_value.as_array().is_some());
        assert_eq!(tag_value.as_array().unwrap().len(), 3);
        assert_eq!(
            tag_value.as_array().unwrap()[0].as_string(),
            Some("keyword1")
        );
    }

    #[test]
    fn test_nested_structures() {
        // Test nested XMP-like structure
        let mut contact_info = HashMap::new();
        contact_info.insert("CiAdrCity".to_string(), TagValue::string("Paris"));
        contact_info.insert("CiAdrCtry".to_string(), TagValue::string("France"));

        let mut main_object = HashMap::new();
        main_object.insert("ContactInfo".to_string(), TagValue::Object(contact_info));
        main_object.insert(
            "Keywords".to_string(),
            TagValue::Array(vec![TagValue::string("travel"), TagValue::string("europe")]),
        );

        let xmp = TagValue::Object(main_object);

        // Test access to nested data
        let contact = xmp
            .as_object()
            .unwrap()
            .get("ContactInfo")
            .unwrap()
            .as_object()
            .unwrap();
        assert_eq!(contact.get("CiAdrCity").unwrap().as_string(), Some("Paris"));

        let keywords = xmp
            .as_object()
            .unwrap()
            .get("Keywords")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(keywords.len(), 2);
    }

    #[test]
    fn test_display_formatting() {
        // Test Object display
        let mut map = HashMap::new();
        map.insert("key1".to_string(), TagValue::string("value1"));
        map.insert("key2".to_string(), TagValue::U32(42));
        let obj = TagValue::Object(map);
        let display = format!("{obj}");
        assert!(display.contains(r#""key1": value1"#) || display.contains(r#""key2": 42"#));

        // Test Array display
        let arr = TagValue::Array(vec![TagValue::string("item1"), TagValue::U32(123)]);
        assert_eq!(format!("{arr}"), "[item1, 123]");
    }

    #[test]
    fn test_string_with_numeric_detection() {
        // Numeric strings should become F64 values
        assert_eq!(
            TagValue::string_with_numeric_detection("14.0"),
            TagValue::F64(14.0)
        );
        assert_eq!(
            TagValue::string_with_numeric_detection("2.8"),
            TagValue::F64(2.8)
        );
        assert_eq!(
            TagValue::string_with_numeric_detection("-5.2"),
            TagValue::F64(-5.2)
        );
        assert_eq!(
            TagValue::string_with_numeric_detection("42"),
            TagValue::U16(42)
        );

        // Scientific notation
        assert_eq!(
            TagValue::string_with_numeric_detection("1.23e4"),
            TagValue::F64(12300.0)
        );
        assert_eq!(
            TagValue::string_with_numeric_detection("1.5e-3"),
            TagValue::F64(0.0015)
        );

        // Non-numeric strings should remain strings
        assert_eq!(
            TagValue::string_with_numeric_detection("24.0 mm"),
            TagValue::String("24.0 mm".to_string())
        );
        assert_eq!(
            TagValue::string_with_numeric_detection("1/200"),
            TagValue::String("1/200".to_string())
        );
        assert_eq!(
            TagValue::string_with_numeric_detection("f/2.8"),
            TagValue::String("f/2.8".to_string())
        );
        assert_eq!(
            TagValue::string_with_numeric_detection("text"),
            TagValue::String("text".to_string())
        );

        // Edge cases
        assert_eq!(
            TagValue::string_with_numeric_detection("0"),
            TagValue::U16(0)
        );
        assert_eq!(
            TagValue::string_with_numeric_detection("0.0"),
            TagValue::F64(0.0)
        );

        // Leading zeros not allowed for multi-digit numbers (ExifTool regex constraint)
        assert_eq!(
            TagValue::string_with_numeric_detection("01.5"),
            TagValue::String("01.5".to_string())
        );
    }
}
