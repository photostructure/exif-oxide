use exif_oxide::generated::file_types::magic_number_patterns::get_magic_number_pattern;
use regex::bytes::Regex;

fn main() {
    // Check if PNG pattern exists
    if let Some(pattern) = get_magic_number_pattern("PNG") {
        println!("PNG pattern found!");
        
        // Test the pattern directly
        let png_data = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
        println!("Pattern matches synthetic data: {}", pattern.is_match(&png_data));
        
        // Try to recreate the pattern manually
        let manual_pattern = Regex::new(r"^(\x89P|\x8aM|\x8bJ)NG\r\n\x1a\n").unwrap();
        println!("Manual pattern matches: {}", manual_pattern.is_match(&png_data));
        
        // Check raw bytes
        println!("First 4 bytes of data: {:02x?}", &png_data[..4]);
        println!("Pattern should match \\x89P (0x89 0x50)");
        
    } else {
        println!("PNG pattern not found!");
        
        // List all available patterns
        println!("\nChecking available patterns...");
        for key in ["JPEG", "GIF", "BMP", "TIFF", "PNG"] {
            if get_magic_number_pattern(key).is_some() {
                println!("  {} pattern exists", key);
            } else {
                println!("  {} pattern NOT FOUND", key);
            }
        }
    }
}