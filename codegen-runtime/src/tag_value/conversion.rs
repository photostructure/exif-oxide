//! Conversion methods and From trait implementations for TagValue

use super::TagValue;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

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
            TagValue::I32(v) => Some(*v as f64),
            TagValue::I16(v) => Some(*v as f64),
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
    /// use codegen_runtime::TagValue;
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
    /// use codegen_runtime::TagValue;
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