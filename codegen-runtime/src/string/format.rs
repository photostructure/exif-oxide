//! String formatting helpers for cleaner generated code
//!
//! These functions provide idiomatic alternatives to repetitive string operations
//! while maintaining ExifTool compatibility.

use crate::TagValue;

/// Create a TagValue::String from any string-like input
///
/// Cleaner than `TagValue::String(expr.to_string())` everywhere
pub fn tag_string<S: Into<String>>(s: S) -> TagValue {
    TagValue::String(s.into())
}

/// Format multiple values into a string TagValue
///
/// Replaces repetitive `TagValue::String(format!(...))` patterns
pub fn format_tag(template: &str, args: &[TagValue]) -> TagValue {
    let formatted = crate::fmt::sprintf_perl(template, args);
    TagValue::String(formatted)
}

/// Convert value to string representation for concatenation
///
/// Handles the common `.to_string()` pattern with proper TagValue handling
pub fn stringify(val: &TagValue) -> String {
    match val {
        TagValue::String(s) => s.clone(),
        TagValue::Empty => String::new(),
        _ => val.to_string(),
    }
}

/// Concatenate two values as strings
///
/// ExifTool's `.` operator: cleaner than `format!("{}{}", left, right)`
pub fn concat(left: &TagValue, right: &TagValue) -> TagValue {
    let left_str = stringify(left);
    let right_str = stringify(right);
    TagValue::String(format!("{}{}", left_str, right_str))
}

/// Repeat a string n times
///
/// ExifTool's `x` operator: `$string x $count`
pub fn repeat_string(val: &TagValue, count: &TagValue) -> TagValue {
    let s = stringify(val);
    let n: f64 = count.clone().into();
    TagValue::String(s.repeat(n as usize))
}

/// Check if value is non-empty/defined
///
/// Common pattern in conditionals: replaces complex empty checks
pub fn is_defined(val: &TagValue) -> bool {
    match val {
        TagValue::Empty => false,
        TagValue::String(s) => !s.is_empty(),
        _ => true,
    }
}

/// Return value if defined, otherwise return default
///
/// Perl-like pattern: `$val || "default"`
pub fn default_if_empty(val: &TagValue, default: &TagValue) -> TagValue {
    if is_defined(val) {
        val.clone()
    } else {
        default.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_string() {
        assert_eq!(tag_string("hello"), TagValue::String("hello".to_string()));
        assert_eq!(
            tag_string(String::from("world")),
            TagValue::String("world".to_string())
        );
    }

    #[test]
    fn test_stringify() {
        assert_eq!(stringify(&TagValue::I32(42)), "42");
        assert_eq!(stringify(&TagValue::String("test".to_string())), "test");
        assert_eq!(stringify(&TagValue::Empty), "");
    }

    #[test]
    fn test_concat() {
        let left = TagValue::String("Hello".to_string());
        let right = TagValue::String(" World".to_string());
        assert_eq!(
            concat(&left, &right),
            TagValue::String("Hello World".to_string())
        );
    }

    #[test]
    fn test_repeat_string() {
        let val = TagValue::String("X".to_string());
        let count = TagValue::I32(3);
        assert_eq!(
            repeat_string(&val, &count),
            TagValue::String("XXX".to_string())
        );
    }

    #[test]
    fn test_is_defined() {
        assert!(!is_defined(&TagValue::Empty));
        assert!(!is_defined(&TagValue::String("".to_string())));
        assert!(is_defined(&TagValue::String("test".to_string())));
        assert!(is_defined(&TagValue::I32(0)));
    }

    #[test]
    fn test_default_if_empty() {
        let empty = TagValue::Empty;
        let default = TagValue::String("default".to_string());
        assert_eq!(default_if_empty(&empty, &default), default);

        let value = TagValue::String("value".to_string());
        assert_eq!(default_if_empty(&value, &default), value);
    }
}
