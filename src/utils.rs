//! Utility functions used across the codebase

/// Ensure a tag name has the specified group prefix, avoiding double nesting
///
/// This function defensively handles cases where a tag name may already have a group prefix,
/// preventing the common bug of creating double-nested group names like "MakerNotes:MakerNotes:TagName".
///
/// # Examples
/// ```
/// use exif_oxide::utils::ensure_group_prefix;
///
/// assert_eq!(ensure_group_prefix("TagName", "MakerNotes"), "MakerNotes:TagName");
/// assert_eq!(ensure_group_prefix("MakerNotes:TagName", "MakerNotes"), "MakerNotes:TagName");
/// ```
///
/// # Panics
/// Panics if the tag already has a different group prefix, indicating a logic error
pub fn ensure_group_prefix(tag_name: &str, expected_group: &str) -> String {
    if let Some((existing_group, _base_name)) = tag_name.split_once(':') {
        // Tag already has a group prefix
        if existing_group == expected_group {
            // Same group - return as-is
            tag_name.to_string()
        } else {
            // Different group - this indicates a logic error
            panic!(
                "Tag '{}' already has group '{}' but expected group '{}'. This indicates a double-grouping logic error.",
                tag_name, existing_group, expected_group
            );
        }
    } else {
        // No group prefix - add the expected one
        format!("{}:{}", expected_group, tag_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_group_prefix_bare_tag() {
        assert_eq!(
            ensure_group_prefix("TagName", "MakerNotes"),
            "MakerNotes:TagName"
        );
        assert_eq!(ensure_group_prefix("FNumber", "EXIF"), "EXIF:FNumber");
    }

    #[test]
    fn test_ensure_group_prefix_same_group() {
        assert_eq!(
            ensure_group_prefix("MakerNotes:TagName", "MakerNotes"),
            "MakerNotes:TagName"
        );
        assert_eq!(ensure_group_prefix("EXIF:FNumber", "EXIF"), "EXIF:FNumber");
    }

    #[test]
    #[should_panic(expected = "double-grouping logic error")]
    fn test_ensure_group_prefix_different_group() {
        ensure_group_prefix("EXIF:TagName", "MakerNotes");
    }
}
