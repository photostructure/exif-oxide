//! Helper functions for working with arrays in generated code

use crate::TagValue;

/// Get an element from any array type at the specified index
///
/// This function handles all array types (U8Array, U16Array, U32Array, etc.)
/// and returns the element as a TagValue, or Empty if index is out of bounds.
pub fn get_array_element(val: &TagValue, index: usize) -> TagValue {
    match val {
        // Generic array of TagValues
        TagValue::Array(vec) => vec.get(index).cloned().unwrap_or(TagValue::Empty),
        // Typed arrays - convert element to appropriate TagValue
        TagValue::U8Array(vec) => vec
            .get(index)
            .map(|&v| TagValue::U8(v))
            .unwrap_or(TagValue::Empty),
        TagValue::U16Array(vec) => vec
            .get(index)
            .map(|&v| TagValue::U16(v))
            .unwrap_or(TagValue::Empty),
        TagValue::U32Array(vec) => vec
            .get(index)
            .map(|&v| TagValue::U32(v))
            .unwrap_or(TagValue::Empty),
        TagValue::F64Array(vec) => vec
            .get(index)
            .map(|&v| TagValue::F64(v))
            .unwrap_or(TagValue::Empty),
        TagValue::RationalArray(vec) => vec
            .get(index)
            .map(|&(n, d)| TagValue::Rational(n, d))
            .unwrap_or(TagValue::Empty),
        TagValue::SRationalArray(vec) => vec
            .get(index)
            .map(|&(n, d)| TagValue::SRational(n, d))
            .unwrap_or(TagValue::Empty),
        // Non-array types return Empty
        _ => TagValue::Empty,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_array_element_u32() {
        let arr = TagValue::U32Array(vec![45, 30, 15]);
        assert_eq!(get_array_element(&arr, 0), TagValue::U32(45));
        assert_eq!(get_array_element(&arr, 1), TagValue::U32(30));
        assert_eq!(get_array_element(&arr, 2), TagValue::U32(15));
        assert_eq!(get_array_element(&arr, 3), TagValue::Empty);
    }

    #[test]
    fn test_get_array_element_generic() {
        let arr = TagValue::Array(vec![
            TagValue::String("first".to_string()),
            TagValue::U32(42),
        ]);
        assert_eq!(
            get_array_element(&arr, 0),
            TagValue::String("first".to_string())
        );
        assert_eq!(get_array_element(&arr, 1), TagValue::U32(42));
        assert_eq!(get_array_element(&arr, 2), TagValue::Empty);
    }

    #[test]
    fn test_get_array_element_non_array() {
        let val = TagValue::U32(100);
        assert_eq!(get_array_element(&val, 0), TagValue::Empty);
    }
}
