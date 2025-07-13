#[test]
fn test_png_pattern_directly() {
    use regex::bytes::Regex;
    
    let png_pattern = r"^(\x89P|\x8aM|\x8bJ)NG\r\n\x1a\n";
    let regex = Regex::new(png_pattern).expect("Failed to compile PNG pattern");
    
    let png_data = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
    
    println!("Pattern: {}", png_pattern);
    println!("Data: {:?}", png_data);
    
    // Test pattern match
    assert!(regex.is_match(&png_data), "PNG pattern should match PNG data");
    
    // Also test using the generated function
    use exif_oxide::generated::file_types::magic_number_patterns::matches_magic_number;
    assert!(matches_magic_number("PNG", &png_data), "Generated PNG pattern should match");
}