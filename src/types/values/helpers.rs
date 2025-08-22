//! Helper functions for PPI-generated code
//!
//! These functions simplify codegen complexity for common operations.
//! P07: PPI Enhancement - Task A support functions

use super::TagValue;

/// Split a TagValue by separator - helper for generated PPI code
/// ExifTool equivalent: split(" ", $val)
pub fn split_tagvalue(val: &TagValue, separator: &str) -> TagValue {
    match val {
        TagValue::String(s) => {
            let parts: Vec<TagValue> = s
                .split(separator)
                .map(|part| TagValue::String(part.to_string()))
                .collect();
            TagValue::Array(parts)
        }
        TagValue::Array(arr) => {
            // If already an array, join and re-split (ExifTool behavior)
            let joined = arr
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            let parts: Vec<TagValue> = joined
                .split(separator)
                .map(|part| TagValue::String(part.to_string()))
                .collect();
            TagValue::Array(parts)
        }
        _ => {
            // For non-string types, convert to string first
            let s = val.to_string();
            let parts: Vec<TagValue> = s
                .split(separator)
                .map(|part| TagValue::String(part.to_string()))
                .collect();
            TagValue::Array(parts)
        }
    }
}