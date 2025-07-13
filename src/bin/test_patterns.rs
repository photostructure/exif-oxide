use exif_oxide::generated::file_types::magic_number_patterns::matches_magic_number;
use std::fs::File;
use std::io::Read;

fn main() {
    // Test with real PNG file
    let mut file = File::open("/mnt/2tb/home/mrm/src/exif-oxide/third-party/exiftool/t/images/PNG.png").unwrap();
    let mut buffer = vec![0; 1024];
    let bytes_read = file.read(&mut buffer).unwrap();
    buffer.truncate(bytes_read);
    
    println!("First 16 bytes: {:02x?}", &buffer[..16]);
    
    // Test PNG pattern
    let png_match = matches_magic_number("PNG", &buffer);
    println!("PNG pattern matches: {}", png_match);
    
    // Test other patterns
    println!("JPEG pattern matches: {}", matches_magic_number("JPEG", &buffer));
    println!("GIF pattern matches: {}", matches_magic_number("GIF", &buffer));
    
    // Test with synthetic PNG data
    let png_data = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
    println!("\nSynthetic PNG data: {:02x?}", png_data);
    println!("Synthetic PNG matches: {}", matches_magic_number("PNG", &png_data));
}