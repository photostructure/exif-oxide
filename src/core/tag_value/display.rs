//! Display formatting implementation for TagValue

use crate::core::TagValue;
use std::fmt;

impl fmt::Display for TagValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TagValue::U8(v) => write!(f, "{v}"),
            TagValue::U16(v) => write!(f, "{v}"),
            TagValue::U32(v) => write!(f, "{v}"),
            TagValue::U64(v) => write!(f, "{v}"),
            TagValue::I16(v) => write!(f, "{v}"),
            TagValue::I32(v) => write!(f, "{v}"),
            TagValue::F64(v) => write!(f, "{v}"),
            TagValue::String(s) => write!(f, "{s}"),
            TagValue::Bool(b) => write!(f, "{b}"),
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
            TagValue::Empty => write!(f, "undef"),
        }
    }
}
