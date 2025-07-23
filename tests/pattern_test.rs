#[test]
fn test_png_pattern_directly() {
    use std::fs;
    use std::path::Path;

    // Test with actual PNG file header
    let png_path = Path::new("third-party/exiftool/t/images/PNG.png");
    if png_path.exists() {
        let png_data = fs::read(png_path).expect("Failed to read PNG file");

        // Just need the first 8 bytes for the PNG signature
        let header = &png_data[..8.min(png_data.len())];

        println!("PNG header bytes: {header:?}");
        println!(
            "PNG header hex: {}",
            header
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<Vec<_>>()
                .join(" ")
        );

        // TODO: Re-enable when magic_number_patterns is generated
        // use exif_oxide::generated::file_types::magic_number_patterns::matches_magic_number;
        // assert!(
        //     matches_magic_number("PNG", header),
        //     "Generated PNG pattern should match real PNG file"
        // );
    } else {
        // Fallback: test with known PNG signature
        let _png_signature = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

        // TODO: Re-enable when magic_number_patterns is generated
        // use exif_oxide::generated::file_types::magic_number_patterns::matches_magic_number;
        // assert!(
        //     matches_magic_number("PNG", &png_signature),
        //     "Generated PNG pattern should match PNG signature"
        // );
    }
}
