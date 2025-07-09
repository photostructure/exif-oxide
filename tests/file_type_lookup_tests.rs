//! Integration tests for file type lookup functionality

use exif_oxide::generated::file_types::{
    extensions_for_format, get_primary_format, resolve_file_type, supports_format,
};

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

#[test]
fn test_extensions_for_format() {
    let pdf_extensions = extensions_for_format("PDF");
    assert!(pdf_extensions.contains(&"PDF".to_string()));
    assert!(pdf_extensions.contains(&"AI".to_string())); // AI supports PDF

    let jpeg_extensions = extensions_for_format("JPEG");
    assert!(jpeg_extensions.contains(&"JPEG".to_string()));
}
