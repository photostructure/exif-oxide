//! Tests for file type lookup functionality
//!
//! This file tests the generated file type lookup infrastructure
//! to ensure it matches ExifTool behavior.

#[cfg(test)]
mod tests {
    use super::super::file_type_lookup::*;

    #[test]
    fn test_alias_resolution() {
        // Test: 3GP2 -> 3G2 -> MOV
        let result = resolve_file_type("3GP2");
        assert!(result.is_some());
        let (formats, desc) = result.unwrap();
        assert_eq!(formats[0], "MOV");
        assert!(desc.contains("3rd Gen"));
    }

    #[test]
    fn test_direct_definition() {
        let result = resolve_file_type("AIFF");
        assert!(result.is_some());
        let (formats, desc) = result.unwrap();
        assert_eq!(formats[0], "AIFF");
        assert_eq!(desc, "Audio Interchange File Format");
    }

    #[test]
    fn test_multiple_formats() {
        let result = resolve_file_type("AI");
        assert!(result.is_some());
        let (formats, _) = result.unwrap();
        assert_eq!(formats, vec!["PDF", "PS"]);
    }

    #[test]
    fn test_unknown_extension() {
        assert!(resolve_file_type("UNKNOWN").is_none());
    }

    #[test]
    fn test_case_insensitivity() {
        // Test that lookup is case-insensitive
        let upper = resolve_file_type("JPEG");
        let lower = resolve_file_type("jpeg");
        assert_eq!(upper, lower);
        assert!(upper.is_some());
    }

    #[test]
    fn test_get_primary_format() {
        assert_eq!(get_primary_format("JPEG"), Some("JPEG".to_string()));
        assert_eq!(get_primary_format("AI"), Some("PDF".to_string())); // First in multiple formats
        assert_eq!(get_primary_format("UNKNOWN"), None);
    }

    #[test]
    fn test_supports_format() {
        assert!(supports_format("JPEG", "JPEG"));
        assert!(supports_format("AI", "PDF"));
        assert!(supports_format("AI", "PS"));
        assert!(!supports_format("JPEG", "PNG"));
        assert!(!supports_format("UNKNOWN", "JPEG"));
    }

    #[test]
    fn test_extensions_for_format() {
        let jpeg_extensions = extensions_for_format("JPEG");
        assert!(jpeg_extensions.contains(&"JPEG".to_string()));
        // JPG is an alias to JPEG, so shouldn't appear in format search
        assert!(!jpeg_extensions.contains(&"JPG".to_string()));

        let pdf_extensions = extensions_for_format("PDF");
        assert!(pdf_extensions.contains(&"PDF".to_string()));
        assert!(pdf_extensions.contains(&"AI".to_string())); // AI supports PDF
    }

    #[test]
    fn test_circular_alias_protection() {
        // This would need to be set up if we had circular references
        // For now, verify reasonable extensions work within depth limit
        let result = resolve_file_type("3GP2"); // 3GP2 -> 3G2 -> Definition
        assert!(result.is_some());
    }

    #[test]
    fn test_office_documents() {
        // Test complex office document formats
        let docx = resolve_file_type("DOCX");
        assert!(docx.is_some());
        let (formats, _) = docx.unwrap();
        assert!(formats.contains(&"ZIP".to_string()));
        assert!(formats.contains(&"FPX".to_string()));
    }

    #[test]
    fn test_raw_formats() {
        // Test camera RAW formats
        assert_eq!(get_primary_format("CR2"), Some("TIFF".to_string()));
        assert_eq!(get_primary_format("NEF"), Some("TIFF".to_string()));
        assert_eq!(get_primary_format("ARW"), Some("TIFF".to_string()));
    }

    #[test]
    fn test_video_formats() {
        // Test video formats
        assert_eq!(get_primary_format("MP4"), Some("MOV".to_string()));
        assert_eq!(get_primary_format("AVI"), Some("RIFF".to_string()));
        assert_eq!(get_primary_format("MKV"), Some("MKV".to_string()));
    }

    #[test]
    fn test_archive_formats() {
        // Test archive formats
        assert_eq!(get_primary_format("ZIP"), Some("ZIP".to_string()));
        assert_eq!(get_primary_format("7Z"), Some("7Z".to_string()));
        assert_eq!(get_primary_format("TAR"), Some("TAR".to_string()));
    }

    #[test]
    fn test_image_formats() {
        // Test common image formats
        assert_eq!(get_primary_format("PNG"), Some("PNG".to_string()));
        assert_eq!(get_primary_format("GIF"), Some("GIF".to_string()));
        assert_eq!(get_primary_format("BMP"), Some("BMP".to_string()));
        assert_eq!(get_primary_format("TIFF"), Some("TIFF".to_string()));
    }

    #[test]
    fn test_adobe_formats() {
        // Test Adobe formats with multiple support
        let ai = resolve_file_type("AI");
        assert!(ai.is_some());
        let (formats, desc) = ai.unwrap();
        assert!(formats.contains(&"PDF".to_string()));
        assert!(formats.contains(&"PS".to_string()));
        assert_eq!(desc, "Adobe Illustrator");
    }

    #[test]
    fn test_total_entry_count() {
        // Verify we have all 343 entries as mentioned in the milestone
        // This is a sanity check to ensure extraction worked completely
        let total_entries: usize = FILE_TYPE_LOOKUP.len();

        // Should have 343 entries as specified in the milestone
        assert_eq!(
            total_entries, 343,
            "Expected 343 file type entries from ExifTool"
        );
    }

    #[test]
    fn test_essential_extensions() {
        // Test extensions that should always be present from ExifTool
        let essential_extensions = [
            "JPEG", "PNG", "TIFF", "GIF", "BMP", // Images
            "PDF", "ZIP", "TAR", // Documents/Archives
            "MP4", "AVI", "MOV", // Video
            "MP3", "WAV", "FLAC", // Audio
            "CR2", "NEF", "ARW", // Camera RAW
        ];

        for ext in essential_extensions {
            let result = resolve_file_type(ext);
            assert!(
                result.is_some(),
                "Essential extension {ext} should be supported"
            );
        }
    }
}
