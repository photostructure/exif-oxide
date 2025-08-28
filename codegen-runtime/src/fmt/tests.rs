#[cfg(test)]
mod sprintf_split_tests {
    use crate::fmt::sprintf_split_values;
    use crate::TagValue;

    #[test]
    fn test_sprintf_split_format() {
        // Test with two float values
        let values = vec![
            TagValue::String("1.234".to_string()),
            TagValue::String("5.678".to_string()),
        ];
        let result = sprintf_split_values("%.3f x %.3f mm", &values);
        assert_eq!(result, "1.234 x 5.678 mm");
    }

    #[test]
    fn test_sprintf_split_single_value_duplicates() {
        // Test with single value that needs to be duplicated
        let values = vec![TagValue::String("2.5".to_string())];
        let result = sprintf_split_values("%.3f x %.3f mm", &values);
        assert_eq!(result, "2.5 x 2.5 mm");
    }

    #[test]
    fn test_sprintf_split_float_values() {
        // Test with actual float values
        let values = vec![TagValue::F64(3.14159), TagValue::F64(2.71828)];
        let result = sprintf_split_values("%.3f x %.3f mm", &values);
        assert_eq!(result, "3.142 x 2.718 mm");
    }
}
